//! YT-272 D2 — two real independently launched *applications*, each one
//! foreground winit GameHost plus a selected NetworkHost or NetworkClient.
//! No Host-per-process/thread, generic Transport API or network exposure.
#[cfg(feature = "f1-web")]
use std::time::Duration;
use std::{error::Error, net::SocketAddrV4, sync::Arc, time::Instant};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use yetlet_composable_host_research::{
    protocol::WireSnapshot,
    simulation::{ARENA_HEIGHT, ARENA_WIDTH, Position, SQUARE_SIZE},
};
use yetlet_native_gamehost_research::{
    RENDER_STEP, Rect, Scene, TickClock, c3_foreground_wake_delay, project, scene_bytes,
    udp_d2::{BIND_IP, NetworkClient, NetworkHost},
};

#[cfg(feature = "f1-web")]
use yetlet_native_gamehost_research::f1_web::{
    F1Completion, F1WebService, REQUIRED_WEB_EXIT_CAUSE, preserve_web_worker_cause,
};

type ResultAny<T> = Result<T, Box<dyn Error>>;

struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    instances: wgpu::Buffer,
    ready: bool,
    drawn: u64,
}
impl Renderer {
    fn new(window: Arc<Window>) -> ResultAny<Self> {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Err("Zero-size Window".into());
        }
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }))?;
        let info = adapter.get_info();
        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("M28 D2 Native GameHost GPU"),
                ..Default::default()
            }))?;
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or("No compatible wgpu Surface format")?;
        surface.configure(&device, &config);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("M28 D2 identical Twin Squares shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shader.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("M28 D2 two-square pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: 32,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("D2 two read-only scene rects"),
            contents: &[0u8; 64],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        eprintln!(
            "D2_RENDERER_READY gpu={:?} backend={:?}",
            info.name, info.backend
        );
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            instances,
            ready: true,
            drawn: 0,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            self.ready = false;
        } else if !self.ready
            || self.config.width != size.width
            || self.config.height != size.height
        {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.ready = true;
        }
    }
    fn draw(&mut self, scene: &Scene) -> ResultAny<bool> {
        if !self.ready {
            return Ok(false);
        }
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Timeout) => return Ok(false),
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(false);
            }
            Err(error) => return Err(format!("D2 frame acquire failed: {error}").into()),
        };
        self.queue
            .write_buffer(&self.instances, 0, &scene_bytes(scene));
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("D2 scene render"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("D2 native draw"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.07,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.draw(0..6, 0..2);
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        self.drawn += 1;
        Ok(true)
    }
}

/// Actual wire-provided *read-only* replica state, rendered with exactly the
/// original C2 square projection math, but no C1 authoritative owner.
fn project_wire(snapshot: &WireSnapshot) -> Scene {
    let colors = [[0.24, 0.65, 1.0, 1.0], [1.0, 0.57, 0.26, 1.0]];
    let rects = std::array::from_fn(|i| {
        let Position { x, y } = snapshot.positions[i];
        Rect {
            center_extent: [
                (2 * x + SQUARE_SIZE) as f32 / ARENA_WIDTH as f32 - 1.0,
                1.0 - (2 * y + SQUARE_SIZE) as f32 / ARENA_HEIGHT as f32,
                SQUARE_SIZE as f32 / ARENA_WIDTH as f32,
                SQUARE_SIZE as f32 / ARENA_HEIGHT as f32,
            ],
            color: colors[i],
        }
    });
    Scene {
        generation: snapshot.generation,
        tick: snapshot.tick,
        revision: snapshot.revision,
        rects,
    }
}

enum Network {
    Host(NetworkHost),
    Client(NetworkClient),
}
impl Network {
    fn name(&self) -> &'static str {
        match self {
            Self::Host(_) => "host",
            Self::Client(_) => "client",
        }
    }
}

/// Never advance the authoritative C1 owner beyond the selected app's
/// control-tick budget, even when a stalled foreground callback yields
/// several due ticks. Saturating subtraction also handles an exhausted cap.
fn d2_bounded_due(control_ticks: u64, max_ticks: u64, due: u32) -> u32 {
    u64::from(due).min(max_ticks.saturating_sub(control_ticks)) as u32
}

