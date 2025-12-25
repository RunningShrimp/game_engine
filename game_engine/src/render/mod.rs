pub mod animation;
pub mod mesh;
pub mod shader_async;
pub mod shader_cache;
pub mod shader_cache_helper;
pub mod text;
pub mod texture_compression;
pub mod tilemap;
#[cfg(target_arch = "wasm32")]
pub mod webgl_adapter;
pub mod wgpu_modules;
pub mod wgpu_utils;

pub trait RenderDevice {}
pub trait RenderQueue {}
pub mod backend;
pub mod batch_builder;
pub mod batch_optimizer;
pub mod clipping;
pub mod csm;
pub mod deferred;
pub mod draw_call_merger;
pub mod frustum;
pub mod gpu_driven;
pub mod gpu_instancing;
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
pub mod ray_tracing_enhanced;
pub mod scene_traversal;
pub mod vxgi;
pub mod light_baking;
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

// Re-export Batch Optimizer components
pub use batch_optimizer::{
    BatchOptimizer, BatchOptimizerStats, BatchPerformanceMonitor, OptimizedBatch,
};

// Re-export Scene Traversal components
pub use scene_traversal::{
    IncrementalSceneUpdater, OptimizedSceneTraverser, SceneTraversalConfig, SceneTraversalResult,
    TraversalStats,
};

// Re-export Draw Call Merger components
pub use draw_call_merger::{
    DrawCallMergeConfig, DrawCallMerger, MergeStats, OptimizedSceneResult,
    SceneTraversalOptimizer,
};

// Re-export GPU Instancing components
pub use gpu_instancing::{
    GpuInstancingConfig, GpuInstancingRenderer, GpuInstancingStats, InstanceData,
};

// Re-export Ray Tracing components
pub use ray_tracing::{
    Camera as RayTracingCamera, Light, LightType, Material, RayTracingConfig, RayTracingPlane,
    RayTracingRenderer, RayTracingScene, Sphere,
};

// Re-export Enhanced Ray Tracing components
pub use ray_tracing_enhanced::{
    RayTracingAcceleration, RayTracingConfigEnhanced, RayTracingRendererEnhanced,
    RayTracingPerformanceStats, BVHNode,
};

// Re-export VXGI components
pub use vxgi::{VxgiConfig, VxgiRenderer, Voxel};

// Re-export Light Baking components
pub use light_baking::{
    LightBaker, Lightmap, LightmapConfig, LightmapFormat, SceneBakingData, StaticMeshData,
    LightBakingData, LightBakingType,
};

// Re-export Volumetric Rendering components
pub use volumetric::{Camera as VolumetricCamera, FogType, VolumetricConfig, VolumetricRenderer};

#[cfg(test)]
mod tests;
