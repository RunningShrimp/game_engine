// 性能分析模块
// 
// 注意：基础profiling功能（metrics, collector, storage等）已统一到 game_engine::profiling 模块。
// 此模块保留高级分析工具，这些工具可以独立使用或与主模块配合使用。

// 高级分析工具模块
pub mod advanced_profiler;
pub mod bottleneck_detector;
pub mod continuous_profiler;
pub mod frame_analyzer;
pub mod memory_profiler;
pub mod performance_analyzer;
pub mod profiler;

// 重新导出高级分析工具
pub use advanced_profiler::{AdvancedProfiler, PerformanceMetrics as AdvancedPerfMetrics};
pub use bottleneck_detector::{
    BottleneckDetector, BottleneckDiagnosis, BottleneckSeverity, BottleneckType,
};
pub use continuous_profiler::ContinuousProfiler;
pub use frame_analyzer::{FrameAnalyzer, FrameSnapshot, PhaseMetrics};
pub use memory_profiler::{GpuProfiler, MemoryProfiler};
pub use performance_analyzer::{Bottleneck, PerformanceAnalysis, PerformanceAnalyzer};
pub use profiler::Profiler;
