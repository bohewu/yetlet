//! Loopback-only synthetic authentication, explicit Session and per-user notes.
//!
//! Session is not authentication. Two OS-random terminal-only login codes prove
//! an explicit application authentication boundary without passwords or custom
//! password hashing. Every note read/write rechecks ownership on the server.

use std::{
    collections::BTreeMap,
    fmt,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    Form, Router,
    extract::{
        DefaultBodyLimit, Extension, Path, State,
        rejection::{FormRejection, PathRejection},
    },
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use subtle::ConstantTimeEq;
use yetlet::server_axum::{App, Page};
use yetlet::session::{Session, SessionConfig, SessionConfigError, SessionError, SessionManager};
use yetlet::view::{Element, View};

const LOGIN_CODE_BYTES: usize = 32;
const LOGIN_CODE_HEX: usize = LOGIN_CODE_BYTES * 2;
const SESSION_TTL: Duration = Duration::from_secs(20 * 60);
const SESSION_CAPACITY: usize = 1_024;
const MAX_FORM_BODY: usize = 4 * 1024;
const MAX_NOTES_TOTAL: usize = 128;
const MAX_NOTES_PER_USER: usize = 16;
const MAX_TITLE_CHARS: usize = 80;
const MAX_BODY_CHARS: usize = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum UserId {
    Alice,
    Bob,
}

impl UserId {
    const fn display(self) -> &'static str {
        match self {
            Self::Alice => "Alice Example",
            Self::Bob => "Bob Example",
        }
    }
}

#[derive(Clone)]
pub struct DemoIdentities {
    alice: String,
    bob: String,
    dummy: String,
}

impl fmt::Debug for DemoIdentities {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("DemoIdentities { synthetic_codes: [REDACTED] }")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IdentityConfigError {
    InvalidCode,
    DuplicateCodes,
    EntropyUnavailable,
}

impl fmt::Display for IdentityConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidCode => {
                "synthetic login codes must be 64 lowercase hexadecimal characters"
            }
            Self::DuplicateCodes => "synthetic login codes must be distinct",
            Self::EntropyUnavailable => "operating-system entropy is unavailable",
        })
    }
}

impl std::error::Error for IdentityConfigError {}

impl DemoIdentities {
    pub fn new(alice: String, bob: String) -> Result<Self, IdentityConfigError> {
        if !valid_code(&alice) || !valid_code(&bob) {
            return Err(IdentityConfigError::InvalidCode);
        }
        if alice.as_bytes().ct_eq(bob.as_bytes()).into() {
            return Err(IdentityConfigError::DuplicateCodes);
        }
        Ok(Self {
            alice,
            bob,
            dummy: "0".repeat(LOGIN_CODE_HEX),
        })
    }

    pub fn generate() -> Result<Self, IdentityConfigError> {
        let alice = random_code()?;
        let mut bob = random_code()?;
        while alice.as_bytes().ct_eq(bob.as_bytes()).into() {
            bob = random_code()?;
        }
        Self::new(alice, bob)
    }

    pub fn alice_code(&self) -> &str {
        &self.alice
    }

    pub fn bob_code(&self) -> &str {
        &self.bob
    }

    fn authenticate(&self, username: &str, code: &str) -> Option<UserId> {
        let (candidate, user) = match username {
            "alice" => (&self.alice, Some(UserId::Alice)),
            "bob" => (&self.bob, Some(UserId::Bob)),
            _ => (&self.dummy, None),
        };
        let valid_length = code.len() == LOGIN_CODE_HEX;
        let mut padded = [0_u8; LOGIN_CODE_HEX];
        if valid_length {
            padded.copy_from_slice(code.as_bytes());
        }
        let matches: bool = candidate.as_bytes().ct_eq(&padded).into();
        if valid_length && matches { user } else { None }
    }
}

fn valid_code(value: &str) -> bool {
    value.len() == LOGIN_CODE_HEX
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
}

