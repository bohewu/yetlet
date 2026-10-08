//! Axum HTTP response integration. Yetlet deliberately does not reimplement HTTP.

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

/// Build a complete HTML document without inline scripts or external runtime.
pub fn render_document(title: &str, view: &View) -> Result<String, RenderError> {
    let mut html = String::from(
        "<!doctype html><html lang=\"zh-Hant\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>",
    );
    html.push_str(&escape_text(title));
    html.push_str("</title></head><body>");
    html.push_str(&render_to_string(view)?);
    html.push_str("</body></html>");
    Ok(html)
}

/// Turn the view into a normal HTTP response. No websocket/circuit is created.
pub fn render_page(title: &str, view: &View) -> Result<Response, HtmlPageError> {
    let body = render_document(title, view)?;
    let mut response = Html(body).into_response();
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(DEFAULT_CSP),
    );
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_CSP, render_document, render_page};
    use axum::http::header;
    use yetlet_core::{Element, View};

    #[test]
    fn document_has_no_forced_browser_runtime() {
        let view: View = Element::new("main").child(View::text("<safe>")).into();
        let document = render_document("Page <title>", &view).unwrap();

        assert!(document.starts_with("<!doctype html>"));
        assert!(document.contains("<title>Page &lt;title&gt;</title>"));
        assert!(document.contains("<main>&lt;safe&gt;</main>"));
        assert!(!document.contains("<script"));
    }

    #[test]
    fn response_emits_strict_csp() {
        let response = render_page("Todo", &View::text("ok")).unwrap();
        assert_eq!(
            response.headers()[header::CONTENT_SECURITY_POLICY],
            DEFAULT_CSP
        );
        assert!(!DEFAULT_CSP.contains("unsafe-inline"));
    }
}
