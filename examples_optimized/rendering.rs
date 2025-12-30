//  渲染示例
//
//  展示PBR渲染、后处理效果等高级渲染功能

use bevy_ecs::prelude::*;
use game_engine::ecs::{Camera, PbrMaterialComp, Projection, Transform};
use glam::{Quat, Vec3};

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Rendering Example ===");

    // 创建 ECS 世界
    let mut world = World::new();

    // 创建相机
    world.spawn((
        Transform {
            pos: Vec3::new(0.0, 5.0, 10.0),
            rot: Quat::from_rotation_x(-0.3),
            scale: Vec3::ONE,
        },
        Camera {
            is_active: true,
            projection: Projection::Perspective {
                fov: 60.0,
                aspect: 16.0 / 9.0,
                near: 0.1,
                far: 1000.0,
            },
        },
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
            PbrMaterialComp {
                base_color: [0.8, 0.2, 0.2, 1.0],
                metallic: 0.5,
                roughness: 0.3,
                ambient_occlusion: 1.0,
                emissive: [0.0, 0.0, 0.0],
                emissive_strength: 0.0,
            },
        ));
    }

    println!("Created 5 PBR entities");
    println!("Rendering example completed successfully!");
    println!("\nNote: In a real engine, rendering would happen in the main render loop.");
    println!("This example demonstrates entity creation with PBR materials.");
}
