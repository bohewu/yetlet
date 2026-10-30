#![cfg(not(target_arch = "wasm32"))]
//! Synthetic verifier fixtures test the foundation, not a credential protocol.
use std::{
    convert::Infallible,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

use axum::{
    Router,
    body::{Body, Bytes, to_bytes},
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header, request::Parts},
    middleware::{Next, from_fn, from_fn_with_state},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use http_body::{Body as HttpBody, Frame};
use tower::{ServiceBuilder, ServiceExt};
use yetlet::auth::{
    Authenticated, Authentication, MAX_AUTHENTICATION_NAME_BYTES, MAX_ISSUER_BYTES,
    MAX_SUBJECT_BYTES, MAX_TENANT_BYTES, Principal, ResolveFuture, authenticate,
};
use yetlet::server_axum::{App, DEFAULT_CSP, Page, RequestContext};

#[derive(Clone)]
struct Fixture {
    scheme: &'static str,
    calls: Arc<AtomicUsize>,
}
impl Fixture {
    fn new(scheme: &'static str) -> Self {
        Self {
            scheme,
            calls: Arc::new(AtomicUsize::new(0)),
        }
    }
}
fn challenge() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        [(header::WWW_AUTHENTICATE, "Test realm=\"fixture\"")],
        "Authentication required",
    )
        .into_response()
}
fn resolve<'a>(parts: &'a mut Parts, state: &'a Fixture) -> ResolveFuture<'a> {
    Box::pin(async move {
        state.calls.fetch_add(1, Ordering::SeqCst);
        let proof = parts
            .headers
            .get("x-fixture-proof")
            .and_then(|v| v.to_str().ok());
        let (subject, tenant, scheme) = match proof {
            None => return Ok(None),
            Some("invalid") => return Err(challenge()),
            Some("unavailable") => {
                return Err((
                    StatusCode::SERVICE_UNAVAILABLE,
                    "Authentication unavailable",
                )
                    .into_response());
            }
            Some("malformed") => return Err(StatusCode::BAD_REQUEST.into_response()),
            Some("wrong-scheme") => ("writer", Some("one"), "other"),
            Some("wrong-tenant") => ("writer", Some("two"), state.scheme),
            Some("no-tenant") => ("writer", None, state.scheme),
            Some("reader") => ("reader", Some("one"), state.scheme),
            Some("unmapped") => ("unmapped", Some("one"), state.scheme),
            Some("valid") => ("writer", Some("one"), state.scheme),
            _ => return Err(challenge()),
        };
        Ok(Some(
            Principal::from_verified("urn:test:accounts", subject, scheme, tenant).unwrap(),
        ))
    })
}
fn configuration(fixture: Fixture) -> Authentication<Fixture> {
    Authentication::new(fixture.scheme, fixture, resolve, challenge).unwrap()
}
async fn required(identity: Authenticated) -> String {
    identity.principal().authentication_name().to_owned()
}
async fn optional(identity: Option<Authenticated>) -> &'static str {
    if identity.is_some() {
        "authenticated"
    } else {
        "anonymous"
    }
}
fn routes(fixture: Fixture) -> Router {
    Router::new()
        .route("/required", get(required))
        .route("/optional", get(optional))
        .route_layer(from_fn_with_state(
            configuration(fixture),
            authenticate::<Fixture>,
        ))
}
fn request(path: &str, proof: Option<&str>) -> Request {
    let mut builder = Request::builder().uri(path);
    if let Some(proof) = proof {
        builder = builder.header("x-fixture-proof", proof);
    }
    builder.body(Body::empty()).unwrap()
}
async fn text(response: Response) -> String {
    String::from_utf8(to_bytes(response.into_body(), 4096).await.unwrap().to_vec()).unwrap()
}
fn private(response: &Response) {
    assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
}

#[test]
fn principal_preserves_exact_identity_and_separates_context() {
    let a =
        Principal::from_verified("https://id.example/", "Subject", "first", Some("one")).unwrap();
    let b =
        Principal::from_verified("https://id.example/", "Subject", "second", Some("two")).unwrap();
    assert!(a.same_identity(&b));
    assert_eq!(a.issuer(), "https://id.example/");
    assert_eq!(a.subject(), "Subject");
    assert_eq!(a.authentication_name(), "first");
    assert_eq!(a.tenant(), Some("one"));
    for (issuer, subject) in [
        ("https://id.example", "Subject"),
        ("https://ID.example/", "Subject"),
        ("https://id.example/", "subject"),
    ] {
        assert!(
            !a.same_identity(&Principal::from_verified(issuer, subject, "first", None).unwrap())
        );
    }
    assert!(!format!("{a:?}").contains("Subject"));
    assert!(!format!("{a:?}").contains("id.example"));
    assert!(!format!("{a:?}").contains("one"));
}

