//! YT-277/278: private, explicitly selected native Web service resource.
//!
//! The foreground root owns this service's single thread, Tokio runtime,
//! shutdown sender and JoinHandle. The YT-278 bridge carries only bounded
//! typed commands and copied immutable C1 publications; no Rc crosses threads.
//! This is not a public WebHost, SDK API, generic Host trait or second authority.

use std::{
    collections::VecDeque,
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use yetlet::server_axum::App;
use yetlet_composable_host_research::simulation::{Command, Snapshot};

const HTTP_READY_REQUEST: &[u8] =
    b"GET /f1/ready HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n";
const PROBE_ATTEMPTS: usize = 8;
const CONTROL_CAPACITY: usize = 8;
const SNAPSHOT_HISTORY_CAPACITY: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct F1Snapshot {
    pub generation: u64,
    pub tick: u64,
    pub revision: u64,
    pub active_ticks: u64,
    pub paused: bool,
    pub positions: [[i32; 2]; 2],
    pub last_applied_control: Option<u64>,
}
impl F1Snapshot {
    pub fn from_c1(source: &Snapshot, last_applied_control: Option<u64>) -> Self {
        let [one, two] = source.positions();
        Self {
            generation: source.generation(),
            tick: source.tick(),
            revision: source.revision(),
            active_ticks: source.active_ticks(),
            paused: source.paused(),
            positions: [[one.x, one.y], [two.x, two.y]],
            last_applied_control,
        }
    }
}

/// The only two supported Web control intents in this private game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ControlVerb {
    Pause,
    Resume,
}

#[derive(Debug, Deserialize)]
struct ControlRequest {
    generation: u64,
    command: ControlVerb,
}

#[derive(Debug, Serialize)]
struct AdmissionReceipt {
    receipt_id: u64,
    status: &'static str,
    observed_revision: u64,
}

/// A receipt is only a queue admission; the original C1 reducer applies it
/// on a later owner tick. It never contains a Simulation or Rc<Snapshot>.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlEnvelope {
    pub receipt_id: u64,
    pub generation: u64,
    pub command: Command,
}

#[derive(Default)]
struct PublicationHistory {
    records: VecDeque<F1Snapshot>,
}
impl PublicationHistory {
    fn insert(&mut self, value: F1Snapshot) -> io::Result<()> {
        if let Some(last) = self.records.back()
            && (last.generation != value.generation || last.revision >= value.revision)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stale or cross-generation F1 publication",
            ));
        }
        if self.records.len() == SNAPSHOT_HISTORY_CAPACITY {
            self.records.pop_front();
        }
        self.records.push_back(value);
        Ok(())
    }

    fn latest(&self) -> Option<F1Snapshot> {
        self.records.back().cloned()
    }

    fn at_revision(&self, revision: u64) -> Option<F1Snapshot> {
        self.records
            .iter()
            .find(|s| s.revision == revision)
            .cloned()
    }
}

struct F1Shared {
    snapshots: Mutex<PublicationHistory>,
    command_tx: SyncSender<ControlEnvelope>,
    // Serialized with try_send so closing admission is an exact boundary.
    accepting: Mutex<bool>,
    next_receipt: AtomicU64,
    #[cfg(test)]
    hold_probe: Option<Arc<DrainProbe>>,
    #[cfg(test)]
    worker_error_on_shutdown: bool,
}

/// YT-279 selected bind address and test-only held-request gate. The
/// production default stays loopback ephemeral, with no public configuration.
#[derive(Default)]
struct F1StartConfig {
    bind: Option<SocketAddr>,
    #[cfg(test)]
    hold_probe: Option<Arc<DrainProbe>>,
    #[cfg(test)]
    worker_error_on_shutdown: bool,
}

#[cfg(test)]
struct DrainProbe {
    entered: Mutex<Option<mpsc::SyncSender<()>>>,
    released: tokio::sync::Notify,
}

#[cfg(test)]
async fn held_test_request(State(state): State<Arc<F1Shared>>) -> Result<&'static str, StatusCode> {
    let probe = state.hold_probe.as_ref().ok_or(StatusCode::NOT_FOUND)?;
    let entered = probe
        .entered
        .lock()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .take();
    if let Some(sender) = entered {
        let _ = sender.send(());
    }
    probe.released.notified().await;
    Ok("released")
}

