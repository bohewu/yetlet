//! SSR-first page with two independently validated and activated WASM islands.
//!
//! Missing or invalid assets remove only the affected enhancement. The complete
//! dashboard remains readable native SSR with no browser runtime requirement.

use std::{
    io,
    net::SocketAddr,
    path::{Path, PathBuf},
};

use axum::{
    Router,
    extract::State,
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use serde_json::{Value, json};
use tower_http::services::ServeDir;
use yetlet::server_axum::{App, DEFAULT_CSP, render_document};
use yetlet::view::{Element, View};

const BROWSER_CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const FILTER_ITEMS: [&str; 4] = [
    "Server rendering",
    "WebAssembly",
    "Accessibility",
    "Rust handlers",
];

#[derive(Clone)]
struct DashboardState {
    asset_root: PathBuf,
}

#[derive(Default)]
struct Enhancement {
    stylesheet: Option<String>,
    counter_loader: Option<String>,
    filter_loader: Option<String>,
}

fn dashboard_view() -> View {
    let counter_state = json!({
        "schema_version": 1,
        "component": "yetlet-dashboard-counter",
        "count": 2,
    })
    .to_string();
    let filter_state = json!({
        "schema_version": 1,
        "component": "yetlet-dashboard-filter",
        "items": FILTER_ITEMS,
    })
    .to_string();

    let counter = Element::new("section")
        .attribute("id", "counter-island")
        .attribute("data-yetlet-island", "counter")
        .attribute("data-island-version", "1")
        .attribute("data-island-state", counter_state)
        .attribute("data-current-count", "2")
        .child(Element::new("h2").child(View::text("Local counter")))
        .child(
            Element::new("p").child(View::text("Count: ")).child(
                Element::new("strong")
                    .attribute("id", "counter-value")
                    .attribute("data-count", "2")
                    .child(View::text("2")),
            ),
        )
        .child(
            Element::new("button")
                .attribute("id", "counter-increment")
                .attribute("type", "button")
                .child(View::text("Increment")),
        )
        .child(
            Element::new("button")
                .attribute("id", "counter-delayed")
                .attribute("type", "button")
                .child(View::text("Delayed increment")),
        )
        .child(
            Element::new("p")
                .attribute("id", "counter-status")
                .attribute("aria-live", "polite")
                .child(View::text("Counter value: 2")),
        );

    let mut list = Element::new("ul").attribute("id", "filter-results");
    for item in FILTER_ITEMS {
        list = list.child(Element::new("li").child(View::text(item)));
    }
    let filter = Element::new("section")
        .attribute("id", "filter-island")
        .attribute("data-yetlet-island", "filter")
        .attribute("data-island-version", "1")
        .attribute("data-island-state", filter_state)
        .child(Element::new("h2").child(View::text("Local filter")))
        .child(
            Element::new("label")
                .attribute("for", "filter-query")
                .child(View::text("Filter dashboard topics")),
        )
        .child(
            Element::new("input")
                .attribute("id", "filter-query")
                .attribute("type", "search")
                .attribute("autocomplete", "off"),
        )
        .child(
            Element::new("p")
                .attribute("id", "filter-count")
                .attribute("aria-live", "polite")
                .child(View::text("4 matching items")),
        )
        .child(list);

    Element::new("main")
        .attribute("id", "dashboard-page")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text("Yetlet Multi-Island Dashboard")))
        .child(Element::new("p").child(View::text(
            "Both panels are complete server-rendered HTML. Each optional island owns only its own panel.",
        )))
        .child(counter)
        .child(filter)
        .into()
}

