//! Local operator SSR and a separate bearer-protected command API.
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use crate::observability::{Active, Metrics};
use axum::{
    Form, Json, Router,
    extract::{
        DefaultBodyLimit, Extension, Path, Query, Request, State,
        rejection::{FormRejection, JsonRejection, PathRejection, QueryRejection},
    },
    http::{HeaderValue, StatusCode, header},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Redirect, Response},
    routing::get,
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tracing::Instrument;
use yetlet::view::{Element, View};
use yetlet::request_id::{RequestId, RequestIds};
use yetlet::server_axum::{App, DocumentMetadata, LanguageTag, Page, middleware::with_global};
use yetlet::session::{Session, SessionConfig, SessionManager};

use crate::store::{Issue, IssueStatus, Store, StoreError};

const BODY_LIMIT: usize = 16 * 1024;

/// Application policy; existing `app` consumers receive these bounded defaults.
#[derive(Clone, Copy)]
pub struct Admission {
    pub max_requests: usize,
    pub request_timeout: Duration,
}
impl Default for Admission {
    fn default() -> Self {
        Self {
            max_requests: 32,
            request_timeout: Duration::from_secs(5),
        }
    }
}

#[derive(Clone)]
struct RequestAdmission {
    gate: Arc<tokio::sync::Semaphore>,
    store: Store,
    timeout: Duration,
}

#[derive(Clone)]
struct WebState {
    store: Store,
    token: Arc<SecretString>,
    readiness: Arc<AtomicBool>,
}

/// Anonymous SSR sessions supply CSRF protection, not operator authentication.
/// The caller must enforce a loopback listener for these local HTTP cookies.
pub fn app(
    store: Store,
    api_token: SecretString,
    readiness: Arc<AtomicBool>,
) -> Result<Router, &'static str> {
    app_with_admission(store, api_token, readiness, Admission::default())
}

pub fn app_with_admission(
    store: Store,
    api_token: SecretString,
    readiness: Arc<AtomicBool>,
    admission: Admission,
) -> Result<Router, &'static str> {
    if admission.max_requests == 0
        || admission.max_requests > 1024
        || admission.request_timeout.is_zero()
        || admission.request_timeout > Duration::from_secs(60)
    {
        return Err("Request admission configuration is invalid");
    }
    let requests = RequestAdmission {
        gate: Arc::new(tokio::sync::Semaphore::new(admission.max_requests)),
        store: store.clone(),
        timeout: admission.request_timeout,
    };
    let token = api_token.expose_secret();
    if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("API token must contain 64 hexadecimal characters");
    }
    let state = WebState {
        store,
        token: Arc::new(api_token),
        readiness,
    };
    let sessions = SessionManager::<()>::new(
        SessionConfig::new("yetlet-starter", Duration::from_secs(1800), 1024)
            .map_err(|_| "Session configuration is invalid")?
            .insecure_for_local_http(),
    );
    let browser = sessions
        .scoped(
            App::<WebState>::new()
                .get("/", board)
                .get("/issues/{id}", detail)
                .post("/issues", create_form)
                .post("/issues/{id}/status", status_form),
        )
        .into_router();
    let api = Router::new()
        .route("/v2/issues", get(api_list_by_status))
        .route("/issues", get(api_list).post(api_create))
        .route("/issues/{id}", get(api_get).patch(api_status))
        .fallback(|| async { api_error(StatusCode::NOT_FOUND, "Route not found") })
        .method_not_allowed_fallback(|| async {
            api_error(StatusCode::METHOD_NOT_ALLOWED, "Method not allowed")
        })
        .layer(from_fn_with_state(state.clone(), authorize_api));
    let metrics = state.store.metrics().cloned();
    let mut router = browser
        .nest("/api", api)
        .fallback(|| async { (StatusCode::NOT_FOUND, "Route not found") })
        .method_not_allowed_fallback(|| async {
            (StatusCode::METHOD_NOT_ALLOWED, "Method not allowed")
        })
        .layer(DefaultBodyLimit::max(BODY_LIMIT))
        .layer(from_fn_with_state(requests, admit))
        // Only existing health routes bypass HTTP admission. Readiness still
        // uses bounded SQL admission; liveness performs no SQL.
        .route("/health/live", get(|| async { "live" }))
        .route("/health/ready", get(ready));
    if let Some(metrics) = metrics.clone() {
        // Collection must remain reachable during application/SQL saturation.
        // This route is loopback-only under the same listener contract as health.
        router = router.route(
            "/metrics",
            get(move || {
                let metrics = metrics.clone();
                async move {
                    match metrics.encode() {
                        Ok(bytes) => (
                            [(
                                header::CONTENT_TYPE,
                                "text/plain; version=0.0.4; charset=utf-8",
                            )],
                            bytes,
                        )
                            .into_response(),
                        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Metrics unavailable")
                            .into_response(),
                    }
                }
            }),
        );
    }
    let router = router.with_state(state);
    let router = with_global(router, from_fn_with_state(metrics, observe));
    Ok(RequestIds::new().global(App::from_router(router)))
}

