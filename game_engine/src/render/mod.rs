//! # 渲染系统（Rendering System）
//!
//! 本模块提供基于WebGPU的现代渲染管线，支持高级渲染技术。
//!
//! ## 核心组件
//!
//! ### 渲染架构（Rendering Architecture）
//! - [`backend`]: 渲染后端，封装WebGPU
//! - [`graph`]: 渲染图，管理渲染pass依赖
//! - [`pbr_renderer`]: PBR（基于物理的渲染）渲染器
//! - [`deferred`]: 延迟渲染管线
//!
//! ### GPU驱动渲染（GPU-Driven Rendering）
//! - [`gpu_driven`]: GPU驱动渲染系统
//! - [`GpuDrivenRenderer`]: GPU驱动渲染器
//! - [`culling`]: GPU剔除
//! - [`indirect`]: 间接绘制
//!
//! ### 批处理优化（Batching Optimization）
//! - [`draw_call_merger`]: Draw call合并
//! - [`instance_batch`]: 实例化批处理
//! - [`batch_builder`]: 批次构建器
//! - [`batch_optimizer`]: 批次优化器
//!
//! ### 高级渲染技术（Advanced Techniques）
//! - [`ray_tracing`]: 光线追踪
//! - [`vxgi`]: 体积全局光照（VXGI）
//! - [`csm`]: 级联阴影映射
//! - [`volumetric`]: 体积光照
//! - [`postprocess`]: 后处理效果
//!
//! ### 性能优化（Performance Optimization）
//! - [`frustum`]: 视锥剔除
//! - [`occlusion_culling`]: 遮挡剔除
//! - [`lod`]: LOD（细节层次）
//! - [`pipeline_optimization`]: 渲染管线优化
//!
//! ## 渲染管线
//!
//! ### 前向渲染（Forward Rendering）
//! 适合简单场景，直接在渲染循环中计算光照：
//!
//! ```rust,no_run
//! // 在render pass中
//! for entity in visible_entities {
//!     let material = entity.material;
//!     for light in lights {
//!         shader.bind_light(light);
//!     }
//!     draw_entity(entity);
//! }
//! ```
//!
//! ### 延迟渲染（Deferred Rendering）
//! 适合多光源场景，将几何和光照分离：
//!
//! 1. **几何pass**: 渲染到G-buffer（位置、法线、颜色等）
//! 2. **光照pass**: 使用G-buffer计算光照
//! 3. **后处理pass**: 抗锯齿、色调映射等
//!
//! ### GPU驱动渲染（GPU-Driven）
//! 将部分渲染逻辑移至GPU，减少CPU开销：
//!
//! - **GPU剔除**: 在GPU上判断对象可见性
//! - **间接绘制**: 使用GPU buffer控制绘制
//! - **实例化**: GPU驱动的实例合并
//!
//! ## 光照系统
//!
//! ### 光源类型
//! - **方向光（Directional Light）**: 如太阳光
//! - **点光源（Point Light）**: 如灯泡
//! - **聚光灯（Spot Light）**: 如手电筒
//! - **环境光（Ambient Light）**: 基础照明
//!
//! ### 阴影技术
//! - **Shadow Mapping**: 基础阴影映射
//! - **CSM（Cascaded Shadow Maps）**: 级联阴影，适合大场景
//! - **PCSS（Percentage-Closer Soft Shadows）**: 软阴影
//!
//! ## 材质系统
//!
//! ### PBR材质（Physically Based Rendering）
//! - **反照率（Albedo）**: 基础颜色
//! - **金属度（Metallic）**: 0=非金属，1=金属
//! - **粗糙度（Roughness）**: 0=光滑，1=粗糙
//! - **法线（Normal）**: 表面朝向
//! - **AO（Ambient Occlusion）**: 环境遮蔽
//!
//! ## 后处理效果
//!
//! - **抗锯齿**: FXAA, TAA, MSAA
//! - **色调映射**: ACES Filmic, HDR
//! - ** bloom**: 辉光效果
//! - **景深**: 模拟相机对焦
//! - **运动模糊**: 模拟动态模糊
//!
//! ## 性能优化技巧
//!
//! 1. **减少Draw Call**: 使用批处理和实例化
//! 2. **LOD**: 远处物体使用低模
//! 3. **剔除**: 不可见对象不渲染
//! 4. **GPU驱动**: 将计算移至GPU
//! 5. **异步着色器**: 异步编译着色器
//!
//! ## 相关模块
//!
//! - [`crate::resources`]: 资源加载和管理
//! - [`crate::physics`]: 物理可视化
//! - [`crate::audio`]: 音频与视频渲染
//!

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
pub mod decals;
pub mod deferred;
pub mod domain_objects;
pub mod draw_call_merger;
pub mod frustum;
pub mod gpu_driven;
pub mod gpu_unified_manager;
pub mod gpu_instancing;
pub mod graph;
pub mod instance_batch;
pub mod material_sort;
pub mod render_pipeline_optimizer;
pub mod lod;
pub mod occlusion_culling;
pub mod offscreen;
pub mod particles;
pub mod pbr;
pub mod pbr_renderer;
pub mod pipeline_optimization;
pub mod postprocess;
pub mod ray_tracing;
pub mod scene_traversal;
pub mod vxgi;
pub mod light_baking;
pub mod procedural;
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

// Re-export Material Sort System components
pub use material_sort::{
    BatchResource, HybridMaterialSorter, MaterialSortConfig, MaterialSorter, MaterialSorterResource,
    SortStrategy, SortStats, material_sort_system,
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

// Re-export unified GPU render manager (整合GPU剔除和间接绘制)
pub use gpu_unified_manager::{
    GpuRenderConfig, GpuRenderManager, GpuRenderStats,
};

// Re-export Ray Tracing components (including enhanced features)
pub use ray_tracing::{
    BVHNode, Camera as RayTracingCamera, Light, LightType, Material, RayTracingAcceleration,
    RayTracingConfig, RayTracingPerformanceStats, RayTracingPlane, RayTracingRenderer,
    RayTracingScene, Sphere,
};

// RayTracingConfigEnhanced 和 RayTracingRendererEnhanced 已删除 - 请使用 RayTracingConfig 和 RayTracingRenderer

// Re-export VXGI components
pub use vxgi::{VxgiConfig, VxgiRenderer, Voxel};

// Re-export Light Baking components
pub use light_baking::{
    LightBaker, Lightmap, LightmapConfig, LightmapFormat, SceneBakingData, StaticMeshData,
    LightBakingData, LightBakingType,
};

// Re-export Volumetric Rendering components
pub use volumetric::{Camera as VolumetricCamera, FogType, VolumetricConfig, VolumetricRenderer};

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

#[cfg(test)]
mod tests;
