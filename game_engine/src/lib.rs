//! # Game Engine Library
//!
//! A high-performance cross-platform game engine built with Rust, providing complete
//! game development infrastructure.
//!
//! ## Core Modules
//!
//! - **ECS**: High-performance Entity Component System based on `bevy_ecs`
//! - **Rendering**: WebGPU rendering pipeline with deferred rendering and shadow systems
//! - **Physics**: Rigid body physics, soft body physics, and spatial partitioning
//! - **Audio**: 3D audio with streaming and async processing
//! - **Resources**: Async loading, hot reloading, and texture caching
//!
//! ## Advanced Features
//!
//! - **AI**: Navigation meshes, A* pathfinding, and behavior tree editor
//! - **Network**: TCP/UDP communication with reconnection and compression
//! - **Performance**: Benchmarking, regression detection, and Tracy integration
//! - **Scripting**: Lua scripting engine and Rust scripting support
//! - **Debugging**: Scene editor, property inspector, and performance monitoring
//!
//! ## Examples

// Clippy allowances for game engine architecture
// Clippy 许可设置：针对游戏引擎架构的特殊需求
#![allow(dead_code)]  // Optional/test/development features - 可选/测试/开发功能
#![allow(clippy::too_many_arguments)]  // Complex render/physics APIs justified - 复杂的渲染/物理 API 需要更多参数
#![allow(clippy::type_complexity)]  // Generic system architecture requires complex types - 泛型系统架构需要复杂类型
#![allow(clippy::module_inception)]  // Intentional module structure - 故意的模块结构
#![allow(clippy::useless_conversion)]  // Type conversions for clarity - 为清晰度进行的类型转换
#![allow(clippy::await_holding_lock)]  // Acceptable in async context - 异步上下文中可接受
#![allow(unsafe_code)]  // FFI bindings require unsafe - FFI 绑定需要 unsafe
#![allow(private_interfaces)]  // Trait encapsulation - Trait 封装
#![allow(unknown_lints)]  // Allow for compatibility - 兼容性考虑
#![allow(improper_ctypes_definitions)]  // FFI boundary for plugins - 插件的 FFI 边界
#![allow(async_fn_in_trait)]  // Acceptable for resource API design - 资源 API 设计中可接受
// Note: async_fn_in_trait warnings are Rust compiler warnings, not clippy
// 注意：async_fn_in_trait 是 Rust 编译器警告，不是 clippy
// These are acceptable trade-offs for the async resource API - 异步资源 API 的可接受权衡

//!
//! ```rust
//! use game_engine::*;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut engine = Engine::new();
//!
//! // Game loop
//! loop {
//!     engine.update().await?;
//!     engine.render().await?;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Features
//!
//! - Multi-threaded and async task scheduling
//! - GPU-accelerated physics computation
//! - Object pooling and memory pooling to reduce allocations
//! - Deferred rendering to minimize GPU stalls
//! - Spatial data structure optimization (BVH, Quadtree)
//!
//! ## Architecture
//!
//! The engine follows a microkernel architecture pattern:
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │              Engine Core                     │
//! │  (Scheduler, Resource Manager, Plugins)     │
//! └─────────────────────────────────────────────┘
//!           │         │         │         │
//!           ▼         ▼         ▼         ▼
//!      ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐
//!      │ ECS  │ │Render│ │Physics│ │Network│
//!      └──────┘ └──────┘ └──────┘ └──────┘
//! ```
//!
//! ## License
//!
//! This project is open-sourced under the MIT License. See LICENSE file for details.
//!
//! ## Development Status
//!
//! Current version: v0.1.0
//! Status: Active development
//!


