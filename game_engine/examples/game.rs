//! 完整游戏示例
//!
//! 展示如何组合使用引擎的各种功能创建一个简单的游戏

use bevy_ecs::prelude::*;
use game_engine::animation::{AnimationClip, AnimationPlayer, InterpolationMode, KeyframeTrack};
use game_engine::core::Engine;
use game_engine::ecs::{Camera, Projection, Sprite, Transform};
use game_engine::physics::{ColliderDesc, RigidBodyDesc, ShapeType};
use glam::{Quat, Vec3};
use rapier3d::prelude::RigidBodyType;

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Complete Game Example ===");
    println!("This example demonstrates:");
    println!("  - Engine initialization");
    println!("  - Entity creation");
    println!("  - Rendering");
    println!("  - Physics simulation");
    println!("  - Animation");
    println!("  - Input handling");

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

    // 创建地面
    world.spawn((
        Transform {
            pos: Vec3::new(0.0, -1.0, 0.0),
            rot: Quat::IDENTITY,
            scale: Vec3::new(20.0, 1.0, 20.0),
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

    // 创建玩家角色（带动画）
    let mut player_clip = AnimationClip::new("player_walk".to_string(), 1.0);
    let mut walk_track = KeyframeTrack::new(InterpolationMode::Linear);
    walk_track.add_keyframe(0.0, Vec3::new(0.0, 1.0, 0.0));
    walk_track.add_keyframe(0.5, Vec3::new(0.0, 1.2, 0.0));
    walk_track.add_keyframe(1.0, Vec3::new(0.0, 1.0, 0.0));
    player_clip.add_track("position_y".to_string(), walk_track);

    world.spawn((
        Transform {
            pos: Vec3::new(0.0, 1.0, 0.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        },
        RigidBodyDesc {
            body_type: RigidBodyType::Dynamic,
            mass: 1.0,
            ..Default::default()
        },
        ColliderDesc {
            shape: ShapeType::Capsule {
                radius: 0.5,
                height: 2.0,
            },
            ..Default::default()
        },
        AnimationPlayer::new(player_clip),
        Sprite {
            color: [0.2, 0.6, 1.0, 1.0],
            size: [1.0, 2.0],
        },
    ));

    // 创建一些障碍物
    for i in 0..10 {
        let angle = (i as f32 / 10.0) * std::f32::consts::PI * 2.0;
        let radius = 5.0;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;

        world.spawn((
            Transform {
                pos: Vec3::new(x, 0.5, z),
                rot: Quat::IDENTITY,
                scale: Vec3::new(1.0, 1.0, 1.0),
            },
            RigidBodyDesc {
                body_type: RigidBodyType::Dynamic,
                mass: 0.5,
                ..Default::default()
            },
            ColliderDesc {
                shape: ShapeType::Box,
                ..Default::default()
            },
            Sprite {
                color: [0.8, 0.2, 0.2, 1.0],
                size: [1.0, 1.0],
            },
        ));
    }

    println!("Game setup complete!");
    println!("Running game loop...");
    println!("Press ESC to exit");

    // 游戏主循环
    let mut frame_count = 0;
    loop {
        if let Err(e) = engine.update() {
            eprintln!("Error during update: {}", e);
            break;
        }

        frame_count += 1;

        if frame_count % 60 == 0 {
            println!("Frame {} - Game running...", frame_count);
        }

        // 简化：运行300帧后退出
        if frame_count >= 300 {
            println!("Game example completed!");
            break;
        }
    }
}
