//! Native HTTP results: a small SSR value plus unchanged Axum result types.
//!
//! JSON, redirects, status/header tuples and Result<T, E> use Axum directly.
//! No ActionResult hierarchy or exception-to-status mapping is introduced.

use axum::http::{HeaderValue, header};
pub use axum::{
    Json,
    response::{IntoResponse, Redirect, Response},
};
use std::borrow::Cow;
use yetlet_core::View;

use crate::{DEFAULT_CSP, DocumentMetadata, render_page_with_metadata};

/// An owned SSR page rendered only when converted into an HTTP response.
///
/// Text, title and attributes retain the existing escaping rules. A render
/// failure returns the existing generic 500 response without internal details.
/// The default strict CSP applies on both the success and render-error paths.
///
/// ```
/// use yetlet_core::View;
/// use yetlet_server_axum::{App, Page};
///
/// async fn home() -> Page {
///     Page::new("Home", View::text("Hello"))
/// }
/// let app: axum::Router = App::new().get("/", home).into_router();
/// ```
#[derive(Debug)]
pub struct Page {
    metadata: DocumentMetadata,
    view: View,
}

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
