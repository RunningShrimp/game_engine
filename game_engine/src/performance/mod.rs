//! # Performance Optimization
//!
//! 本模块提供全面的性能优化工具和监控系统。
//!
//! ## 功能特性
//!
//! - **内存优化** - 高效的内存分配和管理策略
//! - **GPU计算** - GPU加速的通用计算
//! - **渲染优化** - 渲染管线性能优化
//! - **同步工具** - 线程同步和并发优化
//! - **性能监控** - 实时性能指标收集
//! - **基准测试** - 性能回归检测
//!
//! ## 主要组件
//!
//! ### 内存管理
//! - [`memory::arena::Arena`] - 高性能 arenas
//! - [`memory::pool::MemoryPool`] - 内存池
//! - [`memory::allocator::SmartMemoryAllocator`] - 智能内存分配器
//!
//! ### GPU计算
//! - [`gpu::gpu_compute::GpuCompute`] - GPU通用计算
//! - [`gpu::GpuPerformanceMonitor`] - GPU性能监控
//!
//! ### 性能监控
//! - [`monitoring::SystemMonitor`] - 系统性能监控
//! - [`tracing_metrics`] - 分布式追踪
//!
//! ### 基准测试
//! - [`benchmarking`] - 性能基准测试框架
//!
//! ## 性能分析
//!
//! 性能分析和profiling工具已分离到独立的`game_engine_profiling` crate。
//! 为了向后兼容，这些工具仍然可以通过`crate::profiling`访问。
//!
//! ## 使用示例
//!
//! ### 内存分配器
//!
//! ```rust,no_run
//! use game_engine::performance::create_default_allocator;
//!
//! // 创建默认分配器
//! let allocator = create_default_allocator();
//!
//! // 分配内存
//! let memory = allocator.allocate(1024).expect("Test: operation should succeed");
//! ```
//!
//! ### 性能监控
//!
//! ```rust,no_run
//! use game_engine::performance::monitoring::SystemMonitor;
//!
//! let monitor = SystemMonitor::new();
//! monitor.start();
//!
//! // 运行游戏循环...
//!
//! let stats = monitor.get_stats();
//! println!("FPS: {}", stats.fps);
//! println!("Frame time: {:?}", stats.frame_time);
//! ```

// 引擎核心依赖的模块
pub mod analyzer;
pub mod auto_fix;
pub mod benchmarking;
pub mod cache_system;
pub mod cpu_gpu_optimization;
pub mod gpu;
pub mod memory;
pub mod memory_analyzer;
pub mod monitoring;
pub mod optimization;
pub mod optimization_suggestion;
pub mod parallel_optimization;
pub mod profiler;
pub mod render_analyzer;
pub mod rendering;
pub mod report_generator;
pub mod sync;
pub mod tracing_metrics;

// 注意：profiling模块已在根级别声明，这里不重复声明以避免宏重复定义
// 使用 crate::profiling 路径访问
pub use crate::profiling as profiling_api;

// 向后兼容：重新导出常用profiling功能
pub use crate::profiling::continuous_profiler;
pub use crate::profiling::{HighPrecisionTimer, MetricCollector, ProfilingError};

// GpuPerformanceMonitor 从 profiling 模块重新导出
pub use crate::profiling::GpuPerformanceMonitor;

// 重新導出引擎核心模块
pub use gpu::*;
pub use memory::*;
pub use optimization::*;
pub use rendering::*;
pub use sync::*;
