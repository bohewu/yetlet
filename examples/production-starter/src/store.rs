//! Application-owned SQLite persistence; no connections survive an operation.

use std::{
    fmt,
    fs::{self, OpenOptions},
    io,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use crate::observability::{Active, Metrics};
use rusqlite::{
    Connection, ErrorCode, OpenFlags, OptionalExtension, Row, TransactionBehavior, params,
};
use serde::{Deserialize, Serialize};
use tokio::sync::{Notify, OwnedSemaphorePermit, Semaphore};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;

const MIGRATION: &str = include_str!("../migrations/0001_init.sql");
const STATUS_INDEX: &str = include_str!("../migrations/0002_status_index.sql");
const SCHEMA_VERSION: i64 = 2;
const SQL_CAPACITY: usize = 4;

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
    completed: Arc<Notify>,
    metrics: Option<Arc<Metrics>>,
}

// Owned by the closure, including time queued on Tokio's blocking pool. Release
// capacity only after the connection closes, also on errors and unwinding.
struct Completion {
    permit: Option<OwnedSemaphorePermit>,
    completed: Arc<Notify>,
    active: Option<Active>,
}
impl Drop for Completion {
    fn drop(&mut self) {
        drop(self.active.take());
        drop(self.permit.take());
        self.completed.notify_waiters();
    }
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
            gate: Arc::new(Semaphore::new(SQL_CAPACITY)),
            completed: Arc::new(Notify::new()),
            metrics: None,
        })
    }

    /// Attach app-owned collection before cloning the store into consumers.
    /// Existing clones retain their original collection choice.
    pub fn with_metrics(mut self, metrics: Arc<Metrics>) -> Self {
        self.metrics = Some(metrics);
        self
    }

    pub(crate) fn metrics(&self) -> Option<&Arc<Metrics>> {
        self.metrics.as_ref()
    }

    async fn run<T, F>(&self, operation: F) -> Result<T, StoreError>
    where
        T: Send + 'static,
        F: FnOnce(&mut Connection) -> Result<T, StoreError> + Send + 'static,
    {
        let permit = self.gate.clone().try_acquire_owned();
        if let Some(metrics) = &self.metrics {
            let outcome = match &permit {
                Ok(_) => "acquired",
                Err(tokio::sync::TryAcquireError::Closed) => "closed",
                Err(tokio::sync::TryAcquireError::NoPermits) => "saturated",
            };
            metrics.sql_admission.with_label_values(&[outcome]).inc();
            // No queue exists: do not mistake syscall time for permit waiting.
            metrics.sql_wait.observe(0.0);
        }
        let permit = permit.map_err(|_| StoreError::Unavailable)?;
        let completion = Completion {
            permit: Some(permit),
            completed: self.completed.clone(),
            active: self.metrics.as_ref().map(|m| Active::new(&m.sql_active)),
        };
        let path = self.path.clone();
        let metrics = self.metrics.clone();
        let span = tracing::Span::current();
        let dispatcher = tracing::dispatcher::get_default(Clone::clone);
        tokio::task::spawn_blocking(move || {
            tracing::dispatcher::with_default(&dispatcher, || {
                span.in_scope(|| {
                    let _completion = completion;
                    let result = (|| {
                        let mut connection = open_connection(&path)?;
                        operation(&mut connection)
                    })();
                    let outcome = match &result {
                        Ok(_) => "ok",
                        Err(StoreError::NotFound) => "not_found",
                        Err(StoreError::Conflict) => "conflict",
                        Err(StoreError::Unavailable) => "unavailable",
                        Err(StoreError::InvalidSchema) => "invalid_schema",
                        Err(StoreError::Internal) => "internal",
                    };
                    if let Some(metrics) = metrics {
                        metrics.sql_operations.with_label_values(&[outcome]).inc();
                    }
                    // Operation return is not an audit/durability acknowledgment.
                    tracing::info!(event = "sql_finished", outcome);
                    result
                })
            })
        })
        .await
        .map_err(|_| StoreError::Unavailable)?
    }

    /// Reject new work across all clones. Already admitted work may still commit.
    pub fn close(&self) {
        self.gate.close();
    }

    pub fn is_closed(&self) -> bool {
        self.gate.is_closed()
    }

    /// Close admission and wait for real blocking completion, not HTTP waiters.
    /// The application owns the outer shutdown grace deadline.
    pub async fn drain(&self) {
        self.close();
        loop {
            let notified = self.completed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            if self.gate.available_permits() == SQL_CAPACITY {
                return;
            }
            notified.await;
        }
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

    /// V2 status view: apply the filter before the existing 100-row limit.
    pub async fn list_by_status(&self, status: IssueStatus) -> Result<Vec<Issue>, StoreError> {
        self.run(move |connection| {
            let mut statement = connection.prepare(
                "SELECT id, title, body, status, version FROM issues WHERE status=?1 ORDER BY id DESC LIMIT 100",
            )?;
            let rows = statement.query_map([status.as_str()], issue_from_row)?;
            rows.collect::<rusqlite::Result<Vec<_>>>()
                .map_err(Into::into)
        })
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
        }
        1 => verify_issue_table(&transaction)?,
        SCHEMA_VERSION => verify_schema(&transaction)?,
        _ => return Err(StoreError::InvalidSchema),
    }
    if version < SCHEMA_VERSION {
        transaction.execute_batch(STATUS_INDEX)?;
        transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
        verify_schema(&transaction)?;
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
    verify_issue_table(connection)?;
    let index: Option<String> = connection
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type='index' AND name='issues_status_id'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    if version != SCHEMA_VERSION
        || index.as_deref() != Some(STATUS_INDEX.trim().trim_end_matches(';'))
    {
        return Err(StoreError::InvalidSchema);
    }
    Ok(())
}

