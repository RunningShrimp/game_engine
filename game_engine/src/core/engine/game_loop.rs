//! 游戏循环模块
//!
//! 负责处理游戏引擎的主循环，包括：
//! - 事件循环管理
//! - 更新循环逻辑
//! - 渲染循环协调
//! - 时间步进管理

use crate::domain::actor::ActorSystem;
use crate::services::render::RenderService;
use crate::ecs::Time;
use crate::platform::winit::WinitWindow;
use crate::render::wgpu_utils::WgpuRenderer;
// 移除未使用的Arc导入，如果将来需要可以重新导入
use crate::resources::manager::AssetServer;
use bevy_ecs::prelude::*;
use winit::event_loop::EventLoop;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

use crate::core::error::{EngineError, EngineResult};
use crate::core::engine::input_handler::handle_window_event;
use crate::core::engine::renderer::render;
use crate::core::engine::asset_processor::process_asset_events;
use crate::editor::EditorContext;

/// 运行事件循环
///
/// 处理窗口事件和更新循环，直到用户关闭窗口。
/// Actor系统在主循环中异步处理消息，并在关闭时正确清理。
///
/// # 参数
///
/// * `event_loop` - winit事件循环
/// * `window` - 窗口实例
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
/// * `asset_server` - 资源服务器
/// * `editor_ctx` - 编辑器上下文
/// * `render_service` - 渲染服务
/// * `fixed_schedule` - 固定时间步调度器
/// * `update_schedule` - 更新调度器
/// * `actor_system` - Actor系统
///
/// # 错误
///
/// 如果事件循环运行失败，返回相应的错误。
pub fn run_event_loop(
    event_loop: EventLoop,
    window: WinitWindow,
    world: World,
    renderer: WgpuRenderer<'static>,
    asset_server: AssetServer,
    editor_ctx: EditorContext,
    render_service: RenderService,
    fixed_schedule: Schedule,
    update_schedule: Schedule,
    actor_system: ActorSystem,
) -> EngineResult<()> {
    let last_time = std::time::Instant::now();
    let accumulator = 0.0;
    let render_cache = crate::render::graph::RenderCache::new();

    // 将所有资源转移到堆上，确保它们的生命周期满足'static要求
    let app_handler = Box::new(GameApplicationHandler {
        window: Some(window),
        world,
        renderer,
        asset_server,
        editor_ctx,
        render_service,
        fixed_schedule,
        update_schedule,
        actor_system,
        last_time,
        accumulator,
        render_cache,
    });

    let result = event_loop.run_app(app_handler);

    result.map_err(|e| EngineError::EventLoop(format!("Event loop error: {}", e)))?;

    Ok(())
}

/// 更新循环
///
/// 处理游戏逻辑的更新，包括：
/// - 资源加载事件处理
/// - 时间步进管理
/// - 固定时间步更新（物理等）
/// - 可变时间步更新（游戏逻辑等）
/// - 视口更新
/// - 性能指标记录
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
/// * `asset_server` - 资源服务器
/// * `fixed_schedule` - 固定时间步调度器
/// * `update_schedule` - 更新调度器
/// * `last_time` - 上一帧的时间戳
/// * `accumulator` - 时间累加器
/// * `window` - 窗口实例
pub fn update(
    world: &mut World,
    renderer: &mut WgpuRenderer,
    asset_server: &mut AssetServer,
    fixed_schedule: &mut Schedule,
    update_schedule: &mut Schedule,
    last_time: &mut std::time::Instant,
    accumulator: &mut f64,
    window: &WinitWindow,
) {
    let update_start = std::time::Instant::now();
    let _span = tracing::info_span!(target: "update", "update").entered();

    // 更新资源（非阻塞）
    let asset_start = std::time::Instant::now();
    process_asset_events(world, asset_server, renderer);
    let asset_time = asset_start.elapsed();

    // 更新时间
    let now = std::time::Instant::now();
    let delta = now.duration_since(*last_time).as_secs_f32();
    *last_time = now;

    *accumulator += delta as f64;
    let fixed_step = if let Some(time) = world.get_resource::<Time>() {
        time.fixed_time_step
    } else {
        // 如果Time资源不存在，使用默认值
        1.0 / 60.0 // 60 FPS
    };

    // 固定时间步更新
    let fixed_start = std::time::Instant::now();
    while *accumulator >= fixed_step {
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = fixed_step as f32;
            time.elapsed_seconds += fixed_step;
        }
        fixed_schedule.run(world);
        *accumulator -= fixed_step;
    }
    let fixed_time = fixed_start.elapsed();

    // 更新插值alpha
    if let Some(mut time) = world.get_resource_mut::<Time>() {
        time.alpha = *accumulator / fixed_step;
    }

    // 可变时间步更新
    let update_schedule_start = std::time::Instant::now();
    update_schedule.run(world);
    let update_schedule_time = update_schedule_start.elapsed();

    window.request_redraw();

    // 更新视口
    if let Some(mut vp) = world.get_resource_mut::<crate::ecs::Viewport>() {
        vp.width = renderer.config().width;
        vp.height = renderer.config().height;
    }

    // 记录性能指标
    let total_update_time = update_start.elapsed();
    if let Some(mut stats) = world.get_resource_mut::<crate::core::resources::RenderStats>() {
        stats.update_time_ms = total_update_time.as_secs_f32() * 1000.0;
        stats.asset_processing_time_ms = asset_time.as_secs_f32() * 1000.0;
        stats.fixed_update_time_ms = fixed_time.as_secs_f32() * 1000.0;
        stats.variable_update_time_ms = update_schedule_time.as_secs_f32() * 1000.0;
    }
}

