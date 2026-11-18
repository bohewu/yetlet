//! C2/YT-269 opt-in native X11/Vulkan GameHost. Not a reusable SDK API.
//! Window and Surface are created only within winit's foreground callbacks.
//! Simulation::step is exclusively called from about_to_wait, never render.

use std::{
    error::Error,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::PhysicalKey,
    window::{Window, WindowId},
};

use yetlet_native_gamehost_research::{
    GameDriver, InputError, KeyResult, RENDER_STEP, Scene, TickClock, c3_foreground_wake_delay,
    map_key, project, scene_bytes,
};

type NativeResult<T> = Result<T, Box<dyn Error>>;

#[derive(Default)]
struct C3Qa {
    enabled: bool,
    zero_deferred: bool,
    zero_restored: bool,
    synthetic_outdated: bool,
    synthetic_lost: bool,
    slow_callback: bool,
}

struct GpuRenderer {
    // Drop Surface before window; Arc<Window> is also retained by the Host.
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    adapter_backend: String,
    ready: bool,
    configure_count: u64,
    zero_deferred_count: u64,
    recover_count: u64,
    render_cpu_samples: u64,
    render_cpu_ns_total: u128,
    render_cpu_ns_max: u128,
}

impl GpuRenderer {
    fn new(window: Arc<Window>) -> NativeResult<Self> {
        let size = window.inner_size();
        if size.width == 0 || size.height == 0 {
            return Err("Window surface requires nonzero size".into());
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
                label: Some("Yetlet M28 C2 private native device"),
                ..Default::default()
            }))?;
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .ok_or("No compatible surface configuration")?;
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Twin Squares WGSL"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Private Twin Squares pipeline"),
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
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
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
            multiview: None,
            cache: None,
        });
        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Two immutable scene instances"),
            contents: &[0u8; 64],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        eprintln!(
            "M28-C2 renderer_ready adapter={:?} backend={:?}",
            info.name, info.backend
        );
        Ok(Self {
            surface,
            device,
            queue,
            config,
            pipeline,
            instance_buffer,
            adapter_backend: format!("{:?}", info.backend),
            ready: true,
            configure_count: 1,
            zero_deferred_count: 0,
            recover_count: 0,
            render_cpu_samples: 0,
            render_cpu_ns_total: 0,
            render_cpu_ns_max: 0,
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>, origin: &str) {
        if size.width == 0 || size.height == 0 {
            if self.ready {
                self.ready = false;
                self.zero_deferred_count += 1;
                eprintln!("M28-C3 zero_size_deferred origin={origin}");
            }
            return; // Never configure an invalid zero-size surface.
        }
        let changed =
            !self.ready || self.config.width != size.width || self.config.height != size.height;
        self.ready = true;
        if changed {
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.configure_count += 1;
            eprintln!(
                "M28-C3 surface_configured origin={} width={} height={} total={}",
                origin, size.width, size.height, self.configure_count
            );
        }
    }

    /// Both real driver-reported Lost/Outdated errors and explicitly labeled
    /// diagnostic injections reach the same code; synthetic != real loss.
    fn recover_surface(&mut self, issue: wgpu::SurfaceError, synthetic: bool) -> NativeResult<()> {
        match issue {
            wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost => {
                if self.ready {
                    self.surface.configure(&self.device, &self.config);
                    self.configure_count += 1;
                    self.recover_count += 1;
                    eprintln!(
                        "M28-C3 surface_recovered issue={issue:?} synthetic={synthetic} total={}",
                        self.recover_count
                    );
                }
                Ok(())
            }
            wgpu::SurfaceError::Timeout => Ok(()),
            problem => Err(format!("Unrecoverable native surface: {problem}").into()),
        }
    }

    fn draw(&mut self, scene: &Scene) -> NativeResult<bool> {
        if !self.ready {
            return Ok(false);
        }
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(issue) => {
                self.recover_surface(issue, false)?;
                return Ok(false);
            }
        };
        let cpu_start = Instant::now();
        self.queue
            .write_buffer(&self.instance_buffer, 0, &scene_bytes(scene));
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("C2 draw two immutable squares"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("C2 render pass"),
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
            pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
            pass.draw(0..6, 0..2);
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        let elapsed = cpu_start.elapsed().as_nanos();
        self.render_cpu_samples += 1;
        self.render_cpu_ns_total += elapsed;
        self.render_cpu_ns_max = self.render_cpu_ns_max.max(elapsed);
        Ok(true)
    }
}

