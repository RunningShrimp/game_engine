//  资源管理模块
//
//  提供高性能的GPU资源管理，包括Staging Buffer、内存分配和上传队列。

pub mod async_upload;
pub mod atlas;
pub mod coroutine_loader;
pub mod dependency_manager;
pub mod enhanced_staging_buffer;
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
pub mod staging_buffer;
#[cfg(test)]
mod tests;
pub mod texture_compression;
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
pub use enhanced_staging_buffer::{
    EnhancedPerformanceMetrics, EnhancedPoolStats, EnhancedStagingBuffer,
    EnhancedStagingBufferPool, create_enhanced_staging_buffer_pool,
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
pub use hot_reload::{HotReloadEvent, HotReloadManager, HotReloadService};

// Re-export Streaming Loader components
pub use streaming_loader::{
    ProgressiveQualityLoader, ResourceChunk, StreamingConfig, StreamingHandle, StreamingLoader,
};

// Re-export Compressed Cache components
pub use compressed_cache::{CompressedCacheStats, CompressedResourceCache, CompressionAlgorithm};
