//! Concrete, independently selected Vulkan Surface renderer (YT-282-D2).
//!
//! The caller supplies a safe surface target, physical size and scheduling.
//! No Winit, domain, transport, runtime, worker or universal Renderer trait.
//! Construction prepares a device/pipeline but does not configure or present.
//! Call `resize` with the current physical size before drawing, including zero.
//! Release this renderer before closing its target's WindowSlot.

use std::{
    error::Error,
    fmt, io,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    sync::{Arc, mpsc},
};

/// Exact selected WGPU API, including safe SurfaceTarget and adapter metadata.
pub use wgpu;

/// Surface lifecycle, independent of OS Window readiness and present receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfacePhase {
    Unconfigured,
    DeferredZeroSize,
    Configured,
    OutOfMemory,
    Failed,
    Closed,
}

/// Completed API calls, not redraw requests, GPU completion or physical scanout.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RenderCounters {
    pub configure_calls: u64,
    pub acquire_calls: u64,
    pub render_passes: u64,
    pub draw_calls: u64,
    pub submissions: u64,
    /// Incremented only after a real SurfaceTexture::present() returns.
    pub present_calls: u64,
    pub zero_size_deferrals: u64,
    pub recovery_requests: u64,
    pub timeouts: u64,
}

/// One draw attempt. Deferred/retry outcomes never count as a presentation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawReceipt {
    Presented {
        present_calls: u64,
        suboptimal: bool,
    },
    Deferred(SurfacePhase),
    /// Timeout: retry later. Lost/Outdated: resize to live physical size first.
    Retry(wgpu::SurfaceError),
}

/// Maximum rectangle instances accepted in a single frame.
pub const MAX_RECT_INSTANCES: usize = 64;
const RECT_STRIDE: usize = 8 * size_of::<f32>();

/// Axis-aligned rectangle in normalized device coordinates. Positive Y is up.
/// Both derived edges on each axis (center ± half-extent) must be finite.
/// Color channels are in [0, 1]; the rectangle pipeline replaces target pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RectInstance {
    pub center: [f32; 2],
    pub half_extent: [f32; 2],
    pub rgba: [f32; 4],
}

fn encode_rects(
    rects: &[RectInstance],
    bytes: &mut [u8; MAX_RECT_INSTANCES * RECT_STRIDE],
) -> Result<usize, RenderError> {
    if rects.len() > MAX_RECT_INSTANCES {
        return Err(RenderError::TooManyRectangles);
    }
    for (rect, encoded) in rects.iter().zip(bytes.as_chunks_mut::<RECT_STRIDE>().0) {
        if !rect.center.iter().all(|v| v.is_finite())
            || !rect.half_extent.iter().all(|v| v.is_finite() && *v > 0.0)
            || !rect
                .center
                .into_iter()
                .zip(rect.half_extent)
                .all(|(center, half)| (center - half).is_finite() && (center + half).is_finite())
            || !rect.rgba.iter().all(|v| (0.0..=1.0).contains(v))
        {
            return Err(RenderError::InvalidRectangle);
        }
        let channels = rect
            .center
            .into_iter()
            .chain(rect.half_extent)
            .chain(rect.rgba);
        for (channel, slot) in channels.zip(encoded.as_chunks_mut::<{ size_of::<f32>() }>().0) {
            *slot = channel.to_ne_bytes();
        }
    }
    Ok(rects.len() * RECT_STRIDE)
}

enum Frame<'a> {
    Triangle,
    Rectangles(&'a [u8]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderError {
    Closed,
    OutOfMemory,
    SurfaceFailed,
    /// The private Device reported loss; inspect first_device_loss.
    DeviceLost,
    /// A pinned WGPU fatal diagnostic; no typed OOM cause was available.
    Fatal,
    /// Frame resources were released, but WGPU discard did not succeed.
    FrameCleanupFailed,
    SizeExceedsDeviceLimit,
    TooManyRectangles,
    InvalidRectangle,
}
impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Surface renderer: {self:?}")
    }
}
impl Error for RenderError {}

/// Original diagnostic from one exact WGPU27 fatal-call boundary.
/// This is not a typed driver error or proof that an OOM occurred.
#[derive(Debug)]
pub struct FatalError {
    pub operation: &'static str,
    pub diagnostic: String,
}
impl fmt::Display for FatalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.diagnostic)
    }
}
impl Error for FatalError {}

/// Original reason and diagnostic delivered by the private Device's loss callback.
/// Loss is terminal; this does not establish driver recovery or a typed OOM.
#[derive(Debug)]
pub struct DeviceLostError {
    pub reason: wgpu::DeviceLostReason,
    pub diagnostic: String,
}
impl fmt::Display for DeviceLostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Device lost ({:?}): {}", self.reason, self.diagnostic)
    }
}
impl Error for DeviceLostError {}

// Only the pinned library's known string diagnostics are converted. Unknown
// panics resume unchanged. AssertUnwindSafe covers just one foreign call: after
// failure we only dispose of its resources; the renderer is never reused.
// The normal panic hook still runs. Abort/FFI crashes cannot be intercepted.
fn wgpu_fatal<T>(operation: &'static str, call: impl FnOnce() -> T) -> Result<T, FatalError> {
    match catch_unwind(AssertUnwindSafe(call)) {
        Ok(value) => Ok(value),
        Err(payload) => {
            let diagnostic = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied());
            let expected = match operation {
                "Texture::create_view" => "wgpu error: ",
                "Surface::discard_texture" => "Error in Surface::discard_texture:",
                "Surface::present" => "Error in Surface::present:",
                _ => unreachable!("private pinned WGPU boundary"),
            };
            if let Some(diagnostic) = diagnostic.filter(|text| text.starts_with(expected)) {
                return Err(FatalError {
                    operation,
                    diagnostic: diagnostic.to_owned(),
                });
            }
            resume_unwind(payload)
        }
    }
}

// WGPU core27 Texture::destroy snatches a Surface texture even on an invalid
// Device. This releases HAL semaphore Arcs before Surface::drop/unconfigure.
// Merely catching discard would leave them acquired and panic in Vulkan teardown.
// Destroy precedes Drop, outside any active unwind, so discard cannot double-panic
// over a primary error. Its failure is retained separately, never called success.
fn release_failed_frame<T>(
    frame: T,
    destroy: impl FnOnce(&T),
    discard: impl FnOnce(T),
) -> Result<(), FatalError> {
    destroy(&frame);
    wgpu_fatal("Surface::discard_texture", || discard(frame))
}

