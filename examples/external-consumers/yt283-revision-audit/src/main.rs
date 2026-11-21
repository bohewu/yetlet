//! One external application: explicit RA1 --smoke or opt-in Native --tick.
use std::{
    cell::RefCell,
    error::Error,
    fmt,
    future::Future,
    io,
    net::{SocketAddr, UdpSocket},
    rc::Rc,
    sync::{Arc, mpsc},
    task::{Context, Poll, Wake, Waker},
    thread::{self, JoinHandle},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};
use yetlet::composition::{
    Attached, Builder, Completion, Composition, Empty, JoinStatus, Local, Phase, Worker,
    WorkerCompletion,
};
use yetlet_hosts::{
    native_winit::{WindowSignal, WindowSlot},
    renderer_wgpu::{
        DrawReceipt, FatalError, RectInstance, RenderError, SurfacePhase, SurfaceRenderer,
    },
};
use yt283_revision_audit::{
    ARENA_HEIGHT, ARENA_WIDTH, Action, Direction, Player, Position, SQUARE_SIZE, Snapshot,
    native_clock::{NativeScheduler, PumpReport},
    native_input::{Key, KeyResult, map_key},
    three_way::{
        self, AuditAttachment, IO_BUDGET, Network, Ports, Web, close_admission, close_audit,
        encode_command, encode_read, http, observe_web, stop_web,
    },
};

type Close<T> = fn(&mut T) -> io::Result<()>;
type C1Part = Attached<Empty, Local<Ports, Close<Ports>>>;
type AuditPart = Attached<C1Part, Local<AuditAttachment, Close<AuditAttachment>>>;
type UdpPart = Attached<AuditPart, Local<RefCell<Network>, Close<RefCell<Network>>>>;
fn close_network(n: &mut RefCell<Network>) -> io::Result<()> {
    n.get_mut().close()
}
type NativePart = Attached<UdpPart, Local<Native, Close<Native>>>;
type WebPart = Worker<
    Web,
    Close<Web>,
    Close<Web>,
    fn(&mut Web, Duration) -> io::Result<WorkerCompletion<io::Error>>,
>;
type Root = Composition<io::Error, Attached<NativePart, WebPart>>;

struct Native {
    window: WindowSlot,
    gpu: Option<SurfaceRenderer<'static>>,
    presents: u64,
    presented: Option<Snapshot>,
    cleanup_failure: Option<Arc<GpuFailure>>,
}
fn project_scene(snapshot: Snapshot) -> [RectInstance; 2] {
    let width = ARENA_WIDTH as f32;
    let height = ARENA_HEIGHT as f32;
    let size = SQUARE_SIZE as f32;
    let colors = [[0.24, 0.65, 1.0, 1.0], [1.0, 0.57, 0.26, 1.0]];
    std::array::from_fn(|index| {
        let Position { x, y } = snapshot.positions[index];
        RectInstance {
            center: [
                (2.0 * x as f32 + size) / width - 1.0,
                1.0 - (2.0 * y as f32 + size) / height,
            ],
            half_extent: [size / width, size / height],
            rgba: colors[index],
        }
    })
}

#[derive(Debug)]
struct GpuFailure {
    stage: &'static str,
    phase: SurfacePhase,
    category: RenderError,
    cause: Option<Arc<dyn Error + Send + Sync>>,
    cleanup: Option<Arc<FatalError>>,
}
impl fmt::Display for GpuFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "GPU {} {:?} {:?}", self.stage, self.phase, self.category)?;
        if let Some(cause) = &self.cause {
            write!(f, ": {cause}")?;
        }
        if let Some(cleanup) = &self.cleanup {
            write!(f, "; frame cleanup: {cleanup}")?;
        }
        Ok(())
    }
}
impl Error for GpuFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref().map(|e| e as &(dyn Error + 'static))
    }
}
// All fallible resize/draw/close boundaries clone the public retained causes
// before Composition may release the renderer. No invented typed OOM.
fn gpu_failure(
    gpu: &SurfaceRenderer<'_>,
    stage: &'static str,
    category: RenderError,
) -> GpuFailure {
    let cause = gpu
        .first_device_error()
        .map(|e| Arc::clone(e) as Arc<dyn Error + Send + Sync>)
        .or_else(|| {
            gpu.first_device_loss()
                .map(|e| Arc::clone(e) as Arc<dyn Error + Send + Sync>)
        })
        .or_else(|| {
            gpu.first_fatal_error()
                .map(|e| Arc::clone(e) as Arc<dyn Error + Send + Sync>)
        });
    retain_gpu_failure(
        stage,
        gpu.phase(),
        category,
        cause,
        gpu.frame_cleanup_error().cloned(),
    )
}
fn retain_gpu_failure(
    stage: &'static str,
    phase: SurfacePhase,
    category: RenderError,
    cause: Option<Arc<dyn Error + Send + Sync>>,
    cleanup: Option<Arc<FatalError>>,
) -> GpuFailure {
    // A discard-only failure is the primary cause; do not also duplicate it as
    // a secondary diagnostic. An earlier Device/fatal cause keeps priority.
    let (cause, cleanup) = if cause.is_none() {
        (cleanup.map(|e| e as Arc<dyn Error + Send + Sync>), None)
    } else {
        (cause, cleanup)
    };
    GpuFailure {
        stage,
        phase,
        category,
        cause,
        cleanup,
    }
}
#[derive(Debug)]
struct NativeCloseFailure {
    gpu: Arc<GpuFailure>,
    window: Option<io::Error>,
}
impl fmt::Display for NativeCloseFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.gpu)?;
        if let Some(error) = &self.window {
            write!(f, "; Window close: {error}")?;
        }
        Ok(())
    }
}
impl Error for NativeCloseFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.gpu.as_ref())
    }
}
fn close_window_after_surface(
    window: &mut WindowSlot,
    retained: &mut Option<Arc<GpuFailure>>,
    surface_result: Result<(), GpuFailure>,
) -> io::Result<()> {
    if let Err(error) = surface_result {
        retained.get_or_insert_with(|| Arc::new(error));
    }
    // SurfaceRenderer::close releases its Surface even on FrameCleanupFailed.
    // Always attempt this, including later retries of a Window close blocker.
    let result = window.close();
    if window.phase() == yetlet_hosts::native_winit::WindowPhase::Closed {
        eprintln!("NATIVE_WINDOW_CLOSED");
    }
    match retained {
        Some(gpu) => Err(io::Error::other(NativeCloseFailure {
            gpu: Arc::clone(gpu),
            window: result.err(),
        })),
        None => result,
    }
}
fn reconfigure_redraw(
    phase: SurfacePhase,
    eligible: bool,
    live_size: Option<PhysicalSize<u32>>,
    resize: impl FnOnce(PhysicalSize<u32>) -> io::Result<()>,
) -> io::Result<()> {
    if phase == SurfacePhase::Unconfigured
        && eligible
        && let Some(size) = live_size
    {
        resize(size)?;
    }
    Ok(())
}
impl Native {
    fn record_present(&mut self, snapshot: Snapshot, receipt: DrawReceipt) {
        if let DrawReceipt::Presented { present_calls, .. } = receipt {
            self.presents = present_calls;
            self.presented = Some(snapshot);
        }
    }

    fn presented_final_snapshot(&self, snapshot: Snapshot) -> bool {
        snapshot.revision == 2 && self.presented == Some(snapshot)
    }

