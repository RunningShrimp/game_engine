//! 资源管理模块
//!
//! 提供高性能的GPU资源管理，包括Staging Buffer、内存分配和上传队列。

pub mod staging_buffer;
pub mod enhanced_staging_buffer;
pub mod ring_buffer_pool;
pub mod memory_allocator;
pub mod preallocation_manager;
pub mod upload_queue;
pub mod memory_monitor;
pub mod memory_debug;
pub mod manager;
pub mod runtime;
pub mod r#async;
pub mod atlas;
pub mod coroutine_loader;
pub mod events;
pub mod font;
pub mod hot_reload;
#[cfg(test)]
mod tests;

// 重新导出主要类型
pub use staging_buffer::{StagingBuffer, StagingBufferPool, PoolStats};
pub use enhanced_staging_buffer::{
    EnhancedStagingBuffer, 
    EnhancedStagingBufferPool, 
    EnhancedPoolStats, 
    EnhancedPerformanceMetrics,
    create_enhanced_staging_buffer_pool
};
pub use ring_buffer_pool::{
    RingBuffer, 
    RingBufferPool, 
    MemoryBlock, 
    BlockSize, 
    BufferState, 
    AllocationStats
};
pub use memory_allocator::{
    SmartMemoryAllocator,
    AllocationRequest,
    AllocationResult,
    AllocationType,
    AllocationPriority,
    AllocatorConfig,
    MemoryPressure,
    MemoryPressureEvent,
    PressureRecommendation,
    create_default_allocator,
    create_high_performance_allocator,
    create_low_memory_allocator
};
pub use preallocation_manager::{
    PreallocationManager,
    PreallocationConfig,
    PreallocatedBlock,
    AllocationPattern,
    PreallocationStats,
    create_default_preallocation_manager,
    create_high_performance_preallocation_manager,
    create_low_memory_preallocation_manager
};
pub use upload_queue::{
    UploadQueue,
    UploadStats,
    TextureUploadInfo,
    TextureUploadBuilder,
};
pub use memory_monitor::{
    MemoryMonitor,
    MonitorConfig,
    MemorySnapshot,
    PerformanceAnalysis,
    LeakDetectionResult,
    MonitoringStats,
    MonitoringExportData,
    create_default_memory_monitor,
    create_high_performance_memory_monitor,
    create_low_overhead_memory_monitor
};
pub use memory_debug::{
    MemoryDebugger,
    DebugConfig,
    AllocationTrace,
    MemoryVisualizationData,
    MemoryBlockInfo,
    MemoryHotspot,
    PerformanceChartData,
    ChartType,
    DataPoint,
    HotspotType,
    DebugExportData,
    create_default_memory_debugger,
    create_high_performance_memory_debugger,
    create_verbose_memory_debugger
};