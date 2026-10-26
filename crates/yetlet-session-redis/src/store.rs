//! Single-primary engine adapted from the reviewed private YT-066b1 spike.
use crate::{RedisOperationId, RedisSessionConfig, RedisSessionError as Error};
use redis::aio::{ConnectionManager, ConnectionManagerConfig};
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

#[cfg(test)]
mod admission_probe;

const SCRIPT: &str = include_str!("session.lua");
pub const CALL_TIMEOUT: Duration = Duration::from_millis(400);
pub const WAIT_TIMEOUT: Duration = Duration::from_millis(80);
pub const DRAIN_TIMEOUT: Duration = Duration::from_secs(2);

pub struct Snapshot {
    pub state: serde_json::Value,
    pub revision: u64,
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
    pub fn expected_revision(&self) -> u64 {
        self.args[2].parse().unwrap_or(0)
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn replacement(&self) -> Option<&str> {
        self.replacement.as_deref()
    }
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
    #[cfg(test)]
    probe: admission_probe::Probe,
}
#[derive(Clone)]
pub struct Store {
    inner: Arc<Inner>,
    prefix: String,
    config: RedisSessionConfig,
}

pub fn random_id() -> Result<String, Error> {
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|_| Error::Unavailable)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
pub fn valid_id(s: &str) -> bool {
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
    pub async fn connect(url: &str, config: RedisSessionConfig) -> Result<Self, Error> {
        if !endpoint_supported(url) {
            return Err(Error::Configuration);
        }
        let concurrency = config.max_in_flight;
        let client = redis::Client::open(url).map_err(|_| Error::Configuration)?;
        let connection_config = ConnectionManagerConfig::new()
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
            client.get_connection_manager_with_config(connection_config),
        )
        .await
        .map_err(|_| Error::Unavailable)?
        .map_err(|_| Error::Unavailable)?;
        Ok(Self {
            prefix: format!("yetlet:redis:v1:{}:", config.namespace),
            config,
            inner: Arc::new(Inner {
                tasks: Mutex::new(Tasks {
                    manager: Some(manager),
                    handles: vec![],
                }),
                active: Arc::new(Semaphore::new(concurrency)),
                admission: Arc::new(Semaphore::new(concurrency * 2)),
                peak: AtomicUsize::new(0),
                running: AtomicUsize::new(0),
                #[cfg(test)]
                probe: admission_probe::Probe::default(),
            }),
        })
    }

    async fn call(&self, query: Query, end: Instant) -> Result<redis::Value, Error> {
        #[cfg(test)]
        let mut probe = self.inner.probe.start();
        #[cfg(yetlet_session_admission_probe)]
        let mut trace = crate::admission_trace::Call::new();
        if Instant::now() >= end {
            #[cfg(test)]
            probe.failed(&Error::DispatchDeadline);
            #[cfg(yetlet_session_admission_probe)]
            trace.failed(&Error::DispatchDeadline);
            return Err(Error::DispatchDeadline);
        }
        let admission = self
            .inner
            .admission
            .clone()
            .try_acquire_owned()
            .map_err(|e| {
                let error = if matches!(e, tokio::sync::TryAcquireError::Closed) {
                    Error::Closed
                } else {
                    Error::Busy
                };
                #[cfg(test)]
                probe.failed(&error);
                #[cfg(yetlet_session_admission_probe)]
                trace.failed(&error);
                error
            })?;
        #[cfg(test)]
        probe.admitted(self.inner.active.available_permits());
        #[cfg(yetlet_session_admission_probe)]
        trace.admitted(
            self.inner.active.available_permits(),
            self.inner.admission.available_permits(),
        );
        let active = timeout_at(
            end.min(Instant::now() + WAIT_TIMEOUT),
            self.inner.active.clone().acquire_owned(),
        )
        .await
        .map_err(|_| {
            #[cfg(test)]
            probe.failed(&Error::QueueTimeout);
            #[cfg(yetlet_session_admission_probe)]
            trace.failed(&Error::QueueTimeout);
            Error::QueueTimeout
        })?
        .map_err(|_| {
            #[cfg(test)]
            probe.failed(&Error::Closed);
            #[cfg(yetlet_session_admission_probe)]
            trace.failed(&Error::Closed);
            Error::Closed
        })?;
        #[cfg(test)]
        probe.acquired();
        #[cfg(yetlet_session_admission_probe)]
        trace.acquired();
        if Instant::now() >= end {
            #[cfg(test)]
            probe.failed(&Error::DispatchDeadline);
            #[cfg(yetlet_session_admission_probe)]
            trace.failed(&Error::DispatchDeadline);
            return Err(Error::DispatchDeadline);
        }
        let (send, recv) = oneshot::channel();
        {
            let mut tasks = self.inner.tasks.lock().map_err(|_| Error::Unavailable)?;
            let mut manager = tasks.manager.clone().ok_or(Error::Closed)?;
            tasks.handles.retain(|h| !h.is_finished());
            let inner = self.inner.clone();
            #[cfg(yetlet_session_admission_probe)]
            let trace_worker = trace.worker();
            tasks.handles.push(tokio::spawn(async move {
                #[cfg(yetlet_session_admission_probe)]
                let mut trace_worker = crate::admission_trace::Worker::enter(trace_worker);
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
                #[cfg(yetlet_session_admission_probe)]
                trace_worker.reply();
                let _ = send.send(result.map_err(|_| Error::Unavailable));
            }));
        }
        let result = timeout_at(end, recv)
            .await
            .map_err(|_| Error::Unavailable)
            .and_then(|reply| reply.map_err(|_| Error::Unavailable))
            .and_then(std::convert::identity);
        #[cfg(test)]
        probe.finished(&result);
        #[cfg(yetlet_session_admission_probe)]
        trace.finished(&result);
        result
    }

