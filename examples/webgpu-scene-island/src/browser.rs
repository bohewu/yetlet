use crate::{
    Batch, HEIGHT, Lifetime, RECORD_BYTES, RECORDS, WIDTH, backing_size, instances,
    validate_records,
};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;

// Default CPU bytes survive device generations; dynamic CPU bytes are owner-held.
thread_local! {
    static CPU_RECORDS: Rc<Vec<u8>> = Rc::new(instances(RECORDS).expect("bounded synthetic records"));
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = window, js_name = __yt076Fault)]
    fn report_fault(generation: u32, category: &str);
    #[wasm_bindgen(js_namespace = performance)]
    fn now() -> f64;
}

struct Renderer {
    canvas: HtmlCanvasElement,
    config: wgpu::SurfaceConfiguration,
    configured: bool,
    configurations: u32,
    draws: u32,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::RenderPipeline,
    records: wgpu::Buffer,
    capacity: usize,
    buffer_allocations: u32,
    peak_transition_bytes: usize,
    data: Rc<RefCell<Batch>>,
    uploaded: u32,
    writes: u32,
    last_uploaded: u32,
    upload_ms: f64,
    encode_submit_ms: f64,
    startup: [f64; 4],
}
impl Renderer {
    fn resize(
        &mut self,
        width: f64,
        height: f64,
        dpr: f64,
        connected: bool,
    ) -> Result<bool, JsValue> {
        let Some(size) = backing_size(
            width,
            height,
            dpr,
            connected,
            self.device.limits().max_texture_dimension_2d,
        )
        .map_err(JsValue::from_str)?
        else {
            return Ok(false);
        };
        if !self.configured || self.config.width != size.width || self.config.height != size.height
        {
            self.canvas.set_width(size.width);
            self.canvas.set_height(size.height);
            self.config.width = size.width;
            self.config.height = size.height;
            self.surface.configure(&self.device, &self.config);
            self.configurations += 1;
            self.configured = true;
        }
        Ok(true)
    }
    fn draw(&mut self) -> Result<(), JsValue> {
        if !self.configured {
            return Err(JsValue::from_str("GPU target suspended"));
        }
        let upload_start = now();
        let mut data = self.data.borrow_mut();
        self.last_uploaded = 0;
        if data.bytes.len() > self.capacity {
            let replacement = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Bounded dynamic record replacement"),
                size: data.bytes.len() as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.peak_transition_bytes = self
                .peak_transition_bytes
                .max(self.capacity + data.bytes.len());
            self.buffer_allocations += 1;
            self.records.destroy();
            self.records = replacement;
            self.capacity = data.bytes.len();
            data.dirty.fill(true);
        }
        for range in data.dirty_ranges() {
            self.queue.write_buffer(
                &self.records,
                range.start as u64,
                &data.bytes[range.clone()],
            );
            self.last_uploaded += range.len() as u32;
            self.writes += 1;
        }
        data.dirty.fill(false);
        self.uploaded += self.last_uploaded;
        let count = (data.bytes.len() / RECORD_BYTES) as u32;
        drop(data);
        self.upload_ms = now() - upload_start;
        let encode_start = now();
        let frame = self.surface.get_current_texture().map_err(|error| {
            JsValue::from_str(match error {
                wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated => "surface-lost",
                wgpu::SurfaceError::OutOfMemory => "resource-exhausted",
                wgpu::SurfaceError::Timeout => "surface-timeout",
                wgpu::SurfaceError::Other => "surface-error",
            })
        })?;
        let view = frame.texture.create_view(&Default::default());
        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Synthetic work records"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline);
            pass.set_vertex_buffer(0, self.records.slice(..));
            pass.draw(0..6, 0..count);
        }
        self.queue.submit([encoder.finish()]);
        frame.present();
        self.encode_submit_ms = now() - encode_start;
        self.draws += 1;
        Ok(())
    }
}
impl Drop for Renderer {
    fn drop(&mut self) {
        self.records.destroy();
        self.device.destroy();
    }
}

