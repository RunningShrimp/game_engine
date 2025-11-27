pub mod wgpu;
pub mod wgpu_modules;
pub mod animation;
pub mod tilemap;
pub mod mesh;
pub mod text;

pub trait RenderDevice {}
pub trait RenderQueue {}
pub mod graph;
pub mod pbr;
pub mod pbr_renderer;
pub mod deferred;
pub mod csm;
pub mod clipping;
pub mod offscreen;
pub mod sprite_batch;
pub mod postprocess;
pub mod backend;
pub mod gpu_driven;

// Re-export GPU Driven components for convenience
pub use gpu_driven::{GpuDrivenRenderer, GpuDrivenConfig, GpuInstance};

#[cfg(test)]
mod tests;