async fn admit(
    State(admission): State<RequestAdmission>,
    request: Request,
    next: Next,
) -> Response {
    let api = request.uri().path() == "/api" || request.uri().path().starts_with("/api/");
    let error = |status, message| {
        if api {
            api_error(status, message)
        } else {
            text_error(status, message)
        }
    };
    let reject = |reason| {
        if let Some(metrics) = admission.store.metrics() {
            metrics.http_rejections.with_label_values(&[reason]).inc();
        }
        tracing::info!(event = "http_admission_rejected", reason);
    };
    let Ok(_permit) = admission.gate.try_acquire() else {
        reject("capacity");
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Issue storage is unavailable",
        );
    };
    if admission.store.is_closed() {
        reject("shutdown");
        return error(
            StatusCode::SERVICE_UNAVAILABLE,
            "Issue storage is unavailable",
        );
    }
    match tokio::time::timeout(admission.timeout, next.run(request)).await {
        Ok(response) => response,
        Err(_) => {
            reject("deadline");
            error(
                StatusCode::GATEWAY_TIMEOUT,
                "Request deadline exceeded; a submitted write may still complete. Check state before retrying",
            )
        }
    }
}

async fn observe(
    State(metrics): State<Option<Arc<Metrics>>>,
    request: Request,
    next: Next,
) -> Response {
    let collect = metrics
        .as_ref()
        .filter(|_| request.uri().path() != "/metrics");
    let _active = collect.map(|m| Active::new(&m.http_active));
    let request_id = request.extensions().get::<RequestId>().map_or(0, |id| id.0);
    let method = match request.method().as_str() {
        "GET" => "GET",
        "POST" => "POST",
        "PATCH" => "PATCH",
        "PUT" => "PUT",
        "DELETE" => "DELETE",
        "HEAD" => "HEAD",
        "OPTIONS" => "OPTIONS",
        _ => "OTHER",
    };
    let started = Instant::now();
    let span = tracing::info_span!("http_request", request_id, method);
    let mut response = async {
        let response = next.run(request).await;
        tracing::info!(
            event = "http_request",
            request_id,
            method,
            status = response.status().as_u16(),
            duration_ms = started.elapsed().as_millis() as u64
        );
        response
    }
    .instrument(span)
    .await;
    if let Some(metrics) = collect {
        metrics
            .http_requests
            .with_label_values(&[method, response.status().as_str()])
            .inc();
        metrics
            .http_duration
            .with_label_values(&[method])
            .observe(started.elapsed().as_secs_f64());
    }
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(header::CONTENT_SECURITY_POLICY, HeaderValue::from_static(
        "default-src 'none'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'"));
    response
}

