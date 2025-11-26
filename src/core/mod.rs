use crate::render::wgpu::{WgpuRenderer, GpuPointLight};
use crate::resources::manager::{AssetServer, Handle, AssetEvent};
use crate::platform::winit::WinitWindow;
use crate::ecs::{Transform, PreviousTransform, Sprite, Time, PointLight};
use crate::physics::{PhysicsWorld, RigidBodyDesc, ColliderDesc, ShapeType, init_physics_bodies, physics_step_system, sync_physics_to_transform_system};
use crate::editor::{EditorContext, inspect_world_ui};
use crate::services::render::RenderService;
use crate::scripting::{setup_scripting, Script};
use bevy_ecs::prelude::*;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use glam::{Vec3, Quat};
use rapier2d::prelude::RigidBodyType;

pub struct Engine;

impl Engine {
    pub fn run() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .try_init();
        tracing::info!(target: "engine", "Engine starting");
        let event_loop = EventLoop::new().unwrap();
        let window = WinitWindow::new(&event_loop, (800, 600));
        let win_clone = window.clone();
        let mut renderer = pollster::block_on(WgpuRenderer::new(win_clone.raw()));
        let asset_server = AssetServer::new();
        
        let mut editor_ctx = EditorContext::new(window.raw(), renderer.device(), renderer.config().format);

        // --- ECS Setup ---
        let mut world = World::new();
        world.insert_resource(Time::default());
        world.insert_resource(PhysicsWorld::default());
        world.insert_resource(Benchmark { enabled: true, sprite_count: 0 });
        world.insert_resource(crate::ecs::Viewport { width: renderer.config().width, height: renderer.config().height });
        world.insert_resource(AssetMetrics::default());
        world.insert_resource(LogEvents { entries: std::collections::VecDeque::new(), filter: String::new(), capacity: 200 });
        world.insert_resource(RenderStats::default());
        // Events<AssetEvent> not required; handle inline
        
        setup_scripting(&mut world);

        // Initialize Services (Anemic Model: Systems call Services)
        let mut render_service = RenderService::new();

        let mut fixed_schedule = Schedule::default();
        fixed_schedule.add_systems((
            save_previous_transform_system,
            init_physics_bodies,
            physics_step_system,
            sync_physics_to_transform_system,
            rotate_system,
            // scripting_system
        ).chain());
        
        let mut update_schedule = Schedule::default();
        update_schedule.add_systems((
            apply_texture_handles,
            crate::ecs::flipbook_system,
            crate::ecs::tilemap_chunk_system,
            // benchmark_system,
        ).chain());

        // --- Asset Loading & Entity Spawning ---
        let atlas_path = std::path::Path::new("assets/atlas.png");
        let atlas_handle = asset_server.load_texture(atlas_path);
        
        // Spawn Physics Ground
        world.spawn((
            Transform { pos: Vec3::new(400.0, 50.0, 0.0), scale: Vec3::new(800.0, 20.0, 1.0), ..Default::default() },
            PreviousTransform::default(),
            Sprite { color: [0.5, 0.5, 0.5, 1.0], ..Default::default() },
            RigidBodyDesc { body_type: RigidBodyType::Fixed, position: [400.0, 50.0] },
            ColliderDesc { shape_type: ShapeType::Cuboid, half_extents: [400.0, 10.0], radius: 0.0 }
        ));

        // Spawn Falling Boxes
        for i in 0..10 {
            let mut entity = world.spawn((
                Transform { pos: Vec3::new(400.0 + i as f32 * 10.0, 500.0 + i as f32 * 50.0, 0.0), scale: Vec3::new(30.0, 30.0, 1.0), ..Default::default() },
                PreviousTransform::default(),
                Sprite { color: [1.0, 0.2, 0.2, 1.0], ..Default::default() },
                RigidBodyDesc { body_type: RigidBodyType::Dynamic, position: [400.0 + i as f32 * 10.0, 500.0 + i as f32 * 50.0] },
                ColliderDesc { shape_type: ShapeType::Cuboid, half_extents: [15.0, 15.0], radius: 0.0 }
            ));
            
            if i == 0 {
                entity.insert(Script { source: "print('Hello from JS! Entity: ' + entity_id);".to_string(), enabled: true });
            }
        }

