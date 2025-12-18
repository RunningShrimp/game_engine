//! 引擎初始化模块
//!
//! 负责游戏引擎各个子系统的初始化工作，包括：
//! - 日志系统初始化
//! - 窗口和渲染器初始化
//! - ECS和Actor系统初始化
//! - 资源设置

use crate::domain::actor::{
    ActorHandle, ActorSystem, AudioActor, AudioActorMessage, PhysicsActor, PhysicsActorMessage,
    RenderActor, RenderActorMessage,
};
use crate::ecs::{Time, Viewport};
use crate::platform::winit::WinitWindow;
use crate::platform::{InputBuffer, InputActions};
use crate::render::wgpu_utils::WgpuRenderer;
use crate::resources::manager::AssetServer;
use crate::scripting::setup_scripting;
use crate::services::audio::start_audio_driver;
use crate::services::render::RenderService;

/// 窗口资源，存储窗口相关信息
#[derive(Debug)]
pub struct WindowResource {
    pub id: winit::window::WindowId,
    pub inner_size: winit::dpi::PhysicalSize<u32>,
    pub outer_size: winit::dpi::PhysicalSize<u32>,
    pub scale_factor: f64,
}
use bevy_ecs::prelude::*;
use winit::event_loop::EventLoop;

use crate::core::error::{EngineError, EngineResult};
use crate::core::error_aggregator::ErrorAggregator;
use crate::core::resources::{AssetMetrics, Benchmark, LogEvents, RenderStats};
use crate::editor::EditorContext;

