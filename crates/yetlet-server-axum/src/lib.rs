//! Axum HTTP response integration. Yetlet deliberately does not reimplement HTTP.

mod app;
mod context;
mod document;
pub mod middleware;
pub mod response;
pub use app::App;
pub use axum::http;
pub use context::RequestContext;
pub use document::{DocumentMetadata, LanguageTag, LanguageTagError};
pub use response::Page;

/// Native Axum extractors; no separate Yetlet model-binding implementation.
pub mod extract {
    pub use axum::{
        Extension, Json,
        extract::{DefaultBodyLimit, Path, Query, State},
    };
}

use axum::{
    http::{HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use yetlet_core::View;
use yetlet_html::{RenderError, escape_text, render_to_string};

/// Strict by default; browser island assets may be loaded from our own origin.
pub const DEFAULT_CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";

#[derive(Debug)]
pub struct HtmlPageError(pub RenderError);

impl From<RenderError> for HtmlPageError {
    fn from(error: RenderError) -> Self {
        Self(error)
    }
}

impl IntoResponse for HtmlPageError {
    fn into_response(self) -> Response {
        // A rendered error should not leak application or template details.
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to render the page",
        )
            .into_response()
    }
}

/// Build a complete HTML document with the legacy default language.
///
/// Existing callers remain source-compatible. Use
/// [`render_document_with_metadata`] when the root language is explicit.
pub fn render_document(title: &str, view: &View) -> Result<String, RenderError> {
    render_document_with_metadata(&DocumentMetadata::new(title), view)
}

/// Build a complete HTML document from bounded title/language metadata.
pub fn render_document_with_metadata(
    metadata: &DocumentMetadata,
    view: &View,
) -> Result<String, RenderError> {
    let mut html = String::from("<!doctype html><html lang=\"");
    html.push_str(metadata.language().as_str());
    html.push_str("\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>");
    html.push_str(&escape_text(metadata.title()));
    html.push_str("</title></head><body>");
    html.push_str(&render_to_string(view)?);
    html.push_str("</body></html>");
    Ok(html)
}

/// Turn a view into a normal HTTP response using the legacy metadata default.
pub fn render_page(title: &str, view: &View) -> Result<Response, HtmlPageError> {
    render_page_with_metadata(&DocumentMetadata::new(title), view)
}

/// Turn a view and bounded document metadata into a normal HTTP response.
pub fn render_page_with_metadata(
    metadata: &DocumentMetadata,
    view: &View,
) -> Result<Response, HtmlPageError> {
    let body = render_document_with_metadata(metadata, view)?;
    let mut response = Html(body).into_response();
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(DEFAULT_CSP),
    );
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_CSP, DocumentMetadata, LanguageTag, render_document, render_document_with_metadata,
        render_page, render_page_with_metadata,
    };
    use axum::http::header;
    use yetlet_core::{Element, View};

    #[test]
    fn document_has_no_forced_browser_runtime() {
        let view: View = Element::new("main").child(View::text("<safe>")).into();
        let document = render_document("Page <title>", &view).unwrap();

        assert!(document.starts_with("<!doctype html><html lang=\"zh-Hant\">"));
        assert!(document.contains("<title>Page &lt;title&gt;</title>"));
        assert!(document.contains("<main>&lt;safe&gt;</main>"));
        assert!(!document.contains("<script"));
    }

    #[test]
    fn explicit_metadata_owns_the_root_language_only() {
        let metadata =
            DocumentMetadata::with_language("Page <title>", LanguageTag::new("en").unwrap());
        let document = render_document_with_metadata(&metadata, &View::text("<safe>")).unwrap();
        assert!(document.starts_with("<!doctype html><html lang=\"en\">"));
        assert!(document.contains("<title>Page &lt;title&gt;</title>"));
        assert!(!document.contains("<script"));
        assert!(!document.contains("unsafe-inline"));
    }

    #[test]
    fn response_paths_emit_the_same_strict_csp() {
        let legacy = render_page("Todo", &View::text("ok")).unwrap();
        let metadata = DocumentMetadata::with_language("Todo", LanguageTag::new("en").unwrap());
        let explicit = render_page_with_metadata(&metadata, &View::text("ok")).unwrap();
        assert_eq!(
            legacy.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        assert_eq!(
            explicit.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        assert!(!DEFAULT_CSP.contains("unsafe-inline"));
    }
}
