//! Private SQLite dependency used by the B3 example and integration tests.
//!
//! One owned actor thread opens and uses the connection. Clients submit bounded
//! commands without cloning a connection or an independent sender. Final release
//! disconnects the sole shared sender, drains queued commands, and joins the
//! original actor. This is an explicit adapter choice, not a pool or a global
//! executor requirement. Drop does not perform managed shutdown or join.

use rusqlite::{Connection, params};
use std::{
    any::Any,
    io,
    path::PathBuf,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle, ThreadId},
    time::Duration,
};
use tokio::sync::oneshot;
use yetlet_host_lifecycle_research::{ManagedResource, ReleaseCompletion};

type Reply = oneshot::Sender<io::Result<i64>>;

enum Command {
    Insert { value: String, reply: Reply },
    Count { reply: Reply },
}

/// A request capability. Clones share the single sender slot; they cannot keep
/// an independent sender alive after the owner starts release.
#[derive(Clone)]
pub struct SqliteClient {
    sender: Arc<Mutex<Option<SyncSender<Command>>>>,
}

impl SqliteClient {
    /// Whether the owner currently publishes a request sender, not actor liveness.
    pub fn available(&self) -> bool {
        self.sender.lock().is_ok_and(|sender| sender.is_some())
    }

    pub async fn insert(&self, value: String) -> io::Result<i64> {
        let (reply, response) = oneshot::channel();
        self.submit(Command::Insert { value, reply })?;
        response
            .await
            .map_err(|error| io::Error::new(io::ErrorKind::BrokenPipe, error))?
    }

    pub async fn count(&self) -> io::Result<i64> {
        let (reply, response) = oneshot::channel();
        self.submit(Command::Count { reply })?;
        response
            .await
            .map_err(|error| io::Error::new(io::ErrorKind::BrokenPipe, error))?
    }

    fn submit(&self, command: Command) -> io::Result<()> {
        let sender = self
            .sender
            .lock()
            .map_err(|_| io::Error::other("SQLite request sender lock poisoned"))?;
        let sender = sender.as_ref().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotConnected,
                "SQLite requests are unavailable",
            )
        })?;
        sender.try_send(command).map_err(|error| match error {
            TrySendError::Full(_) => {
                io::Error::new(io::ErrorKind::WouldBlock, "SQLite command queue is full")
            }
            TrySendError::Disconnected(_) => {
                io::Error::new(io::ErrorKind::BrokenPipe, "SQLite actor has ended")
            }
        })
    }
}

#[derive(Default)]
struct Status {
    open: AtomicBool,
    release_started: AtomicBool,
    joined: AtomicBool,
    thread_id: OnceLock<ThreadId>,
}

/// Metadata only. These observations do not own a connection, sender, or worker.
#[derive(Clone)]
pub struct SqliteStatus(Arc<Status>);

impl SqliteStatus {
    /// An opened connection is still owned, including a failed-close retry.
    /// This does not by itself establish schema readiness or actor liveness.
    pub fn is_open(&self) -> bool {
        self.0.open.load(Ordering::Acquire)
    }

    pub fn release_started(&self) -> bool {
        self.0.release_started.load(Ordering::Acquire)
    }

    /// True only after the original acquired actor handle has actually joined.
    /// A returned connection can still need close retry after this becomes true.
    pub fn joined(&self) -> bool {
        self.0.joined.load(Ordering::Acquire)
    }

    /// The original actor identity, retained as metadata after it exits.
    pub fn thread_id(&self) -> Option<ThreadId> {
        self.0.thread_id.get().copied()
    }
}

enum WorkerExit {
    Closed,
    CloseFailed {
        connection: Connection,
        error: io::Error,
    },
}

/// Inert until acquisition. The actor is acquired before its dependent WebHost.
/// The caller must retain and drive this owner through Pending and close errors.
pub struct SqliteResource {
    path: PathBuf,
    client: SqliteClient,
    status: SqliteStatus,
    worker: Option<JoinHandle<WorkerExit>>,
    close_retry: Option<Connection>,
    attempted: bool,
}

impl SqliteResource {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            client: SqliteClient {
                sender: Arc::new(Mutex::new(None)),
            },
            status: SqliteStatus(Arc::new(Status::default())),
            worker: None,
            close_retry: None,
            attempted: false,
        }
    }

    pub fn client(&self) -> SqliteClient {
        self.client.clone()
    }

    pub fn status(&self) -> SqliteStatus {
        self.status.clone()
    }

    fn accept_exit(&mut self, exit: WorkerExit) -> io::Result<ReleaseCompletion<io::Error>> {
        match exit {
            WorkerExit::Closed => Ok(ReleaseCompletion::Released(Ok(()))),
            WorkerExit::CloseFailed { connection, error } => {
                // Transfer the original returned connection before propagating
                // its progress error. A later call retries that same owner.
                self.close_retry = Some(connection);
                Err(error)
            }
        }
    }
}

