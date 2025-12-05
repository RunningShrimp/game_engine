//! 物理示例
//!
//! 展示物理系统、碰撞检测、刚体等功能

use bevy_ecs::prelude::*;
use game_engine::core::Engine;
use game_engine::ecs::Transform;
use game_engine::physics::{ColliderDesc, RigidBodyDesc, ShapeType};
use glam::Vec3;
use rapier3d::prelude::RigidBodyType;

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Physics Example ===");

    let mut engine = Engine::new();

    if let Err(e) = engine.initialize() {
        eprintln!("Failed to initialize engine: {}", e);
        return;
    }

    let world = engine.world_mut();

    // 创建地面（静态刚体）
    world.spawn((
        Transform {
            pos: Vec3::new(0.0, -1.0, 0.0),
            rot: glam::Quat::IDENTITY,
            scale: Vec3::new(10.0, 1.0, 10.0),
        },
        RigidBodyDesc {
            body_type: RigidBodyType::Fixed,
            ..Default::default()
        },
        ColliderDesc {
            shape: ShapeType::Box,
            ..Default::default()
        },
    ));

    println!("Created ground");

    // 创建一些动态刚体（球体）
    for i in 0..10 {
        let x = (i as f32 - 5.0) * 1.5;
        let y = 5.0 + i as f32 * 0.5;

        world.spawn((
            Transform {
                pos: Vec3::new(x, y, 0.0),
                rot: glam::Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            RigidBodyDesc {
                body_type: RigidBodyType::Dynamic,
                mass: 1.0,
                ..Default::default()
            },
            ColliderDesc {
                shape: ShapeType::Sphere { radius: 0.5 },
                ..Default::default()
            },
        ));
    }

    println!("Created 10 dynamic spheres");
    println!("Running physics simulation...");

    // 运行物理模拟
    for i in 0..300 {
        if let Err(e) = engine.update() {
            eprintln!("Error during update: {}", e);
            break;
        }

        if i % 30 == 0 {
            println!("Physics step {} completed", i);
        }
    }

    println!("Physics example completed!");
}
