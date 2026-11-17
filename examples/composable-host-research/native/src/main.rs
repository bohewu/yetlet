//! C2/YT-269 opt-in native X11/Vulkan GameHost. Not a reusable SDK API.
//! Window and Surface are created only within winit's foreground callbacks.
//! Simulation::step is exclusively called from about_to_wait, never render.

use std::{error::Error, sync::Arc, time::Instant};

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
    GameDriver, InputError, KeyResult, RENDER_STEP, Scene, TickClock, map_key, project, scene_bytes,
};

type NativeResult<T> = Result<T, Box<dyn Error>>;

struct GpuRenderer {
    // Drop Surface before window; Arc<Window> is also retained by the Host.
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    instance_buffer: wgpu::Buffer,
    adapter_backend: String,
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
        })
    }

    fn resize(&mut self, size: PhysicalSize<u32>) {
        if size.width == 0 || size.height == 0 {
            return;
        }
        self.config.width = size.width;
        self.config.height = size.height;
        self.surface.configure(&self.device, &self.config);
    }

    fn draw(&mut self, scene: &Scene) -> NativeResult<bool> {
        if self.config.width == 0 || self.config.height == 0 {
            return Ok(false);
        }
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Timeout) => return Ok(false),
            Err(wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost) => {
                self.surface.configure(&self.device, &self.config);
                return Ok(false);
            }
            Err(error) => return Err(format!("Failed to acquire native frame: {error}").into()),
        };
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
    failure: Option<String>,
}
impl GameHost {
    fn new(smoke_ticks: Option<u64>) -> NativeResult<Self> {
        let now = Instant::now();
        Ok(Self {
            window: None,
            graphics: None,
            driver: GameDriver::new(1)?,
            clock: TickClock::new(now),
            last_requested_draw: now,
            rendered_frames: 0,
            smoke_ticks,
            failure: None,
        })
    }

    fn error_exit(&mut self, event_loop: &ActiveEventLoop, message: String) {
        eprintln!("M28-C2 host_error: {message}");
        self.failure = Some(message);
        event_loop.exit();
    }

    fn schedule(&mut self, event_loop: &ActiveEventLoop) {
        let Some(window) = self.window.as_ref() else {
            return;
        };
        let now = Instant::now();
        let due = self.clock.advance(now);
        for _ in 0..due {
            if let Err(error) = self.driver.step() {
                self.error_exit(event_loop, format!("Simulation step failed: {error}"));
                return;
            }
        }

        if now.duration_since(self.last_requested_draw) >= RENDER_STEP {
            window.request_redraw();
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
        event_loop.set_control_flow(ControlFlow::WaitUntil(now + next_step.min(next_draw)));
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
        let now = Instant::now();
        self.clock.reset(now);
        self.last_requested_draw = now.checked_sub(RENDER_STEP).unwrap_or(now);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        if self.window.as_ref().is_none_or(|window| window.id() != id) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let Some(graphics) = self.graphics.as_mut() {
                    graphics.resize(size);
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
    }
}

fn main() -> NativeResult<()> {
    let mut args = std::env::args().skip(1);
    let smoke_ticks = match args.next() {
        None => None,
        Some(arg) if arg == "--smoke-ticks" => {
            let count: u64 = args
                .next()
                .ok_or("Missing count for --smoke-ticks")?
                .parse()?;
            if count == 0 || count > 600 {
                return Err("Smoke ticks must be 1..=600".into());
            }
            Some(count)
        }
        Some(_) => return Err("Usage: native-gamehost [--smoke-ticks 1..=600]".into()),
    };
    if args.next().is_some() {
        return Err("Unexpected extra CLI argument".into());
    }
    // The foreground caller owns winit's EventLoop, never each Host separately.
    let event_loop = EventLoop::new()?;
    let mut host = GameHost::new(smoke_ticks)?;
    event_loop.run_app(&mut host)?;
    if let Some(error) = host.failure {
        return Err(error.into());
    }
    Ok(())
}