#[cfg(feature = "physics_2d")]
use crate::physics::physics3d::PhysicsWorld3D;
#[cfg(feature = "physics_2d")]
use crate::physics::PhysicsDomainService;

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
pub fn initialize_window_and_renderer() -> EngineResult<(
    EventLoop,
    WinitWindow,
    WgpuRenderer<'static>,
    AssetServer,
    EditorContext,
)> {
    let event_loop = EventLoop::new()
        .map_err(|e| EngineError::EventLoop(format!("Failed to create event loop: {}", e)))?;

    let window = WinitWindow::try_new(&event_loop, (800, 600))
        .ok_or(EngineError::Window("Failed to create window".to_string()))?;

    // 创建渲染器：使用window.raw()获取窗口引用
    // 注意：由于WgpuRenderer需要'static生命周期，我们需要确保窗口引用在整个生命周期内有效
    // 这里使用unsafe来延长生命周期，因为window会在整个引擎生命周期内存在
    let window_raw = window.raw();
    let renderer = unsafe {
        let window_ref: &'static _ = std::mem::transmute(window_raw);
        pollster::block_on(async { WgpuRenderer::new(window_ref).await })
            .map_err(EngineError::Render)?
    };

    let asset_server = AssetServer::new();
    let editor_ctx =
        EditorContext::new(&window, renderer.device(), renderer.config().format);

    Ok((event_loop, window, renderer, asset_server, editor_ctx))
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
pub fn initialize_ecs_and_actors(
    renderer: &WgpuRenderer,
    window: &WinitWindow,
) -> EngineResult<(World, RenderService, Schedule, Schedule, ActorSystem)> {
    let mut world = World::new();
    setup_resources(&mut world, renderer);
    setup_scripting(&mut world, Default::default());
    
    // 存储窗口信息到世界资源中，供其他系统使用
    world.insert_resource(WindowResource {
        id: window.id(),
        inner_size: window.inner_size(),
        outer_size: window.outer_size(),
        scale_factor: window.scale_factor(),
    });

    let render_service = RenderService::new();

    // Initialize Actor System
    let mut actor_system = ActorSystem::new();
    let audio_actor_handle = actor_system
        .register("audio", AudioActor::new())
        .map_err(|e| {
            EngineError::General(format!("Failed to register audio actor: {:?}", e))
        })?;
    let physics_actor_handle = actor_system
        .register("physics", PhysicsActor::new())
        .map_err(|e| {
            EngineError::General(format!("Failed to register physics actor: {:?}", e))
        })?;
    let render_actor_handle = actor_system
        .register("render", RenderActor::new())
        .map_err(|e| {
            EngineError::General(format!("Failed to register render actor: {:?}", e))
        })?;

    setup_actor_resources(
        &mut world,
        audio_actor_handle,
        physics_actor_handle,
        render_actor_handle,
    );

    let fixed_schedule = create_fixed_schedule();
    let update_schedule = create_update_schedule();

    tracing::info!(target: "engine", "Actor system initialized with audio, physics, and render actors");

    Ok((
        world,
        render_service,
        fixed_schedule,
        update_schedule,
        actor_system,
    ))
}

/// 设置ECS资源
///
/// 初始化引擎运行所需的所有ECS资源，包括时间、物理状态、输入缓冲区等。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `renderer` - wgpu渲染器，用于获取视口配置
pub fn setup_resources(world: &mut World, renderer: &WgpuRenderer) {
    world.insert_resource(Time::default());
    #[cfg(feature = "physics_2d")]
    {
        world.insert_resource(PhysicsDomainService::new());
        world.insert_resource(PhysicsWorld3D::default());
    }
    world.insert_resource(InputBuffer::default());
    world.insert_resource(InputActions::default());
    if let Some(audio_q) = start_audio_driver() {
        world.insert_resource(audio_q);
    }
    world.insert_resource(Benchmark {
        enabled: true,
        sprite_count: 0,
    });
    world.insert_resource(Viewport {
        width: renderer.config().width,
        height: renderer.config().height,
    });
    world.insert_resource(AssetMetrics::default());
    world.insert_resource(crate::ecs::TileChunkConfig { size: [16, 16] });
    world.insert_resource(LogEvents {
        entries: std::collections::VecDeque::new(),
        filter: String::new(),
        capacity: 200,
    });
    world.insert_resource(RenderStats::default());
    world.insert_resource(crate::render::instance_batch::BatchManager::default());
    world.insert_resource(crate::render::instance_batch::BatchManager::default());
    world.insert_resource(crate::ecs::TileEntityPool::default());
    // 初始化错误聚合器
    world.insert_resource(ErrorAggregator::new());
}

/// 设置Actor系统资源
///
/// 将Actor句柄注册为ECS资源，供系统使用。
///
/// # 参数
///
/// * `world` - ECS世界
/// * `audio_handle` - 音频Actor句柄
/// * `physics_handle` - 物理Actor句柄
/// * `render_handle` - 渲染Actor句柄
pub fn setup_actor_resources(
    world: &mut World,
    audio_handle: ActorHandle<AudioActorMessage>,
    physics_handle: ActorHandle<PhysicsActorMessage>,
    render_handle: ActorHandle<RenderActorMessage>,
) {
    // 将Actor句柄存储为ECS资源，供系统使用
    world.insert_resource(audio_handle);
    world.insert_resource(physics_handle);
    world.insert_resource(render_handle);
}

/// 创建固定时间步调度器
pub fn create_fixed_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    #[cfg(feature = "physics_2d")]
    {
        schedule.add_systems(
            (
                crate::core::systems::save_previous_transform_system,
                crate::physics::init_physics_bodies,
                crate::physics::physics_step_system_v2,
                crate::physics::sync_physics_to_transform_system_v2,
                crate::physics::physics3d::init_physics_bodies_3d,
                crate::physics::physics3d::physics_step_system_3d,
                crate::physics::physics3d::sync_physics_to_transform_system_3d,
                crate::core::systems::rotate_system,
            )
                .chain(),
        );
    }
    #[cfg(not(feature = "physics_2d"))]
    {
        schedule.add_systems((crate::core::systems::save_previous_transform_system, crate::core::systems::rotate_system).chain());
    }
    schedule
}

/// 创建更新调度器
pub fn create_update_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            crate::render::instance_batch::batch_collection_system,
            crate::render::instance_batch::batch_visibility_culling_system,
            crate::core::systems::apply_texture_handles,
            crate::ecs::flipbook_system,
            crate::ecs::tilemap_chunk_system,
            crate::core::systems::audio_input_system,
            crate::core::systems::actor::actor_message_system,
            crate::core::systems::error_reporting::error_reporting_system,
            crate::core::systems::error_reporting::error_visualization_system,
            crate::core::systems::movement_system,
        )
            .chain(),
    );
    schedule
}