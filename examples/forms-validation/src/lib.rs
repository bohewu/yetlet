//! SSR-only accessible form validation and stateless POST/Redirect/GET.
//! The sample has no persistence, credentials, CSRF capability, JS or WASM.

use axum::{
    Form, Router,
    extract::{DefaultBodyLimit, rejection::FormRejection},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use yetlet_core::{Element, View};
use yetlet_server_axum::{App, Page};

const MAX_FORM_BYTES: usize = 2 * 1024;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    name: String,
    email: String,
}

#[derive(Default)]
struct Validation {
    name: Option<&'static str>,
    email: Option<&'static str>,
}

impl Validation {
    fn has_errors(&self) -> bool {
        self.name.is_some() || self.email.is_some()
    }
}

fn validate(input: &Input) -> Validation {
    let mut errors = Validation::default();
    let name = input.name.trim();
    if !(2..=40).contains(&name.chars().count()) || name.chars().any(char::is_control) {
        errors.name = Some("Enter a name between 2 and 40 characters.");
    }
    let email = input.email.trim();
    let expected_shape = email
        .split_once('@')
        .map(|(local, domain)| {
            !local.is_empty() && domain.contains('.') && !domain.starts_with('.')
        })
        .unwrap_or(false);
    if email.len() > 120
        || email.chars().any(char::is_whitespace)
        || email.bytes().filter(|byte| *byte == b'@').count() != 1
        || !expected_shape
    {
        errors.email = Some("Enter a valid example email address.");
    }
    errors
}

fn field(
    id: &'static str,
    label: &'static str,
    input_type: &'static str,
    value: &str,
    error: Option<&'static str>,
    focus: bool,
) -> View {
    let mut input = Element::new("input")
        .attribute("id", id)
        .attribute("name", id)
        .attribute("type", input_type)
        .attribute("value", value)
        .attribute("required", "");
    if error.is_some() {
        input = input
            .attribute("aria-invalid", "true")
            .attribute("aria-describedby", format!("{id}-error"));
        if focus {
            input = input.attribute("autofocus", "");
        }
    }
    let mut container = Element::new("div")
        .child(
            Element::new("label")
                .attribute("for", id)
                .child(View::text(label)),
        )
        .child(input);
    if let Some(message) = error {
        container = container.child(
            Element::new("p")
                .attribute("id", format!("{id}-error"))
                .attribute("role", "alert")
                .child(View::text(message)),
        );
    }
    container.into()
}

fn form_view(name: &str, email: &str, errors: &Validation) -> View {
    let mut form = Element::new("form")
        .attribute("method", "post")
        .attribute("action", "/form/submit")
        // Demonstrate the server's validation instead of browser-side blocking.
        .attribute("novalidate", "");
    if errors.has_errors() {
        form = form.child(
            Element::new("p")
                .attribute("role", "alert")
                .child(View::text("Correct the highlighted fields.")),
        );
    }
    form = form
        .child(field("name", "Your name", "text", name, errors.name, true))
        .child(field(
            "email",
            "Email address",
            "email",
            email,
            errors.email,
            errors.name.is_none(),
        ))
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text("Validate (no data saved)")),
        );
    Element::new("main")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text("Yetlet Stateless Form")))
        .child(Element::new("p").child(View::text(
            "This demonstration validates input but never saves or authenticates a user.",
        )))
        .child(form)
        .into()
}

fn no_store(value: impl IntoResponse) -> Response {
    let mut response = value.into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
}

async fn form_page() -> Response {
    no_store(Page::new(
        "Yetlet | Stateless Form",
        form_view("", "", &Validation::default()),
    ))
}

async fn submit(form: Result<Form<Input>, FormRejection>) -> Response {
    let Form(input) = match form {
        Ok(form) => form,
        Err(error) => return no_store(error.into_response()),
    };
    let errors = validate(&input);
    if errors.has_errors() {
        return no_store((
            StatusCode::UNPROCESSABLE_ENTITY,
            Page::new(
                "Yetlet | Form errors",
                form_view(&input.name, &input.email, &errors),
            ),
        ));
    }
    // No side effect or session state. The GET success page contains no echo
    // of the submission, so refresh/back never repeats or exposes a mutation.
    no_store(Redirect::to("/form/success"))
}

async fn success() -> Response {
    no_store(Page::new(
        "Yetlet | Form accepted",
        Element::new("main")
            .attribute("lang", "en")
            .child(Element::new("h1").child(View::text("Form accepted")))
            .child(Element::new("p").child(View::text("No data was saved."))),
    ))
}

