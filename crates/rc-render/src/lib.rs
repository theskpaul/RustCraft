//! GPU layer. Depends on wgpu only; never on simulation crates.
pub mod gpu;
pub mod triangle;

pub use gpu::GpuContext;
pub use triangle::{FrameOutcome, TrianglePipeline, render_frame};
