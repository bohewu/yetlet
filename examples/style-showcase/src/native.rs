//! App-owned HTTP composition. All CSS is immutable build output.
use crate::{CSS, CSS_SHA256, CSS_URL, SOURCE_SHA256, Selection, gallery};
use axum::{
    Router,
    extract::{Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
    routing::get,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Arc};
use yetlet::server_axum::{
    DEFAULT_CSP, DocumentMetadata, LanguageTag, render_document_with_metadata,
};

// Exact existing opt-in policy from todo-server; no additional exception.
pub const BROWSER_CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const FILES: [(&str, &str); 3] = [
    ("loader.js", "text/javascript; charset=utf-8"),
    ("showcase.js", "text/javascript; charset=utf-8"),
    ("showcase_bg.wasm", "application/wasm"),
];

#[derive(Clone)]
struct BrowserAssets {
    digest: String,
    files: Vec<Vec<u8>>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    digest: String,
    css_sha256: String,
    source_sha256: String,
}
impl BrowserAssets {
    fn load(dist: &Path) -> Option<Self> {
        let record_path = dist.join("manifest.json");
        if std::fs::metadata(&record_path).ok()?.len() > 4096 {
            return None;
        }
        let record: Record = serde_json::from_slice(&std::fs::read(record_path).ok()?).ok()?;
        if record.css_sha256 != CSS_SHA256
            || record.source_sha256 != SOURCE_SHA256
            || record.digest.len() != 16
            || !record
                .digest
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return None;
        }
        let folder = dist.join("assets").join(&record.digest);
        // Snapshot only the three known public files; never serve the source tree.
        let mut hash = Sha256::new();
        hash.update(b"yetlet-showcase-v1\0");
        let mut files = Vec::new();
        for (name, _) in FILES {
            let path = folder.join(name);
            if std::fs::metadata(&path).ok()?.len() > 8 * 1024 * 1024 {
                return None;
            }
            let bytes = std::fs::read(path).ok()?;
            hash.update(name.as_bytes());
            hash.update([0]);
            hash.update(&bytes);
            hash.update([0]);
            files.push(bytes);
        }
        if format!("{:x}", hash.finalize())[..16] != record.digest {
            return None;
        }
        Some(Self {
            digest: record.digest,
            files,
        })
    }
}

#[derive(Clone, Default)]
struct AppState(Option<Arc<BrowserAssets>>);
#[derive(Deserialize, Default)]
struct Choices {
    #[serde(default)]
    theme: String,
    #[serde(default)]
    density: String,
    #[serde(default)]
    selected: String,
}
fn headers(mut response: Response, browser: bool) -> Response {
    for (key, value) in [
        (
            header::CONTENT_SECURITY_POLICY,
            if browser { BROWSER_CSP } else { DEFAULT_CSP },
        ),
        (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
        (header::CACHE_CONTROL, "no-store"),
    ] {
        response
            .headers_mut()
            .insert(key, HeaderValue::from_static(value));
    }
    response
}

async fn native(Query(choices): Query<Choices>) -> Response {
    page(choices, false, &AppState::default())
}
async fn interactive(State(state): State<AppState>, Query(choices): Query<Choices>) -> Response {
    page(choices, true, &state)
}
fn page(choices: Choices, interactive: bool, state: &AppState) -> Response {
    let selection = Selection::from_values(&choices.theme, &choices.density, &choices.selected);
    let metadata =
        DocumentMetadata::with_language("Yetlet · Style Showcase", LanguageTag::new("en").unwrap());
    let view = gallery(selection, interactive, state.0.is_some());
    let Ok(html) = render_document_with_metadata(&metadata, &view) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };
    let script = if interactive {
        state
            .0
            .as_ref()
            .map(|assets| {
                format!(
                    "<script type=\"module\" src=\"/showcase-assets/{}/loader.js\"></script>",
                    assets.digest
                )
            })
            .unwrap_or_default()
    } else {
        String::new()
    };
    let html = html.replacen(
        "</head>",
        &format!("<link rel=\"stylesheet\" href=\"{CSS_URL}\">{script}</head>"),
        1,
    );
    headers(Html(html).into_response(), interactive && state.0.is_some())
}

/// Browser output is optional startup input; requests never read or compile it.
pub fn app(dist: &Path) -> Router {
    let assets = BrowserAssets::load(dist).map(Arc::new);
    let mut app = Router::new()
        .route("/", get(|| async { Redirect::to("/showcase") }))
        .route("/healthz", get(|| async { "ok" }))
        .route("/favicon.ico", get(|| async { StatusCode::NO_CONTENT }))
        .route("/showcase", get(native))
        .route("/showcase/interactive", get(interactive))
        .route(
            CSS_URL,
            get(|| async {
                headers(
                    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], CSS).into_response(),
                    false,
                )
            }),
        );
    if let Some(assets) = &assets {
        for (index, (name, mime)) in FILES.into_iter().enumerate() {
            let bytes = assets.files[index].clone();
            app = app.route(
                &format!("/showcase-assets/{}/{name}", assets.digest),
                get(move || {
                    let bytes = bytes.clone();
                    async move {
                        headers(
                            ([(header::CONTENT_TYPE, mime)], bytes).into_response(),
                            false,
                        )
                    }
                }),
            );
        }
    }
    app.with_state(AppState(assets))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tower::ServiceExt;

    async fn fetch(uri: &str) -> Response {
        app(Path::new("target/absent-yt241-browser-output"))
            .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }
    #[tokio::test]
    async fn native_and_missing_browser_output_are_complete_without_scripts() {
        for uri in ["/showcase", "/showcase/interactive"] {
            let response = fetch(uri).await;
            assert_eq!(response.status(), StatusCode::OK);
            assert_eq!(
                response.headers()[header::CONTENT_SECURITY_POLICY],
                DEFAULT_CSP
            );
            let html = String::from_utf8(
                to_bytes(response.into_body(), 1024 * 1024)
                    .await
                    .unwrap()
                    .to_vec(),
            )
            .unwrap();
            assert!(html.contains("<html lang=\"en\">"));
            assert_eq!(html.matches("rel=\"stylesheet\"").count(), 1);
            assert!(html.contains(CSS_URL));
            assert!(html.contains("<details"));
            assert!(!html.contains("<script"));
        }
    }
    #[tokio::test]
    async fn stylesheet_is_exact_build_output_with_strict_headers_and_no_source_routes() {
        let response = fetch(CSS_URL).await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[header::CONTENT_TYPE],
            "text/css; charset=utf-8"
        );
        assert_eq!(
            response.headers()[header::X_CONTENT_TYPE_OPTIONS],
            "nosniff"
        );
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        assert_eq!(bytes.as_ref(), CSS.as_bytes());
        assert_eq!(format!("{:x}", Sha256::digest(bytes)), CSS_SHA256);
        for path in [
            "/showcase-assets/manifest.json",
            "/styles/app.css",
            "/src/lib.rs",
            "/showcase-assets/unknown/showcase.css",
        ] {
            assert_eq!(fetch(path).await.status(), StatusCode::NOT_FOUND);
        }
    }
}
