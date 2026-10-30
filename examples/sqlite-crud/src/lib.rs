//! SQL-first, application-owned SQLite CRUD using `rusqlite` directly.
//!
//! There is no Yetlet ORM, repository framework, query DSL, implicit database
//! provider or browser runtime. Synchronous SQLite work is bounded and moved to
//! Tokio's blocking pool; application state owns only a database path and gate.

use std::{
    fmt,
    fs::{self, OpenOptions},
    io,
    net::SocketAddr,
    path::{Path as FsPath, PathBuf},
    sync::Arc,
    time::Duration,
};

use axum::{
    Json, Router,
    extract::{
        DefaultBodyLimit, Path, Query, State,
        rejection::{JsonRejection, PathRejection, QueryRejection},
    },
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use rusqlite::{
    Connection, ErrorCode, OpenFlags, OptionalExtension, Row, TransactionBehavior, params,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;
use yetlet::server_axum::App;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

const MIGRATION_1: &str = include_str!("../migrations/0001_init.sql");
const SCHEMA_VERSION: i64 = 1;
const MAX_JSON_BODY: usize = 8 * 1024;
const MAX_DB_OPERATIONS: usize = 4;
const BUSY_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Note {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub version: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateNote {
    title: String,
    body: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateNote {
    title: String,
    body: String,
    expected_version: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteQuery {
    expected_version: u64,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct ListQuery {
    limit: Option<u32>,
}

#[derive(Serialize)]
struct NoteList {
    items: Vec<Note>,
    count: usize,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
    schema_version: i64,
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorBody,
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
}

#[derive(Debug)]
pub enum DatabaseError {
    Sqlite(rusqlite::Error),
    Io(io::Error),
    NotFound,
    Conflict,
    UnsupportedSchema(i64),
    SchemaMismatch,
    WorkerUnavailable,
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => error.fmt(formatter),
            Self::Io(error) => error.fmt(formatter),
            Self::NotFound => formatter.write_str("note was not found"),
            Self::Conflict => formatter.write_str("note version is stale"),
            Self::UnsupportedSchema(version) => {
                write!(
                    formatter,
                    "database schema version {version} is newer than supported"
                )
            }
            Self::SchemaMismatch => formatter.write_str("database schema objects are incomplete"),
            Self::WorkerUnavailable => formatter.write_str("blocking database worker unavailable"),
        }
    }
}

impl std::error::Error for DatabaseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(error: rusqlite::Error) -> Self {
        Self::Sqlite(error)
    }
}

impl From<io::Error> for DatabaseError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[derive(Clone)]
pub struct Database {
    inner: Arc<DatabaseInner>,
}

struct DatabaseInner {
    path: PathBuf,
    gate: Arc<Semaphore>,
}

impl Database {
    /// Initialize the explicitly supplied application database once at startup.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, DatabaseError> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(DatabaseError::Sqlite(rusqlite::Error::InvalidPath(path)));
        }
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        if path.is_dir() {
            return Err(DatabaseError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "database path is a directory",
            )));
        }
        prepare_database_file(&path)?;
        let mut connection = open_connection(&path)?;
        migrate(&mut connection)?;
        verify_schema(&connection)?;
        Ok(Self {
            inner: Arc::new(DatabaseInner {
                path,
                gate: Arc::new(Semaphore::new(MAX_DB_OPERATIONS)),
            }),
        })
    }

    pub fn path(&self) -> &FsPath {
        &self.inner.path
    }

    async fn run<T, F>(&self, operation: F) -> Result<T, DatabaseError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, DatabaseError> + Send + 'static,
    {
        let permit = self
            .inner
            .gate
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| DatabaseError::WorkerUnavailable)?;
        let path = self.inner.path.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut connection = open_connection(&path)?;
            operation(&mut connection)
        })
        .await
        .map_err(|_| DatabaseError::WorkerUnavailable)?
    }

    async fn schema_version(&self) -> Result<i64, DatabaseError> {
        self.run(|connection| {
            verify_schema(connection)?;
            connection
                .pragma_query_value(None, "user_version", |row| row.get(0))
                .map_err(Into::into)
        })
        .await
    }

    async fn list(&self, limit: u32) -> Result<Vec<Note>, DatabaseError> {
        self.run(move |connection| {
            let mut statement = connection.prepare(
                "SELECT id, title, body, version, created_at, updated_at \
                 FROM notes ORDER BY id LIMIT ?1",
            )?;
            let rows = statement.query_map(params![i64::from(limit)], note_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(Into::into)
        })
        .await
    }

    async fn get(&self, id: i64) -> Result<Note, DatabaseError> {
        self.run(move |connection| select_note(connection, id)?.ok_or(DatabaseError::NotFound))
            .await
    }

    async fn create(&self, title: String, body: String) -> Result<Note, DatabaseError> {
        self.run(move |connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute(
                "INSERT INTO notes(title, body) VALUES (?1, ?2)",
                params![title, body],
            )?;
            let id = transaction.last_insert_rowid();
            let note = select_note(&transaction, id)?.ok_or(DatabaseError::NotFound)?;
            insert_audit(&transaction, &note, "create")?;
            transaction.commit()?;
            Ok(note)
        })
        .await
    }

    async fn update(
        &self,
        id: i64,
        title: String,
        body: String,
        expected_version: i64,
    ) -> Result<Note, DatabaseError> {
        self.run(move |connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let changed = transaction.execute(
                "UPDATE notes \
                 SET title=?1, body=?2, version=version+1, updated_at=unixepoch() \
                 WHERE id=?3 AND version=?4",
                params![title, body, id, expected_version],
            )?;
            if changed == 0 {
                let exists: bool = transaction.query_row(
                    "SELECT EXISTS(SELECT 1 FROM notes WHERE id=?1)",
                    params![id],
                    |row| row.get(0),
                )?;
                return Err(if exists {
                    DatabaseError::Conflict
                } else {
                    DatabaseError::NotFound
                });
            }
            let note = select_note(&transaction, id)?.ok_or(DatabaseError::NotFound)?;
            insert_audit(&transaction, &note, "update")?;
            transaction.commit()?;
            Ok(note)
        })
        .await
    }

    async fn delete(&self, id: i64, expected_version: i64) -> Result<(), DatabaseError> {
        self.run(move |connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let note = select_note(&transaction, id)?.ok_or(DatabaseError::NotFound)?;
            if note.version != expected_version {
                return Err(DatabaseError::Conflict);
            }
            let changed = transaction.execute(
                "DELETE FROM notes WHERE id=?1 AND version=?2",
                params![id, expected_version],
            )?;
            if changed != 1 {
                return Err(DatabaseError::Conflict);
            }
            insert_audit(&transaction, &note, "delete")?;
            transaction.commit()?;
            Ok(())
        })
        .await
    }
}

