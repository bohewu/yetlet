//! Application-owned SQLite actor used only by the public consumer tests.
//! All blocking database/queue work stays on its original owned thread.

use rusqlite::{Connection, params};
use std::{
    future::Future,
    io,
    path::PathBuf,
    pin::Pin,
    sync::{
        Arc, Condvar, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    task::{Context, Poll},
    thread::{self, JoinHandle, ThreadId},
    time::Duration,
};
use tokio::{
    sync::oneshot,
    time::{Sleep, sleep},
};
use yetlet::{
    composition::{HostRunMode, ManagedResource, ReleaseCompletion},
    web::WebError,
};

#[derive(Clone)]
pub struct Gate(Arc<(Mutex<bool>, Condvar)>);
impl Gate {
    pub fn new(open: bool) -> Self {
        Self(Arc::new((Mutex::new(open), Condvar::new())))
    }
    pub fn open(&self) {
        let (state, wake) = &*self.0;
        *state.lock().unwrap() = true;
        wake.notify_all();
    }
    fn wait_on_actor(&self) {
        let (state, wake) = &*self.0;
        let mut open = state.lock().unwrap();
        while !*open {
            open = wake.wait(open).unwrap();
        }
    }
}

pub struct OpenOnDrop(pub Gate);
impl Drop for OpenOnDrop {
    fn drop(&mut self) {
        self.0.open();
    }
}

struct Insert {
    value: String,
    reply: oneshot::Sender<io::Result<i64>>,
}

#[derive(Clone)]
pub struct SqliteClient {
    sender: Arc<Mutex<Option<SyncSender<Insert>>>>,
}
impl SqliteClient {
    pub fn available(&self) -> bool {
        self.sender.lock().unwrap().is_some()
    }
    pub async fn insert(&self, value: &str) -> io::Result<i64> {
        let (reply, response) = oneshot::channel();
        {
            let sender = self.sender.lock().unwrap();
            let sender = sender.as_ref().ok_or_else(|| {
                io::Error::new(io::ErrorKind::NotConnected, "database admission closed")
            })?;
            sender
                .try_send(Insert {
                    value: value.into(),
                    reply,
                })
                .map_err(|error| match error {
                    TrySendError::Full(_) => {
                        io::Error::new(io::ErrorKind::WouldBlock, "database queue full")
                    }
                    TrySendError::Disconnected(_) => {
                        io::Error::new(io::ErrorKind::BrokenPipe, "database actor exited")
                    }
                })?;
        }
        response
            .await
            .map_err(|error| io::Error::new(io::ErrorKind::BrokenPipe, error))?
    }
}

#[derive(Default)]
struct Status {
    thread_id: OnceLock<ThreadId>,
    open: AtomicBool,
    release_started: AtomicBool,
    joined: AtomicBool,
    startup_error: Mutex<Option<WebError>>,
}

#[derive(Clone)]
pub struct SqliteStatus(Arc<Status>);
impl SqliteStatus {
    pub fn thread_id(&self) -> Option<ThreadId> {
        self.0.thread_id.get().copied()
    }
    pub fn is_open(&self) -> bool {
        self.0.open.load(Ordering::Acquire)
    }
    pub fn release_started(&self) -> bool {
        self.0.release_started.load(Ordering::Acquire)
    }
    pub fn joined(&self) -> bool {
        self.0.joined.load(Ordering::Acquire)
    }
    pub fn startup_error(&self) -> Option<WebError> {
        self.0.startup_error.lock().unwrap().clone()
    }
}

pub struct SqliteResource {
    path: PathBuf,
    client: SqliteClient,
    status: SqliteStatus,
    start_gate: Gate,
    close_gate: Gate,
    pending_sender: Option<SyncSender<Insert>>,
    ready: Option<oneshot::Receiver<Result<(), WebError>>>,
    completed: Option<oneshot::Receiver<()>>,
    worker: Option<JoinHandle<()>>,
    join_tick: Option<Pin<Box<Sleep>>>,
    attempted: bool,
    acquired: bool,
}

impl SqliteResource {
    pub fn new(path: impl Into<PathBuf>, start_gate: Gate, close_gate: Gate) -> Self {
        Self {
            path: path.into(),
            client: SqliteClient {
                sender: Arc::new(Mutex::new(None)),
            },
            status: SqliteStatus(Arc::new(Status::default())),
            start_gate,
            close_gate,
            pending_sender: None,
            ready: None,
            completed: None,
            worker: None,
            join_tick: None,
            attempted: false,
            acquired: false,
        }
    }
    pub fn client(&self) -> SqliteClient {
        self.client.clone()
    }
    pub fn status(&self) -> SqliteStatus {
        self.status.clone()
    }

    fn register_join_wake(&mut self, cx: &mut Context<'_>) {
        if let Some(completed) = self.completed.as_mut()
            && Pin::new(completed).poll(cx).is_ready()
        {
            self.completed = None;
        }
        // Thread completion notification precedes is_finished's final edge.
        // Retain a yielding timer as well, so cancellation cannot lose that wake.
        let elapsed = self
            .join_tick
            .as_mut()
            .is_some_and(|tick| tick.as_mut().poll(cx).is_ready());
        if self.join_tick.is_none() || elapsed {
            let mut tick = Box::pin(sleep(Duration::from_millis(1)));
            let _ = tick.as_mut().poll(cx);
            self.join_tick = Some(tick);
        }
    }
}

impl ManagedResource for SqliteResource {
    type Error = WebError;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), Self::Error> {
        if mode != HostRunMode::Async {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "this sample resource uses an async caller",
            )
            .into());
        }
        tokio::runtime::Handle::try_current()
            .map(|_| ())
            .map_err(|error| WebError::from(io::Error::other(error)))
    }

    fn poll_acquire(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if self.acquired {
            return Poll::Ready(Ok(()));
        }
        if !self.attempted {
            self.attempted = true;
            let (sender, commands) = mpsc::sync_channel(8);
            let (ready, ready_rx) = oneshot::channel();
            let (completed, completed_rx) = oneshot::channel();
            let path = self.path.clone();
            let status = self.status.clone();
            let start = self.start_gate.clone();
            let close = self.close_gate.clone();
            let worker = thread::Builder::new()
                .name("consumer-sqlite-owner".into())
                .spawn(move || {
                    let _completion = CompletionSignal(Some(completed));
                    run_actor(path, commands, ready, status, start, close);
                });
            let worker = match worker {
                Ok(worker) => worker,
                Err(error) => return Poll::Ready(Err(error.into())),
            };
            // Every original owner/receiver is installed before polling readiness.
            self.worker = Some(worker);
            self.pending_sender = Some(sender);
            self.ready = Some(ready_rx);
            self.completed = Some(completed_rx);
        }
        let readiness = Pin::new(self.ready.as_mut().expect("retained readiness")).poll(cx);
        match readiness {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                self.ready = None;
                match result {
                    Ok(Ok(())) => {
                        *self.client.sender.lock().unwrap() = self.pending_sender.take();
                        self.acquired = true;
                        Poll::Ready(Ok(()))
                    }
                    Ok(Err(error)) => {
                        self.pending_sender = None;
                        Poll::Ready(Err(error))
                    }
                    Err(error) => {
                        self.pending_sender = None;
                        Poll::Ready(Err(io::Error::new(io::ErrorKind::BrokenPipe, error).into()))
                    }
                }
            }
        }
    }

    fn poll_release(
        &mut self,
        cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<ReleaseCompletion<Self::Error>, Self::Error> {
        if !self.status.0.release_started.swap(true, Ordering::AcqRel) {
            self.client.sender.lock().unwrap().take();
            self.pending_sender = None;
            self.ready = None;
        }
        let Some(worker) = self.worker.as_ref() else {
            return Ok(ReleaseCompletion::Released(Ok(())));
        };
        if !worker.is_finished() {
            self.register_join_wake(cx);
            return Ok(ReleaseCompletion::Pending);
        }
        let result = self.worker.take().expect("original database thread").join();
        self.join_tick = None;
        self.completed = None;
        self.status.0.joined.store(true, Ordering::Release);
        match result {
            Ok(()) => Ok(ReleaseCompletion::Released(Ok(()))),
            Err(_) => Ok(ReleaseCompletion::Released(Err(io::Error::other(
                "database actor panicked",
            )
            .into()))),
        }
    }
}

