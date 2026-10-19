//! Application-owned SSR-first lab records. No core API, GPU or production IdP.
mod asset_data;
mod assets;
mod database;
mod domain;
mod views;

use axum::{
    Form, Router,
    body::Body,
    extract::{
        DefaultBodyLimit, Extension, Path as RoutePath, Query, State,
        rejection::{FormRejection, QueryRejection},
    },
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::{Next, from_fn_with_state},
    response::{IntoResponse, Redirect, Response, Sse, sse::Event},
};
use database::Database;
pub use domain::DemoIdentities;
use domain::{Fields, ListQuery, LoginSession, Principal};
use futures_util::stream;
use serde::Deserialize;
use std::{convert::Infallible, net::SocketAddr, path::Path, sync::Arc, time::Duration};
use tokio::sync::{Semaphore, watch};
use yetlet_core::View;
use yetlet_server_axum::{
    App, DEFAULT_CSP, DocumentMetadata, LanguageTag, render_document_with_metadata,
};
use yetlet_session::{Session, SessionConfig, SessionManager};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure {
    Invalid,
    Unauthorized,
    Forbidden,
    NotFound,
    Conflict,
    Transition,
    Capacity,
    Busy,
    Unavailable,
    Schema,
}
impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}
impl std::error::Error for Failure {}
impl From<rusqlite::Error> for Failure {
    fn from(error: rusqlite::Error) -> Self {
        match error.sqlite_error_code() {
            Some(rusqlite::ErrorCode::DatabaseBusy | rusqlite::ErrorCode::DatabaseLocked) => {
                Self::Busy
            }
            _ => Self::Unavailable,
        }
    }
}
impl Failure {
    fn status(self) -> StatusCode {
        match self {
            Self::Invalid => StatusCode::BAD_REQUEST,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Conflict => StatusCode::CONFLICT,
            Self::Transition => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Capacity | Self::Busy | Self::Unavailable | Self::Schema => {
                StatusCode::SERVICE_UNAVAILABLE
            }
        }
    }
    fn message(self) -> &'static str {
        match self {
            Self::Invalid => "Invalid request.",
            Self::Unauthorized => "Sign in again to continue.",
            Self::Forbidden => "You do not have permission for this project or action.",
            Self::NotFound => "Work item not found.",
            Self::Conflict => {
                "Another change was saved first. Your draft was not saved; review the latest version before retrying."
            }
            Self::Transition => "Reopen a closed item before changing it to Active or Blocked.",
            Self::Capacity => "The lab data limit has been reached. No change was saved.",
            Self::Busy => "Storage is busy. Check the current record before retrying a write.",
            Self::Unavailable | Self::Schema => {
                "Storage or session is unavailable. Check the current record before retrying a write."
            }
        }
    }
}
impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        let mut response = (
            self.status(),
            yetlet_server_axum::Page::with_metadata(
                DocumentMetadata::with_language(
                    "Lab Operations",
                    LanguageTag::new("en").expect("static language"),
                ),
                views::failure(self.message()),
            ),
        )
            .into_response();
        response.extensions_mut().insert(self);
        response
    }
}
#[derive(Clone)]
struct AppState {
    database: Database,
    identities: DemoIdentities,
    manifest: assets::Manifest,
    origin: HeaderValue,
    streams: Arc<Semaphore>,
    stopping: watch::Receiver<bool>,
}

/// Owns the local application and its graceful stream/worker shutdown signal.
pub struct Board {
    pub router: Router,
    database: Database,
    stopping: watch::Sender<bool>,
}
impl Board {
    /// Build only for a concrete loopback listener. No public-network demo mode.
    pub async fn open(
        path: &Path,
        listen: SocketAddr,
        identities: DemoIdentities,
    ) -> Result<Self, Failure> {
        Self::open_with_stylesheets(path, listen, identities, Vec::new()).await
    }

