//  性能分析模块
// 
//  注意：此模块的功能已统一到 game_engine::profiling 模块。
// 为了向后兼容，这里重新导出主模块的功能。

// 重新导出主模块的所有功能（统一API）
#[path = "../../profiling/mod.rs"]
mod profiling_mod;
pub use profiling_mod::*;
