//! YT-282-E1: one public Composition, one local authority, three required hosts.
//! SOURCE ONLY. Run with --smoke; briefly opens one Window and exits.
//! The fixed triangle does not visually encode domain state.

use std::{
    cell::Cell,
    future::Future,
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpListener, TcpStream, UdpSocket},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle, ThreadId},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, RawQuery, State},
    http::StatusCode,
    routing::{get, post},
};
use yetlet::composition::{
    Attached, Builder, Completion, Composition, Empty, JoinStatus, Local, Phase, Worker,
    WorkerCompletion,
};
use yetlet::native_winit::winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use yetlet::web::HttpWorker;
use yetlet::{
    command_bridge::{AdmissionError, CommandBridge, CommandIngress, SnapshotReader},
    native_winit::{WindowPhase, WindowSignal, WindowSlot},
    renderer_wgpu::{
        DrawReceipt, FatalError, RenderCounters, RenderError, SurfacePhase, SurfaceRenderer, wgpu,
    },
    udp::{SendOutcome, UdpEndpoint},
};

const QUEUE: usize = 4;
const HISTORY: usize = 8;
const IO_WAIT: Duration = Duration::from_millis(500);
const STOP_WAIT: Duration = Duration::from_secs(3);
const SMOKE_WAIT: Duration = Duration::from_secs(12);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Snapshot {
    generation: u64,
    revision: u64,
    paused: bool,
}
impl Snapshot {
    fn text(self) -> String {
        format!("{} {} {}", self.generation, self.revision, self.paused)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Control {
    Pause,
    Resume,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Command {
    generation: u64,
    expected_revision: u64,
    control: Control,
}

// Consumer-local format, deliberately unrelated to the private game protocol:
// exactly 17 bytes: tag (1 Pause, 2 Resume, 3 read), two big-endian u64s.
fn packet(tag: u8, generation: u64, revision: u64) -> [u8; 17] {
    let mut bytes = [0; 17];
    bytes[0] = tag;
    bytes[1..9].copy_from_slice(&generation.to_be_bytes());
    bytes[9..17].copy_from_slice(&revision.to_be_bytes());
    bytes
}
fn decode(bytes: &[u8]) -> io::Result<(u8, u64, u64)> {
    if bytes.len() != 17 || !(1..=3).contains(&bytes[0]) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid packet"));
    }
    Ok((
        bytes[0],
        u64::from_be_bytes(bytes[1..9].try_into().unwrap()),
        u64::from_be_bytes(bytes[9..17].try_into().unwrap()),
    ))
}
fn command(bytes: &[u8]) -> io::Result<Command> {
    let (tag, generation, expected_revision) = decode(bytes)?;
    let control = match tag {
        1 => Control::Pause,
        2 => Control::Resume,
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "read is not a command",
            ));
        }
    };
    Ok(Command {
        generation,
        expected_revision,
        control,
    })
}