#[test]
fn principal_and_configuration_reject_invalid_shapes_without_values() {
    for field in 0..4 {
        let limit = [
            MAX_ISSUER_BYTES,
            MAX_SUBJECT_BYTES,
            MAX_AUTHENTICATION_NAME_BYTES,
            MAX_TENANT_BYTES,
        ][field];
        let exact = "a".repeat(limit);
        let mut values = ["issuer", "subject", "scheme", "tenant"];
        values[field] = &exact;
        assert!(Principal::from_verified(values[0], values[1], values[2], Some(values[3])).is_ok());
        for invalid in [
            String::new(),
            " ".into(),
            "a\n".into(),
            "a\0".into(),
            "a".repeat(limit + 1),
            "é".repeat(limit),
        ] {
            let mut values = ["issuer", "subject", "scheme", "tenant"];
            values[field] = &invalid;
            let error = Principal::from_verified(values[0], values[1], values[2], Some(values[3]))
                .unwrap_err();
            assert_eq!(
                error.to_string(),
                "invalid principal or authentication scheme"
            );
        }
    }
    assert!(Authentication::new("", Fixture::new("fixture"), resolve, challenge).is_err());
}

#[tokio::test]
async fn required_and_optional_keep_anonymous_distinct_from_verified() {
    let fixture = Fixture::new("fixture");
    let app = routes(fixture.clone());
    for (path, proof, status, body) in [
        (
            "/required",
            None,
            StatusCode::UNAUTHORIZED,
            "Authentication required",
        ),
        ("/optional", None, StatusCode::OK, "anonymous"),
        ("/required", Some("valid"), StatusCode::OK, "fixture"),
        ("/optional", Some("valid"), StatusCode::OK, "authenticated"),
    ] {
        let response = app.clone().oneshot(request(path, proof)).await.unwrap();
        assert_eq!(response.status(), status);
        private(&response);
        if status == StatusCode::UNAUTHORIZED {
            assert_eq!(
                response.headers()[header::WWW_AUTHENTICATE],
                "Test realm=\"fixture\""
            );
        }
        assert_eq!(text(response).await, body);
    }
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 4);
}

#[tokio::test]
async fn invalid_and_unavailable_never_downgrade_to_optional_anonymous() {
    let app = routes(Fixture::new("fixture"));
    for path in ["/required", "/optional"] {
        for (proof, status) in [
            ("invalid", StatusCode::UNAUTHORIZED),
            ("unavailable", StatusCode::SERVICE_UNAVAILABLE),
            ("malformed", StatusCode::BAD_REQUEST),
            ("wrong-scheme", StatusCode::INTERNAL_SERVER_ERROR),
        ] {
            let response = app
                .clone()
                .oneshot(request(path, Some(proof)))
                .await
                .unwrap();
            assert_eq!(response.status(), status);
            private(&response);
        }
    }
}

#[tokio::test]
async fn missing_layer_fails_closed_even_for_optional_extraction() {
    let app = Router::new()
        .route("/required", get(required))
        .route("/optional", get(optional));
    for path in ["/required", "/optional"] {
        let response = app
            .clone()
            .oneshot(request(path, Some("valid")))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        private(&response);
        assert_eq!(text(response).await, "Authentication configuration error");
    }
}

#[tokio::test]
async fn raw_principal_and_client_identity_assertions_do_not_authenticate() {
    let mut req = request("/required", None);
    req.headers_mut()
        .insert("x-user", "administrator".parse().unwrap());
    req.headers_mut().insert(
        "authorization",
        "Bearer unverified-placeholder".parse().unwrap(),
    );
    req.extensions_mut().insert(
        Principal::from_verified("urn:forged", "administrator", "fixture", Some("one")).unwrap(),
    );
    let response = routes(Fixture::new("fixture")).oneshot(req).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(!text(response).await.contains("administrator"));
}

#[tokio::test]
async fn duplicate_resolver_is_configuration_error_without_second_lookup() {
    let first = Fixture::new("fixture");
    let second = Fixture::new("fixture");
    let app = Router::new().route("/required", get(required)).route_layer(
        ServiceBuilder::new()
            .layer(from_fn_with_state(
                configuration(first.clone()),
                authenticate::<Fixture>,
            ))
            .layer(from_fn_with_state(
                configuration(second.clone()),
                authenticate::<Fixture>,
            )),
    );
    let response = app
        .oneshot(request("/required", Some("valid")))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(first.calls.load(Ordering::SeqCst), 1);
    assert_eq!(second.calls.load(Ordering::SeqCst), 0);
    private(&response);
}

