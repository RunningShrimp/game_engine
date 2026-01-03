//! Tank Battle Client - 坦克对战客户端
//!
//! 多人在线坦克对战游戏客户端

use game_engine::prelude::*;
use game_engine::scripting::csharp::{CSharpRuntime, CSharpConfig};
use game_engine::network::{NetworkClient, NetworkConfig};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    log::info!("启动坦克对战客户端");

    // 解析命令行参数
    let args: Vec<String> = std::env::args().collect();
    let server_address = if args.len() > 1 {
        args[1].clone()
    } else {
        "127.0.0.1:27015".to_string()
    };

    log::info!("连接到服务器: {}", server_address);

    // 创建引擎实例
    let mut engine = Engine::new(EngineConfig {
        window_title: "Tank Battle".to_string(),
        window_size: (1280, 720),
        vsync: true,
        msaa: Msaa::Sample4,
        ..Default::default()
    })?;

    // 初始化C#运行时
    let scripts_dir = PathBuf::from("scripts");
    let csharp_config = CSharpConfig {
        scripts_dir: scripts_dir.clone(),
        enable_hot_reload: false, // 网络游戏禁用热重载
        ..Default::default()
    };

    let mut csharp_runtime = CSharpRuntime::new(csharp_config)?;
    csharp_runtime.load_scripts(&scripts_dir)?;

    // 注册C#运行时到引擎
    engine.add_script_runtime(csharp_runtime);

    // 创建网络客户端
    let network_config = NetworkConfig {
        server_address: server_address.clone(),
        timeout_seconds: 10,
        ..Default::default()
    };

    let mut network_client = NetworkClient::new(network_config)?;

    // 连接到服务器
    match network_client.connect() {
        Ok(_) => log::info!("成功连接到服务器"),
        Err(e) => {
            log::error!("连接服务器失败: {}", e);
            return Err(e.into());
        }
    }

    // 注册网络客户端到引擎
    engine.add_network_client(network_client);

    // 创建登录场景
    let login_scene = create_login_scene(&mut engine)?;

    // 设置主相机
    let camera_entity = create_camera(&mut engine, login_scene)?;
    engine.set_main_camera(camera_entity);

    // 运行游戏循环
    log::info!("进入游戏循环");
    engine.run()?;

    Ok(())
}

/// 创建登录场景
fn create_login_scene(engine: &mut Engine) -> Result<SceneHandle, EngineError> {
    let scene = engine.create_scene("LoginScene")?;

    // 添加UI
    let ui = engine.create_entity(scene)?;
    engine.set_name(ui, "LoginUI")?;
    engine.add_ui_canvas(ui, UICanvas {
        render_mode: RenderMode::ScreenSpaceOverlay,
        ..Default::default()
    })?;

    // 添加C#脚本 - LoginUI
    engine.add_csharp_script(ui, "UI.LoginUI")?;

    Ok(scene)
}

/// 创建相机
fn create_camera(
    engine: &mut Engine,
    scene: SceneHandle,
) -> Result<EntityHandle, EngineError> {
    let camera = engine.create_entity(scene)?;
    engine.set_name(camera, "MainCamera")?;

    engine.set_transform(camera, Transform {
        position: Vec3::new(0.0, 10.0, -20.0),
        rotation: Quaternion::from_euler(Vec3::new(45.0f32.to_radians(), 0.0, 0.0)),
        ..Default::default()
    })?;

    engine.add_camera(camera, Camera {
        fov: 60.0,
        near: 0.1,
        far: 1000.0,
        ..Default::default()
    })?;

    Ok(camera)
}

/// 创建游戏场景（连接成功后）
pub fn create_game_scene(engine: &mut Engine) -> Result<SceneHandle, EngineError> {
    let scene = engine.create_scene("GameScene")?;

    // 添加灯光
    engine.add_light(scene, Light {
        light_type: LightType::Directional,
        color: Color::rgb(1.0, 0.95, 0.9),
        intensity: 1.0,
        direction: Vec3::new(-0.5, -1.0, -0.3),
        ..Default::default()
    })?;

    // 创建地面
    create_ground(engine, scene)?;

    // 创建掩体
    create_cover(engine, scene)?;

    // 创建生成点
    create_spawn_points(engine, scene)?;

    Ok(scene)
}

/// 创建地面
fn create_ground(engine: &mut Engine, scene: SceneHandle) -> Result<(), EngineError> {
    let ground = engine.create_entity(scene)?;
    engine.set_name(ground, "Ground")?;

    engine.set_transform(ground, Transform {
        position: Vec3::new(0.0, -0.1, 0.0),
        scale: Vec3::new(100.0, 0.2, 100.0),
        ..Default::default()
    })?;

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

    engine.add_collider(ground, Collider {
        shape: ColliderShape::Box {
            half_extents: Vec3::new(50.0, 0.1, 50.0),
        },
        ..Default::default()
    })?;

    Ok(())
}

/// 创建掩体
fn create_cover(engine: &mut Engine, scene: SceneHandle) -> Result<(), EngineError> {
    let cover_positions = vec![
        Vec3::new(10.0, 1.0, 0.0),
        Vec3::new(-10.0, 1.0, 0.0),
        Vec3::new(0.0, 1.0, 10.0),
        Vec3::new(0.0, 1.0, -10.0),
    ];

    for (i, pos) in cover_positions.iter().enumerate() {
        let cover = engine.create_entity(scene)?;
        engine.set_name(cover, &format!("Cover_{}", i))?;

        engine.set_transform(cover, Transform {
            position: *pos,
            scale: Vec3::new(3.0, 2.0, 3.0),
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
                half_extents: Vec3::new(1.5, 1.0, 1.5),
            },
            ..Default::default()
        })?;
    }

    Ok(())
}

/// 创建生成点
fn create_spawn_points(engine: &mut Engine, scene: SceneHandle) -> Result<(), EngineError> {
    let spawn_positions = vec![
        Vec3::new(-20.0, 0.0, -20.0),
        Vec3::new(20.0, 0.0, -20.0),
        Vec3::new(-20.0, 0.0, 20.0),
        Vec3::new(20.0, 0.0, 20.0),
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(-30.0, 0.0, 0.0),
        Vec3::new(30.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -30.0),
        Vec3::new(0.0, 0.0, 30.0),
    ];

    for (i, pos) in spawn_positions.iter().enumerate() {
        let spawn_point = engine.create_entity(scene)?;
        engine.set_name(spawn_point, &format!("SpawnPoint_{}", i))?;

        engine.set_transform(spawn_point, Transform {
            position: *pos,
            ..Default::default()
        })?;

        // 添加C#脚本 - SpawnPoint
        engine.add_csharp_script(spawn_point, "Game.SpawnPoint")?;
    }

    Ok(())
}
