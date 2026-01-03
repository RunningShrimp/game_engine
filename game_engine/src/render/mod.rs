//! # Rendering System
//!
//! This module provides a modern WebGPU-based rendering pipeline with advanced
//! rendering techniques.
//!
//! ## Core Components
//!
//! ### Rendering Architecture
//! - [`backend`][]: Rendering backend, encapsulating WebGPU
//! - [`graph`][]: Render graph, managing render pass dependencies
//! - [`pbr_renderer`][]: PBR (Physically Based Rendering) renderer
//! - [`deferred`][]: Deferred rendering pipeline
//!
//! ### GPU-Driven Rendering
//! - [`gpu_driven`][]: GPU-driven rendering system
//! - [`GpuDrivenRenderer`][]: GPU-driven renderer
//! - [`culling`][]: GPU culling
//! - [`indirect`][]: Indirect drawing
//!
//! ### Batching Optimization
//! - [`draw_call_merger`][]: Draw call merging
//! - [`instance_batch`][]: Instanced batching
//! - [`batch_builder`][]: Batch builder
//! - [`batch_optimizer`][]: Batch optimizer
//!
//! ### Advanced Techniques
//! - [`ray_tracing`][]: Ray tracing
//! - [`vxgi`][]: Voxel Global Illumination (VXGI)
//! - [`gi`][]: Dynamic Diffuse Global Illumination (DDGI)
//! - [`csm`][]: Cascaded Shadow Maps
//! - [`volumetric`][]: Volumetric lighting
//! - [`postprocess`][]: Post-processing effects
//!
//! ### Performance Optimization
//! - [`frustum`][]: Frustum culling
//! - [`occlusion_culling`][]: Occlusion culling
//! - [`lod`][]: Level of Detail (LOD)
//!
//! ## Rendering Pipelines
//!
//! ### Forward Rendering
//! Suitable for simple scenes, calculating lighting directly in the render loop.
//!
//! ### Deferred Rendering
//! Suitable for multi-light scenes, separating geometry and lighting.
//!
//! 1. **Geometry Pass**: Render to G-buffer (position, normal, color, etc.)
//! 2. **Lighting Pass**: Calculate lighting using G-buffer
//! 3. **Post-processing Pass**: Anti-aliasing, tone mapping, etc.
//!
//! ### GPU-Driven Rendering
//! Move some rendering logic to GPU to reduce CPU overhead:
//!
//! - **GPU Culling**: Determine object visibility on GPU
//! - **Indirect Drawing**: Use GPU buffer to control drawing
//! - **Instancing**: GPU-driven instance merging
//!
//! ## Lighting System
//!
//! ### Light Types
//! - **Directional Light**: e.g., sunlight
//! - **Point Light**: e.g., light bulb
//! - **Spot Light**: e.g., flashlight
//! - **Ambient Light**: Base illumination
//!
//! ### Shadow Techniques
//! - **Shadow Mapping**: Basic shadow mapping
//! - **CSM (Cascaded Shadow Maps)**: Cascaded shadows for large scenes
//! - **PCSS (Percentage-Closer Soft Shadows)**: Soft shadows
//!
//! ## Material System
//!
//! ### PBR (Physically Based Rendering)
//! - **Albedo**: Base color
//! - **Metallic**: 0=non-metallic, 1=metallic
//! - **Roughness**: 0=smooth mirror, 1=rough diffuse
//! - **Normal**: Surface orientation
//! - **AO (Ambient Occlusion)**: Ambient occlusion
//!
//! ## Post-Processing Effects
//!
//! - **Anti-aliasing**: FXAA, TAA, MSAA
//! - **Tone Mapping**: ACES Filmic, HDR
//! - **Bloom**: Glow effect
//! - **Depth of Field**: Simulate camera focus
//! - **Motion Blur**: Simulate dynamic blur
//!
//! ## Performance Optimization Tips
//!
//! 1. **Reduce Draw Calls**: Use batching and instancing
//! 2. **LOD**: Use low-poly models for distant objects
//! 3. **Culling**: Don't render invisible objects
//! 4. **GPU-Driven**: Move computation to GPU
//! 5. **Async Shaders**: Compile shaders asynchronously
//!
//! ## Related Modules
//!
//! - [`crate::resources`][]: Resource loading and management
//! - [`crate::physics`][]: Physics visualization
//! - [`crate::audio`][]: Audio and video rendering

