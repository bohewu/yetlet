//! M6b: ordinary functions, shared Views and native typed HTTP contracts.
use axum::{
    Router,
    body::{Body, to_bytes},
};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use yetlet_core::{Element, View};
use yetlet_server_axum::{
    App, DEFAULT_CSP, Page,
    extract::{Json, Path, Query, State},
    http::{Request, StatusCode, header},
    response::{Redirect, Response},
};

async fn send(app: Router, path: &str) -> Response {
    app.oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}
async fn text(response: Response) -> String {
    String::from_utf8(
        to_bytes(response.into_body(), 16384)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

async fn home() -> View {
    View::text("<script>alert(1)</script>")
}

#[tokio::test]
async fn view_handler_has_escaped_title_content_csp_and_zero_runtime() {
    let app: Router = App::new().page("/", "<Title> & name", home).into_router();
    let response = send(app.clone(), "/").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        DEFAULT_CSP
    );
    assert!(!response.headers().contains_key("x-yetlet-request-id"));
    assert!(!response.headers().contains_key(header::SET_COOKIE));
    let html = text(response).await;
    assert!(html.contains("<title>&lt;Title&gt; &amp; name</title>"));
    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!html.contains("<script") && !html.contains(".wasm"));
    let head = app
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(head.status(), StatusCode::OK);
    assert_eq!(head.headers()[header::CONTENT_SECURITY_POLICY], DEFAULT_CSP);
    assert!(text(head).await.is_empty());
}

#[tokio::test]
async fn render_failures_keep_generic_500_and_csp() {
    let app: Router = App::new()
        .page("/", "Error", || async {
            Element::new("script")
                .child(View::text("private detail"))
                .into()
        })
        .into_router();
    let response = send(app, "/").await;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        response.headers()[header::CONTENT_SECURITY_POLICY],
        DEFAULT_CSP
    );
    assert_eq!(text(response).await, "Unable to render the page");
}

#[derive(Clone)]
struct Settings(&'static str);
#[derive(Deserialize)]
struct Parameters {
    name: String,
}
async fn detail(
    State(settings): State<Settings>,
    Path(id): Path<u64>,
    Query(query): Query<Parameters>,
) -> Page {
    Page::new(settings.0, View::text(format!("{id}:{}", query.name)))
}

#[tokio::test]
async fn typed_state_path_query_bind_to_page_without_requestcontext() {
    let app: Router = App::<Settings>::new()
        .get("/items/{id}", detail)
        .with_state(Settings("Details"))
        .into_router();
    let response = send(app.clone(), "/items/7?name=%3Cname%3E").await;
    assert_eq!(response.status(), StatusCode::OK);
    assert!(text(response).await.contains("7:&lt;name&gt;"));
    assert_eq!(
        send(app.clone(), "/items/not-a-number?name=x")
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        send(app, "/items/7").await.status(),
        StatusCode::BAD_REQUEST
    );
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Payload {
    value: u64,
}
async fn create(Json(payload): Json<Payload>) -> (StatusCode, Json<Payload>) {
    (StatusCode::CREATED, Json(payload))
}

#[tokio::test]
async fn native_json_body_and_status_preserve_extractor_rejections() {
    let app: Router = App::new().post("/items", create).into_router();
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"value":7}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
    assert_eq!(text(response).await, r#"{"value":7}"#);
    let malformed = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(malformed.status(), StatusCode::BAD_REQUEST);
    let missing_type = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/items")
                .body(Body::from(r#"{"value":7}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(missing_type.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
}

async fn restricted() -> Result<Page, (StatusCode, &'static str)> {
    Err((StatusCode::FORBIDDEN, "access denied"))
}

#[tokio::test]
async fn fallible_handlers_redirects_and_explicit_headers_are_native_results() {
    let app: Router = App::new()
        .get("/private", restricted)
        .get("/redirect", || async { Redirect::to("/private") })
        .get("/custom", || async {
            (StatusCode::ACCEPTED, [("x-explicit", "yes")], Json(7))
        })
        .into_router();
    let error = send(app.clone(), "/private").await;
    assert_eq!(error.status(), StatusCode::FORBIDDEN);
    assert_eq!(text(error).await, "access denied");
    let redirect = send(app.clone(), "/redirect").await;
    assert_eq!(redirect.status(), StatusCode::SEE_OTHER);
    assert_eq!(redirect.headers()[header::LOCATION], "/private");
    let custom = send(app, "/custom").await;
    assert_eq!(custom.status(), StatusCode::ACCEPTED);
    assert_eq!(custom.headers()["x-explicit"], "yes");
    assert_eq!(text(custom).await, "7");
}