/// Completion is verified by joining the actual owned task. A deadline is
/// not cleanup success: Pending retains the original JoinHandle in its owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum F1Completion {
    Complete,
    Pending,
}

pub const REQUIRED_WEB_EXIT_CAUSE: &str = "F1 required Web service exited unexpectedly";

/// YT-279: the worker's actual failed Join is the first error when Web
/// terminated before another component. A prior Native/UDP error remains
/// authoritative when it preceded Web cleanup.
pub fn preserve_web_worker_cause(
    primary: &mut Option<String>,
    web_exited_first: bool,
    completion: &io::Result<()>,
) -> bool {
    let Err(error) = completion else {
        return false;
    };
    let detail = format!("F1 required Web worker failed: {error}");
    if web_exited_first && primary.as_deref() == Some(REQUIRED_WEB_EXIT_CAUSE) {
        *primary = Some(detail);
    } else {
        primary.get_or_insert(detail);
    }
    true
}

async fn read_current(State(state): State<Arc<F1Shared>>) -> Result<Json<F1Snapshot>, StatusCode> {
    let guard = state
        .snapshots
        .lock()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    guard
        .latest()
        .map(Json)
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)
}

async fn read_exact(
    Path(revision): Path<u64>,
    State(state): State<Arc<F1Shared>>,
) -> Result<Json<F1Snapshot>, StatusCode> {
    let guard = state
        .snapshots
        .lock()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    guard
        .at_revision(revision)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn submit_control(
    State(state): State<Arc<F1Shared>>,
    Json(request): Json<ControlRequest>,
) -> Result<(StatusCode, Json<AdmissionReceipt>), StatusCode> {
    // Hold this short lock through the nonblocking enqueue: after the
    // foreground owner closes admission no new HTTP request can receive 202.
    let accepting = state
        .accepting
        .lock()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    if !*accepting {
        return Err(StatusCode::SERVICE_UNAVAILABLE);
    }
    let current = state
        .snapshots
        .lock()
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .latest()
        .ok_or(StatusCode::SERVICE_UNAVAILABLE)?;
    if request.generation != current.generation {
        return Err(StatusCode::CONFLICT);
    }
    let receipt_id = state
        .next_receipt
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
            current.checked_add(1)
        })
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let command = match request.command {
        ControlVerb::Pause => Command::Pause,
        ControlVerb::Resume => Command::Resume,
    };
    match state.command_tx.try_send(ControlEnvelope {
        receipt_id,
        generation: request.generation,
        command,
    }) {
        Ok(()) => Ok((
            StatusCode::ACCEPTED,
            Json(AdmissionReceipt {
                receipt_id,
                status: "queued",
                observed_revision: current.revision,
            }),
        )),
        Err(TrySendError::Full(_)) => Err(StatusCode::TOO_MANY_REQUESTS),
        Err(TrySendError::Disconnected(_)) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// Concrete owned Web resource; the caller selects it, not the SDK.
pub struct F1WebService {
    endpoint: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<io::Result<()>>>,
    shared: Arc<F1Shared>,
    command_rx: Receiver<ControlEnvelope>,
}

impl F1WebService {
    /// YT-277's standalone readiness-only path stays usable.
    pub fn start() -> io::Result<Self> {
        Self::start_internal(None, F1StartConfig::default())
    }

    /// YT-278: publish the actual sole D2 authority's initial state *before*
    /// exposing HTTP. Never construct a second Simulation.
    pub fn start_game(initial: &Snapshot) -> io::Result<Self> {
        Self::start_internal(Some(initial), F1StartConfig::default())
    }

    /// Private root startup injection of a selected loopback bind, so the
    /// actual foreground root can test rollback after UDP was acquired.
    /// This is not a public Yetlet SDK API or a network exposure default.
    pub fn start_game_on(initial: &Snapshot, bind: SocketAddr) -> io::Result<Self> {
        Self::start_internal(
            Some(initial),
            F1StartConfig {
                bind: Some(bind),
                #[cfg(test)]
                hold_probe: None,
                #[cfg(test)]
                worker_error_on_shutdown: false,
            },
        )
    }

    fn start_internal(initial: Option<&Snapshot>, config: F1StartConfig) -> io::Result<Self> {
        let (command_tx, command_rx) = mpsc::sync_channel(CONTROL_CAPACITY);
        let mut history = PublicationHistory::default();
        if let Some(initial) = initial {
            history.insert(F1Snapshot::from_c1(initial, None))?;
        }
        let shared = Arc::new(F1Shared {
            snapshots: Mutex::new(history),
            command_tx,
            accepting: Mutex::new(true),
            next_receipt: AtomicU64::new(1),
            #[cfg(test)]
            hold_probe: config.hold_probe,
            #[cfg(test)]
            worker_error_on_shutdown: config.worker_error_on_shutdown,
        });
        let (ready_tx, ready_rx) = mpsc::sync_channel::<Result<SocketAddr, String>>(1);
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let web_shared = Arc::clone(&shared);
        let bind = config
            .bind
            .unwrap_or_else(|| SocketAddr::from((Ipv4Addr::LOCALHOST, 0)));
        let thread = thread::Builder::new()
            .name("yetlet-f1-web-owned".to_owned())
            .spawn(move || run_web(ready_tx, shutdown_rx, web_shared, bind))?;

        let endpoint = match ready_rx.recv() {
            Ok(Ok(endpoint)) => endpoint,
            Ok(Err(reason)) => {
                let _ = shutdown_tx.send(());
                let _ = thread.join();
                return Err(io::Error::other(format!("F1 Web setup: {reason}")));
            }
            Err(_) => {
                let _ = shutdown_tx.send(());
                let _ = thread.join();
                return Err(io::Error::other("F1 Web exited before reporting bind"));
            }
        };
        let mut owner = Self {
            endpoint,
            shutdown: Some(shutdown_tx),
            thread: Some(thread),
            shared,
            command_rx,
        };
        if let Err(primary) = owner.await_http_ready() {
            let secondary = owner.stop_and_join().err();
            return Err(io::Error::other(format!(
                "F1 HTTP readiness failed: {primary}; cleanup={secondary:?}"
            )));
        }
        Ok(owner)
    }

    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }

    pub fn is_finished(&self) -> bool {
        self.thread.as_ref().is_some_and(JoinHandle::is_finished)
    }

    /// Join ownership is separate from worker success: Err after join still
    /// means the exact original JoinHandle was consumed and observed.
    pub fn has_pending_join(&self) -> bool {
        self.thread.is_some()
    }

    /// At most one queued request is consumed for each authoritative C1 tick.
    /// No blocking receive and no unbounded data processing in the winit loop.
    pub fn pop_control(&self) -> Option<ControlEnvelope> {
        self.command_rx.try_recv().ok()
    }

    pub fn publish_snapshot(
        &self,
        source: &Snapshot,
        last_applied_control: Option<u64>,
    ) -> io::Result<()> {
        let copy = F1Snapshot::from_c1(source, last_applied_control);
        self.shared
            .snapshots
            .lock()
            .map_err(|_| io::Error::other("F1 Web publication lock poisoned"))?
            .insert(copy)
    }

    fn await_http_ready(&self) -> io::Result<()> {
        let mut last = None;
        for _ in 0..PROBE_ATTEMPTS {
            match self.probe_once() {
                Ok(()) => return Ok(()),
                Err(error) => last = Some(error),
            }
            thread::sleep(Duration::from_millis(25));
        }
        Err(last.unwrap_or_else(|| io::Error::other("F1 Web readiness not observed")))
    }

    fn probe_once(&self) -> io::Result<()> {
        let mut stream = TcpStream::connect_timeout(&self.endpoint, Duration::from_secs(1))?;
        stream.set_read_timeout(Some(Duration::from_secs(1)))?;
        stream.set_write_timeout(Some(Duration::from_secs(1)))?;
        stream.write_all(HTTP_READY_REQUEST)?;
        let mut received = Vec::new();
        stream.read_to_end(&mut received)?;
        if !received.starts_with(b"HTTP/1.1 200 ")
            || !received
                .windows(b"ready".len())
                .any(|part| part == b"ready")
        {
            return Err(io::Error::other("F1 Web did not serve the readiness route"));
        }
        Ok(())
    }

    /// Close command admission BEFORE the C1 owner stops. The same mutex
    /// fences a racing valid POST and its try_send, so a new request after
    /// this function completes receives 503 rather than a doomed 202.
    pub fn close_admission(&self) {
        let mut accepting = self
            .shared
            .accepting
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *accepting = false;
    }

    /// Close-only cleanup never waits on the bounded data queue. The caller
    /// must close admission first, so accepted but unapplied commands cannot
    /// be misreported as executed when the authoritative C1 owner stops.
    pub fn discard_pending_after_close(&self) -> usize {
        let accepting = self
            .shared
            .accepting
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *accepting {
            return 0;
        }
        let mut discarded = 0;
        while self.command_rx.try_recv().is_ok() {
            discarded += 1;
        }
        discarded
    }

    /// An independent control signal, not a command enqueued into application
    /// data. It can stop even when all CONTROL_CAPACITY slots are occupied.
    pub fn request_stop(&mut self) {
        self.close_admission();
        if let Some(sender) = self.shutdown.take() {
            let _ = sender.send(());
        }
    }

    /// A timeout is explicitly Pending and retains the original JoinHandle.
    /// The owner can call this again after releasing an in-flight HTTP request.
    /// No detach, invented completion or unrelated runtime cancellation.
    pub fn complete_within(&mut self, budget: Duration) -> io::Result<F1Completion> {
        self.request_stop();
        let started = Instant::now();
        while self
            .thread
            .as_ref()
            .is_some_and(|thread| !thread.is_finished())
        {
            if started.elapsed() >= budget {
                return Ok(F1Completion::Pending);
            }
            thread::sleep(Duration::from_millis(2).min(budget.saturating_sub(started.elapsed())));
        }
        self.stop_and_join()?;
        Ok(F1Completion::Complete)
    }

    /// Cooperative final join of the SAME owned execution. An explicit
    /// bounded observation can precede this through complete_within().
    pub fn stop_and_join(&mut self) -> io::Result<()> {
        self.request_stop();
        match self.thread.take() {
            Some(thread) => thread
                .join()
                .map_err(|_| io::Error::other("F1 Web thread panicked"))?,
            None => Ok(()),
        }
    }
}