struct GameHost {
    window: Option<Arc<Window>>,
    graphics: Option<GpuRenderer>,
    driver: GameDriver,
    clock: TickClock,
    last_requested_draw: Instant,
    rendered_frames: u64,
    smoke_ticks: Option<u64>,
    qa: C3Qa,
    occluded: bool,
    resize_events: u64,
    occlusion_events: u64,
    close_events: u64,
    suspended_events: u64,
    sim_step_count: u64,
    sim_cpu_ns_total: u128,
    sim_cpu_ns_max: u128,
    max_ticks_per_wake: u32,
    started_at: Instant,
    failure: Option<String>,
}
impl GameHost {
    fn new(smoke_ticks: Option<u64>, c3_qa: bool) -> NativeResult<Self> {
        let now = Instant::now();
        Ok(Self {
            window: None,
            graphics: None,
            driver: GameDriver::new(1)?,
            clock: TickClock::new(now),
            last_requested_draw: now,
            rendered_frames: 0,
            smoke_ticks,
            qa: C3Qa {
                enabled: c3_qa,
                ..Default::default()
            },
            occluded: false,
            resize_events: 0,
            occlusion_events: 0,
            close_events: 0,
            suspended_events: 0,
            sim_step_count: 0,
            sim_cpu_ns_total: 0,
            sim_cpu_ns_max: 0,
            max_ticks_per_wake: 0,
            started_at: now,
            failure: None,
        })
    }

    fn error_exit(&mut self, event_loop: &ActiveEventLoop, message: String) {
        eprintln!("M28-C2 host_error: {message}");
        self.failure = Some(message);
        event_loop.exit();
    }

    /// One C3 bounded diagnostic sequence. The synthetic error/zero-size
    /// branches are explicit, while actual X11 resize/minimize/close comes
    /// from the separately targeted owned-window test driver.
    fn c3_qa_tick(&mut self, tick: u64) {
        if !self.qa.enabled {
            return;
        }
        if tick >= 42 && !self.qa.zero_deferred {
            self.qa.zero_deferred = true;
            if let Some(graphics) = self.graphics.as_mut() {
                graphics.resize(PhysicalSize::new(0, 0), "qa_injected_zero_size");
            }
        }
        if tick >= 48 && !self.qa.zero_restored {
            self.qa.zero_restored = true;
            if let (Some(graphics), Some(window)) = (self.graphics.as_mut(), self.window.as_ref()) {
                graphics.resize(window.inner_size(), "qa_restore_nonzero");
            }
        }
        if tick >= 82 && !self.qa.synthetic_outdated {
            self.qa.synthetic_outdated = true;
            if let Some(graphics) = self.graphics.as_mut() {
                graphics
                    .recover_surface(wgpu::SurfaceError::Outdated, true)
                    .expect("C3 QA synthetic Outdated reconfiguration");
            }
        }
        if tick >= 97 && !self.qa.synthetic_lost {
            self.qa.synthetic_lost = true;
            if let Some(graphics) = self.graphics.as_mut() {
                graphics
                    .recover_surface(wgpu::SurfaceError::Lost, true)
                    .expect("C3 QA synthetic Lost reconfiguration");
            }
        }
        if tick >= 120 && !self.qa.slow_callback {
            self.qa.slow_callback = true;
            let before = Instant::now();
            thread::sleep(Duration::from_millis(260));
            eprintln!(
                "M28-C3 qa_injected_slow_callback elapsed_ms={:.3}",
                before.elapsed().as_secs_f64() * 1000.0
            );
        }
    }