struct CompletionSignal(Option<oneshot::Sender<()>>);
impl Drop for CompletionSignal {
    fn drop(&mut self) {
        if let Some(sender) = self.0.take() {
            let _ = sender.send(());
        }
    }
}

fn startup_failure(
    ready: oneshot::Sender<Result<(), WebError>>,
    status: &SqliteStatus,
    error: rusqlite::Error,
) {
    let error = WebError::from(io::Error::other(error));
    *status.0.startup_error.lock().unwrap() = Some(error.clone());
    let _ = ready.send(Err(error));
}

fn run_actor(
    path: PathBuf,
    commands: Receiver<Insert>,
    ready: oneshot::Sender<Result<(), WebError>>,
    status: SqliteStatus,
    start: Gate,
    close: Gate,
) {
    let _ = status.0.thread_id.set(thread::current().id());
    start.wait_on_actor();
    let connection = match Connection::open(path) {
        Ok(connection) => connection,
        Err(error) => {
            startup_failure(ready, &status, error);
            return;
        }
    };
    status.0.open.store(true, Ordering::Release);
    if let Err(error) = connection.execute(
        "CREATE TABLE records(id INTEGER PRIMARY KEY, value TEXT NOT NULL)",
        [],
    ) {
        startup_failure(ready, &status, error);
    } else if ready.send(Ok(())).is_ok() {
        while let Ok(command) = commands.recv() {
            let result = connection
                .execute(
                    "INSERT INTO records(value) VALUES (?1)",
                    params![command.value],
                )
                .map(|_| connection.last_insert_rowid())
                .map_err(io::Error::other);
            let _ = command.reply.send(result);
        }
    }
    close.wait_on_actor();
    drop(connection);
    status.0.open.store(false, Ordering::Release);
}
