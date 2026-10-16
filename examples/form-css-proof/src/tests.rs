use super::*;
use axum::{
    body::to_bytes,
    http::{HeaderMap, Method, Request},
};
use tower::ServiceExt;

async fn request(
    router: Router,
    method: Method,
    path: &str,
    body: &str,
) -> (StatusCode, HeaderMap, String) {
    let response = router
        .oneshot(
            Request::builder()
                .method(method)
                .uri(path)
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .body(Body::from(body.to_owned()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .unwrap();
    (
        status,
        headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}
fn headers_safe(headers: &HeaderMap) {
    assert_eq!(headers[header::CONTENT_SECURITY_POLICY], DEFAULT_CSP);
    assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
    assert!(!headers.contains_key(header::SET_COOKIE));
    assert!(!DEFAULT_CSP.contains("unsafe-inline"));
    assert!(!DEFAULT_CSP.contains("unsafe-eval"));
    assert!(!DEFAULT_CSP.contains("data:"));
}
#[tokio::test]
async fn native_form_is_styled_script_free_and_semantic() {
    let (status, headers, html) = request(app(), Method::GET, "/proof/native", "").await;
    assert_eq!(status, StatusCode::OK);
    headers_safe(&headers);
    assert!(html.starts_with("<!doctype html><html lang=\"en\">"));
    assert_eq!(html.matches("rel=\"stylesheet\"").count(), 2);
    assert!(!html.contains("<script"));
    assert!(!html.contains(".wasm"));
    for fragment in [
        "<form",
        "<select",
        "<textarea",
        "<table",
        "<caption",
        "<tbody",
        "type=\"reset\"",
        "checked=\"\"",
        "selected=\"\"",
        "disabled=\"\"",
        "required=\"\"",
    ] {
        assert!(html.contains(fragment), "{fragment}");
    }
    assert!(!html.contains("disabled=\"false\""));
}
#[tokio::test]
async fn preview_is_an_explicit_external_application_asset() {
    let (_, headers, html) = request(app(), Method::GET, "/proof/enhanced", "").await;
    headers_safe(&headers);
    assert_eq!(html.matches("<script").count(), 1);
    assert!(html.contains("type=\"module\" src=\"/form-proof-assets/enhance-"));
    assert!(html.contains("Preview inactive."));
}
#[tokio::test]
async fn valid_checked_and_unchecked_forms_use_stateless_prg() {
    for suffix in ["", "&notify=yes"] {
        let (status, headers, body) = request(
            app(),
            Method::POST,
            "/proof/submit",
            &format!("title=Test&notes=Plain+text&priority=normal{suffix}"),
        )
        .await;
        assert_eq!(status, StatusCode::SEE_OTHER);
        headers_safe(&headers);
        assert_eq!(headers[header::LOCATION], "/proof/accepted");
        assert!(!body.contains("Plain text"));
    }
    let (status, headers, html) = request(app(), Method::GET, "/proof/accepted", "").await;
    assert_eq!(status, StatusCode::OK);
    headers_safe(&headers);
    assert!(html.contains("Nothing was saved."));
}
#[tokio::test]
async fn error_echo_escapes_title_and_notes_and_preserves_unchecked_state() {
    let (status, headers, html) = request(app(), Method::POST, "/proof/submit", "title=%22%3E%3Cimg+src%3Dx+onerror%3D1%3E&notes=%3Cscript%3Ebad%3C%2Fscript%3E&priority=unknown").await;
    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    headers_safe(&headers);
    assert!(!html.contains("<script"));
    assert!(!html.contains("<img"));
    assert!(html.contains("&lt;script&gt;bad&lt;/script&gt;"));
    assert!(html.contains("&quot;&gt;&lt;img"));
    assert!(!html.contains("checked=\"\""));
    assert!(html.contains("aria-invalid=\"true\""));
}
#[tokio::test]
async fn malformed_duplicate_unknown_and_invalid_checkbox_fail() {
    for body in [
        "title=x&notes=x&priority=normal&role=admin",
        "title=x&title=y&notes=x&priority=normal",
        "notes=x&priority=normal",
        "title=x&notes=x&priority=normal&notify=false",
    ] {
        let (status, headers, _) = request(app(), Method::POST, "/proof/submit", body).await;
        assert!(status.is_client_error());
        headers_safe(&headers);
    }
}
#[tokio::test]
async fn body_limit_and_routing_errors_keep_security_headers() {
    let (status, headers, _) = request(
        app(),
        Method::POST,
        "/proof/submit",
        &"x".repeat(MAX_BODY + 1),
    )
    .await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    headers_safe(&headers);
    for (method, path, expected) in [
        (Method::GET, "/missing", StatusCode::NOT_FOUND),
        (
            Method::POST,
            "/proof/native",
            StatusCode::METHOD_NOT_ALLOWED,
        ),
        (
            Method::GET,
            "/form-proof-assets/not-registered.css",
            StatusCode::NOT_FOUND,
        ),
    ] {
        let (status, headers, _) = request(app(), method, path, "").await;
        assert_eq!(status, expected);
        headers_safe(&headers);
    }
}
#[tokio::test]
async fn missing_or_injected_manifest_degrades_only_optional_assets() {
    let good: serde_json::Value = serde_json::from_str(MANIFEST).unwrap();
    let mut bad = good.clone();
    bad["stylesheets"][0] = "https://example.invalid/injected.css".into();
    for source in [String::new(), "{}".into(), bad.to_string()] {
        let (status, headers, html) = request(
            app_with_manifest(&source),
            Method::GET,
            "/proof/enhanced",
            "",
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        headers_safe(&headers);
        assert!(!html.contains("<script"));
        assert!(!html.contains("rel=\"stylesheet\""));
        assert!(html.contains("Optional assets are unavailable"));
        assert!(html.contains("<form"));
        assert_eq!(
            request(app_with_manifest(&source), Method::GET, "/healthz", "")
                .await
                .0,
            StatusCode::SERVICE_UNAVAILABLE
        );
    }
}
#[test]
fn finite_manifest_rejects_path_and_mime_confusion_and_deduplicates_in_order() {
    let original: serde_json::Value = serde_json::from_str(MANIFEST).unwrap();
    for path in [
        "//example.invalid/style.css",
        "/form-proof-assets/../manifest.json",
        "/form-proof-assets/%2e%2e%2fsecret",
        "\" onload=evil()",
        "/form-proof-assets/\\bad",
        "javascript:alert(1)",
    ] {
        let mut bad = original.clone();
        bad["stylesheets"][0] = path.into();
        assert!(Manifest::parse(&bad.to_string()).is_err());
    }
    let mut bad = original.clone();
    bad["stylesheets"][0] = original["enhancement"].clone();
    assert!(Manifest::parse(&bad.to_string()).is_err());
    bad = original.clone();
    bad["unexpected"] = true.into();
    assert!(Manifest::parse(&bad.to_string()).is_err());
    bad = original.clone();
    bad["schema_version"] = 2.into();
    assert!(Manifest::parse(&bad.to_string()).is_err());
    assert!(Manifest::parse(&" ".repeat(4097)).is_err());
    let mut duplicate = original.clone();
    duplicate["stylesheets"]
        .as_array_mut()
        .unwrap()
        .push(original["stylesheets"][0].clone());
    assert_eq!(
        Manifest::parse(&duplicate.to_string()).unwrap().stylesheets,
        Manifest::parse(MANIFEST).unwrap().stylesheets
    );
}
#[tokio::test]
async fn all_registered_assets_have_exact_bytes_mimes_and_immutable_cache() {
    for asset in asset_data::ASSETS {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri(format!("{}{name}", assets::PREFIX, name = asset.name))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        headers_safe(response.headers());
        assert_eq!(response.headers()[header::CONTENT_TYPE], asset.content_type);
        assert_eq!(
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            to_bytes(response.into_body(), 1024 * 1024)
                .await
                .unwrap()
                .as_ref(),
            asset.bytes
        );
        if asset.name.starts_with("bootstrap-") {
            assert!(!String::from_utf8_lossy(asset.bytes).contains("data:"));
        }
    }
}
#[test]
fn unicode_lengths_controls_and_textarea_newline_are_explicit() {
    let mut draft = Draft {
        title: "測".repeat(160),
        ..Draft::default()
    };
    assert!(!validate(&draft).title);
    draft.title.push('試');
    assert!(validate(&draft).title);
    draft = Draft::default();
    draft.title.push('\n');
    assert!(validate(&draft).title);
    draft = Draft::default();
    draft.notes = "\nfirst line\n第二行".into();
    assert!(!validate(&draft).notes);
    let html = yetlet_html_for_test(&draft);
    assert!(html.contains("\n\nfirst line\n第二行</textarea>"));
}
fn yetlet_html_for_test(draft: &Draft) -> String {
    let state = AppState { manifest: None };
    // Rendering through the real application page path is tested above; serialize using the same server helper here.
    let _ = state;
    render_document_with_metadata(
        &DocumentMetadata::new("Test"),
        &views::form(draft, &Errors::default(), false, false),
    )
    .unwrap()
}
