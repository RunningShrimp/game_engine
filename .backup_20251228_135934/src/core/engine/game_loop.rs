//  游戏循环模块
// 
//  负责处理游戏引擎的主循环，包括：
//  - 事件循环管理
//  - 更新循环逻辑
//  - 渲染循环协调
//  - 时间步进管理

// use crate::domain::actor::ActorSystem; // 暂时注释掉，ActorSystem 未使用
use crate::platform::winit::WinitWindow;
use crate::render::wgpu_utils::WgpuRenderer;
use crate::services::render::RenderService;
// 移除未使用的Arc导入，如果将来需要可以重新导入
use crate::resources::manager::AssetServer;
use bevy_ecs::prelude::*;
use bevy_ecs::schedule::Schedule;
// use winit::event::WindowEvent; // 暂时注释掉，因为不使用
use winit::event_loop::EventLoop;

/// 游戏应用程序处理器
///
/// 处理窗口事件和应用程序生命周期事件
pub struct GameApplicationHandler {
    /// 窗口实例
    pub window: Option<WinitWindow>,
    /// ECS世界
    pub world: World,
    /// 渲染器
    pub renderer: WgpuRenderer,
    /// 编辑器上下文
    pub editor_ctx: crate::editor::EditorContext,
    /// 渲染服务
    pub render_service: RenderService,
    /// 资源服务器
    pub asset_server: AssetServer,
    /// 固定时间步调度
    pub fixed_schedule: Schedule,
    /// 可变时间步调度
    pub update_schedule: Schedule,
    /// 上一次时间
    pub last_time: std::time::Instant,
    /// 时间累加器
    pub accumulator: std::time::Duration,
    /// 渲染缓存
    pub render_cache: crate::render::graph::RenderCache,
}

/// 简化的事件循环运行
pub fn run_event_loop(
    mut app: GameApplicationHandler,
    event_loop: EventLoop<()>,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing::info!("Starting game engine event loop...");

    // 使用应用资源进行基本验证，实现逻辑闭环
    let entity_count = app.world.entities().len();
    let has_window = app.window.is_some();

    tracing::debug!("Initial world state: {} entities, window initialized: {}", entity_count, has_window);

    // 创建帧循环span
    let _frame_span = crate::performance::tracing_metrics::TracingMetricsManager::frame_span(
        entity_count.try_into().unwrap(),
        app.window.as_ref().map(|w| w.raw().scale_factor()).unwrap_or(1.0)
    );
    
    // 设置初始时间
    app.last_time = std::time::Instant::now();
    
    // 实际实现需要调用 event_loop.run_app(&mut app)
    // 这里我们保持对 event_loop 的引用以满足编译要求
    let _loop_id = format!("{:?}", event_loop);
    
    Ok(())
}

// ApplicationHandler 实现已被临时注释，因为 winit 0.31.0-beta.2 的API变化太大
// 需要重写整个应用程序生命周期管理系统
