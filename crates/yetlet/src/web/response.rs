//! Native HTTP results, with an optional SSR Page when HTML is enabled.
//!
//! JSON, redirects, status/header tuples and Result<T, E> use Axum directly.
//! No ActionResult hierarchy or exception-to-status mapping is introduced.

#[cfg(feature = "html")]
use crate::view::View;
#[cfg(feature = "html")]
use axum::http::{HeaderValue, header};
pub use axum::{
    Json,
    response::{IntoResponse, Redirect, Response},
};
#[cfg(feature = "html")]
use std::borrow::Cow;

#[cfg(feature = "html")]
use crate::web::{DEFAULT_CSP, DocumentMetadata, render_page_with_metadata};

/// An owned SSR page rendered only when converted into an HTTP response.
///
/// Text, title and attributes retain the existing escaping rules. A render
/// failure returns the existing generic 500 response without internal details.
/// The default strict CSP applies on both the success and render-error paths.
///
/// ```
/// use yetlet::view::View;
/// use yetlet::web::{App, Page};
///
/// async fn home() -> Page {
///     Page::new("Home", View::text("Hello"))
/// }
/// let app: axum::Router = App::new().get("/", home).into_router();
/// ```
#[cfg(feature = "html")]
#[derive(Debug)]
pub struct Page {
    metadata: DocumentMetadata,
    view: View,
}

#[cfg(feature = "html")]
impl Page {
    /// Create a page with the legacy `zh-Hant` root language.
    pub fn new(title: impl Into<Cow<'static, str>>, view: impl Into<View>) -> Self {
        let title: Cow<'static, str> = title.into();
        Self::with_metadata(DocumentMetadata::new(title.into_owned()), view)
    }

    /// Create a page with explicitly owned, bounded document metadata.
    pub fn with_metadata(metadata: DocumentMetadata, view: impl Into<View>) -> Self {
        Self {
            metadata,
            view: view.into(),
        }
    }
}

#[cfg(feature = "html")]
impl IntoResponse for Page {
    fn into_response(self) -> Response {
        let mut response = render_page_with_metadata(&self.metadata, &self.view)
            .unwrap_or_else(IntoResponse::into_response);
        response.headers_mut().insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(DEFAULT_CSP),
        );
        response
    }
}
