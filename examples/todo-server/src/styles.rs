//! Optional build-time CSS snapshot. Neither private records nor source are routed.
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::Request,
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Arc};
use yetlet::server_axum::{DEFAULT_CSP, Page};

#[derive(Clone, Default)]
pub(crate) struct NativeCss(Option<Arc<Snapshot>>);
struct Snapshot {
    url: String,
    bytes: Vec<u8>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    stylesheet: String,
    sha256: String,
}
impl NativeCss {
    // Missing or invalid optional output leaves ordinary SSR fully usable.
    pub fn load(dist: &Path) -> Self {
        let load = || -> Option<Snapshot> {
            let record: Record =
                serde_json::from_slice(&std::fs::read(dist.join("todo-css.json")).ok()?).ok()?;
            let digest = record
                .stylesheet
                .strip_prefix("/assets/")?
                .strip_suffix("/styles.css")?;
            if digest.len() != 16
                || !digest
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return None;
            }
            let path = dist.join("assets").join(digest).join("styles.css");
            // Reject redirected assets and bound optional startup reads.
            for path in [
                dist.to_path_buf(),
                dist.join("assets"),
                dist.join("assets").join(digest),
                path.clone(),
            ] {
                if std::fs::symlink_metadata(path)
                    .ok()?
                    .file_type()
                    .is_symlink()
                {
                    return None;
                }
            }
            let bytes = std::fs::read(path).ok()?;
            if bytes.is_empty()
                || bytes.len() > 2 * 1024 * 1024
                || std::str::from_utf8(&bytes).is_err()
                || format!("{:x}", Sha256::digest(&bytes)) != record.sha256
            {
                return None;
            }
            Some(Snapshot {
                url: record.stylesheet,
                bytes,
            })
        };
        Self(load().map(Arc::new))
    }
    pub fn router<S: Clone + Send + Sync + 'static>(&self) -> Router<S> {
        let mut router = Router::new();
        if let Some(css) = &self.0 {
            let snapshot = css.clone();
            router = router.route(
                &css.url,
                get(move || {
                    let snapshot = snapshot.clone();
                    async move {
                        (
                            [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
                            snapshot.bytes.clone(),
                        )
                    }
                }),
            );
        }
        router
    }
    pub async fn page(&self, page: Page) -> Response {
        let response = page.into_response();
        let Some(css) = &self.0 else {
            return response;
        };
        let (mut parts, body) = response.into_parts();
        let Ok(body) = to_bytes(body, 1024 * 1024).await else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let Ok(html) = std::str::from_utf8(&body) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let html = html.replacen(
            "</head>",
            &format!("<link rel=\"stylesheet\" href=\"{}\"><link rel=\"icon\" type=\"image/svg+xml\" href=\"{}\"></head>", css.url, css.url.replace("/styles.css", "/favicon.svg")),
            1,
        );
        parts.headers.remove(header::CONTENT_LENGTH);
        Response::from_parts(parts, Body::from(html))
    }
}
pub(crate) async fn asset_headers(request: Request, next: Next) -> Response {
    let css =
        request.uri().path().starts_with("/assets/") && request.uri().path().ends_with(".css");
    let mut response = next.run(request).await;
    if css {
        for (key, value) in [
            (header::CONTENT_SECURITY_POLICY, DEFAULT_CSP),
            (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
            (header::CACHE_CONTROL, "no-store"),
        ] {
            response
                .headers_mut()
                .insert(key, HeaderValue::from_static(value));
        }
    }
    response
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn optional_record_refuses_missing_traversal_and_digest_mismatch() {
        let dir = std::env::temp_dir().join(format!("yetlet-todo-css-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("assets/0123456789abcdef")).unwrap();
        assert!(NativeCss::load(&dir).0.is_none());
        let bytes = b".todo-shell { color: navy; }";
        let path = dir.join("assets/0123456789abcdef/styles.css");
        std::fs::write(&path, bytes).unwrap();
        let hash = format!("{:x}", Sha256::digest(bytes));
        for (url, hash, expected) in [
            ("/assets/0123456789abcdef/styles.css", hash.as_str(), true),
            ("/assets/0123456789abcdef/styles.css", "bad", false),
            ("/assets/../../private/styles.css", hash.as_str(), false),
        ] {
            std::fs::write(
                dir.join("todo-css.json"),
                serde_json::to_vec(&serde_json::json!({"stylesheet":url,"sha256":hash})).unwrap(),
            )
            .unwrap();
            assert_eq!(NativeCss::load(&dir).0.is_some(), expected);
        }
        std::fs::remove_dir_all(dir).unwrap();
    }
}