        // Spawn Grid (Visual only)
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
                    }
                ));
                
                if (x + y) % 2 != 0 {
                    entity.insert(atlas_handle.clone());
                }
            }
        }

        // Spawn Light
        world.spawn((
            Transform { pos: Vec3::new(400.0, 300.0, 0.0), ..Default::default() },
            PointLight {
                color: [1.0, 0.8, 0.6],
                radius: 300.0,
                intensity: 2.0,
                falloff: 1.0,
            }
        ));

        // --- Event Loop ---
        let mut last_time = std::time::Instant::now();
        let mut accumulator = 0.0;
        let mut render_cache = crate::render::graph::RenderCache::new();

        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event, .. } => {
                    let _ = editor_ctx.handle_event(window.raw(), &event);
                    match event {
                        WindowEvent::CloseRequested => {
                            elwt.exit();
                        },
                        WindowEvent::RedrawRequested => {
                            let _span = tracing::info_span!(target: "render", "frame").entered();
                            // Editor UI
                            editor_ctx.begin_frame(window.raw());
                            inspect_world_ui(&editor_ctx.context, &mut world);
                            let (egui_shapes, egui_renderer) = editor_ctx.end_frame(window.raw());
                            let pixels_per_point = window.raw().scale_factor() as f32;

                            // Render
                            let layer_tree = crate::render::graph::build_from_world(&mut world);
                            let instances = render_cache.update(layer_tree);
                            
                            // Extract Lights
                            let mut lights = Vec::new();
                            let mut query = world.query::<(&Transform, &PointLight)>();
                            for (t, l) in query.iter(&world) {
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

                            // Build Scene (3D)
                            let scene = render_service.build_scene(&mut world);

                            // Camera
                            let mut view_proj = glam::Mat4::IDENTITY.to_cols_array_2d();
                            let mut query_cam = world.query::<(&Transform, &crate::ecs::Camera)>();
                            for (t, c) in query_cam.iter(&world) {
                                if c.is_active {
                                    let view = glam::Mat4::from_rotation_translation(t.rot, t.pos).inverse();
                                    let proj = match c.projection {
                                        crate::ecs::Projection::Orthographic { scale, near, far } => {
                                            let aspect = renderer.config().width as f32 / renderer.config().height as f32;
                                            glam::Mat4::orthographic_rh(
                                                -aspect * scale, aspect * scale, 
                                                -scale, scale, 
                                                near, far
                                            )
                                        },
                                        crate::ecs::Projection::Perspective { fov, aspect, near, far } => {
                                            glam::Mat4::perspective_rh(fov, aspect, near, far)
                                        }
                                    };
                                    view_proj = (proj * view).to_cols_array_2d();
                                    break; // Use first active camera
                                }
                            }

                            render_service.paint(&mut renderer, &scene, instances, view_proj, Some(egui_renderer), &egui_shapes, pixels_per_point);
                            if let Some((_t0, dt)) = renderer.gpu_timings_ms() {
                                if let Some(mut stats) = world.get_resource_mut::<RenderStats>() {
                                    stats.gpu_pass_ms = Some(dt);
                                }
                            }
                            let (dc, ic) = renderer.draw_stats();
                            if let Some(mut stats) = world.get_resource_mut::<RenderStats>() {
                                stats.draw_calls = dc;
                                stats.instances = ic;
                                stats.passes = renderer.pass_count();
                                let (upload, main, ui) = renderer.stage_timings_ms();
                                stats.upload_ms = upload;
                                stats.main_ms = main;
                                stats.ui_ms = ui;
                                stats.offscreen_ms = renderer.offscreen_timing_ms();
                                if let Some(u) = stats.upload_ms { if u > 2.0 { stats.alerts_upload += 1; } }
                                if let Some(m) = stats.main_ms { if m > 16.7 { stats.alerts_main += 1; } }
                                if let Some(u) = stats.ui_ms { if u > 4.0 { stats.alerts_ui += 1; } }
                                if let Some(o) = stats.offscreen_ms { if o > 8.0 { stats.alerts_offscreen += 1; } }
                            }
                            _span.exit();
                        },
                        _ => {}
                    }
                },
                Event::AboutToWait => {
                    let _span = tracing::info_span!(target: "update", "update").entered();
                    // Update Assets
                    let events = asset_server.update(&mut renderer);
                    if !events.is_empty() {
                        for event in events {
                            if let AssetEvent::AtlasLoaded(h, _) = &event {
                                if let Some(atlas) = h.get() {
                                    let mut ts = world.get_resource_or_insert_with::<crate::ecs::TileSet>(Default::default);
                                    for (name, (uv_off, uv_scale)) in atlas.sprites.iter() {
                                        ts.tiles.insert(name.clone(), (*uv_off, *uv_scale));
                                    }
                                }
                            }
                            if let Some(mut am) = world.get_resource_mut::<AssetMetrics>() {
                                match event {
                                    AssetEvent::TextureLoaded(_, ms) => { am.last_latency_ms = Some(ms); am.textures_loaded += 1; }
                                    AssetEvent::AtlasLoaded(_, ms) => { am.last_latency_ms = Some(ms); am.atlases_loaded += 1; }
                                    _ => {}
                                }
                            }
                            if let Some(mut logs) = world.get_resource_mut::<LogEvents>() {
                                let msg = match &event {
                                    AssetEvent::TextureLoaded(_, ms) => format!("TextureLoaded {:.1}ms", ms),
                                    AssetEvent::AtlasLoaded(_, ms) => format!("AtlasLoaded {:.1}ms", ms),
                                    AssetEvent::TextureFailed(_, e) => format!("TextureFailed {}", e),
                                    AssetEvent::AtlasFailed(_, e) => format!("AtlasFailed {}", e),
                                };
                                if logs.entries.len() >= logs.capacity { logs.entries.pop_front(); }
                                logs.entries.push_back(msg);
                            }
                        }
                    }

                    // Update Time Resource
                    let now = std::time::Instant::now();
                    let delta = now.duration_since(last_time).as_secs_f32();
                    last_time = now;
                    
                    accumulator += delta as f64;
                    let fixed_step = world.get_resource::<Time>().unwrap().fixed_time_step;

                    while accumulator >= fixed_step {
                        if let Some(mut time) = world.get_resource_mut::<Time>() {
                            time.delta_seconds = fixed_step as f32;
                            time.elapsed_seconds += fixed_step;
                        }
                        fixed_schedule.run(&mut world);
                        accumulator -= fixed_step;
                    }

                    if let Some(mut time) = world.get_resource_mut::<Time>() {
                        time.alpha = accumulator / fixed_step;
                    }
                    
                    update_schedule.run(&mut world);

                    window.request_redraw();
                    if let Some(mut vp) = world.get_resource_mut::<crate::ecs::Viewport>() {
                        vp.width = renderer.config().width;
                        vp.height = renderer.config().height;
                    }
                    _span.exit();
                },
                _ => {}
            }
        }).unwrap();
    }
}