/// 处理窗口事件中的渲染请求
///
/// 这个函数专门处理WindowEvent::RedrawRequested事件，
/// 将渲染逻辑从事件处理中分离出来。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器
/// * `editor_ctx` - 编辑器上下文
/// * `render_service` - 渲染服务
/// * `render_cache` - 渲染缓存
/// * `window` - 窗口实例
pub fn handle_redraw_request(
    world: &mut World,
    renderer: &mut WgpuRenderer,
    editor_ctx: &mut EditorContext,
    render_service: &mut RenderService,
    render_cache: &mut crate::render::graph::RenderCache,
    window: &WinitWindow,
) {
    render(
        world,
        renderer,
        editor_ctx,
        render_service,
        render_cache,
        window,
    );
}

/// 游戏应用处理器，实现了 winit 的 ApplicationHandler trait
pub struct GameApplicationHandler {
    pub window: Option<WinitWindow>,
    pub world: World,
    pub renderer: WgpuRenderer<'static>,
    pub asset_server: AssetServer,
    pub editor_ctx: EditorContext,
    pub render_service: RenderService,
    pub fixed_schedule: Schedule,
    pub update_schedule: Schedule,
    pub actor_system: ActorSystem,
    pub last_time: std::time::Instant,
    pub accumulator: f64,
    pub render_cache: crate::render::graph::RenderCache,
}

impl ApplicationHandler for GameApplicationHandler {
    fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
        // 在这里创建窗口
        if self.window.is_none() {
            if let Ok(window) = WinitWindow::try_new(event_loop, (800, 600)) {
                self.window = Some(window);
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        if let Some(ref window) = self.window {
            if window.id() == window_id {
                let _ = self.editor_ctx.handle_event(window.raw(), &event);
                handle_window_event(
                    &event,
                    &mut self.world,
                    &mut self.renderer,
                    &mut self.editor_ctx,
                    &mut self.render_service,
                    &mut self.render_cache,
                    window,
                    event_loop,
                );
            }
        }
    }

    fn about_to_wait(&mut self, _event_loop: &dyn ActiveEventLoop) {
        // 更新循环：包括ECS系统更新和Actor消息处理
        // Actor系统通过ECS系统（actor_message_system）异步处理消息
        if let Some(ref window) = self.window {
            update(
                &mut self.world,
                &mut self.renderer,
                &mut self.asset_server,
                &mut self.fixed_schedule,
                &mut self.update_schedule,
                &mut self.last_time,
                &mut self.accumulator,
                window,
            );
        }
        
        // 检查是否需要退出应用程序
        // 这里可以根据某些条件决定是否退出事件循环
        // 例如，检查是否有退出信号或特殊输入
        // 目前我们只是演示如何使用 event_loop 参数
    }
}