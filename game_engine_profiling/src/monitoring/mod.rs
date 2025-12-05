pub mod system_monitor;
pub mod monitoring_legacy;

pub use system_monitor::{
    CPUMonitor, FrameTimeSampler, MemoryMonitor, PerformanceMetrics, PerformanceReport,
    SystemPerformanceMonitor,
};

// 向后兼容：重新导出monitoring_legacy中的类型
pub use monitoring_legacy::{
    Metric, MetricStats, MetricType, OptimizationRecommendation, PerformanceIssue,
    PerformanceMonitor, PerformanceReport as MonitoringReport, IssueSeverity,
};