    fn suspend(&mut self) -> io::Result<()> {
        let surface = match self.gpu.take() {
            Some(mut gpu) => gpu.close().map_err(|e| gpu_failure(&gpu, "suspend", e)),
            None => Ok(()),
        };
        // A new Surface must return its own Present before Tick activation.
        self.presented = None;
        self.presents = 0;
        if let Err(error) = surface {
            self.cleanup_failure.get_or_insert_with(|| Arc::new(error));
        }
        // Surface owner was dropped before releasing the Window, even on error.
        let window = self.window.suspended();
        match &self.cleanup_failure {
            Some(gpu) => Err(io::Error::other(NativeCloseFailure {
                gpu: Arc::clone(gpu),
                window: window.err(),
            })),
            None => window,
        }
    }

    fn close(&mut self) -> io::Result<()> {
        let result = if let Some(gpu) = self.gpu.as_mut() {
            let result = gpu.close().map_err(|e| gpu_failure(gpu, "close", e));
            eprintln!("GPU_RECTANGLE_COUNTERS {:?}", gpu.counters());
            // Release the renderer after cloning its original diagnostics.
            drop(self.gpu.take());
            eprintln!("GPU_SURFACE_CLOSED");
            result
        } else {
            Ok(())
        };
        close_window_after_surface(&mut self.window, &mut self.cleanup_failure, result)
    }
}
struct ThreadWake(thread::Thread);
impl Wake for ThreadWake {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.unpark();
    }
}
fn initialize<F: Future>(future: F) -> io::Result<F::Output> {
    let deadline = Instant::now() + Duration::from_secs(4);
    let waker = Waker::from(Arc::new(ThreadWake(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return Ok(value),
            Poll::Pending => {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "GPU initialization budget",
                    ));
                }
                thread::park_timeout(Duration::from_millis(5));
            }
        }
    }
}
fn ports(root: &Root) -> &Ports {
    root.parts()
        .previous()
        .previous()
        .previous()
        .previous()
        .last()
        .value()
}
fn native(root: &Root) -> &Native {
    root.parts().previous().last().value()
}
fn native_mut(root: &mut Root) -> Option<&mut Native> {
    root.parts_mut().previous().last().value_mut()
}

/// Finite test client: a single joined thread, never a domain owner.
fn exercise(
    http_addr: SocketAddr,
    udp_addr: SocketAddr,
    client: UdpSocket,
    g: u64,
) -> io::Result<Snapshot> {
    let deadline = Instant::now() + Duration::from_secs(6);
    let (ready, _) = http(http_addr, "GET", "/ready", &[])?;
    if ready != 200 {
        return Err(io::Error::other("client HTTP not ready"));
    }
    let one = Action::Move {
        player: Player::One,
        direction: Direction::East,
    };
    let two = Action::Move {
        player: Player::Two,
        direction: Direction::North,
    };
    let (admitted, _) = http(http_addr, "POST", "/command", &encode_command(one, g, 0))?;
    if admitted != 202 {
        return Err(io::Error::other(format!("HTTP admission {admitted}")));
    }
    eprintln!("CLIENT_HTTP ready=200 admission=202 execution=unconfirmed");
    let first = Snapshot {
        generation: g,
        revision: 1,
        paused: false,
        positions: [Position { x: 3, y: 2 }, Position { x: 20, y: 12 }],
    };
    let first_text = first.text();
    loop {
        let (status, body) = http(http_addr, "GET", &format!("/snapshot/{g}/1"), &[])?;
        if status == 200 && body == first_text {
            break;
        }
        if status != 404 {
            return Err(io::Error::other("HTTP applied snapshot mismatch"));
        }
        if Instant::now() >= deadline {
            return Err(io::Error::other("HTTP applied snapshot timeout"));
        }
        thread::sleep(Duration::from_millis(10));
    }
    client.set_read_timeout(Some(IO_BUDGET))?;
    // The UDP read runs on the caller thread and answers only after C1 commit.
    verify_udp(&client, udp_addr, &encode_read(g, 1), first)?;
    eprintln!("CLIENT_APPLIED http=200 udp=committed snapshot={first_text}");
    let pinned = Snapshot {
        generation: g,
        revision: 2,
        paused: false,
        positions: [Position { x: 3, y: 2 }, Position { x: 20, y: 11 }],
    };
    verify_udp(&client, udp_addr, &encode_command(two, g, 1), pinned)?;
    verify_udp(&client, udp_addr, &encode_read(g, 2), pinned)?;
    if http(http_addr, "GET", &format!("/snapshot/{g}/2"), &[])? != (200, pinned.text()) {
        return Err(io::Error::other("HTTP pinned read mismatch"));
    }
    eprintln!(
        "CLIENT_PINNED http=200 udp=actual snapshot={}",
        pinned.text()
    );
    Ok(pinned)
}
fn verify_udp(
    client: &UdpSocket,
    address: SocketAddr,
    request: &[u8],
    expected: Snapshot,
) -> io::Result<()> {
    client.send_to(request, address)?;
    let mut response = [0; 96];
    let (len, sender) = client.recv_from(&mut response)?;
    if sender != address || &response[..len] != expected.text().as_bytes() {
        return Err(io::Error::other("UDP snapshot mismatch"));
    }
    Ok(())
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Smoke,
    Tick,
}
fn parse_mode(mut args: impl Iterator<Item = String>) -> io::Result<RunMode> {
    match (args.next().as_deref(), args.next()) {
        (Some("--smoke"), None) => Ok(RunMode::Smoke),
        (Some("--tick"), None) => Ok(RunMode::Tick),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "use exactly --smoke (RA1 proof) or --tick (Native Tick mode)",
        )),
    }
}

/// Callback policy only; the original Ports remains the sole game authority.
/// The Tick scheduler is created once, at first aggregate/fresh-Native readiness.
enum ApplicationMode {
    Smoke,
    Tick(Option<NativeScheduler>),
}
impl ApplicationMode {
    fn new(mode: RunMode) -> Self {
        match mode {
            RunMode::Smoke => Self::Smoke,
            RunMode::Tick => Self::Tick(None),
        }
    }
    fn is_tick(&self) -> bool {
        matches!(self, Self::Tick(_))
    }
    fn wake(&mut self, ports: &Ports, now: Instant, ready: bool) -> io::Result<PumpReport> {
        match self {
            Self::Smoke => {
                if ready {
                    ports.enable_after_required_ready()?;
                }
                ports.drain()?;
                Ok(PumpReport { published_ticks: 0 })
            }
            Self::Tick(slot) => {
                // Tick reads use HTTP. UDP keeps its reserved peer without a client.
                // Keep RA1 mutation admission closed without closing the Tick ingress.
                ports.revoke_startup_ready();
                if !ready {
                    return Ok(PumpReport { published_ticks: 0 });
                }
                if slot.is_none() {
                    *slot = Some(NativeScheduler::new(ports, now)?);
                }
                let scheduler = slot.as_mut().expect("initialized above");
                if scheduler.is_suspended() {
                    scheduler.resume(ports, now)?;
                }
                // Retain the complete PumpError, including prior successes.
                scheduler.pump(ports, now).map_err(io::Error::other)
            }
        }
    }
    fn key(
        &mut self,
        ports: &Ports,
        current: Option<WindowId>,
        id: WindowId,
        fields: (PhysicalKey, ElementState, bool),
    ) -> io::Result<KeyResult> {
        if current != Some(id) {
            return Ok(KeyResult::Unchanged);
        }
        let Self::Tick(slot) = self else {
            return Ok(KeyResult::Unchanged);
        };
        let (physical, state, repeat) = fields;
        let PhysicalKey::Code(code) = physical else {
            return Ok(KeyResult::Unchanged);
        };
        let Some(key) = map_key(code) else {
            return Ok(KeyResult::Unchanged);
        };
        let pressed = state == ElementState::Pressed;
        // Escape can stop an owned Window before its first Ready/Tick.
        if key == Key::Exit && pressed && !repeat {
            return Ok(KeyResult::ExitRequested);
        }
        match slot {
            Some(scheduler) if !scheduler.is_suspended() => {
                scheduler.key(ports, key, pressed, repeat)
            }
            _ => Ok(KeyResult::Unchanged),
        }
    }
    fn lost_focus(
        &mut self,
        ports: &Ports,
        current: Option<WindowId>,
        id: WindowId,
    ) -> io::Result<()> {
        if current == Some(id)
            && let Self::Tick(Some(scheduler)) = self
            && !scheduler.is_suspended()
        {
            scheduler.lost_focus(ports)?;
        }
        Ok(())
    }
    fn suspend(&mut self, ports: &Ports, now: Instant) -> io::Result<()> {
        match self {
            Self::Smoke => Err(io::Error::other("suspended during bounded proof")),
            Self::Tick(Some(scheduler)) => scheduler.suspend(ports, now),
            Self::Tick(None) => Ok(()),
        }
    }
    fn next_wake(&self, now: Instant) -> Instant {
        let maintenance = Duration::from_millis(12);
        let delay = match self {
            Self::Tick(Some(scheduler)) if !scheduler.is_suspended() => {
                scheduler.next_tick_in().min(maintenance)
            }
            _ => maintenance,
        };
        now + delay
    }
}

