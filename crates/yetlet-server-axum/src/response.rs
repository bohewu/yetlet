//! Native HTTP results: a small SSR value plus unchanged Axum result types.
//!
//! JSON, redirects, status/header tuples and Result<T, E> use Axum directly.
//! No ActionResult hierarchy or exception-to-status mapping is introduced.

use std::borrow::Cow;

use axum::http::{HeaderValue, header};
pub use axum::{
    Json,
    response::{IntoResponse, Redirect, Response},
};
use yetlet_core::View;

use crate::{DEFAULT_CSP, render_page};

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
    title: Cow<'static, str>,
    view: View,
}

impl Page {
    pub fn new(title: impl Into<Cow<'static, str>>, view: impl Into<View>) -> Self {
        Self {
            title: title.into(),
            view: view.into(),
        }
    }
}

impl IntoResponse for Page {
    fn into_response(self) -> Response {
        let mut response = render_page(self.title.as_ref(), &self.view)
            .unwrap_or_else(IntoResponse::into_response);
        response.headers_mut().insert(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(DEFAULT_CSP),
        );
        response
    }
}