fn prepare_database_file(path: &FsPath) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    match options.open(path) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Ok(()),
        Err(error) => Err(error),
    }
}

fn open_connection(path: &FsPath) -> Result<Connection, DatabaseError> {
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(BUSY_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    Ok(connection)
}

fn migrate(connection: &mut Connection) -> Result<(), DatabaseError> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version > SCHEMA_VERSION {
        return Err(DatabaseError::UnsupportedSchema(version));
    }
    let _: String = connection.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    connection.pragma_update(None, "synchronous", "NORMAL")?;
    if version == 0 {
        let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute_batch(MIGRATION_1)?;
        transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        transaction.commit()?;
    }
    Ok(())
}

fn verify_schema(connection: &Connection) -> Result<(), DatabaseError> {
    const NOTES: &[&str] = &["id", "title", "body", "version", "created_at", "updated_at"];
    const AUDIT: &[&str] = &[
        "id",
        "note_id",
        "action",
        "note_version",
        "title",
        "created_at",
    ];

    verify_table_columns(
        connection,
        "SELECT name FROM pragma_table_info('notes') ORDER BY cid",
        NOTES,
    )?;
    verify_table_columns(
        connection,
        "SELECT name FROM pragma_table_info('note_audit') ORDER BY cid",
        AUDIT,
    )?;
    Ok(())
}

