use crate::chunk_renderer::ChunkRenderer;
use crate::gpu::GpuContext;
use anyhow::{Result, bail};
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Eq)]
pub enum FrameOutcome {
    Presented,
    Reconfigured,
    Skipped,
}

/// CPU-side wall time per phase of one frame. Not GPU execution time.
#[derive(Debug, Default, Clone, Copy)]
pub struct FrameTimings {
    /// Blocked in `get_current_texture` (vsync / compositor back-pressure shows up here).
    pub acquire: Duration,
    /// Encoding the pass and `queue.submit`.
    pub submit: Duration,
    /// `frame.present()`.
    pub present: Duration,
}

impl FrameTimings {
    pub fn total(&self) -> Duration {
        self.acquire + self.submit + self.present
    }
}

/// Acquire the next surface texture, draw the chunk, present.
pub fn render_chunk_frame(
    ctx: &GpuContext,
    chunks: &ChunkRenderer,
    view_proj: glam::Mat4,
) -> Result<(FrameOutcome, FrameTimings)> {
    let mut t = FrameTimings::default();
    let t0 = Instant::now();
    let frame = match ctx.surface.get_current_texture() {
        Ok(f) => f,
        Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
            ctx.surface.configure(&ctx.device, &ctx.config);
            return Ok((FrameOutcome::Reconfigured, t));
        }
        Err(wgpu::SurfaceError::Timeout) => return Ok((FrameOutcome::Skipped, t)),
        Err(e) => bail!("surface error: {e}"),
    };
    t.acquire = t0.elapsed();

    let t1 = Instant::now();
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
    t.submit = t1.elapsed();

    let t2 = Instant::now();
    frame.present();
    t.present = t2.elapsed();
    Ok((FrameOutcome::Presented, t))
}