fn verify_issue_table(connection: &Connection) -> Result<(), StoreError> {
    let schema: Option<String> = connection
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type='table' AND name='issues'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    // The historical table and its CHECK constraints remain unchanged in v2.
    // Comparing the DDL also rejects lookalike tables with missing constraints.
    let expected = MIGRATION.trim().trim_end_matches(';');
    if schema.as_deref() != Some(expected) {
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
    async fn v1_upgrade_preserves_all_fields_and_sequence_and_filters_before_limit() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("v1.sqlite3");
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch(MIGRATION).unwrap();
        connection.pragma_update(None, "user_version", 1).unwrap();
        connection
            .execute(
                "INSERT INTO issues(id,title,body,status,version) VALUES (42,?1,?2,'closed',7)",
                params!["old ' <title>", "Unicode 中文 & body"],
            )
            .unwrap();
        let before = select_issue(&connection, 42).unwrap().unwrap();
        drop(connection);
        let store = Store::open(&path).unwrap();
        assert_eq!(store.get(42).await.unwrap(), before);
        let connection = open_connection(&path).unwrap();
        assert_eq!(
            connection
                .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            connection
                .query_row(
                    "SELECT seq FROM sqlite_sequence WHERE name='issues'",
                    [],
                    |r| r.get::<_, i64>(0)
                )
                .unwrap(),
            42
        );
        for _ in 0..105 {
            connection
                .execute("INSERT INTO issues(title,body) VALUES ('new','')", [])
                .unwrap();
        }
        assert_eq!(
            store.list_by_status(IssueStatus::Closed).await.unwrap(),
            vec![before.clone()]
        );
        let open = store.list_by_status(IssueStatus::Open).await.unwrap();
        assert_eq!(open.len(), 100);
        assert_eq!(open[0].id, 147);
        assert_eq!(open[99].id, 48);
        let updated = store
            .set_status(42, IssueStatus::InProgress, 7)
            .await
            .unwrap();
        assert_eq!(updated.version, 8);
        assert_eq!(updated.title, before.title);
        assert_eq!(updated.body, before.body);
        drop(connection);
        drop(store);
        Store::open(&path).unwrap().ready().await.unwrap();
    }

    #[test]
    fn failed_upgrade_rolls_back_and_damaged_v2_index_is_refused() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("v1.sqlite3");
        let connection = Connection::open(&path).unwrap();
        connection.execute_batch(MIGRATION).unwrap();
        connection.execute_batch("PRAGMA user_version=1; INSERT INTO issues(title,body) VALUES ('retain','body'); CREATE INDEX issues_status_id ON issues(title);").unwrap();
        assert!(Store::open(&path).is_err());
        assert_eq!(
            connection
                .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            select_issue(&connection, 1).unwrap().unwrap().title,
            "retain"
        );
        connection
            .execute_batch("DROP INDEX issues_status_id")
            .unwrap();
        drop(connection);
        Store::open(&path).unwrap();
        let connection = open_connection(&path).unwrap();
        connection
            .execute_batch(
                "DROP INDEX issues_status_id; CREATE INDEX issues_status_id ON issues(title);",
            )
            .unwrap();
        assert!(matches!(Store::open(&path), Err(StoreError::InvalidSchema)));
        assert_eq!(select_issue(&connection, 1).unwrap().unwrap().body, "body");
    }

    #[tokio::test]
    async fn admission_has_no_queue_and_cancellation_keeps_blocking_capacity() {
        let directory = tempfile::tempdir().unwrap();
        let metrics = Arc::new(Metrics::new().unwrap());
        let store = Store::open(directory.path().join("issues.sqlite3"))
            .unwrap()
            .with_metrics(metrics.clone());
        let mut callers = Vec::new();
        let mut releases = Vec::new();
        for _ in 0..SQL_CAPACITY {
            let (started, received) = tokio::sync::oneshot::channel();
            let (release, wait) = std::sync::mpsc::channel();
            releases.push(release);
            let store = store.clone();
            callers.push(tokio::spawn(async move {
                store
                    .run(move |_| {
                        let _ = started.send(());
                        wait.recv_timeout(Duration::from_secs(10)).unwrap();
                        Ok(())
                    })
                    .await
            }));
            received.await.unwrap();
        }
        // All four closures are started, so these cannot wait or start work.
        for _ in 0..1000 {
            assert_eq!(store.list().await, Err(StoreError::Unavailable));
        }
        assert_eq!(store.ready().await, Err(StoreError::Unavailable));
        for caller in callers {
            caller.abort();
            assert!(caller.await.unwrap_err().is_cancelled());
        }
        assert_eq!(store.gate.available_permits(), 0);
        assert_eq!(metrics.sql_active.get(), 4);
        assert_eq!(
            metrics
                .sql_admission
                .with_label_values(&["saturated"])
                .get(),
            1001
        );
        assert_eq!(metrics.sql_wait.get_sample_count(), 1005);
        assert_eq!(metrics.sql_wait.get_sample_sum(), 0.0);
        for release in releases {
            release.send(()).unwrap();
        }
        tokio::time::timeout(Duration::from_secs(5), async {
            while store.gate.available_permits() != SQL_CAPACITY {
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        assert_eq!(metrics.sql_active.get(), 0);
        assert_eq!(metrics.sql_operations.with_label_values(&["ok"]).get(), 4);
        store.ready().await.unwrap();
        assert!(store.list().await.unwrap().is_empty());
        store.close();
        assert_eq!(store.ready().await, Err(StoreError::Unavailable));
        assert_eq!(
            metrics.sql_admission.with_label_values(&["closed"]).get(),
            1
        );
    }

    #[tokio::test]
    async fn timeout_write_commits_later_and_shutdown_waits_for_it() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("issues.sqlite3");
        let store = Store::open(&path).unwrap();
        // A future dropped before first poll has dispatched nothing.
        drop(store.create("never dispatched".into(), String::new()));
        assert!(store.list().await.unwrap().is_empty());
        let (started, received) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let worker_store = store.clone();
        let caller = tokio::spawn(async move {
            worker_store
                .run(move |connection| {
                    let _ = started.send(());
                    wait.recv_timeout(Duration::from_secs(10)).unwrap();
                    connection.execute(
                        "INSERT INTO issues(title, body) VALUES ('late write', '')",
                        [],
                    )?;
                    Ok(())
                })
                .await
        });
        received.await.unwrap();
        // Timeout drops this caller, but cannot abort the started SQL closure.
        let mut caller = caller;
        assert!(
            tokio::time::timeout(Duration::from_millis(10), &mut caller)
                .await
                .is_err()
        );
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        store.close();
        assert_eq!(store.list().await, Err(StoreError::Unavailable));
        assert_eq!(store.ready().await, Err(StoreError::Unavailable));
        let drain = store.drain();
        tokio::pin!(drain);
        assert!(
            tokio::time::timeout(Duration::from_millis(10), &mut drain)
                .await
                .is_err()
        );
        release.send(()).unwrap();
        tokio::time::timeout(Duration::from_secs(5), &mut drain)
            .await
            .unwrap();
        assert_eq!(store.gate.available_permits(), SQL_CAPACITY);
        let reopened = Store::open(&path).unwrap();
        let issues = reopened.list().await.unwrap();
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].title, "late write");
    }

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
