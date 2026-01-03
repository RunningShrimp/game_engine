//! 3D FPS Demo - 第一人称射击演示
//!
//! 展示游戏引擎的3D渲染、物理和网络能力

use game_engine::prelude::*;
use game_engine::scripting::csharp::{CSharpRuntime, CSharpConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("启动3D FPS演示");

    // 创建引擎实例
    let mut engine = Engine::new(EngineConfig {
        window_title: "3D FPS Demo".to_string(),
        window_size: (1920, 1080),
        vsync: true,
        msaa: Msaa::Sample4,
        ..Default::default()
    })?;

    // 初始化C#运行时
    let scripts_dir = PathBuf::from("scripts");
    let csharp_config = CSharpConfig {
        scripts_dir: scripts_dir.clone(),
        enable_hot_reload: true,
        ..Default::default()
    };

    let mut csharp_runtime = CSharpRuntime::new(csharp_config)?;
    csharp_runtime.load_scripts(&scripts_dir)?;

    // 注册C#运行时到引擎
    engine.add_script_runtime(csharp_runtime);

    // 创建3D场景
    let scene = create_scene(&mut engine)?;

    // 设置主相机
    let camera_entity = create_player_camera(&mut engine, &scene)?;
    engine.set_main_camera(camera_entity);

    // 创建游戏管理器
    create_game_manager(&mut engine, &scene)?;

    // 创建敌人
    for i in 0..5 {
        create_enemy(&mut engine, &scene, i)?;
    }

    // 运行游戏循环
    log::info!("进入游戏循环");
    engine.run()?;

    Ok(())
}

/// 创建3D场景
fn create_scene(engine: &mut Engine) -> Result<SceneHandle, EngineError> {
    let scene = engine.create_scene("MainScene")?;

    // 添加灯光
    // 环境光
    engine.add_light(scene, Light {
        light_type: LightType::Ambient,
        color: Color::rgb(0.3, 0.3, 0.35),
        intensity: 0.5,
        ..Default::default()
    })?;

    // 定向光（太阳）
    engine.add_light(scene, Light {
        light_type: LightType::Directional,
        color: Color::rgb(1.0, 0.95, 0.9),
        intensity: 1.2,
        direction: Vec3::new(-0.5, -1.0, -0.3),
        ..Default::default()
    })?;

    // 创建地面
    create_ground(engine, scene)?;

    // 创建墙壁和障碍物
    create_level_geometry(engine, scene)?;

    // 创建生成点
    create_spawn_points(engine, scene)?;

    Ok(scene)
}

/// 创建地面
fn create_ground(engine: &mut Engine, scene: SceneHandle) -> Result<(), EngineError> {
    let ground = engine.create_entity(scene)?;
    engine.set_name(ground, "Ground")?;

    // 添加变换组件
    engine.set_transform(ground, Transform {
        position: Vec3::new(0.0, -0.1, 0.0),
        scale: Vec3::new(100.0, 0.2, 100.0),
        ..Default::default()
    })?;

    // 添加网格渲染器
    engine.add_mesh_renderer(ground, MeshRenderer {
        mesh: Mesh::plane(100.0, 100.0),
        material: Material {
            albedo: Color::rgb(0.2, 0.2, 0.2),
            metallic: 0.0,
            roughness: 0.9,
            ..Default::default()
        },
        ..Default::default()
    })?;

    // 添加物理碰撞体
    engine.add_collider(ground, Collider {
        shape: ColliderShape::Box {
            half_extents: Vec3::new(50.0, 0.1, 50.0),
        },
        ..Default::default()
    })?;

    Ok(())
}

/// 创建关卡几何体
fn create_level_geometry(engine: &mut Engine, scene: SceneHandle) -> Result<(), EngineError> {
    // 创建一些墙壁和箱子作为掩体
    let cover_positions = vec![
        Vec3::new(10.0, 1.0, 10.0),
        Vec3::new(-10.0, 1.0, 15.0),
        Vec3::new(15.0, 1.0, -10.0),
        Vec3::new(-15.0, 1.0, -15.0),
        Vec3::new(0.0, 1.0, 25.0),
    ];

    for (i, pos) in cover_positions.iter().enumerate() {
        let cover = engine.create_entity(scene)?;
        engine.set_name(cover, &format!("Cover_{}", i))?;

        engine.set_transform(cover, Transform {
            position: *pos,
            scale: Vec3::new(3.0, 2.0, 1.5),
            ..Default::default()
        })?;

        engine.add_mesh_renderer(cover, MeshRenderer {
            mesh: Mesh::cube(1.0),
            material: Material {
                albedo: Color::rgb(0.4, 0.3, 0.2),
                ..Default::default()
            },
            ..Default::default()
        })?;

        engine.add_collider(cover, Collider {
            shape: ColliderShape::Box {
                half_extents: Vec3::new(1.5, 1.0, 0.75),
            },
            ..Default::default()
        })?;
    }

    // 创建外围墙壁
    let wall_positions = vec![
        (Vec3::new(0.0, 2.5, 50.0), Vec3::new(100.0, 5.0, 0.5)),  // 北墙
        (Vec3::new(0.0, 2.5, -50.0), Vec3::new(100.0, 5.0, 0.5)), // 南墙
        (Vec3::new(50.0, 2.5, 0.0), Vec3::new(0.5, 5.0, 100.0)),  // 东墙
        (Vec3::new(-50.0, 2.5, 0.0), Vec3::new(0.5, 5.0, 100.0)), // 西墙
    ];

    for (i, (pos, scale)) in wall_positions.iter().enumerate() {
        let wall = engine.create_entity(scene)?;
        engine.set_name(wall, &format!("Wall_{}", i))?;

        engine.set_transform(wall, Transform {
            position: *pos,
            scale: *scale,
            ..Default::default()
        })?;

        engine.add_mesh_renderer(wall, MeshRenderer {
            mesh: Mesh::cube(1.0),
            material: Material {
                albedo: Color::rgb(0.5, 0.5, 0.5),
                ..Default::default()
            },
            ..Default::default()
        })?;

        engine.add_collider(wall, Collider {
            shape: ColliderShape::Box {
                half_extents: scale * 0.5,
            },
            ..Default::default()
        })?;
    }

    Ok(())
}