fn verify_table_columns(
    connection: &Connection,
    query: &'static str,
    expected: &[&str],
) -> Result<(), DatabaseError> {
    let mut statement = connection.prepare(query)?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    if columns.len() != expected.len()
        || columns
            .iter()
            .map(String::as_str)
            .ne(expected.iter().copied())
    {
        return Err(DatabaseError::SchemaMismatch);
    }
    Ok(())
}

fn note_from_row(row: &Row<'_>) -> rusqlite::Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        version: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

fn select_note(connection: &Connection, id: i64) -> Result<Option<Note>, DatabaseError> {
    connection
        .query_row(
            "SELECT id, title, body, version, created_at, updated_at \
             FROM notes WHERE id=?1",
            params![id],
            note_from_row,
        )
        .optional()
        .map_err(Into::into)
}

fn insert_audit(
    connection: &Connection,
    note: &Note,
    action: &'static str,
) -> Result<(), DatabaseError> {
    connection.execute(
        "INSERT INTO note_audit(note_id, action, note_version, title) \
         VALUES (?1, ?2, ?3, ?4)",
        params![note.id, action, note.version, note.title],
    )?;
    Ok(())
}

#[derive(Debug, Clone, Copy)]
struct ApiFailure {
    status: StatusCode,
    code: &'static str,
    message: &'static str,
}

impl ApiFailure {
    const fn new(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self {
            status,
            code,
            message,
        }
    }
}

impl IntoResponse for ApiFailure {
    fn into_response(self) -> Response {
        api_error(self.status, self.code, self.message)
    }
}

impl From<DatabaseError> for ApiFailure {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::NotFound => {
                Self::new(StatusCode::NOT_FOUND, "not_found", "Note was not found")
            }
            DatabaseError::Conflict => Self::new(
                StatusCode::CONFLICT,
                "stale_version",
                "Expected version does not match the stored note",
            ),
            DatabaseError::Sqlite(rusqlite::Error::SqliteFailure(error, _))
                if error.code == ErrorCode::ConstraintViolation =>
            {
                Self::new(
                    StatusCode::CONFLICT,
                    "constraint_violation",
                    "The write violates a database constraint",
                )
            }
            DatabaseError::Sqlite(rusqlite::Error::SqliteFailure(error, _))
                if matches!(
                    error.code,
                    ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked
                ) =>
            {
                Self::new(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "database_busy",
                    "Database is temporarily busy",
                )
            }
            _ => Self::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Unable to complete database operation",
            ),
        }
    }
}

fn apply_response_headers(response: &mut Response) {
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response.headers_mut().insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    response
        .headers_mut()
        .insert("x-yetlet-sqlite-schema", HeaderValue::from_static("1"));
}

fn json_response<T: Serialize>(status: StatusCode, body: T) -> Response {
    let mut response = (status, Json(body)).into_response();
    apply_response_headers(&mut response);
    response
}

fn empty_response(status: StatusCode) -> Response {
    let mut response = status.into_response();
    apply_response_headers(&mut response);
    response
}

fn api_error(status: StatusCode, code: &'static str, message: &'static str) -> Response {
    json_response(
        status,
        ErrorEnvelope {
            error: ErrorBody { code, message },
        },
    )
}

fn parse_json<T>(payload: Result<Json<T>, JsonRejection>) -> Result<T, ApiFailure> {
    match payload {
        Ok(Json(value)) => Ok(value),
        Err(JsonRejection::BytesRejection(_)) => Err(ApiFailure::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "payload_too_large",
            "JSON body exceeds the 8 KiB limit",
        )),
        Err(_) => Err(ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid_json",
            "Request body does not match the JSON schema",
        )),
    }
}

fn parse_id(path: Result<Path<i64>, PathRejection>) -> Result<i64, ApiFailure> {
    let id = path.map(|Path(id)| id).map_err(|_| {
        ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid_path",
            "Note id must be a positive integer",
        )
    })?;
    if id < 1 {
        return Err(ApiFailure::new(
            StatusCode::BAD_REQUEST,
            "invalid_path",
            "Note id must be a positive integer",
        ));
    }
    Ok(id)
}