    fn schedule(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            return;
        }
        let now = Instant::now();
        let due = self.clock.advance(now);
        self.max_ticks_per_wake = self.max_ticks_per_wake.max(due);
        for _ in 0..due {
            let step_start = Instant::now();
            if let Err(error) = self.driver.step() {
                self.error_exit(event_loop, format!("Simulation step failed: {error}"));
                return;
            }
            let elapsed = step_start.elapsed().as_nanos();
            self.sim_step_count += 1;
            self.sim_cpu_ns_total += elapsed;
            self.sim_cpu_ns_max = self.sim_cpu_ns_max.max(elapsed);
        }
        self.c3_qa_tick(self.driver.snapshot().tick());
        if !self.occluded && now.duration_since(self.last_requested_draw) >= RENDER_STEP {
            if let Some(window) = self.window.as_ref() {
                window.request_redraw();
            }
            self.last_requested_draw = now;
        }
        let snapshot = self.driver.snapshot();
        if self
            .smoke_ticks
            .is_some_and(|limit| snapshot.tick() >= limit)
            && self.rendered_frames > 0
        {
            event_loop.exit();
            return;
        }
        let next_step = self.clock.next_tick_in();
        let next_draw =
            RENDER_STEP.saturating_sub(now.saturating_duration_since(self.last_requested_draw));
        let delay = c3_foreground_wake_delay(self.occluded, next_step, next_draw);
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + delay));
    }
}

