//! Application-private browser pipeline. No yetlet-wgpu API or dependency.
use crate::model::ROW_FLOATS;
use rapier3d::math::glamx::glam::{Mat4, Quat, Vec3};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use wasm_bindgen::{JsCast, prelude::*};
use web_sys::HtmlCanvasElement;
use wgpu::util::DeviceExt;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name=GPUCanvasContext)]
    type CanvasContext;
    #[wasm_bindgen(method)]
    fn unconfigure(this: &CanvasContext);
    #[wasm_bindgen(js_namespace=window)]
    fn physicsLabFault(generation: u32, category: &str);
    #[wasm_bindgen(js_namespace=window)]
    fn physicsLabCompletion(generation: u32);
    #[wasm_bindgen(js_namespace=window)]
    fn physicsLabProgress(generation: u32, stage: &str);
}
fn bytes(values: &[f32]) -> Vec<u8> {
    values.iter().flat_map(|x| x.to_le_bytes()).collect()
}
fn geometry() -> (Vec<f32>, Vec<u32>, u32) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for (normal, u, v) in [
        (Vec3::X, Vec3::Y, Vec3::Z),
        (-Vec3::X, Vec3::Y, -Vec3::Z),
        (Vec3::Y, Vec3::Z, Vec3::X),
        (-Vec3::Y, Vec3::Z, -Vec3::X),
        (Vec3::Z, Vec3::X, Vec3::Y),
        (-Vec3::Z, Vec3::X, -Vec3::Y),
    ] {
        let base = (vertices.len() / 6) as u32;
        for (a, b) in [(-0.5, -0.5), (0.5, -0.5), (0.5, 0.5), (-0.5, 0.5)] {
            vertices.extend_from_slice(&(normal * 0.5 + u * a + v * b).to_array());
            vertices.extend_from_slice(&normal.to_array());
        }
        indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
    }
    let cube = indices.len() as u32;
    let base = (vertices.len() / 6) as u32;
    for y in 0..=12 {
        for x in 0..=16 {
            let phi = y as f32 * std::f32::consts::PI / 12.0;
            let theta = x as f32 * std::f32::consts::TAU / 16.0;
            let n = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
            vertices.extend_from_slice(&(n * 0.5).to_array());
            vertices.extend_from_slice(&n.to_array());
        }
    }
    for y in 0..12 {
        for x in 0..16 {
            let i = base + y * 17 + x;
            indices.extend([i, i + 17, i + 1, i + 1, i + 17, i + 18]);
        }
    }
    (vertices, indices, cube)
}
fn camera(yaw: f32, pitch: f32, distance: f32, aspect: f32) -> Result<(Mat4, Vec3), JsValue> {
    if !yaw.is_finite()
        || !pitch.is_finite()
        || !distance.is_finite()
        || !(-180.0..=180.0).contains(&yaw)
        || !(10.0..=80.0).contains(&pitch)
        || !(8.0..=70.0).contains(&distance)
        || !aspect.is_finite()
        || aspect <= 0.0
    {
        return Err(JsValue::from_str("invalid-camera"));
    }
    let (y, p) = (yaw.to_radians(), pitch.to_radians());
    let eye = Vec3::new(
        y.sin() * p.cos() * distance,
        2.0 + p.sin() * distance,
        y.cos() * p.cos() * distance,
    );
    Ok((
        rapier3d::math::glamx::glam::camera::rh::proj::directx::perspective(
            45f32.to_radians(),
            aspect,
            0.1,
            150.0,
        ) * rapier3d::math::glamx::glam::camera::rh::view::look_at_mat4(
            eye,
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::Y,
        ),
        eye,
    ))
}
struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    canvas: HtmlCanvasElement,
    config: wgpu::SurfaceConfiguration,
    configured: bool,
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    instances: wgpu::Buffer,
    camera: wgpu::Buffer,
    bind: wgpu::BindGroup,
    depth: Option<wgpu::Texture>,
    cube_indices: u32,
    all_indices: u32,
    pending: Arc<AtomicBool>,
}
impl Drop for Renderer {
    fn drop(&mut self) {
        if self.configured {
            if let Ok(Some(c)) = self.canvas.get_context("webgpu") {
                c.unchecked_into::<CanvasContext>().unconfigure();
            }
        }
        self.vertices.destroy();
        self.indices.destroy();
        self.instances.destroy();
        self.camera.destroy();
        if let Some(d) = &self.depth {
            d.destroy()
        }
        self.device.destroy();
    }
}
#[wasm_bindgen]
pub struct Gpu {
    generation: u32,
    alive: Arc<AtomicBool>,
    begun: Cell<bool>,
    inner: Rc<RefCell<Option<Renderer>>>,
}
#[wasm_bindgen]
impl Gpu {
    #[wasm_bindgen(constructor)]
    pub fn new(generation: u32) -> Gpu {
        Self {
            generation,
            alive: Arc::new(AtomicBool::new(true)),
            begun: Cell::new(false),
            inner: Rc::new(RefCell::new(None)),
        }
    }
    pub async fn initialize(&self, canvas: HtmlCanvasElement) -> Result<String, JsValue> {
        if self.begun.replace(true) || !self.alive.load(Ordering::SeqCst) {
            return Err(JsValue::from_str("retired-owner"));
        }
        physicsLabProgress(self.generation, "adapter");
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::BROWSER_WEBGPU,
            ..Default::default()
        });
        let surface = instance
            .create_surface(wgpu::SurfaceTarget::Canvas(canvas.clone()))
            .map_err(|_| JsValue::from_str("canvas-unavailable"))?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .map_err(|_| JsValue::from_str("adapter-unavailable"))?;
        if !self.alive.load(Ordering::SeqCst) {
            return Err(JsValue::from_str("retired-owner"));
        }
        physicsLabProgress(self.generation, "device");
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Physics laboratory private device"),
                ..Default::default()
            })
            .await
            .map_err(|_| JsValue::from_str("device-unavailable"))?;
        if !self.alive.load(Ordering::SeqCst) {
            device.destroy();
            return Err(JsValue::from_str("retired-owner"));
        }
        let alive = self.alive.clone();
        let generation = self.generation;
        device.set_device_lost_callback(move |_, _| {
            if alive.swap(false, Ordering::SeqCst) {
                physicsLabFault(generation, "device-lost")
            }
        });
        let alive = self.alive.clone();
        device.on_uncaptured_error(Arc::new(move |error| {
            if alive.swap(false, Ordering::SeqCst) {
                physicsLabFault(
                    generation,
                    match error {
                        wgpu::Error::OutOfMemory { .. } => "resource-exhausted",
                        wgpu::Error::Validation { .. } => "validation-error",
                        _ => "device-error",
                    },
                )
            }
        }));
        let config = surface.get_default_config(&adapter, 1, 1).ok_or_else(|| {
            device.destroy();
            JsValue::from_str("surface-unavailable")
        })?;
        physicsLabProgress(self.generation, "pipeline");
        device.push_error_scope(wgpu::ErrorFilter::OutOfMemory);
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Trusted private 3D shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("lab.wgsl").into()),
        });
        let pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{label:Some("Private 3D objects"),layout:None,
   vertex:wgpu::VertexState{module:&shader,entry_point:Some("vs_main"),compilation_options:Default::default(),buffers:&[
    wgpu::VertexBufferLayout{array_stride:24,step_mode:wgpu::VertexStepMode::Vertex,attributes:&wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3]},
    wgpu::VertexBufferLayout{array_stride:128,step_mode:wgpu::VertexStepMode::Instance,attributes:&wgpu::vertex_attr_array![2=>Float32x4,3=>Float32x4,4=>Float32x4,5=>Float32x4,6=>Float32x4,7=>Float32x4,8=>Float32x4,9=>Float32x4]}]},
   primitive:wgpu::PrimitiveState{cull_mode:None,..Default::default()},depth_stencil:Some(wgpu::DepthStencilState{format:wgpu::TextureFormat::Depth24Plus,depth_write_enabled:true,depth_compare:wgpu::CompareFunction::Less,stencil:Default::default(),bias:Default::default()}),multisample:Default::default(),fragment:Some(wgpu::FragmentState{module:&shader,entry_point:Some("fs_main"),compilation_options:Default::default(),targets:&[Some(wgpu::ColorTargetState{format:config.format,blend:None,write_mask:wgpu::ColorWrites::ALL})]}),multiview:None,cache:None});
        let (v, i, cube_indices) = geometry();
        let all_indices = i.len() as u32;
        let vb = bytes(&v);
        let ib = i.iter().flat_map(|n| n.to_le_bytes()).collect::<Vec<_>>();
        if vb.len() + ib.len() + 69 * 128 + 64 > 1024 * 1024 {
            device.destroy();
            return Err(JsValue::from_str("buffer-budget"));
        }
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shared cube and sphere vertices"),
            contents: &vb,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Shared primitive indices"),
            contents: &ib,
            usage: wgpu::BufferUsages::INDEX,
        });
        let instances = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bounded 69 object transforms"),
            size: 69 * 128,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Private camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera.as_entire_binding(),
            }],
        });
        let r = Renderer {
            surface,
            device,
            queue,
            canvas,
            config,
            configured: false,
            pipeline,
            vertices,
            indices,
            instances,
            camera,
            bind,
            depth: None,
            cube_indices,
            all_indices,
            pending: Arc::new(AtomicBool::new(false)),
        };
        physicsLabProgress(self.generation, "validation");
        let validation = r.device.pop_error_scope().await;
        let allocation = r.device.pop_error_scope().await;
        if validation.is_some() || allocation.is_some() || !self.alive.load(Ordering::SeqCst) {
            drop(r);
            return Err(JsValue::from_str("pipeline-unavailable"));
        }
        *self.inner.borrow_mut() = Some(r);
        Ok(format!("{} / {:?}", info.name, info.backend))
    }
    pub fn resize(
        &self,
        width: f64,
        height: f64,
        dpr: f64,
        connected: bool,
    ) -> Result<bool, JsValue> {
        if !width.is_finite()
            || !height.is_finite()
            || !dpr.is_finite()
            || width < 0.0
            || height < 0.0
        {
            return Err(JsValue::from_str("invalid-viewport"));
        }
        if !connected || width == 0.0 || height == 0.0 {
            return Ok(false);
        }
        let mut inner = self.inner.borrow_mut();
        let r = inner
            .as_mut()
            .ok_or_else(|| JsValue::from_str("not-ready"))?;
        let cap = r.device.limits().max_texture_dimension_2d.min(2048) as f64;
        let (w, h) = (width * dpr.clamp(1.0, 2.0), height * dpr.clamp(1.0, 2.0));
        let scale = (cap / w)
            .min(cap / h)
            .min((2097152.0 / (w * h)).sqrt())
            .min(1.0);
        let (w, h) = (
            (w * scale).floor().max(1.0) as u32,
            (h * scale).floor().max(1.0) as u32,
        );
        if !r.configured || r.config.width != w || r.config.height != h {
            r.canvas.set_width(w);
            r.canvas.set_height(h);
            r.config.width = w;
            r.config.height = h;
            r.surface.configure(&r.device, &r.config);
            if let Some(d) = r.depth.take() {
                d.destroy()
            }
            r.depth = Some(r.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Bounded depth"),
                size: wgpu::Extent3d {
                    width: w,
                    height: h,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth24Plus,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            }));
            r.configured = true;
        }
        Ok(true)
    }
    pub fn draw(&self, poses: &[f32], yaw: f32, pitch: f32, distance: f32) -> Result<(), JsValue> {
        if !self.alive.load(Ordering::SeqCst) {
            return Err(JsValue::from_str("retired-owner"));
        }
        if poses.is_empty()
            || poses.len() % ROW_FLOATS != 0
            || poses.len() / ROW_FLOATS > 69
            || poses.iter().any(|n| !n.is_finite())
        {
            return Err(JsValue::from_str("invalid-poses"));
        }
        let mut inner = self.inner.borrow_mut();
        let r = inner
            .as_mut()
            .ok_or_else(|| JsValue::from_str("not-ready"))?;
        if !r.configured {
            return Err(JsValue::from_str("suspended"));
        }
        let (vp, _) = camera(
            yaw,
            pitch,
            distance,
            r.config.width as f32 / r.config.height as f32,
        )?;
        let mut data = Vec::with_capacity(poses.len() / ROW_FLOATS * 32);
        for p in poses.chunks_exact(ROW_FLOATS) {
            if p[2..5].iter().any(|x| x.abs() > 100.0)
                || p[9..12].iter().any(|x| *x <= 0.0 || *x > 20.0)
                || p[12..16].iter().any(|x| !(0.0..=1.0).contains(x))
            {
                return Err(JsValue::from_str("invalid-poses"));
            }
            let q = Quat::from_xyzw(p[5], p[6], p[7], p[8]);
            if (q.length_squared() - 1.0).abs() > 0.001 {
                return Err(JsValue::from_str("invalid-rotation"));
            }
            let m = Mat4::from_scale_rotation_translation(
                Vec3::new(p[9], p[10], p[11]),
                q,
                Vec3::new(p[2], p[3], p[4]),
            );
            let n = m.inverse().transpose().to_cols_array();
            data.extend_from_slice(&m.to_cols_array());
            data.extend_from_slice(&n[..12]);
            data.extend_from_slice(&p[12..16]);
        }
        r.queue.write_buffer(&r.instances, 0, &bytes(&data));
        r.queue
            .write_buffer(&r.camera, 0, &bytes(&vp.to_cols_array()));
        let frame = r
            .surface
            .get_current_texture()
            .map_err(|_| JsValue::from_str("surface-error"))?;
        let target = frame.texture.create_view(&Default::default());
        let depth = r.depth.as_ref().unwrap().create_view(&Default::default());
        let mut encoder = r.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Physics lab 3D"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.05,
                            g: 0.09,
                            b: 0.12,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&r.pipeline);
            pass.set_bind_group(0, &r.bind, &[]);
            pass.set_vertex_buffer(0, r.vertices.slice(..));
            pass.set_vertex_buffer(1, r.instances.slice(..));
            pass.set_index_buffer(r.indices.slice(..), wgpu::IndexFormat::Uint32);
            for (index, p) in poses.chunks_exact(ROW_FLOATS).enumerate() {
                let range = if p[1] == 1.0 {
                    r.cube_indices..r.all_indices
                } else {
                    0..r.cube_indices
                };
                pass.draw_indexed(range, 0, index as u32..index as u32 + 1);
            }
        }
        r.queue.submit(Some(encoder.finish()));
        frame.present();
        if !r.pending.swap(true, Ordering::SeqCst) {
            let pending = r.pending.clone();
            let alive = self.alive.clone();
            let generation = self.generation;
            r.queue.on_submitted_work_done(move || {
                pending.store(false, Ordering::SeqCst);
                if alive.load(Ordering::SeqCst) {
                    physicsLabCompletion(generation)
                }
            });
        }
        Ok(())
    }
    pub fn ray(
        &self,
        x: f32,
        y: f32,
        yaw: f32,
        pitch: f32,
        distance: f32,
    ) -> Result<Vec<f32>, JsValue> {
        if !x.is_finite()
            || !y.is_finite()
            || !(0.0..=1.0).contains(&x)
            || !(0.0..=1.0).contains(&y)
        {
            return Err(JsValue::from_str("invalid-pointer"));
        }
        let inner = self.inner.borrow();
        let r = inner
            .as_ref()
            .ok_or_else(|| JsValue::from_str("not-ready"))?;
        let (vp, eye) = camera(
            yaw,
            pitch,
            distance,
            r.config.width as f32 / r.config.height as f32,
        )?;
        let point = vp
            .inverse()
            .project_point3(Vec3::new(x * 2.0 - 1.0, 1.0 - y * 2.0, 1.0));
        let d = (point - eye).normalize();
        Ok([eye.to_array(), d.to_array()].concat())
    }
    pub fn lose(&self) {
        if let Some(r) = self.inner.borrow().as_ref() {
            r.device.destroy();
        }
    }
    pub fn dispose(&self) -> bool {
        let first = self.alive.swap(false, Ordering::SeqCst);
        self.inner.borrow_mut().take();
        first
    }
}
impl Drop for Gpu {
    fn drop(&mut self) {
        self.dispose();
    }
}
