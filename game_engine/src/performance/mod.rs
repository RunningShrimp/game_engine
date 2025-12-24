//  Performance模块
// 
//  提供性能优化和集成功能。
// 
//  ## 模块结构
// 
//  - `memory/` - 内存优化（引擎核心依赖）
//  - `rendering/` - 渲染优化（引擎核心依赖）
//  - `gpu/` - GPU计算（引擎核心依赖）
//  - `optimization/` - 特定领域优化（引擎核心依赖）
//  - `sync/` - 同步工具（引擎核心依赖）
// 
//  ## 性能分析工具
// 
//  性能分析和基准测试工具在 `crate::profiling` 模块中。
//  该模块提供：
//  - 实时性能指标收集
//  - 低开销计数器和计时器
//  - 性能数据聚合和分析
//  - 自动性能报告和告警

// 引擎核心依赖的模块
pub mod benchmark;
pub mod gpu;
pub mod memory;
pub mod optimization;
pub mod monitoring;
pub mod rendering;
pub mod sync;
pub mod tracing_metrics;
pub mod metrics_storage;

// 重新导出引擎核心模块
pub use benchmark::*;
pub use gpu::*;
pub use memory::*;
pub use optimization::*;
pub use rendering::*;
pub use sync::*;

/// 性能优化版本
pub const PERFORMANCE_VERSION: &str = "1.0.0";

/// 性能优化错误类型
#[derive(Debug, thiserror::Error)]
pub enum PerformanceError {
    #[error("优化错误: {0}")]
    OptimizationError(String),

    #[error("内存错误: {0}")]
    MemoryError(String),

    #[error("GPU错误: {0}")]
    GpuError(String),

    #[error("渲染错误: {0}")]
    RenderingError(String),

    #[error("同步错误: {0}")]
    SyncError(String),

    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),

    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// 性能优化结果类型
pub type PerformanceResult<T> = Result<T, PerformanceError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_version() {
        assert_eq!(PERFORMANCE_VERSION, "1.0.0");
    }
}
