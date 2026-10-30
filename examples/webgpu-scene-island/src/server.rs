use crate::RECORDS;
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::Value;
use std::{
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
};
use tower_http::services::ServeDir;
use yetlet::view::{Element, View};
use yetlet::server_axum::{
    DEFAULT_CSP, DocumentMetadata, LanguageTag, render_document_with_metadata,
};

const CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const PAGE_SIZE: usize = 12;
#[derive(Default, Deserialize)]
pub struct Search {
    #[serde(default)]
    q: String,
    #[serde(default)]
    page: usize,
    #[serde(default)]
    record: u32,
}
fn text(tag: &'static str, value: impl Into<String>) -> Element {
    Element::new(tag).child(View::text(value))
}
fn status(id: u32) -> &'static str {
    match (id - 1) % 3 {
        0 => "Blocked",
        1 => "Done",
        _ => "Active",
    }
}
fn record_label(id: u32) -> String {
    format!("Work record {id:03} — {}", status(id))
}
fn view(search: &Search) -> View {
    let q: String = search.q.chars().take(64).collect();
    let ids: Vec<_> = (1..=RECORDS)
        .filter(|id| record_label(*id).to_lowercase().contains(&q.to_lowercase()))
        .collect();
    let last = ids.len().saturating_sub(1) / PAGE_SIZE;
    let page = search.page.min(last);
    let mut list = Element::new("ul").attribute("id", "records");
    for id in ids.iter().skip(page * PAGE_SIZE).take(PAGE_SIZE) {
        list =
            list.child(Element::new("li").child(
                text("a", record_label(*id)).attribute("href", format!("/scene?record={id}")),
            ));
    }
    let form = Element::new("form")
        .attribute("action", "/scene")
        .attribute("method", "get")
        .child(text("label", "Search synthetic work records").attribute("for", "query"))
        .child(
            Element::new("input")
                .attribute("id", "query")
                .attribute("name", "q")
                .attribute("type", "search")
                .attribute("maxlength", "64")
                .attribute("value", q),
        )
        .child(
            Element::new("input")
                .attribute("name", "page")
                .attribute("type", "number")
                .attribute("min", "0")
                .attribute("max", last.to_string())
                .attribute("value", page.to_string())
                .attribute("aria-label", "Page number, starting at zero"),
        )
        .child(text("button", "Search / open page").attribute("type", "submit"));
    let detail = if (1..=RECORDS).contains(&search.record) {
        record_label(search.record)
    } else {
        "Choose a record in the HTML list for detail.".into()
    };
    Element::new("main")
        .child(text("h1", "Synthetic work-record status overview"))
        .child(text("p", "Synthetic data: 120 work records; 40 blocked, 40 done, 40 active. Each rectangle represents one record in row-major order. Red: blocked; green: done; blue: active."))
        .child(Element::new("section").attribute("aria-label", "Searchable data alternative")
            .child(text("h2", "Records in HTML"))
            .child(form)
            .child(text("p", format!("{} matches; page {} of {} (12 records per page)", ids.len(), page + 1, last + 1)).attribute("id", "matches"))
            .child(list)
            .child(text("p", detail).attribute("id", "detail"))
            .child(text("a", "Reset search").attribute("href", "/scene")))
        .child(Element::new("section").attribute("id", "scene-island")
            .child(text("h2", "Optional GPU overview"))
            .child(text("button", "Activate GPU overview").attribute("id", "activate-scene").attribute("type", "button"))
            .child(text("button", "Dispose GPU overview").attribute("id", "dispose-scene").attribute("type", "button"))
            .child(text("p", "GPU inactive or assets unavailable. The HTML records remain available.").attribute("id", "scene-status").attribute("role", "status"))
            .child(text("p", "Tap a rectangle to select its record. Scrolling or cancelling a touch leaves selection unchanged. Use the HTML list and search with or without GPU. Keyboard: focus Select record, use arrows or Home/End, then Enter or Space; Reset clears selection.").attribute("id", "selection-help"))
            .child(Element::new("div").attribute("id", "selection-controls").attribute("hidden", "")
                .child(text("button", "Select record 001").attribute("id", "scene-navigate").attribute("type", "button").attribute("aria-describedby", "selection-help"))
                .child(text("button", "Reset selection").attribute("id", "scene-reset").attribute("type", "button")))
            .child(text("p", "No scene record selected.").attribute("id", "scene-selection").attribute("role", "status"))
            .child(text("a", "Open selected record in HTML").attribute("id", "scene-detail-link").attribute("hidden", ""))
            .child(Element::new("canvas").attribute("id", "scene-canvas").attribute("width", "600").attribute("height", "400").attribute("aria-label", "Optional synthetic record grid; use the HTML record list for accessible detail").child(View::text("Use the HTML records above."))))
        .child(Element::new("section").attribute("id", "counter-island").attribute("data-yetlet-island", "counter").attribute("data-island-version", "1")
            .attribute("data-island-state", r#"{"schema_version":1,"component":"yetlet-dashboard-counter","count":2}"#).attribute("data-current-count", "2")
            .child(text("h2", "Independent local counter"))
            .child(text("strong", "2").attribute("id", "counter-value"))
            .child(text("button", "Increment").attribute("id", "counter-increment").attribute("type", "button"))
            .child(text("button", "Delayed increment").attribute("id", "counter-delayed").attribute("type", "button"))
            .child(text("p", "Counter value: 2").attribute("id", "counter-status").attribute("role", "status")))
        .into()
}

/// Exact local paths only; each entry has its own content-addressed directory.
async fn loader(root: &Path, key: &str, module: &str) -> Option<String> {
    let path = root.join("manifest.json");
    if tokio::fs::metadata(&path).await.ok()?.len() > 8192 {
        return None;
    }
    let manifest: Value = serde_json::from_slice(&tokio::fs::read(path).await.ok()?).ok()?;
    if manifest["schema_version"].as_u64()? != 1 {
        return None;
    }
    let entry = &manifest[key];
    let digest = entry["digest"].as_str()?;
    if digest.len() != 16 || !digest.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    for (field, name) in [
        ("loader", format!("{key}-loader.js")),
        ("module", format!("{module}.js")),
        ("wasm", format!("{module}_bg.wasm")),
    ] {
        if entry[field].as_str()? != format!("/scene-assets/{digest}/{name}")
            || !root.join("assets").join(digest).join(name).is_file()
        {
            return None;
        }
    }
    entry["loader"].as_str().map(str::to_owned)
}
async fn page(State(root): State<PathBuf>, Query(search): Query<Search>) -> Response {
    render_page(root, search, false).await
}
async fn comparison(State(root): State<PathBuf>, Query(search): Query<Search>) -> Response {
    render_page(root, search, true).await
}
async fn render_page(root: PathBuf, search: Search, comparison: bool) -> Response {
    let metadata = DocumentMetadata::with_language(
        "Yetlet | Synthetic work overview",
        LanguageTag::new("en").expect("static English language tag"),
    );
    let mut html = match render_document_with_metadata(&metadata, &view(&search)) {
        Ok(html) => html,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to render records",
            )
                .into_response();
        }
    };
    let mut tags = String::new();
    for (key, module) in [("scene", "scene"), ("counter", "yetlet_dashboard_counter")] {
        if let Some(url) = loader(&root, key, module).await {
            tags.push_str(&format!("<script type=\"module\" src=\"{url}\"></script>"));
        }
    }
    if comparison && let Some(url) = loader(&root, "scene", "scene").await {
        let url = url.replace("scene-loader.js", "comparison-loader.js");
        let local = root
            .join("assets")
            .join(url.split('/').nth(2).unwrap_or(""))
            .join("comparison-loader.js");
        if local.is_file() {
            tags.push_str(&format!("<script type=\"module\" src=\"{url}\"></script>"));
        }
    }
    let enhanced = !tags.is_empty();
    html = html.replacen(
        "</head>",
        &format!("<link rel=\"stylesheet\" href=\"/scene.css\">{tags}</head>"),
        1,
    );
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(if enhanced { CSP } else { DEFAULT_CSP }),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}
pub fn listen_address(value: &str) -> io::Result<SocketAddr> {
    let address: SocketAddr = value
        .parse()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;
    if !address.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "loopback only",
        ));
    }
    Ok(address)
}
pub fn app(root: PathBuf) -> Router {
    Router::new()
        .route(
            "/scene.css",
            axum::routing::get(|| async {
                (
                    [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                    "#scene-canvas{display:block;width:600px;max-width:100%;height:auto;aspect-ratio:3/2;touch-action:pan-x pan-y pinch-zoom}#comparison-canvas{width:600px;height:400px;max-width:100%}button,input{font:inherit}#selection-controls{margin-block:0.5rem}button{min-height:44px;max-width:100%;white-space:normal}main{max-width:60rem}input[type=search]{max-width:100%;box-sizing:border-box}a:focus-visible,button:focus-visible,input:focus-visible{outline:3px solid currentColor;outline-offset:2px}@media(max-width:480px){main{overflow-wrap:anywhere}form label{display:block}input[type=search]{width:100%}#scene-island{margin-block:1rem}}@media(prefers-reduced-motion:reduce){html{scroll-behavior:auto}}",
                )
            }),
        )
        .route("/scene", axum::routing::get(page))
        .route("/scene-comparison", axum::routing::get(comparison))
        .route("/healthz", axum::routing::get(|| async { "ok" }))
        .route(
            "/favicon.ico",
            axum::routing::get(|| async { StatusCode::NO_CONTENT }),
        )
        .nest_service("/scene-assets", ServeDir::new(root.join("assets")))
        .with_state(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;
    use yetlet::server_axum::render_document;
    #[tokio::test]
    async fn ssr_without_assets_is_searchable_paginated_and_escaped() {
        let root = tempfile::tempdir().unwrap();
        let response = app(root.path().into())
            .oneshot(
                Request::builder()
                    .uri("/scene?q=done&page=1")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        let html = String::from_utf8(
            to_bytes(response.into_body(), 100_000)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.starts_with("<!doctype html><html lang=\"en\">"));
        assert!(html.contains("40 matches; page 2 of 4"));
        assert!(html.contains("Work record 038"));
        assert_eq!(html.matches("<li>").count(), 12);
        assert!(html.contains("aria-describedby=\"selection-help\""));
        assert!(html.contains("id=\"scene-reset\""));
        assert!(html.contains("No scene record selected."));
        assert!(!html.contains("<script"));
        let html = render_document(
            "test",
            &view(&Search {
                q: "<script>".into(),
                page: usize::MAX,
                record: u32::MAX,
            }),
        )
        .unwrap();
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>"));
    }
    #[tokio::test]
    async fn manifest_failures_do_not_remove_sibling() {
        let root = tempfile::tempdir().unwrap();
        let digest = "0123456789abcdef";
        let dir = root.path().join("assets").join(digest);
        std::fs::create_dir_all(&dir).unwrap();
        let mut manifest = serde_json::json!({"schema_version":1});
        for (key, module) in [("scene", "scene"), ("counter", "yetlet_dashboard_counter")] {
            let mut entry = serde_json::json!({"digest":digest});
            for (field, name) in [
                ("loader", format!("{key}-loader.js")),
                ("module", format!("{module}.js")),
                ("wasm", format!("{module}_bg.wasm")),
            ] {
                std::fs::write(dir.join(&name), "fixture").unwrap();
                entry[field] = format!("/scene-assets/{digest}/{name}").into();
            }
            manifest[key] = entry;
        }
        let path = root.path().join("manifest.json");
        std::fs::write(&path, manifest.to_string()).unwrap();
        assert!(loader(root.path(), "scene", "scene").await.is_some());
        std::fs::remove_file(dir.join("scene_bg.wasm")).unwrap();
        assert!(loader(root.path(), "scene", "scene").await.is_none());
        assert!(
            loader(root.path(), "counter", "yetlet_dashboard_counter")
                .await
                .is_some()
        );
        std::fs::write(dir.join("scene_bg.wasm"), "fixture").unwrap();
        manifest["scene"]["loader"] = "//example.com/evil.js".into();
        std::fs::write(&path, manifest.to_string()).unwrap();
        assert!(loader(root.path(), "scene", "scene").await.is_none());
        manifest["scene"]["digest"] = "../../etc/passwd".into();
        std::fs::write(&path, manifest.to_string()).unwrap();
        assert!(loader(root.path(), "scene", "scene").await.is_none());
        std::fs::write(&path, "invalid").unwrap();
        assert!(loader(root.path(), "scene", "scene").await.is_none());
    }
    #[test]
    fn loopback_and_input_bounds() {
        assert!(listen_address("0.0.0.0:3010").is_err());
        assert!(listen_address("127.0.0.1:3010").is_ok());
        let html = render_document(
            "test",
            &view(&Search {
                q: "x".repeat(1000),
                page: usize::MAX,
                record: 120,
            }),
        )
        .unwrap();
        assert!(html.contains("Work record 120"));
        assert!(!html.contains(&"x".repeat(65)));
        assert!(html.contains("0 matches; page 1 of 1"));
    }
    #[test]
    fn semantic_detail_links_are_bounded_and_escape_text() {
        for (id, expected) in [(1, "Work record 001"), (120, "Work record 120")] {
            let html = render_document(
                "test",
                &view(&Search {
                    record: id,
                    ..Search::default()
                }),
            )
            .unwrap();
            assert!(html.contains(expected));
            assert!(html.contains("href=\"/scene?record=1\""));
            assert_eq!(html.matches("<li>").count(), 12);
        }
        for id in [0, 121, u32::MAX] {
            let html = render_document(
                "test",
                &view(&Search {
                    record: id,
                    ..Search::default()
                }),
            )
            .unwrap();
            assert!(html.contains("Choose a record in the HTML list for detail."));
        }
    }
}