async fn authorize_api(State(state): State<WebState>, request: Request, next: Next) -> Response {
    if request.headers().contains_key(header::ORIGIN) {
        return api_error(
            StatusCode::FORBIDDEN,
            "Browser-origin API requests are not allowed",
        );
    }
    let mut values = request.headers().get_all(header::AUTHORIZATION).iter();
    let supplied = values
        .next()
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));
    let valid = values.next().is_none()
        && supplied.is_some_and(|v| {
            v.len() == 64 && bool::from(v.as_bytes().ct_eq(state.token.expose_secret().as_bytes()))
        });
    if !valid {
        return api_error(StatusCode::UNAUTHORIZED, "Valid bearer token required");
    }
    next.run(request).await
}

async fn ready(State(state): State<WebState>) -> Response {
    let ready = state.readiness.load(Ordering::Acquire)
        && tokio::time::timeout(Duration::from_secs(1), state.store.ready())
            .await
            .is_ok_and(|result| result.is_ok())
        && !state.store.is_closed()
        && state.readiness.load(Ordering::Acquire);
    if let Some(metrics) = state.store.metrics() {
        metrics.readiness.set(i64::from(ready));
        metrics
            .readiness_checks
            .with_label_values(&[if ready { "ready" } else { "not_ready" }])
            .inc();
    }
    tracing::info!(event = "readiness_checked", ready);
    if ready {
        (StatusCode::OK, "ready").into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready").into_response()
    }
}

