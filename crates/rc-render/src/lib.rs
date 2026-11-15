//! GPU layer. Depends on wgpu and rc-core (block/chunk data types only);
//! never on simulation logic.
pub mod camera;
pub mod chunk_renderer;
pub mod color;
pub mod frame;
pub mod gpu;
pub mod greedy;
pub mod mesher;

pub use frame::{FrameOutcome, FrameTimings, render_chunk_frame};
pub use gpu::GpuContext;

#[cfg(test)]
mod mesher_oracle;
