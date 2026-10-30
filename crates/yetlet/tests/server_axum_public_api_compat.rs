#![cfg(not(target_arch = "wasm32"))]
//! Compile-and-run compatibility fixture for the pre-public server surface.

use axum::{
    Router,
    body::{Body, to_bytes},
    http::Request,
};
use std::borrow::Cow;
use tower::ServiceExt;
use yetlet::server_axum::{
    App, DEFAULT_CSP, DocumentMetadata, LanguageTag, Page, render_document,
    render_document_with_metadata, render_page, render_page_with_metadata,
};
use yetlet::view::View;

async fn legacy_home() -> View {
    View::text("legacy")
}

fn legacy_generic_page<T: Into<Cow<'static, str>>>(title: T) -> Page {
    Page::new(title, View::text("body"))
}

#[test]
fn legacy_page_and_rendering_entry_points_remain_source_compatible() {
    let _: Router = App::new().page("/", "Legacy", legacy_home).into_router();
    let _: Page = Page::new("Legacy", View::text("body"));
    let _: Page = legacy_generic_page(Cow::Borrowed("Borrowed"));
    let _: Page = legacy_generic_page(String::from("Owned"));

    let html = render_document("Legacy", &View::text("body")).unwrap();
    assert!(html.starts_with("<!doctype html><html lang=\"zh-Hant\">"));
    let response = render_page("Legacy", &View::text("body")).unwrap();
    assert_eq!(response.headers()["content-security-policy"], DEFAULT_CSP);
}

#[tokio::test]
async fn metadata_entry_points_are_additive_and_keep_security_defaults() {
    let language = LanguageTag::new("en").unwrap();
    let metadata = DocumentMetadata::with_language("Title <safe>", language);
    assert_eq!(metadata.title(), "Title <safe>");
    assert_eq!(metadata.language().as_str(), "en");

    let html = render_document_with_metadata(&metadata, &View::text("<body>")).unwrap();
    assert!(html.starts_with("<!doctype html><html lang=\"en\">"));
    assert!(html.contains("<title>Title &lt;safe&gt;</title>"));
    assert!(html.contains("&lt;body&gt;"));
    assert!(!html.contains("<script"));

    let response = render_page_with_metadata(&metadata, &View::text("ok")).unwrap();
    assert_eq!(response.headers()["content-security-policy"], DEFAULT_CSP);

    let app: Router = App::new()
        .get("/", || async {
            Page::with_metadata(
                DocumentMetadata::with_language("English", LanguageTag::new("en-US").unwrap()),
                View::text("Hello"),
            )
        })
        .into_router();
    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.headers()["content-security-policy"], DEFAULT_CSP);
    let body = String::from_utf8(
        to_bytes(response.into_body(), 16 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(body.starts_with("<!doctype html><html lang=\"en-US\">"));
}
