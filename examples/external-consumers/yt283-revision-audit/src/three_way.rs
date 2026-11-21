//! Consumer-owned Web/UDP protocol and public adapter wiring.
use crate::{
    Action, AuditStatus, C1Domain, Command, Direction, Player, PreparedApply, RevisionAuditHost,
    Snapshot, TICK_EVIDENCE_CAPACITY, TickAuditRecord,
    tick_m1::{TickInput, TickObservation},
};
use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::StatusCode,
    routing::{get, post},
};
use std::{
    cell::RefCell,
    io::{self, Read, Write},
    net::{SocketAddr, TcpStream, UdpSocket},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use yetlet_hosts::{
    udp::{SendOutcome, UdpEndpoint},
    web_axum::{AdmissionError, CommandBridge, CommandIngress, HttpWorker, SnapshotReader},
};

pub const IO_BUDGET: Duration = Duration::from_millis(400);
pub const QUEUE: usize = 4;

/// One immutable payload in the original bridge. RA1 readers retain their
/// Snapshot/text projection; Tick readers obtain metadata and audit evidence
/// from the same revision rather than combining independently read values.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Publication {
    snapshot: Snapshot,
    tick: Option<Arc<TickAuditRecord>>,
}
impl Publication {
    fn ra1(snapshot: Snapshot) -> Self {
        Self {
            snapshot,
            tick: None,
        }
    }
}

/// A locally owned audit attachment bound to the exact command-admission
/// gate of the Ports that created it. A cloned audit Rc cannot close the Host.
pub struct AuditAttachment {
    audit: Rc<RevisionAuditHost>,
    admission: CommandIngress<Command>,
}
impl AuditAttachment {
    pub fn audit(&self) -> &Rc<RevisionAuditHost> {
        &self.audit
    }
    pub fn close(&mut self) -> io::Result<()> {
        if !self.admission.is_closed()? {
            return Err(io::Error::other("close owning admission before audit"));
        }
        self.audit.close()
    }
}

/// Each Ports owns its own authority, audit, and published history. Callers
/// can inspect the exact handles but cannot replace one bridge's C1 with
/// another bridge's C1 while keeping the old publication history.
///
/// ```compile_fail
/// use yt283_revision_audit::three_way::Ports;
/// let mut first = Ports::new(61).unwrap();
/// let second = Ports::new(61).unwrap();
/// first.c1 = second.c1.clone();
/// ```
pub struct Ports {
    c1: Rc<C1Domain>,
    audit: Rc<RevisionAuditHost>,
    bridge: CommandBridge<Command, Publication>,
    publication_failure: RefCell<Option<Arc<io::Error>>>,
    // Consumer-owned application startup gate, not a second C1/Simulation.
    startup_ready: Arc<AtomicBool>,
}
impl Ports {
    pub fn new(generation: u64) -> io::Result<Self> {
        let bridge = CommandBridge::new(QUEUE, TICK_EVIDENCE_CAPACITY)?;
        let c1 = Rc::new(C1Domain::new(generation));
        bridge.publish_snapshot(generation, 0, Publication::ra1(c1.snapshot()))?;
        c1.bind_ports()?; // direct C1 aliases cannot bypass this history
        let audit = Rc::new(RevisionAuditHost::new(c1.snapshot()));
        Ok(Self {
            c1,
            audit,
            bridge,
            publication_failure: RefCell::new(None),
            startup_ready: Arc::new(AtomicBool::new(false)),
        })
    }
    /// Read-only access to the bound caller-local authority handle.
    pub fn c1(&self) -> &Rc<C1Domain> {
        &self.c1
    }
    /// Read-only access to this bridge's actual audit Host.
    pub fn audit(&self) -> &Rc<RevisionAuditHost> {
        &self.audit
    }
    /// Bind an audit close callback to this exact Ports' admission gate.
    pub fn audit_attachment(&self) -> AuditAttachment {
        AuditAttachment {
            audit: Rc::clone(&self.audit),
            admission: self.ingress(),
        }
    }
    /// Seal the route and its shutdown admission into one owned launch value.
    pub fn web_launch(&self) -> WebLaunch {
        WebLaunch {
            routes: router(self),
            ingress: self.ingress(),
            startup_ready: Arc::clone(&self.startup_ready),
        }
    }
    pub fn ingress(&self) -> CommandIngress<Command> {
        self.bridge.ingress()
    }
    /// Caller must first observe all required Composition hosts Ready. The
    /// standalone headless C1 API remains separate from Web/UDP startup.
    pub fn enable_after_required_ready(&self) -> io::Result<()> {
        if self.ingress().is_closed()?
            || self.c1.is_stopped()
            || self.audit.status() != AuditStatus::Open
        {
            return Err(io::Error::other("cannot open application admission"));
        }
        self.startup_ready.store(true, Ordering::Release);
        Ok(())
    }

    /// Revoke external application ingress on failure/stop before local close.
    pub fn revoke_startup_ready(&self) {
        self.startup_ready.store(false, Ordering::Release);
    }