fn random_code() -> Result<String, IdentityConfigError> {
    let mut bytes = [0_u8; LOGIN_CODE_BYTES];
    getrandom::fill(&mut bytes).map_err(|_| IdentityConfigError::EntropyUnavailable)?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthSession {
    user: Option<UserId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Note {
    id: u64,
    owner: UserId,
    title: String,
    body: String,
}

#[derive(Default)]
struct NotesStore {
    next_id: u64,
    notes: BTreeMap<u64, Note>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoteError {
    NotFound,
    Forbidden,
    Capacity,
    IdExhausted,
}

impl NotesStore {
    fn create(&mut self, owner: UserId, title: String, body: String) -> Result<Note, NoteError> {
        if self.notes.len() >= MAX_NOTES_TOTAL
            || self
                .notes
                .values()
                .filter(|note| note.owner == owner)
                .count()
                >= MAX_NOTES_PER_USER
        {
            return Err(NoteError::Capacity);
        }
        let id = self.next_id.checked_add(1).ok_or(NoteError::IdExhausted)?;
        self.next_id = id;
        let note = Note {
            id,
            owner,
            title,
            body,
        };
        self.notes.insert(id, note.clone());
        Ok(note)
    }

    fn list(&self, owner: UserId) -> Vec<Note> {
        self.notes
            .values()
            .filter(|note| note.owner == owner)
            .cloned()
            .collect()
    }

    fn authorize(&self, owner: UserId, id: u64) -> Result<Note, NoteError> {
        let note = self.notes.get(&id).ok_or(NoteError::NotFound)?;
        if note.owner != owner {
            return Err(NoteError::Forbidden);
        }
        Ok(note.clone())
    }

    fn update(
        &mut self,
        owner: UserId,
        id: u64,
        title: String,
        body: String,
    ) -> Result<Note, NoteError> {
        self.authorize(owner, id)?;
        let note = self.notes.get_mut(&id).ok_or(NoteError::NotFound)?;
        note.title = title;
        note.body = body;
        Ok(note.clone())
    }

    fn delete(&mut self, owner: UserId, id: u64) -> Result<(), NoteError> {
        self.authorize(owner, id)?;
        self.notes.remove(&id);
        Ok(())
    }
}

#[derive(Clone)]
struct AppState {
    identities: Arc<DemoIdentities>,
    notes: Arc<Mutex<NotesStore>>,
}

#[derive(Debug)]
pub enum AuthAppError {
    NonLoopbackListener,
    NonIsolatedContainerListener,
    Session(SessionConfigError),
}

impl fmt::Display for AuthAppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonLoopbackListener => {
                formatter.write_str("the insecure local auth demo must bind a loopback address")
            }
            Self::NonIsolatedContainerListener => formatter
                .write_str("the isolated container lab must bind a loopback or wildcard address"),
            Self::Session(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for AuthAppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NonLoopbackListener | Self::NonIsolatedContainerListener => None,
            Self::Session(error) => Some(error),
        }
    }
}

impl From<SessionConfigError> for AuthAppError {
    fn from(error: SessionConfigError) -> Self {
        Self::Session(error)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LoginForm {
    csrf: String,
    username: String,
    code: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NoteForm {
    csrf: String,
    title: String,
    body: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CsrfForm {
    csrf: String,
}

#[derive(Debug, Clone, Copy)]
enum Failure {
    InvalidForm,
    PayloadTooLarge,
    InvalidCsrf,
    InvalidCredentials,
    Unauthenticated,
    Forbidden,
    NotFound,
    InvalidNote,
    Capacity,
    Session,
    Store,
}

impl Failure {
    fn status(self) -> StatusCode {
        match self {
            Self::InvalidForm | Self::InvalidNote => StatusCode::BAD_REQUEST,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::InvalidCsrf | Self::Forbidden => StatusCode::FORBIDDEN,
            Self::InvalidCredentials | Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Capacity => StatusCode::CONFLICT,
            Self::Session | Self::Store => StatusCode::SERVICE_UNAVAILABLE,
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::InvalidForm => "Invalid form",
            Self::PayloadTooLarge => "Form body exceeds the 4 KiB limit",
            Self::InvalidCsrf => "Invalid CSRF token",
            Self::InvalidCredentials => "Invalid synthetic identity or login code",
            Self::Unauthenticated => "Authentication required",
            Self::Forbidden => "You are not authorized to access this note",
            Self::NotFound => "Note does not exist",
            Self::InvalidNote => "Title must be 1-80 characters and body at most 2000 characters",
            Self::Capacity => "Note capacity reached",
            Self::Session => "Session unavailable",
            Self::Store => "Note storage unavailable",
        }
    }
}

impl IntoResponse for Failure {
    fn into_response(self) -> Response {
        (self.status(), self.message()).into_response()
    }
}

fn map_session(error: SessionError) -> Failure {
    match error {
        SessionError::Unavailable | SessionError::Expired | SessionError::Destroyed => {
            Failure::Session
        }
    }
}

fn parse_form<T>(form: Result<Form<T>, FormRejection>) -> Result<T, Failure> {
    match form {
        Ok(Form(value)) => Ok(value),
        Err(FormRejection::BytesRejection(_)) => Err(Failure::PayloadTooLarge),
        Err(_) => Err(Failure::InvalidForm),
    }
}

fn require_csrf(session: &Session<AuthSession>, submitted: &str) -> Result<(), Failure> {
    match session.verify_csrf(submitted).map_err(map_session)? {
        true => Ok(()),
        false => Err(Failure::InvalidCsrf),
    }
}

fn current_user(session: &Session<AuthSession>) -> Result<UserId, Failure> {
    session
        .read()
        .map_err(map_session)?
        .user
        .ok_or(Failure::Unauthenticated)
}

fn validate_note(title: String, body: String) -> Result<(String, String), Failure> {
    if title.chars().any(char::is_control) || body.chars().any(|character| character == '\0') {
        return Err(Failure::InvalidNote);
    }
    let title = title.trim().to_owned();
    let body = body.trim().to_owned();
    if title.is_empty()
        || title.chars().count() > MAX_TITLE_CHARS
        || body.chars().count() > MAX_BODY_CHARS
    {
        return Err(Failure::InvalidNote);
    }
    Ok((title, body))
}

fn hidden(name: &'static str, value: impl Into<String>) -> View {
    Element::new("input")
        .attribute("type", "hidden")
        .attribute("name", name)
        .attribute("value", value)
        .into()
}

fn login_view(csrf: &str, error: Option<&str>) -> View {
    let mut main = Element::new("main")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text("Yetlet Synthetic Auth Notes")))
        .child(Element::new("p").child(View::text(
            "Use a terminal-only synthetic Alice or Bob code. This is not a production identity provider.",
        )));
    if let Some(error) = error {
        main = main.child(
            Element::new("p")
                .attribute("role", "alert")
                .child(View::text(error)),
        );
    }
    main.child(
        Element::new("form")
            .attribute("method", "post")
            .attribute("action", "/auth/login")
            .child(hidden("csrf", csrf))
            .child(
                Element::new("label")
                    .attribute("for", "username")
                    .child(View::text("Synthetic identity (alice or bob)")),
            )
            .child(
                Element::new("input")
                    .attribute("id", "username")
                    .attribute("name", "username")
                    .attribute("type", "text")
                    .attribute("required", "")
                    .attribute("autocomplete", "username"),
            )
            .child(
                Element::new("label")
                    .attribute("for", "code")
                    .child(View::text("Terminal login code")),
            )
            .child(
                Element::new("input")
                    .attribute("id", "code")
                    .attribute("name", "code")
                    .attribute("type", "password")
                    .attribute("required", "")
                    .attribute("autocomplete", "current-password"),
            )
            .child(
                Element::new("button")
                    .attribute("type", "submit")
                    .child(View::text("Sign in and rotate session")),
            ),
    )
    .into()
}

fn note_form(action: String, csrf: &str, title: &str, body: &str, label: &str) -> View {
    Element::new("form")
        .attribute("method", "post")
        .attribute("action", action)
        .child(hidden("csrf", csrf))
        .child(
            Element::new("label")
                .attribute("for", "note-title")
                .child(View::text("Title")),
        )
        .child(
            Element::new("input")
                .attribute("id", "note-title")
                .attribute("name", "title")
                .attribute("type", "text")
                .attribute("value", title)
                .attribute("maxlength", MAX_TITLE_CHARS.to_string())
                .attribute("required", ""),
        )
        .child(
            Element::new("label")
                .attribute("for", "note-body")
                .child(View::text("Body")),
        )
        .child(
            Element::new("textarea")
                .attribute("id", "note-body")
                .attribute("name", "body")
                .attribute("maxlength", MAX_BODY_CHARS.to_string())
                .child(View::text(body)),
        )
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text(label)),
        )
        .into()
}

fn logout_form(csrf: &str) -> View {
    Element::new("form")
        .attribute("method", "post")
        .attribute("action", "/auth/logout")
        .child(hidden("csrf", csrf))
        .child(
            Element::new("button")
                .attribute("type", "submit")
                .child(View::text("Sign out and rotate session")),
        )
        .into()
}

fn notes_view(user: UserId, notes: &[Note], csrf: &str) -> View {
    let mut list = Element::new("ul");
    for note in notes {
        list = list.child(
            Element::new("li")
                .attribute("data-note-id", note.id.to_string())
                .child(
                    Element::new("a")
                        .attribute("href", format!("/auth/notes/{}", note.id))
                        .child(View::text(&note.title)),
                ),
        );
    }
    if notes.is_empty() {
        list = list.child(Element::new("li").child(View::text("No notes yet.")));
    }
    Element::new("main")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text(format!("{}'s private notes", user.display()))))
        .child(Element::new("p").child(View::text(
            "Ownership is checked on every request; note IDs alone never authorize access.",
        )))
        .child(list)
        .child(Element::new("h2").child(View::text("Create note")))
        .child(note_form(
            "/auth/notes".to_owned(),
            csrf,
            "",
            "",
            "Create note",
        ))
        .child(logout_form(csrf))
        .into()
}

