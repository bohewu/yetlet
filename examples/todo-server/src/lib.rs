//! Yetlet native HTTP adapter demo.
//!
//! UI components live in `yetlet-todo-ui` and do not depend on Axum.

mod api;
mod pipeline;

use std::sync::{Arc, atomic::AtomicU64};

use axum::{
    Json, Router,
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde_json::{Value, json};
use tower_http::services::ServeDir;
use yetlet_server_axum::{
    DEFAULT_CSP, HtmlPageError, RequestContext,
    middleware::{from_fn, from_fn_with_state, with_global},
    render_document, render_page,
};
use yetlet_todo_ui::{home_page, search_page};

async fn root(context: RequestContext<api::TodoApiState>) -> Result<Response, HtmlPageError> {
    let safe_render = pipeline::RenderRequest::from_context(&context);
    let mut view = home_page();
    if let Some(id) = safe_render.request_id {
        view = yetlet_core::Element::new("div")
            .attribute("data-request-id", id.to_string())
            .child(view)
            .into();
    }
    render_page("Yetlet | Todo", &view)
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

/// An SSR route must remain readable without the optional island bundle.
/// Only validated, build-time versioned assets can become markup attributes.
async fn search_route() -> Response {
    let view = search_page();
    let mut html = match render_document("Yetlet | SSR Search", &view) {
        Ok(value) => value,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to render the page",
            )
                .into_response();
        }
    };
    let manifest = tokio::fs::read_to_string("dist/manifest.json")
        .await
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok());

    let enhanced = manifest.as_ref().and_then(|manifest| {
        let digest = manifest.get("digest")?.as_str()?;
        if digest.len() != 16 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return None;
        }
        let loader = manifest.get("search_loader")?.as_str()?;
        let css = manifest.get("stylesheet")?.as_str()?;
        let icon = manifest.get("favicon")?.as_str()?;
        if loader != format!("/assets/{digest}/search-loader.js")
            || css != format!("/assets/{digest}/styles.css")
            || icon != format!("/assets/{digest}/favicon.svg")
        {
            return None;
        }
        Some(format!(
            "<link rel=\"stylesheet\" href=\"{css}\"><link rel=\"icon\" type=\"image/svg+xml\" href=\"{icon}\"><script type=\"module\" src=\"{loader}\"></script>"
        ))
    });
    if let Some(tags) = enhanced.as_deref() {
        html = html.replacen("</head>", &format!("{tags}</head>"), 1);
    }

    let mut response = Html(html).into_response();
    let csp = if enhanced.is_some() {
        BROWSER_CSP
    } else {
        DEFAULT_CSP
    };
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(csp),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

pub fn app() -> Router {
    let state = api::TodoApiState::default();
    let api_routes = Router::new()
        .route("/ping", get(api_ping))
        .route(
            "/todos",
            get(api::list_todos)
                .post(api::mutate_todos)
                .layer(from_fn_with_state(state.clone(), api::action_headers)),
        )
        .route_layer(from_fn(pipeline::api_scope));
    let routes = Router::new()
        .route("/", get(root))
        .route("/app", get(browser_shell))
        .route("/search", get(search_route))
        .route("/healthz", get(|| async { "ok" }))
        .nest("/api", api_routes)
        .nest_service("/assets", ServeDir::new("dist/assets"))
        .with_state(state);
    with_global(
        routes,
        from_fn_with_state(Arc::new(AtomicU64::new(1)), pipeline::assign_request_id),
    )
}

#[cfg(test)]
mod tests {
    use super::{BROWSER_CSP, DEFAULT_CSP, app};
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
    async fn search_route_returns_full_ssr_markup_with_optional_island() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/search")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let csp = response.headers()[header::CONTENT_SECURITY_POLICY]
            .to_str()
            .unwrap()
            .to_string();
        assert!(!csp.contains("'unsafe-inline'"));
        assert!(!csp.contains("'unsafe-eval'"));
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let html = std::str::from_utf8(&body).unwrap();
        assert!(html.contains("id=\"search-island\""));
        assert!(html.contains("data-island-version=\"1\""));
        assert!(html.contains("id=\"search-query\""));
        assert!(html.contains("<ul class=\"todo-list\">"));
        assert!(html.contains("2 matching tasks"));
        // Missing optional assets only removes enhancement, never SSR content.
        if html.contains("search-loader.js") {
            assert!(csp.contains("'wasm-unsafe-eval'"));
            assert!(html.contains("type=\"module\""));
        } else {
            assert_eq!(csp, DEFAULT_CSP);
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
