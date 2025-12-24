// 性能分析模块
//
// 注意：所有profiling功能已统一到 game_engine::profiling 模块。
// 此模块已废弃，仅保留用于向后兼容。
//
// 新代码应直接从 game_engine::profiling 导入：
// ```rust
// use game_engine::profiling::{AdvancedProfiler, BottleneckDetector, ...};
// ```
//
// 为了保持向后兼容，此模块提供空的重新导出。
// 实际类型定义在 game_engine::profiling 中。

// 空模块 - 所有功能已迁移到 game_engine::profiling
// 使用此模块的代码需要更新导入路径
