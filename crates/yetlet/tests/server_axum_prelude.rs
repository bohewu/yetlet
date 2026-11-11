#![cfg(not(target_arch = "wasm32"))]
//! Common adapter import surface is additive, typed and native-only.

use axum::{
    Router,
    body::{Body, to_bytes},
    http::Request,
};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use yetlet::server_axum::prelude::*;

#[derive(Serialize, Deserialize)]
struct Payload {
    message: String,
}

#[derive(Deserialize)]
struct Search {
    page: u32,
}

async fn home() -> View {
    View::text("Hello from prelude")
}

async fn json_echo(Json(value): Json<Payload>) -> Json<Payload> {
    Json(value)
}

async fn form_value(Form(value): Form<Payload>) -> String {
    value.message
}

async fn resource(Path(id): Path<u32>, Query(search): Query<Search>) -> String {
    format!("{id}:{}", search.page)
}

#[tokio::test]
async fn curated_prelude_keeps_native_handlers_and_responses() {
    let router: Router = App::new()
        .page("/", "Prelude", home)
        .post("/json", json_echo)
        .post("/form", form_value)
        .get("/resource/{id}", resource)
        .into_router();

    let html = router
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(html.status(), StatusCode::OK);
    assert_eq!(html.headers()["content-security-policy"], DEFAULT_CSP);
    let body = to_bytes(html.into_body(), 4096).await.unwrap();
    assert!(String::from_utf8_lossy(&body).contains("Hello from prelude"));

    let json = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/json")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"message":"ready"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(json.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(json.into_body(), 4096).await.unwrap().as_ref(),
        br#"{"message":"ready"}"#
    );

    let form = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/form")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("message=hello"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(form.status(), StatusCode::OK);
    assert_eq!(
        to_bytes(form.into_body(), 4096).await.unwrap().as_ref(),
        b"hello"
    );

    let path = router
        .oneshot(
            Request::builder()
                .uri("/resource/4?page=8")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        to_bytes(path.into_body(), 4096).await.unwrap().as_ref(),
        b"4:8"
    );
}

#[test]
fn prelude_exposes_only_the_selected_support_types() {
    let _attribute = std::mem::size_of::<Attribute>();
    let element = Element::new("main");
    let _: View = element.into();

    let _page: Page = Page::new("Title", View::text("body"));
    let _context = std::mem::size_of::<RequestContext>();
    let _error = std::mem::size_of::<HtmlPageError>();
    let _language_error = std::mem::size_of::<LanguageTagError>();
    let _default_limit = DefaultBodyLimit::max(1024);
    let _extension = std::mem::size_of::<Extension<String>>();
    let _state = std::mem::size_of::<State<String>>();

    let metadata = DocumentMetadata::with_language("Home", LanguageTag::new("en").unwrap());
    assert!(
        render_document("Home", &View::text("ok"))
            .unwrap()
            .contains("ok")
    );
    assert!(
        render_document_with_metadata(&metadata, &View::text("ok"))
            .unwrap()
            .contains("ok")
    );
    assert_eq!(
        render_page("Home", &View::text("ok")).unwrap().status(),
        StatusCode::OK
    );
    assert_eq!(
        render_page_with_metadata(&metadata, &View::text("ok"))
            .unwrap()
            .status(),
        StatusCode::OK
    );

    let mut headers = HeaderMap::new();
    headers.insert("x-yetlet", HeaderValue::from_static("ready"));
    assert_eq!(headers["x-yetlet"], "ready");
    let response: Response = (StatusCode::OK, "native").into_response();
    assert_eq!(response.status(), StatusCode::OK);
    let redirect: Response = Redirect::to("/").into_response();
    assert_eq!(redirect.status(), StatusCode::SEE_OTHER);

    let _: yetlet::server_axum::extract::Form<Payload> = Form(Payload {
        message: "ok".into(),
    });
    let _: yetlet::server_axum::extract::Json<Payload> = Json(Payload {
        message: "ok".into(),
    });
}
