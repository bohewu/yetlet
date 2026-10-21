//! Application-owned SQLite persistence; no connections survive an operation.

use std::{
    fmt,
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use rusqlite::{
    Connection, ErrorCode, OpenFlags, OptionalExtension, Row, TransactionBehavior, params,
};
use serde::{Deserialize, Serialize};
use tokio::sync::Semaphore;

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

const MIGRATION: &str = include_str!("../migrations/0001_init.sql");
const SCHEMA_VERSION: i64 = 1;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IssueStatus {
    Open,
    InProgress,
    Closed,
}

impl IssueStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Closed => "closed",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Issue {
    pub id: i64,
    pub title: String,
    pub body: String,
    pub status: IssueStatus,
    pub version: i64,
}

/// Deliberately contains no database messages, paths, SQL, or input values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreError {
    NotFound,
    Conflict,
    Unavailable,
    InvalidSchema,
    Internal,
}

impl fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::NotFound => "issue was not found",
            Self::Conflict => "issue version is stale",
            Self::Unavailable => "issue storage is unavailable",
            Self::InvalidSchema => "issue storage schema is unsupported",
            Self::Internal => "issue storage operation failed",
        })
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        match error.sqlite_error_code() {
            Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked | ErrorCode::CannotOpen) => {
                Self::Unavailable
            }
            Some(ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase) => Self::InvalidSchema,
            _ => Self::Internal,
        }
    }
}

#[derive(Clone)]
pub struct Store {
    path: Arc<PathBuf>,
    gate: Arc<Semaphore>,
}

impl Store {
    /// Open and migrate an explicitly selected file before accepting requests.
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, StoreError> {
        let path = path.into();
        if path.as_os_str().is_empty()
            || path == Path::new(":memory:")
            || path.to_string_lossy().starts_with("file:")
            || path.is_dir()
        {
            return Err(StoreError::Unavailable);
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|_| StoreError::Unavailable)?;
        }
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        match options.open(&path) {
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err(StoreError::Unavailable),
        }
        let path = fs::canonicalize(path).map_err(|_| StoreError::Unavailable)?;
        let mut connection = open_connection(&path)?;
        migrate(&mut connection)?;
        verify_schema(&connection)?;
        Ok(Self {
            path: Arc::new(path),
            gate: Arc::new(Semaphore::new(4)),
        })
    }

    async fn run<T, F>(&self, operation: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    {
        let permit = self
            .gate
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| StoreError::Unavailable)?;
        let path = self.path.clone();
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let mut connection = open_connection(&path)?;
            operation(&mut connection)
        })
        .await
        .map_err(|_| StoreError::Unavailable)?
    }

    pub async fn ready(&self) -> Result<(), StoreError> {
        self.run(|connection| {
            verify_schema(connection)?;
            connection
                .query_row("SELECT id FROM issues LIMIT 1", [], |_| Ok(()))
                .optional()?;
            Ok(())
        })
        .await
    }

    pub async fn list(&self) -> Result<Vec<Issue>, StoreError> {
        self.run(|connection| {
            let mut statement = connection.prepare(
                "SELECT id, title, body, status, version FROM issues ORDER BY id DESC LIMIT 100",
            )?;
            let rows = statement.query_map([], issue_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(Into::into)
        })
        .await
    }

    pub async fn get(&self, id: i64) -> Result<Issue, StoreError> {
        self.run(move |connection| select_issue(connection, id)?.ok_or(StoreError::NotFound))
            .await
    }

    pub async fn create(&self, title: String, body: String) -> Result<Issue, StoreError> {
        self.run(move |connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute(
                "INSERT INTO issues(title, body) VALUES (?1, ?2)",
                params![title, body],
            )?;
            let issue = select_issue(&transaction, transaction.last_insert_rowid())?
                .ok_or(StoreError::Internal)?;
            transaction.commit()?;
            Ok(issue)
        })
        .await
    }

    pub async fn set_status(
        &self,
        id: i64,
        status: IssueStatus,
        expected_version: i64,
    ) -> Result<Issue, StoreError> {
        self.run(move |connection| {
            let transaction =
                connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
            let changed = transaction.execute(
                "UPDATE issues SET status=?1, version=version+1 WHERE id=?2 AND version=?3",
                params![status.as_str(), id, expected_version],
            )?;
            if changed == 0 {
                return Err(if select_issue(&transaction, id)?.is_some() {
                    StoreError::Conflict
                } else {
                    StoreError::NotFound
                });
            }
            let issue = select_issue(&transaction, id)?.ok_or(StoreError::Internal)?;
            transaction.commit()?;
            Ok(issue)
        })
        .await
    }
}