// 模块私有实现说明：
// - 基于WebGPU的跨平台渲染后端
// - 支持前向和延迟渲染管线
// - 提供GPU驱动的渲染优化
// - 集成PBR材质系统和全局光照

pub mod atmosphere;
pub mod comprehensive_tests;
pub mod gpu_optimization_example;
pub mod gpu_unified_manager_v2;
pub mod integrated_gpu;
pub mod lod_generator;
pub mod mesh;
pub mod mesh_simplifier;
pub mod performance_analyzer;
pub mod quality_assessor;
pub mod shader_async;
pub mod shader_cache;
pub mod shader_cache_helper;
pub mod test_helpers;
pub mod text;
pub mod texture_compression;
pub mod tile_based;
pub mod tilemap;
pub mod uv_atlas;
#[cfg(target_arch = "wasm32")]
pub mod webgl_adapter;
pub mod wgpu_compat;
pub mod wgpu_modules;
pub mod wgpu_utils;

pub trait RenderDevice {}
pub trait RenderQueue {}
pub mod backend;
pub mod batch_builder;
pub mod batch_optimizer;
pub mod clipping;
pub mod cqrs;
pub mod cqrs_performance_tests;
pub mod csm;
pub mod decals;
pub mod deferred;
pub mod domain_objects;
pub mod draw_call_merger;
pub mod frustum;
pub mod gi;
pub mod gpu_driven;
pub mod gpu_instancing;
pub mod gpu_particles;
pub mod gpu_unified_manager;
pub mod graph;
pub mod instance_batch;
pub mod light_baking;
pub mod lighting_trait;
pub mod lod;
pub mod material_sort;
pub mod occlusion_culling;
pub mod offscreen;
pub mod particles;
pub mod pbr;
pub mod pbr_renderer;
pub mod postprocess;
pub mod procedural;
pub mod ray_tracing;
pub mod render_pipeline_optimizer;
pub mod scene_traversal;
pub mod sprite_batch;
pub mod volumetric;
pub mod vxgi;

// Re-export GPU Driven components for convenience
pub use gpu_driven::{GpuDrivenConfig, GpuDrivenRenderer, GpuInstance};

// Re-export indirect draw error type
pub use gpu_driven::indirect::IndirectDrawError;

// Re-export Instance Batching components
pub use instance_batch::{
    BatchKey, BatchManager, BatchStats, DynamicBatchConfig, InstanceBatch, Mesh3DRenderer,
};

// Re-export Material Sort System components
pub use material_sort::{
    BatchResource, HybridMaterialSorter, MaterialSortConfig, MaterialSorter,
    MaterialSorterResource, SortStats, SortStrategy, material_sort_system,
};
pub use render_pipeline_optimizer::{
    OptimizedBatchesResource, PerformanceStats, PipelineBatchStats, PipelineOptimizationResult,
    RenderPipelineOptimizer, RenderPipelineOptimizerConfig, RenderPipelineOptimizerResource,
    render_pipeline_optimization_system,
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

// Re-export UV Atlas components
pub use uv_atlas::{AtlasOptions, PlacedIsland, UvAtlas, UvAtlasGenerator, UvIsland};

// Re-export Integrated GPU Optimization components
pub use integrated_gpu::{
    BandwidthDistribution, BandwidthMonitor, BandwidthOptimization, IntegratedGpuConfig,
    IntegratedGpuOptimizer, IntegratedGpuTier, ResolutionScaler, ShaderSimplification,
    TextureCompressionFormat, get_integrated_gpu_tier, is_integrated_gpu,
};

// Re-export Tile-based Rendering Optimization components
pub use tile_based::{
    BandwidthOptimizationHints,
    ClearOperation,
    ObjectBounds,
    OverdrawVisualizer,
    RenderObject as TileRenderObject, // 避免与domain_objects.RenderObject冲突
    RenderObjectType,
    RenderOrder,
    RenderPassOptimization,
    TextureFormat,
    TileBasedConfig,
    TileBasedOptimizer,
    TileBasedPassOptimizer,
    TileOverdrawStats,
    TileSize,
    is_tile_based_gpu,
    recommended_tile_size,
};

// Re-export CSM components
pub use csm::{CascadedShadowMap, CsmConfig, CsmRenderer, CsmUniforms, ShadowQuality};

// Re-export Frustum Culling components
pub use frustum::{CullingResult, CullingSystem, Frustum, Plane};

// Re-export Occlusion Culling components
pub use occlusion_culling::HierarchicalZCulling;

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
    DrawCallMergeConfig, DrawCallMerger, MergeStats, OptimizedSceneResult, SceneTraversalOptimizer,
};

