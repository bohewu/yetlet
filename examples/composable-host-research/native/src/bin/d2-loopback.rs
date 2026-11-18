//! YT-272 D2 — two real independently launched *applications*, each one
//! foreground winit GameHost plus a selected NetworkHost or NetworkClient.
//! No Host-per-process/thread, generic Transport API or network exposure.
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

struct D2App {
    network: Network,
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
    fn fail(&mut self, event_loop: &ActiveEventLoop, detail: String) {
        eprintln!("D2_HOST_ERROR role={} reason={detail}", self.network.name());
        self.error = Some(detail);
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
            if let Network::Host(host) = &mut self.network
                && let Err(error) = host.step()
            {
                self.fail(event_loop, format!("C1 owner step: {error}"));
                return;
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
    let network = match args.as_slice() {
        [mode] if mode == "host" => Network::Host(NetworkHost::bind()?),
        [mode, addr] if mode == "client" => {
            let address: SocketAddrV4 = addr.parse()?;
            if address.ip() != &BIND_IP || address.port() == 0 {
                return Err("D2 requires a specific 127.0.0.1:<port> host".into());
            }
            Network::Client(NetworkClient::bind(address)?)
        }
        _ => return Err("usage: d2-loopback host | client 127.0.0.1:<port>".into()),
    };
    let (role, endpoint) = match &network {
        Network::Host(host) => (network.name(), host.endpoint()),
        Network::Client(client) => (network.name(), client.endpoint()),
    };
    eprintln!(
        "D2_READY role={} pid={} local={} windows=1 hosts_in_process=2 udp=true",
        role,
        std::process::id(),
        endpoint
    );
    let mut app = D2App::new(network, 300);
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut app)?;
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
}