/// Bounded demo exit is independent of window visibility, rendering success
/// or the arrival of a trusted remote Snapshot.
fn d2_tick_limit_reached(control_ticks: u64, max_ticks: u64) -> bool {
    control_ticks >= max_ticks
}

#[cfg(feature = "f1-web")]
fn required_web_failure(web: Option<&F1WebService>) -> Option<&'static str> {
    web.filter(|service| service.is_finished())
        .map(|_| REQUIRED_WEB_EXIT_CAUSE)
}

struct D2App {
    network: Network,
    // YT-277: only explicitly selected for the private host-web mode.
    #[cfg(feature = "f1-web")]
    web: Option<F1WebService>,
    #[cfg(feature = "f1-web")]
    web_exited_first: bool,
    #[cfg(feature = "f1-web")]
    last_applied_control: Option<u64>,
    #[cfg(feature = "f1-web")]
    last_f1_replica_revision: Option<u64>,
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    clock: TickClock,
    last_draw: Instant,
    control_ticks: u64,
    max_ticks: u64,
    displayed_revision: u64,
    last_logged_revision: u64,
    occluded: bool,
    error: Option<String>,
    frame_count: u64,
    waiting_logged: bool,
}
impl D2App {
    fn new(network: Network, max_ticks: u64) -> Self {
        let now = Instant::now();
        Self {
            network,
            #[cfg(feature = "f1-web")]
            web: None,
            #[cfg(feature = "f1-web")]
            web_exited_first: false,
            #[cfg(feature = "f1-web")]
            last_applied_control: None,
            #[cfg(feature = "f1-web")]
            last_f1_replica_revision: None,
            window: None,
            renderer: None,
            clock: TickClock::new(now),
            last_draw: now,
            control_ticks: 0,
            max_ticks,
            displayed_revision: 0,
            last_logged_revision: 0,
            occluded: false,
            error: None,
            frame_count: 0,
            waiting_logged: false,
        }
    }
    fn record_failure(&mut self, detail: String) {
        // Retain the initiating failure even if cleanup reveals another.
        self.error.get_or_insert(detail);
    }

    /// YT-279: the actual native application's required-Web startup path.
    /// A Web bind/readiness failure must STOP an already-prepared UDP/C1
    /// authority, not merely rely on the host's eventual Drop.
    #[cfg(feature = "f1-web")]
    fn start_required_web(
        &mut self,
        start: impl FnOnce(
            &yetlet_composable_host_research::simulation::Snapshot,
        ) -> std::io::Result<F1WebService>,
    ) -> std::io::Result<std::net::SocketAddr> {
        let Network::Host(host) = &mut self.network else {
            return Err(std::io::Error::other(
                "required Web needs an authoritative Host",
            ));
        };
        match start(&host.snapshot()) {
            Ok(web) => {
                let endpoint = web.endpoint();
                self.web = Some(web);
                Ok(endpoint)
            }
            Err(error) => {
                host.stop();
                self.record_failure(format!("F1 required Web startup failed: {error}"));
                Err(error)
            }
        }
    }