impl ApplicationHandler for GameHost {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }
        let attributes = Window::default_attributes()
            .with_title("Yetlet M28 — Twin Squares (C2 private native)")
            .with_inner_size(LogicalSize::new(960.0, 640.0));
        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                self.error_exit(event_loop, format!("Window creation failed: {error}"));
                return;
            }
        };
        eprintln!(
            "M28-C2 window_created id={:?} size={:?}",
            window.id(),
            window.inner_size()
        );
        let graphics = match GpuRenderer::new(Arc::clone(&window)) {
            Ok(graphics) => graphics,
            Err(error) => {
                self.error_exit(
                    event_loop,
                    format!("Vulkan surface initialization failed: {error}"),
                );
                return;
            }
        };
        self.graphics = Some(graphics);
        self.window = Some(window);
        self.occluded = false;
        let now = Instant::now();
        self.clock.reset(now);
        self.last_requested_draw = now.checked_sub(RENDER_STEP).unwrap_or(now);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|window| window.id() != id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => {
                self.close_events += 1;
                eprintln!("M28-C3 close_requested");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.resize_events += 1;
                eprintln!("M28-C3 resized width={} height={}", size.width, size.height);
                if let Some(graphics) = self.graphics.as_mut() {
                    graphics.resize(size, "x11_resized");
                }
            }
            WindowEvent::Occluded(hidden) => {
                self.occlusion_events += 1;
                self.occluded = hidden;
                eprintln!("M28-C3 occluded hidden={hidden}");
                if !hidden {
                    // Redraw is explicitly requested when visible again; the
                    // obsolete pre-occlusion deadline must not cause a spin.
                    self.last_requested_draw = Instant::now();
                    if let Some(window) = self.window.as_ref() {
                        window.request_redraw();
                    }
                }
            }
            WindowEvent::Focused(false) => {
                if let Err(error) = self.driver.lost_focus() {
                    self.error_exit(
                        event_loop,
                        format!("Failed to release held keys: {error:?}"),
                    );
                }
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(code) = event.physical_key
                    && let Some(key) = map_key(code)
                {
                    let pressed = event.state == ElementState::Pressed;
                    match self.driver.key(key, pressed, event.repeat) {
                        Ok(KeyResult::ExitRequested) => event_loop.exit(),
                        Ok(_) => (),
                        Err(InputError::QueueFull) => self.error_exit(
                            event_loop,
                            "Bounded keyboard command admission exhausted".to_string(),
                        ),
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                if self.occluded {
                    return;
                }
                let scene = project(&self.driver.snapshot());
                if let Some(graphics) = self.graphics.as_mut() {
                    match graphics.draw(&scene) {
                        Ok(true) => self.rendered_frames += 1,
                        Ok(false) => (),
                        Err(error) => self.error_exit(event_loop, error.to_string()),
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
        self.suspended_events += 1;
        eprintln!("M28-C3 suspended_foreground_release");
        // The owned surface and device are destroyed before the Window.
        self.graphics.take();
        self.window.take();
        self.clock.reset(Instant::now());
    }

    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        let renderer_backend = self
            .graphics
            .as_ref()
            .map(|gpu| gpu.adapter_backend.clone());
        if let Some(graphics) = self.graphics.as_ref() {
            eprintln!(
                "M28-C3 gpu_metrics configured={} deferred_zero={} recovered={} cpu_frames={} cpu_avg_us={:.3} cpu_max_us={:.3}",
                graphics.configure_count,
                graphics.zero_deferred_count,
                graphics.recover_count,
                graphics.render_cpu_samples,
                if graphics.render_cpu_samples == 0 {
                    0.0
                } else {
                    graphics.render_cpu_ns_total as f64
                        / graphics.render_cpu_samples as f64
                        / 1000.0
                },
                graphics.render_cpu_ns_max as f64 / 1000.0,
            );
        }
        self.graphics.take();
        self.window.take();
        let snapshot = self.driver.snapshot();
        eprintln!(
            "M28-C2 exiting tick={} active_ticks={} revision={} positions={:?} paused={} rendered_frames={} dropped_debt_ns={} backend={:?}",
            snapshot.tick(),
            snapshot.active_ticks(),
            snapshot.revision(),
            snapshot.positions(),
            snapshot.paused(),
            self.rendered_frames,
            self.clock.dropped_ns(),
            renderer_backend,
        );
        eprintln!(
            "M28-C3 host_metrics control_ticks={} rendered_frames={} resize_events={} occlusion_events={} close_events={} suspended_events={} max_ticks_per_wake={} dropped_debt_ns={} sim_cpu_avg_us={:.3} sim_cpu_max_us={:.3} elapsed_ms={:.3} qa_zero={} qa_restore={} qa_outdated={} qa_lost={} qa_slow={}",
            snapshot.tick(),
            self.rendered_frames,
            self.resize_events,
            self.occlusion_events,
            self.close_events,
            self.suspended_events,
            self.max_ticks_per_wake,
            self.clock.dropped_ns(),
            if self.sim_step_count == 0 {
                0.0
            } else {
                self.sim_cpu_ns_total as f64 / self.sim_step_count as f64 / 1000.0
            },
            self.sim_cpu_ns_max as f64 / 1000.0,
            self.started_at.elapsed().as_secs_f64() * 1000.0,
            self.qa.zero_deferred,
            self.qa.zero_restored,
            self.qa.synthetic_outdated,
            self.qa.synthetic_lost,
            self.qa.slow_callback,
        );
        eprintln!("M28-C3 owned_renderer_and_window_dropped");
    }
}

fn main() -> NativeResult<()> {
    let mut args = std::env::args().skip(1);
    let (smoke_ticks, c3_qa) = match args.next() {
        None => (None, false),
        Some(option) if option == "--smoke-ticks" || option == "--c3-qa-ticks" => {
            let count: u64 = args
                .next()
                .ok_or("Missing tick count for selected mode")?
                .parse()?;
            if count == 0 || count > 600 {
                return Err("Bounded tick count must be 1..=600".into());
            }
            (Some(count), option == "--c3-qa-ticks")
        }
        Some(_) => return Err("Usage: native-gamehost [--smoke-ticks N|--c3-qa-ticks N]".into()),
    };
    if args.next().is_some() {
        return Err("Unexpected extra CLI argument".into());
    }
    // The foreground caller owns winit's EventLoop, never each Host separately.
    let event_loop = EventLoop::new()?;
    let mut host = GameHost::new(smoke_ticks, c3_qa)?;
    event_loop.run_app(&mut host)?;
    if let Some(error) = host.failure {
        return Err(error.into());
    }
    Ok(())
}