struct C1Domain {
    state: Cell<Snapshot>,
    bridge: CommandBridge<Command, Snapshot>,
    stopped: Cell<bool>,
    // Startup admission is shared with the Web worker, never authority state.
    startup_ready: Arc<AtomicBool>,
    creator: ThreadId,
}
impl C1Domain {
    fn new(generation: u64) -> io::Result<Self> {
        let state = Snapshot {
            generation,
            revision: 0,
            paused: false,
        };
        let bridge = CommandBridge::new(QUEUE, HISTORY)?;
        bridge.publish_snapshot(generation, 0, state)?;
        Ok(Self {
            state: Cell::new(state),
            bridge,
            stopped: Cell::new(false),
            startup_ready: Arc::new(AtomicBool::new(false)),
            creator: thread::current().id(),
        })
    }
    fn apply(&self, command: Command) -> io::Result<Snapshot> {
        let old = self.state.get();
        if self.creator != thread::current().id()
            || self.stopped.get()
            || self.bridge.ingress().is_closed()?
        {
            return Err(io::Error::other("C1 is unavailable for application"));
        }
        // Even trusted UDP must not mutate shared state before all required
        // Hosts are actually Ready; HTTP additionally rejects at admission.
        if !self.startup_ready.load(Ordering::Acquire) {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "required hosts are not ready",
            ));
        }
        if command.generation != old.generation || command.expected_revision != old.revision {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stale authority identity",
            ));
        }
        let next = Snapshot {
            generation: old.generation,
            revision: old
                .revision
                .checked_add(1)
                .ok_or_else(|| io::Error::other("revision exhausted"))?,
            paused: command.control == Control::Pause,
        };
        // Publish from the sole domain-owned bridge before committing its Cell.
        self.bridge
            .publish_snapshot(next.generation, next.revision, next)?;
        self.state.set(next);
        eprintln!(
            "E1_APPLIED thread={:?} snapshot={}",
            thread::current().id(),
            next.text()
        );
        Ok(next)
    }
    fn drain(&self) -> io::Result<()> {
        for _ in 0..QUEUE {
            let Some(command) = self
                .bridge
                .commands()
                .try_recv()
                .map_err(io::Error::other)?
            else {
                break;
            };
            match self.apply(command) {
                Ok(_) => {}
                Err(error) if error.kind() == io::ErrorKind::InvalidData => {
                    eprintln!("E1_COMMAND_REJECTED {error}")
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
    fn pinned(&self, revision: u64) -> io::Result<Snapshot> {
        self.bridge
            .snapshots()
            .at_revision(revision)?
            .map(|p| p.snapshot)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "revision not retained"))
    }
}
fn stop_domain(domain: &mut Rc<C1Domain>) -> io::Result<()> {
    if !domain.bridge.ingress().is_closed()? {
        return Err(io::Error::other("HTTP admission must close before C1"));
    }
    domain.startup_ready.store(false, Ordering::Release);
    let discarded = domain.bridge.commands().discard_after_close()?;
    domain.stopped.set(true);
    eprintln!(
        "E1_C1_STOPPED discarded={discarded} snapshot={}",
        domain.state.get().text()
    );
    Ok(())
}

#[derive(Clone)]
struct WebPorts {
    ingress: CommandIngress<Command>,
    reader: SnapshotReader<Snapshot>,
    startup_ready: Arc<AtomicBool>,
}
async fn admit(State(ports): State<WebPorts>, body: Bytes) -> StatusCode {
    let Ok(command) = command(&body) else {
        return StatusCode::BAD_REQUEST;
    };
    if !ports.startup_ready.load(Ordering::Acquire) {
        return StatusCode::SERVICE_UNAVAILABLE;
    }
    match ports.ingress.try_submit(command) {
        Ok(()) => StatusCode::ACCEPTED,
        Err(AdmissionError::Full) => StatusCode::TOO_MANY_REQUESTS,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}
async fn snapshot(
    State(ports): State<WebPorts>,
    RawQuery(query): RawQuery,
) -> Result<String, StatusCode> {
    let published = match query {
        None => ports.reader.latest(),
        Some(query) => {
            let revision = query
                .strip_prefix("revision=")
                .filter(|v| !v.is_empty() && v.bytes().all(|b| b.is_ascii_digit()))
                .and_then(|v| v.parse::<u64>().ok())
                .ok_or(StatusCode::BAD_REQUEST)?;
            ports.reader.at_revision(revision)
        }
    }
    .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    published
        .map(|s| s.snapshot.text())
        .ok_or(StatusCode::NOT_FOUND)
}
fn router(domain: &C1Domain) -> Router {
    Router::new()
        .route("/ready", get(|| async { "ready" }))
        .route("/snapshot", get(snapshot))
        .route("/command", post(admit))
        .layer(DefaultBodyLimit::max(17))
        .with_state(WebPorts {
            ingress: domain.bridge.ingress(),
            reader: domain.bridge.snapshots(),
            startup_ready: Arc::clone(&domain.startup_ready),
        })
}

struct Network {
    endpoint: UdpEndpoint,
    // An OS clone of the finite client's bound socket reserves its address
    // even after that client returns. Release only after endpoint.close().
    reservation: Option<UdpSocket>,
    trusted_peer: SocketAddr,
    rejected: usize,
}
impl Network {
    fn reply(&self, domain: &C1Domain, bytes: &[u8], sender: SocketAddr) -> io::Result<Snapshot> {
        if !sender.ip().is_loopback() || sender != self.trusted_peer {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "untrusted loopback peer",
            ));
        }
        let (tag, generation, revision) = decode(bytes)?;
        if tag == 3 {
            let pinned = domain.pinned(revision)?;
            if generation != pinned.generation {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "wrong generation",
                ));
            }
            Ok(pinned)
        } else {
            domain.apply(command(bytes)?)
        }
    }
    fn poll(&mut self, domain: &C1Domain) -> io::Result<()> {
        // Collect at most eight small owned packets; no borrow of the endpoint
        // escapes the callback and no other task can apply domain commands.
        let mut packets = Vec::with_capacity(8);
        self.endpoint.poll_bounded(8, |bytes, sender| {
            packets.push((bytes.to_vec(), sender));
            Ok(())
        })?;
        for (bytes, sender) in packets {
            match self.reply(domain, &bytes, sender) {
                Ok(state) => match self.endpoint.send_to(sender, state.text().as_bytes())? {
                    SendOutcome::Sent(_) => {
                        eprintln!("E1_UDP_ACK peer={sender} snapshot={}", state.text())
                    }
                    SendOutcome::WouldBlock => eprintln!("E1_UDP_ACK_WOULD_BLOCK peer={sender}"),
                },
                Err(error)
                    if matches!(
                        error.kind(),
                        io::ErrorKind::PermissionDenied
                            | io::ErrorKind::InvalidData
                            | io::ErrorKind::NotFound
                    ) =>
                {
                    self.rejected += 1;
                    eprintln!("E1_UDP_REJECTED peer={sender} cause={error}");
                }
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }
}
fn close_udp(network: &mut Network) -> io::Result<()> {
    network.endpoint.close()?;
    drop(network.reservation.take());
    eprintln!(
        "E1_UDP_CLOSED actual_socket_closed={} peer_reservation_released={}",
        network.endpoint.is_closed(),
        network.reservation.is_none()
    );
    Ok(())
}

fn gpu_error(renderer: &SurfaceRenderer<'_>, error: RenderError) -> io::Error {
    if let Some(cause) = renderer.first_device_error() {
        return io::Error::other(Arc::clone(cause));
    }
    if let Some(cause) = renderer.first_fatal_error() {
        return io::Error::other(Arc::clone(cause));
    }
    if let Some(cause) = renderer.first_device_loss() {
        return io::Error::other(Arc::clone(cause));
    }
    io::Error::other(error)
}
struct Native {
    // Ordered even on ordinary Drop. No implicit event loop or Host thread.
    renderer: Option<SurfaceRenderer<'static>>,
    cleanup_error: Option<Arc<FatalError>>,
    window: WindowSlot,
    counters: RenderCounters,
    observed: Option<Snapshot>,
}
impl Native {
    fn new() -> Self {
        Self {
            renderer: None,
            cleanup_error: None,
            window: WindowSlot::new(
                Window::default_attributes()
                    .with_title("Yetlet E1 public three-way composition")
                    .with_inner_size(LogicalSize::new(640.0, 420.0)),
            ),
            counters: RenderCounters::default(),
            observed: None,
        }
    }
    fn prepare(&mut self, active: &ActiveEventLoop) -> io::Result<()> {
        if self.renderer.is_some() {
            return Ok(());
        }
        let id = self.window.resumed(active)?;
        let size = self
            .window
            .last_size()
            .ok_or_else(|| io::Error::other("missing Window size"))?;
        eprintln!(
            "E1_WINDOW_CREATED pid={} id={id:?} size={size:?} window_ready={} gpu_present=0",
            std::process::id(),
            self.window.is_ready()
        );
        // Retain the renderer before any fallible configure operation, so the
        // composition callback owns all cleanup after a preparation failure.
        self.renderer = Some(initialize(SurfaceRenderer::new(
            self.window
                .shared_window()
                .ok_or_else(|| io::Error::other("missing safe Window target"))?,
            size.width,
            size.height,
        ))??);
        let renderer = self.renderer.as_mut().unwrap();
        let info = renderer.adapter_info();
        eprintln!(
            "E1_GPU adapter={:?} backend={:?} driver={:?} device_type={:?}",
            info.name, info.backend, info.driver_info, info.device_type
        );
        if info.backend != wgpu::Backend::Vulkan
            || info.device_type != wgpu::DeviceType::DiscreteGpu
        {
            return Err(io::Error::other(
                "this bounded smoke requires hardware Vulkan",
            ));
        }
        renderer
            .resize(size.width, size.height)
            .map_err(|e| gpu_error(renderer, e))?;
        Ok(())
    }
    fn event(&mut self, id: WindowId, event: &WindowEvent, state: Snapshot) -> io::Result<bool> {
        match self.window.observe_window_event(id, event) {
            WindowSignal::CloseRequested | WindowSignal::Destroyed => {
                return Err(io::Error::other("Window closed before smoke completion"));
            }
            WindowSignal::Resized(size) => {
                if let Some(renderer) = &mut self.renderer {
                    renderer
                        .resize(size.width, size.height)
                        .map_err(|e| gpu_error(renderer, e))?;
                }
            }
            WindowSignal::RedrawReady => {
                let renderer = self
                    .renderer
                    .as_mut()
                    .ok_or_else(|| io::Error::other("redraw before GPU preparation"))?;
                eprintln!(
                    "E1_WINDOW_REDRAW id={id:?} visible={} accepted={} presents_before={}",
                    self.window.is_ready(),
                    self.window.redraw_event_count(),
                    renderer.counters().present_calls
                );
                if renderer.phase() == SurfacePhase::Unconfigured {
                    let size = self.window.window().unwrap().inner_size();
                    renderer
                        .resize(size.width, size.height)
                        .map_err(|e| gpu_error(renderer, e))?;
                }
                if let DrawReceipt::Presented { present_calls, .. } =
                    renderer.draw().map_err(|e| gpu_error(renderer, e))?
                {
                    self.observed = Some(state); // immutable observation, no domain mutation
                    eprintln!(
                        "E1_NATIVE_PRESENT snapshot={} present_calls={present_calls} counters={:?} triangle_projects_state=false scanout_verified=false",
                        state.text(),
                        renderer.counters()
                    );
                    return Ok(self.window.is_ready());
                }
            }
            _ => {}
        }
        Ok(false)
    }
}
fn close_native(native: &mut Native) -> io::Result<()> {
    if let Some(mut renderer) = native.renderer.take() {
        let result = renderer.close(); // actual Surface/owned target released first
        native.counters = renderer.counters();
        native.cleanup_error = renderer.frame_cleanup_error().cloned();
        eprintln!(
            "E1_SURFACE_CLOSED result={result:?} counters={:?}",
            native.counters
        );
        // Preserve unexpected close errors too; the known discard error has its
        // retained original Arc and must never become successful completion.
        if let Err(error) = result
            && native.cleanup_error.is_none()
        {
            native.renderer = Some(renderer);
            return Err(gpu_error(native.renderer.as_ref().unwrap(), error));
        }
    }
    native.window.close()?;
    eprintln!(
        "E1_WINDOW_CLOSED phase={:?} surface_released={}",
        native.window.phase(),
        native.renderer.is_none()
    );
    if let Some(error) = &native.cleanup_error {
        return Err(io::Error::other(Arc::clone(error)));
    }
    Ok(())
}

// Caller-thread polling only. Cooperative deadline cannot preempt driver FFI.
fn initialize<F: Future>(future: F) -> io::Result<F::Output> {
    struct WakeCaller(thread::Thread);
    impl Wake for WakeCaller {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let waker = Waker::from(Arc::new(WakeCaller(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "GPU initialization deadline",
            ));
        }
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return Ok(value),
            Poll::Pending => thread::park_timeout(Duration::from_millis(5)),
        }
    }
}