    /// Application-only composition for a separately built reference frontend.
    /// Empty selection retains the original embedded theme. Nonempty selection
    /// accepts at most eight same-origin Vite CSS filenames; the caller owns the
    /// matching static route. This changes neither authentication nor storage.
    pub async fn open_with_stylesheets(
        path: &Path,
        listen: SocketAddr,
        identities: DemoIdentities,
        stylesheets: Vec<String>,
    ) -> Result<Self, Failure> {
        if !listen.ip().is_loopback() || listen.port() == 0 {
            return Err(Failure::Invalid);
        }
        let mut seen = std::collections::BTreeSet::new();
        if stylesheets.len() > 8
            || stylesheets.iter().any(|path| {
                !seen.insert(path)
                    || path
                        .strip_prefix("/static/assets/")
                        .and_then(|name| name.strip_suffix(".css"))
                        .is_none_or(|name| {
                            name.is_empty()
                                || name.len() > 160
                                || !name
                                    .bytes()
                                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'-' | b'_'))
                        })
            })
        {
            return Err(Failure::Invalid);
        }
        let database = Database::open(path).await?;
        let mut manifest = assets::Manifest::parse(include_str!("../assets/manifest.json"))
            .map_err(|_| Failure::Unavailable)?;
        if !stylesheets.is_empty() {
            manifest.stylesheets = stylesheets;
        }
        let (stopping, stop_rx) = watch::channel(false);
        let state = AppState {
            database: database.clone(),
            identities,
            manifest,
            origin: HeaderValue::from_str(&format!("http://{listen}"))
                .map_err(|_| Failure::Invalid)?,
            streams: Arc::new(Semaphore::new(16)),
            stopping: stop_rx,
        };
        let sessions = SessionManager::<LoginSession>::new(
            SessionConfig::new("yetlet_lab", Duration::from_secs(1800), 1024)
                .map_err(|_| Failure::Invalid)?
                .insecure_for_local_http(),
        );
        let app = App::new()
            .group("/board", |group| {
                sessions.scoped(
                    group
                        .get("/", home)
                        .get("/login", login_page)
                        .post("/login", login)
                        .post("/logout", logout)
                        .get("/projects/{project}/items", listing)
                        .post("/projects/{project}/items", create)
                        .get("/projects/{project}/items/new", new_item)
                        .get("/projects/{project}/items/{id}", detail)
                        .post("/projects/{project}/items/{id}", update)
                        .get("/projects/{project}/items/{id}/edit", edit)
                        .get("/projects/{project}/events", events),
                )
            })
            .get("/", || async { Redirect::to("/board") })
            .get("/healthz", || async { StatusCode::OK })
            .get("/readyz", ready)
            .get("/favicon.ico", || async { StatusCode::NO_CONTENT })
            .get("/board-assets/{name}", asset)
            .with_state(state.clone())
            .into_router()
            .layer(DefaultBodyLimit::max(64 * 1024));
        let router =
            yetlet_server_axum::middleware::with_global(app, from_fn_with_state(state, security));
        Ok(Self {
            router,
            database,
            stopping,
        })
    }
    pub fn stop(&self) {
        let _ = self.stopping.send(true);
    }
    pub fn stop_signal(&self) -> watch::Sender<bool> {
        self.stopping.clone()
    }
    pub async fn drain(&self) -> Result<(), Failure> {
        self.stop();
        self.database.drain().await
    }
}
async fn security(
    State(state): State<AppState>,
    request: axum::extract::Request,
    next: Next,
) -> Response {
    // Exact authority prevents browser DNS-rebinding use of the loopback demo.
    let host = state
        .origin
        .to_str()
        .ok()
        .and_then(|v| v.strip_prefix("http://"));
    let valid_host = request.headers().get_all(header::HOST).iter().count() == 1
        && request
            .headers()
            .get(header::HOST)
            .and_then(|v| v.to_str().ok())
            == host;
    let origin = request.headers().get(header::ORIGIN);
    let valid_origin = request.headers().get_all(header::ORIGIN).iter().count() <= 1
        && origin.is_none_or(|value| *value == state.origin);
    let safe_method = matches!(
        *request.method(),
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    );
    let mut response = if !valid_host || (!safe_method && !valid_origin) {
        Failure::Forbidden.into_response()
    } else {
        next.run(request).await
    };
    // Failure has no State extractor. Apply the selected application theme here,
    // retaining its status and session headers without buffering response bodies.
    if let Some(failure) = response.extensions_mut().remove::<Failure>()
        && let Ok(themed) = page(
            &state,
            "Unable to complete the request | Lab Operations",
            views::failure(failure.message()),
            false,
        )
    {
        *response.body_mut() = themed.into_body();
        response.headers_mut().remove(header::CONTENT_LENGTH);
    }
    response.headers_mut().insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(DEFAULT_CSP),
    );
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        header::REFERRER_POLICY,
        // Native form POST must retain its same-origin Origin. no-referrer
        // serializes that header as null, which our exact-origin guard rejects.
        HeaderValue::from_static("same-origin"),
    );
    response
        .headers_mut()
        .entry(header::CACHE_CONTROL)
        .or_insert(HeaderValue::from_static("no-store"));
    response
}
fn page(state: &AppState, title: &str, view: View, live: bool) -> Result<Response, Failure> {
    let metadata =
        DocumentMetadata::with_language(title, LanguageTag::new("en").expect("static language"));
    let mut html =
        render_document_with_metadata(&metadata, &view).map_err(|_| Failure::Unavailable)?;
    let mut tags = String::new();
    for path in &state.manifest.stylesheets {
        tags.push_str(&format!("<link rel=\"stylesheet\" href=\"{path}\">"));
    }
    if live {
        tags.push_str(&format!(
            "<script type=\"module\" src=\"{}\"></script>",
            state.manifest.enhancement
        ));
    }
    html = html.replacen("</head>", &format!("{tags}</head>"), 1);
    Ok(axum::response::Html(html).into_response())
}
fn user(
    session: &Session<LoginSession>,
    project: Option<i64>,
    write: bool,
) -> Result<Principal, Failure> {
    let user = session
        .read()
        .map_err(|_| Failure::Unauthorized)?
        .user
        .ok_or(Failure::Unauthorized)?;
    if project.is_some_and(|project| project != user.project) || (write && !user.write) {
        return Err(Failure::Forbidden);
    }
    Ok(user)
}
fn csrf(session: &Session<LoginSession>) -> Result<String, Failure> {
    session.csrf_token().map_err(|_| Failure::Unauthorized)
}
fn require_csrf(session: &Session<LoginSession>, value: &str) -> Result<(), Failure> {
    if session
        .verify_csrf(value)
        .map_err(|_| Failure::Unauthorized)?
    {
        Ok(())
    } else {
        Err(Failure::Forbidden)
    }
}
async fn home(Extension(session): Extension<Session<LoginSession>>) -> Response {
    match user(&session, None, false) {
        Ok(user) => {
            Redirect::to(&format!("/board/projects/{}/items", user.project)).into_response()
        }
        Err(_) => Redirect::to("/board/login").into_response(),
    }
}
async fn login_page(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
) -> Result<Response, Failure> {
    page(
        &state,
        "Sign in | Lab Operations",
        views::login(&csrf(&session)?, None),
        false,
    )
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginForm {
    csrf: String,
    username: String,
    code: String,
}
async fn login(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    form: Result<Form<LoginForm>, FormRejection>,
) -> Result<Response, Failure> {
    let Form(form) = form.map_err(|_| Failure::Invalid)?;
    require_csrf(&session, &form.csrf)?;
    let Some(principal) = state.identities.authenticate(&form.username, &form.code) else {
        return Ok((
            StatusCode::UNAUTHORIZED,
            page(
                &state,
                "Sign in | Lab Operations",
                views::login(&csrf(&session)?, Some("The identity or code is not valid.")),
                false,
            )?,
        )
            .into_response());
    };
    session
        .replace_and_rotate(LoginSession {
            user: Some(principal),
        })
        .map_err(|_| Failure::Unavailable)?;
    Ok(Redirect::to(&format!("/board/projects/{}/items", principal.project)).into_response())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CsrfForm {
    csrf: String,
}
async fn logout(
    Extension(session): Extension<Session<LoginSession>>,
    form: Result<Form<CsrfForm>, FormRejection>,
) -> Result<Response, Failure> {
    let Form(form) = form.map_err(|_| Failure::Invalid)?;
    require_csrf(&session, &form.csrf)?;
    session.destroy().map_err(|_| Failure::Unauthorized)?;
    Ok(Redirect::to("/board/login").into_response())
}
async fn listing(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath(project): RoutePath<i64>,
    query: Result<Query<ListQuery>, QueryRejection>,
) -> Result<Response, Failure> {
    let user = user(&session, Some(project), false)?;
    let query = query.map_err(|_| Failure::Invalid)?.0.normalize()?;
    let values = state.database.list(project, query.clone()).await?;
    page(
        &state,
        "Work items | Lab Operations",
        views::listing(user, &csrf(&session)?, &query, &values),
        query.live,
    )
}
async fn new_item(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath(project): RoutePath<i64>,
) -> Result<Response, Failure> {
    let user = user(&session, Some(project), true)?;
    page(
        &state,
        "New work item | Lab Operations",
        views::edit(user, &csrf(&session)?, None, 0, &Fields::default(), None),
        false,
    )
}
async fn detail(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath((project, id)): RoutePath<(i64, i64)>,
) -> Result<Response, Failure> {
    let user = user(&session, Some(project), false)?;
    let (value, history) = state.database.get(project, id).await?;
    page(
        &state,
        "Work item | Lab Operations",
        views::detail(user, &csrf(&session)?, &value, &history),
        false,
    )
}
async fn edit(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath((project, id)): RoutePath<(i64, i64)>,
) -> Result<Response, Failure> {
    let user = user(&session, Some(project), true)?;
    let (value, _) = state.database.get(project, id).await?;
    page(
        &state,
        "Edit work item | Lab Operations",
        views::edit(
            user,
            &csrf(&session)?,
            Some(id),
            value.version,
            &value.fields,
            None,
        ),
        false,
    )
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditForm {
    csrf: String,
    expected_version: i64,
    title: String,
    description: String,
    status: String,
    priority: String,
    assignee: String,
}
impl EditForm {
    fn fields(&self) -> Fields {
        Fields {
            title: self.title.clone(),
            description: self.description.clone(),
            status: self.status.clone(),
            priority: self.priority.clone(),
            assignee: self.assignee.clone(),
        }
    }
}
async fn save(
    state: AppState,
    session: Session<LoginSession>,
    project: i64,
    id: Option<i64>,
    form: Result<Form<EditForm>, FormRejection>,
) -> Result<Response, Failure> {
    let principal = user(&session, Some(project), true)?;
    let Form(form) = match form {
        Ok(form) => form,
        Err(error) => {
            return Ok((error.status(), "Invalid or oversized form submission").into_response());
        }
    };
    require_csrf(&session, &form.csrf)?;
    let fields = form.fields();
    if !fields.errors(project).is_empty() {
        return Ok((
            StatusCode::UNPROCESSABLE_ENTITY,
            page(
                &state,
                "Correct fields | Lab Operations",
                views::edit(
                    principal,
                    &csrf(&session)?,
                    id,
                    form.expected_version,
                    &fields,
                    Some("Correct the highlighted fields. Your draft has not been saved."),
                ),
                false,
            )?,
        )
            .into_response());
    }
    // Recheck current session after parsing, before starting the bounded SQL operation.
    user(&session, Some(project), true)?;
    match state
        .database
        .save(principal, id, form.expected_version, fields.clone())
        .await
    {
        Ok(item) => Ok(
            Redirect::to(&format!("/board/projects/{project}/items/{}", item.id)).into_response(),
        ),
        Err(failure @ (Failure::Conflict | Failure::Transition)) => Ok((
            failure.status(),
            page(
                &state,
                "Draft not saved | Lab Operations",
                views::edit(
                    principal,
                    &csrf(&session)?,
                    id,
                    form.expected_version,
                    &fields,
                    Some(failure.message()),
                ),
                false,
            )?,
        )
            .into_response()),
        Err(error) => Err(error),
    }
}
async fn create(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath(project): RoutePath<i64>,
    form: Result<Form<EditForm>, FormRejection>,
) -> Result<Response, Failure> {
    save(state, session, project, None, form).await
}
async fn update(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath((project, id)): RoutePath<(i64, i64)>,
    form: Result<Form<EditForm>, FormRejection>,
) -> Result<Response, Failure> {
    save(state, session, project, Some(id), form).await
}
async fn ready(State(state): State<AppState>) -> Result<StatusCode, Failure> {
    if *state.stopping.borrow() {
        return Err(Failure::Unavailable);
    }
    state.database.ready().await?;
    Ok(StatusCode::OK)
}
async fn asset(RoutePath(name): RoutePath<String>) -> Response {
    let Some(asset) = assets::find(&format!("{}{name}", assets::PREFIX)) else {
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
async fn events(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath(project): RoutePath<i64>,
    headers: HeaderMap,
) -> Result<Response, Failure> {
    let principal = user(&session, Some(project), false)?;
    let ids: Vec<_> = headers.get_all("last-event-id").iter().collect();
    if ids.len() > 1
        || ids.first().is_some_and(|id| {
            id.to_str().ok().is_none_or(|s| {
                s.is_empty()
                    || s.len() > 19
                    || !s.bytes().all(|b| b.is_ascii_digit())
                    || s.parse::<i64>().is_err()
            })
        })
    {
        return Err(Failure::Invalid);
    }
    let permit = state
        .streams
        .clone()
        .try_acquire_owned()
        .map_err(|_| Failure::Busy)?;
    struct Cursor {
        state: AppState,
        session: Session<LoginSession>,
        principal: Principal,
        first: bool,
        deadline: tokio::time::Instant,
        _permit: tokio::sync::OwnedSemaphorePermit,
        finished: bool,
    }
    let cursor = Cursor {
        state,
        session,
        principal,
        first: true,
        deadline: tokio::time::Instant::now() + Duration::from_secs(300),
        _permit: permit,
        finished: false,
    };
    let stream = stream::unfold(cursor, |mut cursor| async move {
        if cursor.finished
            || *cursor.state.stopping.borrow()
            || tokio::time::Instant::now() >= cursor.deadline
        {
            return None;
        }
        if !cursor.first {
            tokio::select! {_ = tokio::time::sleep(Duration::from_secs(1))=>{},_ = cursor.state.stopping.changed()=>return None,}
        }
        cursor.first = false;
        if user(&cursor.session, Some(cursor.principal.project), false).ok()
            != Some(cursor.principal)
        {
            return None;
        }
        let event = match cursor
            .state
            .database
            .snapshot(cursor.principal.project)
            .await
        {
            Ok(snapshot) => Event::default()
                .event("snapshot")
                .id(snapshot.revision.to_string())
                .json_data(snapshot)
                .ok()?,
            Err(_) => {
                cursor.finished = true;
                Event::default()
                    .event("unavailable")
                    .data("Storage unavailable; retain the last snapshot.")
            }
        };
        // Every connection receives a full authoritative snapshot; gaps need no replay queue.
        Some((Ok::<_, Infallible>(event), cursor))
    });
    Ok(Sse::new(stream).into_response())
}

#[cfg(test)]
mod tests;