impl ManagedResource for SqliteResource {
    fn acquire(&mut self) -> io::Result<()> {
        if self.attempted {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "SQLite acquisition was already attempted",
            ));
        }
        self.attempted = true;
        let (sender, commands) = mpsc::sync_channel(32);
        let (ready, observed_ready) = mpsc::sync_channel(1);
        let path = self.path.clone();
        let status = self.status.clone();
        self.worker = Some(
            thread::Builder::new()
                .name("yetlet-sqlite-resource".into())
                .spawn(move || run_worker(path, commands, ready, status))?,
        );
        // The original handle is already retained if readiness fails or the
        // sender cannot be published. Dropping the local sender disconnects it.
        observed_ready
            .recv()
            .map_err(|error| io::Error::new(io::ErrorKind::BrokenPipe, error))??;
        *self
            .client
            .sender
            .lock()
            .map_err(|_| io::Error::other("SQLite request sender lock poisoned"))? = Some(sender);
        Ok(())
    }

    fn release_within(&mut self, _budget: Duration) -> io::Result<ReleaseCompletion<io::Error>> {
        if !self.status.release_started() {
            // Taking the sole sender is independent of queue capacity. The
            // guarded Option remains valid even if a previous lock was poisoned.
            let sender = self
                .client
                .sender
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .take();
            drop(sender);
            self.status.0.release_started.store(true, Ordering::Release);
            if self.worker.is_some() {
                // Completion has not yet been observed through the original
                // handle. This does not assert that SQLite is still open.
                return Ok(ReleaseCompletion::Pending);
            }
        }

        if let Some(connection) = self.close_retry.take() {
            let exit = close_connection(connection, &self.status);
            return self.accept_exit(exit);
        }
        let Some(worker) = self.worker.as_ref() else {
            return Ok(ReleaseCompletion::Released(Ok(())));
        };
        if !worker.is_finished() {
            return Ok(ReleaseCompletion::Pending);
        }

        let outcome = self.worker.take().expect("original SQLite actor").join();
        self.status.0.joined.store(true, Ordering::Release);
        match outcome {
            Ok(exit) => self.accept_exit(exit),
            Err(payload) => {
                // The original thread has ended and unwound its owned connection.
                self.status.0.open.store(false, Ordering::Release);
                Ok(ReleaseCompletion::Released(Err(panic_error(payload))))
            }
        }
    }
}

fn run_worker(
    path: PathBuf,
    commands: Receiver<Command>,
    ready: SyncSender<io::Result<()>>,
    status: SqliteStatus,
) -> WorkerExit {
    let _ = status.0.thread_id.set(thread::current().id());
    let connection = match Connection::open(path) {
        Ok(connection) => connection,
        Err(error) => {
            let _ = ready.send(Err(io::Error::other(error)));
            return WorkerExit::Closed;
        }
    };
    status.0.open.store(true, Ordering::Release);
    if let Err(error) = connection.execute(
        "CREATE TABLE IF NOT EXISTS records(id INTEGER PRIMARY KEY,value TEXT NOT NULL)",
        [],
    ) {
        let _ = ready.send(Err(io::Error::other(error)));
        return close_connection(connection, &status);
    }
    if ready.send(Ok(())).is_err() {
        return close_connection(connection, &status);
    }

    // Disconnection drains commands accepted before the owner's release fence.
    while let Ok(command) = commands.recv() {
        match command {
            Command::Insert { value, reply } => {
                let result = connection
                    .execute("INSERT INTO records(value) VALUES (?1)", params![value])
                    .map(|_| connection.last_insert_rowid())
                    .map_err(io::Error::other);
                let _ = reply.send(result);
            }
            Command::Count { reply } => {
                let result = connection
                    .query_row("SELECT COUNT(*) FROM records", [], |row| row.get(0))
                    .map_err(io::Error::other);
                let _ = reply.send(result);
            }
        }
    }
    close_connection(connection, &status)
}

fn close_connection(connection: Connection, status: &SqliteStatus) -> WorkerExit {
    match connection.close() {
        Ok(()) => {
            status.0.open.store(false, Ordering::Release);
            WorkerExit::Closed
        }
        Err((connection, error)) => WorkerExit::CloseFailed {
            connection,
            error: io::Error::other(error),
        },
    }
}

fn panic_error(payload: Box<dyn Any + Send>) -> io::Error {
    let message = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("non-string panic payload");
    io::Error::other(format!("SQLite actor panicked: {message}"))
}
