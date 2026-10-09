//! Integration tests for the M6a handler-first App facade.

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Path, Query, State},
    http::{Request, StatusCode, header},
    middleware::from_fn,
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;
use tower::ServiceExt;
use yetlet_core::{Element, View};
use yetlet_server_axum::{
    App, DEFAULT_CSP, HtmlPageError, RequestContext,
    middleware::{Next, with_global},
    render_page,
};

async fn homepage() -> Result<Response, HtmlPageError> {
    let content: View = Element::new("h1").child(View::text("<Yetlet>")).into();
    render_page("Example", &content)
}

async fn status() -> Json<&'static str> {
    Json("ready")
}

async fn body_of(response: Response) -> String {
    String::from_utf8(
        to_bytes(response.into_body(), 1 << 18)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap()
}

async fn request_get(router: Router, path: &str) -> Response {
    router
        .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn handlers_render_html_and_json_without_controllers() {
    let app = App::new().get("/", homepage).get("/api/status", status);
    let native: Router = app.into_router();
    let page = request_get(native.clone(), "/").await;
    assert_eq!(page.status(), StatusCode::OK);
    assert_eq!(page.headers()[header::CONTENT_SECURITY_POLICY], DEFAULT_CSP);
    assert!(!page.headers().contains_key("x-yetlet-request-id"));
    assert!(body_of(page).await.contains("<h1>&lt;Yetlet&gt;</h1>"));

    let json = request_get(native.clone(), "/api/status").await;
    assert_eq!(json.status(), StatusCode::OK);
    assert_eq!(json.headers()[header::CONTENT_TYPE], "application/json");
    assert_eq!(body_of(json).await, "\"ready\"");

    assert_eq!(
        request_get(native, "/missing").await.status(),
        StatusCode::NOT_FOUND
    );
}

#[derive(Deserialize)]
struct Filter {
    limit: usize,
}

async fn read_item(Path(id): Path<u32>, Query(filter): Query<Filter>) -> String {
    format!("{id}:{}", filter.limit)
}

async fn created() -> StatusCode {
    StatusCode::CREATED
}

#[tokio::test]
async fn groups_preserve_path_query_method_and_404_405() {
    let router = App::new()
        .group("/api", |api| {
            api.group("/v1", |v1| {
                v1.get("/items/{id}", read_item).post("/items", created)
            })
        })
        .into_router();

    let found = request_get(router.clone(), "/api/v1/items/7?limit=5").await;
    assert_eq!(body_of(found).await, "7:5");
    assert_eq!(
        request_get(router.clone(), "/api/items/7?limit=5")
            .await
            .status(),
        StatusCode::NOT_FOUND
    );

    let method = router
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/v1/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(method.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert!(
        method.headers()[header::ALLOW]
            .to_str()
            .unwrap()
            .contains("POST")
    );

    let created_response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/items")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(created_response.status(), StatusCode::CREATED);
}

async fn outer(mut context: RequestContext, next: Next) -> Response {
    context.extensions_mut().insert("global marker");
    let mut response = next.run(context.into_request()).await;
    response
        .headers_mut()
        .insert("x-outer", "yes".parse().unwrap());
    response
}

async fn scoped(context: RequestContext, next: Next) -> Response {
    if context.headers().contains_key("x-deny") {
        return (StatusCode::FORBIDDEN, "blocked").into_response();
    }
    let mut response = next.run(context.into_request()).await;
    response
        .headers_mut()
        .insert("x-scoped", "yes".parse().unwrap());
    response
}

async fn scoped_handler(context: RequestContext) -> &'static str {
    context
        .extensions()
        .get::<&'static str>()
        .copied()
        .unwrap_or("missing")
}

#[tokio::test]
async fn scoped_middleware_short_circuits_and_global_unwinds() {
    let router = App::new()
        .get("/", homepage)
        .group("/api", |api| {
            api.get("/private", scoped_handler)
                .route_layer(from_fn(scoped))
        })
        .into_router();
    let router = with_global(router, from_fn(outer));

    let response = request_get(router.clone(), "/api/private").await;
    assert_eq!(response.headers()["x-outer"], "yes");
    assert_eq!(response.headers()["x-scoped"], "yes");
    assert_eq!(body_of(response).await, "global marker");
    // Native Router::route_layer retains method-mismatch and Allow semantics.
    let method = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/private")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(method.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(method.headers()["x-outer"], "yes");
    assert_eq!(method.headers()["x-scoped"], "yes");
    assert!(
        method.headers()[header::ALLOW]
            .to_str()
            .unwrap()
            .contains("GET")
    );

    let blocked = router
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/private")
                .header("x-deny", "1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(blocked.status(), StatusCode::FORBIDDEN);
    assert_eq!(blocked.headers()["x-outer"], "yes");
    assert!(!blocked.headers().contains_key("x-scoped"));
    assert_eq!(body_of(blocked).await, "blocked");

    let home = request_get(router.clone(), "/").await;
    assert_eq!(home.headers()["x-outer"], "yes");
    assert!(!home.headers().contains_key("x-scoped"));

    let fallback = request_get(router, "/not-found").await;
    assert_eq!(fallback.status(), StatusCode::NOT_FOUND);
    assert_eq!(fallback.headers()["x-outer"], "yes");
    assert!(!fallback.headers().contains_key("x-scoped"));
}

#[derive(Clone)]
struct Counter {
    value: u32,
}

async fn read_state(State(counter): State<Counter>) -> Json<u32> {
    Json(counter.value)
}

#[tokio::test]
async fn axum_state_and_native_router_extensions_are_unchanged() {
    let app: App<Counter> = App::new().get("/state", read_state);
    let native: Router = app
        .with_state(Counter { value: 7 })
        .map_router(|router| {
            router
                .route("/native", get(|| async { Redirect::temporary("/state") }))
                .fallback(|| async { (StatusCode::IM_A_TEAPOT, "fallback") })
        })
        .into_router();

    assert_eq!(
        body_of(request_get(native.clone(), "/state").await).await,
        "7"
    );
    let redirect = request_get(native.clone(), "/native").await;
    assert_eq!(redirect.status(), StatusCode::TEMPORARY_REDIRECT);
    assert_eq!(redirect.headers()[header::LOCATION], "/state");
    let fallback = request_get(native, "/missing").await;
    assert_eq!(fallback.status(), StatusCode::IM_A_TEAPOT);
    assert_eq!(body_of(fallback).await, "fallback");
}

#[tokio::test]
async fn accepts_native_method_router_and_existing_axum_router() {
    let native = Router::new().route("/axum", get(|| async { "native" }));
    let router = App::from_router(native)
        .route("/both", get(|| async { "GET" }).post(|| async { "POST" }))
        .into_router();
    assert_eq!(
        body_of(request_get(router.clone(), "/axum").await).await,
        "native"
    );
    assert_eq!(
        body_of(request_get(router.clone(), "/both").await).await,
        "GET"
    );
    let post = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/both")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(body_of(post).await, "POST");
}
