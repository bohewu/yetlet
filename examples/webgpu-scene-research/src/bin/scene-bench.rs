use std::{
    error::Error,
    sync::mpsc,
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
use yetlet_webgpu_scene_research::{
    HEIGHT, IMAGE_BYTES, INSTANCE_BYTES, WIDTH, WORKLOADS, instance_data, scene, verify_pixels,
};

const WARMUPS: usize = 5;
const SAMPLES: usize = 30;

fn wait(device: &wgpu::Device, submission: wgpu::SubmissionIndex) -> Result<(), Box<dyn Error>> {
    device.poll(wgpu::PollType::Wait {
        submission_index: Some(submission),
        timeout: Some(Duration::from_secs(10)),
    })?;
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN,
        ..Default::default()
    });
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))?;
    let info = adapter.get_info();
    eprintln!("adapter={info:?}");
    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("YT-023 native offscreen scene"),
        ..Default::default()
    }))?;
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("scene WGSL"),
        source: wgpu::ShaderSource::Wgsl(include_str!("../scene.wgsl").into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("flat rectangle scene"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: INSTANCE_BYTES,
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
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    });
    println!(
        "nodes,width,height,warmups,sample,encode_submit_completion_ns,instance_buffer_bytes,color_texture_logical_bytes,readback_buffer_bytes,total_explicit_bytes"
    );
    for count in WORKLOADS {
        let nodes = scene(count);
        let data = instance_data(&nodes);
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("scene instances"),
            contents: &data,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let extent = wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("offscreen scene"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene proof readback"),
            size: IMAGE_BYTES,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        // Initial draw/readback is outside the timed samples, including uploads.
        let mut encoder = device.create_command_encoder(&Default::default());
        draw(&mut encoder, &pipeline, &vertices, &view, count);
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(WIDTH * 4),
                    rows_per_image: Some(HEIGHT),
                },
            },
            extent,
        );
        let submission = queue.submit([encoder.finish()]);
        let (tx, rx) = mpsc::channel();
        readback
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let _ = tx.send(result);
            });
        wait(&device, submission)?;
        rx.recv_timeout(Duration::from_secs(10))??;
        {
            let pixels = readback.slice(..).get_mapped_range();
            verify_pixels(&nodes, &pixels)?;
            let colored = pixels
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|pixel| pixel[..3] != [0, 0, 0])
                .count();
            // FNV-1a is a reproducibility fingerprint, not a cryptographic guarantee.
            let fingerprint = pixels.iter().fold(0xcbf29ce484222325u64, |h, b| {
                (h ^ *b as u64).wrapping_mul(0x100000001b3)
            });
            eprintln!(
                "proof nodes={count} verified_centers={count} colored_pixels={colored} rgba_fnv1a={fingerprint:016x}"
            );
        }
        readback.unmap();
        for index in 0..WARMUPS + SAMPLES {
            let start = Instant::now();
            let mut encoder = device.create_command_encoder(&Default::default());
            draw(&mut encoder, &pipeline, &vertices, &view, count);
            let submission = queue.submit([encoder.finish()]);
            wait(&device, submission)?;
            let ns = start.elapsed().as_nanos();
            if index >= WARMUPS {
                let bytes = count as u64 * INSTANCE_BYTES;
                println!(
                    "{count},{WIDTH},{HEIGHT},{WARMUPS},{},{ns},{bytes},{IMAGE_BYTES},{IMAGE_BYTES},{}",
                    index - WARMUPS + 1,
                    bytes + 2 * IMAGE_BYTES
                );
            }
        }
        // Free each workload before allocating the next one.
        vertices.destroy();
        readback.destroy();
        texture.destroy();
    }
    drop(pipeline);
    drop(shader);
    device.destroy();
    drop(queue);
    drop(device);
    drop(adapter);
    drop(instance);
    eprintln!("cleanup=all_workloads_destroyed_device_and_instance_dropped");
    Ok(())
}

fn draw(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::RenderPipeline,
    vertices: &wgpu::Buffer,
    view: &wgpu::TextureView,
    count: u32,
) {
    let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("YT-023 actual instanced draw"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view,
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
    pass.set_pipeline(pipeline);
    pass.set_vertex_buffer(0, vertices.slice(..));
    pass.draw(0..6, 0..count);
}
