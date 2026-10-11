//! Unpublished, single-primary semantics experiment; not a Session provider.
pub mod cookies;
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::{Semaphore, oneshot},
    task::JoinHandle,
    time::{Instant, timeout, timeout_at},
};

const SCRIPT: &str = include_str!("session.lua");
pub const CALL_TIMEOUT: Duration = Duration::from_millis(400);
pub const WAIT_TIMEOUT: Duration = Duration::from_millis(80);
pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);
pub const PAYLOAD_LIMIT: usize = 1024;
pub const OPERATION_TTL_MS: u64 = 5000;
pub const TOMBSTONE_TTL_MS: u64 = 400;
pub const MAX_SESSION_TTL_MS: u64 = 60000;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub count: u64,
    pub note: String,
}
impl State {
    pub fn new(count: u64) -> Self {
        Self {
            count,
            note: String::new(),
        }
    }
}

pub struct Snapshot {
    pub state: State,
    pub revision: u64,
    pub expires: u64,
    pub generation: u64,
    pub issuance: u64,
    pub csrf: String,
}

#[derive(Clone)]
pub struct Ticket {
    keys: Vec<String>,
    args: Vec<String>,
    identity: String,
    replacement: Option<String>,
}
impl fmt::Debug for Ticket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Ticket([redacted])")
    }
}
impl Ticket {
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn replacement(&self) -> Option<&str> {
        self.replacement.as_deref()
    }
}

#[derive(Debug)]
pub enum Error {
    Configuration,
    Busy,
    QueueTimeout,
    Closed,
    Unavailable,
    InvalidRecord,
    PayloadTooLarge,
    Codec,
    Protocol,
    Lifecycle(String),
    OutcomeUnknown(Ticket),
    Conflict { expected: u64, observed: u64 },
}
#[derive(Debug)]
pub enum UpdateError<E> {
    Application(E),
    Store(Error),
}
#[derive(Debug, PartialEq, Eq)]
pub struct Mutation {
    pub status: String,
    pub revision: u64,
}

enum Query {
    Command(redis::Cmd),
    Script(Vec<String>, Vec<String>),
}
struct Tasks {
    manager: Option<ConnectionManager>,
    handles: Vec<JoinHandle<()>>,
}
struct Inner {
    tasks: Mutex<Tasks>,
    active: Arc<Semaphore>,
    admission: Arc<Semaphore>,
    peak: AtomicUsize,
    running: AtomicUsize,
}
#[derive(Clone)]
pub struct Store {
    inner: Arc<Inner>,
    prefix: String,
}