fn store_failure(error: StoreError) -> (StatusCode, &'static str) {
    match error {
        StoreError::NotFound => (StatusCode::NOT_FOUND, "Issue not found"),
        StoreError::Conflict => (
            StatusCode::CONFLICT,
            "Issue changed; reload before trying again",
        ),
        StoreError::Unavailable => (
            StatusCode::SERVICE_UNAVAILABLE,
            "Issue storage is unavailable",
        ),
        StoreError::InvalidSchema | StoreError::Internal => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to complete the request",
        ),
    }
}
fn text_error(status: StatusCode, message: &'static str) -> Response {
    (status, message).into_response()
}
fn api_error(status: StatusCode, message: &'static str) -> Response {
    #[derive(Serialize)]
    struct ErrorBody {
        error: &'static str,
    }
    (status, Json(ErrorBody { error: message })).into_response()
}
fn session_token(session: &Session<()>) -> Result<String, (StatusCode, &'static str)> {
    session
        .csrf_token()
        .map_err(|_| (StatusCode::SERVICE_UNAVAILABLE, "Session is unavailable"))
}
fn csrf(session: &Session<()>, token: &str) -> Result<(), (StatusCode, &'static str)> {
    match session.verify_csrf(token) {
        Ok(true) => Ok(()),
        Ok(false) => Err((
            StatusCode::FORBIDDEN,
            "Invalid form token; reload and try again",
        )),
        Err(_) => Err((StatusCode::SERVICE_UNAVAILABLE, "Session is unavailable")),
    }
}
fn path_id(path: Result<Path<i64>, PathRejection>) -> Result<i64, &'static str> {
    match path {
        Ok(Path(id)) if id > 0 => Ok(id),
        _ => Err("Invalid issue ID"),
    }
}
fn validate_issue(title: &str, body: &str) -> Result<(), &'static str> {
    if title.trim().is_empty() || title.chars().count() > 120 || title.chars().any(char::is_control)
    {
        return Err("Title must contain 1 to 120 characters and no control characters");
    }
    if body.chars().count() > 4000
        || body
            .chars()
            .any(|c| c.is_control() && c != '\n' && c != '\t' && c != '\r')
    {
        return Err(
            "Body must contain at most 4000 characters and no unsupported control characters",
        );
    }
    Ok(())
}
fn parse_status(value: &str) -> Result<IssueStatus, &'static str> {
    match value {
        "open" => Ok(IssueStatus::Open),
        "in_progress" => Ok(IssueStatus::InProgress),
        "closed" => Ok(IssueStatus::Closed),
        _ => Err("Choose a valid issue status"),
    }
}
fn hidden(name: &'static str, value: impl Into<String>) -> View {
    Element::new("input")
        .attribute("type", "hidden")
        .attribute("name", name)
        .attribute("value", value)
        .into()
}
fn heading(level: &'static str, text: impl Into<String>) -> View {
    Element::new(level).child(View::text(text)).into()
}
fn alert(message: Option<&str>) -> View {
    match message {
        Some(message) => Element::new("p")
            .attribute("role", "alert")
            .child(View::text(message))
            .into(),
        None => View::text(""),
    }
}
fn board_page(issues: &[Issue], token: &str, title: &str, body: &str, error: Option<&str>) -> Page {
    let mut list = Element::new("ul");
    for issue in issues {
        list = list.child(
            Element::new("li")
                .child(
                    Element::new("a")
                        .attribute("href", format!("/issues/{}", issue.id))
                        .child(View::text(&issue.title)),
                )
                .child(View::text(format!(" — {}", issue.status.as_str()))),
        );
    }
    Page::with_metadata(DocumentMetadata::with_language("Issue Tracker", LanguageTag::new("en").expect("static language tag")), Element::new("main")
        .child(heading("h1", "Issue Tracker"))
        .child(Element::new("p").child(View::text("Local operator board. Anyone with access to this listener can view and change issues.")))
        .child(heading("h2", "Create issue"))
        .child(alert(error))
        .child(Element::new("form").attribute("method", "post").attribute("action", "/issues")
            .child(hidden("csrf", token))
            .child(Element::new("p").child(Element::new("label").attribute("for", "title").child(View::text("Title (required, up to 120 characters)"))))
            .child(Element::new("input").attribute("id", "title").attribute("name", "title").attribute("required", "required").attribute("maxlength", "120").attribute("value", title))
            .child(Element::new("p").child(Element::new("label").attribute("for", "body").child(View::text("Description (up to 4000 characters)"))))
            .child(Element::new("textarea").attribute("id", "body").attribute("name", "body").attribute("maxlength", "4000").attribute("rows", "6").child(View::text(body)))
            .child(Element::new("p").child(Element::new("button").attribute("type", "submit").child(View::text("Create issue")))))
        .child(heading("h2", "Latest issues"))
        .child(if issues.is_empty() { View::from(Element::new("p").child(View::text("No issues yet."))) } else { View::from(list) })
        .child(Element::new("p").child(View::text("Showing up to 100 newest issues."))))
}
fn detail_page(
    issue: &Issue,
    token: &str,
    selected: &str,
    version: &str,
    error: Option<&str>,
) -> Page {
    let mut options = Element::new("select")
        .attribute("id", "status")
        .attribute("name", "status");
    for (value, label) in [
        ("open", "Open"),
        ("in_progress", "In progress"),
        ("closed", "Closed"),
    ] {
        let mut option = Element::new("option")
            .attribute("value", value)
            .child(View::text(label));
        if selected == value {
            option = option.attribute("selected", "selected");
        }
        options = options.child(option);
    }
    Page::with_metadata(
        DocumentMetadata::with_language(
            "Issue details",
            LanguageTag::new("en").expect("static language tag"),
        ),
        Element::new("main")
            .child(
                Element::new("a")
                    .attribute("href", "/")
                    .child(View::text("All issues")),
            )
            .child(heading("h1", &issue.title))
            .child(Element::new("p").child(View::text(format!(
                "Issue #{} · {} · version {}",
                issue.id,
                issue.status.as_str(),
                issue.version
            ))))
            .child(Element::new("pre").child(View::text(&issue.body)))
            .child(heading("h2", "Change status"))
            .child(alert(error))
            .child(
                Element::new("form")
                    .attribute("method", "post")
                    .attribute("action", format!("/issues/{}/status", issue.id))
                    .child(hidden("csrf", token))
                    .child(hidden("expected_version", version))
                    .child(
                        Element::new("label")
                            .attribute("for", "status")
                            .child(View::text("Status")),
                    )
                    .child(options)
                    .child(
                        Element::new("button")
                            .attribute("type", "submit")
                            .child(View::text("Update status")),
                    ),
            ),
    )
}

