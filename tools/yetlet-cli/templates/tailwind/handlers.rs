async fn home(
    axum::Extension(stylesheet): axum::Extension<std::sync::Arc<tailwind::Stylesheet>>,
@ARGUMENTS@) -> axum::response::Response {
    stylesheet.page(
        "Yetlet",
        Element::new("main")
            .attribute("class", "mx-auto max-w-3xl p-8 font-sans text-slate-900")
            .child(
                Element::new("h1")
                    .attribute("class", "text-3xl font-bold")
                    .child(View::text(@MESSAGE@)),
            )
            .into(),
    )
}

async fn hello(
    axum::Extension(stylesheet): axum::Extension<std::sync::Arc<tailwind::Stylesheet>>,
    Path(name): Path<String>,
) -> axum::response::Response {
    stylesheet.page(
        "Hello",
        Element::new("main")
            .attribute("class", "mx-auto max-w-3xl p-8 font-sans text-slate-900")
            .child(View::text(format!("Hello, {name}!")))
            .into(),
    )
}

#[cfg(test)]
mod tailwind_app_tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    #[tokio::test]
    async fn generated_hello_route_serves_escaped_ssr_and_digest_link() {
        let stylesheet = tailwind::Stylesheet::load().expect("explicit CSS build before tests");
        let router = tailwind::install(
            axum::Router::new().route("/hello/{name}", get(hello)),
            stylesheet,
        );
        let response = router
            .oneshot(
                Request::builder()
                    .uri("/hello/%3Cscript%3E%26")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), axum::http::StatusCode::OK);
        assert_eq!(
            response.headers()[axum::http::header::CONTENT_SECURITY_POLICY],
            yetlet::server_axum::DEFAULT_CSP
        );
        let bytes = axum::body::to_bytes(response.into_body(), 8192)
            .await
            .unwrap();
        let html = std::str::from_utf8(&bytes).unwrap();
        assert!(html.contains("Hello, &lt;script&gt;&amp;!"));
        assert!(html.contains("<link rel=\"stylesheet\" href=\"/assets/tailwind."));
        assert!(!html.contains("<script") && !html.contains("<style"));
    }
}
