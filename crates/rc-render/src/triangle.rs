use crate::frame::FrameOutcome;
use crate::gpu::GpuContext;
use anyhow::{Result, bail};

pub struct TrianglePipeline {
    pipeline: wgpu::RenderPipeline,
}

impl TrianglePipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Result<Self> {
        device.push_error_scope(wgpu::ErrorFilter::Validation);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("triangle-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("triangle-layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("triangle-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        if let Some(err) = pollster::block_on(device.pop_error_scope()) {
            bail!("shader/pipeline validation error: {err}");
        }
        log::info!("shader compiled and pipeline created without validation errors");
        Ok(Self { pipeline })
    }
}

pub fn render_frame(ctx: &GpuContext, tri: &TrianglePipeline) -> Result<FrameOutcome> {
    let frame = match ctx.surface.get_current_texture() {
        Ok(f) => f,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            ctx.surface.configure(&ctx.device, &ctx.config);
            return Ok(FrameOutcome::Reconfigured);
        }
        Err(wgpu::SurfaceError::Timeout) => return Ok(FrameOutcome::Skipped),
        Err(e) => bail!("surface error: {e}"),
    };
    let view = frame
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut enc = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("frame"),
        });
    {
        let mut pass = enc.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("triangle-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.05,
                        g: 0.07,
                        b: 0.10,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&tri.pipeline);
        pass.draw(0..3, 0..1);
    }
    ctx.queue.submit(Some(enc.finish()));
    frame.present();
    Ok(FrameOutcome::Presented)
}
