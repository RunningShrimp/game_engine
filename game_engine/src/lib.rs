//! # Game Engine Library
//!
//! 高性能游戏引擎，提供完整的游戏开发基础设施：
//! - ECS（Entity Component System）
//! - 渲染系统
//! - 物理系统
//! - 音频系统
//! - 资源管理
//! - 脚本系统
//! - AI系统（寻路、行为树）
//! - 网络通信
//! - 性能监控和分析
//!
//! ## 核心模块
//!
//! - **ECS**: 基于 `bevy_ecs` 的高性能实体组件系统
//! - **渲染**: WebGPU 渲染管线、延迟渲染、阴影系统
//! - **物理**: 刚体物理、软体物理、空间分区
//! - **音频**: 3D 音频、流式处理、异步处理
//! - **资源**: 异步加载、热重载、纹理缓存
//!
//! ## 高级功能
//!
//! - **AI**: 导航网格、A* 寻路、行为树编辑器
//! - **网络**: TCP/UDP 通信、重连机制、压缩传输
//! - **性能**: 基准测试、性能回归检测、Tracy 集成
//! - **脚本**: Lua 脚本引擎、Rust 脚本
//! - **调试**: 场景编辑器、属性检查器、性能监控
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::*;
//!
//! // 初始化引擎
//! let mut engine = GameEngine::new();
//!
//! // 渲染循环
//! loop {
//!     engine.update().await;
//!     engine.render().await;
//! }
//! ```
//!
//! ## 性能特性
//!
//! - 支持多线程和异步任务调度
//! - GPU 加速的物理计算
//! - 对象池和内存池减少分配
//! - 延迟渲染减少 GPU 等待
//! - 空间数据结构优化（BVH、四叉树）
//!
//! ## 许可证
//!
//! 本项目基于 MIT 许可证开源。详见 LICENSE 文件。
//!
//! ## 文档说明
//!
//! 本库正在持续改进文档覆盖率。如有发现文档不足之处，欢迎提交Issue或PR。
//!
//! ## 开发状态
//!
//! 当前版本：v0.1.0
//! 状态：活跃开发中
//!

// 允许部分clippy lint以便渐进式改进代码质量
#![allow(
    // TODO: 将在后续迭代中移除这些允许
    unused_variables,
    unused_mut,
    dead_code,
    unreachable_pub,
    non_snake_case,
    non_camel_case_types,
    deprecated,
    while_true,
    non_upper_case_globals,
    // clippy lint将逐步修复
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable,
    clippy::indexing_slicing,
)]

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
pub mod scripting;
pub mod scene;
pub mod services;
pub mod ui;
pub mod world;
pub mod xr;

// 注意：为了避免歧义的 glob 重导出警告，不再使用 `pub use module::*;` 模式。
// 请从特定模块导入类型，例如：
// - use game_engine::ecs::{Transform, Velocity};
// - use game_engine::physics::PhysicsWorld;
// - use game_engine::render::deferred::DeferredRenderer;
// 等等。

// 重新导出核心类型（常用的顶级类型）
pub use build::BuildManager;
pub use config::EngineConfig;
pub use core::engine::Engine;
pub use domain::events::DomainEvent;
pub use network::NetworkState;
pub use performance::benchmarking::PerformanceRegression;
pub use plugins::registry::PluginRegistry;

/// 游戏引擎核心版本号
///
/// 格式：`major.minor.patch`（例如：`0.1.0`）
pub const ENGINE_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 引擎版本信息结构体
///
/// 包含版本号、Git提交信息和构建时间戳，用于运行时版本检查和调试。
///
/// # 字段
///
/// - `version` - 语义化版本号
/// - `git_commit` - Git提交哈希（如果在构建时可用）
/// - `build_time` - 构建时间戳
///
/// # 示例
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
    /// 语义化版本号（major.minor.patch）
    pub version: String,
    /// Git提交哈希（如果构建时可用）
    pub git_commit: Option<String>,
    /// 构建时间戳（格式：YYYY-MM-DD HH:MM:SS）
    pub build_time: String,
}

impl VersionInfo {
    /// 获取当前引擎版本信息
    ///
    /// 返回包含版本号、Git提交哈希和构建时间的VersionInfo实例。
    ///
    /// # 示例
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
            build_time: option_env!("BUILD_TIMESTAMP")
                .unwrap_or("unknown")
                .to_string(),
        }
    }
}

// ============================================================================
// Feature交叉检测
// ============================================================================

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
