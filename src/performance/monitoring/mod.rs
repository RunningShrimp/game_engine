pub mod system_monitor;

<<<<<<< HEAD
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
=======
// 所有功能已合并到system_monitor.rs中
pub use system_monitor::{
    CPUMonitor, FrameTimeSampler, MemoryMonitor, Metric, MetricStats, MetricType,
    OptimizationRecommendation, PerformanceIssue, PerformanceMetrics, PerformanceReport,
    SystemPerformanceMonitor,
};

// 向后兼容：保持PerformanceMonitor类型别名可用
pub use system_monitor::PerformanceMonitor;
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

