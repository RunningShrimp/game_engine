//! 引擎主入口
//!
//! 定义Engine结构和主运行循环

use crate::domain::actor::{
    ActorHandle, ActorSystem, AudioActor, AudioActorMessage, PhysicsActor, PhysicsActorMessage,
    RenderActor, RenderActorMessage,
};
use crate::ecs::{PointLight, PreviousTransform, Sprite, Time, Transform};
use crate::editor::{inspect_world_ui, EditorContext};
#[cfg(feature = "physics_2d")]
use crate::physics::physics3d::{
    init_physics_bodies_3d, physics_step_system_3d, sync_physics_to_transform_system_3d,
    PhysicsWorld3D,
};
#[cfg(feature = "physics_2d")]
use crate::physics::{
    init_physics_bodies, physics_step_system_v2, sync_physics_to_transform_system_v2, ColliderDesc,
    PhysicsDomainService, RigidBodyDesc,
};
use crate::platform::winit::WinitWindow;
use crate::platform::Window;
use crate::platform::{InputBuffer, InputEvent, KeyCode, Modifiers, MouseButton};
use crate::render::wgpu::{GpuPointLight, WgpuRenderer};
use crate::resources::manager::{AssetEvent, AssetServer};
use crate::scripting::{setup_scripting, Script};
use crate::services::audio::start_audio_driver;
use crate::services::render::RenderService;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;

