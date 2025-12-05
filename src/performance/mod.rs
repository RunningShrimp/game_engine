//! Performance模块
//!
//! 提供性能优化和集成功能。
//!
//! ## 模块结构
//!
//! - `memory/` - 内存优化（引擎核心依赖）
//! - `rendering/` - 渲染优化（引擎核心依赖）
//! - `gpu/` - GPU计算（引擎核心依赖）
//! - `optimization/` - 特定领域优化（引擎核心依赖）
//! - `sync/` - 同步工具（引擎核心依赖）
//!
//! ## Profiling工具
//!
//! 性能分析和基准测试工具已分离到`game_engine_profiling` crate。
//! 为了向后兼容，这些工具仍然可以通过`game_engine::performance`访问。

// 引擎核心依赖的模块
pub mod memory;
pub mod rendering;
pub mod gpu;
pub mod optimization;
pub mod sync;

// 重新导出profiling crate的公共API（向后兼容）
pub use game_engine_profiling::*;

// 重新导出引擎核心模块
pub use memory::*;
pub use rendering::*;
pub use gpu::*;
pub use optimization::*;
pub use sync::*;