fn expected_version(version: u64) -> Result<i64, ApiFailure> {
    let value = i64::try_from(version).map_err(|_| {
        ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_version",
            "Expected version is outside the supported range",
        )
    })?;
    if value < 1 {
        return Err(ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_version",
            "Expected version must be positive",
        ));
    }
    Ok(value)
}

fn validate_note(title: String, body: String) -> Result<(String, String), ApiFailure> {
    let title = title.trim().to_owned();
    if title.is_empty() || title.chars().count() > 120 || title.chars().any(char::is_control) {
        return Err(ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_title",
            "Title must contain 1 to 120 non-control characters",
        ));
    }
    if body.chars().count() > 4_000
        || body
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(ApiFailure::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_body",
            "Body must contain at most 4000 text characters",
        ));
    }
    Ok((title, body))
}

fn require_mutation_policy(headers: &HeaderMap) -> Result<(), ApiFailure> {
    if headers.get_all(header::ORIGIN).iter().next().is_some() {
        return Err(ApiFailure::new(
            StatusCode::FORBIDDEN,
            "browser_origin_forbidden",
            "Browser-originated mutations are not enabled by this API sample",
        ));
    }
    Ok(())
}

fn require_json_content_type(headers: &HeaderMap) -> Result<(), ApiFailure> {
    let mut values = headers.get_all(header::CONTENT_TYPE).iter();
    let Some(first) = values.next() else {
        return Err(ApiFailure::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Content-Type must be application/json",
        ));
    };
    if values.next().is_some() {
        return Err(ApiFailure::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Exactly one Content-Type header is required",
        ));
    }
    let value = first.to_str().map_err(|_| {
        ApiFailure::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Content-Type must be application/json",
        )
    })?;
    if !value
        .split(';')
        .next()
        .is_some_and(|value| value.trim().eq_ignore_ascii_case("application/json"))
    {
        return Err(ApiFailure::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Content-Type must be application/json",
        ));
    }
    Ok(())
}

async fn health(State(database): State<Database>) -> Response {
    match database.schema_version().await {
        Ok(schema_version) => json_response(
            StatusCode::OK,
            Health {
                status: "ok",
                schema_version,
            },
        ),
        Err(error) => ApiFailure::from(error).into_response(),
    }
}

async fn list_notes(
    State(database): State<Database>,
    query: Result<Query<ListQuery>, QueryRejection>,
) -> Response {
    let Query(query) = match query {
        Ok(query) => query,
        Err(_) => {
            return api_error(
                StatusCode::BAD_REQUEST,
                "invalid_query",
                "Query does not match the list schema",
            );
        }
    };
    let limit = query.limit.unwrap_or(50);
    if !(1..=100).contains(&limit) {
        return api_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_limit",
            "Limit must be between 1 and 100",
        );
    }
    match database.list(limit).await {
        Ok(items) => {
            let count = items.len();
            json_response(StatusCode::OK, NoteList { items, count })
        }
        Err(error) => ApiFailure::from(error).into_response(),
    }
}

async fn get_note(
    State(database): State<Database>,
    path: Result<Path<i64>, PathRejection>,
) -> Response {
    let id = match parse_id(path) {
        Ok(id) => id,
        Err(error) => return error.into_response(),
    };
    match database.get(id).await {
        Ok(note) => json_response(StatusCode::OK, note),
        Err(error) => ApiFailure::from(error).into_response(),
    }
}

async fn create_note(
    State(database): State<Database>,
    headers: HeaderMap,
    payload: Result<Json<CreateNote>, JsonRejection>,
) -> Response {
    let result = async {
        require_mutation_policy(&headers)?;
        require_json_content_type(&headers)?;
        let payload = parse_json(payload)?;
        let (title, body) = validate_note(payload.title, payload.body)?;
        database.create(title, body).await.map_err(ApiFailure::from)
    }
    .await;
    match result {
        Ok(note) => {
            let mut response = json_response(StatusCode::CREATED, note.clone());
            response.headers_mut().insert(
                header::LOCATION,
                HeaderValue::from_str(&format!("/api/notes/{}", note.id))
                    .expect("numeric location is valid"),
            );
            response
        }
        Err(error) => error.into_response(),
    }
}

