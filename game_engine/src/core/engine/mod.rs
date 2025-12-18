//! 游戏引擎核心模块
//!
//! 这个模块包含了游戏引擎的核心功能，被重构为多个子模块以提高可维护性。

pub mod initialization;
pub mod game_loop;
pub mod demo_scene;
pub mod input_handler;
pub mod renderer;
pub mod asset_processor;

use crate::domain::actor::ActorSystem;
use crate::services::render::RenderService;
use crate::platform::winit::WinitWindow;
// 移除未使用的InputBuffer导入，如果将来需要可以重新导入
use crate::render::wgpu_utils::WgpuRenderer;
use crate::resources::manager::AssetServer;
use bevy_ecs::prelude::*;
use winit::event_loop::EventLoop;

use super::error::EngineResult;
use crate::editor::EditorContext;

/// 游戏引擎主结构
///
/// `Engine` 是游戏引擎的核心入口点，负责：
/// - 初始化所有子系统（渲染、物理、音频等）
/// - 管理主循环
/// - 协调各系统之间的交互
///
/// # 示例
///
/// ```no_run
/// use game_engine::core::Engine;
///
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     Engine::run()?;
///     Ok(())
/// }
/// ```
///
/// # 生命周期
///
/// 引擎的生命周期包括：
/// 1. **初始化阶段**：创建窗口、初始化渲染器、加载资源
/// 2. **运行阶段**：执行游戏循环，更新系统
/// 3. **关闭阶段**：清理资源，关闭子系统
pub struct Engine;

impl Engine {
    /// 运行引擎主循环
    pub fn run() -> EngineResult<()> {
        Self::initialize_logging();

        let (event_loop, window, mut renderer, asset_server, editor_ctx) =
            Self::initialize_window_and_renderer()?;

        let (
            mut world,
            render_service,
            fixed_schedule,
            update_schedule,
            actor_system,
        ) = Self::initialize_ecs_and_actors(&mut renderer, &window)?;

        Self::spawn_demo_scene(&mut world, &asset_server);

        Self::run_event_loop(
            event_loop,
            window,
            world,
            renderer,
            asset_server,
            editor_ctx,
            render_service,
            fixed_schedule,
            update_schedule,
            actor_system,
        )?;

        tracing::info!(target: "engine", "Engine shutting down");
        Ok(())
    }

    /// 初始化日志系统
    ///
    /// 配置tracing日志框架，设置环境变量过滤器。
    /// 日志级别可以通过`RUST_LOG`环境变量控制。
    fn initialize_logging() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
        tracing::info!(target: "engine", "Engine starting");
    }

    /// 初始化窗口和渲染器
    ///
    /// 创建事件循环、窗口、wgpu渲染器和资源服务器。
    ///
    /// # 返回
    ///
    /// 返回包含事件循环、窗口、渲染器、资源服务器和编辑器上下文的元组。
    ///
    /// # 错误
    ///
    /// 如果窗口创建失败或渲染器初始化失败，返回相应的错误。
    fn initialize_window_and_renderer() -> EngineResult<(
        EventLoop,
        WinitWindow,
        WgpuRenderer<'static>,
        AssetServer,
        EditorContext,
    )> {
        initialization::initialize_window_and_renderer()
    }

    /// 初始化ECS和Actor系统
    ///
    /// 创建ECS世界，设置资源，初始化脚本系统、Actor系统和调度器。
    ///
    /// # 参数
    ///
    /// * `renderer` - wgpu渲染器引用
    /// * `window` - 窗口引用
    ///
    /// # 返回
    ///
    /// 返回包含ECS世界、渲染服务、固定时间步调度器、更新调度器和Actor系统的元组。
    ///
    /// # 错误
    ///
    /// 如果Actor注册失败，返回相应的错误。
    fn initialize_ecs_and_actors(
        renderer: &WgpuRenderer,
        window: &WinitWindow,
    ) -> EngineResult<(World, RenderService, Schedule, Schedule, ActorSystem)> {
        initialization::initialize_ecs_and_actors(renderer, window)
    }

    /// 生成演示场景
    ///
    /// 创建一个简单的演示场景，包含精灵和物理对象（如果启用了物理特性）。
    ///
    /// # 参数
    ///
    /// * `world` - ECS世界
    /// * `asset_server` - 资源服务器，用于加载纹理
    fn spawn_demo_scene(world: &mut World, asset_server: &AssetServer) {
        demo_scene::spawn_demo_scene(world, asset_server)
    }

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
    fn run_event_loop(
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
        game_loop::run_event_loop(
            event_loop,
            window,
            world,
            renderer,
            asset_server,
            editor_ctx,
            render_service,
            fixed_schedule,
            update_schedule,
            actor_system,
        )
    }
}