    fn fail(&mut self, event_loop: &ActiveEventLoop, detail: String) {
        eprintln!("D2_HOST_ERROR role={} reason={detail}", self.network.name());
        self.record_failure(detail);
        event_loop.exit();
    }
    /// Only the server-owned C1 State or a validated full Replica can be
    /// rendered. A Client without a received Snapshot has no game Scene.
    fn current_scene(&self) -> Option<Scene> {
        match &self.network {
            Network::Host(host) => Some(project(&host.snapshot())),
            Network::Client(client) => client.snapshot().as_deref().map(project_wire),
        }
    }
    fn schedule(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            return;
        }
        #[cfg(feature = "f1-web")]
        if let Some(detail) = required_web_failure(self.web.as_ref()) {
            // This placeholder is replaced by the worker's actual Join
            // failure after the native root exits. An earlier native error
            // remains primary instead of being overwritten by cleanup.
            if self.error.is_none() {
                self.web_exited_first = true;
            }
            self.fail(event_loop, detail.to_owned());
            return;
        }
        let now = Instant::now();
        let poll = match &mut self.network {
            Network::Host(host) => host.poll(),
            Network::Client(client) => client.poll(now),
        };
        if let Err(error) = poll {
            self.fail(event_loop, format!("UDP poll: {error}"));
            return;
        }
        let due = self.clock.advance(now);
        for _ in 0..d2_bounded_due(self.control_ticks, self.max_ticks, due) {
            if let Network::Host(host) = &mut self.network {
                #[cfg(feature = "f1-web")]
                let result = (|| -> std::io::Result<()> {
                    // One typed Web admission at most per C1 Tick, after UDP
                    // poll but before the sole authoritative reducer step.
                    let received = self.web.as_ref().and_then(F1WebService::pop_control);
                    let accepted = received.filter(|request| {
                        let valid = request.generation == host.snapshot().generation();
                        if !valid {
                            eprintln!(
                                "F1_CONTROL_REJECTED pid={} receipt={} reason=stale_generation",
                                std::process::id(),
                                request.receipt_id,
                            );
                        }
                        valid
                    });
                    host.step_with_control(accepted.map(|request| request.command))?;
                    if let Some(web) = self.web.as_ref() {
                        let published = host.snapshot();
                        if let Some(request) = accepted {
                            self.last_applied_control = Some(request.receipt_id);
                            eprintln!(
                                "F1_CONTROL_APPLIED pid={} receipt={} revision={} tick={} paused={}",
                                std::process::id(),
                                request.receipt_id,
                                published.revision(),
                                published.tick(),
                                published.paused(),
                            );
                        }
                        web.publish_snapshot(&published, self.last_applied_control)?;
                    }
                    Ok(())
                })();

                #[cfg(not(feature = "f1-web"))]
                let result = host.step();

                if let Err(error) = result {
                    self.fail(event_loop, format!("C1 owner step: {error}"));
                    return;
                }
            }
            self.control_ticks += 1;
        }
        if let Network::Client(client) = &self.network
            && let Some(replica) = client.snapshot()
            && replica.revision != self.last_logged_revision
            && replica.revision.is_multiple_of(30)
        {
            eprintln!(
                "D2_SYNC_RECV role=client revision={} tick={} p1_x={} p2_x={}",
                replica.revision, replica.tick, replica.positions[0].x, replica.positions[1].x,
            );
            self.last_logged_revision = replica.revision;
        }
        // The selected F1 client is an independent OS process; retain its
        // exact wire-provided revision, positions and paused state so one
        // pinned HTTP read can be matched to a real UDP replica publication.
        #[cfg(feature = "f1-web")]
        if let Network::Client(client) = &self.network
            && let Some(replica) = client.snapshot()
            && self.last_f1_replica_revision != Some(replica.revision)
        {
            eprintln!(
                "F1_UDP_REPLICA pid={} revision={} tick={} p1_x={} p1_y={} p2_x={} p2_y={} paused={}",
                std::process::id(),
                replica.revision,
                replica.tick,
                replica.positions[0].x,
                replica.positions[0].y,
                replica.positions[1].x,
                replica.positions[1].y,
                replica.paused,
            );
            self.last_f1_replica_revision = Some(replica.revision);
        }
        if let Network::Host(host) = &self.network {
            let authoritative = host.snapshot();
            if authoritative.revision().is_multiple_of(30)
                && authoritative.revision() != self.last_logged_revision
            {
                eprintln!(
                    "D2_SYNC_SENT role=host revision={} tick={} p1_x={} p2_x={}",
                    authoritative.revision(),
                    authoritative.tick(),
                    authoritative.positions()[0].x,
                    authoritative.positions()[1].x,
                );
                self.last_logged_revision = authoritative.revision();
            }
        }
        if !self.occluded && now.duration_since(self.last_draw) >= RENDER_STEP {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            self.last_draw = now;
        }
        if d2_tick_limit_reached(self.control_ticks, self.max_ticks) {
            if self.frame_count == 0 {
                eprintln!(
                    "D2_EXIT_NO_FRAME role={} pid={} reason=tick_limit replica_present={}",
                    self.network.name(),
                    std::process::id(),
                    self.current_scene().is_some()
                );
            }
            event_loop.exit();
            return;
        }
        let next = self.clock.next_tick_in();
        let render = RENDER_STEP.saturating_sub(now.duration_since(self.last_draw));
        let delay = c3_foreground_wake_delay(self.occluded, next, render);
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + delay));
    }
}
impl ApplicationHandler for D2App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let title = match self.network {
            Network::Host(_) => "Yetlet M28 D2 Host",
            Network::Client(_) => "Yetlet M28 D2 Client",
        };
        let attributes = Window::default_attributes()
            .with_title(title)
            .with_inner_size(LogicalSize::new(960.0, 640.0));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.fail(event_loop, format!("CreateWindow: {error}"));
                return;
            }
        };
        let renderer = match Renderer::new(Arc::clone(&window)) {
            Ok(renderer) => renderer,
            Err(error) => {
                self.fail(event_loop, format!("wgpu renderer init: {error}"));
                return;
            }
        };
        eprintln!(
            "D2_WINDOW role={} pid={} id={:?} size={:?}",
            self.network.name(),
            std::process::id(),
            window.id(),
            window.inner_size()
        );
        self.window = Some(window);
        self.renderer = Some(renderer);
        // All three actual required resources now exist: rendered Window,
        // native UDP and the separately HTTP-probed owned Web service.
        #[cfg(feature = "f1-web")]
        if let (Network::Host(host), Some(web)) = (&self.network, &self.web) {
            eprintln!(
                "F1_THREE_HOST_READY pid={} authority=network_host window=true gpu=true udp={} web={} http_response=true",
                std::process::id(),
                host.endpoint(),
                web.endpoint(),
            );
        }
        let now = Instant::now();
        self.clock.reset(now);
        self.last_draw = now.checked_sub(RENDER_STEP).unwrap_or(now);
    }
    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|win| win.id() != id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(renderer) = self.renderer.as_mut() {
                    renderer.resize(size);
                }
            }
            WindowEvent::Occluded(hidden) => {
                self.occluded = hidden;
                if !hidden {
                    self.last_draw = Instant::now();
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.occluded {
                    return;
                }
                let Some(scene) = self.current_scene() else {
                    if !self.waiting_logged {
                        eprintln!(
                            "D2_WAITING_REPLICA role=client pid={} rendering_players=false",
                            std::process::id()
                        );
                        self.waiting_logged = true;
                    }
                    return;
                };
                if let Some(renderer) = self.renderer.as_mut() {
                    match renderer.draw(&scene) {
                        Ok(true) => {
                            self.frame_count += 1;
                            self.displayed_revision = scene.revision;
                            #[cfg(feature = "f1-web")]
                            if self.web.is_some()
                                && let Network::Host(host) = &self.network
                            {
                                let authority = host.snapshot();
                                if authority.revision() == scene.revision {
                                    eprintln!(
                                        "F1_NATIVE_SCENE pid={} revision={} tick={} p1_x={} p1_y={} p2_x={} p2_y={} paused={} frame={}",
                                        std::process::id(),
                                        scene.revision,
                                        scene.tick,
                                        authority.positions()[0].x,
                                        authority.positions()[0].y,
                                        authority.positions()[1].x,
                                        authority.positions()[1].y,
                                        authority.paused(),
                                        self.frame_count,
                                    );
                                }
                            }
                        }
                        Ok(false) => (),
                        Err(error) => self.fail(event_loop, error.to_string()),
                    }
                }
            }
            _ => (),
        }
    }
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        self.schedule(event_loop);
    }
    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.renderer.take();
        self.window.take();
        self.clock.reset(Instant::now());
    }
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        // A fresh command must not be admitted after the C1 owner closes.
        // Web may remain alive briefly while the native resources unwind,
        // but this path is now a real 503 admission boundary.
        #[cfg(feature = "f1-web")]
        if let Some(web) = self.web.as_mut() {
            web.close_admission();
            // Pending commands were admitted but NOT applied by C1. Discard
            // them with bounded nonblocking receives, not a saturated queue
            // that could stall required-component stop.
            let discarded = web.discard_pending_after_close();
            eprintln!(
                "F1_WEB_ADMISSION_CLOSED pid={} pending_discarded={}",
                std::process::id(),
                discarded,
            );
        }
        let pid = std::process::id();
        let role = self.network.name();
        let scene = self.current_scene();
        match &mut self.network {
            Network::Host(host) => {
                let stats = host.counters().clone();
                let source = host.endpoint();
                let peer = host.peer();
                eprintln!(
                    "D2_FINAL role=host pid={} local={} peer={:?} connected={} tick={} rev={} p1_x={} p2_x={} frame={} received={} sent={} accepted={} rejected={} oversize={} welcomed={} snapshots={} dropped_ns={}",
                    pid,
                    source,
                    peer,
                    host.connected(),
                    scene.as_ref().map_or(0, |scene| scene.tick),
                    scene.as_ref().map_or(0, |scene| scene.revision),
                    host.snapshot().positions()[0].x,
                    host.snapshot().positions()[1].x,
                    self.frame_count,
                    stats.rx,
                    stats.tx,
                    stats.rx_accepted,
                    stats.rx_rejected,
                    stats.rx_oversize,
                    stats.tx_welcomes,
                    stats.tx_snapshots,
                    self.clock.dropped_ns(),
                );
                host.stop();
                eprintln!("D2_SOCKET_CLOSED role=host result={}", host.socket_closed());
            }
            Network::Client(client) => {
                let stats = client.counters().clone();
                eprintln!(
                    "D2_FINAL role=client pid={} local={} host={} connected={} tick={} rev={} p1_x={} p2_x={} frame={} displayed_rev={} received={} sent={} accepted={} rejected={} oversize={} hellos={} inputs={} dropped_ns={}",
                    pid,
                    client.endpoint(),
                    client.expected_host(),
                    client.connected(),
                    scene
                        .as_ref()
                        .map_or("absent".to_string(), |scene| scene.tick.to_string()),
                    scene
                        .as_ref()
                        .map_or("absent".to_string(), |scene| scene.revision.to_string()),
                    client
                        .snapshot()
                        .as_ref()
                        .map_or("absent".to_string(), |s| s.positions[0].x.to_string()),
                    client
                        .snapshot()
                        .as_ref()
                        .map_or("absent".to_string(), |s| s.positions[1].x.to_string()),
                    self.frame_count,
                    if self.frame_count == 0 {
                        "absent".to_string()
                    } else {
                        self.displayed_revision.to_string()
                    },
                    stats.rx,
                    stats.tx,
                    stats.rx_accepted,
                    stats.rx_rejected,
                    stats.rx_oversize,
                    stats.tx_hellos,
                    stats.tx_inputs,
                    self.clock.dropped_ns(),
                );
                client.stop();
                eprintln!(
                    "D2_SOCKET_CLOSED role=client result={}",
                    client.socket_closed()
                );
            }
        }
        self.renderer.take();
        self.window.take();
        eprintln!("D2_RESOURCES_RELEASED role={role} pid={pid} window=true gpu=true network=true");
    }
}
fn main() -> ResultAny<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let web_mode = {
        #[cfg(feature = "f1-web")]
        {
            matches!(args.as_slice(), [mode] if mode == "host-web")
        }
        #[cfg(not(feature = "f1-web"))]
        {
            false
        }
    };
    let network = match args.as_slice() {
        [mode] if mode == "host" => Network::Host(NetworkHost::bind()?),
        #[cfg(feature = "f1-web")]
        [mode] if mode == "host-web" => Network::Host(NetworkHost::bind()?),
        [mode, addr] if mode == "client" => {
            let address: SocketAddrV4 = addr.parse()?;
            if address.ip() != &BIND_IP || address.port() == 0 {
                return Err("D2 requires a specific 127.0.0.1:<port> host".into());
            }
            Network::Client(NetworkClient::bind(address)?)
        }
        _ => return Err("usage: d2-loopback host | client 127.0.0.1:<port> | host-web [requires f1-web feature]".into()),
    };
    let (role, endpoint) = match &network {
        Network::Host(host) => (network.name(), host.endpoint()),
        Network::Client(client) => (network.name(), client.endpoint()),
    };
    if !web_mode {
        eprintln!(
            "D2_READY role={} pid={} local={} windows=1 hosts_in_process=2 udp=true",
            role,
            std::process::id(),
            endpoint
        );
    }
    let event_loop = EventLoop::new()?;
    // F1's bounded 480-Tick host permits an HTTP command and an independent
    // UDP client to observe the same exact publication. Original D2 remains 300.
    let mut app = D2App::new(network, if web_mode { 480 } else { 300 });
    #[cfg(feature = "f1-web")]
    if web_mode {
        let endpoint = app.start_required_web(F1WebService::start_game)?;
        eprintln!(
            "F1_WEB_HTTP_READY pid={} endpoint={} http_response=true",
            std::process::id(),
            endpoint,
        );
    }
    let run_result = event_loop.run_app(&mut app);
    #[cfg(feature = "f1-web")]
    if let Some(mut web) = app.web.take() {
        let endpoint = web.endpoint();
        let observed = web.complete_within(Duration::from_secs(2));
        let was_pending = matches!(&observed, Ok(F1Completion::Pending));
        if was_pending {
            eprintln!(
                "F1_WEB_COMPLETION_PENDING pid={} endpoint={} original_join_retained=true cleanup_complete=false",
                std::process::id(),
                endpoint,
            );
            app.error.get_or_insert(
                "F1 Web cleanup deadline exceeded; original JoinHandle retained".to_owned(),
            );
        }
        // Pending is never silently detached: the same owner resumes its
        // original join. On unrecoverable in-flight work the process stays
        // pending rather than returning a false completed status.
        let completion = match observed {
            Ok(F1Completion::Complete) => Ok(()),
            Ok(F1Completion::Pending) => web.stop_and_join(),
            Err(error) => Err(error),
        };
        let joined = !web.has_pending_join();
        let worker_ok = completion.is_ok();
        eprintln!(
            "F1_WEB_COMPLETION pid={} endpoint={} joined={} worker_ok={} result={} prior_pending={}",
            std::process::id(),
            endpoint,
            joined,
            worker_ok,
            if worker_ok { "complete" } else { "failed" },
            was_pending,
        );
        if let Err(error) = &completion {
            eprintln!(
                "F1_WEB_WORKER_CAUSE pid={} cause={error}",
                std::process::id()
            );
        }
        preserve_web_worker_cause(&mut app.error, app.web_exited_first, &completion);
    }
    run_result?;
    if let Some(error) = app.error {
        return Err(error.into());
    }
    Ok(())
}

