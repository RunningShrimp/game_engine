//! 动画示例
//!
//! 展示骨骼动画、关键帧动画等功能

use bevy_ecs::prelude::*;
use game_engine::animation::{AnimationClip, AnimationPlayer, InterpolationMode, KeyframeTrack};
use game_engine::core::Engine;
use game_engine::ecs::Transform;
use glam::{Quat, Vec3};

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Animation Example ===");

    let mut engine = Engine::new();

    if let Err(e) = engine.initialize() {
        eprintln!("Failed to initialize engine: {}", e);
        return;
    }

    let world = engine.world_mut();

    // 创建动画剪辑
    let mut clip = AnimationClip::new("rotation_animation".to_string(), 5.0);

    // 添加位置关键帧
    let mut position_track = KeyframeTrack::new(InterpolationMode::Linear);
    position_track.add_keyframe(0.0, Vec3::new(0.0, 0.0, 0.0));
    position_track.add_keyframe(2.5, Vec3::new(5.0, 2.0, 0.0));
    position_track.add_keyframe(5.0, Vec3::new(0.0, 0.0, 0.0));

    // 添加旋转关键帧
    let mut rotation_track = KeyframeTrack::new(InterpolationMode::Slerp);
    rotation_track.add_keyframe(0.0, Quat::IDENTITY);
    rotation_track.add_keyframe(2.5, Quat::from_rotation_y(std::f32::consts::PI));
    rotation_track.add_keyframe(5.0, Quat::from_rotation_y(std::f32::consts::PI * 2.0));

    clip.add_track("position".to_string(), position_track);
    clip.add_track("rotation".to_string(), rotation_track);

    // 创建动画实体
    let entity = world
        .spawn((
            Transform {
                pos: Vec3::ZERO,
                rot: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            AnimationPlayer::new(clip),
        ))
        .id();

    println!("Created animated entity");
    println!("Running animation...");

    // 运行动画
    for i in 0..300 {
        if let Err(e) = engine.update() {
            eprintln!("Error during update: {}", e);
            break;
        }

        // 获取动画进度
        if let Some(player) = world.get::<AnimationPlayer>(entity) {
            if i % 30 == 0 {
                println!(
                    "Animation time: {:.2}s / {:.2}s",
                    player.current_time, player.clip.duration
                );
            }
        }
    }

    println!("Animation example completed!");
}
