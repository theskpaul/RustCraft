use crate::mesher::{ChunkMesh, Vertex};
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use wgpu::util::DeviceExt;

const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Globals {
    view_proj: [[f32; 4]; 4],
    origin: [f32; 4],
}

struct GpuMesh {
    vb: wgpu::Buffer,
    ib: wgpu::Buffer,
    index_count: u32,
}

/// 2a: a single chunk, origin in a uniform. 2b replaces this with per-chunk draw data.
pub struct ChunkRenderer {
    pipeline: wgpu::RenderPipeline,
    globals_buf: wgpu::Buffer,
    globals_bg: wgpu::BindGroup,
    depth_view: wgpu::TextureView,
    mesh: Option<GpuMesh>,
    origin: [f32; 4],
}

fn create_depth_view(device: &wgpu::Device, size: (u32, u32)) -> wgpu::TextureView {
    device
        .create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width: size.0.max(1),
                height: size.1.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        })
        .create_view(&wgpu::TextureViewDescriptor::default())
}

impl ChunkRenderer {
    pub fn new(device: &wgpu::Device, color_format: wgpu::TextureFormat, size: (u32, u32)) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("chunk.wgsl"),
            source: wgpu::ShaderSource::Wgsl(include_str!("chunk.wgsl").into()),
        });

        let globals_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("globals_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let globals_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("globals_bg"),
            layout: &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals_buf.as_entire_binding(),
            }],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("chunk_layout"),
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let attrs = wgpu::vertex_attr_array![0 => Uint8x4, 1 => Uint16x2];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("chunk_pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attrs,
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            pipeline,
            globals_buf,
            globals_bg,
            depth_view: create_depth_view(device, size),
            mesh: None,
            origin: [0.0; 4],
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, size: (u32, u32)) {
        self.depth_view = create_depth_view(device, size);
    }

    pub fn upload_chunk(&mut self, device: &wgpu::Device, mesh: &ChunkMesh, origin: [i32; 3]) {
        self.origin = [origin[0] as f32, origin[1] as f32, origin[2] as f32, 0.0];
        if mesh.is_empty() {
            self.mesh = None;
            return;
        }
        let vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_vb"),
            contents: bytemuck::cast_slice(&mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("chunk_ib"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        self.mesh = Some(GpuMesh {
            vb,
            ib,
            index_count: mesh.indices.len() as u32,
        });
    }

    pub fn render(
        &self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        color_view: &wgpu::TextureView,
        view_proj: Mat4,
    ) {
        let globals = Globals {
            view_proj: view_proj.to_cols_array_2d(),
            origin: self.origin,
        };
        queue.write_buffer(&self.globals_buf, 0, bytemuck::bytes_of(&globals));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("chunk_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: color_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.53,
                        g: 0.72,
                        b: 0.92,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        if let Some(m) = &self.mesh {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.globals_bg, &[]);
            pass.set_vertex_buffer(0, m.vb.slice(..));
            pass.set_index_buffer(m.ib.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..m.index_count, 0, 0..1);
        }
    }
}