async fn board(
    State(state): State<WebState>,
    Extension(session): Extension<Session<()>>,
) -> Response {
    let token = match session_token(&session) {
        Ok(token) => token,
        Err(response) => return response.into_response(),
    };
    match state.store.list().await {
        Ok(issues) => board_page(&issues, &token, "", "", None).into_response(),
        Err(error) => {
            let (status, message) = store_failure(error);
            text_error(status, message)
        }
    }
}
async fn detail(
    State(state): State<WebState>,
    Extension(session): Extension<Session<()>>,
    path: Result<Path<i64>, PathRejection>,
) -> Response {
    let id = match path_id(path) {
        Ok(id) => id,
        Err(error) => return text_error(StatusCode::BAD_REQUEST, error),
    };
    let token = match session_token(&session) {
        Ok(token) => token,
        Err(response) => return response.into_response(),
    };
    match state.store.get(id).await {
        Ok(issue) => detail_page(
            &issue,
            &token,
            issue.status.as_str(),
            &issue.version.to_string(),
            None,
        )
        .into_response(),
        Err(error) => {
            let (status, message) = store_failure(error);
            text_error(status, message)
        }
    }
}
#[derive(Deserialize, Default)]
struct CreateForm {
    #[serde(default)]
    csrf: String,
    #[serde(default)]
    title: String,
    #[serde(default)]
    body: String,
}
fn form_failure(error: FormRejection) -> Response {
    let status = if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        StatusCode::PAYLOAD_TOO_LARGE
    } else {
        StatusCode::BAD_REQUEST
    };
    text_error(status, "Invalid or oversized form")
}
async fn create_form(
    State(state): State<WebState>,
    Extension(session): Extension<Session<()>>,
    form: Result<Form<CreateForm>, FormRejection>,
) -> Response {
    let form = match form {
        Ok(Form(form)) => form,
        Err(error) => return form_failure(error),
    };
    if let Err(response) = csrf(&session, &form.csrf) {
        return response.into_response();
    }
    let failure = match validate_issue(&form.title, &form.body) {
        Err(error) => (StatusCode::UNPROCESSABLE_ENTITY, error),
        Ok(()) => match state
            .store
            .create(form.title.clone(), form.body.clone())
            .await
        {
            Ok(issue) => return Redirect::to(&format!("/issues/{}", issue.id)).into_response(),
            Err(error) => store_failure(error),
        },
    };
    let issues = state.store.list().await.unwrap_or_default();
    (
        failure.0,
        board_page(
            &issues,
            &form.csrf,
            &form.title,
            &form.body,
            Some(failure.1),
        ),
    )
        .into_response()
}
#[derive(Deserialize)]
struct StatusForm {
    #[serde(default)]
    csrf: String,
    #[serde(default)]
    status: String,
    #[serde(default)]
    expected_version: String,
}
async fn status_form(
    State(state): State<WebState>,
    Extension(session): Extension<Session<()>>,
    path: Result<Path<i64>, PathRejection>,
    form: Result<Form<StatusForm>, FormRejection>,
) -> Response {
    let id = match path_id(path) {
        Ok(id) => id,
        Err(error) => return text_error(StatusCode::BAD_REQUEST, error),
    };
    let form = match form {
        Ok(Form(form)) => form,
        Err(error) => return form_failure(error),
    };
    if let Err(response) = csrf(&session, &form.csrf) {
        return response.into_response();
    }
    let status = parse_status(&form.status);
    let version = form
        .expected_version
        .parse::<i64>()
        .ok()
        .filter(|v| *v >= 1);
    let failure = match (status, version) {
        (Err(message), _) => (StatusCode::UNPROCESSABLE_ENTITY, message),
        (_, None) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "Expected version must be a positive integer",
        ),
        (Ok(status), Some(version)) => match state.store.set_status(id, status, version).await {
            Ok(_) => return Redirect::to(&format!("/issues/{id}")).into_response(),
            Err(error) => store_failure(error),
        },
    };
    match state.store.get(id).await {
        Ok(issue) => (
            failure.0,
            detail_page(
                &issue,
                &form.csrf,
                &form.status,
                &form.expected_version,
                Some(failure.1),
            ),
        )
            .into_response(),
        Err(error) => {
            let (status, message) = store_failure(error);
            text_error(status, message)
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateInput {
    title: String,
    #[serde(default)]
    body: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusInput {
    status: String,
    expected_version: i64,
}
fn json_failure(error: JsonRejection) -> Response {
    let status = if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        StatusCode::PAYLOAD_TOO_LARGE
    } else {
        StatusCode::BAD_REQUEST
    };
    api_error(status, "Invalid or oversized JSON request")
}
async fn api_list(State(state): State<WebState>) -> Response {
    match state.store.list().await {
        Ok(issues) => Json(issues).into_response(),
        Err(error) => {
            let (status, message) = store_failure(error);
            api_error(status, message)
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusQuery {
    status: IssueStatus,
}

async fn api_list_by_status(
    State(state): State<WebState>,
    query: Result<Query<StatusQuery>, QueryRejection>,
) -> Response {
    let Ok(Query(query)) = query else {
        return api_error(StatusCode::BAD_REQUEST, "Invalid status query");
    };
    match state.store.list_by_status(query.status).await {
        Ok(issues) => Json(issues).into_response(),
        Err(error) => {
            let (status, message) = store_failure(error);
            api_error(status, message)
        }
    }
}
async fn api_get(
    State(state): State<WebState>,
    path: Result<Path<i64>, PathRejection>,
) -> Response {
    let id = match path_id(path) {
        Ok(id) => id,
        Err(error) => return api_error(StatusCode::BAD_REQUEST, error),
    };
    match state.store.get(id).await {
        Ok(issue) => Json(issue).into_response(),
        Err(error) => {
            let (status, message) = store_failure(error);
            api_error(status, message)
        }
    }
}
async fn api_create(
    State(state): State<WebState>,
    input: Result<Json<CreateInput>, JsonRejection>,
) -> Response {
    let input = match input {
        Ok(Json(input)) => input,
        Err(error) => return json_failure(error),
    };
    if let Err(error) = validate_issue(&input.title, &input.body) {
        return api_error(StatusCode::UNPROCESSABLE_ENTITY, error);
    }
    match state.store.create(input.title, input.body).await {
        Ok(issue) => (
            StatusCode::CREATED,
            [(header::LOCATION, format!("/api/issues/{}", issue.id))],
            Json(issue),
        )
            .into_response(),
        Err(error) => {
            let (status, message) = store_failure(error);
            api_error(status, message)
        }
    }
}
async fn api_status(
    State(state): State<WebState>,
    path: Result<Path<i64>, PathRejection>,
    input: Result<Json<StatusInput>, JsonRejection>,
) -> Response {
    let id = match path_id(path) {
        Ok(id) => id,
        Err(error) => return api_error(StatusCode::BAD_REQUEST, error),
    };
    let input = match input {
        Ok(Json(input)) => input,
        Err(error) => return json_failure(error),
    };
    let status = match parse_status(&input.status) {
        Ok(status) => status,
        Err(error) => return api_error(StatusCode::UNPROCESSABLE_ENTITY, error),
    };
    if input.expected_version < 1 {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Expected version must be a positive integer",
        );
    }
    match state
        .store
        .set_status(id, status, input.expected_version)
        .await
    {
        Ok(issue) => Json(issue).into_response(),
        Err(error) => {
            let (status, message) = store_failure(error);
            api_error(status, message)
        }
    }
}