struct Application {
    root: Root,
    mode: ApplicationMode,
    client: Option<UdpSocket>,
    worker: Option<JoinHandle<()>>,
    result: Option<mpsc::Receiver<io::Result<Snapshot>>>,
    started: Instant,
    stop_deadline: Option<Instant>,
    last_completion: Instant,
    last_redraw: Instant,
    complete: bool,
    verified: Option<Snapshot>,
    native_pinned: bool,
}
impl Application {
    fn fail(&mut self, error: io::Error) {
        self.fail_at(error, Instant::now());
    }
    fn fail_at(&mut self, error: io::Error, now: Instant) {
        eprintln!("APPLICATION_FAILURE {error}");
        ports(&self.root).revoke_startup_ready();
        self.root.fail_required(error);
        self.stop_deadline
            .get_or_insert(now + Duration::from_secs(3));
    }
    fn request_stop_at(&mut self, now: Instant) {
        ports(&self.root).revoke_startup_ready();
        self.root.request_stop();
        self.stop_deadline
            .get_or_insert(now + Duration::from_secs(3));
    }
    fn accepts_callbacks(&self) -> bool {
        self.stop_deadline.is_none()
            && matches!(self.root.phase(), Phase::Prepared | Phase::Running)
    }
    /// Actual callback work, also usable with controlled time and no EventLoop.
    fn advance_at(&mut self, now: Instant) {
        if !self.accepts_callbacks() {
            return;
        }
        let ready = self.root.all_required_ready()
            && (!self.mode.is_tick() || native(&self.root).presented.is_some());
        if let Err(error) = self.mode.wake(ports(&self.root), now, ready) {
            // Bounded application policy: fail and close the original Root.
            // No callback retries or clearing of failed input/owed work.
            self.fail_at(error, now);
        }
        if !self.mode.is_tick() {
            self.smoke_work();
        }
    }
    fn suspend_at(&mut self, now: Instant) {
        if !self.accepts_callbacks() {
            return;
        }
        ports(&self.root).revoke_startup_ready();
        let result = self.mode.suspend(ports(&self.root), now).and_then(|()| {
            native_mut(&mut self.root)
                .ok_or_else(|| io::Error::other("native inactive"))?
                .suspend()
        });
        if let Err(error) = result {
            self.fail_at(error, now);
        }
    }
    fn smoke_work(&mut self) {
        if self.root.phase() == Phase::Prepared || self.root.phase() == Phase::Running {
            if matches!(self.root.phase(), Phase::Prepared | Phase::Running) {
                let p_c1 = ports(&self.root).c1().snapshot();
                if !self.native_pinned && native(&self.root).presented_final_snapshot(p_c1) {
                    let p = ports(&self.root);
                    match p.pinned(p_c1.generation, 2) {
                        Ok(s) if s == p_c1 && p.audit().pinned() == s => {
                            eprintln!(
                                "NATIVE_PINNED pipeline=rectangles instances=2 present_calls={} immutable=true audit_exact=true snapshot={}",
                                native(&self.root).presents,
                                s.text()
                            );
                            self.native_pinned = true;
                        }
                        _ => self.fail(io::Error::other("native/audit pinned mismatch")),
                    }
                }
            }
            if self.root.all_required_ready() && self.client.is_some() {
                let http = self.root.parts().last().value().endpoint();
                let udp = self
                    .root
                    .parts()
                    .previous()
                    .previous()
                    .last()
                    .value()
                    .borrow()
                    .local_addr();
                match udp {
                    Ok(udp) => {
                        let client = self.client.take().unwrap();
                        let g = ports(&self.root).c1().snapshot().generation;
                        let (tx, rx) = mpsc::sync_channel(1);
                        match thread::Builder::new()
                            .name("external-finite-client".into())
                            .spawn(move || {
                                let _ = tx.send(exercise(http, udp, client, g));
                            }) {
                            Ok(handle) => {
                                self.worker = Some(handle);
                                self.result = Some(rx);
                            }
                            Err(e) => self.fail(e),
                        }
                    }
                    Err(e) => self.fail(e),
                }
            }
            if let Some(result) = self.result.as_ref().and_then(|rx| rx.try_recv().ok()) {
                self.result.take();
                match result {
                    Ok(s) => {
                        if s != ports(&self.root).c1().snapshot() {
                            self.fail(io::Error::other("client exact identity mismatch"));
                        } else {
                            self.verified = Some(s);
                        }
                    }
                    Err(e) => self.fail(e),
                }
            }
            let current = ports(&self.root).c1().snapshot();
            if self.verified == Some(current)
                && self.native_pinned
                && native(&self.root).presented_final_snapshot(current)
                && self.started.elapsed() >= Duration::from_secs(2)
            {
                ports(&self.root).revoke_startup_ready();
                self.root.request_stop();
                self.stop_deadline = Some(Instant::now() + Duration::from_secs(3));
            }
            if self.started.elapsed() > Duration::from_secs(10) && self.stop_deadline.is_none() {
                self.fail(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "finite application deadline",
                ));
            }
        }
    }
    fn tick(&mut self, active: &ActiveEventLoop) {
        let now = Instant::now();
        self.advance_at(now);
        active.set_control_flow(ControlFlow::WaitUntil(if self.stop_deadline.is_some() {
            now + Duration::from_millis(12)
        } else {
            self.mode.next_wake(now)
        }));
        if self.stop_deadline.is_some()
            && self.last_completion.elapsed() >= Duration::from_millis(20)
        {
            self.last_completion = Instant::now();
            // A persistent driver blocker is retained. Do not hammer it after
            // the original HTTP worker has reached terminal Join.
            let blocked = native(&self.root).cleanup_failure.is_some()
                && native(&self.root).window.phase()
                    == yetlet_hosts::native_winit::WindowPhase::Closed
                && self.root.parts().last().join_status() != JoinStatus::Pending;
            if !blocked
                && self
                    .root
                    .complete_until(Instant::now() + Duration::from_millis(2))
                    == Completion::Complete
            {
                self.complete = true;
                active.exit();
            }
            if self.stop_deadline.is_some_and(|d| Instant::now() >= d) {
                eprintln!("APPLICATION_PENDING original_root_retained=true");
                active.exit();
            }
        }
        if self.stop_deadline.is_none() && self.last_redraw.elapsed() >= Duration::from_millis(16) {
            self.last_redraw = Instant::now();
            let _ = native(&self.root).window.request_redraw();
        }
    }
}
impl ApplicationHandler for Application {
    fn resumed(&mut self, active: &ActiveEventLoop) {
        if !self.accepts_callbacks() {
            return;
        }
        if self.mode.is_tick()
            && native(&self.root).window.window_id().is_some()
            && native(&self.root).gpu.is_some()
        {
            return; // duplicate active resumed: no new renderer or clock reset
        }
        let result = (|| {
            let n =
                native_mut(&mut self.root).ok_or_else(|| io::Error::other("native inactive"))?;
            let id = n.window.resumed(active)?;
            let handle = n
                .window
                .window_handle()
                .ok_or_else(|| io::Error::other("window absent"))?;
            let size = handle.inner_size();
            let mut gpu = initialize(SurfaceRenderer::new(handle, size.width, size.height))??;
            eprintln!(
                "NATIVE_CREATED pid={} window={id:?} adapter={:?}",
                std::process::id(),
                gpu.adapter_info()
            );
            gpu.resize(size.width, size.height)
                .map_err(|e| io::Error::other(gpu_failure(&gpu, "initial resize", e)))?;
            n.gpu = Some(gpu);
            Ok(())
        })();
        if let Err(e) = result {
            self.fail(e);
        }
    }
    fn window_event(&mut self, active: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let current = native(&self.root).window.window_id();
        if !self.accepts_callbacks() || current != Some(id) {
            return;
        }
        // Optional external-consumer evidence, never an SDK input API.
        // The engine/caller retains the original Winit event and its EventLoop.
        if self.mode.is_tick() && std::env::var_os("YT283_NATIVE_EVENT_PROBE").is_some() {
            match &event {
                WindowEvent::KeyboardInput { event, .. } => eprintln!(
                    "NATIVE_RAW keyboard={:?} state={:?} repeat={}",
                    event.physical_key, event.state, event.repeat
                ),
                WindowEvent::MouseInput { state, button, .. } => {
                    eprintln!("NATIVE_RAW mouse_button={button:?} state={state:?}");
                }
                WindowEvent::CursorMoved { .. } => eprintln!("NATIVE_RAW cursor_moved"),
                WindowEvent::MouseWheel { .. } => eprintln!("NATIVE_RAW mouse_wheel"),
                WindowEvent::Focused(focused) => eprintln!("NATIVE_RAW focused={focused}"),
                WindowEvent::Resized(size) => {
                    eprintln!("NATIVE_RAW resized={}x{}", size.width, size.height);
                }
                WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                    eprintln!("NATIVE_RAW scale_factor={scale_factor}");
                }
                _ => {}
            }
        }
        let now = Instant::now();
        if self.mode.is_tick()
            && matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed)
        {
            if let Some(n) = native_mut(&mut self.root) {
                n.window.observe_window_event(id, &event);
            }
            self.request_stop_at(now);
            self.tick(active);
            return;
        }
        let input = match &event {
            WindowEvent::KeyboardInput { event, .. } => self.mode.key(
                ports(&self.root),
                current,
                id,
                (event.physical_key, event.state, event.repeat),
            ),
            WindowEvent::Focused(false) => self
                .mode
                .lost_focus(ports(&self.root), current, id)
                .map(|()| KeyResult::Unchanged),
            _ => Ok(KeyResult::Unchanged),
        };
        match input {
            Ok(KeyResult::ExitRequested) => {
                self.request_stop_at(now);
                self.tick(active);
                return;
            }
            Err(error) => {
                self.fail_at(error, now);
                self.tick(active);
                return;
            }
            _ => {}
        }
        let snapshot = ports(&self.root).c1().snapshot();
        let result = (|| {
            let n =
                native_mut(&mut self.root).ok_or_else(|| io::Error::other("native inactive"))?;
            match n.window.observe_window_event(id, &event) {
                WindowSignal::RedrawReady => {
                    let live_size = n.window.window().map(Window::inner_size);
                    let eligible = n.window.is_ready();
                    let gpu = n
                        .gpu
                        .as_mut()
                        .ok_or_else(|| io::Error::other("GPU absent"))?;
                    reconfigure_redraw(gpu.phase(), eligible, live_size, |size| {
                        gpu.resize(size.width, size.height)
                            .map(|_| ())
                            .map_err(|e| io::Error::other(gpu_failure(gpu, "redraw resize", e)))
                    })?;
                    let receipt = gpu
                        .draw_rects(&project_scene(snapshot))
                        .map_err(|e| io::Error::other(gpu_failure(gpu, "draw rectangles", e)))?;
                    n.record_present(snapshot, receipt);
                    if n.window.is_ready() && n.presented.is_some() {
                        self.root.parts_mut().previous().last().mark_ready();
                    }
                }
                WindowSignal::Resized(size) => {
                    if let Some(g) = n.gpu.as_mut() {
                        g.resize(size.width, size.height)
                            .map_err(|e| io::Error::other(gpu_failure(g, "window resize", e)))?;
                    }
                }
                WindowSignal::CloseRequested | WindowSignal::Destroyed => {
                    return Err(io::Error::other("window closed before proof"));
                }
                _ => {}
            }
            Ok(())
        })();
        if let Err(e) = result {
            self.fail(e);
        }
        self.tick(active);
    }
    fn about_to_wait(&mut self, active: &ActiveEventLoop) {
        if self.stop_deadline.is_none()
            && matches!(self.root.phase(), Phase::Prepared | Phase::Running)
        {
            let result = self
                .root
                .parts()
                .previous()
                .previous()
                .last()
                .value()
                .borrow_mut()
                .poll(ports(&self.root));
            if let Err(e) = result {
                self.fail(e);
            }
        }
        self.tick(active);
    }
    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.suspend_at(Instant::now());
    }
}
fn prepare_udp_then_ports(
    generation: u64,
    client_bind: SocketAddr,
    acquire_ports: impl FnOnce(u64) -> io::Result<Ports>,
) -> io::Result<(Ports, UdpSocket, Network, SocketAddr, SocketAddr)> {
    let client = UdpSocket::bind(client_bind)?;
    let network = Network::new(&client)?;
    let endpoint = network.local_addr()?;
    let peer = network.peer()?;
    // Both potentially failing address lookups must finish before any C1,
    // Audit or admission resource is acquired. Only sockets exist on error.
    let ports = acquire_ports(generation)?;
    Ok((ports, client, network, endpoint, peer))
}
/// Prepare original Hosts without creating a Window or executing a Tick.
fn prepare_application(mode: RunMode, generation: u64, now: Instant) -> io::Result<Application> {
    let (p, client, network, udp, peer) =
        prepare_udp_then_ports(generation, "127.0.0.1:0".parse().unwrap(), Ports::new)?;
    let audit = p.audit_attachment();
    let web_launch = p.web_launch();
    eprintln!(
        "EXTERNAL_START pid={} generation={} udp={} peer={}",
        std::process::id(),
        generation,
        udp,
        peer
    );
    let native_slot = Native {
        window: WindowSlot::new(
            Window::default_attributes()
                .with_title("Yetlet external revision audit (Twin Squares)")
                .with_inner_size(LogicalSize::new(640.0, 420.0)),
        ),
        gpu: None,
        presents: 0,
        presented: None,
        cleanup_failure: None,
    };
    let builder = Builder::<io::Error>::new()
        .required_local("C1", p, Ports::close as Close<Ports>)
        .required_local(
            "revision-audit",
            audit,
            close_audit as Close<AuditAttachment>,
        )
        .required_local(
            "UDP",
            RefCell::new(network),
            close_network as Close<RefCell<Network>>,
        )
        .required_foreground("Native", native_slot, Native::close as Close<Native>);
    let builder = builder.try_required_worker(
        "Web",
        || three_way::start_web(web_launch, "127.0.0.1:0".parse().unwrap()),
        close_admission as Close<Web>,
        stop_web as Close<Web>,
        observe_web as fn(&mut Web, Duration) -> io::Result<WorkerCompletion<io::Error>>,
    );
    let mut root: Root = match builder {
        Ok(b) => b.build(),
        Err(f) => {
            let mut r = f.into_root();
            let result = r.complete_until(Instant::now() + IO_BUDGET);
            eprintln!(
                "STARTUP_ROLLBACK {result:?} primary={:?}",
                r.failures().primary()
            );
            if result == Completion::Pending {
                Box::leak(Box::new(r));
            }
            return Err(io::Error::other("Web startup failed"));
        }
    };
    eprintln!(
        "HTTP_BOUND endpoint={}",
        root.parts().last().value().endpoint()
    );
    if let Err(e) = root
        .parts()
        .last()
        .value()
        .wait_http_ready("/ready", IO_BUDGET)
    {
        root.fail_required(e);
    } else {
        eprintln!("HTTP_READY actual=200");
        root.parts_mut().last().mark_ready();
    }
    let initial_stop = (root.phase() == Phase::Stopping).then(|| now + Duration::from_secs(3));
    Ok(Application {
        root,
        mode: ApplicationMode::new(mode),
        client: if mode == RunMode::Smoke {
            Some(client)
        } else {
            None
        },
        worker: None,
        result: None,
        started: now,
        stop_deadline: initial_stop,
        last_completion: now,
        last_redraw: now,
        complete: false,
        verified: None,
        native_pinned: false,
    })
}
fn main() -> io::Result<()> {
    let mode = parse_mode(std::env::args().skip(1))?;
    let generation = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(io::Error::other)?
        .as_nanos() as u64;
    let mut app = prepare_application(mode, generation, Instant::now())?;
    let event_loop = match EventLoop::new() {
        Ok(l) => l,
        Err(e) => {
            app.fail(io::Error::other(e));
            let c = app
                .root
                .complete_until(Instant::now() + Duration::from_secs(3));
            if c == Completion::Pending {
                Box::leak(Box::new(app));
            }
            return Err(io::Error::other("EventLoop startup failed"));
        }
    };
    // Preserve all original loop budgets after Host/EventLoop preparation.
    app.started = Instant::now();
    app.stop_deadline = app
        .stop_deadline
        .map(|_| app.started + Duration::from_secs(3));
    app.last_completion = app.started;
    app.last_redraw = app.started;
    if let Err(e) = event_loop.run_app(&mut app) {
        app.fail(io::Error::other(e));
    }
    if !app.complete
        && !(native(&app.root).cleanup_failure.is_some()
            && native(&app.root).window.phase() == yetlet_hosts::native_winit::WindowPhase::Closed
            && app.root.parts().last().join_status() != JoinStatus::Pending)
    {
        app.complete = app
            .root
            .complete_until(Instant::now() + Duration::from_secs(3))
            == Completion::Complete;
    }
    if let Some(handle) = app.worker.take() {
        if handle.join().is_err() {
            app.fail(io::Error::other("finite client panicked"));
        }
        eprintln!("FINITE_CLIENT_JOINED");
    }
    let presents = native(&app.root).presents;
    eprintln!(
        "EXTERNAL_FINAL complete={} phase={:?} verified={} native_pinned={} presents={} web_join={:?} primary={:?} secondary={:?} c1_owners={}",
        app.complete,
        app.root.phase(),
        app.verified.is_some(),
        app.native_pinned,
        presents,
        app.root.parts().last().join_status(),
        app.root.failures().primary(),
        app.root.failures().secondary(),
        Rc::strong_count(ports(&app.root).c1())
    );
    if !app.complete {
        Box::leak(Box::new(app));
        return Err(io::Error::other(
            "Pending root retained until process exit; proof incomplete",
        ));
    }
    if app.root.failures().primary().is_some()
        || (!app.mode.is_tick() && (app.verified.is_none() || !app.native_pinned || presents == 0))
    {
        return Err(io::Error::other("external proof failed"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use yetlet_hosts::{
        native_winit::WindowPhase,
        renderer_wgpu::{DeviceLostError, wgpu},
    };
    use yt283_revision_audit::AuditStatus;

    fn prepared_native() -> Native {
        Native {
            window: WindowSlot::new(Window::default_attributes()),
            gpu: None,
            presents: 0,
            presented: None,
            cleanup_failure: None,
        }
    }
    fn synthetic_native_ready(app: &mut Application) {
        // Headless fixture only: no Window, GPU, draw or returned Present claim.
        let snapshot = ports(&app.root).c1().snapshot();
        native_mut(&mut app.root).unwrap().presented = Some(snapshot);
        app.root.parts_mut().previous().last().mark_ready();
    }
    fn scheduler(app: &Application) -> &NativeScheduler {
        match &app.mode {
            ApplicationMode::Tick(Some(s)) => s,
            _ => panic!("Tick scheduler absent"),
        }
    }
    fn close_headless(app: &mut Application) {
        app.request_stop_at(Instant::now());
        assert!(ports(&app.root).ingress().is_closed().unwrap());
        assert_eq!(
            app.root
                .complete_until(Instant::now() + Duration::from_secs(3)),
            Completion::Complete
        );
        assert!(ports(&app.root).c1().is_stopped());
        assert_eq!(ports(&app.root).audit().status(), AuditStatus::Closed);
        assert_ne!(app.root.parts().last().join_status(), JoinStatus::Pending);
        assert!(app.root.failures().secondary().is_empty());
        // An unspawned RA1 test client is not part of the original Root.
        drop(app.client.take());
    }
    fn routed_key(mode: &mut ApplicationMode, p: &Ports, code: winit::keyboard::KeyCode) {
        let id = WindowId::from(1_u64);
        assert_eq!(
            mode.key(
                p,
                Some(id),
                id,
                (PhysicalKey::Code(code), ElementState::Pressed, false)
            )
            .unwrap(),
            KeyResult::Queued
        );
    }

    #[test]
    fn native_tick_cli_requires_one_explicit_mode_before_resources() {
        for (args, expected) in [
            (vec!["--smoke"], RunMode::Smoke),
            (vec!["--tick"], RunMode::Tick),
        ] {
            assert_eq!(
                parse_mode(args.into_iter().map(str::to_owned)).unwrap(),
                expected
            );
        }
        for args in [
            vec![],
            vec!["--other"],
            vec!["--tick", "--smoke"],
            vec!["--smoke", "--tick"],
            vec!["--tick", "extra"],
        ] {
            assert_eq!(
                parse_mode(args.into_iter().map(str::to_owned))
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidInput
            );
        }
    }

    #[test]
    fn native_tick_key_fields_owned_window_and_scene_share_original_publication() {
        use winit::keyboard::KeyCode;
        use yt283_revision_audit::{native_clock::FIXED_STEP, tick_m1::HeldDirection};
        let p = Ports::new(91).unwrap();
        let start = Instant::now();
        let id = WindowId::from(1_u64);
        let foreign = WindowId::from(2_u64);
        let mut mode = ApplicationMode::new(RunMode::Tick);
        let key = (
            PhysicalKey::Code(KeyCode::KeyD),
            ElementState::Pressed,
            false,
        );
        assert_eq!(
            mode.key(&p, Some(id), id, key).unwrap(),
            KeyResult::Unchanged
        );
        assert_eq!(
            mode.key(
                &p,
                Some(id),
                foreign,
                (
                    PhysicalKey::Code(KeyCode::Escape),
                    ElementState::Pressed,
                    false
                )
            )
            .unwrap(),
            KeyResult::Unchanged
        );
        assert_eq!(
            mode.key(
                &p,
                Some(id),
                id,
                (
                    PhysicalKey::Code(KeyCode::Escape),
                    ElementState::Pressed,
                    false
                )
            )
            .unwrap(),
            KeyResult::ExitRequested
        );
        assert_eq!(
            mode.wake(&p, start + Duration::from_secs(30), false)
                .unwrap()
                .published_ticks,
            0
        );
        let ready = start + Duration::from_secs(40);
        assert_eq!(mode.wake(&p, ready, true).unwrap().published_ticks, 0);
        assert_eq!(p.c1().snapshot().revision, 0);
        routed_key(&mut mode, &p, KeyCode::KeyD);
        routed_key(&mut mode, &p, KeyCode::ArrowUp);
        assert_eq!(
            mode.key(
                &p,
                Some(id),
                foreign,
                (
                    PhysicalKey::Code(KeyCode::KeyW),
                    ElementState::Pressed,
                    false
                )
            )
            .unwrap(),
            KeyResult::Unchanged
        );
        assert_eq!(
            mode.key(
                &p,
                Some(id),
                id,
                (
                    PhysicalKey::Code(KeyCode::KeyW),
                    ElementState::Pressed,
                    true
                )
            )
            .unwrap(),
            KeyResult::Unchanged
        );
        assert_eq!(
            mode.wake(&p, ready + Duration::from_millis(12), true)
                .unwrap()
                .published_ticks,
            0
        );
        assert_eq!(
            mode.next_wake(ready + Duration::from_millis(12)),
            ready + FIXED_STEP
        );
        assert_eq!(
            mode.wake(&p, ready + FIXED_STEP, true)
                .unwrap()
                .published_ticks,
            1
        );
        let proof = p.pinned_tick(91, 1).unwrap();
        assert_eq!(proof.applied.snapshot.text(), "91 1 false 3 2 20 11");
        assert_eq!(
            proof.applied.clock.held,
            [HeldDirection::East, HeldDirection::North]
        );
        let scene = project_scene(proof.applied.snapshot);
        assert_eq!(scene[0].center, project_scene(p.c1().snapshot())[0].center);
        assert_eq!(p.c1().tick_observation(), proof.applied);
        routed_key(&mut mode, &p, KeyCode::Space);
        assert_eq!(
            mode.wake(&p, ready + FIXED_STEP * 3, true)
                .unwrap()
                .published_ticks,
            1
        );
        assert!(p.c1().snapshot().paused);
        routed_key(&mut mode, &p, KeyCode::Enter);
        routed_key(&mut mode, &p, KeyCode::KeyD);
        assert_eq!(
            mode.wake(&p, ready + FIXED_STEP * 4, true)
                .unwrap()
                .published_ticks,
            1
        );
        mode.lost_focus(&p, Some(id), foreign).unwrap();
        mode.lost_focus(&p, Some(id), id).unwrap();
        assert_eq!(
            mode.wake(&p, ready + FIXED_STEP * 5, true)
                .unwrap()
                .published_ticks,
            1
        );
        assert_eq!(
            p.c1().tick_observation().clock.held,
            [HeldDirection::Still; 2]
        );
        assert_eq!(p.c1().tick_observation().clock.last_sequence, 6);
        assert_eq!(*proof, *p.pinned_tick(91, 1).unwrap());
    }

    #[test]
    fn native_tick_application_waits_for_fresh_ready_across_suspend_resume() {
        use winit::keyboard::KeyCode;
        use yt283_revision_audit::{native_clock::FIXED_STEP, tick_m1::HeldDirection};
        let start = Instant::now();
        let mut app = prepare_application(RunMode::Tick, 91, start).unwrap();
        app.advance_at(start + Duration::from_secs(5));
        assert!(matches!(app.mode, ApplicationMode::Tick(None)));
        assert!(app.client.is_none() && app.worker.is_none());
        synthetic_native_ready(&mut app);
        let ready = start + Duration::from_secs(10);
        app.advance_at(ready);
        assert_eq!(ports(&app.root).c1().snapshot().revision, 0);
        routed_key(&mut app.mode, ports(&app.root), KeyCode::KeyD);
        routed_key(&mut app.mode, ports(&app.root), KeyCode::ArrowUp);
        app.advance_at(ready + FIXED_STEP);
        let before = ports(&app.root).c1().tick_observation();
        app.suspend_at(ready + FIXED_STEP + Duration::from_nanos(3));
        assert!(app.stop_deadline.is_none());
        assert!(scheduler(&app).is_suspended());
        assert_eq!(scheduler(&app).pending_input_count(), 2);
        assert_eq!(native(&app.root).window.phase(), WindowPhase::Suspended);
        assert!(native(&app.root).presented.is_none());
        let timing = scheduler(&app).timing();
        app.suspend_at(ready + Duration::from_secs(8));
        assert_eq!(scheduler(&app).timing(), timing);
        let resumed = ready + Duration::from_secs(10);
        app.advance_at(resumed); // old Core Ready is insufficient without fresh Present
        assert!(scheduler(&app).is_suspended());
        assert_eq!(ports(&app.root).c1().tick_observation(), before);
        synthetic_native_ready(&mut app);
        app.advance_at(resumed);
        assert!(!scheduler(&app).is_suspended());
        assert_eq!(ports(&app.root).c1().tick_observation(), before);
        app.advance_at(resumed + FIXED_STEP);
        let after = ports(&app.root).c1().tick_observation();
        assert_eq!(after.clock.tick, 2);
        assert_eq!(after.clock.last_sequence, 4);
        assert_eq!(after.clock.held, [HeldDirection::Still; 2]);
        assert_eq!(after.snapshot.positions, before.snapshot.positions);
        assert_eq!(ports(&app.root).pinned_tick(91, 2).unwrap().inputs.len(), 2);
        close_headless(&mut app);
        assert!(app.root.failures().primary().is_none());
        assert_eq!(ports(&app.root).audit().pinned_tick(), Some(after));
    }

    #[test]
    fn native_tick_publication_error_retains_input_and_stops_original_root() {
        use winit::keyboard::KeyCode;
        use yt283_revision_audit::native_clock::{FIXED_STEP, PumpError};
        let start = Instant::now();
        let mut app = prepare_application(RunMode::Tick, 91, start).unwrap();
        synthetic_native_ready(&mut app);
        app.advance_at(start);
        routed_key(&mut app.mode, ports(&app.root), KeyCode::KeyD);
        app.advance_at(start + FIXED_STEP);
        let before = ports(&app.root).c1().tick_observation();
        let proof = ports(&app.root).pinned_tick(91, 1).unwrap();
        let saved = (*proof).clone();
        routed_key(&mut app.mode, ports(&app.root), KeyCode::KeyW);
        let timing = scheduler(&app).timing();
        ports(&app.root).ingress().close().unwrap(); // genuine original publication gate
        app.advance_at(start + FIXED_STEP * 2);
        assert!(!app.accepts_callbacks());
        assert_eq!(scheduler(&app).pending_input_count(), 1);
        assert_eq!(scheduler(&app).timing(), timing);
        assert_eq!(ports(&app.root).c1().tick_observation(), before);
        let error = app
            .root
            .failures()
            .primary()
            .unwrap()
            .get_ref()
            .unwrap()
            .downcast_ref::<PumpError>()
            .unwrap();
        assert_eq!(error.published_ticks, 0);
        assert!(error.error.to_string().contains("closed"));
        let deadline = app.stop_deadline;
        app.advance_at(start + Duration::from_secs(1));
        app.suspend_at(start + Duration::from_secs(1));
        assert_eq!(app.stop_deadline, deadline);
        assert_eq!(scheduler(&app).timing(), timing);
        close_headless(&mut app);
        assert_eq!(scheduler(&app).pending_input_count(), 1);
        assert_eq!(ports(&app.root).audit().pinned_tick(), Some(before));
        assert_eq!(*proof, saved);
    }

    #[test]
    fn native_tick_full_release_queue_fails_suspend_without_partial_rewrite() {
        use winit::keyboard::KeyCode;
        use yt283_revision_audit::native_clock::FIXED_STEP;
        let start = Instant::now();
        let mut app = prepare_application(RunMode::Tick, 91, start).unwrap();
        synthetic_native_ready(&mut app);
        app.advance_at(start);
        for code in [KeyCode::KeyD, KeyCode::ArrowUp] {
            routed_key(&mut app.mode, ports(&app.root), code);
        }
        for _ in 0..29 {
            routed_key(&mut app.mode, ports(&app.root), KeyCode::Enter);
        }
        let before = scheduler(&app).timing();
        app.suspend_at(start + FIXED_STEP);
        assert!(!app.accepts_callbacks());
        assert!(!scheduler(&app).is_suspended());
        assert_eq!(scheduler(&app).timing(), before);
        assert_eq!(scheduler(&app).pending_input_count(), 31);
        assert_eq!(ports(&app.root).c1().snapshot().revision, 0);
        assert_eq!(
            app.root.failures().primary().unwrap().kind(),
            io::ErrorKind::WouldBlock
        );
        close_headless(&mut app);
        assert_eq!(scheduler(&app).pending_input_count(), 31);
        assert!(ports(&app.root).audit().tick_records().is_empty());
    }

    #[test]
    fn native_tick_zero_and_thirty_three_ticks_close_without_smoke_client_or_extra_tick() {
        use winit::keyboard::KeyCode;
        use yt283_revision_audit::native_clock::FIXED_STEP;
        for ticks in [0_u32, 33] {
            let start = Instant::now();
            let mut app = prepare_application(RunMode::Tick, 91, start).unwrap();
            let mut first = None;
            if ticks != 0 {
                synthetic_native_ready(&mut app);
                app.advance_at(start);
                routed_key(&mut app.mode, ports(&app.root), KeyCode::KeyD);
                for tick in 1..=ticks {
                    app.advance_at(start + FIXED_STEP * tick);
                    if tick == 1 {
                        let proof = ports(&app.root).pinned_tick(91, 1).unwrap();
                        first = Some((Arc::clone(&proof), (*proof).clone()));
                    }
                }
                assert_eq!(ports(&app.root).audit().tick_records().len(), 16);
                assert_eq!(
                    ports(&app.root).pinned_tick(91, 1).unwrap_err().kind(),
                    io::ErrorKind::NotFound
                );
                routed_key(&mut app.mode, ports(&app.root), KeyCode::KeyW);
            }
            let before = ports(&app.root).c1().tick_observation();
            assert_eq!(before.clock.tick, u64::from(ticks));
            assert!(app.client.is_none() && app.worker.is_none() && app.result.is_none());
            assert!(app.verified.is_none() && !app.native_pinned);
            close_headless(&mut app);
            assert!(app.root.failures().primary().is_none());
            assert_eq!(ports(&app.root).c1().tick_observation(), before);
            if let Some((proof, saved)) = first {
                assert_eq!(*proof, saved);
                assert_eq!(scheduler(&app).pending_input_count(), 1);
            }
        }
    }

    #[test]
    fn native_tick_and_ra1_http_admission_are_isolated_through_same_mode_wake() {
        use yt283_revision_audit::native_clock::FIXED_STEP;
        for mode in [RunMode::Tick, RunMode::Smoke] {
            let start = Instant::now();
            let mut app = prepare_application(mode, 91, start).unwrap();
            let endpoint = app.root.parts().last().value().endpoint();
            let command = encode_command(Action::Pause, 91, 0);
            assert_eq!(command.len(), 20);
            assert_eq!(http(endpoint, "POST", "/command", &command).unwrap().0, 503);
            // Call the exact mode branch; no finite --smoke client or Window.
            app.mode.wake(ports(&app.root), start, true).unwrap();
            let admitted = http(endpoint, "POST", "/command", &command).unwrap().0;
            assert_eq!(admitted, if mode == RunMode::Tick { 503 } else { 202 });
            assert_eq!(ports(&app.root).c1().snapshot().revision, 0);
            app.mode
                .wake(ports(&app.root), start + FIXED_STEP, true)
                .unwrap();
            let snapshot = ports(&app.root).c1().snapshot();
            assert_eq!(snapshot.revision, 1);
            assert_eq!(
                http(endpoint, "GET", "/snapshot/91/1", &[]).unwrap(),
                (200, snapshot.text())
            );
            assert_eq!(
                ports(&app.root).audit().records().len(),
                usize::from(mode == RunMode::Smoke)
            );
            assert_eq!(
                ports(&app.root).audit().tick_records().len(),
                usize::from(mode == RunMode::Tick)
            );
            assert_eq!(snapshot.paused, mode == RunMode::Smoke);
            close_headless(&mut app);
            assert!(app.root.failures().primary().is_none());
        }
    }

    #[test]
    fn projection_maps_two_square_centers_colors_and_arena_edges() {
        let state = Snapshot {
            generation: 1,
            revision: 2,
            paused: false,
            positions: [Position { x: 3, y: 2 }, Position { x: 20, y: 11 }],
        };
        let [one, two] = project_scene(state);
        assert!((one.center[0] - (-2.0 / 3.0)).abs() < 1e-6);
        assert_eq!(one.center[1], 0.625);
        assert_eq!(one.half_extent, [1.0 / 12.0, 1.0 / 8.0]);
        assert_eq!(one.rgba, [0.24, 0.65, 1.0, 1.0]);
        assert_eq!(two.center, [0.75, -0.5]);
        assert_eq!(two.half_extent, one.half_extent);
        assert_eq!(two.rgba, [1.0, 0.57, 0.26, 1.0]);
        let [top_left, bottom_right] = project_scene(Snapshot {
            positions: [Position { x: 0, y: 0 }, Position { x: 22, y: 14 }],
            ..state
        });
        assert!((top_left.center[0] - top_left.half_extent[0] + 1.0).abs() < 1e-6);
        assert_eq!(top_left.center[1] + top_left.half_extent[1], 1.0);
        assert!((bottom_right.center[0] + bottom_right.half_extent[0] - 1.0).abs() < 1e-6);
        assert_eq!(bottom_right.center[1] - bottom_right.half_extent[1], -1.0);
    }

    #[test]
    fn closeout_pin_requires_returned_present_of_exact_revision_and_positions() {
        let mut native = prepared_native();
        let snapshot = Snapshot {
            generation: 2,
            revision: 2,
            paused: false,
            positions: [Position { x: 3, y: 2 }, Position { x: 20, y: 11 }],
        };
        let prior = Snapshot {
            revision: 1,
            ..snapshot
        };
        native.record_present(
            prior,
            DrawReceipt::Presented {
                present_calls: 1,
                suboptimal: false,
            },
        );
        native.record_present(snapshot, DrawReceipt::Retry(wgpu::SurfaceError::Outdated));
        native.record_present(snapshot, DrawReceipt::Deferred(SurfacePhase::Unconfigured));
        assert_eq!(native.presents, 1);
        assert!(!native.presented_final_snapshot(snapshot));
        let mut wrong = snapshot;
        wrong.positions[1].y += 1;
        native.record_present(
            wrong,
            DrawReceipt::Presented {
                present_calls: 2,
                suboptimal: false,
            },
        );
        assert!(!native.presented_final_snapshot(snapshot));
        native.record_present(
            snapshot,
            DrawReceipt::Presented {
                present_calls: 3,
                suboptimal: false,
            },
        );
        assert!(native.presented_final_snapshot(snapshot));
    }

    #[test]
    fn surface_cleanup_error_still_closes_window_and_retains_pending_owners() {
        let mut n = prepared_native();
        let diagnostic = Arc::new(FatalError {
            operation: "SurfaceTexture::discard",
            diagnostic: "synthetic discard failure".into(),
        });
        let failure = retain_gpu_failure(
            "close",
            SurfacePhase::Closed,
            RenderError::FrameCleanupFailed,
            None,
            Some(diagnostic.clone()),
        );
        // Public close semantics: Surface already released despite this Err.
        // Use the same production continuation and real WindowSlot::close.
        assert!(
            close_window_after_surface(&mut n.window, &mut n.cleanup_failure, Err(failure))
                .is_err()
        );
        assert_eq!(n.window.phase(), WindowPhase::Closed);
        let original = n.cleanup_failure.clone().unwrap();
        assert!(original.cleanup.is_none());
        let expected: Arc<dyn Error + Send + Sync> = diagnostic.clone();
        assert!(Arc::ptr_eq(original.cause.as_ref().unwrap(), &expected));
        let ports = Ports::new(90).unwrap();
        let c1 = ports.c1().clone();
        let audit = ports.audit().clone();
        let audit_attachment = ports.audit_attachment();
        let mut root = Builder::<io::Error>::new()
            .required_local("C1", ports, Ports::close)
            .required_local("audit", audit_attachment, close_audit)
            .required_local("native", n, Native::close)
            .build();
        assert_eq!(root.complete_until(Instant::now()), Completion::Pending);
        assert_eq!(
            root.parts().last().value().window.phase(),
            WindowPhase::Closed
        );
        assert!(!c1.is_stopped());
        assert_eq!(audit.status(), AuditStatus::Open);
        assert!(Rc::strong_count(&c1) > 1);
        let primary = root
            .failures()
            .primary()
            .unwrap()
            .get_ref()
            .unwrap()
            .downcast_ref::<NativeCloseFailure>()
            .unwrap();
        assert!(Arc::ptr_eq(&primary.gpu, &original));
        assert_eq!(root.complete_until(Instant::now()), Completion::Pending);
        // Composition records the explicit failed retry as secondary. Both
        // failures must still own the exact first diagnostic, never replace it.
        assert_eq!(root.failures().secondary().len(), 1);
        let retry = root.failures().secondary()[0]
            .get_ref()
            .unwrap()
            .downcast_ref::<NativeCloseFailure>()
            .unwrap();
        assert!(Arc::ptr_eq(&retry.gpu, &original));
    }
    #[test]
    fn next_eligible_redraw_reconfigures_live_size_after_lost_or_outdated() {
        for reason in [wgpu::SurfaceError::Lost, wgpu::SurfaceError::Outdated] {
            // Synthetic post-acquisition phase; no actual driver fault induced.
            let phase = SurfacePhase::Unconfigured;
            let live = PhysicalSize::new(731, 489);
            let calls = Cell::new(0);
            reconfigure_redraw(phase, true, Some(live), |size| {
                assert_eq!(size, live, "{reason:?}");
                calls.set(calls.get() + 1);
                Ok(())
            })
            .unwrap();
            assert_eq!(calls.get(), 1);
            for (phase, eligible, size) in [
                (SurfacePhase::Configured, true, Some(live)),
                (SurfacePhase::Unconfigured, false, Some(live)),
                (SurfacePhase::Unconfigured, true, None),
            ] {
                reconfigure_redraw(phase, eligible, size, |_| panic!("ineligible reconfigure"))
                    .unwrap();
            }
        }
    }
    #[test]
    fn original_gpu_cause_survives_renderer_owner_cleanup_as_required_primary() {
        for stage in ["draw", "window resize", "redraw resize", "initial resize"] {
            let loss = Arc::new(DeviceLostError {
                reason: wgpu::DeviceLostReason::Unknown,
                diagnostic: "synthetic original driver diagnostic".into(),
            });
            let cleanup = Arc::new(FatalError {
                operation: "SurfaceTexture::discard",
                diagnostic: "synthetic secondary discard diagnostic".into(),
            });
            let cause: Arc<dyn Error + Send + Sync> = loss.clone();
            let failure = retain_gpu_failure(
                stage,
                SurfacePhase::Failed,
                RenderError::DeviceLost,
                Some(cause.clone()),
                Some(cleanup.clone()),
            );
            let mut root = Builder::<io::Error>::new()
                .required_local("native", prepared_native(), Native::close)
                .build();
            root.fail_required(io::Error::other(failure));
            assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
            let stored = root
                .failures()
                .primary()
                .unwrap()
                .get_ref()
                .unwrap()
                .downcast_ref::<GpuFailure>()
                .unwrap();
            assert_eq!(stored.stage, stage);
            assert_eq!(stored.phase, SurfacePhase::Failed);
            assert_eq!(stored.category, RenderError::DeviceLost);
            assert!(Arc::ptr_eq(stored.cause.as_ref().unwrap(), &cause));
            assert!(Arc::ptr_eq(stored.cleanup.as_ref().unwrap(), &cleanup));
            assert!(
                stored
                    .source()
                    .unwrap()
                    .to_string()
                    .contains(&loss.diagnostic)
            );
            assert!(root.failures().secondary().is_empty());
        }
    }
    #[test]
    fn startup_returns_both_live_udp_addresses_before_owning_domain() {
        let acquired = Cell::new(0);
        let (mut ports, client, mut network, endpoint, peer) =
            prepare_udp_then_ports(92, "127.0.0.1:0".parse().unwrap(), |generation| {
                acquired.set(acquired.get() + 1);
                Ports::new(generation)
            })
            .unwrap();
        assert_eq!(acquired.get(), 1);
        assert_eq!(endpoint, network.local_addr().unwrap());
        assert_eq!(peer, client.local_addr().unwrap());
        assert_eq!(peer, network.peer().unwrap());
        assert_eq!(ports.c1().snapshot().generation, 92);
        network.close().unwrap();
        assert_eq!(
            network.peer().unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
        ports.ingress().close().unwrap();
        let mut audit = ports.audit_attachment();
        audit.close().unwrap();
        ports.close().unwrap();
    }

    #[test]
    fn real_udp_bind_collision_precedes_domain_audit_and_admission_acquisition() {
        let blocker = UdpSocket::bind("127.0.0.1:0").unwrap();
        let acquired = Cell::new(false);
        let result = prepare_udp_then_ports(91, blocker.local_addr().unwrap(), |generation| {
            acquired.set(true);
            Ports::new(generation)
        });
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("UDP collision succeeded"),
        };
        assert_eq!(error.kind(), io::ErrorKind::AddrInUse);
        assert!(!acquired.get(), "C1/audit/admission factory must not run");
    }
}