struct Web {
    worker: HttpWorker,
    ingress: CommandIngress<Command>,
}
fn close_http(web: &mut Web) -> io::Result<()> {
    web.ingress.close()?;
    eprintln!(
        "E1_HTTP_ADMISSION_CLOSED closed={}",
        web.ingress.is_closed()?
    );
    Ok(())
}
fn signal_http(web: &mut Web) -> io::Result<()> {
    web.worker.request_stop()
}
fn observe_http(web: &mut Web, budget: Duration) -> io::Result<WorkerCompletion<io::Error>> {
    let receipt = web.worker.observe_within(budget)?;
    eprintln!(
        "E1_HTTP_COMPLETION receipt={receipt:?} original_handle_retained={}",
        web.worker.has_pending_join()
    );
    Ok(receipt)
}
type DomainClose = fn(&mut Rc<C1Domain>) -> io::Result<()>;
type UdpClose = fn(&mut Network) -> io::Result<()>;
type NativeClose = fn(&mut Native) -> io::Result<()>;
type WebFence = fn(&mut Web) -> io::Result<()>;
type WebSignal = fn(&mut Web) -> io::Result<()>;
type WebObserve = fn(&mut Web, Duration) -> io::Result<WorkerCompletion<io::Error>>;
type DomainParts = Attached<Empty, Local<Rc<C1Domain>, DomainClose>>;
type NetworkParts = Attached<DomainParts, Local<Network, UdpClose>>;
type BaseParts = Attached<NetworkParts, Local<Native, NativeClose>>;
type BaseRoot = Composition<io::Error, BaseParts>;
type Parts = Attached<BaseParts, Worker<Web, WebFence, WebSignal, WebObserve>>;
type Root = Composition<io::Error, Parts>;

fn base(domain: Rc<C1Domain>, network: Network) -> Builder<io::Error, BaseParts> {
    Builder::new()
        .required_local("sole C1", domain, stop_domain as DomainClose)
        .required_local("UDP", network, close_udp as UdpClose)
        .required_foreground(
            "Native Window + selected Surface renderer",
            Native::new(),
            close_native as NativeClose,
        )
}
fn start_http(
    builder: Builder<io::Error, BaseParts>,
    routes: Router,
    ingress: CommandIngress<Command>,
    bind: SocketAddr,
) -> Result<Root, Box<BaseRoot>> {
    let mut fence_error = None;
    let result = builder.try_required_worker(
        "HTTP",
        || {
            match HttpWorker::start_on(routes, bind) {
                Ok(worker) => Ok(Web { worker, ingress }),
                Err(error) => {
                    // The factory must close issued admission before the core's
                    // immediate rollback. Preserve the bind error as first cause.
                    fence_error = ingress.close().err();
                    Err(error)
                }
            }
        },
        close_http as WebFence,
        signal_http as WebSignal,
        observe_http as WebObserve,
    );
    result.map(|builder| builder.build()).map_err(|failure| {
        let mut root = failure.into_root();
        if let Some(error) = fence_error {
            root.fail_required(error);
        }
        Box::new(root)
    })
}
fn domain(root: &Root) -> &Rc<C1Domain> {
    root.parts().previous().previous().previous().last().value()
}

// Only the caller-thread owner can open application command admission after
// the public Composition has observed every required Host ready. HTTP /ready
// remains independently probeable during Native Window/GPU initialization.
fn enable_startup_commands_if_ready(root: &mut Root) -> bool {
    let ready = root.all_required_ready();
    if ready {
        domain(root).startup_ready.store(true, Ordering::Release);
    }
    ready
}

fn terminal_cleanup_blocker(root: &Root) -> bool {
    let native = root.parts().previous().last().value();
    let web = root.parts().last();
    native.cleanup_error.is_some()
        && native.renderer.is_none()
        && native.window.phase() == WindowPhase::Closed
        && web.join_status() != JoinStatus::Pending
        && !web.value().worker.has_pending_join()
}

