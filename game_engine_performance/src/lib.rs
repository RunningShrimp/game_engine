//  # Game Engine Profiling
//
//  Performance profiling and benchmarking tools for game engines.
//
//  This crate provides a comprehensive set of tools for performance analysis,
//  benchmarking, monitoring, visualization, and CI/CD integration.
//
//  ## Modules
//
//  - [`profiling`] - Performance profiling tools
//  - [`benchmarking`] - Benchmarking tools
//  - [`monitoring`] - System monitoring tools
//  - [`visualization`] - Performance visualization tools
//  - [`cicd`] - CI/CD integration tools
//
//  ## Example
//
//  ```rust
//  use game_engine_profiling::{Profiler, Benchmark};
//
//  // Create a profiler
//  let mut profiler = Profiler::new();
//  profiler.start_scope("my_function");
//  // ... do work ...
//  profiler.end_scope("my_function");
//
//  // Run a benchmark
//  let mut benchmark = Benchmark::new("my_benchmark");
//  benchmark.run(|| {
//      // ... code to benchmark ...
//  });
//  ```

// Macro for implementing Default trait
#[macro_export]
macro_rules! impl_default {
    ($type:ident {
        $($field:ident: $value:expr),* $(,)?
    }) => {
        impl Default for $type {
            fn default() -> Self {
                Self {
                    $($field: $value),*
                }
            }
        }
    };
}

pub mod benchmarking;
pub mod cicd;
pub mod memory;
pub mod monitoring;
// 注意：profiling 模块已迁移到 game_engine::profiling
// 保留此模块仅用于向后兼容，但不再重新导出
pub mod profiling;
pub mod rendering;
pub mod visualization;

// Re-export public APIs
pub use benchmarking::*;
pub use cicd::*;
pub use memory::*;
pub use monitoring::*;
// profiling 不再重新导出，使用 game_engine::profiling 代替
pub use rendering::*;
pub use visualization::*;
