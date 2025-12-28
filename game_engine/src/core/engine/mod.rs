//! # 游戏引擎核心（Core Engine）
//!
//! 本模块提供游戏引擎的核心运行时和主循环实现。
//!
//! ## 核心组件
//!
//! ### Engine（引擎）
//! - [`Engine`](engine::Engine) - 游戏引擎主结构，管理所有子系统
//! - [`EngineConfig`](crate::config::EngineConfig) - 引擎配置，定义初始化参数
//! - [`initialization`](initialization) - 引擎初始化逻辑
//!
//! ### Game Loop（游戏循环）
//! - [`GameLoop`](game_loop::GameLoop) - 游戏循环trait，定义循环接口
//! - [`GameLoopFixed`](game_loop_fixed::GameLoopFixed) - 固定时间步长游戏循环
//! - [`GameLoopCoroutine`](game_loop_coroutine::GameLoopCoroutine) - 协程式游戏循环
//!
//! ### Renderer（渲染器）
//! - [`Renderer`](renderer::Renderer) - 渲染器接口
//! - [`RenderState`](renderer::RenderState) - 渲染状态管理
//! - [`Frame`](renderer::Frame) - 帧数据结构
//!
//! ### Input（输入处理）
//! - [`InputHandler`](input_handler::InputHandler) - 输入事件处理器
//! - [`InputEvent`](input_handler::InputEvent) - 输入事件类型
//!
//! ### Async Optimization（异步优化）
//! - [`AsyncScheduler`](async_optimization::AsyncScheduler) - 异步任务调度器
//! - [`TaskPriority`](async_optimization::TaskPriority) - 任务优先级
//! - [`PhysicsSyncGuard`](async_optimization::PhysicsSyncGuard) - 物理同步守卫
//! - [`PhysicsSyncChecker`](async_optimization::PhysicsSyncChecker) - 物理同步检查器
//!
//! ## 游戏循环模式
//!
//! ### 固定时间步长（Fixed Time Step）
//! ```rust,no_run
//! use game_engine::core::engine::GameLoopFixed;
//!
//! let mut loop = GameLoopFixed::new(60.0); // 60 FPS
//! loop {
//!     loop.tick(|dt| {
//!         // 游戏逻辑更新，dt为固定时间步长
//!         update_game(dt);
//!     });
//! }
//! ```
//!
//! ### 协程式循环（Coroutine）
//! ```rust,no_run
//! use game_engine::core::engine::GameLoopCoroutine;
//!
//! let mut loop = GameLoopCoroutine::new();
//! loop {
//!     loop.tick().await;
//!     // 支持异步操作
//! }
//! ```
//!
//! ## 异步任务管理
//!
//! 引擎提供异步任务调度器，支持优先级和超时控制：
//!
//! ```rust,no_run
//! use game_engine::core::engine::{AsyncScheduler, TaskPriority};
//!
//! let scheduler = AsyncScheduler::new();
//!
//! // 提交高优先级任务
//! scheduler.spawn_task(
//!     "load_texture",
//!     TaskPriority::High,
//!     async {
//!         // 异步加载纹理
//!         Ok::<(), ()>(())
//!     }
//! );
//! ```
//!
//! ## 物理同步
//!
//! 物理更新必须在主线程同步执行，确保确定性：
//!
//! ```rust,no_run
//! use game_engine::core::engine::PhysicsSyncGuard;
//!
//! async fn update_physics() {
//!     let _guard = PhysicsSyncGuard::acquire().await;
//!     // 安全的物理更新
//!     physics_world.step(dt);
//!     // guard自动释放
//! }
//! ```
//!
//! ## 性能考虑
//!
//! - **固定时间步长**: 保证物理模拟确定性
//! - **异步任务**: 避免阻塞主线程
//! - **优先级调度**: 关键任务优先执行
//! - **超时检测**: 防止任务挂起
//!
//! ## 相关模块
//!
//! - [`crate::render`]: 渲染系统
//! - [`crate::physics`]: 物理系统
//! - [`crate::audio`]: 音频系统
//! - [`crate::resources`]: 资源管理
//!

pub mod asset_processor;
pub mod async_optimization;
pub mod demo_scene;
pub mod engine;
#[cfg(test)]
mod engine_tests;
pub mod game_loop;
pub mod game_loop_coroutine;
pub mod game_loop_fixed;
pub mod initialization;
pub mod input_handler;
pub mod renderer;

pub use crate::config::EngineConfig;
pub use crate::core::engine::engine::Engine;
pub use asset_processor::*;
pub use async_optimization::{
    AsyncScheduler, PhysicsSyncChecker, PhysicsSyncGuard, SchedulerStats, SyncError, TaskError,
    TaskPriority, with_timeout,
};
pub use game_loop::*;
pub use renderer::*;