async fn update_note(
    State(database): State<Database>,
    path: Result<Path<i64>, PathRejection>,
    headers: HeaderMap,
    payload: Result<Json<UpdateNote>, JsonRejection>,
) -> Response {
    let result = async {
        require_mutation_policy(&headers)?;
        require_json_content_type(&headers)?;
        let id = parse_id(path)?;
        let payload = parse_json(payload)?;
        let expected_version = expected_version(payload.expected_version)?;
        let (title, body) = validate_note(payload.title, payload.body)?;
        database
            .update(id, title, body, expected_version)
            .await
            .map_err(ApiFailure::from)
    }
    .await;
    match result {
        Ok(note) => json_response(StatusCode::OK, note),
        Err(error) => error.into_response(),
    }
}

async fn delete_note(
    State(database): State<Database>,
    path: Result<Path<i64>, PathRejection>,
    headers: HeaderMap,
    query: Result<Query<DeleteQuery>, QueryRejection>,
) -> Response {
    let result = async {
        require_mutation_policy(&headers)?;
        let id = parse_id(path)?;
        let Query(query) = query.map_err(|_| {
            ApiFailure::new(
                StatusCode::BAD_REQUEST,
                "invalid_query",
                "Delete requires expected_version",
            )
        })?;
        let version = expected_version(query.expected_version)?;
        database.delete(id, version).await.map_err(ApiFailure::from)
    }
    .await;
    match result {
        Ok(()) => empty_response(StatusCode::NO_CONTENT),
        Err(error) => error.into_response(),
    }
}

/// This anonymous persistent sample is intentionally loopback-only.
pub fn listen_address(value: &str) -> Result<SocketAddr, io::Error> {
    let address: SocketAddr = value
        .parse()
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    if !address.ip().is_loopback() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the anonymous SQLite sample must bind a loopback address",
        ));
    }
    Ok(address)
}