#[tokio::test]
async fn sibling_schemes_and_nested_authorization_keep_route_scope() {
    async fn policy(identity: Authenticated, req: Request, next: Next) -> Response {
        if identity.principal().tenant() != Some("one") {
            return StatusCode::FORBIDDEN.into_response();
        }
        next.run(req).await
    }
    let group = |scheme| {
        Router::new()
            .nest(
                "/nested",
                Router::new()
                    .route("/required", get(required))
                    .route_layer(from_fn(policy)),
            )
            .route_layer(from_fn_with_state(
                configuration(Fixture::new(scheme)),
                authenticate::<Fixture>,
            ))
    };
    let app = Router::new()
        .nest("/a", group("first"))
        .nest("/b", group("second"))
        .route("/public", get(|| async { "public" }));
    for (path, scheme) in [
        ("/a/nested/required", "first"),
        ("/b/nested/required", "second"),
    ] {
        assert_eq!(
            text(
                app.clone()
                    .oneshot(request(path, Some("valid")))
                    .await
                    .unwrap()
            )
            .await,
            scheme
        );
        assert_eq!(
            app.clone()
                .oneshot(request(path, Some("wrong-tenant")))
                .await
                .unwrap()
                .status(),
            StatusCode::FORBIDDEN
        );
    }
    let public = app
        .oneshot(request("/public", Some("invalid")))
        .await
        .unwrap();
    assert_eq!(public.status(), StatusCode::OK);
    assert!(!public.headers().contains_key(header::CACHE_CONTROL));
}

#[tokio::test]
async fn native_not_found_and_method_not_allowed_behavior_survives() {
    let fixture = Fixture::new("fixture");
    let app = routes(fixture.clone());
    assert_eq!(
        app.clone()
            .oneshot(request("/missing", None))
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_FOUND
    );
    assert_eq!(fixture.calls.load(Ordering::SeqCst), 0);
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/required")
                .header("x-fixture-proof", "valid")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(response.headers()[header::ALLOW], "GET,HEAD");
    private(&response);
}

fn authorize(principal: &Principal, write: bool) -> Result<(), StatusCode> {
    let permission = match (principal.issuer(), principal.subject(), principal.tenant()) {
        ("urn:test:accounts", "writer", Some("one")) => Some(true),
        ("urn:test:accounts", "reader", Some("one")) => Some(false),
        _ => None,
    };
    if permission.is_some_and(|can_write| !write || can_write) {
        Ok(())
    } else {
        Err(StatusCode::FORBIDDEN)
    }
}

#[tokio::test]
async fn explicit_application_policy_denies_unmapped_and_wrong_tenant_before_mutation() {
    async fn write(
        identity: Authenticated,
        State(writes): State<Arc<AtomicUsize>>,
    ) -> Result<&'static str, StatusCode> {
        authorize(identity.principal(), true)?;
        writes.fetch_add(1, Ordering::SeqCst);
        Ok("saved")
    }
    let writes = Arc::new(AtomicUsize::new(0));
    let app = Router::new()
        .route("/write", post(write))
        .route_layer(from_fn_with_state(
            configuration(Fixture::new("fixture")),
            authenticate::<Fixture>,
        ))
        .with_state(writes.clone());
    for (proof, expected) in [
        ("reader", StatusCode::FORBIDDEN),
        ("unmapped", StatusCode::FORBIDDEN),
        ("wrong-tenant", StatusCode::FORBIDDEN),
        ("no-tenant", StatusCode::FORBIDDEN),
        ("valid", StatusCode::OK),
    ] {
        let mut req = request("/write", Some(proof));
        *req.method_mut() = axum::http::Method::POST;
        let response = app.clone().oneshot(req).await.unwrap();
        assert_eq!(response.status(), expected);
        private(&response);
    }
    assert_eq!(writes.load(Ordering::SeqCst), 1);
}

