use crate::chunk_renderer::ChunkRenderer;
use crate::gpu::GpuContext;
use anyhow::{Result, bail};

#[derive(Debug, PartialEq, Eq)]
pub enum FrameOutcome {
    Presented,
    Reconfigured,
    Skipped,
}

/// Acquire the next surface texture, draw the chunk, present.
/// Surface-loss handling is identical to the M1 `render_frame`.
pub fn render_chunk_frame(
    ctx: &GpuContext,
    chunks: &ChunkRenderer,
    view_proj: glam::Mat4,
) -> Result<FrameOutcome> {
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
            label: Some("chunk-frame"),
        });
    chunks.render(&ctx.queue, &mut enc, &view, view_proj);
    ctx.queue.submit(Some(enc.finish()));
    frame.present();
    Ok(FrameOutcome::Presented)
}