// Reuse the same root and original Web handle. A permanent retained cleanup
// error cannot improve after Web joins, so later calls do no further work.
fn drive_completion(root: &mut Root, deadline: Instant) -> Completion {
    loop {
        if terminal_cleanup_blocker(root) {
            return Completion::Pending;
        }
        let completion = root.complete_until(deadline);
        if completion == Completion::Complete
            || terminal_cleanup_blocker(root)
            || Instant::now() >= deadline
        {
            return completion;
        }
        thread::sleep(
            Duration::from_millis(5).min(deadline.saturating_duration_since(Instant::now())),
        );
    }
}

// Finite client-only I/O, never an application authority or a second fixture.
fn http(endpoint: SocketAddr, path: &str, body: Option<&[u8]>) -> io::Result<(u16, String)> {
    let mut socket = TcpStream::connect_timeout(&endpoint, IO_WAIT)?;
    socket.set_read_timeout(Some(IO_WAIT))?;
    socket.set_write_timeout(Some(IO_WAIT))?;
    let method = if body.is_some() { "POST" } else { "GET" };
    let body = body.unwrap_or_default();
    write!(
        socket,
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    socket.write_all(body)?;
    let mut response = String::new();
    socket.take(4096).read_to_string(&mut response)?;
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| io::Error::other("malformed HTTP response"))?;
    let status = head
        .split_whitespace()
        .nth(1)
        .and_then(|v| v.parse().ok())
        .ok_or_else(|| io::Error::other("missing HTTP status"))?;
    Ok((status, body.to_owned()))
}
fn expect(condition: bool, message: &str) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message))
    }
}
fn client(
    http_addr: SocketAddr,
    udp_addr: SocketAddr,
    peer: UdpSocket,
    generation: u64,
) -> io::Result<Snapshot> {
    eprintln!(
        "E1_CLIENT_ONLY pid={} thread={:?} peer={} owns_c1=false",
        std::process::id(),
        thread::current().id(),
        peer.local_addr()?
    );
    let initial = Snapshot {
        generation,
        revision: 0,
        paused: false,
    };
    expect(
        http(http_addr, "/snapshot", None)? == (200, initial.text()),
        "initial HTTP snapshot mismatch",
    )?;
    let admission_deadline = Instant::now() + SMOKE_WAIT;
    loop {
        match http(http_addr, "/command", Some(&packet(1, generation, 0)))?.0 {
            202 => break,
            503 if Instant::now() < admission_deadline => {
                thread::sleep(Duration::from_millis(10));
            }
            status => {
                return Err(io::Error::other(format!(
                    "Pause admission not ready or rejected: {status}"
                )));
            }
        }
    }
    eprintln!("E1_HTTP_202 action=Pause all_required_ready=true execution_claim=false");
    let paused = Snapshot {
        generation,
        revision: 1,
        paused: true,
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let result = http(http_addr, "/snapshot?revision=1", None)?;
        if result == (200, paused.text()) {
            break;
        }
        expect(Instant::now() < deadline, "Pause publication deadline")?;
        thread::sleep(Duration::from_millis(10));
    }
    peer.set_read_timeout(Some(IO_WAIT))?;
    peer.set_write_timeout(Some(IO_WAIT))?;
    let exchange = |bytes: &[u8]| -> io::Result<String> {
        peer.send_to(bytes, udp_addr)?;
        let mut reply = [0; 128];
        let (count, sender) = peer.recv_from(&mut reply)?;
        expect(sender == udp_addr, "wrong UDP ack sender")?;
        String::from_utf8(reply[..count].to_vec()).map_err(io::Error::other)
    };
    expect(
        exchange(&packet(3, generation, 1))? == paused.text(),
        "HTTP/UDP pinned Pause mismatch",
    )?;
    let resumed = Snapshot {
        generation,
        revision: 2,
        paused: false,
    };
    expect(
        exchange(&packet(2, generation, 1))? == resumed.text(),
        "UDP Resume ack mismatch",
    )?;
    expect(
        http(http_addr, "/snapshot?revision=2", None)? == (200, resumed.text()),
        "HTTP/UDP pinned Resume mismatch",
    )?;
    expect(
        exchange(&packet(3, generation, 2))? == resumed.text(),
        "UDP exact revision readback mismatch",
    )?;
    eprintln!(
        "E1_CLIENT_MATCH http={} udp={} exact_snapshot={} client_handle_closes_on_return=true trusted_address_reserved_by_network=true",
        http_addr,
        udp_addr,
        resumed.text()
    );
    Ok(resumed)
}

struct App {
    root: Root,
    client: Option<JoinHandle<io::Result<Snapshot>>>,
    pinned: Option<Snapshot>,
    started: Instant,
    proof: bool,
    next_wake: Instant,
}
impl App {
    fn fail(&mut self, active: &ActiveEventLoop, error: io::Error) {
        eprintln!("E1_REQUIRED_FAILURE {error}");
        domain(&self.root)
            .startup_ready
            .store(false, Ordering::Release);
        self.root.fail_required(error); // preserve cause BEFORE exit/stop
        active.exit();
    }
    fn step(&mut self) -> io::Result<()> {
        if self.root.phase() == Phase::Stopping {
            return Ok(());
        }
        domain(&self.root).drain()?;
        // A temporary caller-local Rc alias resolves the disjoint typed
        // navigation borrows; it refers to the SAME C1 and is dropped this wake.
        let owner = Rc::clone(domain(&self.root));
        if let Some(network) = self
            .root
            .parts_mut()
            .previous()
            .previous()
            .last()
            .value_mut()
        {
            network.poll(&owner)?;
        }
        drop(owner);
        if self.client.as_ref().is_some_and(JoinHandle::is_finished) {
            let handle = self.client.take().unwrap();
            self.pinned = Some(
                handle
                    .join()
                    .map_err(|_| io::Error::other("client thread panicked"))??,
            );
            eprintln!("E1_CLIENT_JOINED exact_pin={}", self.pinned.unwrap().text());
        }
        Ok(())
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, active: &ActiveEventLoop) {
        let Some(native) = self.root.parts_mut().previous().last().value_mut() else {
            return;
        };
        if let Err(error) = native.prepare(active) {
            self.fail(active, error);
        }
    }
    fn window_event(&mut self, active: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let state = match self.pinned {
            Some(pin) => match domain(&self.root).pinned(pin.revision) {
                Ok(state) if state == pin => state,
                _ => {
                    self.fail(active, io::Error::other("pinned native snapshot mismatch"));
                    return;
                }
            },
            None => domain(&self.root).state.get(),
        };
        let Some(native) = self.root.parts_mut().previous().last().value_mut() else {
            return;
        };
        match native.event(id, &event, state) {
            Ok(true) => {
                self.root.parts_mut().previous().last().mark_ready();
                if enable_startup_commands_if_ready(&mut self.root) && self.pinned == Some(state) {
                    self.proof = true;
                    eprintln!(
                        "E1_THREE_WAY_MATCH pid={} snapshot={} native_present_returned=true all_required_ready=true",
                        std::process::id(),
                        state.text()
                    );
                }
            }
            Ok(false) => {}
            Err(error) => self.fail(active, error),
        }
    }
    fn about_to_wait(&mut self, active: &ActiveEventLoop) {
        if let Err(error) = self.step() {
            self.fail(active, error);
            return;
        }
        let now = Instant::now();
        if self.proof && now.duration_since(self.started) >= Duration::from_secs(2) {
            active.exit();
            return;
        }
        if now.duration_since(self.started) >= SMOKE_WAIT {
            self.fail(
                active,
                io::Error::new(io::ErrorKind::TimedOut, "three-way smoke deadline"),
            );
            return;
        }
        if now >= self.next_wake {
            if let Some(native) = self.root.parts_mut().previous().last().value_mut()
                && let Err(error) = native.window.request_redraw()
            {
                self.fail(active, error);
                return;
            }
            self.next_wake = now + Duration::from_millis(40);
        }
        active.set_control_flow(ControlFlow::WaitUntil(self.next_wake));
    }
    fn suspended(&mut self, active: &ActiveEventLoop) {
        self.fail(
            active,
            io::Error::other("suspended before bounded smoke completed"),
        );
    }
    fn exiting(&mut self, _: &ActiveEventLoop) {
        if !self.proof {
            self.root.fail_required(io::Error::other(
                "event loop exited without exact three-way proof",
            ));
        }
        domain(&self.root)
            .startup_ready
            .store(false, Ordering::Release);
        self.root.request_stop();
    }
}