fn valid_digest(value: &str) -> bool {
    value.len() == 16 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn exact_asset(value: &Value, key: &str, expected: &str, file: &Path) -> Option<String> {
    let path = value.get(key)?.as_str()?;
    if path != expected || !file.is_file() {
        return None;
    }
    Some(path.to_owned())
}

fn validate_island(
    manifest: &Value,
    root: &Path,
    digest: &str,
    island: &str,
    module_name: &str,
) -> Option<String> {
    let value = manifest.get(island)?;
    let directory = root.join("assets").join(digest);
    let loader_name = format!("{island}-loader.js");
    let module_file = format!("{module_name}.js");
    let wasm_file = format!("{module_name}_bg.wasm");
    let loader_path = format!("/dashboard-assets/{digest}/{loader_name}");
    let module_path = format!("/dashboard-assets/{digest}/{module_file}");
    let wasm_path = format!("/dashboard-assets/{digest}/{wasm_file}");
    exact_asset(value, "loader", &loader_path, &directory.join(&loader_name))?;
    exact_asset(value, "module", &module_path, &directory.join(&module_file))?;
    exact_asset(value, "wasm", &wasm_path, &directory.join(&wasm_file))?;
    Some(loader_path)
}

async fn enhancement(root: &Path) -> Enhancement {
    let Ok(text) = tokio::fs::read_to_string(root.join("manifest.json")).await else {
        return Enhancement::default();
    };
    let Ok(manifest) = serde_json::from_str::<Value>(&text) else {
        return Enhancement::default();
    };
    if manifest.get("schema_version").and_then(Value::as_u64) != Some(1) {
        return Enhancement::default();
    }
    let Some(digest) = manifest.get("digest").and_then(Value::as_str) else {
        return Enhancement::default();
    };
    if !valid_digest(digest) {
        return Enhancement::default();
    }
    let directory = root.join("assets").join(digest);
    let stylesheet_name = "dashboard.css";
    let stylesheet_path = format!("/dashboard-assets/{digest}/{stylesheet_name}");
    Enhancement {
        stylesheet: exact_asset(
            &manifest,
            "stylesheet",
            &stylesheet_path,
            &directory.join(stylesheet_name),
        ),
        counter_loader: validate_island(
            &manifest,
            root,
            digest,
            "counter",
            "yetlet_dashboard_counter",
        ),
        filter_loader: validate_island(
            &manifest,
            root,
            digest,
            "filter",
            "yetlet_dashboard_filter",
        ),
    }
}

async fn favicon() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn dashboard(State(state): State<DashboardState>) -> Response {
    let mut html = match render_document("Yetlet | Multi-Island Dashboard", &dashboard_view()) {
        Ok(html) => html,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to render the dashboard",
            )
                .into_response();
        }
    };
    let enhancement = enhancement(&state.asset_root).await;
    let mut tags = String::new();
    if let Some(stylesheet) = &enhancement.stylesheet {
        tags.push_str(&format!("<link rel=\"stylesheet\" href=\"{stylesheet}\">"));
    }
    if let Some(loader) = &enhancement.counter_loader {
        tags.push_str(&format!(
            "<script type=\"module\" data-island-loader=\"counter\" src=\"{loader}\"></script>"
        ));
    }
    if let Some(loader) = &enhancement.filter_loader {
        tags.push_str(&format!(
            "<script type=\"module\" data-island-loader=\"filter\" src=\"{loader}\"></script>"
        ));
    }
    if !tags.is_empty() {
        html = html.replacen("</head>", &format!("{tags}</head>"), 1);
    }

    let enhanced = enhancement.counter_loader.is_some() || enhancement.filter_loader.is_some();
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(if enhanced { BROWSER_CSP } else { DEFAULT_CSP }),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

/// This anonymous browser fixture remains local-only.
pub fn listen_address(value: &str) -> io::Result<SocketAddr> {
    let address: SocketAddr = value
        .parse()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    if !address.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the anonymous dashboard must bind a loopback address",
        ));
    }
    Ok(address)
}