pub fn random_id() -> Result<String, Error> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|_| Error::Unavailable)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
fn valid_id(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn strings(value: redis::Value) -> Result<Vec<String>, Error> {
    redis::from_redis_value(value).map_err(|_| Error::Protocol)
}
fn mutation(value: redis::Value) -> Result<Mutation, Error> {
    let row = strings(value)?;
    if row.len() != 2 {
        return Err(Error::Protocol);
    }
    Ok(Mutation {
        status: row[0].clone(),
        revision: row[1].parse().map_err(|_| Error::Protocol)?,
    })
}

impl Store {
    /// The experiment deliberately accepts only a loopback, unauthenticated fixture URL.
    pub async fn connect(url: &str, namespace: &str, concurrency: usize) -> Result<Self, Error> {
        let Some(port) = url
            .strip_prefix("redis://127.0.0.1:")
            .and_then(|s| s.strip_suffix('/'))
        else {
            return Err(Error::Configuration);
        };
        if port.parse::<u16>().ok().filter(|p| *p != 0).is_none()
            || !valid_id(namespace)
            || !(1..=4).contains(&concurrency)
        {
            return Err(Error::Configuration);
        }
        let client = redis::Client::open(url).map_err(|_| Error::Configuration)?;
        let config = ConnectionManagerConfig::new()
            .set_number_of_retries(1)
            .set_min_delay(Duration::from_millis(10))
            .set_max_delay(Duration::from_millis(20))
            .set_connection_timeout(Some(Duration::from_millis(200)))
            // A caller timeout must not free a slot while a reply is still outstanding.
            // Owned workers retain slots until reply/disconnect, or shutdown reaps them.
            .set_response_timeout(None)
            .set_pipeline_buffer_size(concurrency)
            .set_concurrency_limit(concurrency);
        let manager = timeout(
            CALL_TIMEOUT,
            client.get_connection_manager_with_config(config),
        )
        .await
        .map_err(|_| Error::Unavailable)?
        .map_err(|_| Error::Unavailable)?;
        Ok(Self {
            prefix: format!("yt066b1:{namespace}:"),
            inner: Arc::new(Inner {
                tasks: Mutex::new(Tasks {
                    manager: Some(manager),
                    handles: vec![],
                }),
                active: Arc::new(Semaphore::new(concurrency)),
                admission: Arc::new(Semaphore::new(concurrency * 2)),
                peak: AtomicUsize::new(0),
                running: AtomicUsize::new(0),
            }),
        })
    }

    async fn call(&self, query: Query) -> Result<redis::Value, Error> {
        let end = Instant::now() + CALL_TIMEOUT;
        let admission = self
            .inner
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::Busy)?;
        let active = timeout(WAIT_TIMEOUT, self.inner.active.clone().acquire_owned())
            .await
            .map_err(|_| Error::QueueTimeout)?
            .map_err(|_| Error::Closed)?;
        let (send, recv) = oneshot::channel();
        {
            let mut tasks = self.inner.tasks.lock().map_err(|_| Error::Unavailable)?;
            let mut manager = tasks.manager.clone().ok_or(Error::Closed)?;
            tasks.handles.retain(|h| !h.is_finished());
            let inner = self.inner.clone();
            tasks.handles.push(tokio::spawn(async move {
                struct Running(Arc<Inner>);
                impl Drop for Running {
                    fn drop(&mut self) {
                        self.0.running.fetch_sub(1, Ordering::SeqCst);
                    }
                }
                // Drop the count before releasing permits, so the next admitted worker
                // cannot observe a transient overcount during the previous worker's exit.
                let _permits = (admission, active);
                let count = inner.running.fetch_add(1, Ordering::SeqCst) + 1;
                inner.peak.fetch_max(count, Ordering::SeqCst);
                let _running = Running(inner);
                let result: redis::RedisResult<redis::Value> = match query {
                    Query::Command(command) => command.query_async(&mut manager).await,
                    Query::Script(keys, args) => {
                        let script = redis::Script::new(SCRIPT);
                        let mut invocation = script.prepare_invoke();
                        for key in keys {
                            invocation.key(key);
                        }
                        for arg in args {
                            invocation.arg(arg);
                        }
                        invocation.invoke_async(&mut manager).await
                    }
                };
                let _ = send.send(result.map_err(|_| Error::Unavailable));
            }));
        }
        timeout_at(end, recv)
            .await
            .map_err(|_| Error::Unavailable)?
            .map_err(|_| Error::Unavailable)?
    }

    pub fn peak(&self) -> usize {
        self.inner.peak.load(Ordering::SeqCst)
    }
    pub fn running(&self) -> usize {
        self.inner.running.load(Ordering::SeqCst)
    }
    pub fn record_key(&self, id: &str) -> Result<String, Error> {
        if !valid_id(id) {
            return Err(Error::Configuration);
        }
        Ok(format!("{}session:{id}", self.prefix))
    }
    /// Fixture-only commands permit deterministic fault injection; no public provider API.
    pub async fn command(&self, command: redis::Cmd) -> Result<redis::Value, Error> {
        self.call(Query::Command(command)).await
    }
    fn keys(&self, id: &str) -> Result<Vec<String>, Error> {
        Ok(vec![
            self.record_key(id)?,
            format!("{}unused-result", self.prefix),
            format!("{}unused-replacement", self.prefix),
            format!("{}operations", self.prefix),
            format!("{}issuance", self.prefix),
        ])
    }
    fn args(kind: &str) -> Vec<String> {
        [
            kind, "", "0", "", "", "0", "2500", "5000", "400", "1024", "64",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }
    pub async fn read(&self, id: &str) -> Result<Snapshot, Error> {
        let row = strings(self.read_script(self.keys(id)?, Self::args("read")).await?)?;
        if row.first().map(String::as_str) != Some("ok") {
            let code = row.first().cloned().ok_or(Error::Protocol)?;
            return Err(if code == "invalid" {
                Error::InvalidRecord
            } else {
                Error::Lifecycle(code)
            });
        }
        if row.len() != 7 || row[2].len() > PAYLOAD_LIMIT || !valid_id(&row[3]) {
            return Err(Error::InvalidRecord);
        }
        Ok(Snapshot {
            state: serde_json::from_str(&row[2]).map_err(|_| Error::Codec)?,
            revision: row[1].parse().map_err(|_| Error::InvalidRecord)?,
            csrf: row[3].clone(),
            expires: row[4].parse().map_err(|_| Error::InvalidRecord)?,
            generation: row[5].parse().map_err(|_| Error::InvalidRecord)?,
            issuance: row[6].parse().map_err(|_| Error::InvalidRecord)?,
        })
    }

    pub async fn prepare(
        &self,
        kind: &str,
        id: Option<&str>,
        revision: u64,
        state: &State,
        ttl_ms: u64,
    ) -> Result<Ticket, Error> {
        if !matches!(kind, "create" | "update" | "rotate" | "replace" | "revoke")
            || !(1..=60000).contains(&ttl_ms)
        {
            return Err(Error::Configuration);
        }
        // Create always allocates a new identity; it cannot adopt an unknown browser ID.
        let identity = if kind == "create" {
            if id.is_some() {
                return Err(Error::Configuration);
            }
            random_id()?
        } else {
            id.ok_or(Error::Configuration)?.to_owned()
        };
        let mut keys = self.keys(&identity)?;
        keys[1] = format!("{}operation:{}", self.prefix, random_id()?);
        let replacement = if matches!(kind, "rotate" | "replace") {
            Some(random_id()?)
        } else {
            None
        };
        if let Some(id) = &replacement {
            keys[2] = self.record_key(id)?;
        }
        let mut args = Self::args(kind);
        args[2] = revision.to_string();
        args[3] = serde_json::to_string(state).map_err(|_| Error::Codec)?;
        if args[3].len() > PAYLOAD_LIMIT {
            return Err(Error::PayloadTooLarge);
        }
        args[4] = random_id()?;
        let time = strings(self.command(redis::cmd("TIME")).await?)?;
        if time.len() != 2 {
            return Err(Error::Protocol);
        }
        let now = time[0].parse::<u64>().map_err(|_| Error::Protocol)? * 1000
            + time[1].parse::<u64>().map_err(|_| Error::Protocol)? / 1000;
        args[5] = (now + 1000).to_string();
        args[6] = ttl_ms.to_string();
        args[1] = format!(
            "{:x}",
            Sha256::digest(serde_json::to_vec(&(&keys, &args)).map_err(|_| Error::Codec)?)
        );
        Ok(Ticket {
            keys,
            args,
            identity,
            replacement,
        })
    }
    pub async fn execute(&self, ticket: &Ticket) -> Result<Mutation, Error> {
        match self
            .call(Query::Script(ticket.keys.clone(), ticket.args.clone()))
            .await
        {
            Ok(value) => mutation(value).map_err(|_| Error::OutcomeUnknown(ticket.clone())),
            Err(e @ (Error::Busy | Error::QueueTimeout | Error::Closed)) => Err(e),
            Err(_) => Err(Error::OutcomeUnknown(ticket.clone())),
        }
    }
    pub async fn resolve(&self, ticket: &Ticket) -> Result<Mutation, Error> {
        let mut args = ticket.args.clone();
        args.push(args[0].clone());
        args[0] = "resolve".into();
        mutation(self.read_script(ticket.keys.clone(), args).await?)
    }
    async fn read_script(
        &self,
        keys: Vec<String>,
        args: Vec<String>,
    ) -> Result<redis::Value, Error> {
        let mut command = redis::cmd("EVAL_RO");
        command.arg(SCRIPT).arg(keys.len()).arg(keys).arg(args);
        self.command(command).await
    }
    pub async fn try_update<R, E>(
        &self,
        id: &str,
        action: impl FnOnce(&mut State) -> Result<R, E>,
    ) -> Result<R, UpdateError<E>> {
        let snapshot = self.read(id).await.map_err(UpdateError::Store)?;
        let mut candidate = snapshot.state;
        let value = action(&mut candidate).map_err(UpdateError::Application)?;
        let ticket = self
            .prepare("update", Some(id), snapshot.revision, &candidate, 2500)
            .await
            .map_err(UpdateError::Store)?;
        let result = self.execute(&ticket).await.map_err(UpdateError::Store)?;
        match result.status.as_str() {
            "ok" => Ok(value),
            "conflict" => Err(UpdateError::Store(Error::Conflict {
                expected: snapshot.revision,
                observed: result.revision,
            })),
            _ => Err(UpdateError::Store(Error::Lifecycle(result.status))),
        }
    }

    /// Close admission, drain outstanding replies, then abort/join any owned workers.
    /// Dropping every canonical manager clone also drops its redis-rs driver handles.
    pub async fn shutdown(&self) -> Result<usize, Error> {
        self.inner.active.close();
        self.inner.admission.close();
        let handles = {
            let mut tasks = self.inner.tasks.lock().map_err(|_| Error::Unavailable)?;
            tasks.manager.take();
            std::mem::take(&mut tasks.handles)
        };
        let end = Instant::now() + DRAIN_TIMEOUT;
        let mut reaped = 0;
        for mut handle in handles {
            if timeout_at(end, &mut handle).await.is_err() {
                handle.abort();
                let _ = handle.await;
                reaped += 1;
            }
        }
        Ok(reaped)
    }
}