/// 创建玩家相机
fn create_player_camera(
    engine: &mut Engine,
    scene: SceneHandle,
) -> Result<EntityHandle, EngineError> {
    // 创建玩家容器
    let player = engine.create_entity(scene)?;
    engine.set_name(player, "Player")?;

    engine.set_transform(player, Transform {
        position: Vec3::new(0.0, 1.7, 0.0), // 眼睛高度
        ..Default::default()
    })?;

    // 添加角色控制器
    engine.add_character_controller(player, CharacterController {
        radius: 0.4,
        height: 1.8,
        ..Default::default()
    })?;

    // 添加C#脚本组件 - PlayerController
    engine.add_csharp_script(player, "Components.PlayerController")?;

    // 创建相机子对象
    let camera = engine.create_entity(scene)?;
    engine.set_name(camera, "MainCamera")?;
    engine.set_parent(camera, Some(player))?;

    engine.set_transform(camera, Transform {
        position: Vec3::new(0.0, 0.0, 0.0),
        ..Default::default()
    })?;

    // 添加相机组件
    engine.add_camera(camera, Camera {
        fov: 70.0,
        near: 0.1,
        far: 1000.0,
        ..Default::default()
    })?;

    // 添加C#脚本组件 - FirstPersonCamera
    engine.add_csharp_script(camera, "Components.FirstPersonCamera")?;

    // 添加音频监听器
    engine.add_audio_listener(camera, AudioListener::default())?;

    Ok(player)
}

/// 创建游戏管理器
fn create_game_manager(
    engine: &mut Engine,
    scene: SceneHandle,
) -> Result<EntityHandle, EngineError> {
    let game_manager = engine.create_entity(scene)?;
    engine.set_name(game_manager, "GameManager")?;

    // 添加C#脚本组件 - GameMode
    engine.add_csharp_script(game_manager, "Game.GameMode")?;

    Ok(game_manager)
}

/// 创建敌人
fn create_enemy(
    engine: &mut Engine,
    scene: SceneHandle,
    index: usize,
) -> Result<EntityHandle, EngineError> {
    let angle = (index as f32 * 72.0).to_radians(); // 360/5 = 72度间隔
    let radius = 20.0;
    let pos = Vec3::new(angle.cos() * radius, 0.0, angle.sin() * radius);

    let enemy = engine.create_entity(scene)?;
    engine.set_name(enemy, &format!("Enemy_{}", index))?;

    engine.set_transform(enemy, Transform {
        position: pos,
        ..Default::default()
    })?;

    // 添加网格渲染器（简单的胶囊体代表敌人）
    engine.add_mesh_renderer(enemy, MeshRenderer {
        mesh: Mesh::capsule(0.5, 1.8),
        material: Material {
            albedo: Color::rgb(0.8, 0.2, 0.2),
            ..Default::default()
        },
        ..Default::default()
    })?;

    // 添加物理碰撞体
    engine.add_collider(enemy, Collider {
        shape: ColliderShape::Capsule {
            radius: 0.5,
            height: 1.8,
        },
        ..Default::default()
    })?;

    // 添加导航网格代理
    engine.add_navmesh_agent(enemy, NavMeshAgent {
        radius: 0.5,
        speed: 3.0,
        acceleration: 8.0,
        ..Default::default()
    })?;

    // 添加C#脚本组件 - Enemy
    engine.add_csharp_script(enemy, "Components.Enemy")?;

    // 添加生命值组件
    engine.add_csharp_script(enemy, "Components.Health")?;

    Ok(enemy)
}

/// 创建生成点
fn create_spawn_points(
    engine: &mut Engine,
    scene: SceneHandle,
) -> Result<(), EngineError> {
    let spawn_positions = vec![
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(-20.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 20.0),
        Vec3::new(0.0, 0.0, -20.0),
        Vec3::new(15.0, 0.0, 15.0),
        Vec3::new(-15.0, 0.0, -15.0),
    ];

    for (i, pos) in spawn_positions.iter().enumerate() {
        let spawn_point = engine.create_entity(scene)?;
        engine.set_name(spawn_point, &format!("SpawnPoint_{}", i))?;

        engine.set_transform(spawn_point, Transform {
            position: *pos,
            ..Default::default()
        })?;

        // 添加C#脚本组件 - SpawnPoint
        engine.add_csharp_script(spawn_point, "Game.SpawnPoint")?;
    }

    Ok(())
}
