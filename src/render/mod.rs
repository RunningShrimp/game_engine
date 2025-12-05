pub mod animation;
pub mod mesh;
pub mod shader_async;
pub mod shader_cache;
pub mod shader_cache_helper;
pub mod text;
pub mod texture_compression;
pub mod tilemap;
pub mod wgpu;
pub mod wgpu_modules;

pub trait RenderDevice {}
pub trait RenderQueue {}
pub mod backend;
pub mod batch_builder;
pub mod clipping;
pub mod csm;
pub mod deferred;
pub mod frustum;
pub mod gpu_driven;
pub mod graph;
pub mod instance_batch;
pub mod lod;
pub mod occlusion_culling;
pub mod offscreen;
pub mod particles;
pub mod pbr;
pub mod pbr_renderer;
pub mod pipeline_optimization;
pub mod postprocess;
pub mod ray_tracing;
pub mod sprite_batch;
pub mod volumetric;

// Re-export GPU Driven components for convenience
pub use gpu_driven::{GpuDrivenConfig, GpuDrivenRenderer, GpuInstance};

// Re-export indirect draw error type
pub use gpu_driven::indirect::IndirectDrawError;

// Re-export Instance Batching components
pub use instance_batch::{
    BatchKey, BatchManager, BatchStats, DynamicBatchConfig, InstanceBatch, Mesh3DRenderer,
};

// Re-export GPU Particle System components
pub use particles::{
    ColorGradient, ColorStop, GpuParticleSystem, ParticleEmitter, ParticleEmitterConfig,
    ParticleShape, SizeOverLifetime,
};

// Re-export LOD System components
pub use lod::{
    LodConfig, LodConfigBuilder, LodGroup, LodLevel, LodQuality, LodSelection, LodSelector,
    LodStats, LodTransition,
};

// Re-export CSM components
pub use csm::{CascadedShadowMap, CsmConfig, CsmRenderer, CsmUniforms, ShadowQuality};

// Re-export Frustum Culling components
pub use frustum::{CullingResult, CullingSystem, Frustum, Plane};

// Re-export Occlusion Culling components
pub use occlusion_culling::HierarchicalZCulling;

// Re-export Pipeline Optimization components
pub use pipeline_optimization::{
    CommandBuffer, DrawCallOptimizer, GPUMemoryManager, RenderMetrics, RenderPipelineOptimization,
};

// Re-export Ray Tracing components
pub use ray_tracing::{
    Camera as RayTracingCamera, Light, LightType, Material, RayTracingConfig, RayTracingPlane,
    RayTracingRenderer, RayTracingScene, Sphere,
};

// Re-export Volumetric Rendering components
pub use volumetric::{Camera as VolumetricCamera, FogType, VolumetricConfig, VolumetricRenderer};

#[cfg(test)]
mod tests;