#[cfg(test)]
mod d2_regressions {
    use super::*;

    #[test]
    fn d2_client_without_snapshot_never_projects_placeholder_players() {
        let client = NetworkClient::bind(SocketAddrV4::new(BIND_IP, 9)).unwrap();
        let app = D2App::new(Network::Client(client), 2);
        assert!(app.current_scene().is_none());
        assert_eq!(app.frame_count, 0);
        assert_eq!(app.displayed_revision, 0);
    }

    #[test]
    fn d2_due_batch_never_steps_past_authoritative_tick_budget() {
        assert_eq!(d2_bounded_due(299, 300, 4), 1);
        assert_eq!(d2_bounded_due(298, 300, 4), 2);
        assert_eq!(d2_bounded_due(300, 300, 4), 0);
        assert_eq!(d2_bounded_due(303, 300, 4), 0);
        assert_eq!(d2_bounded_due(0, 300, 4), 4);
        assert_eq!(d2_bounded_due(u64::MAX, u64::MAX, 4), 0);
    }

    #[test]
    fn d2_max_control_ticks_does_not_require_rendered_frames() {
        assert!(!d2_tick_limit_reached(1, 2));
        assert!(d2_tick_limit_reached(2, 2)); // even with zero rendered frames
        assert!(d2_tick_limit_reached(3, 2)); // even if window stays occluded
    }