fn note_view(user: UserId, note: &Note, csrf: &str) -> View {
    Element::new("main")
        .attribute("lang", "en")
        .child(Element::new("h1").child(View::text(&note.title)))
        .child(Element::new("p").child(View::text(&note.body)))
        .child(Element::new("p").child(View::text(format!("Owner: {}", user.display()))))
        .child(note_form(
            format!("/auth/notes/{}/update", note.id),
            csrf,
            &note.title,
            &note.body,
            "Update note",
        ))
        .child(
            Element::new("form")
                .attribute("method", "post")
                .attribute("action", format!("/auth/notes/{}/delete", note.id))
                .child(hidden("csrf", csrf))
                .child(
                    Element::new("button")
                        .attribute("type", "submit")
                        .child(View::text("Delete note")),
                ),
        )
        .child(
            Element::new("a")
                .attribute("href", "/auth/notes")
                .child(View::text("Back to notes")),
        )
        .child(logout_form(csrf))
        .into()
}

async fn login_page(Extension(session): Extension<Session<AuthSession>>) -> Result<Page, Failure> {
    if session.read().map_err(map_session)?.user.is_some() {
        return Ok(Page::new(
            "Yetlet | Already authenticated",
            Element::new("main")
                .attribute("lang", "en")
                .child(Element::new("h1").child(View::text("Already authenticated")))
                .child(
                    Element::new("a")
                        .attribute("href", "/auth/notes")
                        .child(View::text("Open private notes")),
                ),
        ));
    }
    let csrf = session.csrf_token().map_err(map_session)?;
    Ok(Page::new(
        "Yetlet | Synthetic login",
        login_view(&csrf, None),
    ))
}

