#![cfg(not(target_arch = "wasm32"))]
//! Optional provider absence, scoping, errors and request-local identity.
use axum::{
    Router,
    body::{Body, to_bytes},
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tower::ServiceExt;
use yetlet::request_id::{HEADER, InvalidStart, RequestId, RequestIds};
use yetlet::server_axum::{
    App, RequestContext,
    extract::Extension,
    http::{Request, StatusCode},
    middleware::{Next, from_fn},
    response::{IntoResponse, Response},
};

async fn label(id: Option<Extension<RequestId>>) -> String {
    id.map_or_else(|| "absent".to_owned(), |Extension(id)| id.0.to_string())
}
async fn call(app: Router, path: &str) -> Response {
    app.oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}
async fn body(response: Response) -> String {
    String::from_utf8(to_bytes(response.into_body(), 1024).await.unwrap().to_vec()).unwrap()
}

#[tokio::test]
async fn dependency_does_not_implicitly_activate_provider() {
    let app: Router = App::new().get("/", label).into_router();
    let response = call(app, "/").await;
    assert!(!response.headers().contains_key(HEADER));
    assert_eq!(body(response).await, "absent");
}

#[tokio::test]
async fn scoped_installation_never_leaks_to_siblings_or_unmatched_routes() {
    let ids = RequestIds::new();
    let app: Router = App::new()
        .get("/plain", label)
        .group("/selected", |g| {
            g.get("/item", label).configure(|g| ids.scoped(g))
        })
        .into_router();
    let first = call(app.clone(), "/selected/item").await;
    assert_eq!(first.headers()[HEADER], "1");
    assert_eq!(body(first).await, "1");
    let second = call(app.clone(), "/selected/item").await;
    assert_eq!(second.headers()[HEADER], "2");
    for path in ["/plain", "/selected/missing", "/missing"] {
        let response = call(app.clone(), path).await;
        assert!(!response.headers().contains_key(HEADER));
    }
}

#[tokio::test]
async fn global_identity_is_server_owned_and_wraps_404_and_405() {
    let app = RequestIds::new().global(App::new().get("/", label));
    let mut req = Request::builder()
        .uri("/")
        .header(HEADER, "untrusted")
        .body(Body::empty())
        .unwrap();
    req.extensions_mut().insert(RequestId(999));
    let response = app.clone().oneshot(req).await.unwrap();
    assert_eq!(response.headers()[HEADER], "1");
    assert_eq!(body(response).await, "1");
    let missing = call(app.clone(), "/missing").await;
    assert_eq!(missing.status(), StatusCode::NOT_FOUND);
    assert_eq!(missing.headers()[HEADER], "2");
    let method = app
        .oneshot(
            Request::builder()
                .uri("/")
                .method("POST")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(method.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(method.headers()[HEADER], "3");
    assert!(method.headers()["allow"].to_str().unwrap().contains("GET"));
}

async fn deny(_request: RequestContext, _next: Next) -> Response {
    (StatusCode::FORBIDDEN, "denied").into_response()
}

#[tokio::test]
async fn outer_optional_layer_unwinds_around_a_short_circuit_without_running_handler() {
    let calls = Arc::new(AtomicUsize::new(0));
    let handler_calls = calls.clone();
    let app = RequestIds::new().global(
        App::new()
            .get("/", move || {
                let calls = handler_calls.clone();
                async move {
                    calls.fetch_add(1, Ordering::Relaxed);
                    "should not run"
                }
            })
            .route_layer(from_fn(deny)),
    );
    let response = call(app, "/").await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    assert_eq!(response.headers()[HEADER], "1");
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}

#[test]
fn typed_setup_failure_propagates_through_try_configure() {
    let result = App::<()>::new()
        .get("/", label)
        .try_configure(|app| RequestIds::try_new(0).map(|provider| provider.scoped(app)));
    assert!(matches!(result, Err(InvalidStart)));
    assert!(RequestIds::try_new(u64::MAX).is_err());
}

#[tokio::test]
async fn exhaustion_fails_closed_instead_of_wrapping_identity() {
    let app = RequestIds::try_new(u64::MAX - 1)
        .unwrap()
        .global(App::new().get("/", label));
    let first = call(app.clone(), "/").await;
    assert_eq!(
        first.headers()[HEADER].to_str().unwrap(),
        (u64::MAX - 1).to_string()
    );
    let exhausted = call(app, "/").await;
    assert_eq!(exhausted.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert!(!exhausted.headers().contains_key(HEADER));
    assert_eq!(body(exhausted).await, "Request ID capacity exhausted");
}

#[tokio::test]
async fn concurrent_requests_have_individual_typed_values_and_shared_counter() {
    let app = RequestIds::new().global(App::new().get("/", label));
    let mut tasks = Vec::new();
    for _ in 0..12 {
        let app = app.clone();
        tasks.push(tokio::spawn(async move {
            let response = call(app, "/").await;
            let id = response.headers()[HEADER].to_str().unwrap().to_owned();
            assert_eq!(body(response).await, id);
            id
        }));
    }
    let mut seen = std::collections::HashSet::new();
    for task in tasks {
        assert!(seen.insert(task.await.unwrap()));
    }
    assert_eq!(seen.len(), 12);
}
