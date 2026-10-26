use crate::{
    Failure,
    domain::{Fields, History, ListQuery, Listing, Principal, Snapshot, WorkItem},
};
use rusqlite::{Connection, OptionalExtension, Row, TransactionBehavior, params};
use std::{
    path::Path,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::Semaphore;

#[cfg(feature = "commit-write-auth")]
mod write_authorization;

const WORKERS: usize = 4;
const MAX_ITEMS: i64 = 10_000;
const MAX_HISTORY: i64 = 100_000;

#[derive(Clone)]
pub(crate) struct Database {
    connection: Arc<Mutex<Connection>>,
    gate: Arc<Semaphore>,
    #[cfg(feature = "commit-write-auth")]
    write_authority: Option<Arc<()>>,
    #[cfg(feature = "request-tracing")]
    trace_requests: bool,
}
impl Database {
    pub async fn open(path: &Path) -> Result<Self, Failure> {
        Self::open_selected(path, false).await
    }
    #[cfg(feature = "commit-write-auth")]
    pub(crate) async fn open_with_write_authorization(path: &Path) -> Result<Self, Failure> {
        Self::open_selected(path, true).await
    }
    async fn open_selected(path: &Path, write_authorization: bool) -> Result<Self, Failure> {
        let path = path.to_owned();
        let connection = tokio::task::spawn_blocking(move || -> Result<Connection, Failure> {
            let fresh = match std::fs::symlink_metadata(&path) {
                Ok(m) if !m.is_file() || m.file_type().is_symlink() => {
                    return Err(Failure::Invalid);
                }
                Ok(_) => false,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    let mut options = std::fs::OpenOptions::new();
                    options.write(true).create_new(true);
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::OpenOptionsExt;
                        options.mode(0o600);
                    }
                    options.open(&path).map_err(|_| Failure::Unavailable)?;
                    true
                }
                Err(_) => return Err(Failure::Unavailable),
            };
            let mut c =
                Connection::open_with_flags(&path, rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE)?;
            c.busy_timeout(Duration::from_millis(250))?;
            c.pragma_update(None, "foreign_keys", "ON")?;
            let version: i64 = c.pragma_query_value(None, "user_version", |r| r.get(0))?;
            if (write_authorization
                && (!cfg!(feature = "commit-write-auth") || (!fresh && version != 2)))
                || (!write_authorization && version > 1)
            {
                return Err(Failure::Schema);
            }
            let _: String = c.query_row("PRAGMA journal_mode=WAL", [], |r| r.get(0))?;
            c.pragma_update(None, "synchronous", "FULL")?;
            #[cfg(feature = "commit-write-auth")]
            if write_authorization {
                write_authorization::prepare(&mut c, fresh)?;
            }
            if !write_authorization && version == 0 {
                let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
                tx.execute_batch(include_str!("../migrations/0001.sql"))?;
                tx.pragma_update(None, "user_version", 1)?;
                tx.commit()?;
            }
            for (table, expected) in [
                (
                    "work_items",
                    vec![
                        "id",
                        "project",
                        "title",
                        "description",
                        "status",
                        "priority",
                        "assignee",
                        "version",
                        "created_at",
                        "updated_at",
                    ],
                ),
                (
                    "history",
                    vec![
                        "id",
                        "item_id",
                        "project",
                        "version",
                        "actor",
                        "action",
                        "title",
                        "status",
                        "occurred_at",
                    ],
                ),
            ] {
                let mut stmt = c.prepare("SELECT name FROM pragma_table_info(?1) ORDER BY cid")?;
                let actual = stmt
                    .query_map([table], |r| r.get::<_, String>(0))?
                    .collect::<Result<Vec<_>, _>>()?;
                if actual != expected {
                    return Err(Failure::Schema);
                }
            }
            Ok(c)
        })
        .await
        .map_err(|_| Failure::Unavailable)??;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
            gate: Arc::new(Semaphore::new(WORKERS)),
            #[cfg(feature = "commit-write-auth")]
            write_authority: write_authorization.then(|| Arc::new(())),
            #[cfg(feature = "request-tracing")]
            trace_requests: false,
        })
    }
    #[cfg(feature = "request-tracing")]
    pub(crate) fn with_request_tracing(mut self) -> Self {
        self.trace_requests = true;
        self
    }
    pub async fn run<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, Failure> + Send + 'static,
    ) -> Result<T, Failure> {
        let permit =
            tokio::time::timeout(Duration::from_secs(1), self.gate.clone().acquire_owned())
                .await
                .map_err(|_| Failure::Busy)?
                .map_err(|_| Failure::Unavailable)?;
        let connection = self.connection.clone();
        #[cfg(feature = "request-tracing")]
        let trace = crate::request_tracing::WorkerTrace::capture(self.trace_requests);
        // The worker owns admission even if the waiting request is cancelled.
        tokio::task::spawn_blocking(move || {
            let _permit = permit;
            let run = || {
                let mut c = connection.lock().map_err(|_| Failure::Unavailable)?;
                operation(&mut c)
            };
            #[cfg(feature = "request-tracing")]
            if let Some(trace) = trace {
                return trace.run(run);
            }
            run()
        })
        .await
        .map_err(|_| Failure::Unavailable)?
    }
    pub async fn drain(&self) -> Result<(), Failure> {
        self.gate.close();
        tokio::time::timeout(Duration::from_secs(3), async {
            while self.gate.available_permits() != WORKERS {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| Failure::Busy)
    }
    pub async fn ready(&self) -> Result<(), Failure> {
        #[cfg(feature = "commit-write-auth")]
        if self.write_authority.is_some() {
            self.run(|c| write_authorization::validate(c)).await?;
        }
        self.run(|c| {
            c.query_row("SELECT COUNT(*) FROM work_items", [], |r| {
                r.get::<_, i64>(0)
            })?;
            Ok(())
        })
        .await
    }
    pub async fn list(&self, project: i64, query: ListQuery) -> Result<Listing, Failure> {
        self.run(move |c| {
            let tx = c.transaction()?;
            let filter = "project=?1 AND (?2='' OR instr(title,?2)>0) AND (?3='' OR status=?3)";
            let matching = tx.query_row(&format!("SELECT COUNT(*) FROM work_items WHERE {filter}"), params![project, query.q, query.status], |r| r.get(0))?;
            let order = match query.sort.as_str() { "title" => "title ASC,id ASC", "priority" => "CASE priority WHEN 'high' THEN 0 WHEN 'normal' THEN 1 ELSE 2 END,id DESC", _ => "updated_at DESC,id DESC" };
            let items = {
                let mut statement = tx.prepare(&format!("SELECT id,project,title,description,status,priority,assignee,version,created_at,updated_at FROM work_items WHERE {filter} ORDER BY {order} LIMIT 50 OFFSET ?4"))?;
                statement.query_map(params![project, query.q, query.status, i64::from(query.page - 1) * 50], item)?.collect::<Result<Vec<_>,_>>()?
            };
            let snapshot = snapshot(&tx, project)?;
            tx.commit()?;
            Ok(Listing { items, matching, snapshot })
        }).await
    }
    pub async fn get(&self, project: i64, id: i64) -> Result<(WorkItem, Vec<History>), Failure> {
        self.run(move |c| {
            let tx = c.transaction()?;
            let value = get_item(&tx, project, id)?.ok_or(Failure::NotFound)?;
            let history = {
                let mut stmt = tx.prepare("SELECT version,actor,action,title,status,occurred_at FROM history WHERE item_id=?1 AND project=?2 ORDER BY version DESC LIMIT 100")?;
                stmt.query_map(params![id,project], |r| Ok(History { version:r.get(0)?, actor:r.get(1)?, action:r.get(2)?, title:r.get(3)?, status:r.get(4)?, occurred_at:r.get(5)? }))?.collect::<Result<Vec<_>,_>>()?
            };
            tx.commit()?;
            Ok((value, history))
        }).await
    }
    pub async fn save(
        &self,
        principal: Principal,
        id: Option<i64>,
        expected: i64,
        fields: Fields,
    ) -> Result<WorkItem, Failure> {
        if !principal.write {
            return Err(Failure::Forbidden);
        }
        if !fields.errors(principal.project).is_empty() {
            return Err(Failure::Invalid);
        }
        #[cfg(feature = "commit-write-auth")]
        {
            let authorization = if self.write_authority.is_some() {
                Some(self.authorize_write(principal).await?)
            } else {
                None
            };
            self.save_checked(principal, id, expected, fields, authorization)
                .await
        }
        #[cfg(not(feature = "commit-write-auth"))]
        self.save_checked(principal, id, expected, fields).await
    }
    async fn save_checked(
        &self,
        principal: Principal,
        id: Option<i64>,
        expected: i64,
        fields: Fields,
        #[cfg(feature = "commit-write-auth")] authorization: Option<
            write_authorization::WriteAuthorization,
        >,
    ) -> Result<WorkItem, Failure> {
        #[cfg(feature = "commit-write-auth")]
        let authority = self.write_authority.clone();
        self.run(move |c| {
            let tx = c.transaction_with_behavior(TransactionBehavior::Immediate)?;
            #[cfg(feature = "commit-write-auth")]
            write_authorization::check(&tx, authority.as_ref(), authorization, principal)?;
            let value = apply_save(&tx, principal, id, expected, fields)?;
            tx.commit()?;
            Ok(value)
        })
        .await
    }
    pub async fn snapshot(&self, project: i64) -> Result<Snapshot, Failure> {
        self.run(move |c| snapshot(c, project)).await
    }
}
fn apply_save(
    tx: &rusqlite::Transaction<'_>,
    principal: Principal,
    id: Option<i64>,
    expected: i64,
    fields: Fields,
) -> Result<WorkItem, Failure> {
    let history_count: i64 = tx.query_row("SELECT COUNT(*) FROM history", [], |r| r.get(0))?;
    if history_count >= MAX_HISTORY {
        return Err(Failure::Capacity);
    }
    let (id, action) = match id {
        Some(id) => {
            let previous = get_item(tx, principal.project, id)?.ok_or(Failure::NotFound)?;
            if expected != previous.version || expected == i64::MAX {
                return Err(Failure::Conflict);
            }
            if previous.fields.status == "closed"
                && !matches!(fields.status.as_str(), "closed" | "open")
            {
                return Err(Failure::Transition);
            }
            let changed = tx.execute("UPDATE work_items SET title=?1,description=?2,status=?3,priority=?4,assignee=?5,version=version+1,updated_at=unixepoch() WHERE id=?6 AND project=?7 AND version=?8", params![fields.title,fields.description,fields.status,fields.priority,fields.assignee,id,principal.project,expected])?;
            if changed != 1 {
                return Err(Failure::Conflict);
            }
            (id, "update")
        }
        None => {
            if expected != 0 {
                return Err(Failure::Invalid);
            }
            let count: i64 = tx.query_row("SELECT COUNT(*) FROM work_items", [], |r| r.get(0))?;
            if count >= MAX_ITEMS {
                return Err(Failure::Capacity);
            }
            tx.execute("INSERT INTO work_items(project,title,description,status,priority,assignee) VALUES(?1,?2,?3,?4,?5,?6)", params![principal.project,fields.title,fields.description,fields.status,fields.priority,fields.assignee])?;
            (tx.last_insert_rowid(), "create")
        }
    };
    let value = get_item(tx, principal.project, id)?.ok_or(Failure::NotFound)?;
    tx.execute("INSERT INTO history(item_id,project,version,actor,action,title,status) VALUES(?1,?2,?3,?4,?5,?6,?7)", params![value.id,value.project,value.version,principal.name,action,value.fields.title,value.fields.status])?;
    Ok(value)
}
fn item(r: &Row<'_>) -> rusqlite::Result<WorkItem> {
    Ok(WorkItem {
        id: r.get(0)?,
        project: r.get(1)?,
        fields: Fields {
            title: r.get(2)?,
            description: r.get(3)?,
            status: r.get(4)?,
            priority: r.get(5)?,
            assignee: r.get(6)?,
        },
        version: r.get(7)?,
        created_at: r.get(8)?,
        updated_at: r.get(9)?,
    })
}
fn get_item(c: &Connection, project: i64, id: i64) -> Result<Option<WorkItem>, Failure> {
    Ok(c.query_row("SELECT id,project,title,description,status,priority,assignee,version,created_at,updated_at FROM work_items WHERE id=?1 AND project=?2", params![id,project], item).optional()?)
}
fn snapshot(c: &Connection, project: i64) -> Result<Snapshot, Failure> {
    Ok(c.query_row("SELECT (SELECT COALESCE(MAX(id),0) FROM history WHERE project=?1),COUNT(*),COALESCE(SUM(status='open'),0),COALESCE(SUM(status='active'),0),COALESCE(SUM(status='blocked'),0),COALESCE(SUM(status='closed'),0) FROM work_items WHERE project=?1", [project], |r| Ok(Snapshot { revision:r.get(0)?,total:r.get(1)?,open:r.get(2)?,active:r.get(3)?,blocked:r.get(4)?,closed:r.get(5)? }))?)
}