impl Drop for F1WebService {
    fn drop(&mut self) {
        if self.thread.is_some() {
            let _ = self.stop_and_join();
        }
    }
}

fn run_web(
    ready_tx: SyncSender<Result<SocketAddr, String>>,
    shutdown_rx: oneshot::Receiver<()>,
    shared: Arc<F1Shared>,
    bind: SocketAddr,
) -> io::Result<()> {
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            let _ = ready_tx.send(Err(error.to_string()));
            return Err(error);
        }
    };
    runtime.block_on(async move {
        let listener = match tokio::net::TcpListener::bind(bind).await {
            Ok(listener) => listener,
            Err(error) => {
                let _ = ready_tx.send(Err(error.to_string()));
                return Err(error);
            }
        };
        let endpoint = listener.local_addr()?;
        let app = App::<Arc<F1Shared>>::new()
            .get("/f1/ready", || async { "ready" })
            .get("/f1/snapshot", read_current)
            .get("/f1/snapshot/{revision}", read_exact)
            .post("/f1/control", submit_control);
        #[cfg(test)]
        let app = app.get("/f1/test/held", held_test_request);
        #[cfg(test)]
        let worker_error_on_shutdown = shared.worker_error_on_shutdown;
        let router = app.with_state::<()>(shared).into_router();
        let _ = ready_tx.send(Ok(endpoint));
        let served = axum::serve(listener, router)
            .with_graceful_shutdown(async move {
                let _ = shutdown_rx.await;
            })
            .await;
        #[cfg(test)]
        if worker_error_on_shutdown && served.is_ok() {
            return Err(io::Error::other("YT-279 injected actual Web worker error"));
        }
        served
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, TcpListener};
    use yetlet_composable_host_research::simulation::{Input, Simulation};

    fn http(endpoint: SocketAddr, request: &str) -> String {
        let mut conn = TcpStream::connect_timeout(&endpoint, Duration::from_secs(2)).unwrap();
        conn.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
        conn.write_all(request.as_bytes()).unwrap();
        let mut received = String::new();
        conn.read_to_string(&mut received).unwrap();
        received
    }

    fn get(endpoint: SocketAddr, path: &str) -> String {
        http(
            endpoint,
            &format!("GET {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"),
        )
    }

    fn post(endpoint: SocketAddr, payload: &str) -> String {
        http(
            endpoint,
            &format!(
                "POST /f1/control HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{payload}",
                payload.len(),
            ),
        )
    }

    #[test]
    fn f1_web_one_owned_thread_real_http_ready_then_actual_join_and_rebind() {
        let mut web = F1WebService::start().expect("HTTP route actually responds");
        let endpoint = web.endpoint();
        assert_eq!(endpoint.ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
        assert_ne!(endpoint.port(), 0);
        assert!(!web.is_finished());
        web.stop_and_join().expect("actual native Axum drain");
        assert!(web.thread.is_none(), "no owned task detached");
        let rebound = TcpListener::bind(endpoint).expect("original HTTP port released");
        drop(rebound);
        web.stop_and_join().expect("normal stop is idempotent");
    }

    #[test]
    fn f1_web_drop_fallback_joins_the_cooperative_service() {
        let endpoint = {
            let web = F1WebService::start().unwrap();
            web.endpoint()
        };
        let rebound = TcpListener::bind(endpoint).expect("Drop completed owned service");
        drop(rebound);
    }

    #[test]
    fn yt278_http_control_admission_is_not_application_until_c1_step() {
        let mut authority = Simulation::new(1).unwrap();
        let mut web = F1WebService::start_game(&authority.snapshot()).unwrap();
        let endpoint = web.endpoint();
        let initial = get(endpoint, "/f1/snapshot");
        assert!(initial.starts_with("HTTP/1.1 200 "));
        assert!(initial.contains(r#""revision":0"#));
        assert!(initial.contains(r#""paused":false"#));
        let response = post(endpoint, r#"{"generation":1,"command":"pause"}"#);
        assert!(response.starts_with("HTTP/1.1 202 "));
        assert!(response.contains(r#""status":"queued""#));
        assert_eq!(authority.snapshot().revision(), 0);
        assert!(!authority.snapshot().paused());
        let receipt = web.pop_control().unwrap();
        assert_eq!(receipt.receipt_id, 1);
        assert_eq!(receipt.generation, 1);
        assert_eq!(receipt.command, Command::Pause);
        assert!(web.pop_control().is_none());

        let applied = authority
            .step(
                1,
                &[Input {
                    generation: 1,
                    tick: 1,
                    sequence: 1,
                    command: receipt.command,
                }],
            )
            .unwrap();
        web.publish_snapshot(&applied, Some(receipt.receipt_id))
            .unwrap();
        assert!(applied.paused());
        assert_eq!(applied.revision(), 1);
        let exact = get(endpoint, "/f1/snapshot/1");
        assert!(exact.starts_with("HTTP/1.1 200 "));
        assert!(exact.contains(r#""revision":1"#));
        assert!(exact.contains(r#""paused":true"#));
        assert!(exact.contains(r#""last_applied_control":1"#));
        assert_eq!(
            get(endpoint, "/f1/snapshot/0").contains(r#""revision":0"#),
            true
        );
        web.stop_and_join().unwrap();
    }

    #[test]
    fn yt278_http_rejects_stale_generation_and_bounded_queue_overflow() {
        let authority = Simulation::new(1).unwrap();
        let mut web = F1WebService::start_game(&authority.snapshot()).unwrap();
        let endpoint = web.endpoint();
        assert!(
            post(endpoint, r#"{"generation":2,"command":"pause"}"#).starts_with("HTTP/1.1 409 ")
        );
        assert!(
            post(endpoint, r#"{"generation":1,"command":"invalid"}"#).starts_with("HTTP/1.1 422 ")
        );
        for _ in 0..CONTROL_CAPACITY {
            let response = post(endpoint, r#"{"generation":1,"command":"pause"}"#);
            assert!(response.starts_with("HTTP/1.1 202 "));
        }
        assert!(
            post(endpoint, r#"{"generation":1,"command":"resume"}"#).starts_with("HTTP/1.1 429 ")
        );
        assert_eq!(web.pop_control().unwrap().receipt_id, 1);
        assert_eq!(web.pop_control().unwrap().receipt_id, 2);
        web.stop_and_join().unwrap();
    }

    #[test]
    fn yt278_closed_admission_rejects_fresh_post_with_no_enqueued_command() {
        let authority = Simulation::new(1).unwrap();
        let mut web = F1WebService::start_game(&authority.snapshot()).unwrap();
        let endpoint = web.endpoint();
        web.close_admission();
        let denied = post(endpoint, r#"{"generation":1,"command":"pause"}"#);
        assert!(denied.starts_with("HTTP/1.1 503 "), "{denied}");
        assert!(
            web.pop_control().is_none(),
            "closed queue must not add work"
        );
        assert_eq!(authority.snapshot().revision(), 0);
        web.stop_and_join().unwrap();
    }

    #[test]
    fn yt278_snapshot_history_is_revision_pinned_and_rejects_stale_publications() {
        let mut authority = Simulation::new(1).unwrap();
        let mut web = F1WebService::start_game(&authority.snapshot()).unwrap();
        let initial = authority.snapshot();
        assert!(web.publish_snapshot(&initial, None).is_err());
        for next in 1..=SNAPSHOT_HISTORY_CAPACITY + 3 {
            let published = authority.step(next as u64, &[]).unwrap();
            web.publish_snapshot(&published, None).unwrap();
        }
        let endpoint = web.endpoint();
        assert!(get(endpoint, "/f1/snapshot/0").starts_with("HTTP/1.1 404 "));
        assert!(get(endpoint, "/f1/snapshot/3").starts_with("HTTP/1.1 404 "));
        assert!(get(endpoint, "/f1/snapshot/4").contains(r#""revision":4"#));
        assert!(get(endpoint, "/f1/snapshot").contains(r#""revision":259"#));
        web.stop_and_join().unwrap();
    }

    #[test]
    fn yt279_required_web_bind_conflict_retains_primary_and_releases_prepared_udp() {
        let mut host = crate::udp_d2::NetworkHost::bind().unwrap();
        let udp_endpoint = host.endpoint();
        let occupied = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let bind = occupied.local_addr().unwrap();
        let original = host.snapshot();
        let failure = F1WebService::start_internal(
            Some(&original),
            F1StartConfig {
                bind: Some(bind),
                ..Default::default()
            },
        )
        .err()
        .expect("an actually occupied loopback TCP address cannot be rebound");
        assert!(failure.to_string().contains("F1 Web setup"), "{failure}");
        assert_eq!(host.snapshot().revision(), original.revision());
        assert!(
            !host.socket_closed(),
            "startup failure must preserve the original owner for cleanup"
        );
        host.stop();
        assert!(host.socket_closed());
        let reusable_udp = std::net::UdpSocket::bind(udp_endpoint).unwrap();
        drop(reusable_udp);
        drop(occupied);
        let reusable_tcp = TcpListener::bind(bind).unwrap();
        drop(reusable_tcp);
    }

    #[test]
    fn yt279_saturated_command_queue_cannot_starve_stop_or_report_applied() {
        let authority = Simulation::new(1).unwrap();
        let mut web = F1WebService::start_game(&authority.snapshot()).unwrap();
        let endpoint = web.endpoint();
        for _ in 0..CONTROL_CAPACITY {
            assert!(
                post(endpoint, r#"{"generation":1,"command":"pause"}"#)
                    .starts_with("HTTP/1.1 202 ")
            );
        }
        assert!(
            post(endpoint, r#"{"generation":1,"command":"resume"}"#).starts_with("HTTP/1.1 429 ")
        );
        let started = Instant::now();
        web.close_admission();
        assert_eq!(web.discard_pending_after_close(), CONTROL_CAPACITY);
        assert!(web.pop_control().is_none());
        assert!(
            post(endpoint, r#"{"generation":1,"command":"resume"}"#).starts_with("HTTP/1.1 503 ")
        );
        assert_eq!(
            web.complete_within(Duration::from_secs(2)).unwrap(),
            F1Completion::Complete
        );
        assert!(started.elapsed() < Duration::from_secs(2));
        assert_eq!(authority.snapshot().revision(), 0);
    }

    #[test]
    fn yt279_inflight_real_http_drains_and_pending_timeout_retains_original_join() {
        let (entered_sender, entered) = mpsc::sync_channel::<()>(1);
        let gate = Arc::new(DrainProbe {
            entered: Mutex::new(Some(entered_sender)),
            released: tokio::sync::Notify::new(),
        });
        let mut web = F1WebService::start_internal(
            None,
            F1StartConfig {
                hold_probe: Some(Arc::clone(&gate)),
                ..Default::default()
            },
        )
        .unwrap();
        let endpoint = web.endpoint();
        let response = thread::spawn(move || get(endpoint, "/f1/test/held"));
        entered
            .recv_timeout(Duration::from_secs(2))
            .expect("actual HTTP handler entered");
        assert_eq!(
            web.complete_within(Duration::from_millis(25)).unwrap(),
            F1Completion::Pending,
            "an in-flight HTTP handler prevents premature complete"
        );
        assert!(
            web.thread.is_some(),
            "timeout must not detach original JoinHandle"
        );
        gate.released.notify_one();
        assert_eq!(
            web.complete_within(Duration::from_secs(2)).unwrap(),
            F1Completion::Complete
        );
        assert!(web.thread.is_none());
        let received = response.join().unwrap();
        assert!(received.starts_with("HTTP/1.1 200 "), "{received}");
        assert!(received.contains("released"));
        let reusable = TcpListener::bind(endpoint).unwrap();
        drop(reusable);
    }

    #[test]
    fn yt279_actual_failing_worker_join_propagates_cause_and_preserves_earlier_native_failure() {
        let mut web = F1WebService::start_internal(
            None,
            F1StartConfig {
                worker_error_on_shutdown: true,
                ..Default::default()
            },
        )
        .unwrap();
        web.request_stop();
        for _ in 0..100 {
            if web.is_finished() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            web.is_finished(),
            "the actual failing Web worker must terminate"
        );
        assert!(
            web.has_pending_join(),
            "failure must retain the original join"
        );
        let actual_failure = web.complete_within(Duration::from_secs(2)).unwrap_err();
        assert!(
            actual_failure
                .to_string()
                .contains("YT-279 injected actual Web worker error"),
            "{actual_failure}"
        );
        assert!(
            !web.has_pending_join(),
            "a failed worker may still have been joined"
        );

        let failed: io::Result<()> = Err(actual_failure);
        let mut web_first = Some(REQUIRED_WEB_EXIT_CAUSE.to_owned());
        assert!(preserve_web_worker_cause(&mut web_first, true, &failed));
        assert!(
            web_first
                .unwrap()
                .contains("YT-279 injected actual Web worker error")
        );
        let mut native_first = Some("first native UDP failure".to_owned());
        assert!(preserve_web_worker_cause(&mut native_first, false, &failed));
        assert_eq!(native_first.as_deref(), Some("first native UDP failure"));
    }

    #[test]
    fn yt279_early_web_worker_exit_retains_original_handle_until_joined() {
        let mut web = F1WebService::start().unwrap();
        web.request_stop();
        for _ in 0..100 {
            if web.is_finished() {
                break;
            }
            thread::sleep(Duration::from_millis(5));
        }
        assert!(
            web.is_finished(),
            "required Web worker finished before foreground stopped"
        );
        assert!(web.thread.is_some(), "the original task still needs a join");
        assert_eq!(
            web.complete_within(Duration::from_secs(2)).unwrap(),
            F1Completion::Complete
        );
        assert!(web.thread.is_none());
    }
}