fn rotate_system(mut query: Query<&mut Transform>, time: Res<Time>) {
    for mut t in query.iter_mut() {
        t.rot *= Quat::from_rotation_z(1.0 * time.delta_seconds);
    }
}

fn apply_texture_handles(mut query: Query<(&Handle<u32>, &mut Sprite)>) {
    for (handle, mut sprite) in query.iter_mut() {
        if let Some(tex_id) = handle.get() {
            sprite.tex_index = tex_id;
        }
    }
}

fn save_previous_transform_system(mut query: Query<(&Transform, &mut PreviousTransform)>) {
    for (t, mut pt) in query.iter_mut() {
        pt.pos = t.pos;
        pt.rot = t.rot;
        pt.scale = t.scale;
    }
}

#[derive(Resource, Default)]
pub struct Benchmark {
    pub enabled: bool,
    pub sprite_count: usize,
}

#[derive(Resource, Default)]
pub struct RenderStats {
    pub gpu_pass_ms: Option<f32>,
    pub draw_calls: u32,
    pub instances: u32,
    pub passes: u32,
    pub upload_ms: Option<f32>,
    pub main_ms: Option<f32>,
    pub ui_ms: Option<f32>,
    pub offscreen_ms: Option<f32>,
    pub alerts_upload: u32,
    pub alerts_main: u32,
    pub alerts_ui: u32,
    pub alerts_offscreen: u32,
}

#[derive(Resource, Default)]
pub struct AssetMetrics {
    pub last_latency_ms: Option<f32>,
    pub textures_loaded: u32,
    pub atlases_loaded: u32,
}

fn benchmark_system(mut commands: Commands, mut benchmark: ResMut<Benchmark>) {
    if benchmark.enabled && benchmark.sprite_count < 50000 {
        // Spawn 500 sprites per frame until 50000
        for _ in 0..500 {
            commands.spawn((
                Transform { 
                    pos: Vec3::new(rand::random::<f32>() * 800.0, rand::random::<f32>() * 600.0, 0.0), 
                    scale: Vec3::new(5.0, 5.0, 1.0), 
                    ..Default::default() 
                },
                Sprite { color: [rand::random(), rand::random(), rand::random(), 1.0], ..Default::default() }
            ));
            benchmark.sprite_count += 1;
        }
    }
}
fn compute_passes(timing: Option<(f32, f32)>) -> u32 {
    match timing {
        Some((_t0, _total)) => 0, // Placeholder: aggregate count not exposed externally in renderer
        None => 0,
    }
}
#[derive(Resource, Default)]
pub struct LogEvents {
    pub entries: std::collections::VecDeque<String>,
    pub filter: String,
    pub capacity: usize,
}