// Clippy lint allowances for gradual code quality improvement
// Clippy lint 许可：渐进式改进代码质量
#![allow(
    // ✅ P0-1 完成进度 (P0-1 Progress):
    // ✅ P0-1.1: 移除未使用的导入警告（通过cargo fix自动处理）
    // ✅ P0-1.2: 已移除 unused_variables 和 unused_mut
    // ✅ P0-1.3: 已移除 dead_code 和 unreachable_pub（改为局部使用）
    // ✅ P0-1.5: 已移除 non_snake_case, non_camel_case_types, non_upper_case_globals
    // ✅ P0-1.6: 已移除 panic, unimplemented, todo, unreachable（改为局部使用或编译期错误）
    // ✅ P0-1.7: 已移除 while_true（代码中未使用）
    deprecated,        // 保留: key_exchange.rs和engine.rs有局部allow，有合理TODO说明
    // clippy lint将逐步修复
    clippy::unwrap_used,      // P0-1.4: 待处理（主包中已优化至10个以内）
    clippy::expect_used,      // P0-1.4: 待处理（主包中已优化至10个以内）
    clippy::indexing_slicing, // TODO: 检查索引越界（待验证）
)]

// Public module re-exports
// 公开模块重导出
pub mod ai;
pub mod animation;
pub mod audio;
pub mod build;
pub mod common_errors;
pub mod config;
pub mod core;
pub mod domain;
pub mod ecs;
pub mod editor;
pub mod engine;
pub mod error;
pub mod network;
pub mod performance;
pub mod physics;
pub mod platform;
pub mod plugins;
pub mod profiling;
pub mod render;
pub mod resources;
pub mod scene;
pub mod scripting;
pub mod serialization;
pub mod services;
pub mod ui;
pub mod world;
pub mod xr;

// Note: To avoid ambiguous glob re-export warnings, we no longer use `pub use module::*;` pattern.
// 注意：为了避免歧义的 glob 重导出警告，不再使用 `pub use module::*;` 模式。
// Please import types from specific modules:
// 请从特定模块导入类型：
// - use game_engine::ecs::{Transform, Velocity};
// - use game_engine::physics::PhysicsWorld;
// - use game_engine::render::deferred::DeferredRenderer;

// Re-export commonly used top-level types
// 重新导出核心类型（常用的顶级类型）
pub use build::BuildManager;
pub use config::EngineConfig;
pub use core::engine::Engine;
pub use domain::events::DomainEvent;
pub use network::NetworkState;
pub use performance::benchmarking::PerformanceRegression;
pub use plugins::registry::PluginRegistry;

/// Core engine version string
///
/// Format: `major.minor.patch` (e.g., `0.1.0`)
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Engine version information structure
///
/// Contains version number, Git commit info, and build timestamp for runtime
/// version checking and debugging.
///
/// # Fields
///
/// - `version` - Semantic version number
/// - `git_commit` - Git commit hash (if available at build time)
/// - `build_time` - Build timestamp
///
/// # Examples
///
/// ```rust
/// use game_engine::VersionInfo;
///
/// let info = VersionInfo::current();
/// println!("Engine version: {}", info.version);
/// if let Some(commit) = info.git_commit {
///     println!("Git commit: {}", commit);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct VersionInfo {
    /// Semantic version number (major.minor.patch)
    pub version: String,
    /// Git commit hash (if available at build time)
    pub git_commit: Option<String>,
    /// Build timestamp (format: YYYY-MM-DD HH:MM:SS)
    pub build_time: String,
}

impl VersionInfo {
    /// Returns the current engine version information
    ///
    /// Returns a VersionInfo instance containing version number, Git commit hash,
    /// and build time.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use game_engine::VersionInfo;
    ///
    /// let info = VersionInfo::current();
    /// assert!(!info.version.is_empty());
    /// ```
    pub fn current() -> Self {
        Self {
            version: ENGINE_VERSION.to_string(),
            git_commit: option_env!("GIT_COMMIT_HASH").map(|s| s.to_string()),
            build_time: option_env!("BUILD_TIMESTAMP").unwrap_or("unknown").to_string(),
        }
    }
}

// ============================================================================
// Feature Cross-Detection / Feature 交叉检测
// ============================================================================

// Compile-time error for conflicting features
// 编译时检测冲突的 feature
#[cfg(all(feature = "secure_key_exchange", feature = "insecure_key_exchange"))]
compile_error!(
    "error: Cannot enable both 'secure_key_exchange' and 'insecure_key_exchange' features. \
    \nChoose one: \
    \n  - 'secure_key_exchange' for production (ECDH + HKDF) \
    \n  - 'insecure_key_exchange' for testing only \
    \n\
    \nExample: \
    \n  cargo build --features secure_key_exchange \
    \n  cargo build --features insecure_key_exchange"
);
