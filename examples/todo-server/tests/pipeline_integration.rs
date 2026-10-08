use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use tower::ServiceExt;

#[tokio::test]
async fn production_demo_routes_share_global_but_api_scope_is_local() {
    let app = yetlet_todo_server::app();
    let mut ids = std::collections::HashSet::new();
    for (path, api_scope, status) in [
        ("/", false, StatusCode::OK),
        ("/api/ping", true, StatusCode::OK),
        ("/api/todos", true, StatusCode::OK),
        ("/search", false, StatusCode::OK),
        ("/api/unknown", false, StatusCode::NOT_FOUND),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(path)
                    .header("cookie", "test-only=must-not-render")
                    .header("x-private", "do-not-copy-this-header")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(response.headers().contains_key("x-yetlet-scope"), api_scope);
        let id = response.headers()["x-yetlet-request-id"]
            .to_str()
            .unwrap()
            .to_owned();
        assert!(ids.insert(id.clone()));
        if path == "/" {
            let body = to_bytes(response.into_body(), 32 * 1024).await.unwrap();
            let html = std::str::from_utf8(&body).unwrap();
            assert!(html.contains(&format!("data-request-id=\"{id}\"")));
            assert!(!html.contains("must-not-render"));
            assert!(!html.contains("do-not-copy-this-header"));
            assert!(!html.contains("<script"));
        }
    }
}

#[tokio::test]
async fn header_guard_short_circuit_and_method_errors_keep_outer_headers() {
    let app = yetlet_todo_server::app();
    for (method, status) in [
        ("POST", StatusCode::UNSUPPORTED_MEDIA_TYPE),
        ("DELETE", StatusCode::METHOD_NOT_ALLOWED),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/todos")
                    .method(method)
                    .body(Body::from("not-json"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), status);
        assert_eq!(response.headers()["x-yetlet-scope"], "api");
        assert!(response.headers().contains_key("x-yetlet-request-id"));
        if method == "DELETE" {
            assert!(
                response.headers()["allow"]
                    .to_str()
                    .unwrap()
                    .contains("POST")
            );
        }
    }
}