fn finish_start_failure(mut root: BaseRoot) -> io::Error {
    let until = Instant::now() + STOP_WAIT;
    let mut completion = root.complete_until(Instant::now());
    while completion == Completion::Pending && Instant::now() < until {
        thread::sleep(Duration::from_millis(5));
        completion = root.complete_until(Instant::now());
    }
    let cause = root
        .failures()
        .primary()
        .map(ToString::to_string)
        .unwrap_or_else(|| "startup failed".into());
    eprintln!(
        "E1_STARTUP_ROLLBACK completion={completion:?} primary={cause} secondary={:?}",
        root.failures().secondary()
    );
    if completion == Completion::Pending {
        // Explicitly retain storage until process exit. No dropped JoinHandle,
        // detach, successful cleanup or hard cancellation is inferred.
        let _ = Box::leak(Box::new(root));
    }
    io::Error::other(cause)
}
// Every fallible UDP acquisition finishes before C1 and its command ingress
// exist. On bind/address/descriptor failure only OS socket owners need Drop.
fn prepare_network_then_domain(
    peer: &UdpSocket,
    trusted_peer: SocketAddr,
    bind: impl FnOnce() -> io::Result<UdpEndpoint>,
    make_domain: impl FnOnce() -> io::Result<Rc<C1Domain>>,
) -> io::Result<(Network, SocketAddr, Rc<C1Domain>)> {
    let endpoint = bind()?;
    let udp_addr = endpoint.local_addr()?;
    let reservation = peer.try_clone()?;
    let network = Network {
        endpoint,
        reservation: Some(reservation),
        trusted_peer,
        rejected: 0,
    };
    let owner = make_domain()?;
    Ok((network, udp_addr, owner))
}

