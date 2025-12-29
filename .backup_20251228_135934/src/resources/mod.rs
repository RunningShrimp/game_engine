//! # 资源管理模块（Resource Management）
//!
//! 本模块提供高性能的GPU资源管理系统，优化资源加载和内存使用。
//!
//! ## 核心组件
//!
//! ### 统一资源管理（Unified Resource Management）
//! - [`UnifiedResourceManager`]: 统一资源管理器
//! - [`Resource`]: 资源trait
//! - [`ResourceLoader`]: 资源加载器trait
//! - [`ResourceLoaderRegistry`]: 加载器注册表
//!
//! ### 异步加载（Async Loading）
//! - [`AsyncUploader`]: 异步上传器
//! - [`UploadTask`]: 上传任务
//! - [`async_upload`]: 异步上传模块
//!
//! ### Staging Buffer优化
//! - [`RingBufferStagingBuffer`]: 环形缓冲区staging buffer
//! - [`RingBufferStagingPool`]: Staging buffer池
//! - [`staging_buffer`]: 基础staging buffer
//! - [`ring_buffer_pool`]: 环形buffer池
//!
//! ### 内存管理（Memory Management）
//! - [`MemoryAllocator`]: 内存分配器
//! - [`memory_monitor`]: 内存监控
//! - [`PreallocationManager`]: 预分配管理器
//! - [`memory_debug`]: 内存调试工具
//!
//! ### 缓存系统（Caching）
//! - [`CompressedCache`]: 压缩缓存
//! - [`shader_cache`]: 着色器缓存
//! - [`CacheStats`]: 缓存统计
//!
//! ### 热重载（Hot Reload）
//! - [`hot_reload`]: 资源热重载系统
//! - 支持运行时更新资源
//! - 自动检测文件变化
//!
//! ## 使用示例
//!
//! ### 基础资源加载
//!
//! ```rust,no_run
//! use game_engine::resources::{UnifiedResourceManager, TextureLoader, ModelLoader};
//!
//! # async fn load_resources() -> Result<(), Box<dyn std::error::Error>> {
//!     let manager = UnifiedResourceManager::new();
//!
//!     // 加载纹理
//!     let texture = manager.load_texture("player.png").await?;
//!
//!     // 加载模型
//!     let model = manager.load_model("scene.gltf").await?;
//!
//!     Ok(())
//! # }
//! ```
//!
//! ### 异步上传
//!
//! ```rust,no_run
//! use game_engine::resources::AsyncUploader;
//!
//! # async fn upload_to_gpu() {
//! let uploader = AsyncUploader::new();
//!
//! // 异步上传资源到GPU
//! uploader.upload_texture(texture_data).await;
//! uploader.upload_buffer(vertex_buffer).await;
//! # }
//! ```
//!
//! ### Staging Buffer池
//!
//! ```rust,no_run
//! use game_engine::resources::create_ring_buffer_staging_pool;
//!
//! fn setup_staging_buffers() {
//!     // 创建环形缓冲区池
//!     let pool = create_ring_buffer_staging_pool(
//!         1024 * 1024, // 1MB buffer size
//!         10,           // 10 buffers
//!     );
//!
//!     // 从池中获取buffer
//!     let staging_buffer = pool.acquire();
//!
//!     // 使用后自动归还
//! }
//! ```
//!
//! ## 性能优化
//!
//! - **异步加载**: 不阻塞主线程
//! - **Staging Buffer池**: 减少内存分配
//! - **纹理压缩**: 降低GPU内存占用
//! - **预分配**: 提前分配常用资源
//! - **缓存策略**: LRU缓存优化
//!
//! ## 支持的资源类型
//!
//! - **纹理（Textures）**: PNG, JPEG, 压缩格式（BC1-BC5）
//! - **模型（Models）**: GLTF 2.0格式
//! - **着色器（Shaders）**: WGSL, SPIR-V
//! - **音频（Audio）**: WAV, MP3, OGG
//! - **字体（Fonts）**: TTF, OTF
//! - **脚本（Scripts）**: Lua, Rust, JavaScript
//!
//! ## 热重载工作流程
//!
//! 1. 文件系统监控检测文件变化
//! 2. 重新加载变化的资源
//! 3. 更新GPU资源
//! 4. 通知相关系统更新
//!
//! ## 相关模块
//!
//! - [`crate::render`]: 渲染资源使用
//! - [`crate::audio`]: 音频资源使用
//! - [`crate::physics`]: 物理资源使用
//!