    #[cfg(feature = "f1-web")]
    #[test]
    fn yt279_required_web_early_completion_is_detected_and_original_join_retained() {
        let mut web = F1WebService::start().unwrap();
        assert_eq!(required_web_failure(Some(&web)), None);
        web.request_stop();
        for _ in 0..100 {
            if web.is_finished() {
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(
            required_web_failure(Some(&web)),
            Some("F1 required Web service exited unexpectedly")
        );
        assert_eq!(
            web.complete_within(Duration::from_secs(2)).unwrap(),
            F1Completion::Complete
        );
        assert_eq!(required_web_failure(None), None);
    }

    #[cfg(feature = "f1-web")]
    #[test]
    fn yt279_required_failure_preserves_original_primary_cause() {
        let host = NetworkHost::bind().unwrap();
        let mut app = D2App::new(Network::Host(host), 4);
        app.record_failure("original required UDP error".to_owned());
        app.record_failure("later Web cleanup error".to_owned());
        assert_eq!(app.error.as_deref(), Some("original required UDP error"));
        if let Network::Host(host) = &mut app.network {
            host.stop();
            assert!(host.socket_closed());
        }
    }

    #[cfg(feature = "f1-web")]
    #[test]
    fn yt279_actual_root_http_bind_failure_stops_prepared_udp_without_test_side_stop() {
        use std::net::{Ipv4Addr, TcpListener, UdpSocket};
        let host = NetworkHost::bind().unwrap();
        let original_udp = host.endpoint();
        let mut app = D2App::new(Network::Host(host), 2);
        let reserved_http = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let occupied_http = reserved_http.local_addr().unwrap();
        let error = app
            .start_required_web(|snapshot| F1WebService::start_game_on(snapshot, occupied_http))
            .err()
            .expect("root must fail when its actual Web bind is occupied");
        assert!(error.to_string().contains("F1 Web setup"), "{error}");
        assert!(app.web.is_none());
        assert!(
            app.error
                .as_deref()
                .is_some_and(|cause| cause.contains(&error.to_string())),
            "the native root must retain its original Web bind failure"
        );
        let Network::Host(host) = &app.network else {
            panic!("a Web bind error must never replace the original C1 owner");
        };
        assert!(host.socket_closed(), "native root must explicitly stop UDP");
        assert_eq!(host.snapshot().revision(), 0);
        let rebind_udp = UdpSocket::bind(original_udp).unwrap();
        drop(rebind_udp);
        drop(reserved_http);
        let rebind_http = TcpListener::bind(occupied_http).unwrap();
        drop(rebind_http);
    }

    #[cfg(feature = "f1-web")]
    #[test]
    fn yt279_foreground_worker_failure_result_keeps_real_cause_and_prior_native_error() {
        let host = NetworkHost::bind().unwrap();
        let mut app = D2App::new(Network::Host(host), 2);
        app.web_exited_first = true;
        app.record_failure(REQUIRED_WEB_EXIT_CAUSE.to_owned());
        let failed_join: std::io::Result<()> = Err(std::io::Error::other(
            "actual Web worker failed after readiness",
        ));
        assert!(preserve_web_worker_cause(
            &mut app.error,
            app.web_exited_first,
            &failed_join
        ));
        assert!(
            app.error
                .as_deref()
                .is_some_and(|message| message.contains("actual Web worker failed"))
        );
        app.error = Some("original Native UDP fault".to_owned());
        app.web_exited_first = false;
        assert!(preserve_web_worker_cause(
            &mut app.error,
            app.web_exited_first,
            &failed_join
        ));
        assert_eq!(app.error.as_deref(), Some("original Native UDP fault"));
        if let Network::Host(host) = &mut app.network {
            host.stop();
            assert!(host.socket_closed());
        }
    }
}
