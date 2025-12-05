pub mod system_monitor;

// 临时保留monitoring.rs以保持向后兼容
// TODO: 将monitoring.rs的独特功能合并到system_monitor.rs
pub mod monitoring_legacy;

pub use system_monitor::{
    CPUMonitor, FrameTimeSampler, MemoryMonitor, PerformanceMetrics, PerformanceReport,
    SystemPerformanceMonitor,
};

// 向后兼容：重新导出monitoring.rs中的类型
pub use monitoring_legacy::{
    Metric, MetricStats, MetricType, OptimizationRecommendation, PerformanceIssue,
    PerformanceMonitor, PerformanceReport as MonitoringReport,
};

