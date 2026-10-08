//! Yetlet native HTTP adapter demo.
//!
//! UI components live in `yetlet-todo-ui` and do not depend on Axum.

use axum::{
    Json, Router,
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde_json::{Value, json};
use tower_http::services::ServeDir;
use yetlet_server_axum::{HtmlPageError, render_page};
use yetlet_todo_ui::home_page;

async fn root() -> Result<Response, HtmlPageError> {
    render_page("Yetlet | Todo", &home_page())
}

async fn api_ping() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

// Browser WASM needs the narrow wasm-unsafe-eval CSP token.
// The root SSR response retains its original stricter policy.
const BROWSER_CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";

async fn browser_shell() -> Response {
    let mut response = match tokio::fs::read_to_string("dist/app.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Build browser assets first: python3 scripts/build_browser.py",
        )
            .into_response(),
    };
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(BROWSER_CSP),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/app", get(browser_shell))
        .route("/healthz", get(|| async { "ok" }))
        .route("/api/ping", get(api_ping))
        .nest_service("/assets", ServeDir::new("dist/assets"))
}

#[cfg(test)]
mod tests {
    use super::{BROWSER_CSP, app};
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn get_root_is_static_ssr_without_script() {
        let response = app()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response
                .headers()
                .contains_key(header::CONTENT_SECURITY_POLICY)
        );
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let document = std::str::from_utf8(&body).unwrap();

        assert!(document.contains("<main id=\"app\">"));
        assert!(document.contains("Share one Rust View across renderers"));
        assert!(!document.contains("<script"));
        assert!(!document.contains(".wasm"));
    }

    #[tokio::test]
    async fn api_pipeline_returns_json_without_ui_renderer() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/ping")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(
            response.headers()[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("application/json")
        );
        let body = to_bytes(response.into_body(), 1024).await.unwrap();
        assert_eq!(&body[..], br#"{"status":"ok"}"#);
    }

    #[tokio::test]
    async fn browser_shell_has_wasm_csp_and_explicit_missing_assets() {
        let response = app()
            .oneshot(Request::builder().uri("/app").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::OK | StatusCode::SERVICE_UNAVAILABLE
        ));
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            BROWSER_CSP
        );
        assert!(!BROWSER_CSP.contains("'unsafe-inline'"));
        assert!(!BROWSER_CSP.contains("'unsafe-eval'"));
        let status = response.status();
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let page = std::str::from_utf8(&body).unwrap();
        if status == StatusCode::OK {
            assert!(page.contains("type=\"module\""));
            assert!(page.contains("/assets/"));
        } else {
            assert!(page.contains("Build browser assets first"));
        }
    }

    #[tokio::test]
    async fn get_healthz() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }
}
