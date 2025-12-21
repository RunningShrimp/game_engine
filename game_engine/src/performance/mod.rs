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
//  ## Profiling工具
// 
//  性能分析和基准测试工具已分离到`game_engine_profiling` crate。
//  为了向后兼容，这些工具仍然可以通过`game_engine::performance`访问。

// 引擎核心依赖的模块
pub mod gpu;
pub mod memory;
pub mod optimization;
pub mod profiling;
pub mod monitoring;
pub mod rendering;
pub mod sync;
pub mod tracing_metrics;

// 重新导出profiling crate的公共API（向后兼容）
// pub use game_engine_profiling::*;
pub use profiling::*;
// 向后兼容：保留 `game_engine::performance::continuous_profiler` 访问路径
pub use profiling::continuous_profiler;

// 重新導出引擎核心模块
pub use gpu::*;
pub use memory::*;
pub use optimization::*;
pub use rendering::*;
pub use sync::*;