// Each private Device has its own bounded first-error mailbox. The WGPU callback
// only tries to enqueue the original error: no wait, formatting, panic or GPU
// reentry. Keep the first error in Lifecycle after receiving it, including close.
struct DeviceErrors {
    errors: mpsc::Receiver<wgpu::Error>,
    loss: mpsc::Receiver<DeviceLostError>,
}
impl DeviceErrors {
    fn new() -> (
        Self,
        Arc<dyn wgpu::UncapturedErrorHandler>,
        impl Fn(wgpu::DeviceLostReason, String) + Send + 'static,
    ) {
        let (sender, receiver) = mpsc::sync_channel(1);
        let handler = Arc::new(move |error| {
            let _ = sender.try_send(error);
        });
        // WGPU27 routes DeviceLost separately from uncaptured errors, even
        // before the first successful Surface configuration installs its sink.
        let (loss_sender, loss_receiver) = mpsc::sync_channel(1);
        let loss_callback = move |reason, diagnostic| {
            let _ = loss_sender.try_send(DeviceLostError { reason, diagnostic });
        };
        (
            Self {
                errors: receiver,
                loss: loss_receiver,
            },
            handler,
            loss_callback,
        )
    }
}

struct Lifecycle {
    phase: SurfacePhase,
    size: [u32; 2],
    counters: RenderCounters,
    first_device_error: Option<Arc<wgpu::Error>>,
    first_device_loss: Option<Arc<DeviceLostError>>,
    first_fatal_error: Option<Arc<FatalError>>,
    frame_cleanup_error: Option<Arc<FatalError>>,
}
impl Lifecycle {
    fn new(width: u32, height: u32) -> Self {
        Self {
            phase: if width == 0 || height == 0 {
                SurfacePhase::DeferredZeroSize
            } else {
                SurfacePhase::Unconfigured
            },
            size: [width, height],
            counters: RenderCounters::default(),
            first_device_error: None,
            first_device_loss: None,
            first_fatal_error: None,
            frame_cleanup_error: None,
        }
    }

    fn check_device_errors(&mut self, errors: &DeviceErrors) -> Result<(), RenderError> {
        self.check_live()?;
        if self.first_device_error.is_none()
            && let Ok(error) = errors.errors.try_recv()
        {
            self.first_device_error = Some(Arc::new(error));
        }
        if let Some(error) = &self.first_device_error {
            // Validation (including configure's InvalidSurface) and Internal
            // remain terminal failures too; they are never silently ignored.
            self.phase = match error.as_ref() {
                wgpu::Error::OutOfMemory { .. } => SurfacePhase::OutOfMemory,
                _ => SurfacePhase::Failed,
            };
        } else if let Ok(loss) = errors.loss.try_recv() {
            // Check typed errors first: an OOM and loss reported by the same
            // operation must retain the typed OOM. check_live above prevents
            // any later event from replacing an already retained primary.
            self.first_device_loss = Some(Arc::new(loss));
            self.phase = SurfacePhase::Failed;
        }
        self.check_live()
    }

    fn device_call<T>(
        &mut self,
        errors: &DeviceErrors,
        call: impl FnOnce() -> T,
    ) -> Result<T, RenderError> {
        self.check_device_errors(errors)?;
        let result = call();
        // Check both callbacks after the actual configure/acquire boundary.
        // Configure loss must not admit acquisition on an unconfigured Surface;
        // acquisition OOM otherwise returns generic Surface::Other.
        self.check_device_errors(errors)?;
        Ok(result)
    }

    fn configure(
        &mut self,
        errors: &DeviceErrors,
        call: impl FnOnce(),
    ) -> Result<bool, RenderError> {
        self.device_call(errors, call)?;
        self.counters.configure_calls += 1;
        self.phase = SurfacePhase::Configured;
        Ok(true)
    }

    fn fatal(&mut self, error: FatalError) -> RenderError {
        if self.first_device_error.is_none()
            && self.first_device_loss.is_none()
            && self.first_fatal_error.is_none()
        {
            self.first_fatal_error = Some(Arc::new(error));
            self.phase = SurfacePhase::Failed;
        }
        self.check_live().expect_err("fatal renderer is terminal")
    }

    fn release_frame(&mut self, frame: wgpu::SurfaceTexture) {
        self.note_frame_release(release_failed_frame(frame, |f| f.texture.destroy(), drop));
    }

    fn note_frame_release(&mut self, result: Result<(), FatalError>) {
        if let Err(error) = result {
            self.frame_cleanup_error.get_or_insert(Arc::new(error));
        }
    }

    fn present(&mut self, errors: &DeviceErrors, call: impl FnOnce()) -> Result<(), RenderError> {
        let result = wgpu_fatal("Surface::present", call);
        // Preserve a typed callback cause if the call also reported one before
        // its fatal diagnostic. The consumed frame sets presented=true first.
        self.check_device_errors(errors)?;
        result.map_err(|error| self.fatal(error))?;
        self.counters.present_calls += 1;
        Ok(())
    }

    fn closed(&mut self) -> Result<(), RenderError> {
        self.phase = SurfacePhase::Closed;
        if self.frame_cleanup_error.is_some() {
            Err(RenderError::FrameCleanupFailed)
        } else {
            Ok(())
        }
    }

    fn check_live(&self) -> Result<(), RenderError> {
        match self.phase {
            SurfacePhase::Closed => Err(RenderError::Closed),
            SurfacePhase::OutOfMemory => Err(RenderError::OutOfMemory),
            SurfacePhase::Failed if self.first_fatal_error.is_some() => Err(RenderError::Fatal),
            SurfacePhase::Failed if self.first_device_loss.is_some() => {
                Err(RenderError::DeviceLost)
            }
            SurfacePhase::Failed => Err(RenderError::SurfaceFailed),
            _ => Ok(()),
        }
    }

    // Returns whether an actual configure is allowed; zero dominates limits.
    fn resize(&mut self, width: u32, height: u32, limit: u32) -> Result<bool, RenderError> {
        self.check_live()?;
        if width == 0 || height == 0 {
            self.size = [width, height];
            self.phase = SurfacePhase::DeferredZeroSize;
            self.counters.zero_size_deferrals += 1;
            return Ok(false);
        }
        if width > limit || height > limit {
            return Err(RenderError::SizeExceedsDeviceLimit);
        }
        self.size = [width, height];
        self.phase = SurfacePhase::Unconfigured;
        Ok(true)
    }

    fn surface_error(&mut self, issue: wgpu::SurfaceError) -> Result<DrawReceipt, RenderError> {
        self.check_live()?;
        match issue {
            wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => {
                if self.phase != SurfacePhase::DeferredZeroSize {
                    self.phase = SurfacePhase::Unconfigured;
                }
                self.counters.recovery_requests += 1;
            }
            wgpu::SurfaceError::Timeout => self.counters.timeouts += 1,
            wgpu::SurfaceError::OutOfMemory => {
                self.phase = SurfacePhase::OutOfMemory;
                return Err(RenderError::OutOfMemory);
            }
            wgpu::SurfaceError::Other => {
                self.phase = SurfacePhase::Failed;
                return Err(RenderError::SurfaceFailed);
            }
        }
        Ok(DrawReceipt::Retry(issue))
    }
}

