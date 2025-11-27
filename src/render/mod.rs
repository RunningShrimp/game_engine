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
pub mod instance_batch;
pub mod batch_builder;
pub mod particles;
pub mod lod;

// Re-export GPU Driven components for convenience
pub use gpu_driven::{GpuDrivenRenderer, GpuDrivenConfig, GpuInstance};

// Re-export Instance Batching components
pub use instance_batch::{BatchManager, InstanceBatch, BatchKey, BatchStats, Mesh3DRenderer};

// Re-export GPU Particle System components
pub use particles::{
    GpuParticleSystem,
    ParticleEmitter,
    ParticleEmitterConfig,
    ParticleShape,
    ColorGradient,
    ColorStop,
    SizeOverLifetime,
};

// Re-export LOD System components
pub use lod::{
    LodConfig,
    LodConfigBuilder,
    LodLevel,
    LodQuality,
    LodSelector,
    LodSelection,
    LodTransition,
    LodGroup,
    LodStats,
};

// Re-export CSM components
pub use csm::{CsmConfig, CsmRenderer, CascadedShadowMap, ShadowQuality, CsmUniforms};

#[cfg(test)]
mod tests;