// Re-export GPU Instancing components
pub use gpu_instancing::{
    GpuInstancingConfig, GpuInstancingRenderer, GpuInstancingStats, InstanceData,
};

// Re-export unified GPU render manager (整合GPU剔除和间接绘制)
pub use gpu_unified_manager::{GpuRenderConfig, GpuRenderManager, GpuRenderStats};

// Re-export enhanced GPU render manager v2
pub use gpu_unified_manager_v2::{
    EnhancedGpuRenderConfig, EnhancedGpuRenderManager, EnhancedGpuRenderStats,
};

// Re-export GPU optimization example
pub use gpu_optimization_example::{
    CullingComparisonResult, GpuOptimizationExample, PerformanceTestResult, VramStressTestResult,
    run_gpu_optimization_demo,
};

pub use performance_analyzer::{
    FpsTrend, OptimizationSuggestion, PerfConfig, PerformanceAnalyzer, PerformanceBottleneck,
    PerformanceReport, SuggestionType,
};

// Re-export Ray Tracing components (including enhanced features)
pub use ray_tracing::{
    BVHNode, Camera as RayTracingCamera, Light, LightType, Material, RayTracingAcceleration,
    RayTracingConfig, RayTracingPerformanceStats, RayTracingPlane, RayTracingRenderer,
    RayTracingScene, Sphere,
};

// RayTracingConfigEnhanced 和 RayTracingRendererEnhanced 已删除 - 请使用 RayTracingConfig 和 RayTracingRenderer

// Re-export VXGI components
pub use vxgi::{Voxel, VxgiConfig, VxgiRenderer};

// Re-export Light Baking components
pub use light_baking::{
    LightBaker, LightBakingData, LightBakingType, Lightmap, LightmapConfig, LightmapFormat,
    SceneBakingData, StaticMeshData,
};

// Re-export Volumetric Rendering components
pub use volumetric::{Camera as VolumetricCamera, FogType, VolumetricConfig, VolumetricRenderer};

// Re-export Atmospheric Rendering components
pub use atmosphere::{
    AtmosphereConfig, AtmosphereQuality, AtmosphereSystem, CloudConfig, CloudQuality,
    CloudRenderer, CloudType, FogConfig, FogQuality, FogRenderer, FogType as AtmosphereFogType,
    GroundFogConfig, HeightFogConfig, LightScatteringConfig, VolumetricFogConfig,
    VolumetricLightConfig, WeatherState, WeatherSystem,
};

// Re-export DDGI components
pub use gi::{
    DDGIConfig, DDGIError, DDGIProbe, DDGIQuality, DDGIVolume, GIDebugVisualizer,
    IrradianceTexture, ProbeManager, ProbeVisualization,
};

// Re-export Deferred Rendering components
pub use deferred::{
    CameraUniform, DeferredConfig, DeferredRenderer, DeferredRendererBase,
    DeferredRendererEnhanced, GBuffer, LightingUniform,
};

// Re-export Render Domain Objects (moved from domain layer to fix circular dependency)
pub use domain_objects::{
    LightSource, PbrScene, RenderCommand, RenderObject, RenderObjectCompensation, RenderObjectId,
    RenderScene, RenderStrategy,
};

// CQRS exports
pub use cqrs::{
    BatchGetTransformsHandler, BatchGetTransformsQuery, CreateRenderObjectCommand,
    GetObjectsInRadiusHandler, GetObjectsInRadiusQuery, GetStaticObjectsHandler,
    GetStaticObjectsQuery, GetVisibilityHandler, GetVisibilityQuery, GetVisibleObjectsHandler,
    GetVisibleObjectsQuery, GetWorldTransformHandler, GetWorldTransformQuery,
    RemoveRenderObjectCommand, RenderApplicationService, RenderBatchData, RenderQueryModel,
    SetVisibilityCommand, SetVisibilityHandler, UpdateTransformCommand, UpdateTransformHandler,
};

#[cfg(test)]
mod tests;

// ========================================
// 综合测试模块
// ========================================

#[cfg(test)]
mod render_backend_tests;

#[cfg(test)]
mod render_batch_tests;

#[cfg(test)]
mod extended_tests;

// Commenting out temporarily to allow compilation
// #[cfg(test)]