    pub fn record_key(&self, id: &str) -> Result<String, Error> {
        if !valid_id(id) {
            return Err(Error::Configuration);
        }
        Ok(format!("{}session:{id}", self.prefix))
    }
    /// Fixture-only commands permit deterministic fault injection; no public provider API.
    async fn command(&self, command: redis::Cmd, end: Instant) -> Result<redis::Value, Error> {
        self.call(Query::Command(command), end).await
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
    fn args(&self, kind: &str) -> Vec<String> {
        let mut args: Vec<String> = [
            kind, "", "0", "", "", "0", "2500", "5000", "400", "1024", "64",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        args[6] = self.config.ttl_ms.to_string();
        args[9] = self.config.payload_limit.to_string();
        args
    }
    pub async fn read(&self, id: &str, end: Instant) -> Result<Snapshot, Error> {
        let row = strings(
            self.read_script(self.keys(id)?, self.args("read"), end)
                .await?,
        )?;
        if row.first().map(String::as_str) != Some("ok") {
            let code = row.first().cloned().ok_or(Error::Protocol)?;
            return Err(if code == "invalid" {
                Error::InvalidRecord
            } else {
                lifecycle(&code)
            });
        }
        if row.len() != 7 || row[2].len() > self.config.payload_limit || !valid_id(&row[3]) {
            return Err(Error::InvalidRecord);
        }
        for cell in &row[4..7] {
            cell.parse::<u64>().map_err(|_| Error::InvalidRecord)?;
        }
        Ok(Snapshot {
            state: serde_json::from_str(&row[2]).map_err(|_| Error::Codec)?,
            revision: row[1].parse().map_err(|_| Error::InvalidRecord)?,
            csrf: row[3].clone(),
        })
    }

    pub async fn prepare(
        &self,
        kind: &str,
        id: Option<&str>,
        revision: u64,
        state: &serde_json::Value,
        end: Instant,
    ) -> Result<Ticket, Error> {
        if !matches!(kind, "create" | "update" | "rotate" | "replace" | "revoke") {
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
        let mut args = self.args(kind);
        args[2] = revision.to_string();
        args[3] = serde_json::to_string(state).map_err(|_| Error::Codec)?;
        if args[3].len() > self.config.payload_limit {
            return Err(Error::PayloadTooLarge);
        }
        args[4] = random_id()?;
        let time = strings(self.command(redis::cmd("TIME"), end).await?)?;
        if time.len() != 2 {
            return Err(Error::Protocol);
        }
        let now = time[0].parse::<u64>().map_err(|_| Error::Protocol)? * 1000
            + time[1].parse::<u64>().map_err(|_| Error::Protocol)? / 1000;
        if Instant::now() >= end {
            return Err(Error::DispatchDeadline);
        }
        // The caller deadline bounds waiting, not cancellation of a dispatched
        // Redis write. Its separate immutable server window remains one second.
        args[5] = (now + 1000).to_string();
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
    pub async fn execute(&self, ticket: &Ticket, end: Instant) -> Result<Mutation, Error> {
        match self
            .call(Query::Script(ticket.keys.clone(), ticket.args.clone()), end)
            .await
        {
            Ok(value) => {
                mutation(value).map_err(|_| Error::OutcomeUnknown(RedisOperationId(ticket.clone())))
            }
            Err(
                e @ (Error::Busy | Error::QueueTimeout | Error::Closed | Error::DispatchDeadline),
            ) => Err(e),
            Err(_) => Err(Error::OutcomeUnknown(RedisOperationId(ticket.clone()))),
        }
    }
    pub async fn resolve(&self, ticket: &Ticket, end: Instant) -> Result<Mutation, Error> {
        if ticket.keys.first() != Some(&self.record_key(ticket.identity())?) {
            return Err(Error::Configuration);
        }
        let mut args = ticket.args.clone();
        args.push(args[0].clone());
        args[0] = "resolve".into();
        mutation(self.read_script(ticket.keys.clone(), args, end).await?)
    }
    async fn read_script(
        &self,
        keys: Vec<String>,
        args: Vec<String>,
        end: Instant,
    ) -> Result<redis::Value, Error> {
        let mut command = redis::cmd("EVAL_RO");
        command.arg(SCRIPT).arg(keys.len()).arg(keys).arg(args);
        self.command(command, end).await
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

// Only closed, redacted categories cross the public API boundary.
pub fn lifecycle(code: &str) -> Error {
    match code {
        "missing" => Error::Missing,
        "expired" => Error::Expired,
        "rotated" => Error::Rotated,
        "revoked" => Error::Revoked,
        "invalid" => Error::InvalidRecord,
        "capacity" => Error::Capacity,
        "deadline" => Error::DispatchDeadline,
        _ => Error::Protocol,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn command(name: &str, args: &[&str]) -> redis::Cmd {
        let mut cmd = redis::cmd(name);
        cmd.arg(args);
        cmd
    }
    async fn wait_blocked(observer: &Store, count: usize) {
        timeout(Duration::from_secs(1), async {
            loop {
                let value = observer
                    .command(command("INFO", &["clients"]), Instant::now() + CALL_TIMEOUT)
                    .await
                    .unwrap();
                let info: String = redis::from_redis_value(value).unwrap();
                if info
                    .lines()
                    .any(|l| l.trim() == format!("blocked_clients:{count}"))
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .expect("owned Redis blocking-client boundary");
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "requires the disposable Redis fixture"]
    async fn cancellation_keeps_capacity_and_shutdown_reaps_owned_work() {
        let endpoint = std::env::var("YT_REDIS_URL").unwrap();
        let config =
            RedisSessionConfig::new(random_id().unwrap(), "b2", Duration::from_secs(60), 1, 1024)
                .unwrap();
        let a = Store::connect(&endpoint, config.clone()).await.unwrap();
        let observer = Store::connect(&endpoint, config).await.unwrap();
        let a2 = a.clone();
        let key = a.record_key(&random_id().unwrap()).unwrap();
        let caller = tokio::spawn(async move {
            a2.command(
                command("BLPOP", &[&key, "1"]),
                Instant::now() + CALL_TIMEOUT,
            )
            .await
        });
        wait_blocked(&observer, 1).await;
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        let end = Instant::now() + CALL_TIMEOUT;
        let (one, two, three) = tokio::join!(
            a.command(redis::cmd("PING"), end),
            a.command(redis::cmd("PING"), end),
            a.command(redis::cmd("PING"), end)
        );
        let results = [one, two, three];
        assert!(
            results
                .iter()
                .filter(|v| matches!(v, Err(Error::Busy)))
                .count()
                == 2
        );
        assert!(
            results
                .iter()
                .filter(|v| matches!(v, Err(Error::QueueTimeout)))
                .count()
                == 1
        );
        timeout(Duration::from_secs(2), async {
            while a.inner.running.load(Ordering::SeqCst) != 0 {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await
        .unwrap();
        assert!(a.inner.peak.load(Ordering::SeqCst) == 1);
        let key = a.record_key(&random_id().unwrap()).unwrap();
        assert!(matches!(
            a.command(
                command("BLPOP", &[&key, "60"]),
                Instant::now() + CALL_TIMEOUT
            )
            .await,
            Err(Error::Unavailable)
        ));
        assert!(a.shutdown().await.unwrap() == 1 && a.inner.running.load(Ordering::SeqCst) == 0);
        assert!(matches!(
            a.command(redis::cmd("PING"), Instant::now() + CALL_TIMEOUT)
                .await,
            Err(Error::Closed)
        ));
        wait_blocked(&observer, 0).await;
        observer.shutdown().await.unwrap();
    }
}

fn endpoint_supported(url: &str) -> bool {
    url.starts_with("redis://")
        || (cfg!(feature = "tls") && url.starts_with("rediss://") && !url.contains('#'))
}

#[cfg(test)]
mod tls_selection_tests {
    use super::*;
    #[cfg(feature = "tls")]
    #[test]
    fn yt184_tls_backend_is_constructible_without_other_consumers() {
        let _ = rustls::ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();
    }
    #[test]
    fn yt184_tls_endpoint_selection_and_no_insecure_fragment() {
        assert!(endpoint_supported("redis://127.0.0.1:6379/"));
        assert_eq!(
            endpoint_supported("rediss://valkey.yt184.test:6379/"),
            cfg!(feature = "tls")
        );
        assert!(!endpoint_supported(
            "rediss://valkey.yt184.test:6379/#insecure"
        ));
        assert!(!endpoint_supported("http://valkey.yt184.test:6379/"));
    }
}
