//! # Game Engine Profiling
//!
//! Performance profiling and benchmarking tools for game engines.
//!
//! This crate provides a comprehensive set of tools for performance analysis,
//! benchmarking, monitoring, visualization, and CI/CD integration.
//!
//! ## Modules
//!
//! - [`profiling`] - Performance profiling tools
//! - [`benchmarking`] - Benchmarking tools
//! - [`monitoring`] - System monitoring tools
//! - [`visualization`] - Performance visualization tools
//! - [`cicd`] - CI/CD integration tools
//!
//! ## Example
//!
//! ```rust
//! use game_engine_profiling::{Profiler, Benchmark};
//!
//! // Create a profiler
//! let mut profiler = Profiler::new();
//! profiler.start_scope("my_function");
//! // ... do work ...
//! profiler.end_scope("my_function");
//!
//! // Run a benchmark
//! let mut benchmark = Benchmark::new("my_benchmark");
//! benchmark.run(|| {
//!     // ... code to benchmark ...
//! });
//! ```

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

pub mod profiling;
pub mod benchmarking;
pub mod monitoring;
pub mod visualization;
pub mod cicd;

// Re-export public APIs
pub use profiling::*;
pub use benchmarking::*;
pub use monitoring::*;
pub use visualization::*;
pub use cicd::*;