fn open_connection(path: &Path) -> Result<Connection, StoreError> {
    // CREATE is intentionally absent: failed readiness must not recreate a lost file.
    let connection = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    connection.busy_timeout(Duration::from_secs(2))?;
    Ok(connection)
}

fn migrate(connection: &mut Connection) -> Result<(), StoreError> {
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    match version {
        0 => {
            let tables: i64 = transaction.query_row(
                "SELECT count(*) FROM sqlite_schema WHERE name NOT LIKE 'sqlite_%'",
                [],
                |row| row.get(0),
            )?;
            if tables != 0 {
                return Err(StoreError::InvalidSchema);
            }
            transaction.execute_batch(MIGRATION)?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        }
        SCHEMA_VERSION => verify_schema(&transaction)?,
        _ => return Err(StoreError::InvalidSchema),
    }
    transaction.commit()?;
    let mode: String = connection.query_row("PRAGMA journal_mode=WAL", [], |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("wal") {
        return Err(StoreError::Unavailable);
    }
    Ok(())
}

fn verify_schema(connection: &Connection) -> Result<(), StoreError> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let schema: Option<String> = connection
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type='table' AND name='issues'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    // This application owns one fixed migration, including its CHECK constraints.
    // Comparing the DDL also rejects lookalike tables with missing constraints.
    let expected = MIGRATION.trim().trim_end_matches(';');
    if version != SCHEMA_VERSION || schema.as_deref() != Some(expected) {
        return Err(StoreError::InvalidSchema);
    }
    Ok(())
}

fn issue_from_row(row: &Row<'_>) -> rusqlite::Result<Issue> {
    let status: String = row.get(3)?;
    let status = match status.as_str() {
        "open" => IssueStatus::Open,
        "in_progress" => IssueStatus::InProgress,
        "closed" => IssueStatus::Closed,
        _ => return Err(rusqlite::Error::InvalidQuery),
    };
    Ok(Issue {
        id: row.get(0)?,
        title: row.get(1)?,
        body: row.get(2)?,
        status,
        version: row.get(4)?,
    })
}