// Rust drops fields in declaration order: Surface (and its owned handle)
// before other GPU objects; Instance last. No externally outstanding frames.
struct Gpu<'window> {
    surface: wgpu::Surface<'window>,
    pipeline: wgpu::RenderPipeline,
    rect_pipeline: wgpu::RenderPipeline,
    rect_buffer: wgpu::Buffer,
    queue: wgpu::Queue,
    device: wgpu::Device,
    _instance: wgpu::Instance,
    config: wgpu::SurfaceConfiguration,
}

/// Owns one Instance/Surface/Device/Queue and its pipelines, with no Window-system choice.
/// An owned Arc target is retained by WGPU's safe Surface until close/drop.
pub struct SurfaceRenderer<'window> {
    gpu: Option<Gpu<'window>>,
    lifecycle: Lifecycle,
    device_errors: DeviceErrors,
    adapter_info: wgpu::AdapterInfo,
}

impl<'window> SurfaceRenderer<'window> {
    /// Prepare a Vulkan Surface/device and the triangle and rectangle pipelines.
    /// No configure/acquire/draw/present occurs, even for positive dimensions.
    /// This async method needs no particular executor. The caller controls its
    /// initialization budget; native driver calls themselves are not cancellable.
    ///
    /// # Errors
    /// Surface creation, compatible adapter/device or configuration unavailable.
    pub async fn new(
        target: impl Into<wgpu::SurfaceTarget<'window>>,
        width: u32,
        height: u32,
    ) -> io::Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });
        let surface = instance.create_surface(target).map_err(io::Error::other)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(io::Error::other)?;
        let adapter_info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Yetlet D2 Surface device"),
                ..Default::default()
            })
            .await
            .map_err(io::Error::other)?;
        let (device_errors, handler, loss_callback) = DeviceErrors::new();
        device.on_uncaptured_error(handler);
        device.set_device_lost_callback(loss_callback);
        // Placeholder dimensions select only capabilities. Never configure 0x0.
        let mut config = surface
            .get_default_config(&adapter, 1, 1)
            .ok_or_else(|| io::Error::other("no compatible Surface configuration"))?;
        config.present_mode = wgpu::PresentMode::Fifo;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Yetlet D2 neutral triangle"),
            source: wgpu::ShaderSource::Wgsl(include_str!("triangle.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Yetlet D2 triangle pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
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
        let rect_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Yetlet rectangle shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("rectangles.wgsl").into()),
        });
        let rect_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Yetlet rectangle pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &rect_shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: RECT_STRIDE as u64,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
                }],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &rect_shader,
                entry_point: Some("fragment"),
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
        let rect_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Yetlet rectangle instances"),
            size: (MAX_RECT_INSTANCES * RECT_STRIDE) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let mut lifecycle = Lifecycle::new(width, height);
        if lifecycle.check_device_errors(&device_errors).is_err() {
            return Err(match lifecycle.first_device_error {
                Some(error) => io::Error::other(error),
                None => io::Error::other(
                    lifecycle
                        .first_device_loss
                        .expect("captured construction loss"),
                ),
            });
        }
        Ok(Self {
            gpu: Some(Gpu {
                surface,
                pipeline,
                rect_pipeline,
                rect_buffer,
                queue,
                device,
                _instance: instance,
                config,
            }),
            lifecycle,
            device_errors,
            adapter_info,
        })
    }

    pub fn phase(&self) -> SurfacePhase {
        self.lifecycle.phase
    }
    pub fn counters(&self) -> RenderCounters {
        self.lifecycle.counters
    }
    pub fn adapter_info(&self) -> &wgpu::AdapterInfo {
        &self.adapter_info
    }

    /// First captured Device error, including its original WGPU source chain.
    /// Clone the Arc to retain the cause in caller/Composition failure storage.
    /// Remains available after terminal rejection and explicit close.
    pub fn first_device_error(&self) -> Option<&Arc<wgpu::Error>> {
        self.lifecycle.first_device_error.as_ref()
    }

    /// First Device loss when no earlier primary or same-call typed error won.
    /// Original callback reason/diagnostic remains available after close.
    pub fn first_device_loss(&self) -> Option<&Arc<DeviceLostError>> {
        self.lifecycle.first_device_loss.as_ref()
    }

    /// First fatal diagnostic when WGPU provided no typed Device cause.
    /// Remains available after close, for caller/Composition failure storage.
    pub fn first_fatal_error(&self) -> Option<&Arc<FatalError>> {
        self.lifecycle.first_fatal_error.as_ref()
    }

    /// Secondary frame-discard failure. Never replaces the first draw cause.
    /// Explicit close reports FrameCleanupFailed even after handles are released.
    pub fn frame_cleanup_error(&self) -> Option<&Arc<FatalError>> {
        self.lifecycle.frame_cleanup_error.as_ref()
    }

    /// Forward the latest physical size, including zero on either axis.
    /// Positive sizes always reconfigure (also the Lost/Outdated recovery path).
    /// Returns false for zero: no configure or subsequent acquire is allowed.
    ///
    /// # Errors
    /// Terminal/closed renderer, or positive dimensions beyond device limits.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<bool, RenderError> {
        self.lifecycle.check_device_errors(&self.device_errors)?;
        let gpu = self.gpu.as_mut().expect("live renderer owns GPU");
        if !self
            .lifecycle
            .resize(width, height, gpu.device.limits().max_texture_dimension_2d)?
        {
            return Ok(false);
        }
        gpu.config.width = width;
        gpu.config.height = height;
        self.lifecycle.configure(&self.device_errors, || {
            gpu.surface.configure(&gpu.device, &gpu.config);
        })
    }

    /// At most one acquire and one real triangle pass/draw/submit/present.
    /// A retry needs a caller-scheduled future callback, never an internal loop.
    /// Lost/Outdated and suboptimal frames require resize to live size first.
    ///
    /// # Errors
    /// OutOfMemory and Other acquisition errors are terminal until close/drop.
    /// Device OOM is terminal OutOfMemory; Validation/Internal are SurfaceFailed.
    /// Inspect first_device_error for the original cause, rather than retrying.
    pub fn draw(&mut self) -> Result<DrawReceipt, RenderError> {
        self.draw_frame(Frame::Triangle)
    }

    /// Clear the surface and present up to [`MAX_RECT_INSTANCES`] rectangles
    /// using one instanced draw. An empty slice clears without drawing geometry.
    /// The same lifecycle, retry and error rules as [`Self::draw`] apply.
    ///
    /// # Errors
    /// Invalid values, non-finite derived edges or counts above the limit are
    /// rejected before any upload or acquisition.
    pub fn draw_rects(&mut self, rects: &[RectInstance]) -> Result<DrawReceipt, RenderError> {
        self.lifecycle.check_device_errors(&self.device_errors)?;
        let mut bytes = [0; MAX_RECT_INSTANCES * RECT_STRIDE];
        let len = encode_rects(rects, &mut bytes)?;
        self.draw_frame(Frame::Rectangles(&bytes[..len]))
    }

    fn draw_frame(&mut self, scene: Frame<'_>) -> Result<DrawReceipt, RenderError> {
        self.lifecycle.check_device_errors(&self.device_errors)?;
        if self.phase() != SurfacePhase::Configured {
            return Ok(DrawReceipt::Deferred(self.phase()));
        }
        let gpu = self.gpu.as_mut().expect("live renderer owns GPU");
        self.lifecycle.counters.acquire_calls += 1;
        let acquired = gpu.surface.get_current_texture();
        if let Err(error) = self.lifecycle.check_device_errors(&self.device_errors) {
            if let Ok(frame) = acquired {
                self.lifecycle.release_frame(frame);
            }
            return Err(error);
        }
        let frame = match acquired {
            Ok(frame) => frame,
            Err(issue) => return self.lifecycle.surface_error(issue),
        };
        let suboptimal = frame.suboptimal;
        // Keep the frame outside the view-call unwind. Dispose explicitly after
        // recording the primary diagnostic, before any Surface/Window teardown.
        let rendered = (|| {
            let view = wgpu_fatal("Texture::create_view", || {
                frame.texture.create_view(&Default::default())
            })
            .map_err(|error| self.lifecycle.fatal(error))?;
            if let Frame::Rectangles(bytes) = scene
                && !bytes.is_empty()
            {
                self.lifecycle.device_call(&self.device_errors, || {
                    gpu.queue.write_buffer(&gpu.rect_buffer, 0, bytes);
                })?;
            }
            let mut encoder = gpu
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Yetlet surface encoder"),
                });
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Yetlet surface pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.035,
                                g: 0.045,
                                b: 0.065,
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
                self.lifecycle.counters.render_passes += 1;
                match scene {
                    Frame::Triangle => {
                        pass.set_pipeline(&gpu.pipeline);
                        pass.draw(0..3, 0..1);
                        self.lifecycle.counters.draw_calls += 1;
                    }
                    Frame::Rectangles(bytes) if !bytes.is_empty() => {
                        pass.set_pipeline(&gpu.rect_pipeline);
                        pass.set_vertex_buffer(0, gpu.rect_buffer.slice(..bytes.len() as u64));
                        pass.draw(0..6, 0..(bytes.len() / RECT_STRIDE) as u32);
                        self.lifecycle.counters.draw_calls += 1;
                    }
                    Frame::Rectangles(_) => {}
                }
            }
            self.lifecycle.check_device_errors(&self.device_errors)?;
            let commands = self
                .lifecycle
                .device_call(&self.device_errors, || encoder.finish())?;
            self.lifecycle
                .device_call(&self.device_errors, || gpu.queue.submit([commands]))?;
            self.lifecycle.counters.submissions += 1;
            Ok::<(), RenderError>(())
        })();
        if let Err(error) = rendered {
            self.lifecycle.release_frame(frame);
            return Err(error);
        }
        // present sets presented=true before its fatal inner call, so unwind
        // cannot trigger discard. Retain a Texture handle solely to destroy any
        // still-acquired raw texture if present failed before taking it in core.
        let texture = frame.texture.clone();
        let presented = self
            .lifecycle
            .present(&self.device_errors, || frame.present());
        if presented.is_err() {
            texture.destroy();
        }
        presented?;
        if suboptimal {
            self.lifecycle.phase = SurfacePhase::Unconfigured;
        }
        Ok(DrawReceipt::Presented {
            present_calls: self.lifecycle.counters.present_calls,
            suboptimal,
        })
    }

    /// Idempotently release Surface and its owned target before Window close.
    /// Keeps counters/adapter metadata; does not stop a domain or EventLoop.
    ///
    /// # Errors
    /// FrameCleanupFailed if a prior discard failed. Handles are still released;
    /// callers must retain that diagnostic and must not claim complete cleanup.
    pub fn close(&mut self) -> Result<(), RenderError> {
        drop(self.gpu.take());
        self.lifecycle.closed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RECT: RectInstance = RectInstance {
        center: [-0.25, 0.5],
        half_extent: [0.1, 0.2],
        rgba: [0.0, 0.4, 0.75, 1.0],
    };

    #[test]
    fn rectangle_encoding_matches_instance_layout_and_is_bounded() {
        let mut bytes = [0; MAX_RECT_INSTANCES * RECT_STRIDE];
        assert_eq!(encode_rects(&[], &mut bytes), Ok(0));
        let mut rects = [RECT; MAX_RECT_INSTANCES];
        rects[1].center = [0.75, -0.5];
        assert_eq!(encode_rects(&rects, &mut bytes), Ok(bytes.len()));
        let floats: Vec<_> = bytes[..2 * RECT_STRIDE]
            .as_chunks::<{ size_of::<f32>() }>()
            .0
            .iter()
            .copied()
            .map(f32::from_ne_bytes)
            .collect();
        assert_eq!(
            floats,
            [
                -0.25, 0.5, 0.1, 0.2, 0.0, 0.4, 0.75, 1.0, 0.75, -0.5, 0.1, 0.2, 0.0, 0.4, 0.75,
                1.0
            ]
        );
        assert_eq!(
            encode_rects(&[RECT; MAX_RECT_INSTANCES + 1], &mut bytes),
            Err(RenderError::TooManyRectangles)
        );
    }

    #[test]
    fn rectangle_validation_rejects_invalid_values_in_later_instances() {
        let mut bytes = [0; MAX_RECT_INSTANCES * RECT_STRIDE];
        for invalid in [
            RectInstance {
                center: [f32::NAN, 0.0],
                ..RECT
            },
            RectInstance {
                center: [0.0, f32::INFINITY],
                ..RECT
            },
            RectInstance {
                half_extent: [f32::INFINITY, 0.2],
                ..RECT
            },
            RectInstance {
                half_extent: [0.0, 0.2],
                ..RECT
            },
            RectInstance {
                half_extent: [0.1, -0.2],
                ..RECT
            },
            RectInstance {
                center: [f32::MAX, 0.0],
                half_extent: [f32::MAX, 0.5],
                ..RECT
            },
            RectInstance {
                center: [0.0, -f32::MAX],
                half_extent: [0.5, f32::MAX],
                ..RECT
            },
            RectInstance {
                rgba: [0.0, f32::NAN, 0.0, 1.0],
                ..RECT
            },
            RectInstance {
                rgba: [-0.1, 0.0, 0.0, 1.0],
                ..RECT
            },
            RectInstance {
                rgba: [0.0, 0.0, 0.0, 1.1],
                ..RECT
            },
        ] {
            assert_eq!(
                encode_rects(&[RECT, invalid], &mut bytes),
                Err(RenderError::InvalidRectangle)
            );
        }
    }

    #[test]
    fn synthetic_first_configure_loss_is_terminal_without_configuration_or_later_acquire() {
        let (errors, _, lost) = DeviceErrors::new();
        let mut state = Lifecycle::new(640, 420);
        assert!(state.resize(640, 420, 8192).unwrap());
        let diagnostic = "synthetic first configure DeviceLost\noriginal diagnostic";
        let mut configure_attempts = 0;
        // Invoke the very closure installed by new on the private Device, from
        // inside the production configure helper. No driver loss is induced.
        assert_eq!(
            state.configure(&errors, || {
                configure_attempts += 1;
                lost(wgpu::DeviceLostReason::Unknown, diagnostic.into());
            }),
            Err(RenderError::DeviceLost)
        );
        assert_eq!(configure_attempts, 1);
        assert_eq!(state.phase, SurfacePhase::Failed);
        assert_eq!(state.counters, RenderCounters::default());
        // Deliberately no GPU: actual public draw/resize must reject before
        // touching any GPU object, including get_current_texture.
        let mut renderer = SurfaceRenderer {
            gpu: None,
            lifecycle: state,
            device_errors: errors,
            adapter_info: wgpu::AdapterInfo {
                name: "synthetic; no GPU".into(),
                vendor: 0,
                device: 0,
                device_type: wgpu::DeviceType::Other,
                driver: String::new(),
                driver_info: String::new(),
                backend: wgpu::Backend::Vulkan,
            },
        };
        let original = Arc::clone(renderer.first_device_loss().unwrap());
        assert_eq!(original.reason, wgpu::DeviceLostReason::Unknown);
        assert_eq!(original.diagnostic, diagnostic);
        assert_eq!(renderer.draw(), Err(RenderError::DeviceLost));
        assert_eq!(renderer.draw_rects(&[]), Err(RenderError::DeviceLost));
        assert_eq!(renderer.resize(640, 420), Err(RenderError::DeviceLost));
        assert_eq!(renderer.resize(0, 0), Err(RenderError::DeviceLost));
        assert_eq!(
            renderer.lifecycle.configure(&renderer.device_errors, || {
                panic!("terminal renderer must never configure again")
            }),
            Err(RenderError::DeviceLost)
        );
        let mut root = yetlet::composition::Builder::<io::Error>::new().build();
        root.fail_required(io::Error::other(Arc::clone(&original)));
        root.fail_required(io::Error::other("later cleanup cause"));
        assert_eq!(renderer.close(), Ok(()));
        assert!(Arc::ptr_eq(
            renderer.first_device_loss().unwrap(),
            &original
        ));
        let retained = root
            .failures()
            .primary()
            .unwrap()
            .get_ref()
            .unwrap()
            .downcast_ref::<Arc<DeviceLostError>>()
            .unwrap();
        assert!(Arc::ptr_eq(retained, &original));
        assert_eq!(renderer.counters(), RenderCounters::default());
    }

    #[test]
    fn synthetic_configure_typed_oom_wins_over_same_call_loss_in_either_callback_order() {
        for loss_first in [false, true] {
            let (errors, error, lost) = DeviceErrors::new();
            let mut state = Lifecycle::new(640, 420);
            assert_eq!(
                state.configure(&errors, || {
                    if loss_first {
                        lost(wgpu::DeviceLostReason::Unknown, "synthetic loss".into());
                    }
                    error(synthetic_oom("original same-call typed OOM"));
                    if !loss_first {
                        lost(wgpu::DeviceLostReason::Unknown, "synthetic loss".into());
                    }
                }),
                Err(RenderError::OutOfMemory)
            );
            assert_eq!(state.phase, SurfacePhase::OutOfMemory);
            assert_eq!(state.counters, RenderCounters::default());
            let original = Arc::clone(state.first_device_error.as_ref().unwrap());
            assert!(matches!(original.as_ref(), wgpu::Error::OutOfMemory { .. }));
            assert_eq!(
                original.source().unwrap().to_string(),
                "original same-call typed OOM"
            );
            assert!(state.first_device_loss.is_none());
            assert_eq!(
                state.check_device_errors(&errors),
                Err(RenderError::OutOfMemory)
            );
            assert!(Arc::ptr_eq(
                state.first_device_error.as_ref().unwrap(),
                &original
            ));
        }
    }

    #[test]
    fn synthetic_loss_callbacks_are_bounded_per_device_and_safe_after_receiver_drop() {
        let (first_errors, _, first_lost) = DeviceErrors::new();
        let (second_errors, _, second_lost) = DeviceErrors::new();
        first_lost(wgpu::DeviceLostReason::Unknown, "first device loss".into());
        // Synchronous repeated calls with a full one-slot channel return without
        // a receiver draining it; the initial reason/diagnostic stays intact.
        for _ in 0..16 {
            first_lost(wgpu::DeviceLostReason::Destroyed, "later loss".into());
        }
        let mut first = Lifecycle::new(0, 0);
        let mut second = Lifecycle::new(640, 420);
        // Same post-setup check used by the constructor; zero size cannot hide loss.
        assert_eq!(
            first.check_device_errors(&first_errors),
            Err(RenderError::DeviceLost)
        );
        let loss = first.first_device_loss.as_ref().unwrap();
        assert_eq!(loss.reason, wgpu::DeviceLostReason::Unknown);
        assert_eq!(loss.diagnostic, "first device loss");
        assert_eq!(first.counters, RenderCounters::default());
        assert!(second.configure(&second_errors, || {}).unwrap());
        assert_eq!(second.phase, SurfacePhase::Configured);
        drop(first_errors);
        first_lost(wgpu::DeviceLostReason::Destroyed, "receiver gone".into());
        second_lost(wgpu::DeviceLostReason::Unknown, "second device loss".into());
        assert_eq!(
            second.check_device_errors(&second_errors),
            Err(RenderError::DeviceLost)
        );
        assert_eq!(
            second.first_device_loss.as_ref().unwrap().diagnostic,
            "second device loss"
        );
    }

    #[test]
    fn synthetic_later_loss_preserves_older_primary_and_later_oom_does_not_replace_loss() {
        let (errors, handler, lost) = DeviceErrors::new();
        let mut state = Lifecycle::new(640, 420);
        assert_eq!(
            state.fatal(FatalError {
                operation: "Texture::create_view",
                diagnostic: "original synthetic fatal".into(),
            }),
            RenderError::Fatal
        );
        let original = Arc::clone(state.first_fatal_error.as_ref().unwrap());
        lost(wgpu::DeviceLostReason::Unknown, "later loss".into());
        assert_eq!(state.check_device_errors(&errors), Err(RenderError::Fatal));
        assert!(state.first_device_loss.is_none());
        assert!(Arc::ptr_eq(
            state.first_fatal_error.as_ref().unwrap(),
            &original
        ));

        let mut state = Lifecycle::new(640, 420);
        assert_eq!(
            state.check_device_errors(&errors),
            Err(RenderError::DeviceLost)
        );
        let original = Arc::clone(state.first_device_loss.as_ref().unwrap());
        handler(synthetic_oom("later operation OOM"));
        assert_eq!(
            state.check_device_errors(&errors),
            Err(RenderError::DeviceLost)
        );
        assert_eq!(
            state.fatal(FatalError {
                operation: "Surface::present",
                diagnostic: "later fatal".into(),
            }),
            RenderError::DeviceLost
        );
        assert!(state.first_device_error.is_none() && state.first_fatal_error.is_none());
        assert!(Arc::ptr_eq(
            state.first_device_loss.as_ref().unwrap(),
            &original
        ));
    }

    #[test]
    fn synthetic_view_fatal_keeps_diagnostic_and_rejects_all_later_work() {
        let (errors, _, _) = DeviceErrors::new();
        let mut state = Lifecycle::new(640, 420);
        state.phase = SurfacePhase::Configured;
        let diagnostic = "wgpu error: Out of Memory\nsynthetic view diagnostic\n";
        // Same narrow boundary as acquired Texture::create_view. This is a
        // synthetic library panic, not an induced Vulkan allocation failure.
        let fatal = wgpu_fatal("Texture::create_view", || panic!("{diagnostic}")).unwrap_err();
        assert_eq!(state.fatal(fatal), RenderError::Fatal);
        assert_eq!(state.check_live(), Err(RenderError::Fatal));
        assert_eq!(state.resize(640, 420, 8192), Err(RenderError::Fatal));
        assert_eq!(
            state.configure(&errors, || panic!("must not run")),
            Err(RenderError::Fatal)
        );
        let original = Arc::clone(state.first_fatal_error.as_ref().unwrap());
        assert_eq!(original.operation, "Texture::create_view");
        assert_eq!(original.diagnostic, diagnostic);
        // Diagnostic wording never invents a typed wgpu::Error::OutOfMemory.
        assert!(state.first_device_error.is_none());
        state.fatal(FatalError {
            operation: "Surface::present",
            diagnostic: "secondary".into(),
        });
        assert!(Arc::ptr_eq(
            state.first_fatal_error.as_ref().unwrap(),
            &original
        ));
        let mut root = yetlet::composition::Builder::<io::Error>::new().build();
        root.fail_required(io::Error::other(Arc::clone(&original)));
        root.fail_required(io::Error::other("secondary cleanup"));
        let retained = root
            .failures()
            .primary()
            .unwrap()
            .get_ref()
            .unwrap()
            .downcast_ref::<Arc<FatalError>>()
            .unwrap();
        assert!(Arc::ptr_eq(retained, &original));
        assert_eq!(state.counters, RenderCounters::default());
    }

    #[test]
    fn synthetic_submission_oom_destroys_before_fatal_discard_and_retains_cleanup_failure() {
        use std::cell::Cell;
        let (errors, handler, _) = DeviceErrors::new();
        let mut state = Lifecycle::new(640, 420);
        state.phase = SurfacePhase::Configured;
        assert_eq!(
            state.device_call(&errors, || handler(synthetic_oom("primary submit OOM"))),
            Err(RenderError::OutOfMemory)
        );
        let first = Arc::clone(state.first_device_error.as_ref().unwrap());
        let destroyed = Cell::new(false);
        let dropped = Cell::new(false);
        struct Frame<'a>(&'a Cell<bool>);
        impl Drop for Frame<'_> {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }
        // Production release helper: destroy raw frame first, then a separate
        // discard unwind. Synthetic invalid-Device fatal, no real driver fault.
        let result = release_failed_frame(
            Frame(&dropped),
            |_| destroyed.set(true),
            |frame| {
                assert!(destroyed.get());
                drop(frame);
                panic!("Error in Surface::discard_texture: synthetic Device is invalid");
            },
        );
        assert!(destroyed.get() && dropped.get());
        state.note_frame_release(result);
        assert_eq!(
            state.frame_cleanup_error.as_ref().unwrap().operation,
            "Surface::discard_texture"
        );
        assert_eq!(state.check_live(), Err(RenderError::OutOfMemory));
        assert!(Arc::ptr_eq(
            state.first_device_error.as_ref().unwrap(),
            &first
        ));
        assert_eq!(first.source().unwrap().to_string(), "primary submit OOM");
        assert_eq!(state.closed(), Err(RenderError::FrameCleanupFailed));
        assert_eq!(state.closed(), Err(RenderError::FrameCleanupFailed));
        assert_eq!(state.check_live(), Err(RenderError::Closed));
        assert_eq!(state.counters.present_calls, 0);
        assert_eq!(state.counters.submissions, 0);
    }

    #[test]
    fn synthetic_present_fatal_consumes_presented_frame_without_discard_or_fake_receipt() {
        use std::cell::Cell;
        let discarded = Cell::new(false);
        struct Frame<'a> {
            presented: bool,
            discarded: &'a Cell<bool>,
        }
        impl Drop for Frame<'_> {
            fn drop(&mut self) {
                if !self.presented && !std::thread::panicking() {
                    self.discarded.set(true);
                }
            }
        }
        let frame = Frame {
            presented: false,
            discarded: &discarded,
        };
        let (errors, _, _) = DeviceErrors::new();
        let mut state = Lifecycle::new(640, 420);
        state.phase = SurfacePhase::Configured;
        state.counters.present_calls = 2;
        // Mirrors pinned SurfaceTexture::present's assignment before inner call.
        assert_eq!(
            state.present(&errors, move || {
                let mut frame = frame;
                frame.presented = true;
                panic!("Error in Surface::present: synthetic Vulkan queue-present OOM");
            }),
            Err(RenderError::Fatal)
        );
        assert!(!discarded.get());
        assert_eq!(state.counters.present_calls, 2);
        assert_eq!(state.resize(640, 420, 8192), Err(RenderError::Fatal));
        assert!(state.first_device_error.is_none());
        assert_eq!(
            state.first_fatal_error.as_ref().unwrap().operation,
            "Surface::present"
        );
        assert_eq!(state.closed(), Ok(())); // no discard failed in this case
    }

    #[test]
    fn synthetic_present_callback_oom_remains_primary_and_success_counts_once() {
        let (errors, handler, _) = DeviceErrors::new();
        let mut state = Lifecycle::new(640, 420);
        state.present(&errors, || {}).unwrap();
        assert_eq!(state.counters.present_calls, 1);
        assert_eq!(
            state.present(&errors, || {
                handler(synthetic_oom("typed first cause"));
                panic!("Error in Surface::present: synthetic secondary fatal");
            }),
            Err(RenderError::OutOfMemory)
        );
        assert_eq!(state.counters.present_calls, 1);
        assert_eq!(
            state
                .first_device_error
                .as_ref()
                .unwrap()
                .source()
                .unwrap()
                .to_string(),
            "typed first cause"
        );
        assert!(state.first_fatal_error.is_none());
    }

    #[test]
    fn unknown_panics_escape_the_exact_foreign_boundary_unchanged() {
        for operation in [
            "Texture::create_view",
            "Surface::discard_texture",
            "Surface::present",
        ] {
            let payload = catch_unwind(|| {
                let _ = wgpu_fatal(operation, || std::panic::panic_any(17_u32));
            })
            .unwrap_err();
            assert_eq!(*payload.downcast::<u32>().unwrap(), 17);
            let payload = catch_unwind(|| {
                let _ = wgpu_fatal(operation, || panic!("unrelated panic"));
            })
            .unwrap_err();
            assert_eq!(payload.downcast_ref::<&str>(), Some(&"unrelated panic"));
        }
    }

    fn synthetic_oom(cause: &'static str) -> wgpu::Error {
        wgpu::Error::OutOfMemory {
            source: io::Error::other(cause).into(),
        }
    }

    #[test]
    fn acquisition_device_oom_precedes_generic_surface_error_and_keeps_first_cause() {
        let (errors, handler, _) = DeviceErrors::new();
        let mut state = Lifecycle::new(640, 420);
        state.phase = SurfacePhase::Configured;
        let mut calls = 0;
        // Synthetic pinned-WGPU boundary: the Device callback fires before
        // get_current_texture returns Other, not SurfaceError::OutOfMemory.
        let result = state.device_call(&errors, || {
            calls += 1;
            handler(synthetic_oom("first acquisition OOM"));
            handler(wgpu::Error::Validation {
                source: io::Error::other("secondary error").into(),
                description: "must not replace OOM".into(),
            });
            Err::<(), _>(wgpu::SurfaceError::Other)
        });
        assert_eq!(result, Err(RenderError::OutOfMemory));
        assert_eq!(calls, 1);
        assert_eq!(state.phase, SurfacePhase::OutOfMemory);
        let first = Arc::clone(state.first_device_error.as_ref().unwrap());
        assert_eq!(first.source().unwrap().to_string(), "first acquisition OOM");
        // Same Arc handoff as the concrete consumer: Composition retains the
        // original Device error even after terminal rejection/renderer cleanup.
        let mut root = yetlet::composition::Builder::<io::Error>::new().build();
        root.fail_required(io::Error::other(Arc::clone(&first)));
        root.fail_required(io::Error::other("later cleanup failure"));
        let retained = root
            .failures()
            .primary()
            .unwrap()
            .get_ref()
            .unwrap()
            .downcast_ref::<Arc<wgpu::Error>>()
            .unwrap();
        assert!(Arc::ptr_eq(retained, &first));
        assert_eq!(state.resize(0, 0, 8192), Err(RenderError::OutOfMemory));
        assert_eq!(state.resize(640, 420, 8192), Err(RenderError::OutOfMemory));
        assert_eq!(
            state.device_call(&errors, || calls += 1),
            Err(RenderError::OutOfMemory)
        );
        assert_eq!(calls, 1);
        assert_eq!(state.counters.present_calls, 0);
        state.phase = SurfacePhase::Closed;
        assert!(Arc::ptr_eq(
            &first,
            state.first_device_error.as_ref().unwrap()
        ));
    }

    #[test]
    fn lost_or_outdated_reconfigure_catches_invalid_surface_at_configure_call() {
        for issue in [wgpu::SurfaceError::Lost, wgpu::SurfaceError::Outdated] {
            let (errors, handler, _) = DeviceErrors::new();
            let mut state = Lifecycle::new(640, 420);
            state.phase = SurfacePhase::Configured;
            state.surface_error(issue).unwrap();
            assert!(state.resize(640, 420, 8192).unwrap());
            let mut calls = 0;
            assert_eq!(
                state.configure(&errors, || {
                    calls += 1;
                    handler(wgpu::Error::Validation {
                        source: io::Error::other("synthetic configure InvalidSurface").into(),
                        description: "Surface::configure: InvalidSurface".into(),
                    });
                }),
                Err(RenderError::SurfaceFailed)
            );
            assert_eq!(calls, 1);
            assert_eq!(state.phase, SurfacePhase::Failed);
            assert_eq!(state.counters.configure_calls, 0);
            assert_eq!(state.counters.acquire_calls, 0);
            assert_eq!(state.counters.present_calls, 0);
            assert_eq!(
                state
                    .first_device_error
                    .as_ref()
                    .unwrap()
                    .source()
                    .unwrap()
                    .to_string(),
                "synthetic configure InvalidSurface"
            );
            assert_eq!(
                state.resize(640, 420, 8192),
                Err(RenderError::SurfaceFailed)
            );
            assert_eq!(
                state.configure(&errors, || calls += 1),
                Err(RenderError::SurfaceFailed)
            );
            assert_eq!(calls, 1);
        }
    }

    #[test]
    fn configure_oom_and_unrelated_internal_errors_are_terminal_not_configured() {
        for (issue, expected) in [
            (
                synthetic_oom("configure allocation"),
                RenderError::OutOfMemory,
            ),
            (
                wgpu::Error::Internal {
                    source: io::Error::other("unrelated internal failure").into(),
                    description: "internal error must remain visible".into(),
                },
                RenderError::SurfaceFailed,
            ),
        ] {
            let (errors, handler, _) = DeviceErrors::new();
            let mut state = Lifecycle::new(640, 420);
            assert!(state.resize(640, 420, 8192).unwrap());
            assert_eq!(state.configure(&errors, || handler(issue)), Err(expected));
            assert_eq!(state.check_live(), Err(expected));
            assert_ne!(state.phase, SurfacePhase::Configured);
            assert_eq!(state.counters, RenderCounters::default());
            assert!(state.first_device_error.is_some());
        }
    }

    #[test]
    fn pending_device_error_prevents_another_api_call_even_at_zero_size() {
        let (errors, handler, _) = DeviceErrors::new();
        let mut state = Lifecycle::new(0, 0);
        handler(synthetic_oom("pending device error"));
        let mut called = false;
        assert_eq!(
            state.device_call(&errors, || called = true),
            Err(RenderError::OutOfMemory)
        );
        assert!(!called);
        assert_eq!(state.resize(0, 0, 8192), Err(RenderError::OutOfMemory));
        assert_eq!(state.counters, RenderCounters::default());
    }

    #[test]
    fn callback_mailboxes_are_per_device_bounded_and_safe_after_receiver_drop() {
        let (first_errors, first_handler, _) = DeviceErrors::new();
        let (second_errors, second_handler, _) = DeviceErrors::new();
        first_handler(synthetic_oom("first device"));
        first_handler(synthetic_oom("full mailbox"));
        let mut first = Lifecycle::new(640, 420);
        let mut second = Lifecycle::new(640, 420);
        assert_eq!(
            first.check_device_errors(&first_errors),
            Err(RenderError::OutOfMemory)
        );
        assert!(second.configure(&second_errors, || {}).unwrap());
        assert_eq!(second.phase, SurfacePhase::Configured);
        assert_eq!(second.counters.configure_calls, 1);
        assert_eq!(second.counters.present_calls, 0);
        drop(second_errors);
        second_handler(synthetic_oom("receiver already released"));
    }

    #[test]
    fn preparation_and_each_zero_axis_defer_without_gpu_calls() {
        for (width, height) in [(0, 0), (0, 720), (1280, 0)] {
            let mut state = Lifecycle::new(width, height);
            assert_eq!(state.phase, SurfacePhase::DeferredZeroSize);
            assert!(!state.resize(width, height, 8192).unwrap());
            assert_eq!(state.size, [width, height]);
            assert_eq!(state.counters.configure_calls, 0);
            assert_eq!(state.counters.acquire_calls, 0);
            assert_eq!(state.counters.present_calls, 0);
        }
        assert_eq!(Lifecycle::new(640, 420).phase, SurfacePhase::Unconfigured);
    }

    #[test]
    fn positive_size_and_restore_allow_configure_but_do_not_claim_it() {
        let mut state = Lifecycle::new(0, 0);
        assert!(state.resize(640, 420, 8192).unwrap());
        assert_eq!(state.phase, SurfacePhase::Unconfigured);
        assert!(state.resize(640, 420, 8192).unwrap());
        assert!(!state.resize(0, 0, 8192).unwrap());
        assert!(state.resize(800, 600, 8192).unwrap());
        assert_eq!(state.size, [800, 600]);
        assert_eq!(state.counters.configure_calls, 0);
        assert_eq!(state.counters.present_calls, 0);
    }

    #[test]
    fn lost_and_outdated_require_reconfigure_without_fake_present() {
        for issue in [wgpu::SurfaceError::Lost, wgpu::SurfaceError::Outdated] {
            let mut state = Lifecycle::new(640, 420);
            state.phase = SurfacePhase::Configured; // synthetic transition fixture
            assert_eq!(
                state.surface_error(issue.clone()).unwrap(),
                DrawReceipt::Retry(issue)
            );
            assert_eq!(state.phase, SurfacePhase::Unconfigured);
            assert_eq!(state.counters.recovery_requests, 1);
            assert_eq!(state.counters.present_calls, 0);
            assert!(state.resize(640, 420, 8192).unwrap());
        }
    }

    #[test]
    fn recovery_at_zero_still_defers_configuration() {
        let mut state = Lifecycle::new(0, 0);
        state.surface_error(wgpu::SurfaceError::Lost).unwrap();
        assert_eq!(state.phase, SurfacePhase::DeferredZeroSize);
        assert!(!state.resize(0, 0, 8192).unwrap());
        assert_eq!(state.counters.configure_calls, 0);
    }

    #[test]
    fn timeout_is_retryable_without_invalidating_configuration() {
        let mut state = Lifecycle::new(640, 420);
        state.phase = SurfacePhase::Configured;
        assert_eq!(
            state.surface_error(wgpu::SurfaceError::Timeout).unwrap(),
            DrawReceipt::Retry(wgpu::SurfaceError::Timeout)
        );
        assert_eq!(state.phase, SurfacePhase::Configured);
        assert_eq!(state.counters.timeouts, 1);
        assert_eq!(state.counters.present_calls, 0);
    }

    #[test]
    fn out_of_memory_and_other_are_terminal_even_after_resize() {
        for (issue, error) in [
            (wgpu::SurfaceError::OutOfMemory, RenderError::OutOfMemory),
            (wgpu::SurfaceError::Other, RenderError::SurfaceFailed),
        ] {
            let mut state = Lifecycle::new(640, 420);
            assert_eq!(state.surface_error(issue), Err(error));
            assert_eq!(state.check_live(), Err(error));
            assert_eq!(state.resize(640, 420, 8192), Err(error));
            assert_eq!(state.resize(0, 0, 8192), Err(error));
            assert_eq!(state.surface_error(wgpu::SurfaceError::Timeout), Err(error));
            assert_eq!(state.counters.present_calls, 0);
        }
    }

    #[test]
    fn oversize_rejected_without_changing_previous_size_or_phase() {
        let mut state = Lifecycle::new(640, 420);
        state.phase = SurfacePhase::Configured;
        assert_eq!(
            state.resize(8193, 420, 8192),
            Err(RenderError::SizeExceedsDeviceLimit)
        );
        assert_eq!(
            state.resize(640, 8193, 8192),
            Err(RenderError::SizeExceedsDeviceLimit)
        );
        assert_eq!(state.size, [640, 420]);
        assert_eq!(state.phase, SurfacePhase::Configured);
        assert!(!state.resize(0, u32::MAX, 8192).unwrap());
    }

    #[test]
    fn closed_rejects_late_resize_and_error_without_changing_receipts() {
        let mut state = Lifecycle::new(640, 420);
        state.phase = SurfacePhase::Closed;
        let before = state.counters;
        assert_eq!(state.check_live(), Err(RenderError::Closed));
        assert_eq!(state.resize(640, 420, 8192), Err(RenderError::Closed));
        assert_eq!(
            state.surface_error(wgpu::SurfaceError::Lost),
            Err(RenderError::Closed)
        );
        assert_eq!(state.counters, before);
    }
}