struct CountingBody {
    polls: Arc<AtomicUsize>,
    sent: bool,
}
impl HttpBody for CountingBody {
    type Data = Bytes;
    type Error = Infallible;
    fn poll_frame(
        mut self: Pin<&mut Self>,
        _: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, Infallible>>> {
        self.polls.fetch_add(1, Ordering::SeqCst);
        if self.sent {
            Poll::Ready(None)
        } else {
            self.sent = true;
            Poll::Ready(Some(Ok(Frame::data(Bytes::from_static(b"stream-body")))))
        }
    }
}

#[tokio::test]
async fn authentication_rejection_never_polls_body() {
    async fn body_handler(_: Authenticated, body: String) -> String {
        body
    }
    let app = Router::new()
        .route("/body", post(body_handler))
        .route_layer(from_fn_with_state(
            configuration(Fixture::new("fixture")),
            authenticate::<Fixture>,
        ));
    for proof in [None, Some("invalid"), Some("unavailable")] {
        let polls = Arc::new(AtomicUsize::new(0));
        let mut req = request("/body", proof);
        *req.method_mut() = axum::http::Method::POST;
        *req.body_mut() = Body::new(CountingBody {
            polls: polls.clone(),
            sent: false,
        });
        let response = app.clone().oneshot(req).await.unwrap();
        assert!(response.status().is_client_error() || response.status().is_server_error());
        assert_eq!(polls.load(Ordering::SeqCst), 0);
    }
}

#[tokio::test]
async fn successful_extraction_preserves_original_request_body_and_context() {
    async fn handler(identity: Authenticated, context: RequestContext) -> Response {
        assert_eq!(identity.principal().subject(), "writer");
        context.into_body().into_response()
    }
    let app = App::new()
        .post("/body", handler)
        .route_layer(from_fn_with_state(
            configuration(Fixture::new("fixture")),
            authenticate::<Fixture>,
        ))
        .into_router();
    let polls = Arc::new(AtomicUsize::new(0));
    let mut req = request("/body", Some("valid"));
    *req.method_mut() = axum::http::Method::POST;
    *req.body_mut() = Body::new(CountingBody {
        polls: polls.clone(),
        sent: false,
    });
    let response = app.oneshot(req).await.unwrap();
    assert_eq!(polls.load(Ordering::SeqCst), 0);
    assert_eq!(text(response).await, "stream-body");
    assert!(polls.load(Ordering::SeqCst) > 0);
}

#[tokio::test]
async fn explicit_login_redirect_and_missing_challenge_configuration() {
    for (mapper, expected) in [
        (
            (|| Redirect::to("/login").into_response()) as fn() -> Response,
            StatusCode::SEE_OTHER,
        ),
        (
            (|| StatusCode::UNAUTHORIZED.into_response()) as fn() -> Response,
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    ] {
        let auth =
            Authentication::new("fixture", Fixture::new("fixture"), resolve, mapper).unwrap();
        let response = Router::new()
            .route("/required", get(required))
            .route_layer(from_fn_with_state(auth, authenticate::<Fixture>))
            .oneshot(request("/required", None))
            .await
            .unwrap();
        assert_eq!(response.status(), expected);
        private(&response);
        if expected == StatusCode::SEE_OTHER {
            assert_eq!(response.headers()[header::LOCATION], "/login");
        }
    }
}

#[tokio::test]
async fn private_headers_preserve_cookies_vary_and_streaming_errors() {
    async fn handler() -> Response {
        let mut response = StatusCode::SERVICE_UNAVAILABLE.into_response();
        response.headers_mut().append(
            header::SET_COOKIE,
            HeaderValue::from_static("app=value; HttpOnly"),
        );
        response
            .headers_mut()
            .append(header::VARY, HeaderValue::from_static("Cookie"));
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("public"));
        response
    }
    let response = Router::new()
        .route("/", get(handler))
        .route_layer(from_fn_with_state(
            configuration(Fixture::new("fixture")),
            authenticate::<Fixture>,
        ))
        .oneshot(request("/", None))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    private(&response);
    assert_eq!(
        response.headers()[header::SET_COOKIE],
        "app=value; HttpOnly"
    );
    assert_eq!(response.headers()[header::VARY], "Cookie");
}

#[tokio::test]
async fn explicit_extraction_selects_display_data_without_serializing_principal() {
    async fn handler(identity: Authenticated) -> Page {
        Page::new(
            "Account",
            yetlet::view::View::text(if identity.principal().subject() == "writer" {
                "signed in"
            } else {
                "anonymous"
            }),
        )
    }
    let response = Router::new()
        .route("/", get(handler))
        .route_layer(from_fn_with_state(
            configuration(Fixture::new("fixture")),
            authenticate::<Fixture>,
        ))
        .oneshot(request("/", Some("valid")))
        .await
        .unwrap();
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        DEFAULT_CSP
    );
    private(&response);
    let html = text(response).await;
    for secret_context in [
        "urn:test:accounts",
        "writer",
        "fixture",
        "Principal",
        "wasm",
        "<script",
    ] {
        assert!(!html.contains(secret_context));
    }
    assert!(html.contains("signed in"));
}