fn main() -> io::Result<()> {
    if std::env::args().skip(1).collect::<Vec<_>>() != ["--smoke"] {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: composition_three_way --smoke",
        ));
    }
    let event_loop = EventLoop::new().map_err(io::Error::other)?;
    let peer = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
    let peer_addr = peer.local_addr()?;
    let generation = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos() as u64;
    let (network, udp_addr, owner) = prepare_network_then_domain(
        &peer,
        peer_addr,
        || UdpEndpoint::bind_loopback(128),
        || C1Domain::new(generation).map(Rc::new),
    )?;
    let routes = router(&owner);
    let ingress = owner.bridge.ingress();
    let root = start_http(
        base(owner, network),
        routes,
        ingress,
        SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
    )
    .map_err(|root| finish_start_failure(*root))?;
    let mut app = App {
        root,
        client: None,
        pinned: None,
        started: Instant::now(),
        proof: false,
        next_wake: Instant::now(),
    };
    let http_addr = app.root.parts().last().value().worker.endpoint();
    eprintln!(
        "E1_ENDPOINTS pid={} foreground={:?} http={http_addr} udp={udp_addr} trusted_peer={peer_addr}",
        std::process::id(),
        thread::current().id()
    );
    // Readiness failure still flows through the exact root and original worker.
    match app
        .root
        .parts()
        .last()
        .value()
        .worker
        .wait_http_ready("/ready", Duration::from_secs(2))
    {
        Ok(()) => {
            app.root.parts_mut().last().mark_ready();
            eprintln!("E1_HTTP_READY actual_get_ready_200=true worker_marked_ready=true");
            match thread::Builder::new()
                .name("e1-finite-client-only".into())
                .spawn(move || client(http_addr, udp_addr, peer, generation))
            {
                Ok(handle) => {
                    app.client = Some(handle);
                    if let Err(error) = event_loop.run_app(&mut app) {
                        app.root.fail_required(io::Error::other(error));
                    }
                }
                Err(error) => app.root.fail_required(error),
            }
        }
        Err(error) => app.root.fail_required(error),
    }
    // run_app may return without the exiting callback on an error. Never let
    // that bypass required-failure recording, admission closure or completion.
    if !app.proof && app.root.failures().primary().is_none() {
        app.root
            .fail_required(io::Error::other("run_app returned without proof"));
    }
    let completion = drive_completion(&mut app.root, Instant::now() + STOP_WAIT);
    if let Some(client) = app.client.take() {
        // Client is finite: every socket operation and polling loop is bounded.
        // Always join it, including all negative foreground paths.
        match client.join() {
            Ok(Ok(_)) => eprintln!("E1_CLIENT_JOINED after_stop=true"),
            Ok(Err(error)) => app.root.fail_required(error),
            Err(_) => app
                .root
                .fail_required(io::Error::other("client thread panicked")),
        }
    }
    let native = app.root.parts().previous().last().value();
    let network = app.root.parts().previous().previous().last().value();
    let web = app.root.parts().last();
    eprintln!(
        "E1_FINAL pid={} completion={completion:?} phase={:?} proof={} pin={:?} native_observed={:?} counters={:?} surface_released={} window_closed={} udp_closed={} c1_stopped={} c1_owners={} http_join={:?} pending_handle={} client_joined={} secondary={:?}",
        std::process::id(),
        app.root.phase(),
        app.proof,
        app.pinned,
        native.observed,
        native.counters,
        native.renderer.is_none(),
        native.window.phase() == WindowPhase::Closed,
        network.endpoint.is_closed(),
        domain(&app.root).stopped.get(),
        Rc::strong_count(domain(&app.root)),
        web.join_status(),
        web.value().worker.has_pending_join(),
        app.client.is_none(),
        app.root.failures().secondary()
    );
    let accepted = app.proof
        && app.pinned == native.observed
        && native.counters.present_calls > 0
        && native.counters.present_calls == native.counters.draw_calls
        && native.counters.present_calls == native.counters.submissions
        && native.renderer.is_none()
        && native.window.phase() == WindowPhase::Closed
        && network.endpoint.is_closed()
        && network.reservation.is_none()
        && domain(&app.root).stopped.get()
        && Rc::strong_count(domain(&app.root)) == 1
        && web.join_status() == JoinStatus::Succeeded
        && !web.value().worker.has_pending_join()
        && completion == Completion::Complete
        && app.root.phase() == Phase::Stopped;
    let error = app.root.failures().primary().map(ToString::to_string);
    if completion == Completion::Pending {
        eprintln!(
            "E1_INCOMPLETE original_root_retained_until_process_exit=true no_join_or_cleanup_success_claim=true"
        );
        let _ = Box::leak(Box::new(app.root));
        return Err(io::Error::other(
            error.unwrap_or_else(|| "incomplete cleanup".into()),
        ));
    }
    if let Some(error) = error {
        return Err(io::Error::other(error));
    }
    expect(accepted, "three-way acceptance incomplete")?;
    // Actual endpoint ownership readback after original worker Join/socket close.
    let rebound_tcp = TcpListener::bind(http_addr)?;
    let rebound_udp = UdpSocket::bind(udp_addr)?;
    let rebound_peer = UdpSocket::bind(peer_addr)?;
    drop((rebound_tcp, rebound_udp, rebound_peer));
    eprintln!(
        "E1_TEARDOWN tcp_rebound=true udp_rebound=true trusted_peer_rebound=true active_owned_workers=0 active_owned_sockets=0 active_owned_windows=0"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use tokio::sync::Notify;

    fn fixture(extra: impl FnOnce(Router, CommandIngress<Command>) -> Router) -> (Root, UdpSocket) {
        let owner = Rc::new(C1Domain::new(7).unwrap());
        let routes = extra(router(&owner), owner.bridge.ingress());
        let ingress = owner.bridge.ingress();
        let peer = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let endpoint = UdpEndpoint::bind_loopback(128).unwrap();
        let mut root = start_http(
            base(
                owner,
                Network {
                    endpoint,
                    reservation: Some(peer.try_clone().unwrap()),
                    trusted_peer: peer.local_addr().unwrap(),
                    rejected: 0,
                },
            ),
            routes,
            ingress,
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
        )
        .unwrap_or_else(|root| panic!("{}", finish_start_failure(*root)));
        root.parts()
            .last()
            .value()
            .worker
            .wait_http_ready("/ready", IO_WAIT)
            .unwrap();
        assert!(root.parts_mut().last().mark_ready());
        assert!(
            !root.all_required_ready(),
            "HTTP bind/200 cannot qualify Native"
        );
        (root, peer)
    }
    fn finish(root: &mut Root) {
        let result = drive_completion(root, Instant::now() + STOP_WAIT);
        assert_eq!(result, Completion::Complete);
        assert_eq!(root.parts().last().join_status(), JoinStatus::Succeeded);
        assert!(!root.parts().last().value().worker.has_pending_join());
    }
    fn poll(root: &mut Root) {
        let owner = Rc::clone(domain(root));
        root.parts_mut()
            .previous()
            .previous()
            .last()
            .value_mut()
            .unwrap()
            .poll(&owner)
            .unwrap();
    }

    #[test]
    fn udp_bind_collision_is_rejected_before_domain_creation() {
        let peer = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let occupied = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = occupied.local_addr().unwrap();
        let domain_invoked = std::cell::Cell::new(false);
        let result = prepare_network_then_domain(
            &peer,
            peer.local_addr().unwrap(),
            || UdpEndpoint::bind_on(address, 128),
            || {
                domain_invoked.set(true);
                C1Domain::new(7).map(Rc::new)
            },
        );
        assert!(matches!(result, Err(ref err) if err.kind() == io::ErrorKind::AddrInUse));
        assert!(
            !domain_invoked.get(),
            "UDP bind failure must precede C1 creation"
        );
    }

    #[test]
    fn http_admission_is_not_execution_and_history_is_exact_and_read_only() {
        let (mut root, _peer) = fixture(|r, _| r);
        let address = root.parts().last().value().worker.endpoint();
        assert_eq!(http(address, "/ready", None).unwrap().0, 200);
        assert_eq!(
            http(address, "/command", Some(&packet(1, 7, 0))).unwrap().0,
            503,
            "pre-ready HTTP must reject a valid command"
        );
        assert_eq!(domain(&root).bridge.commands().try_recv().unwrap(), None);
        assert_eq!(domain(&root).state.get().revision, 0);
        // Synthetic foreground Ready in the focused fixture only; the real
        // Winit integration opens this gate after actual Window/GPU Present.
        assert!(root.parts_mut().previous().last().mark_ready());
        assert!(enable_startup_commands_if_ready(&mut root));
        assert_eq!(
            http(address, "/command", Some(&packet(1, 7, 0))).unwrap().0,
            202
        );
        assert_eq!(domain(&root).state.get().revision, 0);
        assert_eq!(http(address, "/snapshot?revision=1", None).unwrap().0, 404);
        domain(&root).drain().unwrap();
        assert_eq!(
            http(address, "/snapshot?revision=1", None).unwrap(),
            (200, "7 1 true".into())
        );
        assert_eq!(
            http(address, "/snapshot?revision=0", None).unwrap(),
            (200, "7 0 false".into())
        );
        assert_eq!(
            http(address, "/snapshot?revision=1&extra=2", None)
                .unwrap()
                .0,
            400
        );
        assert_eq!(
            http(address, "/command", Some(&packet(3, 7, 1))).unwrap().0,
            400
        );
        let state = domain(&root).state.get();
        assert!(domain(&root).bridge.publish_snapshot(7, 1, state).is_err());
        assert!(domain(&root).bridge.publish_snapshot(8, 2, state).is_err());
        assert!(
            domain(&root)
                .apply(Command {
                    generation: 7,
                    expected_revision: 0,
                    control: Control::Resume
                })
                .is_err()
        );
        assert!(
            domain(&root)
                .apply(Command {
                    generation: 8,
                    expected_revision: 1,
                    control: Control::Resume
                })
                .is_err()
        );
        assert_eq!(domain(&root).state.get(), state);
        finish(&mut root);
        assert!(domain(&root).stopped.get());
        assert!(
            domain(&root)
                .apply(Command {
                    generation: 7,
                    expected_revision: 1,
                    control: Control::Resume
                })
                .is_err()
        );
    }

    #[test]
    fn real_udp_rejects_malformed_and_wrong_sender_then_uses_same_c1_as_http() {
        let (mut root, peer) = fixture(|r, _| r);
        let address = root
            .parts()
            .previous()
            .previous()
            .last()
            .value()
            .endpoint
            .local_addr()
            .unwrap();
        let wrong = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        wrong.send_to(&packet(1, 7, 0), address).unwrap();
        peer.send_to(&packet(1, 7, 0)[..16], address).unwrap();
        peer.send_to(&packet(9, 7, 0), address).unwrap();
        let until = Instant::now() + IO_WAIT;
        while root.parts().previous().previous().last().value().rejected < 3
            && Instant::now() < until
        {
            poll(&mut root);
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(
            root.parts().previous().previous().last().value().rejected,
            3
        );
        assert_eq!(domain(&root).state.get().revision, 0);
        // A correctly bound peer is still denied application before Native
        // Ready; the rejected UDP datagram cannot advance the sole C1.
        peer.send_to(&packet(1, 7, 0), address).unwrap();
        let pre_ready_until = Instant::now() + IO_WAIT;
        while root.parts().previous().previous().last().value().rejected < 4
            && Instant::now() < pre_ready_until
        {
            poll(&mut root);
        }
        assert_eq!(
            root.parts().previous().previous().last().value().rejected,
            4
        );
        assert_eq!(domain(&root).state.get().revision, 0);
        assert!(root.parts_mut().previous().last().mark_ready());
        assert!(enable_startup_commands_if_ready(&mut root));
        peer.send_to(&packet(1, 7, 0), address).unwrap();
        let applied_until = Instant::now() + IO_WAIT;
        while domain(&root).state.get().revision == 0 && Instant::now() < applied_until {
            poll(&mut root);
        }
        assert_eq!(
            domain(&root).state.get(),
            Snapshot {
                generation: 7,
                revision: 1,
                paused: true
            }
        );
        peer.set_read_timeout(Some(IO_WAIT)).unwrap();
        let mut ack = [0; 128];
        let (len, source) = peer.recv_from(&mut ack).unwrap();
        assert_eq!(source, address);
        assert_eq!(&ack[..len], b"7 1 true");
        let http_addr = root.parts().last().value().worker.endpoint();
        assert_eq!(
            http(http_addr, "/snapshot?revision=1", None).unwrap(),
            (200, "7 1 true".into())
        );
        assert_eq!(
            http(http_addr, "/command", Some(&packet(2, 7, 1)))
                .unwrap()
                .0,
            202
        );
        domain(&root).drain().unwrap();
        assert_eq!(
            domain(&root).pinned(2).unwrap(),
            Snapshot {
                generation: 7,
                revision: 2,
                paused: false
            }
        );
        finish(&mut root);
        assert!(UdpSocket::bind(address).is_ok());
    }

    #[test]
    fn trusted_udp_address_stays_reserved_after_client_return_until_network_close() {
        for client_error in [false, true] {
            let (mut root, peer) = fixture(|r, _| r);
            let peer_address = peer.local_addr().unwrap();
            let network = root.parts().previous().previous().last().value();
            let address = network.endpoint.local_addr().unwrap();
            assert_eq!(network.trusted_peer, peer_address);
            assert_eq!(
                network.reservation.as_ref().unwrap().local_addr().unwrap(),
                peer_address
            );
            let return_from_client = |peer: UdpSocket| -> io::Result<()> {
                drop(peer);
                if client_error {
                    Err(io::Error::other("synthetic finite client error"))
                } else {
                    Ok(())
                }
            };
            assert_eq!(return_from_client(peer).is_err(), client_error);
            assert_eq!(
                UdpSocket::bind(peer_address).unwrap_err().kind(),
                io::ErrorKind::AddrInUse
            );
            let wrong = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
            wrong.send_to(&packet(1, 7, 0), address).unwrap();
            let until = Instant::now() + IO_WAIT;
            while root.parts().previous().previous().last().value().rejected == 0
                && Instant::now() < until
            {
                poll(&mut root);
                thread::sleep(Duration::from_millis(1));
            }
            let network = root.parts().previous().previous().last().value();
            assert_eq!(network.rejected, 1);
            assert_eq!(domain(&root).state.get().revision, 0);
            // A remaining client handle also reserves the port after Network
            // closes; only the last OS handle's drop permits a new bind.
            let remaining = network.reservation.as_ref().unwrap().try_clone().unwrap();
            finish(&mut root);
            let network = root.parts().previous().previous().last().value();
            assert!(network.endpoint.is_closed());
            assert!(network.reservation.is_none());
            assert!(UdpSocket::bind(address).is_ok());
            assert_eq!(
                UdpSocket::bind(peer_address).unwrap_err().kind(),
                io::ErrorKind::AddrInUse
            );
            drop(remaining);
            assert!(UdpSocket::bind(peer_address).is_ok());
        }
    }

    #[test]
    fn web_bind_conflict_rolls_back_acquired_c1_udp_without_creating_gui() {
        let occupied = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let owner = Rc::new(C1Domain::new(7).unwrap());
        let routes = router(&owner);
        let ingress = owner.bridge.ingress();
        let late = ingress.clone();
        let endpoint = UdpEndpoint::bind_loopback(128).unwrap();
        let address = endpoint.local_addr().unwrap();
        let peer = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let peer_address = peer.local_addr().unwrap();
        let builder = base(
            owner,
            Network {
                endpoint,
                reservation: Some(peer.try_clone().unwrap()),
                trusted_peer: peer_address,
                rejected: 0,
            },
        );
        let mut failed = match start_http(builder, routes, ingress, occupied.local_addr().unwrap())
        {
            Ok(_) => panic!("bind conflict must fail"),
            Err(root) => root,
        };
        assert_eq!(
            failed.failures().primary().unwrap().kind(),
            io::ErrorKind::AddrInUse
        );
        assert!(failed.failures().secondary().is_empty());
        assert!(failed.parts().last().value().window.window_id().is_none());
        assert_eq!(failed.complete_until(Instant::now()), Completion::Complete);
        assert_eq!(failed.phase(), Phase::Failed);
        assert!(
            failed
                .parts()
                .previous()
                .last()
                .value()
                .endpoint
                .is_closed()
        );
        assert!(
            failed
                .parts()
                .previous()
                .previous()
                .last()
                .value()
                .stopped
                .get()
        );
        assert_eq!(
            late.try_submit(command(&packet(1, 7, 0)).unwrap()),
            Err(AdmissionError::Closed)
        );
        assert!(UdpSocket::bind(address).is_ok());
        assert!(
            failed
                .parts()
                .previous()
                .last()
                .value()
                .reservation
                .is_none()
        );
        assert_eq!(
            UdpSocket::bind(peer_address).unwrap_err().kind(),
            io::ErrorKind::AddrInUse
        );
        drop(peer);
        assert!(UdpSocket::bind(peer_address).is_ok());
    }

    #[test]
    fn full_queue_does_not_block_shutdown_and_inflight_late_admission_is_503_before_original_join()
    {
        let release = Arc::new(Notify::new());
        let (entered_tx, entered_rx) = mpsc::sync_channel(1);
        let held_release = release.clone();
        let (mut root, _peer) = fixture(|r, ingress| {
            // A held real request exercises the same ingress as /command.
            r.route(
                "/late",
                get(move || {
                    let release = held_release.clone();
                    let entered = entered_tx.clone();
                    let ingress = ingress.clone();
                    async move {
                        entered.try_send(()).unwrap();
                        release.notified().await;
                        match ingress.try_submit(command(&packet(1, 7, 0)).unwrap()) {
                            Err(AdmissionError::Closed) => StatusCode::SERVICE_UNAVAILABLE,
                            _ => StatusCode::INTERNAL_SERVER_ERROR,
                        }
                    }
                }),
            )
        });
        let address = root.parts().last().value().worker.endpoint();
        let ingress = domain(&root).bridge.ingress();
        for _ in 0..QUEUE {
            ingress
                .try_submit(command(&packet(1, 7, 0)).unwrap())
                .unwrap();
        }
        assert_eq!(
            ingress.try_submit(command(&packet(1, 7, 0)).unwrap()),
            Err(AdmissionError::Full)
        );
        let request = thread::spawn(move || http(address, "/late", None));
        entered_rx.recv_timeout(IO_WAIT).unwrap();
        let original_worker_address = root.parts().last().value() as *const Web;
        assert_eq!(root.complete_until(Instant::now()), Completion::Pending);
        assert_eq!(
            root.parts().last().value() as *const Web,
            original_worker_address
        );
        assert!(root.parts().last().value().worker.has_pending_join());
        assert!(root.parts().last().stop_signalled());
        assert!(domain(&root).stopped.get());
        assert_eq!(domain(&root).state.get().revision, 0);
        assert_eq!(
            ingress.try_submit(command(&packet(1, 7, 0)).unwrap()),
            Err(AdmissionError::Closed)
        );
        release.notify_one();
        let response = request.join().unwrap().unwrap();
        assert_eq!(response.0, 503);
        finish(&mut root);
        assert_eq!(
            root.parts().last().value() as *const Web,
            original_worker_address
        );
        assert!(TcpListener::bind(address).is_ok());
    }

    #[test]
    fn synthetic_gpu_cleanup_failure_retains_c1_pending_and_first_required_cause() {
        let (mut root, peer) = fixture(|r, _| r);
        let peer_address = peer.local_addr().unwrap();
        drop(peer);
        let c1_address = Rc::as_ptr(domain(&root));
        let original_web = root.parts().last().value() as *const Web;
        root.parts_mut()
            .previous()
            .last()
            .value_mut()
            .unwrap()
            .cleanup_error = Some(Arc::new(FatalError {
            operation: "Surface::discard_texture",
            diagnostic: "synthetic retained cleanup failure".into(),
        }));
        root.fail_required(io::Error::other("original required failure"));
        assert_eq!(
            drive_completion(&mut root, Instant::now() + STOP_WAIT),
            Completion::Pending
        );
        assert!(!root.parts().last().value().worker.has_pending_join());
        assert_eq!(root.parts().last().join_status(), JoinStatus::Succeeded);
        let secondary_count = root.failures().secondary().len();
        assert!((1..=2).contains(&secondary_count));
        // Exercise the former busy-loop condition repeatedly with a future
        // deadline, after the original Web Join is already terminal.
        for _ in 0..1024 {
            assert_eq!(
                drive_completion(&mut root, Instant::now() + STOP_WAIT),
                Completion::Pending
            );
            assert_eq!(root.failures().secondary().len(), secondary_count);
            assert_eq!(root.phase(), Phase::Stopping);
            assert_eq!(Rc::as_ptr(domain(&root)), c1_address);
            assert_eq!(root.parts().last().value() as *const Web, original_web);
            assert_eq!(root.parts().last().join_status(), JoinStatus::Succeeded);
            assert!(!root.parts().last().value().worker.has_pending_join());
            assert_eq!(
                root.failures().primary().unwrap().to_string(),
                "original required failure"
            );
            assert!(!domain(&root).stopped.get());
            assert!(
                !root
                    .parts()
                    .previous()
                    .previous()
                    .last()
                    .value()
                    .endpoint
                    .is_closed()
            );
            assert_eq!(
                root.parts().previous().last().value().window.phase(),
                WindowPhase::Closed
            );
            assert!(
                root.failures()
                    .secondary()
                    .iter()
                    .any(|e| e.to_string().contains("synthetic retained cleanup failure"))
            );
        }
        let network = root.parts().previous().previous().last().value();
        assert!(network.reservation.is_some());
        assert_eq!(
            UdpSocket::bind(peer_address).unwrap_err().kind(),
            io::ErrorKind::AddrInUse
        );
        assert_eq!(
            UdpSocket::bind(network.endpoint.local_addr().unwrap())
                .unwrap_err()
                .kind(),
            io::ErrorKind::AddrInUse
        );
        eprintln!(
            "E1_SYNTHETIC_PENDING repeated_driver_calls=1024 secondary_count={secondary_count} original_web_join=Succeeded c1_stopped=false udp_open=true peer_reserved=true"
        );
        // Match main's explicit retention: no false Complete or dropped Pending
        // root. All actual workers joined; owned sockets live until process exit.
        let _ = Box::leak(Box::new(root));
    }

    #[test]
    fn strict_codec_rejects_all_wrong_lengths_and_non_command_read_without_mutation() {
        let owner = C1Domain::new(7).unwrap();
        for len in 0..20 {
            if len != 17 {
                assert!(command(&vec![1; len]).is_err());
            }
        }
        assert!(command(&packet(3, 7, 0)).is_err());
        assert!(command(&packet(0, 7, 0)).is_err());
        assert_eq!(owner.state.get().revision, 0);
        let command = command(&packet(1, 7, 0)).unwrap();
        assert_eq!(
            owner.apply(command).unwrap_err().kind(),
            io::ErrorKind::PermissionDenied
        );
        owner.startup_ready.store(true, Ordering::Release); // synthetic unit readiness
        assert_eq!(owner.apply(command).unwrap().revision, 1);
        assert!(owner.apply(command).is_err());
        assert_eq!(owner.state.get().revision, 1);
        owner.bridge.close_admission().unwrap();
    }
}