pub mod async_upload;
pub mod atlas;
pub mod coroutine_loader;
pub mod dependency_manager;
pub mod time;
pub mod ring_buffer_staging_pool;
pub mod events;
pub mod font;
pub mod gltf_loader;
pub mod hot_reload;
pub mod manager;
pub mod memory_allocator;
pub mod memory_debug;
pub mod memory_monitor;
pub mod preallocation_manager;
pub mod preload_manager;
pub mod ring_buffer_pool;
pub mod runtime;
pub mod shader_cache;
pub mod staging_buffer;
#[cfg(test)]
mod tests;
pub mod texture_compression;
pub mod texture_decoder;
pub mod upload_queue;

// 统一资源接口
pub mod compressed_cache;
pub mod loader_trait;
pub mod resource_trait;
pub mod streaming_loader;
pub mod unified_manager;

// 重新导出统一资源接口
pub use loader_trait::{
    AudioLoader, AudioResource, ModelLoader, ModelResource, TextureLoader, TextureResource,
};
pub use resource_trait::{
    Resource, ResourceError, ResourceLoader, ResourceLoaderRegistry, ResourceMetadata,
};
pub use unified_manager::{CacheStats, UnifiedResourceManager};

// 重新导出主要类型
pub use async_upload::{AsyncUploader, UploadTask};
pub use ring_buffer_staging_pool::{
    RingBufferPerformanceMetrics, RingBufferPoolStats, RingBufferStagingBuffer,
    RingBufferStagingPool, create_ring_buffer_staging_pool,
};
pub use memory_allocator::{
    AllocationPriority, AllocationRequest, AllocationResult, AllocationType, AllocatorConfig,
    MemoryPressure, MemoryPressureEvent, PressureRecommendation, SmartMemoryAllocator,
    create_default_allocator, create_high_performance_allocator, create_low_memory_allocator,
};
pub use memory_debug::{
    AllocationTrace, ChartType, DataPoint, DebugConfig, DebugExportData, HotspotType,
    MemoryBlockInfo, MemoryDebugger, MemoryHotspot, MemoryVisualizationData, PerformanceChartData,
    create_default_memory_debugger, create_high_performance_memory_debugger,
    create_verbose_memory_debugger,
};
pub use memory_monitor::{
    LeakDetectionResult, MemoryMonitor, MemorySnapshot, MonitorConfig, MonitoringExportData,
    MonitoringStats, PerformanceAnalysis, create_default_memory_monitor,
    create_high_performance_memory_monitor, create_low_overhead_memory_monitor,
};
pub use preallocation_manager::{
    AllocationPattern, PreallocatedBlock, PreallocationConfig, PreallocationManager,
    PreallocationStats, create_default_preallocation_manager,
    create_high_performance_preallocation_manager, create_low_memory_preallocation_manager,
};
pub use ring_buffer_pool::{
    AllocationStats, BlockSize, BufferState, MemoryBlock, RingBuffer, RingBufferPool,
};
pub use staging_buffer::{PoolStats, StagingBuffer, StagingBufferPool};
pub use texture_compression::{
    BC1Format, BC2Format, BC3Format, CompressedTexture, CompressionError, CompressionFormat,
    TextureCompression, TextureCompressionManager,
};
pub use upload_queue::{TextureUploadBuilder, TextureUploadInfo, UploadQueue, UploadStats};

// Re-export Dependency Manager components
pub use dependency_manager::{
    DependencyError, DependencyGraph, DependencyNode, LoadState, ResourceDependency,
};

// Re-export Preload Manager components
pub use preload_manager::{
    PreloadConfig, PreloadManager, PreloadRequest, PreloadState, PreloadStats, PreloadStatus,
    PreloadStrategy,
};

// Re-export Hot Reload components
pub use hot_reload::{HotReloadEvent, ResourceHotReloadManager, HotReloadService};

// Re-export Streaming Loader components
pub use streaming_loader::{
    ProgressiveQualityLoader, ResourceChunk, StreamingConfig, StreamingHandle, StreamingLoader,
};

// Re-export Compressed Cache components
pub use compressed_cache::{CompressedCacheStats, CompressedResourceCache, CompressionAlgorithm};

// Re-export Shader Cache components
pub use shader_cache::{
    ShaderCache, ShaderCacheConfig, ShaderCacheError, ShaderCacheKey, ShaderCacheStats,
};

// Re-export Texture Decoder components
pub use texture_decoder::{
    DecodedTexture, TextureDecoder, TextureDecodeConfig, TextureDecodeError, TextureFormat,
};

// 向后兼容类型别名
#[deprecated(since = "0.1.0", note = "Use TextureDecoder instead")]
pub type OptimizedTextureDecoder = texture_decoder::TextureDecoder;
