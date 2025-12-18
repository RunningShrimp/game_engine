//! 渲染示例
//!
//! 展示PBR渲染、后处理效果等高级渲染功能

use bevy_ecs::prelude::*;
use game_engine::core::Engine;
use game_engine::ecs::{Camera, Projection, Transform};
use game_engine::render::pbr::PbrMaterial;
use glam::{Quat, Vec3};

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Rendering Example ===");

    let mut engine = Engine::new();

    if let Err(e) = engine.initialize() {
        eprintln!("Failed to initialize engine: {}", e);
        return;
    }

    let world = engine.world_mut();

    // 创建相机
    world.spawn((
        Transform {
            pos: Vec3::new(0.0, 5.0, 10.0),
            rot: Quat::from_rotation_x(-0.3),
            scale: Vec3::ONE,
        },
        Camera {
            fov: 60.0,
            near: 0.1,
            far: 1000.0,
        },
        Projection::Perspective,
    ));

    println!("Created camera");

    // 创建一些使用PBR材质的实体
    for i in 0..5 {
        let x = (i as f32 - 2.0) * 3.0;
        world.spawn((
            Transform {
                pos: Vec3::new(x, 0.0, 0.0),
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            PbrMaterial {
                base_color: glam::Vec4::new(0.8, 0.2, 0.2, 1.0),
                metallic: 0.5,
                roughness: 0.3,
                ..Default::default()
            },
        ));
    }

    println!("Created 5 PBR entities");
    println!("Running rendering example...");

    // 运行渲染循环
    for i in 0..60 {
        if let Err(e) = engine.update() {
            eprintln!("Error during update: {}", e);
            break;
        }

        if i % 10 == 0 {
            println!("Rendered frame {}", i);
        }
    }

    println!("Rendering example completed!");
}
