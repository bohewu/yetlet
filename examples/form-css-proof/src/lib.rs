//! YT-070 isolated native form/CSS proof; no persistence, auth or core API expansion.
mod asset_data;
mod assets;
mod views;

use assets::Manifest;
use axum::{
    Form, Router,
    body::Body,
    extract::{DefaultBodyLimit, Path, State, rejection::FormRejection},
    http::{HeaderValue, StatusCode, header},
    middleware::{Next, from_fn},
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use yetlet::server_axum::{
    App, DEFAULT_CSP, DocumentMetadata, LanguageTag, render_document_with_metadata,
};
use yetlet::view::{Element, View};

const MANIFEST: &str = include_str!("../assets/manifest.json");
const MAX_BODY: usize = 64 * 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Draft {
    title: String,
    notes: String,
    priority: String,
    notify: Option<String>,
}
impl Default for Draft {
    fn default() -> Self {
        Self {
            title: "Inspect lab backup".into(),
            notes: "Check the latest test report.".into(),
            priority: "normal".into(),
            notify: Some("yes".into()),
        }
    }
}
#[derive(Default)]
struct Errors {
    title: bool,
    notes: bool,
    priority: bool,
    notify: bool,
}
impl Errors {
    fn any(&self) -> bool {
        self.title || self.notes || self.priority || self.notify
    }
}
fn validate(draft: &Draft) -> Errors {
    Errors {
        title: draft.title.trim().is_empty()
            || draft.title.chars().count() > 160
            || draft.title.chars().any(char::is_control),
        notes: draft.notes.chars().count() > 4000
            || draft
                .notes
                .chars()
                .any(|ch| ch.is_control() && !matches!(ch, '\r' | '\n' | '\t')),
        priority: !matches!(draft.priority.as_str(), "low" | "normal" | "high"),
        notify: draft.notify.as_deref().is_some_and(|v| v != "yes"),
    }
}
#[derive(Clone)]
struct AppState {
    manifest: Option<Manifest>,
}

fn page(state: &AppState, title: &str, content: View, enhanced: bool) -> Response {
    let metadata =
        DocumentMetadata::with_language(title, LanguageTag::new("en").expect("static language"));
    let mut html = match render_document_with_metadata(&metadata, &content) {
        Ok(html) => html,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Unable to render the page",
            )
                .into_response();
        }
    };
    if let Some(manifest) = &state.manifest {
        // Every reference was matched against a finite embedded asset table before use.
        let mut tags = String::new();
        for href in &manifest.stylesheets {
            tags.push_str(&format!("<link rel=\"stylesheet\" href=\"{href}\">"));
        }
        if enhanced {
            tags.push_str(&format!(
                "<script type=\"module\" src=\"{}\"></script>",
                manifest.enhancement
            ));
        }
        html = html.replacen("</head>", &format!("{tags}</head>"), 1);
    }
    axum::response::Html(html).into_response()
}
async fn security(request: axum::extract::Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(DEFAULT_CSP),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
        .headers_mut()
        .entry(header::CACHE_CONTROL)
        .or_insert(HeaderValue::from_static("no-store"));
    response
}
async fn native(State(state): State<AppState>) -> Response {
    page(
        &state,
        "New work item | Lab Operations",
        views::form(
            &Draft::default(),
            &Errors::default(),
            false,
            state.manifest.is_some(),
        ),
        false,
    )
}
async fn enhanced(State(state): State<AppState>) -> Response {
    page(
        &state,
        "Form preview | Lab Operations",
        views::form(
            &Draft::default(),
            &Errors::default(),
            true,
            state.manifest.is_some(),
        ),
        true,
    )
}
async fn submit(
    State(state): State<AppState>,
    form: Result<Form<Draft>, FormRejection>,
) -> Response {
    let Form(draft) = match form {
        Ok(input) => input,
        Err(error) => {
            return (error.status(), "Invalid or oversized form submission").into_response();
        }
    };
    let errors = validate(&draft);
    if errors.any() {
        let mut response = page(
            &state,
            "Correct form errors | Lab Operations",
            views::form(&draft, &errors, false, state.manifest.is_some()),
            false,
        );
        *response.status_mut() = StatusCode::UNPROCESSABLE_ENTITY;
        return response;
    }
    Redirect::to("/proof/accepted").into_response()
}
async fn accepted(State(state): State<AppState>) -> Response {
    let content = Element::new("main")
        .attribute("class", "container py-5")
        .child(Element::new("h1").child(View::text("Form validated")))
        .child(Element::new("p").child(View::text(
            "Nothing was saved. This isolated proof validates native form behavior only.",
        )))
        .child(
            Element::new("a")
                .attribute("href", "/proof/native")
                .child(View::text("Return to the form")),
        )
        .into();
    page(&state, "Form validated | Lab Operations", content, false)
}
async fn asset(Path(name): Path<String>) -> Response {
    let path = format!("{}{name}", assets::PREFIX);
    let Some(asset) = assets::find(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let mut response = Response::new(Body::from(asset.bytes));
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(asset.content_type),
    );
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    response
}
async fn health(State(state): State<AppState>) -> StatusCode {
    if state.manifest.is_some() {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}
/// Construct the proof with its checked-in, content-addressed assets.
pub fn app() -> Router {
    app_with_manifest(MANIFEST)
}
/// Sample-local negative-fixture seam, not a Yetlet library API.
pub fn app_with_manifest(manifest: &str) -> Router {
    let router = App::new()
        .get("/", || async { Redirect::to("/proof/native") })
        .get("/healthz", health)
        .get("/favicon.ico", || async { StatusCode::NO_CONTENT })
        .get("/proof/native", native)
        .get("/proof/enhanced", enhanced)
        .post("/proof/submit", submit)
        .get("/proof/accepted", accepted)
        .get("/form-proof-assets/{name}", asset)
        .map_router(|r| r.fallback(|| async { (StatusCode::NOT_FOUND, "Not found") }))
        .with_state(AppState {
            manifest: Manifest::parse(manifest).ok(),
        })
        .into_router()
        .layer(DefaultBodyLimit::max(MAX_BODY));
    yetlet::server_axum::middleware::with_global(router, from_fn(security))
}

#[cfg(test)]
mod tests;
