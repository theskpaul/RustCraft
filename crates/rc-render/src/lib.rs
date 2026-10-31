//! GPU layer. Depends on wgpu only; never on simulation crates.
pub mod gpu;

pub use gpu::GpuContext;
