//  性能监控模块
// 
//  提供全面的性能监控基础设施，包括：
//  - 实时性能指标收集
//  - 低开销计数器和计时器
//  - 性能数据聚合和分析
//  - 自动性能报告和告警
// 
//  ## 架构设计
// 
//  ```text
//  ┌─────────────────────────────────────────────────────────────┐
//  │                    性能监控架构                              │
//  ├─────────────────────────────────────────────────────────────┤
//  │  指标收集层                                                │
//  │  ├── 渲染指标 (帧率、GPU利用率、Draw Call)                  │
//  │  ├── 内存指标 (分配次数、使用率、碎片化)                     │
//  │  ├── 物理指标 (计算时间、碰撞检测)                          │
//  │  ├── 音频指标 (延迟、缓冲区使用率)                          │
//  │  └── 系统指标 (CPU使用率、任务调度延迟)                     │
//  ├─────────────────────────────────────────────────────────────┤
//  │  数据处理层                                                │
//  │  ├── 高精度时间测量 (纳秒级)                               │
//  │  ├── 低开销计数器 (原子操作)                               │
//  │  ├── 批量数据聚合 (滑动窗口)                               │
//  │  └── 异步数据传输 (后台线程)                               │
//  ├─────────────────────────────────────────────────────────────┤
//  │  存储层                                                    │
//  │  ├── 内存中环形缓冲区 (实时访问)                            │
//  │  ├── 持久化存储 (历史数据)                                 │
//  │  ├── 数据压缩和归档 (长期存储)                             │
//  │  └── 查询和检索接口 (数据分析)                             │
//  ├─────────────────────────────────────────────────────────────┤
//  │  分析层                                                    │
//  │  ├── 实时聚合 (滑动窗口统计)                               │
//  │  ├── 历史数据分析 (趋势检测)                               │
//  │  ├── 异常检测 (性能回归)                                   │
//  │  └── 自动报告生成 (定期摘要)                               │
//  └─────────────────────────────────────────────────────────────┘
//  ```

pub mod metrics;
pub mod collector;
pub mod storage;
pub mod dashboard;
pub mod visualization;
pub mod alerting;
pub mod service;

// 重新导出公共API
pub use metrics::*;
pub use collector::*;
pub use storage::*;
pub use dashboard::*;
pub use visualization::*;
pub use alerting::*;
pub use service::*;

// 注意：高级分析工具（advanced_profiler, bottleneck_detector等）位于
// game_engine_performance crate 中，可以通过 game_engine_performance::profiling 访问。
// 为了向后兼容，game_engine::performance::profiling 也提供了这些工具的简化版本。

/// 性能监控版本
pub const PROFILING_VERSION: &str = "1.0.0";

/// 默认监控配置
pub mod defaults {
    /// 默认采样频率 (Hz)
    pub const DEFAULT_SAMPLE_RATE: f32 = 10.0;
    
    /// 默认环形缓冲区大小
    pub const DEFAULT_RING_BUFFER_SIZE: usize = 3600; // 1分钟 @ 60Hz
    
    /// 默认性能回归检测阈值 (百分比)
    pub const DEFAULT_REGRESSION_THRESHOLD: f32 = 5.0;
    
    /// 默认告警延迟 (秒)
    pub const DEFAULT_ALERT_DELAY: u64 = 5;
    
    /// 默认数据压缩阈值 (字节)
    pub const DEFAULT_COMPRESSION_THRESHOLD: usize = 1024 * 1024; // 1MB
}

/// 性能监控错误类型
#[derive(Debug, thiserror::Error)]
pub enum ProfilingError {
    #[error("指标收集错误: {0}")]
    CollectionError(String),
    
    #[error("存储错误: {0}")]
    StorageError(String),
    
    #[error("数据处理错误: {0}")]
    ProcessingError(String),
    
    #[error("配置错误: {0}")]
    ConfigurationError(String),
    
    #[error("IO错误: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("序列化错误: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// 性能监控结果类型
pub type ProfilingResult<T> = Result<T, ProfilingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults() {
        assert_eq!(defaults::DEFAULT_SAMPLE_RATE, 10.0);
        assert_eq!(defaults::DEFAULT_RING_BUFFER_SIZE, 3600);
        assert_eq!(defaults::DEFAULT_REGRESSION_THRESHOLD, 5.0);
    }

    #[test]
    fn test_profiling_version() {
        assert_eq!(PROFILING_VERSION, "1.0.0");
    }
}