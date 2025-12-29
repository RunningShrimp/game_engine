//! # Core Engine Module
//!
//! This module provides the core engine functionality including:
//!
//! - Engine initialization and lifecycle management
//! - Main loop control
//! - System coordination
//! - Resource management
//!
//! ## Architecture
//!
//! The engine follows a microkernel architecture pattern with event-driven
//! game loop, supporting both fixed and variable timestep systems.
//!
//! ## Examples
//!
//! ```rust,no_run
//! use game_engine::core::Engine;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     Engine::run()
//! }
//! ```

// 模块私有实现说明：
// - 使用winit 0.30作为窗口和事件循环管理
// - 基于协程的异步任务系统（tokio运行时）
// - 支持固定时间步（物理）和可变时间步（游戏逻辑）调度器
// - 集成ECS（Bevy ECS）作为核心数据模型

use crate::config::EngineConfig;
use crate::core::engine::game_loop_coroutine::CoroutineGameLoop;

/// Main game engine structure
///
/// Manages engine configuration and lifecycle, providing initialization
/// and runtime functionality for the entire game engine.
///
/// # Examples
///
/// ```rust,no_run
/// use game_engine::core::Engine;
/// use game_engine::config::EngineConfig;
///
/// let config = EngineConfig::default();
/// let engine = Engine::new(config);
/// ```
#[derive(Debug)]
pub struct Engine {
    /// Engine configuration settings
    pub config: EngineConfig,
}