pub fn app(asset_root: PathBuf) -> Router {
    let static_root = asset_root.join("assets");
    App::<DashboardState>::new()
        .get("/healthz", || async { "ok" })
        .get("/favicon.ico", favicon)
        .get("/dashboard", dashboard)
        .map_router(|router| router.nest_service("/dashboard-assets", ServeDir::new(static_root)))
        .with_state(DashboardState { asset_root })
        .into_router()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request},
    };
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn write_fixture(root: &Path) -> String {
        let digest = "0123456789abcdef";
        let directory = root.join("assets").join(digest);
        std::fs::create_dir_all(&directory).unwrap();
        for name in [
            "dashboard.css",
            "counter-loader.js",
            "yetlet_dashboard_counter.js",
            "yetlet_dashboard_counter_bg.wasm",
            "filter-loader.js",
            "yetlet_dashboard_filter.js",
            "yetlet_dashboard_filter_bg.wasm",
        ] {
            std::fs::write(directory.join(name), format!("fixture {name}")).unwrap();
        }
        let prefix = format!("/dashboard-assets/{digest}");
        let manifest = json!({
            "schema_version": 1,
            "digest": digest,
            "stylesheet": format!("{prefix}/dashboard.css"),
            "counter": {
                "loader": format!("{prefix}/counter-loader.js"),
                "module": format!("{prefix}/yetlet_dashboard_counter.js"),
                "wasm": format!("{prefix}/yetlet_dashboard_counter_bg.wasm"),
            },
            "filter": {
                "loader": format!("{prefix}/filter-loader.js"),
                "module": format!("{prefix}/yetlet_dashboard_filter.js"),
                "wasm": format!("{prefix}/yetlet_dashboard_filter_bg.wasm"),
            }
        });
        std::fs::write(
            root.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        digest.to_owned()
    }

    async fn request(
        root: &Path,
        method: Method,
        uri: &str,
    ) -> (StatusCode, axum::http::HeaderMap, String) {
        let response = app(root.to_owned())
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        (status, headers, String::from_utf8_lossy(&body).into_owned())
    }

    #[tokio::test]
    async fn missing_manifest_keeps_complete_script_free_ssr() {
        let root = TempDir::new().unwrap();
        let (status, headers, html) = request(root.path(), Method::GET, "/dashboard").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(headers[header::CONTENT_SECURITY_POLICY], DEFAULT_CSP);
        assert!(html.contains("Yetlet Multi-Island Dashboard"));
        assert!(html.contains("id=\"counter-island\""));
        assert!(html.contains("id=\"filter-island\""));
        assert!(html.contains("Server rendering"));
        assert!(!html.contains("<script"));
        assert!(html.contains("&quot;component&quot;:&quot;yetlet-dashboard-counter&quot;"));
    }

    #[tokio::test]
    async fn valid_manifest_enhances_both_islands_with_separate_loaders() {
        let root = TempDir::new().unwrap();
        let digest = write_fixture(root.path());
        let (_, headers, html) = request(root.path(), Method::GET, "/dashboard").await;
        assert_eq!(headers[header::CONTENT_SECURITY_POLICY], BROWSER_CSP);
        assert!(html.contains(&format!("/{digest}/counter-loader.js")));
        assert!(html.contains(&format!("/{digest}/filter-loader.js")));
        assert!(html.contains(&format!("/{digest}/dashboard.css")));
        assert_eq!(html.matches("data-island-loader=").count(), 2);
    }

    #[tokio::test]
    async fn one_missing_bundle_degrades_only_that_island() {
        let root = TempDir::new().unwrap();
        let digest = write_fixture(root.path());
        std::fs::remove_file(
            root.path()
                .join("assets")
                .join(&digest)
                .join("yetlet_dashboard_counter_bg.wasm"),
        )
        .unwrap();
        let (_, headers, html) = request(root.path(), Method::GET, "/dashboard").await;
        assert_eq!(headers[header::CONTENT_SECURITY_POLICY], BROWSER_CSP);
        assert!(!html.contains("counter-loader.js"));
        assert!(html.contains("filter-loader.js"));
        assert!(html.contains("id=\"counter-island\""));
        assert!(html.contains("Counter value: 2"));
    }

    #[tokio::test]
    async fn invalid_manifest_path_degrades_only_affected_island() {
        let root = TempDir::new().unwrap();
        write_fixture(root.path());
        let mut manifest: Value =
            serde_json::from_slice(&std::fs::read(root.path().join("manifest.json")).unwrap())
                .unwrap();
        manifest["filter"]["loader"] = Value::String("/outside.js".to_owned());
        std::fs::write(
            root.path().join("manifest.json"),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        let (_, _, html) = request(root.path(), Method::GET, "/dashboard").await;
        assert!(html.contains("counter-loader.js"));
        assert!(!html.contains("filter-loader.js"));
        assert!(html.contains("id=\"filter-query\""));
    }

    #[tokio::test]
    async fn static_assets_are_served_only_from_the_owned_asset_root() {
        let root = TempDir::new().unwrap();
        let digest = write_fixture(root.path());
        let (status, _, body) = request(
            root.path(),
            Method::GET,
            &format!("/dashboard-assets/{digest}/counter-loader.js"),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, "fixture counter-loader.js");
        assert_eq!(
            request(
                root.path(),
                Method::GET,
                "/dashboard-assets/../manifest.json"
            )
            .await
            .0,
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn loopback_and_native_routing_remain_explicit() {
        assert!(listen_address("127.0.0.1:3009").is_ok());
        assert!(listen_address("[::1]:3009").is_ok());
        assert!(listen_address("0.0.0.0:3009").is_err());
        let root = TempDir::new().unwrap();
        assert_eq!(
            request(root.path(), Method::GET, "/favicon.ico").await.0,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            request(root.path(), Method::GET, "/missing").await.0,
            StatusCode::NOT_FOUND
        );
        let (status, headers, _) = request(root.path(), Method::POST, "/dashboard").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert!(headers.contains_key(header::ALLOW));
    }
}