    /// Exact retained Snapshot; an expired, absent or wrong-generation identity
    /// returns NotFound. This query never substitutes the latest revision.
    pub fn pinned(&self, generation: u64, revision: u64) -> io::Result<Snapshot> {
        Self::read_pinned(&self.bridge.snapshots(), generation, revision)
    }
    fn read_pinned(
        reader: &SnapshotReader<Publication>,
        generation: u64,
        revision: u64,
    ) -> io::Result<Snapshot> {
        reader
            .at_revision(revision)?
            .filter(|p| {
                p.generation == generation
                    && p.revision == revision
                    && p.snapshot.snapshot.generation == generation
                    && p.snapshot.snapshot.revision == revision
            })
            .map(|p| p.snapshot.snapshot)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "pinned identity absent"))
    }
    /// Read a complete immutable Tick proof from this exact bridge revision.
    /// Expired, absent, wrong-generation or non-Tick identities return NotFound.
    /// A returned handle stays valid after eviction without pinning the history.
    pub fn pinned_tick(&self, generation: u64, revision: u64) -> io::Result<Arc<TickAuditRecord>> {
        self.bridge
            .snapshots()
            .at_revision(revision)?
            .filter(|p| p.generation == generation && p.revision == revision)
            .and_then(|p| {
                p.snapshot.tick.filter(|record| {
                    record.applied.snapshot == p.snapshot.snapshot
                        && record.applied.snapshot.generation == generation
                        && record.applied.snapshot.revision == revision
                        && record.applied.clock.tick == revision
                })
            })
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "pinned tick identity absent"))
    }

    /// Publish one deterministic logical Tick through this Ports' only bridge.
    /// The first accepted RA1 action or Tick selects the exclusive mode.
    /// Validation/reservation and the bridge write can fail without changing
    /// C1, audit or retained evidence. A full Tick window evicts its oldest proof
    /// only in the infallible commit after successful bridge publication.
    /// There is no fallible postpublication readback on this path.
    pub fn step_tick(&self, tick: u64, inputs: &[TickInput]) -> io::Result<TickObservation> {
        if let Some(error) = self.publication_failure() {
            return Err(io::Error::other(error));
        }
        if self.ingress().is_closed()? {
            return Err(io::Error::other("admission closed"));
        }
        let prepared = self.c1.prepare_tick(&self.audit, tick, inputs)?;
        let record = prepared.record();
        let snapshot = record.applied.snapshot;
        self.bridge.publish_snapshot(
            snapshot.generation,
            snapshot.revision,
            Publication {
                snapshot,
                tick: Some(record),
            },
        )?;
        // The immutable bridge write is the linearization point. HTTP readers
        // see the complete proof; C1/audit are Rc-local and cannot be observed
        // concurrently before this allocation-free commit returns.
        Ok(prepared.commit())
    }

    pub fn publication_failure(&self) -> Option<Arc<io::Error>> {
        self.publication_failure.borrow().clone()
    }
    // This exact seam also permits a focused unavailable-readback test without
    // poisoning private adapter state or pretending to roll back publication.
    fn finish_published(
        &self,
        prepared: PreparedApply,
        reader: &SnapshotReader<Publication>,
    ) -> io::Result<Snapshot> {
        let candidate = prepared.candidate();
        let result = Self::read_pinned(reader, candidate.generation, candidate.revision)
            .and_then(|readback| self.c1.commit_published(prepared, &self.audit, readback));
        result.map_err(|error| {
            let mut failure = self.publication_failure.borrow_mut();
            let first = failure.get_or_insert_with(|| Arc::new(io::Error::other(format!(
                "publication succeeded but readback/commit failed; immutable history retained, C1/audit uncommitted: {error}"
            ))));
            // Preserve the publication failure even if admission close fails.
            let _ = self.ingress().close();
            io::Error::other(Arc::clone(first))
        })
    }
    pub fn apply(&self, c: Command) -> io::Result<Snapshot> {
        if let Some(error) = self.publication_failure() {
            return Err(io::Error::other(error));
        }
        if self.ingress().is_closed()? {
            return Err(io::Error::other("admission closed"));
        }
        let prepared = self.c1.preflight(c, &self.audit)?;
        let s = prepared.candidate();
        self.bridge
            .publish_snapshot(s.generation, s.revision, Publication::ra1(s))?;
        self.finish_published(prepared, &self.bridge.snapshots())
    }
    pub fn drain(&self) -> io::Result<()> {
        for _ in 0..QUEUE {
            let Some(c) = self
                .bridge
                .commands()
                .try_recv()
                .map_err(io::Error::other)?
            else {
                break;
            };
            match self.apply(c) {
                Err(e) if e.kind() == io::ErrorKind::InvalidData => eprintln!("WEB_REJECTED {e}"),
                other => {
                    other?;
                }
            }
        }
        Ok(())
    }
    pub fn close(&mut self) -> io::Result<()> {
        self.revoke_startup_ready();
        if let Some(error) = self.publication_failure() {
            return Err(io::Error::other(error));
        }
        if !self.ingress().is_closed()? {
            return Err(io::Error::other("close HTTP admission before C1"));
        }
        self.bridge.commands().discard_after_close()?;
        self.c1.close(&self.audit)
    }
}
#[derive(Clone)]
struct HttpPorts {
    ingress: CommandIngress<Command>,
    reader: SnapshotReader<Publication>,
    startup_ready: Arc<AtomicBool>,
}
async fn command(State(p): State<HttpPorts>, body: Bytes) -> StatusCode {
    let Ok(Request::Apply(command)) = decode(&body) else {
        return StatusCode::BAD_REQUEST;
    };
    if !p.startup_ready.load(Ordering::Acquire) {
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    match p.ingress.try_submit(command) {
        Ok(()) => StatusCode::ACCEPTED,
        Err(AdmissionError::Full) => StatusCode::TOO_MANY_REQUESTS,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
async fn snapshot(
    State(p): State<HttpPorts>,
    Path((g, r)): Path<(u64, u64)>,
) -> Result<String, StatusCode> {
    let s = p
        .reader
        .at_revision(r)
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?
        .filter(|s| s.generation == g)
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(s.snapshot.snapshot.text())
}
pub fn router(p: &Ports) -> Router {
    Router::new()
        .route("/ready", get(|| async { "ready" }))
        .route("/command", post(command))
        .route("/snapshot/{generation}/{revision}", get(snapshot))
        .layer(DefaultBodyLimit::max(FRAME_BYTES))
        .with_state(HttpPorts {
            ingress: p.ingress(),
            reader: p.bridge.snapshots(),
            startup_ready: Arc::clone(&p.startup_ready),
        })
}
const FRAME_BYTES: usize = 20;

#[derive(Debug, PartialEq, Eq)]
enum Request {
    Apply(Command),
    Read { generation: u64, revision: u64 },
}

/// Consumer-specific fixed-length RA1 command, with explicit player/direction tags.
pub fn encode_command(action: Action, generation: u64, revision: u64) -> [u8; FRAME_BYTES] {
    let tag = match action {
        Action::Pause => 1,
        Action::Resume => 2,
        Action::Move { player, direction } => {
            let first = match player {
                Player::One => 4,
                Player::Two => 8,
            };
            first
                + match direction {
                    Direction::North => 0,
                    Direction::East => 1,
                    Direction::South => 2,
                    Direction::West => 3,
                }
        }
    };
    encode(tag, generation, revision)
}

pub fn encode_read(generation: u64, revision: u64) -> [u8; FRAME_BYTES] {
    encode(3, generation, revision)
}

fn encode(tag: u8, generation: u64, revision: u64) -> [u8; FRAME_BYTES] {
    let mut bytes = [0; FRAME_BYTES];
    bytes[..4].copy_from_slice(&[b'R', b'A', 1, tag]);
    bytes[4..12].copy_from_slice(&generation.to_be_bytes());
    bytes[12..].copy_from_slice(&revision.to_be_bytes());
    bytes
}

fn decode(bytes: &[u8]) -> io::Result<Request> {
    let invalid = || io::Error::new(io::ErrorKind::InvalidData, "invalid RA1 frame");
    if bytes.len() != FRAME_BYTES || bytes[..3] != [b'R', b'A', 1] {
        return Err(invalid());
    }
    let generation = u64::from_be_bytes(bytes[4..12].try_into().unwrap());
    let revision = u64::from_be_bytes(bytes[12..].try_into().unwrap());
    let action = match bytes[3] {
        1 => Action::Pause,
        2 => Action::Resume,
        3 => {
            return Ok(Request::Read {
                generation,
                revision,
            });
        }
        tag @ 4..=11 => Action::Move {
            player: if tag < 8 { Player::One } else { Player::Two },
            direction: [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ][usize::from((tag - 4) % 4)],
        },
        _ => return Err(invalid()),
    };
    Ok(Request::Apply(Command {
        generation,
        expected_revision: revision,
        action,
    }))
}

/// A loopback endpoint paired with the exact reserved client socket.
///
/// ```compile_fail,E0615
/// use std::net::UdpSocket;
/// use yt283_revision_audit::three_way::Network;
/// let client = UdpSocket::bind("127.0.0.1:0").unwrap();
/// let mut network = Network::new(&client).unwrap();
/// network.peer = "127.0.0.1:12345".parse().unwrap();
/// ```
///
/// ```compile_fail,E0616
/// use std::net::UdpSocket;
/// use yetlet_hosts::udp::UdpEndpoint;
/// use yt283_revision_audit::three_way::Network;
/// let client = UdpSocket::bind("127.0.0.1:0").unwrap();
/// let mut network = Network::new(&client).unwrap();
/// network.endpoint = UdpEndpoint::bind_loopback(96).unwrap();
/// ```
pub struct Network {
    endpoint: UdpEndpoint,
    reservation: Option<UdpSocket>,
}
impl Network {
    pub fn new(client: &UdpSocket) -> io::Result<Self> {
        let endpoint = UdpEndpoint::bind_loopback(96)?;
        let reservation = client.try_clone()?;
        reservation.local_addr()?;
        Ok(Self {
            endpoint,
            reservation: Some(reservation),
        })
    }
    /// Current application endpoint address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.endpoint.local_addr()
    }
    /// Live reserved peer address. Returns NotConnected after close.
    pub fn peer(&self) -> io::Result<SocketAddr> {
        self.reservation
            .as_ref()
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotConnected,
                    "UDP peer reservation is closed",
                )
            })?
            .local_addr()
    }
    pub fn poll(&mut self, p: &Ports) -> io::Result<()> {
        let peer = self.peer()?;
        let mut replies = Vec::new();
        self.endpoint.poll_bounded(4, |b, sender| {
            if sender != peer {
                eprintln!("UDP_REJECTED untrusted-peer");
                return Ok(());
            }
            let Ok(request) = decode(b) else {
                eprintln!("UDP_REJECTED codec");
                return Ok(());
            };
            let result = match request {
                Request::Apply(_) if !p.startup_ready.load(Ordering::Acquire) => Err(
                    io::Error::new(io::ErrorKind::PermissionDenied, "required Hosts not ready"),
                ),
                Request::Apply(command) => p.apply(command),
                Request::Read {
                    generation,
                    revision,
                } => {
                    // The caller-thread read also verifies C1 has committed
                    // this revision, beyond HTTP's publication visibility.
                    let current = p.c1.snapshot();
                    if current.generation != generation || current.revision < revision {
                        Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            "C1 revision absent",
                        ))
                    } else {
                        p.pinned(generation, revision)
                    }
                }
            };
            match result {
                Ok(s) => replies.push(s.text()),
                Err(e)
                    if matches!(
                        e.kind(),
                        io::ErrorKind::InvalidData
                            | io::ErrorKind::NotFound
                            | io::ErrorKind::PermissionDenied
                    ) =>
                {
                    eprintln!("UDP_REJECTED {e}")
                }
                Err(e) => return Err(e),
            }
            Ok(())
        })?;
        for reply in replies {
            if !matches!(
                self.endpoint.send_to(peer, reply.as_bytes())?,
                SendOutcome::Sent(_)
            ) {
                return Err(io::Error::other("UDP response would block"));
            }
        }
        Ok(())
    }
    pub fn close(&mut self) -> io::Result<()> {
        self.endpoint.close()?;
        self.reservation.take();
        eprintln!("UDP_CLOSED reservation_released=true");
        Ok(())
    }
}
/// Real bounded HTTP exchange used by the finite client and focused tests.
pub fn http(addr: SocketAddr, method: &str, path: &str, body: &[u8]) -> io::Result<(u16, String)> {
    let mut stream = TcpStream::connect_timeout(&addr, IO_BUDGET)?;
    stream.set_read_timeout(Some(IO_BUDGET))?;
    stream.set_write_timeout(Some(IO_BUDGET))?;
    write!(
        stream,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    stream.write_all(body)?;
    let mut response = String::new();
    stream.take(8192).read_to_string(&mut response)?;
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| io::Error::other("HTTP framing"))?;
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .ok_or_else(|| io::Error::other("HTTP status"))?;
    Ok((status, body.to_string()))
}
/// The route and admission belong to the same Ports. Neither field may be
/// replaced by another Ports' handles while the worker is being started.
///
/// ```compile_fail
/// use yt283_revision_audit::three_way::{Ports, WebLaunch, router};
/// let a = Ports::new(61).unwrap();
/// let b = Ports::new(62).unwrap();
/// let _mixed = WebLaunch { routes: router(&a), ingress: b.ingress() };
/// ```
pub struct WebLaunch {
    routes: Router,
    ingress: CommandIngress<Command>,
    startup_ready: Arc<AtomicBool>,
}
/// One actual owned HTTP worker whose route and shutdown gate are inseparable.
pub struct Web {
    worker: HttpWorker,
    ingress: CommandIngress<Command>,
    startup_ready: Arc<AtomicBool>,
}
impl Web {
    pub fn endpoint(&self) -> SocketAddr {
        self.worker.endpoint()
    }
    pub fn wait_http_ready(&self, path: &str, budget: Duration) -> io::Result<()> {
        self.worker.wait_http_ready(path, budget)
    }
    pub fn has_pending_join(&self) -> bool {
        self.worker.has_pending_join()
    }
}
pub fn start_web(launch: WebLaunch, bind: SocketAddr) -> io::Result<Web> {
    let WebLaunch {
        routes,
        ingress,
        startup_ready,
    } = launch;
    match HttpWorker::start_on(routes, bind) {
        Ok(worker) => Ok(Web {
            worker,
            ingress,
            startup_ready,
        }),
        Err(error) => {
            startup_ready.store(false, Ordering::Release);
            ingress.close()?;
            Err(error)
        }
    }
}
pub fn close_admission(web: &mut Web) -> io::Result<()> {
    web.startup_ready.store(false, Ordering::Release);
    web.ingress.close()?;
    eprintln!("HTTP_ADMISSION_CLOSED");
    Ok(())
}
pub fn stop_web(web: &mut Web) -> io::Result<()> {
    web.worker.request_stop()
}
pub fn observe_web(
    web: &mut Web,
    budget: Duration,
) -> io::Result<yetlet::composition::WorkerCompletion<io::Error>> {
    let result = web.worker.observe_within(budget)?;
    if matches!(result, yetlet::composition::WorkerCompletion::Joined(_)) {
        eprintln!(
            "HTTP_ORIGINAL_JOIN pending_handle={}",
            web.worker.has_pending_join()
        );
    }
    Ok(result)
}
pub fn close_audit(a: &mut AuditAttachment) -> io::Result<()> {
    a.close()
}
pub fn audit_closed(p: &Ports) -> bool {
    p.audit.status() == AuditStatus::Closed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tick_m1::{HeldDirection, TickCommand, TickError};
    use std::{cell::Cell, net::TcpListener, thread, time::Instant};
    use yetlet::composition::{Builder, Completion, JoinStatus};

    fn m2_input(tick: u64, sequence: u64, command: TickCommand) -> TickInput {
        TickInput {
            generation: 91,
            tick,
            sequence,
            command,
        }
    }

    #[test]
    fn native_clock_bridge_failure_keeps_full_window_and_retries_original_batch() {
        use crate::native_clock::{FIXED_STEP, NativeScheduler};
        use crate::native_input::{Key, KeyResult};

        for reset in [false, true] {
            let mut p = Ports::new(91).unwrap();
            let start = Instant::now();
            let mut scheduler = NativeScheduler::new(&p, start).unwrap();
            for wake in 1..=4 {
                assert_eq!(
                    scheduler
                        .pump(&p, start + FIXED_STEP * (wake * 4))
                        .unwrap()
                        .published_ticks,
                    4
                );
            }
            let before = p.c1().tick_observation();
            let records = p.audit().tick_records();
            let oldest = Arc::clone(&records[0]);
            let saved = (*oldest).clone();
            let history: Vec<_> = (1..=16)
                .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
                .collect();
            let keys: &[Key] = if reset {
                &[Key::Pause, Key::Resume, Key::D]
            } else {
                &[Key::D, Key::Up]
            };
            for &key in keys {
                assert_eq!(
                    scheduler.key(&p, key, true, false).unwrap(),
                    KeyResult::Queued
                );
            }
            // Real CommandBridge rejection; only this private fixture swaps the
            // bridge and saves the exact original for retry. No production seam.
            let rejecting = CommandBridge::new(QUEUE, TICK_EVIDENCE_CAPACITY).unwrap();
            rejecting
                .publish_snapshot(92, 0, Publication::ra1(C1Domain::new(92).snapshot()))
                .unwrap();
            let original = std::mem::replace(&mut p.bridge, rejecting);
            let rejected_history = p.bridge.snapshots().latest().unwrap();
            let now = start + FIXED_STEP * 20 + Duration::from_nanos(3);
            let error = scheduler.pump(&p, now).unwrap_err();
            assert_eq!(error.published_ticks, 0);
            assert_eq!(error.error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(scheduler.pending_input_count(), keys.len());
            assert_eq!(scheduler.timing().pending_ticks, 4);
            assert_eq!(scheduler.timing().remainder, Duration::from_nanos(3));
            assert_eq!(scheduler.timing().reset_ns, 0);
            assert_eq!(scheduler.next_tick_in(), Duration::ZERO);
            assert_eq!(p.c1().tick_observation(), before);
            assert_eq!(p.audit().pinned_tick(), Some(before));
            let current = p.audit().tick_records();
            assert_eq!(current, records);
            for (old, current) in records.iter().zip(&current) {
                assert!(Arc::ptr_eq(old, current));
            }
            assert_eq!(p.bridge.snapshots().latest().unwrap(), rejected_history);
            assert!(p.publication_failure().is_none());
            p.bridge = original;
            assert_eq!(
                (1..=16)
                    .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
                    .collect::<Vec<_>>(),
                history
            );

            let count = if reset { 1 } else { 4 };
            assert_eq!(scheduler.pump(&p, now).unwrap().published_ticks, count);
            assert_eq!(scheduler.pending_input_count(), 0);
            assert_eq!(scheduler.timing().pending_ticks, 0);
            assert_eq!(
                scheduler.timing().reset_ns,
                if reset {
                    FIXED_STEP.as_nanos() * 3 + 3
                } else {
                    0
                }
            );
            let last = p.c1().tick_observation();
            assert_eq!(last.clock.tick, 16 + u64::from(count));
            assert_eq!(last.clock.last_sequence, keys.len() as u64);
            assert_eq!(last.snapshot.revision, last.clock.tick);
            assert_eq!(p.audit().pinned_tick(), Some(last));
            for tick in 17..=last.clock.tick {
                let proof = p.pinned_tick(91, tick).unwrap();
                assert_eq!(proof.inputs.len(), if tick == 17 { keys.len() } else { 0 });
                if tick == 17 {
                    assert_eq!(proof.before, before);
                    for (index, input) in proof.inputs.iter().enumerate() {
                        assert_eq!(
                            (input.generation, input.tick, input.sequence),
                            (91, 17, index as u64 + 1)
                        );
                    }
                }
                assert_eq!(p.pinned(91, tick).unwrap(), proof.applied.snapshot);
            }
            let proof = p.pinned_tick(91, last.clock.tick).unwrap();
            assert!(Arc::ptr_eq(
                p.audit().tick_records().last().unwrap(),
                &proof
            ));
            assert_eq!(p.audit().tick_records().len(), TICK_EVIDENCE_CAPACITY);
            assert_eq!(
                p.pinned_tick(91, 1).unwrap_err().kind(),
                io::ErrorKind::NotFound
            );
            assert_eq!(*oldest, saved);
            assert_eq!(
                scheduler.key(&p, Key::D, true, false).unwrap(),
                KeyResult::Unchanged
            );
        }
    }

    #[test]
    fn native_clock_failed_work_survives_original_composition_close() {
        use crate::native_clock::{FIXED_STEP, NativeScheduler};
        use crate::native_input::Key;

        let mut p = Ports::new(91).unwrap();
        let start = Instant::now();
        let mut scheduler = NativeScheduler::new(&p, start).unwrap();
        scheduler.key(&p, Key::D, true, false).unwrap();
        let before = p.c1().tick_observation();
        let rejecting = CommandBridge::new(QUEUE, TICK_EVIDENCE_CAPACITY).unwrap();
        rejecting
            .publish_snapshot(92, 0, Publication::ra1(C1Domain::new(92).snapshot()))
            .unwrap();
        let original = std::mem::replace(&mut p.bridge, rejecting);
        let now = start + FIXED_STEP * 3;
        assert_eq!(scheduler.pump(&p, now).unwrap_err().published_ticks, 0);
        p.bridge = original;
        let pending = scheduler.timing();
        p.ingress().close().unwrap();
        assert_eq!(scheduler.pump(&p, now).unwrap_err().published_ticks, 0);
        assert_eq!(scheduler.timing(), pending);
        assert_eq!(scheduler.pending_input_count(), 1);
        let owner = Rc::clone(p.c1());
        let audit = Rc::clone(p.audit());
        let attachment = p.audit_attachment();
        let mut root = Builder::<io::Error>::new()
            .required_local("C1", p, Ports::close)
            .required_local("audit", attachment, close_audit)
            .build();
        assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
        assert!(root.failures().primary().is_none());
        assert!(owner.is_stopped());
        assert_eq!(audit.status(), AuditStatus::Closed);
        assert_eq!(owner.tick_observation(), before);
        assert!(audit.pinned_tick().is_none());
        assert!(audit.tick_records().is_empty());
        assert_eq!(scheduler.timing(), pending);
        assert_eq!(pending.pending_ticks, 3);
        assert_eq!(scheduler.pending_input_count(), 1);
    }

    #[test]
    fn native_input_publication_failure_keeps_pending_identity_and_full_evidence() {
        use crate::native_input::{Key, KeyResult, NativeInput};

        let mut p = Ports::new(91).unwrap();
        let mut input = NativeInput::new(&p).unwrap();
        for _ in 0..TICK_EVIDENCE_CAPACITY {
            input.step_tick(&p).unwrap();
        }
        let before = p.c1().tick_observation();
        let records = p.audit().tick_records();
        let oldest = Arc::clone(&records[0]);
        let old_copy = (*oldest).clone();
        let original_history: Vec<_> = (1..=16)
            .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
            .collect();
        for key in [Key::W, Key::Up] {
            assert_eq!(input.key(&p, key, true, false).unwrap(), KeyResult::Queued);
        }
        // Only a private test fixture replaces the bridge, retaining the exact
        // original for retry. Production has no replacement/injection API.
        let rejecting = CommandBridge::new(QUEUE, TICK_EVIDENCE_CAPACITY).unwrap();
        rejecting
            .publish_snapshot(92, 0, Publication::ra1(C1Domain::new(92).snapshot()))
            .unwrap();
        let original = std::mem::replace(&mut p.bridge, rejecting);
        let rejected_history = p.bridge.snapshots().latest().unwrap();
        assert_eq!(
            input.step_tick(&p).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(input.pending_count(), 2);
        assert_eq!(p.c1().tick_observation(), before);
        assert_eq!(p.audit().pinned_tick(), Some(before));
        let current = p.audit().tick_records();
        assert_eq!(current, records);
        for (old, now) in records.iter().zip(&current) {
            assert!(Arc::ptr_eq(old, now));
        }
        assert_eq!(p.bridge.snapshots().latest().unwrap(), rejected_history);
        assert!(p.publication_failure().is_none());
        p.bridge = original;
        assert_eq!(
            (1..=16)
                .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
                .collect::<Vec<_>>(),
            original_history
        );

        let applied = input.step_tick(&p).unwrap();
        assert_eq!(input.pending_count(), 0);
        assert_eq!(applied.clock.tick, 17);
        assert_eq!(applied.clock.last_sequence, 2);
        assert_eq!(applied.clock.held, [HeldDirection::North; 2]);
        let proof = p.pinned_tick(91, 17).unwrap();
        assert_eq!(proof.before, before);
        assert_eq!(proof.applied, applied);
        assert_eq!(
            proof.inputs,
            vec![
                m2_input(17, 1, TickCommand::Hold(Player::One, HeldDirection::North)),
                m2_input(17, 2, TickCommand::Hold(Player::Two, HeldDirection::North)),
            ]
        );
        assert_eq!(p.audit().pinned_tick(), Some(applied));
        let retained = p.audit().tick_records();
        for (old, now) in records[1..].iter().zip(&retained) {
            assert!(Arc::ptr_eq(old, now));
        }
        assert!(Arc::ptr_eq(retained.last().unwrap(), &proof));
        assert_eq!(
            p.pinned_tick(91, 1).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(*oldest, old_copy);
        for key in [Key::W, Key::Up] {
            assert_eq!(
                input.key(&p, key, true, false).unwrap(),
                KeyResult::Unchanged
            );
        }
        input.lost_focus(&p).unwrap();
        let released = input.step_tick(&p).unwrap();
        assert_eq!(released.clock.last_sequence, 4);
        assert_eq!(released.clock.held, [HeldDirection::Still; 2]);
    }

    #[test]
    fn tick_publication_pins_one_canonical_proof_and_retains_old_revisions() {
        let p = Ports::new(91).unwrap();
        let initial = p.c1().tick_observation();
        let one = m2_input(1, 1, TickCommand::Hold(Player::One, HeldDirection::East));
        let two = m2_input(1, 2, TickCommand::Hold(Player::Two, HeldDirection::North));
        let first = p.step_tick(1, &[two, one]).unwrap();
        let retained = p.pinned_tick(91, 1).unwrap();
        assert_eq!(retained.before, initial);
        assert_eq!(retained.applied, first);
        assert_eq!(retained.inputs, vec![one, two]);
        assert_eq!(first.snapshot.text(), "91 1 false 3 2 20 11");
        assert_eq!(
            first.clock.held,
            [HeldDirection::East, HeldDirection::North]
        );
        assert_eq!(p.c1().tick_observation(), first);
        assert_eq!(p.pinned(91, 1).unwrap(), first.snapshot);
        assert_eq!(p.audit().pinned(), first.snapshot);
        assert_eq!(p.audit().pinned_tick(), Some(first));
        assert!(Arc::ptr_eq(&p.audit().tick_records()[0], &retained));
        assert!(p.audit().records().is_empty());

        let second = p.step_tick(2, &[]).unwrap();
        assert_eq!(second.snapshot.text(), "91 2 false 4 2 20 10");
        assert_eq!(p.pinned_tick(91, 2).unwrap().before, first);
        assert_eq!(second.clock.last_sequence, 2);
        assert_eq!(*p.pinned_tick(91, 1).unwrap(), *retained);
        assert_eq!(retained.applied, first);
        let paused = p
            .step_tick(3, &[m2_input(3, 3, TickCommand::Pause)])
            .unwrap();
        assert!(paused.snapshot.paused);
        assert_eq!(paused.snapshot.positions, second.snapshot.positions);
        assert_eq!(paused.clock.active_ticks, 2);
        assert_eq!(paused.clock.held, [HeldDirection::Still; 2]);
        let resumed = p
            .step_tick(4, &[m2_input(4, 4, TickCommand::Resume)])
            .unwrap();
        assert!(!resumed.snapshot.paused);
        assert_eq!(resumed.snapshot.positions, paused.snapshot.positions);
        assert_eq!(resumed.clock.active_ticks, 3);
        assert_eq!(p.audit().pinned_tick(), Some(resumed));
        for record in p.audit().tick_records() {
            let published = p.pinned_tick(91, record.applied.snapshot.revision).unwrap();
            assert!(Arc::ptr_eq(&published, &record));
            assert_eq!(
                p.pinned(91, record.applied.snapshot.revision).unwrap(),
                record.applied.snapshot
            );
        }
    }

    #[test]
    fn tick_retention_preflight_failure_preserves_full_window_and_input_sequence() {
        let p = Ports::new(91).unwrap();
        p.step_tick(
            1,
            &[m2_input(
                1,
                1,
                TickCommand::Hold(Player::One, HeldDirection::East),
            )],
        )
        .unwrap();
        for tick in 2..=16 {
            p.step_tick(tick, &[]).unwrap();
        }
        let before = p.c1().tick_observation();
        let records = p.audit().tick_records();
        let history: Vec<_> = (1..=16)
            .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
            .collect();
        let valid = m2_input(17, 2, TickCommand::Hold(Player::Two, HeldDirection::North));
        let unchanged = || {
            assert_eq!(p.c1().tick_observation(), before);
            assert_eq!(p.audit().pinned(), before.snapshot);
            assert_eq!(p.audit().pinned_tick(), Some(before));
            let current = p.audit().tick_records();
            assert_eq!(current, records);
            for (old, now) in records.iter().zip(&current) {
                assert!(Arc::ptr_eq(old, now));
            }
            assert_eq!(
                (1..=16)
                    .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
                    .collect::<Vec<_>>(),
                history
            );
            assert_eq!(
                p.pinned_tick(91, 17).unwrap_err().kind(),
                io::ErrorKind::NotFound
            );
        };
        let duplicate = [valid, m2_input(17, 2, TickCommand::Pause)];
        assert!(p.step_tick(17, &duplicate).is_err());
        unchanged();
        let guard = p.audit.records.borrow();
        assert!(p.step_tick(17, &[valid]).is_err());
        drop(guard);
        unchanged();
        assert!(p.publication_failure().is_none());
        let applied = p.step_tick(17, &[valid]).unwrap();
        assert_eq!(applied.clock.last_sequence, 2);
        assert_eq!(p.pinned_tick(91, 17).unwrap().before, before);
        assert_eq!(p.audit().pinned_tick(), Some(applied));
        let retained = p.audit().tick_records();
        assert_eq!(retained.len(), 16);
        assert_eq!(retained[0].applied.clock.tick, 2);
        for (old, current) in records[1..].iter().zip(&retained) {
            assert!(Arc::ptr_eq(old, current));
        }
        assert_eq!(
            p.pinned_tick(91, 1).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
    }
    #[test]
    fn real_bridge_tick_rejection_never_partially_commits() {
        for cross_generation in [false, true] {
            let mut p = Ports::new(91).unwrap();
            if cross_generation {
                // Private fault injection reaches the real bridge's rejection.
                p.bridge = CommandBridge::new(QUEUE, 16).unwrap();
                p.bridge
                    .publish_snapshot(92, 0, Publication::ra1(C1Domain::new(92).snapshot()))
                    .unwrap();
            } else {
                p.bridge
                    .publish_snapshot(
                        91,
                        1,
                        Publication::ra1(Snapshot {
                            revision: 1,
                            ..p.c1().snapshot()
                        }),
                    )
                    .unwrap();
            }
            let before = p.c1().tick_observation();
            let history = p.bridge.snapshots().latest().unwrap();
            assert!(
                p.step_tick(
                    1,
                    &[m2_input(
                        1,
                        1,
                        TickCommand::Hold(Player::One, HeldDirection::East)
                    )]
                )
                .is_err()
            );
            assert_eq!(p.c1().tick_observation(), before);
            assert_eq!(p.audit().pinned(), before.snapshot);
            assert!(p.audit().pinned_tick().is_none());
            assert!(p.audit().tick_records().is_empty());
            assert!(p.audit().records().is_empty());
            assert_eq!(p.bridge.snapshots().latest().unwrap(), history);
            assert!(p.publication_failure().is_none());
            assert!(!p.ingress().is_closed().unwrap());
        }
    }

    #[test]
    fn tick_retention_crosses_capacity_without_resetting_identity_or_old_proofs() {
        let p = Ports::new(91).unwrap();
        assert_eq!(TICK_EVIDENCE_CAPACITY, 16);
        assert!(p.audit().pinned_tick().is_none());
        assert!(p.audit().tick_records().is_empty());
        let mut sequence = 0;
        let mut active_ticks = 0;
        let mut retained = None;
        let mut full_capacity = None;
        for tick in 1..=49 {
            let commands: &[TickCommand] = match tick {
                1 => &[
                    TickCommand::Hold(Player::One, HeldDirection::East),
                    TickCommand::Hold(Player::Two, HeldDirection::North),
                ],
                16 | 48 => &[TickCommand::Pause],
                17 => &[TickCommand::Hold(Player::One, HeldDirection::West)],
                19 | 49 => &[TickCommand::Resume],
                32 => &[
                    TickCommand::Hold(Player::One, HeldDirection::West),
                    TickCommand::Hold(Player::Two, HeldDirection::South),
                ],
                _ => &[],
            };
            let canonical: Vec<_> = commands
                .iter()
                .enumerate()
                .map(|(offset, command)| m2_input(tick, sequence + offset as u64 + 1, *command))
                .collect();
            sequence += canonical.len() as u64;
            let mut inputs = canonical.clone();
            inputs.reverse();
            let before = p.c1().tick_observation();
            let applied = p.step_tick(tick, &inputs).unwrap();
            if !(16..=18).contains(&tick) && tick != 48 {
                active_ticks += 1;
            }
            let held = match tick {
                1..=15 => [HeldDirection::East, HeldDirection::North],
                32..=47 => [HeldDirection::West, HeldDirection::South],
                _ => [HeldDirection::Still; 2],
            };
            assert_eq!(applied.snapshot.generation, 91);
            assert_eq!(applied.snapshot.revision, tick);
            assert_eq!(applied.clock.tick, tick);
            assert_eq!(applied.clock.active_ticks, active_ticks);
            assert_eq!(applied.clock.last_sequence, sequence);
            assert_eq!(applied.clock.held, held);
            assert_eq!(p.c1().tick_observation(), applied);
            assert_eq!(p.audit().pinned(), applied.snapshot);
            assert_eq!(p.audit().pinned_tick(), Some(applied));
            assert!(p.audit().records().is_empty());
            let latest = p.pinned_tick(91, tick).unwrap();
            assert_eq!(latest.before, before);
            assert_eq!(latest.applied, applied);
            assert_eq!(latest.inputs, canonical);
            let records = p.audit().tick_records();
            let oldest = tick.saturating_sub(15).max(1);
            assert_eq!(records.len(), tick.min(16) as usize);
            assert_eq!(
                records
                    .iter()
                    .map(|r| r.applied.clock.tick)
                    .collect::<Vec<_>>(),
                (oldest..=tick).collect::<Vec<_>>()
            );
            for record in &records {
                let revision = record.applied.snapshot.revision;
                assert!(Arc::ptr_eq(record, &p.pinned_tick(91, revision).unwrap()));
                assert_eq!(p.pinned(91, revision).unwrap(), record.applied.snapshot);
            }
            if tick == 1 {
                retained = Some((Arc::clone(&latest), (*latest).clone()));
            }
            if tick == 16 {
                full_capacity = Some(p.audit.records.borrow().capacity());
            }
            if tick >= 16 {
                assert_eq!(Some(p.audit.records.borrow().capacity()), full_capacity);
                assert_eq!(
                    p.pinned(91, tick - 16).unwrap_err().kind(),
                    io::ErrorKind::NotFound
                );
                assert_eq!(
                    p.pinned_tick(91, tick - 16).unwrap_err().kind(),
                    io::ErrorKind::NotFound
                );
            }
        }
        let (old_handle, old_value) = retained.unwrap();
        assert_eq!(*old_handle, old_value);
        assert_eq!(old_handle.applied.clock.tick, 1);
        assert_eq!(old_handle.applied.clock.last_sequence, 2);
        assert_eq!(
            p.pinned_tick(91, 1).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(
            p.pinned_tick(92, 49).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(
            p.pinned_tick(91, 50).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
        assert_eq!(p.audit().pinned_tick().unwrap().clock.last_sequence, 9);
        let before = p.c1().tick_observation();
        let records = p.audit().tick_records();
        assert!(
            p.apply(Command {
                generation: 91,
                expected_revision: 49,
                action: Action::Pause,
            })
            .is_err()
        );
        assert_eq!(p.c1().step_tick(50, &[]), Err(TickError::ModeConflict));
        assert_eq!(p.c1().tick_observation(), before);
        assert_eq!(p.audit().tick_records(), records);
    }
    #[test]
    fn tick_retention_preserves_ra1_finite_ledger_and_mode() {
        let p = Ports::new(91).unwrap();
        for revision in 0..16 {
            p.apply(Command {
                generation: 91,
                expected_revision: revision,
                action: if revision % 2 == 0 {
                    Action::Pause
                } else {
                    Action::Resume
                },
            })
            .unwrap();
        }
        let before = p.c1().tick_observation();
        let records = p.audit().records();
        let history: Vec<_> = (1..=16)
            .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
            .collect();
        assert_eq!(before.snapshot.revision, 16);
        assert_eq!(before.clock.tick, 0);
        assert_eq!(records.len(), 16);
        assert!(p.audit().tick_records().is_empty());
        assert!(p.audit().pinned_tick().is_none());
        let error = p
            .apply(Command {
                generation: 91,
                expected_revision: 16,
                action: Action::Pause,
            })
            .unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(error.to_string(), "finite audit ledger full");
        assert!(p.step_tick(1, &[]).is_err());
        assert_eq!(p.c1().tick_observation(), before);
        assert_eq!(p.audit().pinned(), before.snapshot);
        assert_eq!(p.audit().records(), records);
        assert!(p.audit().tick_records().is_empty());
        assert_eq!(
            (1..=16)
                .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
                .collect::<Vec<_>>(),
            history
        );
        assert!(p.bridge.snapshots().at_revision(17).unwrap().is_none());
        assert_eq!(
            p.pinned_tick(91, 16).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
    }

    #[test]
    fn tick_retention_publish_failure_does_not_evict_and_same_input_can_retry() {
        let mut p = Ports::new(91).unwrap();
        for tick in 1..=16 {
            p.step_tick(tick, &[]).unwrap();
        }
        let before = p.c1().tick_observation();
        let records = p.audit().tick_records();
        let original_history: Vec<_> = (1..=16)
            .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
            .collect();
        // Private fault fixture: keep the genuine full history intact while the
        // real bridge rejects a cross-generation publication into another full window.
        let rejecting = CommandBridge::new(QUEUE, TICK_EVIDENCE_CAPACITY).unwrap();
        for revision in 1..=16 {
            rejecting
                .publish_snapshot(
                    92,
                    revision,
                    Publication::ra1(Snapshot {
                        revision,
                        ..C1Domain::new(92).snapshot()
                    }),
                )
                .unwrap();
        }
        let original = std::mem::replace(&mut p.bridge, rejecting);
        let rejected_history: Vec<_> = (1..=16)
            .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
            .collect();
        let next = m2_input(17, 1, TickCommand::Hold(Player::One, HeldDirection::East));
        assert_eq!(
            p.step_tick(17, &[next]).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(p.c1().tick_observation(), before);
        assert_eq!(p.audit().pinned(), before.snapshot);
        assert_eq!(p.audit().pinned_tick(), Some(before));
        let unchanged = p.audit().tick_records();
        assert_eq!(unchanged, records);
        for (old, current) in records.iter().zip(&unchanged) {
            assert!(Arc::ptr_eq(old, current));
        }
        assert_eq!(
            (1..=16)
                .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
                .collect::<Vec<_>>(),
            rejected_history
        );
        assert!(p.bridge.snapshots().at_revision(17).unwrap().is_none());
        assert!(p.publication_failure().is_none());
        assert!(!p.ingress().is_closed().unwrap());
        p.bridge = original;
        assert_eq!(
            (1..=16)
                .map(|revision| p.bridge.snapshots().at_revision(revision).unwrap())
                .collect::<Vec<_>>(),
            original_history
        );
        let applied = p.step_tick(17, &[next]).unwrap();
        assert_eq!(applied.clock.last_sequence, 1);
        assert_eq!(p.audit().pinned_tick(), Some(applied));
        let retained = p.audit().tick_records();
        assert_eq!(retained.len(), 16);
        assert_eq!(retained[0].applied.clock.tick, 2);
        for (old, current) in records[1..].iter().zip(&retained) {
            assert!(Arc::ptr_eq(old, current));
        }
        assert!(Arc::ptr_eq(&retained[15], &p.pinned_tick(91, 17).unwrap()));
        assert_eq!(
            p.pinned_tick(91, 1).unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
    }
    #[test]
    fn accepted_mode_is_exclusive_and_public_c1_cannot_bypass_ports() {
        let ra1 = Ports::new(91).unwrap();
        assert!(ra1.step_tick(2, &[]).is_err());
        let applied = ra1
            .apply(Command {
                generation: 91,
                expected_revision: 0,
                action: Action::Pause,
            })
            .unwrap();
        assert!(ra1.step_tick(1, &[]).is_err());
        assert_eq!(ra1.c1().snapshot(), applied);
        assert_eq!(ra1.audit().records().len(), 1);
        assert!(ra1.audit().tick_records().is_empty());
        assert!(ra1.pinned_tick(91, 1).is_err());

        let ticked = Ports::new(91).unwrap();
        let before = ticked.step_tick(1, &[]).unwrap();
        let history = ticked.bridge.snapshots().latest().unwrap();
        let records = ticked.audit().tick_records();
        let ra1_command = Command {
            generation: 91,
            expected_revision: 1,
            action: Action::Pause,
        };
        assert!(ticked.apply(ra1_command).is_err());
        ticked.ingress().try_submit(ra1_command).unwrap();
        ticked.drain().unwrap();
        assert_eq!(ticked.c1().step_tick(2, &[]), Err(TickError::ModeConflict));
        assert_eq!(ticked.c1().close_tick_m1(), Err(TickError::ModeConflict));
        assert!(!ticked.c1().is_stopped());
        assert_eq!(ticked.c1().tick_observation(), before);
        assert_eq!(ticked.audit().pinned_tick(), Some(before));
        assert_eq!(ticked.audit().tick_records(), records);
        assert!(ticked.audit().records().is_empty());
        assert_eq!(ticked.bridge.snapshots().latest().unwrap(), history);
    }

    #[test]
    fn tick_retention_concurrent_reader_observes_complete_envelopes() {
        use std::sync::mpsc;
        let p = Ports::new(91).unwrap();
        let reader = p.bridge.snapshots();
        let (ready_tx, ready_rx) = mpsc::sync_channel(0);
        let (observed_tx, observed_rx) = mpsc::sync_channel(0);
        let budget = Duration::from_secs(2);
        let worker = thread::spawn(move || {
            for revision in 1..=33 {
                let previous = reader.at_revision(revision - 1).unwrap().unwrap();
                ready_tx.send(revision).unwrap();
                let deadline = Instant::now() + budget;
                loop {
                    let published = reader.latest().unwrap().unwrap();
                    if published.revision < revision {
                        assert!(Instant::now() < deadline, "tick publication timed out");
                        thread::yield_now();
                        continue;
                    }
                    assert_eq!(published.generation, 91);
                    assert_eq!(published.revision, revision);
                    let proof = published.snapshot.tick.as_ref().unwrap();
                    assert_eq!(published.snapshot.snapshot, proof.applied.snapshot);
                    assert_eq!(proof.applied.snapshot.generation, published.generation);
                    assert_eq!(proof.applied.snapshot.revision, published.revision);
                    assert_eq!(proof.applied.clock.tick, revision);
                    assert_eq!(proof.applied.clock.active_ticks, revision);
                    assert_eq!(proof.applied.clock.last_sequence, revision);
                    assert_eq!(
                        proof.applied.clock.held,
                        [HeldDirection::East, HeldDirection::Still]
                    );
                    assert_eq!(proof.before.snapshot, previous.snapshot.snapshot);
                    assert_eq!(proof.inputs.len(), 1);
                    assert_eq!(proof.inputs[0].tick, revision);
                    assert_eq!(proof.inputs[0].sequence, revision);
                    assert_eq!(reader.at_revision(revision - 1).unwrap().unwrap(), previous);
                    observed_tx.send(revision).unwrap();
                    break;
                }
            }
        });
        for tick in 1..=33 {
            assert_eq!(ready_rx.recv_timeout(budget).unwrap(), tick);
            let applied = p
                .step_tick(
                    tick,
                    &[m2_input(
                        tick,
                        tick,
                        TickCommand::Hold(Player::One, HeldDirection::East),
                    )],
                )
                .unwrap();
            assert_eq!(observed_rx.recv_timeout(budget).unwrap(), tick);
            assert_eq!(p.audit().pinned_tick(), Some(applied));
            assert_eq!(p.pinned_tick(91, tick).unwrap().applied, applied);
        }
        worker.join().unwrap();
    }

    #[test]
    fn tick_retention_completes_original_composition_before_and_after_turnover() {
        for scenario in 0..4 {
            let p = Ports::new(91).unwrap();
            match scenario {
                0 => {}
                1 => {
                    p.step_tick(1, &[]).unwrap();
                }
                2 => {
                    assert!(p.step_tick(2, &[]).is_err());
                }
                _ => {
                    for tick in 1..=33 {
                        p.step_tick(tick, &[]).unwrap();
                    }
                }
            }
            let owner = Rc::clone(p.c1());
            let audit = Rc::clone(p.audit());
            let before = owner.tick_observation();
            let records = audit.tick_records();
            let attachment = p.audit_attachment();
            p.ingress().close().unwrap();
            assert!(p.step_tick(before.clock.tick + 1, &[]).is_err());
            assert_eq!(owner.tick_observation(), before);
            let mut root = Builder::<io::Error>::new()
                .required_local("C1", p, Ports::close)
                .required_local("audit", attachment, close_audit)
                .build();
            assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
            assert!(owner.is_stopped());
            assert_eq!(owner.tick_observation(), before);
            assert_eq!(audit.status(), AuditStatus::Closed);
            assert_eq!(audit.pinned(), before.snapshot);
            assert_eq!(audit.tick_records(), records);
            if scenario == 1 || scenario == 3 {
                assert_eq!(audit.pinned_tick(), Some(before));
            }
            assert!(root.failures().primary().is_none());
            assert_eq!(
                owner.step_tick(before.clock.tick + 1, &[]),
                Err(TickError::Stopped)
            );
        }
    }

    #[test]
    fn tick_retention_keeps_original_http_udp_projection_and_owned_close() {
        let mut p = Ports::new(91).unwrap();
        for tick in 1..=32 {
            p.step_tick(tick, &[]).unwrap();
        }
        let applied = p
            .step_tick(
                33,
                &[m2_input(
                    33,
                    1,
                    TickCommand::Hold(Player::Two, HeldDirection::North),
                )],
            )
            .unwrap();
        let mut web = start_web(p.web_launch(), "127.0.0.1:0".parse().unwrap()).unwrap();
        web.wait_http_ready("/ready", IO_BUDGET).unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        client.set_read_timeout(Some(IO_BUDGET)).unwrap();
        let mut network = Network::new(&client).unwrap();
        let response = http(web.endpoint(), "GET", "/snapshot/91/33", &[]).unwrap();
        assert_eq!(response.0, 200);
        assert_eq!(response.1.trim(), applied.snapshot.text());
        let frame = encode_read(91, 33);
        assert_eq!(frame.len(), 20);
        client
            .send_to(&frame, network.local_addr().unwrap())
            .unwrap();
        network.poll(&p).unwrap();
        let mut bytes = [0; 96];
        let length = client.recv(&mut bytes).unwrap();
        assert_eq!(&bytes[..length], applied.snapshot.text().as_bytes());
        assert_eq!(p.audit().pinned_tick(), Some(applied));
        assert_eq!(p.pinned_tick(91, 33).unwrap().applied, applied);
        close_admission(&mut web).unwrap();
        network.close().unwrap();
        close_audit(&mut p.audit_attachment()).unwrap();
        p.close().unwrap();
        stop_web(&mut web).unwrap();
        assert!(matches!(
            observe_web(&mut web, IO_BUDGET).unwrap(),
            yetlet::composition::WorkerCompletion::Joined(Ok(()))
        ));
        assert!(!web.has_pending_join());
        assert!(p.c1().is_stopped());
    }

    #[test]
    fn ra1_fixed_move_tags_round_trip_and_read_is_distinct() {
        for (player, first) in [(Player::One, 4), (Player::Two, 8)] {
            for (offset, direction) in [
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ]
            .into_iter()
            .enumerate()
            {
                let action = Action::Move { player, direction };
                let frame = encode_command(action, u64::MAX, 6);
                assert_eq!(frame[3], first + offset as u8);
                assert_eq!(
                    decode(&frame).unwrap(),
                    Request::Apply(Command {
                        generation: u64::MAX,
                        expected_revision: 6,
                        action
                    })
                );
                assert!(decode(&frame[..FRAME_BYTES - 1]).is_err());
            }
        }
        assert_eq!(
            decode(&encode_read(42, 6)).unwrap(),
            Request::Read {
                generation: 42,
                revision: 6
            }
        );
    }

    fn wait_udp(n: &mut Network, p: &Ports) {
        let deadline = Instant::now() + IO_BUDGET;
        while Instant::now() < deadline {
            n.poll(p).unwrap();
            if p.c1.snapshot().revision == 2 {
                return;
            }
            thread::sleep(Duration::from_millis(2));
        }
        panic!("UDP application timeout");
    }
    #[test]
    fn distinct_ports_never_advance_another_ports_unpublished_revision() {
        let first = Ports::new(54).unwrap();
        let second = Ports::new(54).unwrap();
        assert!(!Rc::ptr_eq(first.c1(), second.c1()));
        assert!(!Rc::ptr_eq(first.audit(), second.audit()));
        let applied = second
            .apply(Command {
                generation: 54,
                expected_revision: 0,
                action: Action::Pause,
            })
            .unwrap();
        assert_eq!(applied.revision, 1);
        assert_eq!(second.pinned(54, 1).unwrap(), applied);
        assert_eq!(second.audit().pinned(), applied);
        assert_eq!(first.c1().snapshot().revision, 0);
        assert!(first.audit().records().is_empty());
        assert!(first.pinned(54, 1).is_err());
    }

    #[test]
    fn a_closed_forged_audit_cannot_stop_the_real_ports_authority() {
        let mut ports = Ports::new(55).unwrap();
        let forged = Rc::new(RevisionAuditHost::new(ports.c1().snapshot()));
        // Same-valued audit is not the original attached audit.
        forged.close().unwrap();
        let mut attachment = ports.audit_attachment();
        assert!(Rc::ptr_eq(attachment.audit(), ports.audit()));
        assert!(
            close_audit(&mut attachment).is_err(),
            "owning gate still open"
        );
        assert_eq!(ports.audit().status(), AuditStatus::Open);
        assert!(ports.close().is_err(), "admission must close first");
        assert!(!ports.c1().is_stopped());
        ports.ingress().close().unwrap();
        assert!(ports.close().is_err(), "the actual audit is still open");
        assert!(!ports.c1().is_stopped());
        close_audit(&mut attachment).unwrap();
        ports.close().unwrap();
        assert!(ports.c1().is_stopped());
    }

    #[test]
    fn real_router_blocks_pre_native_http_then_applies_after_aggregate_ready() {
        let p = Ports::new(82).unwrap();
        let audit = p.audit_attachment();
        let web = start_web(p.web_launch(), "127.0.0.1:0".parse().unwrap()).unwrap();
        web.wait_http_ready("/ready", IO_BUDGET).unwrap();
        let address = web.endpoint();
        // Synthetic foreground readiness only: actual Native/Present is
        // verified in the separate OS app, not fabricated by this test.
        let mut root = Builder::<io::Error>::new()
            .required_local("C1", p, Ports::close)
            .required_local("audit", audit, close_audit)
            .required_foreground("synthetic Native", (), |_| Ok(()))
            .required_worker("Web", web, close_admission, stop_web, observe_web)
            .build();
        assert!(root.parts_mut().last().mark_ready());
        assert!(!root.all_required_ready());
        let body = encode_command(
            Action::Move {
                player: Player::One,
                direction: Direction::East,
            },
            82,
            0,
        );
        assert_eq!(http(address, "GET", "/ready", &[]).unwrap().0, 200);
        assert_eq!(http(address, "POST", "/command", &body).unwrap().0, 503);
        let authority = root.parts().previous().previous().previous().last().value();
        authority.drain().unwrap();
        assert_eq!(authority.c1().snapshot().revision, 0);
        assert!(authority.audit().records().is_empty());
        assert!(root.parts_mut().previous().last().mark_ready());
        assert!(root.all_required_ready());
        let authority = root.parts().previous().previous().previous().last().value();
        authority.enable_after_required_ready().unwrap();
        assert_eq!(http(address, "POST", "/command", &body).unwrap().0, 202);
        assert_eq!(authority.c1().snapshot().revision, 0);
        assert!(authority.audit().records().is_empty());
        authority.drain().unwrap();
        assert_eq!(authority.c1().snapshot().text(), "82 1 false 3 2 20 12");
        assert_eq!(authority.audit().records().len(), 1);
        assert_eq!(authority.audit().pinned(), authority.c1().snapshot());
        root.request_stop();
        assert_eq!(
            root.complete_until(Instant::now() + IO_BUDGET),
            Completion::Complete
        );
        assert_eq!(root.parts().last().join_status(), JoinStatus::Succeeded);
        assert!(http(address, "GET", "/ready", &[]).is_err());
    }
    #[test]
    fn closing_one_live_web_never_closes_another_ports_admission() {
        let mut a = Ports::new(71).unwrap();
        let mut b = Ports::new(72).unwrap();
        let mut web_a = start_web(a.web_launch(), "127.0.0.1:0".parse().unwrap()).unwrap();
        let mut web_b = start_web(b.web_launch(), "127.0.0.1:0".parse().unwrap()).unwrap();
        web_a.wait_http_ready("/ready", IO_BUDGET).unwrap();
        web_b.wait_http_ready("/ready", IO_BUDGET).unwrap();
        // Both synthetic host groups are declared ready for this focused
        // cross-Ports shutdown identity test.
        a.enable_after_required_ready().unwrap();
        b.enable_after_required_ready().unwrap();
        assert!(Rc::ptr_eq(a.audit(), a.audit_attachment().audit()));
        close_admission(&mut web_a).unwrap();
        assert!(a.ingress().is_closed().unwrap());
        assert!(!b.ingress().is_closed().unwrap());
        assert_eq!(
            http(
                web_a.endpoint(),
                "POST",
                "/command",
                &encode_command(Action::Pause, 71, 0)
            )
            .unwrap()
            .0,
            503,
            "A's actual command route must share A's shutdown admission"
        );
        assert_eq!(
            http(
                web_b.endpoint(),
                "POST",
                "/command",
                &encode_command(Action::Pause, 72, 0)
            )
            .unwrap()
            .0,
            202,
            "B's independent admission must remain open"
        );
        assert_eq!(a.c1().snapshot().revision, 0);
        assert_eq!(b.c1().snapshot().revision, 0, "202 only enqueues");
        b.drain().unwrap();
        assert_eq!(b.c1().snapshot().revision, 1);
        assert_eq!(b.audit().records().len(), 1);
        assert_eq!(b.pinned(72, 1).unwrap(), b.c1().snapshot());
        close_admission(&mut web_b).unwrap();
        let mut audit_a = a.audit_attachment();
        let mut audit_b = b.audit_attachment();
        close_audit(&mut audit_a).unwrap();
        close_audit(&mut audit_b).unwrap();
        a.close().unwrap();
        b.close().unwrap();
        stop_web(&mut web_a).unwrap();
        stop_web(&mut web_b).unwrap();
        assert!(matches!(
            observe_web(&mut web_a, IO_BUDGET).unwrap(),
            yetlet::composition::WorkerCompletion::Joined(Ok(()))
        ));
        assert!(matches!(
            observe_web(&mut web_b, IO_BUDGET).unwrap(),
            yetlet::composition::WorkerCompletion::Joined(Ok(()))
        ));
    }

    #[test]
    fn early_cloned_audit_close_keeps_real_http_202_command_applicable() {
        let mut ports = Ports::new(56).unwrap();
        let cloned = Rc::clone(ports.audit());
        let mut attached = ports.audit_attachment();
        assert!(Rc::ptr_eq(&cloned, attached.audit()));
        assert!(close_audit(&mut attached).is_err());
        assert_eq!(cloned.status(), AuditStatus::Open);
        let mut web = HttpWorker::start(router(&ports)).unwrap();
        web.wait_http_ready("/ready", IO_BUDGET).unwrap();
        ports.enable_after_required_ready().unwrap(); // synthetic host readiness
        assert_eq!(
            http(
                web.endpoint(),
                "POST",
                "/command",
                &encode_command(Action::Pause, 56, 0)
            )
            .unwrap()
            .0,
            202
        );
        assert_eq!(
            ports.c1().snapshot().revision,
            0,
            "HTTP202 is admission only"
        );
        ports.drain().unwrap();
        let applied = ports.c1().snapshot();
        assert_eq!(applied.revision, 1);
        assert!(applied.paused);
        assert_eq!(ports.pinned(56, 1).unwrap(), applied);
        assert_eq!(cloned.pinned(), applied);
        assert_eq!(cloned.records().len(), 1);
        assert!(close_audit(&mut attached).is_err());
        assert_eq!(cloned.status(), AuditStatus::Open);
        ports.ingress().close().unwrap();
        close_audit(&mut attached).unwrap();
        ports.close().unwrap();
        assert_eq!(cloned.status(), AuditStatus::Closed);
        assert!(ports.c1().is_stopped());
        web.request_stop().unwrap();
        assert!(matches!(
            web.observe_within(IO_BUDGET).unwrap(),
            yetlet::composition::WorkerCompletion::Joined(Ok(()))
        ));
    }

    #[test]
    fn real_web_admission_and_udp_share_one_c1_and_audit() {
        let mut p = Ports::new(42).unwrap();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        client.set_read_timeout(Some(IO_BUDGET)).unwrap();
        let mut network = Network::new(&client).unwrap();
        let mut worker = HttpWorker::start(router(&p)).unwrap();
        worker.wait_http_ready("/ready", IO_BUDGET).unwrap();
        p.enable_after_required_ready().unwrap(); // synthetic host readiness
        let one = Action::Move {
            player: Player::One,
            direction: Direction::East,
        };
        let two = Action::Move {
            player: Player::Two,
            direction: Direction::North,
        };
        let mut bad = encode_command(one, 42, 0);
        bad[3] = 12;
        for body in [encode_read(42, 0), bad] {
            assert_eq!(
                http(worker.endpoint(), "POST", "/command", &body)
                    .unwrap()
                    .0,
                400
            );
        }
        assert_eq!(
            http(worker.endpoint(), "POST", "/command", &[0; FRAME_BYTES + 1])
                .unwrap()
                .0,
            413
        );
        assert_eq!(
            http(
                worker.endpoint(),
                "POST",
                "/command",
                &encode_command(one, 42, 0)
            )
            .unwrap()
            .0,
            202
        );
        assert_eq!(p.c1.snapshot().revision, 0);
        assert!(p.audit.records().is_empty());
        assert_eq!(
            http(worker.endpoint(), "GET", "/snapshot/42/1", &[])
                .unwrap()
                .0,
            404
        );
        p.drain().unwrap();
        let r1 = p.c1.snapshot();
        assert_eq!(r1.text(), "42 1 false 3 2 20 12");
        assert_eq!(
            http(worker.endpoint(), "GET", "/snapshot/42/1", &[]).unwrap(),
            (200, r1.text())
        );
        let address = network.local_addr().unwrap();
        client.send_to(&encode_read(42, 1), address).unwrap();
        let mut bytes = [0; 96];
        let deadline = Instant::now() + IO_BUDGET;
        client.set_nonblocking(true).unwrap();
        loop {
            network.poll(&p).unwrap();
            match client.recv_from(&mut bytes) {
                Ok((len, peer)) => {
                    assert_eq!(peer, address);
                    assert_eq!(&bytes[..len], r1.text().as_bytes());
                    break;
                }
                Err(e) if e.kind() == io::ErrorKind::WouldBlock && Instant::now() < deadline => {
                    thread::yield_now();
                }
                other => panic!("expected pinned UDP read, got {other:?}"),
            }
        }
        client.set_nonblocking(false).unwrap();
        assert_eq!(p.c1.snapshot(), r1);
        client
            .send_to(&encode_command(two, 42, 1), address)
            .unwrap();
        wait_udp(&mut network, &p);
        let mut b = [0; 96];
        let len = client.recv(&mut b).unwrap();
        let exact = p.c1.snapshot();
        assert_eq!(exact.text(), "42 2 false 3 2 20 11");
        assert_eq!(&b[..len], exact.text().as_bytes());
        assert_eq!(
            http(worker.endpoint(), "GET", "/snapshot/42/2", &[]).unwrap(),
            (200, exact.text())
        );
        assert_eq!(p.audit.pinned(), exact);
        assert_eq!(p.audit.records().len(), 2);
        for record in p.audit.records() {
            assert_eq!(
                p.pinned(record.applied.generation, record.applied.revision)
                    .unwrap(),
                record.applied
            );
        }
        drop(client);
        assert!(UdpSocket::bind(network.peer().unwrap()).is_err());
        p.ingress().close().unwrap();
        network.close().unwrap();
        let mut audit = p.audit_attachment();
        close_audit(&mut audit).unwrap();
        p.close().unwrap();
        worker.request_stop().unwrap();
        assert!(matches!(
            worker.observe_within(IO_BUDGET).unwrap(),
            yetlet::composition::WorkerCompletion::Joined(Ok(()))
        ));
    }
    #[test]
    fn real_bridge_rejected_publication_does_not_commit_domain_or_audit() {
        for cross_generation in [false, true] {
            let mut p = Ports::new(46).unwrap();
            if cross_generation {
                // A real bridge pinned to another authority rejects generation46.
                p.bridge = CommandBridge::new(QUEUE, 16).unwrap();
                p.bridge
                    .publish_snapshot(99, 0, Publication::ra1(C1Domain::new(99).snapshot()))
                    .unwrap();
            } else {
                // The real history already owns revision1: publish must collide.
                p.bridge
                    .publish_snapshot(
                        46,
                        1,
                        Publication::ra1(Snapshot {
                            revision: 1,
                            ..p.c1.snapshot()
                        }),
                    )
                    .unwrap();
            }
            let before = p.c1.snapshot();
            let history = p.bridge.snapshots().latest().unwrap();
            let error = p
                .apply(Command {
                    generation: 46,
                    expected_revision: 0,
                    action: Action::Pause,
                })
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(p.c1.snapshot(), before);
            assert_eq!(p.audit.pinned(), before);
            assert!(p.audit.records().is_empty());
            assert!(p.publication_failure().is_none());
            assert_eq!(p.bridge.snapshots().latest().unwrap(), history);
            p.ingress().close().unwrap();
            let mut audit = p.audit_attachment();
            close_audit(&mut audit).unwrap();
            p.close().unwrap();
        }
    }
    #[test]
    fn postpublication_missing_or_wrong_readback_latches_first_failure_and_pending() {
        for wrong_state in [false, true] {
            let p = Ports::new(47).unwrap();
            let c1 = p.c1.clone();
            let audit = p.audit.clone();
            let command = Command {
                generation: 47,
                expected_revision: 0,
                action: Action::Pause,
            };
            let prepared = c1.preflight(command, &audit).unwrap();
            let candidate = prepared.candidate();
            assert_eq!(c1.snapshot().revision, 0);
            assert!(audit.records().is_empty());
            p.bridge
                .publish_snapshot(47, 1, Publication::ra1(candidate))
                .unwrap();
            // Public APIs cannot poison Snapshot's private history lock. Supply
            // a real unavailable/wrong reader at the production finish seam.
            let unavailable = CommandBridge::<Command, Publication>::new(QUEUE, 16).unwrap();
            if wrong_state {
                unavailable
                    .publish_snapshot(
                        47,
                        1,
                        Publication::ra1(Snapshot {
                            paused: false,
                            ..candidate
                        }),
                    )
                    .unwrap();
            }
            let first = p
                .finish_published(prepared, &unavailable.snapshots())
                .unwrap_err();
            let retained = p.publication_failure().unwrap();
            assert_eq!(p.pinned(47, 1).unwrap(), candidate); // Never fake rollback.
            assert_eq!(c1.snapshot().revision, 0);
            assert_eq!(audit.pinned(), c1.snapshot());
            assert!(audit.records().is_empty());
            assert!(p.ingress().is_closed().unwrap());
            assert_eq!(p.apply(command).unwrap_err().to_string(), first.to_string());
            assert!(Arc::ptr_eq(&retained, &p.publication_failure().unwrap()));
            let audit_attachment = p.audit_attachment();
            let mut root = Builder::<io::Error>::new()
                .required_local("C1", p, Ports::close)
                .required_local("audit", audit_attachment, close_audit)
                .build();
            root.fail_required(first);
            assert_eq!(root.complete_until(Instant::now()), Completion::Pending);
            assert!(!c1.is_stopped());
            assert!(Rc::strong_count(&c1) > 1);
            assert_eq!(
                root.failures().primary().unwrap().to_string(),
                retained.to_string()
            );
            assert_eq!(
                root.parts()
                    .previous()
                    .last()
                    .value()
                    .pinned(47, 1)
                    .unwrap(),
                candidate
            );
        }
    }
    #[test]
    fn only_the_reserved_peer_can_apply_a_move_and_receive_its_snapshot() {
        let p = Ports::new(73).unwrap();
        let reserved = UdpSocket::bind("127.0.0.1:0").unwrap();
        let other = UdpSocket::bind("127.0.0.1:0").unwrap();
        reserved.set_read_timeout(Some(IO_BUDGET)).unwrap();
        other
            .set_read_timeout(Some(Duration::from_millis(30)))
            .unwrap();
        let mut network = Network::new(&reserved).unwrap();
        let address = network.local_addr().unwrap();
        let peer = reserved.local_addr().unwrap();
        assert_eq!(network.peer().unwrap(), peer);
        let movement = encode_command(
            Action::Move {
                player: Player::Two,
                direction: Direction::North,
            },
            73,
            0,
        );

        let mut bytes = [0; 96];
        other.send_to(&movement, address).unwrap();
        network.poll(&p).unwrap();
        assert!(matches!(
            other.recv(&mut bytes).unwrap_err().kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ));
        assert_eq!(p.c1().snapshot().revision, 0);
        assert!(p.audit().records().is_empty());
        p.enable_after_required_ready().unwrap(); // synthetic host readiness

        reserved.send_to(&movement, address).unwrap();
        network.poll(&p).unwrap();
        let (length, sender) = reserved.recv_from(&mut bytes).unwrap();
        assert_eq!(sender, address);
        assert_eq!(&bytes[..length], b"73 1 false 2 2 20 11");
        let applied = p.c1().snapshot();
        assert_eq!(applied.text(), "73 1 false 2 2 20 11");
        assert_eq!(p.pinned(73, 1).unwrap(), applied);
        assert_eq!(p.audit().pinned(), applied);
        assert_eq!(p.audit().records().len(), 1);
        assert!(matches!(
            other.recv(&mut bytes).unwrap_err().kind(),
            io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
        ));

        drop(reserved);
        assert_eq!(
            UdpSocket::bind(peer).unwrap_err().kind(),
            io::ErrorKind::AddrInUse
        );
        network.close().unwrap();
        assert_eq!(
            network.peer().unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
        assert_eq!(
            network.local_addr().unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
        assert!(UdpSocket::bind(peer).is_ok());
        assert!(UdpSocket::bind(address).is_ok());
    }
    #[test]
    fn strict_udp_rejects_wrong_peer_codec_and_stale_generation() {
        let p = Ports::new(43).unwrap();
        p.enable_after_required_ready().unwrap(); // exercise strict UDP codec, not readiness
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let attacker = UdpSocket::bind("127.0.0.1:0").unwrap();
        let mut n = Network::new(&client).unwrap();
        let addr = n.local_addr().unwrap();
        let action = Action::Move {
            player: Player::Two,
            direction: Direction::West,
        };
        attacker
            .send_to(&encode_command(action, 43, 0), addr)
            .unwrap();
        let mut bad = encode_command(action, 43, 0);
        bad[3] = 12;
        client.send_to(&bad, addr).unwrap();
        client
            .send_to(&encode_command(action, 42, 0), addr)
            .unwrap();
        client
            .send_to(&encode_command(action, 43, 1), addr)
            .unwrap();
        for _ in 0..10 {
            n.poll(&p).unwrap();
            thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(p.c1.snapshot().revision, 0);
        assert!(p.audit.records().is_empty());
        n.close().unwrap();
    }
    #[test]
    fn web_bind_collision_rolls_back_acquired_c1_audit_udp() {
        let p = Ports::new(44).unwrap();
        let c = p.c1.clone();
        let a = p.audit.clone();
        let web_launch = p.web_launch();
        let client = UdpSocket::bind("127.0.0.1:0").unwrap();
        let n = Network::new(&client).unwrap();
        let udp = n.local_addr().unwrap();
        let peer = n.peer().unwrap();
        drop(client);
        let blocker = TcpListener::bind("127.0.0.1:0").unwrap();
        let audit_attachment = p.audit_attachment();
        let failed = Builder::<io::Error>::new()
            .required_local("C1", p, Ports::close)
            .required_local("audit", audit_attachment, close_audit)
            .required_local("udp", n, Network::close)
            .try_required_worker(
                "web",
                || start_web(web_launch, blocker.local_addr().unwrap()),
                close_admission,
                stop_web,
                observe_web,
            );
        let mut root = match failed {
            Err(failure) => failure.into_root(),
            Ok(_) => panic!("bind collision unexpectedly succeeded"),
        };
        assert_eq!(
            root.complete_until(Instant::now() + IO_BUDGET),
            Completion::Complete
        );
        assert!(c.is_stopped());
        assert_eq!(a.status(), AuditStatus::Closed);
        assert!(root.failures().primary().is_some());
        assert!(UdpSocket::bind(udp).is_ok());
        assert!(UdpSocket::bind(peer).is_ok());
    }
    #[test]
    fn external_audit_blocker_keeps_original_web_join_and_c1_until_retry() {
        let p = Ports::new(45).unwrap();
        let c = p.c1.clone();
        let a = p.audit.clone();
        let allow = Rc::new(Cell::new(false));
        let gate = allow.clone();
        let web = start_web(p.web_launch(), "127.0.0.1:0".parse().unwrap()).unwrap();
        web.wait_http_ready("/ready", IO_BUDGET).unwrap();
        let address = web.endpoint();
        let audit_attachment = p.audit_attachment();
        let mut root = Builder::<io::Error>::new()
            .required_local("C1", p, Ports::close)
            .required_local("audit", audit_attachment, move |a| {
                if gate.get() {
                    close_audit(a)
                } else {
                    Err(io::Error::other("synthetic audit close Pending"))
                }
            })
            .required_worker("web", web, close_admission, stop_web, observe_web)
            .build();
        let original = root.parts().last().value().endpoint();
        root.fail_required(io::Error::other("primary audit failure"));
        assert_eq!(
            root.complete_until(Instant::now() + IO_BUDGET),
            Completion::Pending
        );
        assert_eq!(root.parts().last().join_status(), JoinStatus::Succeeded);
        assert_eq!(original, address);
        assert!(!root.parts().last().value().has_pending_join());
        assert!(!c.is_stopped());
        assert_eq!(a.status(), AuditStatus::Open);
        assert_eq!(
            root.failures().primary().unwrap().to_string(),
            "primary audit failure"
        );
        allow.set(true);
        assert_eq!(
            root.complete_until(Instant::now() + IO_BUDGET),
            Completion::Complete
        );
        assert!(c.is_stopped());
    }
}
