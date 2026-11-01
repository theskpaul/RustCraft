//! GPU layer. Depends on wgpu only; never on simulation crates.
pub mod camera;
pub mod chunk_renderer;
pub mod gpu;
pub mod mesher;
pub mod triangle;

pub use gpu::GpuContext;
pub use triangle::{FrameOutcome, TrianglePipeline, render_frame};
