//! # Core Engine
//!
//! This module provides the core runtime and main loop implementation for the game engine.
//!
//! ## Core Components
//!
//! ### Engine
//! - [`Engine`](engine::Engine) - Main engine structure that manages all subsystems
//! - [`EngineConfig`](crate::config::EngineConfig) - Engine configuration defining initialization parameters
//! - [`initialization`](initialization) - Engine initialization logic
// 本模块提供游戏引擎的核心运行时和主循环实现

#![allow(clippy::module_inception)]  // Intentional module structure - 故意的模块结构
//!
//! ### Game Loop
//! - [`GameLoop`](game_loop::GameLoop) - Game loop trait defining the loop interface
//! - [`GameLoopFixed`](game_loop_fixed::GameLoopFixed) - Fixed-timestep game loop
//! - [`GameLoopCoroutine`](game_loop_coroutine::GameLoopCoroutine) - Coroutine-style game loop
//! - [`HybridGameLoop`](game_loop_hybrid::HybridGameLoop) - **推荐**: Hybrid sync main loop + async background tasks
//!
//! 游戏循环trait，定义循环接口
//!
//! ## 性能优化建议 (P0-4)
//!
//! 使用 [`HybridGameLoop`](game_loop_hybrid::HybridGameLoop) 可以减少 1-2% 帧时间：
//!
//! ```rust,no_run
//! use game_engine::core::engine::HybridGameLoop;
//!
//! let mut game_loop = HybridGameLoop::new(60); // 60 FPS
//!
//! game_loop.run(
//!     |world, dt| {
//!         // 同步物理更新 - 可预测的性能
//!         println!("Physics: {:?}", dt);
//!     },
//!     |world| {
//!         // 同步游戏逻辑
//!         println!("Logic update");
//!     },
//!     |world| {
//!         // 同步渲染
//!         println!("Render");
//!     }
//! );
//! ```
//!
//! 异步任务（资源加载、网络IO）在后台运行，不阻塞主循环。
//!
//! ### Renderer
//! - [`Renderer`](renderer::Renderer) - Renderer interface
//! - [`RenderState`](renderer::RenderState) - Render state management
//! - [`Frame`](renderer::Frame) - Frame data structure
//! 渲染器接口
//!
//! ### Input Handling
//! - [`InputHandler`](input_handler::InputHandler) - Input event processor
//! - [`InputEvent`](input_handler::InputEvent) - Input event types
//! 输入事件处理器
//!
//! ## Game Loop Patterns
//!
//! ### Fixed Time Step
//! 固定时间步长：保证物理模拟确定性
//! ```rust,no_run
//! use game_engine::core::engine::GameLoopFixed;
//!
//! # fn update_game(dt: f32) {}
//! let mut game_loop = GameLoopFixed::new(60.0); // 60 FPS
//! loop {
//!     game_loop.tick(|dt| {
//!         // Game logic update with fixed timestep
//!         // 游戏逻辑更新，dt为固定时间步长
//!         update_game(dt);
//!     });
//! }
//! ```
//!
//! ### Coroutine Loop
//! 协程式循环：支持异步操作
//! ```rust,no_run
//! use game_engine::core::engine::GameLoopCoroutine;
//!
//! # async fn example() {
//! let mut game_loop = GameLoopCoroutine::new();
//! loop {
//!     game_loop.tick().await;
//!     // Supports async operations
//!     // 支持异步操作
//! }
//! # }
//! ```
//!
//! ## Performance Considerations
//!
//! 性能考虑：
//! - **Fixed time step**: Ensures deterministic physics simulation - 固定时间步长保证物理模拟确定性
//! - **Efficient game loop**: Minimizes CPU usage while maximizing frame rate - 高效游戏循环：最小化CPU使用同时最大化帧率
//!
//! ## Related Modules
//!
//! 相关模块：
//! - [`crate::render`][]: Rendering system - 渲染系统
//! - [`crate::physics`][]: Physics system - 物理系统
//! - [`crate::audio`][]: Audio system - 音频系统
//! - [`crate::resources`][]: Resource management - 资源管理
//!


pub mod asset_processor;
pub mod demo_scene;
pub mod engine;
#[cfg(test)]
mod engine_tests;
pub mod game_loop;
pub mod game_loop_coroutine;
pub mod game_loop_fixed;
pub mod game_loop_hybrid; // 新增：混合模式游戏循环
pub mod initialization;
pub mod input_handler;
pub mod renderer;

pub use crate::config::EngineConfig;
pub use crate::core::engine::engine::Engine;
pub use asset_processor::*;
pub use game_loop::*;
pub use game_loop_hybrid::*; // 导出混合模式
pub use renderer::*;