pub fn app(database: Database) -> Router {
    App::<Database>::new()
        .get("/healthz", health)
        .route("/api/notes", get(list_notes).post(create_note))
        .route(
            "/api/notes/{id}",
            get(get_note).put(update_note).delete(delete_note),
        )
        .route_layer(DefaultBodyLimit::max(MAX_JSON_BODY))
        .with_state(database)
        .into_router()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Method, Request},
    };
    use serde_json::{Value, json};
    use tempfile::TempDir;
    use tower::ServiceExt;

    fn fixture() -> (TempDir, Database) {
        let directory = tempfile::tempdir().unwrap();
        let database = Database::open(directory.path().join("notes.sqlite3")).unwrap();
        (directory, database)
    }

    async fn request(
        database: Database,
        method: Method,
        uri: &str,
        body: Option<Value>,
        origin: Option<&str>,
    ) -> (StatusCode, HeaderMap, Vec<u8>) {
        let mut builder = Request::builder().method(method).uri(uri);
        let payload = match body {
            Some(body) => {
                builder = builder.header(header::CONTENT_TYPE, "application/json");
                serde_json::to_vec(&body).unwrap()
            }
            None => Vec::new(),
        };
        if let Some(origin) = origin {
            builder = builder.header(header::ORIGIN, origin);
        }
        let response = app(database)
            .oneshot(builder.body(Body::from(payload)).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = to_bytes(response.into_body(), 32 * 1024)
            .await
            .unwrap()
            .to_vec();
        (status, headers, bytes)
    }

    fn json_body(bytes: &[u8]) -> Value {
        serde_json::from_slice(bytes).unwrap()
    }

    fn audit_count(path: &FsPath) -> i64 {
        let connection = open_connection(path).unwrap();
        connection
            .query_row("SELECT COUNT(*) FROM note_audit", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn migration_is_explicit_idempotent_and_rejects_newer_schema() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("notes.sqlite3");
        let database = Database::open(&path).unwrap();
        let connection = open_connection(database.path()).unwrap();
        let version: i64 = connection
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        drop(connection);
        drop(database);
        Database::open(&path).unwrap();
        let connection = open_connection(&path).unwrap();
        connection.pragma_update(None, "user_version", 99).unwrap();
        drop(connection);
        let incomplete_path = directory.path().join("incomplete.sqlite3");
        Database::open(&incomplete_path).unwrap();
        let incomplete = open_connection(&incomplete_path).unwrap();
        incomplete.execute_batch("DROP TABLE note_audit").unwrap();
        drop(incomplete);
        assert!(matches!(
            Database::open(&incomplete_path),
            Err(DatabaseError::SchemaMismatch)
        ));
        let missing_note_column_path = directory.path().join("missing-note-column.sqlite3");
        Database::open(&missing_note_column_path).unwrap();
        let missing_note_column = open_connection(&missing_note_column_path).unwrap();
        missing_note_column
            .execute_batch("ALTER TABLE notes RENAME COLUMN body TO content")
            .unwrap();
        drop(missing_note_column);
        assert!(matches!(
            Database::open(&missing_note_column_path),
            Err(DatabaseError::SchemaMismatch)
        ));

        let missing_audit_column_path = directory.path().join("missing-audit-column.sqlite3");
        Database::open(&missing_audit_column_path).unwrap();
        let missing_audit_column = open_connection(&missing_audit_column_path).unwrap();
        missing_audit_column
            .execute_batch("ALTER TABLE note_audit RENAME COLUMN title TO note_title")
            .unwrap();
        drop(missing_audit_column);
        assert!(matches!(
            Database::open(&missing_audit_column_path),
            Err(DatabaseError::SchemaMismatch)
        ));
        assert!(matches!(
            Database::open(&path),
            Err(DatabaseError::UnsupportedSchema(99))
        ));
    }

    #[tokio::test]
    async fn direct_sql_crud_and_parameter_binding_preserve_hostile_text() {
        let (_directory, database) = fixture();
        let title = "Robert'); DROP TABLE notes;--".to_owned();
        let created = database
            .create(title.clone(), "quoted ?1 body".to_owned())
            .await
            .unwrap();
        assert_eq!(created.title, title);
        assert_eq!(database.list(10).await.unwrap().len(), 1);
        let updated = database
            .update(
                created.id,
                "Safe update".to_owned(),
                "body".to_owned(),
                created.version,
            )
            .await
            .unwrap();
        assert_eq!(updated.version, 2);
        database.delete(updated.id, updated.version).await.unwrap();
        assert!(matches!(
            database.get(updated.id).await,
            Err(DatabaseError::NotFound)
        ));
        assert_eq!(audit_count(database.path()), 3);
    }

    #[tokio::test]
    async fn failed_audit_rolls_back_a_prior_update() {
        let (_directory, database) = fixture();
        let created = database
            .create("Original".to_owned(), "body".to_owned())
            .await
            .unwrap();
        let connection = open_connection(database.path()).unwrap();
        connection
            .execute_batch(
                "CREATE TRIGGER fail_test_audit BEFORE INSERT ON note_audit \
                 WHEN NEW.title='Rollback me' BEGIN \
                 SELECT RAISE(ABORT, 'forced audit failure'); END;",
            )
            .unwrap();
        drop(connection);
        assert!(matches!(
            database
                .update(
                    created.id,
                    "Rollback me".to_owned(),
                    "changed".to_owned(),
                    created.version,
                )
                .await,
            Err(DatabaseError::Sqlite(_))
        ));
        let retained = database.get(created.id).await.unwrap();
        assert_eq!(retained.title, "Original");
        assert_eq!(retained.body, "body");
        assert_eq!(retained.version, 1);
        assert_eq!(audit_count(database.path()), 1);
    }

    #[tokio::test]
    async fn stale_update_rolls_back_and_concurrent_expected_version_has_one_winner() {
        let (_directory, database) = fixture();
        let created = database
            .create("Concurrent".to_owned(), "start".to_owned())
            .await
            .unwrap();
        assert!(matches!(
            database
                .update(created.id, "stale".to_owned(), "stale".to_owned(), 99,)
                .await,
            Err(DatabaseError::Conflict)
        ));
        assert_eq!(audit_count(database.path()), 1);

        let first = database.clone();
        let second = database.clone();
        let (a, b) = tokio::join!(
            first.update(
                created.id,
                "Winner A".to_owned(),
                "a".to_owned(),
                created.version,
            ),
            second.update(
                created.id,
                "Winner B".to_owned(),
                "b".to_owned(),
                created.version,
            )
        );
        let successes = [&a, &b].into_iter().filter(|result| result.is_ok()).count();
        let conflicts = [a, b]
            .into_iter()
            .filter(|result| matches!(result, Err(DatabaseError::Conflict)))
            .count();
        assert_eq!((successes, conflicts), (1, 1));
        assert_eq!(database.get(created.id).await.unwrap().version, 2);
        assert_eq!(audit_count(database.path()), 2);
    }

    #[tokio::test]
    async fn reopening_the_same_file_preserves_committed_state() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("notes.sqlite3");
        let first = Database::open(&path).unwrap();
        let created = first
            .create("Persistent".to_owned(), "survives restart".to_owned())
            .await
            .unwrap();
        drop(first);
        let reopened = Database::open(&path).unwrap();
        assert_eq!(reopened.get(created.id).await.unwrap(), created);
    }

    #[tokio::test]
    async fn api_enforces_json_origin_validation_and_optimistic_versions() {
        let (_directory, database) = fixture();
        let (status, headers, bytes) = request(
            database.clone(),
            Method::POST,
            "/api/notes",
            Some(json!({"title":"API note","body":"text"})),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(headers["x-yetlet-sqlite-schema"], "1");
        let created = json_body(&bytes);
        let id = created["id"].as_i64().unwrap();

        let (status, _, bytes) = request(
            database.clone(),
            Method::PUT,
            &format!("/api/notes/{id}"),
            Some(json!({"title":"stale","body":"x","expected_version":9})),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json_body(&bytes)["error"]["code"], "stale_version");

        let (status, _, _) = request(
            database.clone(),
            Method::POST,
            "/api/notes",
            Some(json!({"title":"browser","body":"x"})),
            Some("https://other.example"),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(database.list(10).await.unwrap().len(), 1);

        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/notes")
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from("{}"))
            .unwrap();
        let response = app(database).oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }

    #[tokio::test]
    async fn duplicate_titles_and_delete_conflicts_fail_without_partial_audit() {
        let (_directory, database) = fixture();
        let created = database
            .create("Unique".to_owned(), "one".to_owned())
            .await
            .unwrap();
        assert!(matches!(
            database.create("unique".to_owned(), "two".to_owned()).await,
            Err(DatabaseError::Sqlite(rusqlite::Error::SqliteFailure(error, _)))
                if error.code == ErrorCode::ConstraintViolation
        ));
        assert_eq!(audit_count(database.path()), 1);
        assert!(matches!(
            database.delete(created.id, 7).await,
            Err(DatabaseError::Conflict)
        ));
        assert_eq!(audit_count(database.path()), 1);
        assert_eq!(database.get(created.id).await.unwrap().version, 1);
    }

    #[tokio::test]
    async fn list_limits_and_native_routing_errors_are_bounded() {
        let (_directory, database) = fixture();
        let (status, _, _) = request(
            database.clone(),
            Method::GET,
            "/api/notes?limit=0",
            None,
            None,
        )
        .await;
        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        let (status, headers, _) =
            request(database.clone(), Method::POST, "/api/notes/1", None, None).await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
        assert!(headers.contains_key(header::ALLOW));
        let (status, _, _) = request(database, Method::GET, "/missing", None, None).await;
        assert_eq!(status, StatusCode::NOT_FOUND);
    }

    #[test]
    fn anonymous_sample_is_loopback_only() {
        assert!(listen_address("127.0.0.1:0").is_ok());
        assert!(listen_address("[::1]:0").is_ok());
        assert!(listen_address("0.0.0.0:3007").is_err());
        assert!(listen_address("192.0.2.1:3007").is_err());
    }
}