/// One owned canvas/device. Shared lifetime guards each fallible asynchronous boundary.
#[wasm_bindgen]
pub struct SceneOwner {
    lifetime: Lifetime,
    generation: u32,
    renderer: Rc<RefCell<Option<Renderer>>>,
    data: Rc<RefCell<Batch>>,
}
#[wasm_bindgen]
impl SceneOwner {
    #[wasm_bindgen(constructor)]
    pub fn new(generation: u32) -> Self {
        Self {
            lifetime: Lifetime::new(),
            generation,
            renderer: Rc::new(RefCell::new(None)),
            data: Rc::new(RefCell::new(
                Batch::new(CPU_RECORDS.with(|b| b.as_ref().clone())).expect("default records"),
            )),
        }
    }

    pub async fn initialize(&self, canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        if !self.lifetime.begin() {
            return Err(JsValue::from_str(
                "GPU owner already initialized or retired",
            ));
        }
        let adapter_start = now();
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|_| JsValue::from_str("GPU canvas unavailable"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|_| JsValue::from_str("No WebGPU adapter; use HTML records"))?;
        if !self.lifetime.active() {
            return Err(JsValue::from_str(
                "GPU owner disposed during adapter request",
            ));
        }
        let adapter_ms = now() - adapter_start;
        let device_start = now();
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("YT-076 owned browser device"),
                ..Default::default()
            })
            .await
            .map_err(|_| JsValue::from_str("WebGPU device request failed; use HTML records"))?;
        if !self.lifetime.active() {
            device.destroy();
            return Err(JsValue::from_str(
                "GPU owner disposed during device request; late device destroyed",
            ));
        }
        let device_ms = now() - device_start;
        let pipeline_start = now();
        // Callbacks capture only this lifetime/generation; they cannot mutate a replacement.
        let lifetime = self.lifetime.clone();
        let generation = self.generation;
        device.set_device_lost_callback(move |_, _| {
            if lifetime.fail() {
                report_fault(generation, "device-lost");
            }
        });
        let lifetime = self.lifetime.clone();
        device.on_uncaptured_error(std::sync::Arc::new(move |error| {
            if lifetime.fail() {
                report_fault(
                    generation,
                    match error {
                        wgpu::Error::OutOfMemory { .. } => "resource-exhausted",
                        wgpu::Error::Validation { .. } => "validation-error",
                        wgpu::Error::Internal { .. } => "device-error",
                    },
                );
            }
        }));
        let config = surface
            .get_default_config(&adapter, WIDTH, HEIGHT)
            .ok_or_else(|| {
                device.destroy();
                JsValue::from_str("Unsupported GPU surface configuration")
            })?;
        // WebGPU validation errors are asynchronous and do not make submit fail.
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Trusted build-time synthetic status shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("scene.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Synthetic record rectangles"),
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
        let pipeline_ms = now() - pipeline_start;
        let upload_start = now();
        let (records, capacity) = {
            let data = self.data.borrow();
            let records = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Bounded synthetic records"),
                contents: &data.bytes,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });
            (records, data.bytes.len())
        };
        self.data.borrow_mut().dirty.fill(false);
        let upload_ms = now() - upload_start;
        let renderer = Renderer {
            canvas,
            config,
            configured: false,
            configurations: 0,
            draws: 0,
            surface,
            device,
            queue,
            pipeline,
            records,
            capacity,
            buffer_allocations: 1,
            peak_transition_bytes: capacity,
            data: Rc::clone(&self.data),
            uploaded: capacity as u32,
            writes: 1,
            last_uploaded: 0,
            upload_ms: 0.0,
            encode_submit_ms: 0.0,
            startup: [adapter_ms, device_ms, pipeline_ms, upload_ms],
        };
        let validation_error = renderer.device.pop_error_scope().await;
        if !self.lifetime.active() {
            return Err(JsValue::from_str(
                "GPU owner retired during validation; resources destroyed",
            ));
        }
        if validation_error.is_some() {
            return Err(JsValue::from_str(
                "GPU initialization validation failed; use HTML records",
            ));
        }
        if !self.lifetime.commit() {
            return Err(JsValue::from_str("GPU initialization result retired"));
        }
        *self.renderer.borrow_mut() = Some(renderer);
        Ok(format!("{info:?}"))
    }
    pub fn resize(
        &self,
        width: f64,
        height: f64,
        dpr: f64,
        connected: bool,
    ) -> Result<bool, JsValue> {
        if !self.lifetime.active() {
            return Err(JsValue::from_str("GPU owner retired"));
        }
        self.renderer
            .borrow_mut()
            .as_mut()
            .ok_or_else(|| JsValue::from_str("GPU not ready"))?
            .resize(width, height, dpr, connected)
    }
    pub fn configurations(&self) -> u32 {
        self.renderer
            .borrow()
            .as_ref()
            .map_or(0, |r| r.configurations)
    }
    pub fn draws(&self) -> u32 {
        self.renderer.borrow().as_ref().map_or(0, |r| r.draws)
    }
    pub fn device_limit(&self) -> u32 {
        self.renderer
            .borrow()
            .as_ref()
            .map_or(0, |r| r.device.limits().max_texture_dimension_2d)
    }
    pub fn buffer_bytes(&self) -> u32 {
        self.renderer
            .borrow()
            .as_ref()
            .map_or(0, |r| r.capacity as u32)
    }
    pub fn replace(&self, bytes: &[u8]) -> Result<(), JsValue> {
        if !self.lifetime.active() {
            return Err(JsValue::from_str("GPU owner retired"));
        }
        validate_records(bytes).map_err(JsValue::from_str)?;
        self.data
            .borrow_mut()
            .replace(bytes)
            .map_err(JsValue::from_str)
    }
    pub fn update(&self, start: u32, bytes: &[u8]) -> Result<(), JsValue> {
        if !self.lifetime.active() {
            return Err(JsValue::from_str("GPU owner retired"));
        }
        self.data
            .borrow_mut()
            .update(start, bytes)
            .map_err(JsValue::from_str)
    }
    pub fn cpu_bytes(&self) -> u32 {
        self.data.borrow().bytes.len() as u32
    }
    pub fn cpu_capacity_bytes(&self) -> u32 {
        let data = self.data.borrow();
        (data.bytes.capacity() + data.dirty.capacity()) as u32
    }
    /// Small scalar telemetry: explicit allocation lengths and CPU wall durations.
    pub fn metrics(&self) -> Vec<f64> {
        self.renderer.borrow().as_ref().map_or_else(
            || vec![0.0; 13],
            |r| {
                vec![
                    f64::from(r.uploaded),
                    f64::from(r.writes),
                    f64::from(r.last_uploaded),
                    r.upload_ms,
                    r.encode_submit_ms,
                    r.startup[0],
                    r.startup[1],
                    r.startup[2],
                    r.startup[3],
                    r.capacity as f64,
                    f64::from(r.draws),
                    f64::from(r.buffer_allocations),
                    r.peak_transition_bytes as f64,
                ]
            },
        )
    }
    /// Queue completion wall wait is neither pure GPU time nor presentation.
    pub async fn completion(&self) -> Result<(), JsValue> {
        use std::sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        };
        let done = Arc::new(AtomicBool::new(false));
        let waker = Arc::new(Mutex::new(None::<std::task::Waker>));
        {
            let renderer = self.renderer.borrow();
            let r = renderer
                .as_ref()
                .ok_or_else(|| JsValue::from_str("GPU not ready"))?;
            let done = Arc::clone(&done);
            let waker = Arc::clone(&waker);
            r.queue.on_submitted_work_done(move || {
                done.store(true, Ordering::SeqCst);
                if let Some(w) = waker.lock().expect("callback waker").take() {
                    w.wake();
                }
            });
        }
        std::future::poll_fn(|cx| {
            *waker.lock().expect("completion waker") = Some(cx.waker().clone());
            if done.load(Ordering::SeqCst) {
                std::task::Poll::Ready(())
            } else {
                std::task::Poll::Pending
            }
        })
        .await;
        if !self.lifetime.active() {
            return Err(JsValue::from_str("GPU owner retired"));
        }
        Ok(())
    }
    pub fn draw(&self) -> Result<(), JsValue> {
        if !self.lifetime.active() {
            return Err(JsValue::from_str("GPU owner disposed"));
        }
        self.renderer
            .borrow_mut()
            .as_mut()
            .ok_or_else(|| JsValue::from_str("GPU not ready"))?
            .draw()
    }
    pub fn dispose(&self) {
        self.lifetime.retire();
        self.renderer.borrow_mut().take();
    }
}
impl Default for SceneOwner {
    fn default() -> Self {
        Self::new(0)
    }
}
impl Drop for SceneOwner {
    fn drop(&mut self) {
        self.dispose();
    }
}