fn select_issue(connection: &Connection, id: i64) -> Result<Option<Issue>, StoreError> {
    connection
        .query_row(
            "SELECT id, title, body, status, version FROM issues WHERE id=?1",
            params![id],
            issue_from_row,
        )
        .optional()
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn migration_reopen_and_bound_parameters_preserve_data() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("issues.sqlite3");
        let store = Store::open(&path).unwrap();
        let title = "Robert'); DROP TABLE issues;-- <script>".to_owned();
        let body = "中文 ' ?1 & <img src=x onerror=alert(1)>".to_owned();
        let issue = store.create(title.clone(), body.clone()).await.unwrap();
        assert_eq!(issue.title, title);
        assert_eq!(issue.body, body);
        assert_eq!(issue.status, IssueStatus::Open);
        assert_eq!(issue.version, 1);
        drop(store);
        let reopened = Store::open(&path).unwrap();
        assert_eq!(reopened.get(issue.id).await.unwrap(), issue);
        reopened.ready().await.unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[test]
    fn startup_rejects_nonfiles_newer_and_malformed_schema() {
        assert!(Store::open(":memory:").is_err());
        assert!(Store::open("").is_err());
        let directory = tempfile::tempdir().unwrap();
        assert!(Store::open(directory.path()).is_err());
        let newer = directory.path().join("newer.sqlite3");
        Store::open(&newer).unwrap();
        let connection = open_connection(&newer).unwrap();
        connection.pragma_update(None, "user_version", 99).unwrap();
        drop(connection);
        assert!(matches!(Store::open(newer), Err(StoreError::InvalidSchema)));

        let malformed = directory.path().join("malformed.sqlite3");
        let connection = Connection::open(&malformed).unwrap();
        connection.execute_batch(
            "CREATE TABLE issues(id INTEGER PRIMARY KEY, title TEXT, body TEXT, status TEXT, version INTEGER); PRAGMA user_version=1;",
        ).unwrap();
        drop(connection);
        assert!(matches!(
            Store::open(malformed),
            Err(StoreError::InvalidSchema)
        ));
    }

    #[tokio::test]
    async fn concurrent_updates_compare_versions_and_distinguish_missing() {
        let directory = tempfile::tempdir().unwrap();
        let store = Store::open(directory.path().join("issues.sqlite3")).unwrap();
        let issue = store
            .create("An issue".into(), String::new())
            .await
            .unwrap();
        let (first, second) = tokio::join!(
            store.set_status(issue.id, IssueStatus::InProgress, 1),
            store.set_status(issue.id, IssueStatus::Closed, 1),
        );
        assert_eq!(usize::from(first.is_ok()) + usize::from(second.is_ok()), 1);
        assert!(
            matches!(first, Err(StoreError::Conflict))
                || matches!(second, Err(StoreError::Conflict))
        );
        assert_eq!(store.get(issue.id).await.unwrap().version, 2);
        assert_eq!(
            store.set_status(999, IssueStatus::Closed, 1).await,
            Err(StoreError::NotFound)
        );
        assert_eq!(store.get(999).await, Err(StoreError::NotFound));
    }

    #[tokio::test]
    async fn list_is_bounded_and_newest_first_and_sql_enforces_constraints() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("issues.sqlite3");
        let store = Store::open(&path).unwrap();
        let connection = open_connection(&path).unwrap();
        for _ in 0..105 {
            connection
                .execute("INSERT INTO issues(title, body) VALUES ('issue', '')", [])
                .unwrap();
        }
        assert!(
            connection
                .execute("INSERT INTO issues(title, body) VALUES ('', '')", [])
                .is_err()
        );
        assert!(
            connection
                .execute("UPDATE issues SET status='unknown'", [])
                .is_err()
        );
        assert!(
            connection
                .execute("UPDATE issues SET version=0", [])
                .is_err()
        );
        assert!(
            connection
                .execute("UPDATE issues SET title=?1", ["x".repeat(121)])
                .is_err()
        );
        assert!(
            connection
                .execute("UPDATE issues SET body=?1", ["x".repeat(4001)])
                .is_err()
        );
        drop(connection);
        let issues = store.list().await.unwrap();
        assert_eq!(issues.len(), 100);
        assert_eq!(issues.first().unwrap().id, 105);
        assert_eq!(issues.last().unwrap().id, 6);
    }

    #[tokio::test]
    async fn readiness_detects_schema_damage_and_missing_file_without_recreating() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("issues.sqlite3");
        let store = Store::open(&path).unwrap();
        let connection = open_connection(&path).unwrap();
        connection.execute_batch("DROP TABLE issues").unwrap();
        drop(connection);
        assert_eq!(store.ready().await, Err(StoreError::InvalidSchema));
        fs::remove_file(&path).unwrap();
        assert_eq!(store.ready().await, Err(StoreError::Unavailable));
        assert!(!path.exists());
    }
}