use super::error::{EngineError, EngineResult};
use super::error_aggregator::ErrorAggregator;
use super::resources::{AssetMetrics, Benchmark, LogEvents, RenderStats};
use super::systems::{
    apply_texture_handles, audio_input_system, rotate_system, save_previous_transform_system,
};

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

        let (event_loop, window, mut renderer, mut asset_server, mut editor_ctx) =
            Self::initialize_window_and_renderer()?;

        let (
            mut world,
            mut render_service,
            mut fixed_schedule,
            mut update_schedule,
            mut actor_system,
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
        EventLoop<()>,
        WinitWindow,
        WgpuRenderer<'static>,
        AssetServer,
        EditorContext,
    )> {
        let event_loop = EventLoop::new()
            .map_err(|e| EngineError::EventLoop(format!("Failed to create event loop: {}", e)))?;

        let window = WinitWindow::try_new(&event_loop, (800, 600))
            .ok_or_else(|| EngineError::Window("Failed to create window".to_string()))?;

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
            EditorContext::new(window.raw(), renderer.device(), renderer.config().format);

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
    fn initialize_ecs_and_actors(
        renderer: &WgpuRenderer,
        window: &WinitWindow,
    ) -> EngineResult<(World, RenderService, Schedule, Schedule, ActorSystem)> {
        let mut world = World::new();
        Self::setup_resources(&mut world, renderer);
        setup_scripting(&mut world, Default::default());

        let render_service = RenderService::new();

        // Initialize Actor System
        let mut actor_system = ActorSystem::new();
        let audio_actor_handle =
            actor_system
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

        Self::setup_actor_resources(
            &mut world,
            audio_actor_handle,
            physics_actor_handle,
            render_actor_handle,
        );

        let fixed_schedule = Self::create_fixed_schedule();
        let update_schedule = Self::create_update_schedule();

        tracing::info!(target: "engine", "Actor system initialized with audio, physics, and render actors");

        Ok((
            world,
            render_service,
            fixed_schedule,
            update_schedule,
            actor_system,
        ))
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
        event_loop: EventLoop<()>,
        window: WinitWindow,
        mut world: World,
        mut renderer: WgpuRenderer,
        mut asset_server: AssetServer,
        mut editor_ctx: EditorContext,
        mut render_service: RenderService,
        mut fixed_schedule: Schedule,
        mut update_schedule: Schedule,
        mut actor_system: ActorSystem,
    ) -> EngineResult<()> {
        let mut last_time = std::time::Instant::now();
        let mut accumulator = 0.0;
        let mut render_cache = crate::render::graph::RenderCache::new();

        let result = event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => {
                    let _ = editor_ctx.handle_event(window.raw(), &event);
                    Self::handle_window_event(
                        &event,
                        &mut world,
                        &mut renderer,
                        &mut editor_ctx,
                        &mut render_service,
                        &mut render_cache,
                        &window,
                        elwt,
                    );
                }
                Event::AboutToWait => {
                    // 更新循环：包括ECS系统更新和Actor消息处理
                    // Actor系统通过ECS系统（actor_message_system）异步处理消息
                    Self::update(
                        &mut world,
                        &mut renderer,
                        &mut asset_server,
                        &mut fixed_schedule,
                        &mut update_schedule,
                        &mut last_time,
                        &mut accumulator,
                        &window,
                    );
                }
                _ => {}
            }
        });

        // 清理Actor系统
        tracing::info!(target: "engine", "Shutting down actor system");
        if let Err(e) = actor_system.shutdown() {
            tracing::warn!(target: "engine", "Error shutting down actor system: {:?}", e);
        }

        result.map_err(|e| EngineError::EventLoop(format!("Event loop error: {}", e)))?;

        Ok(())
    }

    /// 设置ECS资源
    ///
    /// 初始化引擎运行所需的所有ECS资源，包括时间、物理状态、输入缓冲区等。
    ///
    /// # 参数
    ///
    /// * `world` - ECS世界
    /// * `renderer` - wgpu渲染器，用于获取视口配置
    fn setup_resources(world: &mut World, renderer: &WgpuRenderer) {
        world.insert_resource(Time::default());
        #[cfg(feature = "physics_2d")]
        {
            world.insert_resource(PhysicsDomainService::new());
            world.insert_resource(PhysicsWorld3D::default());
        }
        world.insert_resource(InputBuffer::default());
        if let Some(audio_q) = start_audio_driver() {
            world.insert_resource(audio_q);
        }
        world.insert_resource(Benchmark {
            enabled: true,
            sprite_count: 0,
        });
        world.insert_resource(crate::ecs::Viewport {
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
    fn setup_actor_resources(
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

    /// 生成演示场景
    ///
    /// 创建一个简单的演示场景，包含精灵和物理对象（如果启用了物理特性）。
    ///
    /// # 参数
    ///
    /// * `world` - ECS世界
    /// * `asset_server` - 资源服务器，用于加载纹理
    fn spawn_demo_scene(world: &mut World, asset_server: &AssetServer) {
        let atlas_path = std::path::Path::new("assets/atlas.png");
        let atlas_handle = asset_server.load_texture(atlas_path);

        #[cfg(feature = "physics_2d")]
        {
            use crate::domain::physics::RigidBodyType;
            // 生成物理地面
            world.spawn((
                Transform {
                    pos: Vec3::new(400.0, 50.0, 0.0),
                    scale: Vec3::new(800.0, 20.0, 1.0),
                    ..Default::default()
                },
                PreviousTransform::default(),
                Sprite {
                    color: [0.5, 0.5, 0.5, 1.0],
                    ..Default::default()
                },
                RigidBodyDesc {
                    body_type: crate::domain::physics::RigidBodyType::Fixed,
                    position: glam::Vec3::new(400.0, 50.0, 0.0),
                    rotation: glam::Quat::IDENTITY,
                },
                ColliderDesc {
                    shape_type: crate::domain::physics::ShapeType::Cuboid,
                    half_extents: glam::Vec3::new(400.0, 10.0, 0.0),
                    radius: 0.0,
                },
            ));

            // 生成下落方块
            for i in 0..10 {
                let mut entity = world.spawn((
                    Transform {
                        pos: Vec3::new(400.0 + i as f32 * 10.0, 500.0 + i as f32 * 50.0, 0.0),
                        scale: Vec3::new(30.0, 30.0, 1.0),
                        ..Default::default()
                    },
                    PreviousTransform::default(),
                    Sprite {
                        color: [1.0, 0.2, 0.2, 1.0],
                        ..Default::default()
                    },
                    RigidBodyDesc {
                        body_type: crate::domain::physics::RigidBodyType::Dynamic,
                        position: glam::Vec3::new(
                            400.0 + i as f32 * 10.0,
                            500.0 + i as f32 * 50.0,
                            0.0,
                        ),
                        rotation: glam::Quat::IDENTITY,
                    },
                    ColliderDesc {
                        shape_type: crate::domain::physics::ShapeType::Cuboid,
                        half_extents: glam::Vec3::new(15.0, 15.0, 0.0),
                        radius: 0.0,
                    },
                ));

                if i == 0 {
                    entity.insert(Script {
                        source: "print('Hello from JS! Entity: ' + entity_id);".to_string(),
                        enabled: true,
                    });
                }
            }
        }

        // 生成网格
        for y in -2..=2 {
            for x in -8..=8 {
                let mut entity = world.spawn((
                    Transform {
                        pos: Vec3::new(400.0 + x as f32 * 30.0, 300.0 + y as f32 * 30.0, 0.0),
                        scale: Vec3::new(24.0, 24.0, 1.0),
                        rot: Quat::from_rotation_z((x as f32 + y as f32) * 0.05),
                    },
                    PreviousTransform::default(),
                    Sprite {
                        color: [0.2 + x as f32 * 0.02, 0.6, 0.3 + y as f32 * 0.02, 0.9],
                        tex_index: 0,
                        normal_tex_index: 0,
                        uv_off: [0.0, 0.0],
                        uv_scale: [1.0, 1.0],
                        layer: if (x + y) % 2 == 0 { 0.0 } else { 1.0 },
                    },
                ));

                if (x + y) % 2 != 0 {
                    entity.insert(atlas_handle.clone());
                }
            }
        }

        // 生成光源
        world.spawn((
            Transform {
                pos: Vec3::new(400.0, 300.0, 0.0),
                ..Default::default()
            },
            PointLight {
                color: [1.0, 0.8, 0.6],
                radius: 300.0,
                intensity: 2.0,
                falloff: 1.0,
            },
        ));
    }

    /// 处理窗口事件
    fn handle_window_event(
        event: &WindowEvent,
        world: &mut World,
        renderer: &mut WgpuRenderer,
        editor_ctx: &mut EditorContext,
        render_service: &mut RenderService,
        render_cache: &mut crate::render::graph::RenderCache,
        window: &WinitWindow,
        elwt: &winit::event_loop::EventLoopWindowTarget<()>,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
                    buf.events.push(InputEvent::WindowCloseRequested);
                }
                elwt.exit();
            }
            WindowEvent::RedrawRequested => {
                Self::render(
                    world,
                    renderer,
                    editor_ctx,
                    render_service,
                    render_cache,
                    window,
                );
            }
            _ => {}
        }

        // 输入事件处理
        Self::handle_input_event(event, world);
    }

    /// 处理输入事件
    fn handle_input_event(event: &WindowEvent, world: &mut World) {
        if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
            match event {
                WindowEvent::Resized(sz) => {
                    buf.events.push(InputEvent::WindowResized {
                        width: sz.width,
                        height: sz.height,
                    });
                }
                WindowEvent::Focused(f) => {
                    buf.events.push(InputEvent::WindowFocused(*f));
                }
                WindowEvent::CursorMoved { position, .. } => {
                    buf.events.push(InputEvent::MouseMoved {
                        x: position.x as f32,
                        y: position.y as f32,
                    });
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let (dx, dy) = match delta {
                        winit::event::MouseScrollDelta::LineDelta(x, y) => (*x, *y),
                        winit::event::MouseScrollDelta::PixelDelta(p) => (p.x as f32, p.y as f32),
                    };
                    buf.events.push(InputEvent::MouseWheel {
                        delta_x: dx,
                        delta_y: dy,
                    });
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    let mb = match button {
                        winit::event::MouseButton::Left => MouseButton::Left,
                        winit::event::MouseButton::Right => MouseButton::Right,
                        winit::event::MouseButton::Middle => MouseButton::Middle,
                        winit::event::MouseButton::Other(b) => MouseButton::Other(*b),
                        winit::event::MouseButton::Back => MouseButton::Other(8),
                        winit::event::MouseButton::Forward => MouseButton::Other(9),
                    };
                    let (x, y) = (0.0f32, 0.0f32);
                    match state {
                        winit::event::ElementState::Pressed => buf
                            .events
                            .push(InputEvent::MouseButtonPressed { button: mb, x, y }),
                        winit::event::ElementState::Released => buf
                            .events
                            .push(InputEvent::MouseButtonReleased { button: mb, x, y }),
                    }
                }
                WindowEvent::KeyboardInput { event, .. } => {
                    let pressed = matches!(event.state, winit::event::ElementState::Pressed);
                    let kc = match &event.logical_key {
                        winit::keyboard::Key::Character(c) => {
                            if c.chars().count() == 1 {
                                buf.events
                                    .push(InputEvent::CharInput(c.chars().next().unwrap()));
                            }
                            KeyCode::Unknown(0)
                        }
                        winit::keyboard::Key::Named(n) => {
                            use winit::keyboard::NamedKey;
                            match n {
                                NamedKey::Escape => KeyCode::Escape,
                                NamedKey::Enter => KeyCode::Enter,
                                NamedKey::Tab => KeyCode::Tab,
                                NamedKey::Space => KeyCode::Space,
                                _ => KeyCode::Unknown(0),
                            }
                        }
                        winit::keyboard::Key::Unidentified(_) | winit::keyboard::Key::Dead(_) => {
                            KeyCode::Unknown(0)
                        }
                    };
                    let m = Modifiers::default();
                    if pressed {
                        buf.events.push(InputEvent::KeyPressed {
                            key: kc,
                            modifiers: m,
                        });
                    } else {
                        buf.events.push(InputEvent::KeyReleased {
                            key: kc,
                            modifiers: m,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    /// 创建固定时间步调度器
    fn create_fixed_schedule() -> Schedule {
        let mut schedule = Schedule::default();
        #[cfg(feature = "physics_2d")]
        {
            schedule.add_systems(
                (
                    save_previous_transform_system,
                    init_physics_bodies,
                    physics_step_system_v2,
                    sync_physics_to_transform_system_v2,
                    init_physics_bodies_3d,
                    physics_step_system_3d,
                    sync_physics_to_transform_system_3d,
                    rotate_system,
                )
                    .chain(),
            );
        }
        #[cfg(not(feature = "physics_2d"))]
        {
            schedule.add_systems((save_previous_transform_system, rotate_system).chain());
        }
        schedule
    }

    /// 创建更新调度器
    fn create_update_schedule() -> Schedule {
        let mut schedule = Schedule::default();
        schedule.add_systems(
            (
                crate::render::instance_batch::batch_collection_system,
                crate::render::instance_batch::batch_visibility_culling_system,
                apply_texture_handles,
                crate::ecs::flipbook_system,
                crate::ecs::tilemap_chunk_system,
                audio_input_system,
                crate::core::systems::actor::actor_message_system,
                crate::core::systems::error_reporting::error_reporting_system,
                crate::core::systems::error_reporting::error_visualization_system,
            )
                .chain(),
        );
        schedule
    }

    /// 渲染帧
    fn render(
        world: &mut World,
        renderer: &mut WgpuRenderer,
        editor_ctx: &mut EditorContext,
        render_service: &mut RenderService,
        render_cache: &mut crate::render::graph::RenderCache,
        window: &WinitWindow,
    ) {
        let _span = tracing::info_span!(target: "render", "frame").entered();

        // Editor UI
        editor_ctx.begin_frame(window.raw());
        inspect_world_ui(&editor_ctx.context, world);
        let (egui_shapes, egui_renderer) = editor_ctx.end_frame(window.raw());
        let pixels_per_point = window.raw().scale_factor() as f32;

        // Render with frustum culling
        let (layer_tree, culled, total) = crate::render::graph::build_from_world_culled(world);
        render_cache.culled_count = culled;
        render_cache.total_count = total;
        let _instances = render_cache.update(layer_tree);
        // 实例数量已由render_cache.update()内部记录

        // Extract Lights
        let mut lights = Vec::new();
        let mut query = world.query::<(&Transform, &PointLight)>();
        for (t, l) in query.iter(world) {
            lights.push(GpuPointLight {
                pos: [t.pos.x, t.pos.y],
                color: l.color,
                radius: l.radius,
                intensity: l.intensity,
                falloff: l.falloff,
                _pad: [0.0, 0.0],
            });
        }
        renderer.set_lights(lights);

        // Build Scene (PBR)
        let scene = render_service.build_pbr_scene(world);

        // Camera
        let mut view_proj = glam::Mat4::IDENTITY.to_cols_array_2d();
        let mut camera_pos = [0.0; 3];
        let mut query_cam = world.query::<(&Transform, &crate::ecs::Camera)>();
        for (t, c) in query_cam.iter(world) {
            if c.is_active {
                camera_pos = t.pos.to_array();
                let view = glam::Mat4::from_rotation_translation(t.rot, t.pos).inverse();
                let proj = match c.projection {
                    crate::ecs::Projection::Orthographic { scale, near, far } => {
                        let aspect =
                            renderer.config().width as f32 / renderer.config().height as f32;
                        glam::Mat4::orthographic_rh(
                            -aspect * scale,
                            aspect * scale,
                            -scale,
                            scale,
                            near,
                            far,
                        )
                    }
                    crate::ecs::Projection::Perspective {
                        fov,
                        aspect,
                        near,
                        far,
                    } => glam::Mat4::perspective_rh(fov, aspect, near, far),
                };
                view_proj = (proj * view).to_cols_array_2d();
                break;
            }
        }

        if let Some(mut bm) =
            world.get_resource_mut::<crate::render::instance_batch::BatchManager>()
        {
            renderer.upload_batches(&mut bm);
            if let Err(e) = render_service.paint_pbr(
                renderer,
                &mut bm,
                &scene,
                view_proj,
                camera_pos,
                Some(egui_renderer),
                &egui_shapes,
                pixels_per_point,
            ) {
                tracing::warn!(target: "render", "Render error: {}", e);
            }
        }

        // Flush material pending updates
        let updates = if let Some(mut pend) =
            world.get_resource_mut::<crate::resources::manager::MaterialPendingUpdates>()
        {
            pend.take_all()
        } else {
            Vec::new()
        };
        if !updates.is_empty() {
            if let Some(mut reg) =
                world.get_resource_mut::<crate::resources::manager::MaterialRegistry>()
            {
                if let Some(ref pbr) = renderer.pbr_renderer {
                    for (id, mat) in updates {
                        reg.update_material_params(
                            renderer.device(),
                            renderer.queue(),
                            pbr,
                            id,
                            &mat,
                        );
                    }
                }
            }
        }

        // Update stats
        Self::update_render_stats(world, renderer, culled, total, window);
    }

    /// 更新渲染统计
    fn update_render_stats(
        world: &mut World,
        renderer: &WgpuRenderer,
        culled: u32,
        total: u32,
        window: &WinitWindow,
    ) {
        if let Some((_t0, dt)) = renderer.gpu_timings_ms() {
            if let Some(mut stats) = world.get_resource_mut::<RenderStats>() {
                stats.gpu_pass_ms = Some(dt);
            }
        }
        let (dc, ic) = renderer.draw_stats();
        let bm_stats = world
            .get_resource::<crate::render::instance_batch::BatchManager>()
            .map(|bm| bm.stats);
        if let Some(mut stats) = world.get_resource_mut::<RenderStats>() {
            stats.draw_calls = dc;
            stats.instances = ic;
            stats.passes = renderer.pass_count();
            stats.culled_objects = culled;
            stats.total_objects = total;
            if let Some(bms) = bm_stats {
                stats.batch_total = bms.total_batches;
                stats.batch_instances = bms.total_instances;
                stats.batch_saved_draw_calls = bms.saved_draw_calls;
                stats.batch_small_draw_calls = bms.small_draw_calls;
                stats.batch_visible_batches = bms.visible_batches;
            }
            let (upload, main, ui) = renderer.stage_timings_ms();
            stats.upload_ms = upload;
            stats.main_ms = main;
            stats.ui_ms = ui;
            stats.offscreen_ms = renderer.offscreen_timing_ms();

            // 性能警告
            if let Some(u) = stats.upload_ms {
                if u > 2.0 {
                    stats.alerts_upload += 1;
                }
            }
            if let Some(m) = stats.main_ms {
                if m > 16.7 {
                    stats.alerts_main += 1;
                }
            }
            if let Some(u) = stats.ui_ms {
                if u > 4.0 {
                    stats.alerts_ui += 1;
                }
            }
            if let Some(o) = stats.offscreen_ms {
                if o > 8.0 {
                    stats.alerts_offscreen += 1;
                }
            }

            // 写入CSV日志
            let path = std::env::temp_dir().join("render_stats.csv");
            let _ = (|| {
                let mut f = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .ok()?;
                let line = format!(
                    "{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
                    dc,
                    ic,
                    stats.passes,
                    stats.upload_ms.unwrap_or(0.0),
                    stats.main_ms.unwrap_or(0.0),
                    stats.ui_ms.unwrap_or(0.0),
                    stats.offscreen_ms.unwrap_or(0.0),
                    window.scale_factor(),
                    stats.batch_total,
                    stats.batch_instances,
                    stats.batch_saved_draw_calls,
                    stats.batch_small_draw_calls,
                    stats.batch_visible_batches
                );
                use std::io::Write;
                let _ = f.write_all(line.as_bytes());
                Some(())
            })();
        }
    }

    /// 更新循环
    fn update(
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
        Self::process_asset_events(world, asset_server, renderer);
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
        if let Some(mut stats) = world.get_resource_mut::<RenderStats>() {
            stats.update_time_ms = total_update_time.as_secs_f32() * 1000.0;
            stats.asset_processing_time_ms = asset_time.as_secs_f32() * 1000.0;
            stats.fixed_update_time_ms = fixed_time.as_secs_f32() * 1000.0;
            stats.variable_update_time_ms = update_schedule_time.as_secs_f32() * 1000.0;
        }
    }

    /// 处理资源加载事件
    fn process_asset_events(
        world: &mut World,
        asset_server: &mut AssetServer,
        renderer: &mut WgpuRenderer,
    ) {
        let events = asset_server.update(renderer);
        if events.is_empty() {
            return;
        }

        for event in events {
            // 处理图集加载
            if let AssetEvent::AtlasLoaded(h, _) = &event {
                if let Some(atlas) = h.get() {
                    let mut ts =
                        world.get_resource_or_insert_with::<crate::ecs::TileSet>(Default::default);
                    for (name, (uv_off, uv_scale)) in atlas.sprites.iter() {
                        ts.tiles.insert(name.clone(), (*uv_off, *uv_scale));
                    }
                }
            }

            // 更新资源指标
            if let Some(mut am) = world.get_resource_mut::<AssetMetrics>() {
                match &event {
                    AssetEvent::TextureLoaded(_, ms) => {
                        am.last_latency_ms = Some(*ms);
                        am.textures_loaded += 1;
                    }
                    AssetEvent::AtlasLoaded(_, ms) => {
                        am.last_latency_ms = Some(*ms);
                        am.atlases_loaded += 1;
                    }
                    _ => {}
                }
            }

            // 记录日志
            if let Some(mut logs) = world.get_resource_mut::<LogEvents>() {
                let msg = match &event {
                    AssetEvent::TextureLoaded(_, ms) => format!("TextureLoaded {:.1}ms", ms),
                    AssetEvent::AtlasLoaded(_, ms) => format!("AtlasLoaded {:.1}ms", ms),
                    AssetEvent::TextureFailed(_, e) => format!("TextureFailed {}", e),
                    AssetEvent::AtlasFailed(_, e) => format!("AtlasFailed {}", e),
                    AssetEvent::GltfLoaded(_, ms) => format!("GltfLoaded {:.1}ms", ms),
                    AssetEvent::GltfFailed(_, e) => format!("GltfFailed {}", e),
                };
                if logs.entries.len() >= logs.capacity {
                    logs.entries.pop_front();
                }
                logs.entries.push_back(msg);
            }

            // GLTF 导入：在同帧构建网格与纹理绑定并注入世界
            if let AssetEvent::GltfLoaded(handle, _) = &event {
                crate::resources::manager::import_gltf_to_world(world, renderer, handle);
            }
        }
    }
}