impl Engine {
    /// Creates a new engine instance
    ///
    /// # Arguments
    ///
    /// * `config` - Engine configuration settings
    ///
    /// # Returns
    ///
    /// Returns the newly created engine instance
    ///
    /// # Examples
    ///
    /// ```
    /// use game_engine::core::Engine;
    /// use game_engine::config::EngineConfig;
    ///
    /// let config = EngineConfig::default();
    /// let engine = Engine::new(config);
    /// ```
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }

    /// Runs the engine
    ///
    /// This is the main entry point for the engine, performing the following operations:
    ///
    /// 1. Initialize tracing and metrics systems
    /// 2. Create default configuration
    /// 3. Initialize engine instance
    /// 4. Create window and event loop
    /// 5. Initialize ECS world and renderer
    /// 6. Start main game loop
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error on failure
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use game_engine::core::Engine;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     Engine::run()
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Window creation fails
    /// - Renderer initialization fails
    /// - Editor initialization fails
    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        // Initialize tracing and metrics systems
        // 初始化tracing和metrics系统
        crate::performance::tracing_metrics::TracingMetricsManager::init();

        tracing::info!("Game Engine starting...");

        // Create default configuration
        // 创建默认配置
        let config = EngineConfig::default();

        // Create engine instance
        // 创建引擎实例
        let engine = Self::new(config);

        tracing::info!("Game Engine initialized successfully");

        // Use pollster to run async initialization
        // 使用pollster运行异步初始化
        pollster::block_on(engine.run_async())
    }

    /// Async engine runner
    ///
    /// Internal async method called by `run()`.
    /// Executes the engine's main event loop, handling window events,
    /// updating game logic, and rendering.
    ///
    /// # Flow
    ///
    /// 1. Create event loop and window
    /// 2. Initialize ECS world, renderer, and editor
    /// 3. Enter event loop:
    ///    - Process window events (input, close, etc.)
    ///    - Update fixed-timestep scheduler (physics, etc.)
    ///    - Update variable-timestep scheduler (game logic)
    ///    - Execute coroutine tasks
    ///    - Render frame
    /// 4. Cleanup on exit
    // 内部异步运行方法：实现引擎的主事件循环
    // - 使用winit事件循环处理窗口事件
    // - 支持固定时间步（物理模拟）和可变时间步（游戏逻辑）
    // - 集成协程系统支持异步任务
    #[allow(deprecated)] // TODO: winit 0.30某些API仍标记为deprecated，等待上游稳定
    async fn run_async(self) -> Result<(), Box<dyn std::error::Error>> {
        use winit::event::{Event, WindowEvent};
        use winit::event_loop::{ControlFlow, EventLoop};

        // Create event loop
        // 创建事件循环
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);

        // Create window (using winit 0.30 API: EventLoop::create_window)
        // 创建窗口（使用winit 0.30 API：通过 EventLoop::create_window）
        let window_attrs = winit::window::WindowAttributes::default()
            .with_title("Game Engine")
            .with_inner_size(winit::dpi::LogicalSize::new(
                self.config.graphics.resolution.width,
                self.config.graphics.resolution.height,
            ));
        let window = std::sync::Arc::new(event_loop.create_window(window_attrs)?);

        // Initialize ECS world
        // 初始化ECS世界
        let mut world = bevy_ecs::world::World::new();

        // Initialize renderer (async)
        // 初始化渲染器（异步）
        let mut renderer = crate::render::wgpu_utils::WgpuRenderer::new(window.clone()).await?;

        // Initialize editor context (async)
        // 初始化编辑器上下文（异步）
        let device = renderer.device();
        let format = renderer.surface_format();
        let mut editor_ctx = crate::editor::EditorContext::new(&window, device, format).await;

        // 初始化渲染服务
        let mut render_service = crate::services::render::RenderService::new();
        render_service.use_default_lod();

        // 初始化资源服务器
        let asset_server = crate::resources::manager::AssetServer::new();

        // 初始化渲染缓存
        let mut render_cache = crate::render::graph::RenderCache::default();

        // 创建固定时间步调度器
        let mut fixed_schedule = crate::core::engine::initialization::create_fixed_schedule();

        // 创建可变时间步调度器
        let mut update_schedule = bevy_ecs::schedule::Schedule::default();

        // 初始化时间资源
        world.insert_resource(crate::ecs::Time {
            delta_seconds: 0.0,
            elapsed_seconds: 0.0,
            fixed_time_step: 1.0 / 60.0,
            alpha: 0.0,
        });

        // 初始化输入缓冲区
        world.insert_resource(crate::platform::InputBuffer::default());

        // 创建演示场景
        crate::core::engine::demo_scene::spawn_demo_scene(&mut world, &asset_server);

        tracing::info!("Engine setup complete, starting event loop...");

        // 初始化 Tokio 运行时（用于协程支持）
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .enable_all()
            .build()?;

        let runtime_handle = runtime.handle().clone();
        let _guard = runtime.enter();

        // 创建协程游戏循环
        let mut coroutine_game_loop =
            CoroutineGameLoop::new(std::time::Duration::from_secs_f64(1.0 / 60.0));

        // 创建协程任务管理器并添加到ECS世界
        let task_manager = crate::core::engine::game_loop_coroutine::CoroutineTaskManager::new(
            runtime_handle.clone(),
        );
        world.insert_resource(task_manager);

        tracing::info!("Coroutine game loop initialized");

        // 时间管理 - 使用固定时间步长循环管理器
        let mut last_time = std::time::Instant::now();
        let _fixed_timestep_loop = crate::core::engine::game_loop_fixed::FixedTimestepLoop::new(
            std::time::Duration::from_secs_f64(1.0 / 60.0),
        );

        // 创建WinitWindow包装器
        // 注意：由于WinitWindow::new需要ActiveEventLoop，我们需要在事件循环中创建
        // 但为了简化，我们直接使用window的Arc来创建
        let winit_window_arc = window.clone();

        // 运行事件循环
        event_loop.run(move |event, elwt| {
            let runtime_handle = runtime_handle.clone();
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent { event, window_id } if window_id == window.id() => {
                    // 处理窗口事件
                    crate::core::engine::input_handler::handle_window_event(
                        &event,
                        &mut world,
                        &mut renderer,
                        &mut editor_ctx,
                        &mut render_service,
                        &mut render_cache,
                        &window,
                        elwt,
                    );

                    match event {
                        WindowEvent::CloseRequested => {
                            tracing::info!("Window close requested, exiting...");
                            elwt.exit();
                        }
                        WindowEvent::RedrawRequested => {
                            // 计算帧时间
                            let current_time = std::time::Instant::now();
                            let _frame_time = current_time.duration_since(last_time);
                            last_time = current_time;

                            // 使用协程游戏循环更新固定时间步
                            let alpha =
                                coroutine_game_loop.update_fixed_step(&mut world, |world, dt| {
                                    // 更新固定时间步资源
                                    if let Some(mut time) =
                                        world.get_resource_mut::<crate::ecs::Time>()
                                    {
                                        time.delta_seconds = dt.as_secs_f32();
                                        time.elapsed_seconds += dt.as_secs_f64();
                                    }

                                    // 运行固定时间步调度器
                                    fixed_schedule.run(world);
                                });

                            // 更新插值因子（用于平滑渲染）
                            if let Some(mut time) = world.get_resource_mut::<crate::ecs::Time>() {
                                time.alpha = alpha;
                            }

                            // 运行可变时间步调度器
                            update_schedule.run(&mut world);

                            // 轮询tokio运行时处理异步任务（非阻塞）
                            // 这允许后台协程任务在主循环中执行
                            runtime_handle.spawn(async {
                                // 短暂让出控制权，允许其他协程执行
                                tokio::task::yield_now().await;
                            });

                            // 使用协程任务管理器处理异步任务
                            // 示例：定期生成异步任务（每60帧生成一次）
                            // 在实际游戏中，这里可以用于AI寻路、资源加载等异步任务
                            if let Some(time) = world.get_resource::<crate::ecs::Time>() {
                                let frame_count = (time.elapsed_seconds * 60.0) as u64;
                                if frame_count.is_multiple_of(60)
                                    && let Some(task_manager) = world.get_resource::<crate::core::engine::game_loop_coroutine::CoroutineTaskManager>() {
                                        let task_manager = task_manager.clone();
                                        runtime_handle.spawn(async move {
                                            let _task_id = task_manager.spawn_task(
                                                "background_task".to_string(),
                                                crate::core::engine::game_loop_coroutine::TaskPriority::Background,
                                                || async move {
                                                    tracing::debug!("Async task running in background");
                                                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                                                    tracing::debug!("Async task completed");
                                                    Ok(())
                                                },
                                            ).await;
                                        });
                                    }
                            }

                            // 创建WinitWindow包装器用于渲染
                            let winit_window = crate::platform::winit::WinitWindow::from_arc(
                                winit_window_arc.clone(),
                            );

                            // 渲染
                            crate::core::engine::renderer::render(
                                &mut world,
                                &mut renderer,
                                &mut editor_ctx,
                                &mut render_service,
                                &mut render_cache,
                                &winit_window,
                            );

                            // 请求下一帧重绘
                            window.request_redraw();
                        }
                        _ => {}
                    }
                }
                Event::AboutToWait => {
                    // 请求重绘以保持循环运行
                    window.request_redraw();
                }
                _ => {}
            }
        })?;

        Ok(())
    }
}