/// The form is intentionally stateless. State-changing credentialed forms must
/// select and verify an appropriate CSRF/auth boundary separately.
pub fn app() -> Router {
    App::new()
        .get("/form", form_page)
        .post("/form/submit", submit)
        .get("/form/success", success)
        .route_layer(DefaultBodyLimit::max(MAX_FORM_BYTES))
        .into_router()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{HeaderMap, Method, Request},
    };
    use tower::ServiceExt;

    async fn send(
        method: Method,
        uri: &str,
        payload: Option<&str>,
        media_type: Option<&str>,
    ) -> (StatusCode, HeaderMap, String) {
        let mut builder = Request::builder().method(method).uri(uri);
        if let Some(media) = media_type {
            builder = builder.header(header::CONTENT_TYPE, media);
        }
        let response = app()
            .oneshot(
                builder
                    .body(Body::from(payload.unwrap_or("").to_owned()))
                    .unwrap(),
            )
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = to_bytes(response.into_body(), 16 * 1024).await.unwrap();
        (status, headers, String::from_utf8(bytes.to_vec()).unwrap())
    }

    #[tokio::test]
    async fn get_form_is_semantic_no_script_and_no_session() {
        let (status, headers, html) = send(Method::GET, "/form", None, None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Yetlet Stateless Form"));
        assert!(html.contains("<main lang=\"en\">"));
        assert!(html.contains("for=\"name\""));
        assert!(html.contains("for=\"email\""));
        assert!(html.contains("action=\"/form/submit\""));
        assert!(!html.contains("<script"));
        assert!(
            headers[header::CONTENT_SECURITY_POLICY]
                .to_str()
                .unwrap()
                .contains("form-action 'self'")
        );
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
        assert!(!headers.contains_key(header::SET_COOKIE));
    }

    #[tokio::test]
    async fn invalid_fields_render_explicit_associated_errors_and_focus() {
        let (status, headers, html) = send(
            Method::POST,
            "/form/submit",
            Some("name=x&email=invalid"),
            Some("application/x-www-form-urlencoded"),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(html.contains("<main lang=\"en\">"));
        assert!(html.contains("name-error"));
        assert!(html.contains("email-error"));
        assert!(html.contains("aria-invalid=\"true\""));
        assert!(html.contains("autofocus"));
        assert!(html.contains("Correct the highlighted fields."));
        assert!(
            headers[header::CONTENT_SECURITY_POLICY]
                .to_str()
                .unwrap()
                .contains("default-src 'none'")
        );
        assert!(!headers.contains_key(header::SET_COOKIE));
    }

    #[tokio::test]
    async fn reflected_values_are_html_attribute_and_text_escaped() {
        let (status, _, html) = send(
            Method::POST,
            "/form/submit",
            Some("name=%3Cscript%3Ealert(1)%3C%2Fscript%3E&email=a%22%20onfocus%3Dalert(1)"),
            Some("application/x-www-form-urlencoded"),
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(html.contains("&lt;script&gt;"));
        assert!(html.contains("&quot;"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("value=\"a\" onfocus"));
    }

    #[tokio::test]
    async fn valid_submission_is_stateless_prg_and_refresh_cannot_resubmit() {
        let (status, headers, _) = send(
            Method::POST,
            "/form/submit",
            Some("name=Ada%20Lovelace&email=ada%40example.com"),
            Some("application/x-www-form-urlencoded"),
        )
        .await;
        assert_eq!(status, StatusCode::SEE_OTHER);
        assert_eq!(headers[header::LOCATION], "/form/success");
        assert_eq!(headers[header::CACHE_CONTROL], "no-store");
        assert!(!headers.contains_key(header::SET_COOKIE));
        let (status, _, html) = send(Method::GET, "/form/success", None, None).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("No data was saved."));
        assert!(html.contains("<main lang=\"en\">"));
        assert!(!html.contains("Ada Lovelace"));
        let (_, _, empty_form) = send(Method::GET, "/form", None, None).await;
        assert!(empty_form.contains("value=\"\""));
    }

    #[tokio::test]
    async fn bad_media_type_extra_fields_and_large_payload_are_rejected() {
        let (status, _, _) = send(
            Method::POST,
            "/form/submit",
            Some("name=Ada&email=a%40example.com"),
            Some("text/plain"),
        )
        .await;
        assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);

        let (status, _, _) = send(
            Method::POST,
            "/form/submit",
            Some("name=Ada&email=a%40example.com&admin=true"),
            Some("application/x-www-form-urlencoded"),
        )
        .await;
        // Axum's native Form rejection reports unrecognized/invalid fields as 422.
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);

        let oversized = format!("name={}&email=a%40example.com", "x".repeat(MAX_FORM_BYTES));
        let (status, _, _) = send(
            Method::POST,
            "/form/submit",
            Some(&oversized),
            Some("application/x-www-form-urlencoded"),
        )
        .await;
        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn method_fallbacks_are_native_and_do_not_submit_anything() {
        assert_eq!(
            send(Method::GET, "/missing", None, None).await.0,
            StatusCode::NOT_FOUND
        );
        let (status, headers, _) = send(Method::GET, "/form/submit", None, None).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert!(headers.contains_key(header::ALLOW));
    }
}