async fn login(
    Extension(session): Extension<Session<AuthSession>>,
    State(state): State<AppState>,
    form: Result<Form<LoginForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let user = state
            .identities
            .authenticate(form.username.trim(), &form.code)
            .ok_or(Failure::InvalidCredentials)?;
        session
            .replace_and_rotate(AuthSession { user: Some(user) })
            .map_err(map_session)?;
        Ok::<_, Failure>(Redirect::to("/auth/notes").into_response())
    })();
    match result {
        Ok(response) => response,
        Err(Failure::InvalidCredentials) => {
            let csrf = match session.csrf_token() {
                Ok(csrf) => csrf,
                Err(error) => return map_session(error).into_response(),
            };
            (
                StatusCode::UNAUTHORIZED,
                Page::new(
                    "Yetlet | Login failed",
                    login_view(&csrf, Some(Failure::InvalidCredentials.message())),
                ),
            )
                .into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn notes_page(
    Extension(session): Extension<Session<AuthSession>>,
    State(state): State<AppState>,
) -> Response {
    let result = (|| {
        let user = current_user(&session)?;
        let notes = state.notes.lock().map_err(|_| Failure::Store)?.list(user);
        let csrf = session.csrf_token().map_err(map_session)?;
        Ok::<_, Failure>(
            Page::new("Yetlet | Private notes", notes_view(user, &notes, &csrf)).into_response(),
        )
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn create_note(
    Extension(session): Extension<Session<AuthSession>>,
    State(state): State<AppState>,
    form: Result<Form<NoteForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let user = current_user(&session)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let (title, body) = validate_note(form.title, form.body)?;
        let note = state
            .notes
            .lock()
            .map_err(|_| Failure::Store)?
            .create(user, title, body)
            .map_err(map_note)?;
        Ok::<_, Failure>(Redirect::to(&format!("/auth/notes/{}", note.id)).into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

fn parse_id(path: Result<Path<u64>, PathRejection>) -> Result<u64, Failure> {
    path.map(|Path(id)| id).map_err(|_| Failure::NotFound)
}

async fn note_page(
    Extension(session): Extension<Session<AuthSession>>,
    State(state): State<AppState>,
    path: Result<Path<u64>, PathRejection>,
) -> Response {
    let result = (|| {
        let user = current_user(&session)?;
        let id = parse_id(path)?;
        let note = state
            .notes
            .lock()
            .map_err(|_| Failure::Store)?
            .authorize(user, id)
            .map_err(map_note)?;
        let csrf = session.csrf_token().map_err(map_session)?;
        Ok::<_, Failure>(
            Page::new("Yetlet | Private note", note_view(user, &note, &csrf)).into_response(),
        )
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn update_note(
    Extension(session): Extension<Session<AuthSession>>,
    State(state): State<AppState>,
    path: Result<Path<u64>, PathRejection>,
    form: Result<Form<NoteForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let user = current_user(&session)?;
        let id = parse_id(path)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        let (title, body) = validate_note(form.title, form.body)?;
        state
            .notes
            .lock()
            .map_err(|_| Failure::Store)?
            .update(user, id, title, body)
            .map_err(map_note)?;
        Ok::<_, Failure>(Redirect::to(&format!("/auth/notes/{id}")).into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn delete_note(
    Extension(session): Extension<Session<AuthSession>>,
    State(state): State<AppState>,
    path: Result<Path<u64>, PathRejection>,
    form: Result<Form<CsrfForm>, FormRejection>,
) -> Response {
    let result = (|| {
        let user = current_user(&session)?;
        let id = parse_id(path)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        state
            .notes
            .lock()
            .map_err(|_| Failure::Store)?
            .delete(user, id)
            .map_err(map_note)?;
        Ok::<_, Failure>(Redirect::to("/auth/notes").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

async fn logout(
    Extension(session): Extension<Session<AuthSession>>,
    form: Result<Form<CsrfForm>, FormRejection>,
) -> Response {
    let result = (|| {
        current_user(&session)?;
        let form = parse_form(form)?;
        require_csrf(&session, &form.csrf)?;
        session
            .replace_and_rotate(AuthSession::default())
            .map_err(map_session)?;
        Ok::<_, Failure>(Redirect::to("/auth/login").into_response())
    })();
    result.unwrap_or_else(IntoResponse::into_response)
}

fn map_note(error: NoteError) -> Failure {
    match error {
        NoteError::NotFound => Failure::NotFound,
        NoteError::Forbidden => Failure::Forbidden,
        NoteError::Capacity | NoteError::IdExhausted => Failure::Capacity,
    }
}

fn build_app(identities: DemoIdentities) -> Result<Router, AuthAppError> {
    let sessions = SessionManager::<AuthSession>::new(
        SessionConfig::new("yetlet-auth-notes", SESSION_TTL, SESSION_CAPACITY)?
            .insecure_for_local_http(),
    );
    let state = AppState {
        identities: Arc::new(identities),
        notes: Arc::new(Mutex::new(NotesStore::default())),
    };
    Ok(App::<AppState>::new()
        .get("/", || async { Redirect::to("/auth/notes") })
        .get("/healthz", || async { "ok" })
        .group("/auth", move |group| {
            let sessions = sessions.clone();
            group
                .get("/login", login_page)
                .post("/login", login)
                .get("/notes", notes_page)
                .post("/notes", create_note)
                .get("/notes/{id}", note_page)
                .post("/notes/{id}/update", update_note)
                .post("/notes/{id}/delete", delete_note)
                .post("/logout", logout)
                .route_layer(DefaultBodyLimit::max(MAX_FORM_BODY))
                .configure(move |group| sessions.scoped(group))
        })
        .with_state(state)
        .into_router())
}

pub fn app(listener: SocketAddr, identities: DemoIdentities) -> Result<Router, AuthAppError> {
    if !listener.ip().is_loopback() {
        return Err(AuthAppError::NonLoopbackListener);
    }
    build_app(identities)
}

/// Build the insecure HTTP fixture for an explicitly isolated container lab.
///
/// A wildcard listener is accepted only through this separately named entrypoint.
/// The caller must keep the container network private and publish any reverse proxy
/// only on host loopback. This does not make the sample production deployable.
pub fn app_for_isolated_container_lab(
    listener: SocketAddr,
    identities: DemoIdentities,
) -> Result<Router, AuthAppError> {
    if !(listener.ip().is_loopback() || listener.ip().is_unspecified()) {
        return Err(AuthAppError::NonIsolatedContainerListener);
    }
    build_app(identities)
}
#[cfg(test)]
mod tests;
