//! Application-owned static CSS; no compiler, file lookup or network on requests.
use axum::{
    Extension, Router,
    body::Bytes,
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use sha2::{Digest, Sha256};
use std::{fs::File, io, io::Read, path::Path, sync::Arc};
use yetlet::view::View;
use yetlet::server_axum::{DEFAULT_CSP, render_document};

const MAX_CSS: u64 = 1024 * 1024;
const BANNER: &str = "/*! tailwindcss v4.3.3 | MIT License | https://tailwindcss.com */";
const BUILD_ERROR: &str =
    "Build dist/tailwind.css explicitly with the pinned Tailwind CSS 4.3.3 CLI; see README";

pub struct Stylesheet {
    path: String,
    bytes: Bytes,
}

impl Stylesheet {
    pub fn load() -> io::Result<Arc<Self>> {
        // A fixed application-root path, independent of runtime cwd or request.
        Self::from_file(&Path::new(env!("CARGO_MANIFEST_DIR")).join("dist/tailwind.css"))
    }

    fn from_file(path: &Path) -> io::Result<Arc<Self>> {
        let invalid = || io::Error::new(io::ErrorKind::InvalidData, BUILD_ERROR);
        if !path.symlink_metadata().map_err(|_| invalid())?.is_file() {
            return Err(invalid());
        }
        let file = File::open(path).map_err(|_| invalid())?;
        if !file.metadata()?.is_file() {
            return Err(invalid());
        }
        let mut bytes = Vec::new();
        file.take(MAX_CSS + 1).read_to_end(&mut bytes)?;
        Self::from_bytes(bytes)
    }

    fn from_bytes(bytes: Vec<u8>) -> io::Result<Arc<Self>> {
        // This identifies the pinned compiler output, not a signature or CSS
        // sanitizer. Only serve developer-built, trusted application assets.
        let valid = bytes.len() as u64 <= MAX_CSS
            && std::str::from_utf8(&bytes)
                .is_ok_and(|css| css.starts_with(BANNER) && !css[BANNER.len()..].trim().is_empty());
        if !valid {
            return Err(io::Error::new(io::ErrorKind::InvalidData, BUILD_ERROR));
        }
        Ok(Arc::new(Self {
            path: format!("/assets/tailwind.{:x}.css", Sha256::digest(&bytes)),
            bytes: Bytes::from(bytes),
        }))
    }

    pub fn page(&self, title: &str, view: View) -> Response {
        let mut response = match render_document(title, &view) {
            Ok(document) => {
                // Title/body are already escaped by Yetlet. The only inserted
                // value is our fixed prefix + lowercase SHA-256 + fixed suffix.
                let link = format!("<link rel=\"stylesheet\" href=\"{}\"></head>", self.path);
                Html(document.replacen("</head>", &link, 1)).into_response()
            }
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to render the page",
            )
                .into_response(),
        };
        response.headers_mut().insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(DEFAULT_CSP),
        );
        response
            .headers_mut()
            .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        response
    }

    fn response(&self) -> Response {
        (
            [
                (header::CONTENT_TYPE, "text/css; charset=utf-8"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                (header::CONTENT_SECURITY_POLICY, DEFAULT_CSP),
            ],
            self.bytes.clone(),
        )
            .into_response()
    }
}

pub fn install(router: Router, stylesheet: Arc<Stylesheet>) -> Router {
    let asset = stylesheet.clone();
    router
        .route(
            &stylesheet.path,
            get(move || {
                let asset = asset.clone();
                async move { asset.response() }
            }),
        )
        .layer(Extension(stylesheet))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    fn compiled() -> Arc<Stylesheet> {
        Stylesheet::from_bytes(format!("{BANNER}\n.p-8{{padding:2rem}}").into_bytes()).unwrap()
    }

    #[test]
    fn missing_uncompiled_wrong_version_empty_and_oversize_fail_closed() {
        assert!(Stylesheet::from_file(Path::new("dist/missing-tailwind.css")).is_err());
        for bytes in [
            Vec::new(),
            b"@import \"tailwindcss\";".to_vec(),
            format!("{}\na{{}}", BANNER.replace("4.3.3", "4.0.0")).into_bytes(),
            BANNER.as_bytes().to_vec(),
            vec![b'x'; MAX_CSS as usize + 1],
            vec![255],
        ] {
            assert!(Stylesheet::from_bytes(bytes).is_err());
        }
    }

    #[tokio::test]
    async fn digest_route_headers_and_snapshot_are_bound_to_bytes() {
        let stylesheet = compiled();
        let expected = format!("{BANNER}\n.p-8{{padding:2rem}}");
        assert_eq!(
            stylesheet.path,
            format!(
                "/assets/tailwind.{:x}.css",
                Sha256::digest(expected.as_bytes())
            )
        );
        let other = Stylesheet::from_bytes(format!("{expected}\n").into_bytes()).unwrap();
        assert_ne!(stylesheet.path, other.path);
        let router = install(Router::new(), stylesheet.clone());
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&stylesheet.path)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
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
            response.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        assert_eq!(
            axum::body::to_bytes(response.into_body(), MAX_CSS as usize)
                .await
                .unwrap(),
            expected
        );
        for path in [
            "/assets/tailwind.css",
            &other.path,
            "/assets/../tailwind.css",
        ] {
            assert_eq!(
                router
                    .clone()
                    .oneshot(Request::builder().uri(path).body(Body::empty()).unwrap())
                    .await
                    .unwrap()
                    .status(),
                StatusCode::NOT_FOUND
            );
        }
    }

    #[tokio::test]
    async fn html_link_escaping_strict_csp_and_render_error() {
        let stylesheet = compiled();
        let response = stylesheet.page("</title><script>", View::text("<img> & safe"));
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let bytes = axum::body::to_bytes(response.into_body(), 8192)
            .await
            .unwrap();
        let html = std::str::from_utf8(&bytes).unwrap();
        assert!(html.contains(&format!(
            "<link rel=\"stylesheet\" href=\"{}\">",
            stylesheet.path
        )));
        assert!(html.contains("&lt;/title&gt;&lt;script&gt;"));
        assert!(html.contains("&lt;img&gt; &amp; safe"));
        assert!(!html.contains("<script") && !html.contains("<style") && !html.contains(" style="));
        let response = stylesheet.page("error", yetlet::view::Element::new("script").into());
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
    }
}
