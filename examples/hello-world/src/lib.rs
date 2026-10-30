//! Smallest independent native Yetlet SSR application.
//! No optional Session, request ID, database, DOM or WASM dependencies.

use axum::{Router, extract::Path};
use yetlet::server_axum::{App, Page};
use yetlet::view::{Element, View};

async fn home() -> View {
    Element::new("main")
        .child(Element::new("h1").child(View::text("Hello, world!")))
        .into()
}

async fn hello(Path(name): Path<String>) -> Page {
    Page::new(
        "Yetlet Hello",
        Element::new("main").child(Element::new("h1").child(View::text(format!("Hello, {name}!")))),
    )
}

/// The same router can be served by native Axum, without a Yetlet runner.
pub fn app() -> Router {
    App::new()
        .page("/", "Yetlet Hello", home)
        .get("/hello/{name}", hello)
        .into_router()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request, StatusCode, header},
    };
    use tower::ServiceExt;

    async fn response(method: Method, uri: &str) -> (StatusCode, axum::http::HeaderMap, String) {
        let request = Request::builder()
            .method(method)
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let response = app().oneshot(request).await.unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
        (status, headers, String::from_utf8(body.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn root_is_native_ssr_with_strict_csp_and_no_browser_runtime() {
        let (status, headers, html) = response(Method::GET, "/").await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Hello, world!"));
        assert!(html.contains("<main>"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("wasm"));
        assert!(
            headers[header::CONTENT_TYPE]
                .to_str()
                .unwrap()
                .starts_with("text/html")
        );
        assert!(
            headers[header::CONTENT_SECURITY_POLICY]
                .to_str()
                .unwrap()
                .contains("default-src 'none'")
        );
        assert!(!headers.contains_key(header::SET_COOKIE));
        assert!(!headers.contains_key("x-yetlet-request-id"));
    }

    #[tokio::test]
    async fn untrusted_path_text_is_escaped() {
        let (status, headers, html) = response(
            Method::GET,
            "/hello/%3Cimg%20src%3Dx%20onerror%3Dalert(1)%3E",
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(!html.contains("<img"));
        assert!(!headers.contains_key(header::SET_COOKIE));
    }

    #[tokio::test]
    async fn unmatched_and_unsupported_methods_keep_native_statuses() {
        assert_eq!(
            response(Method::GET, "/unknown").await.0,
            StatusCode::NOT_FOUND
        );
        let (status, headers, _) = response(Method::POST, "/hello/Yetlet").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert!(headers.contains_key(header::ALLOW));
    }
}
