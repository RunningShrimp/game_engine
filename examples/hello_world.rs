//! Hello World 示例
//!
//! 最简单的引擎使用示例，展示如何初始化和运行引擎

use bevy_ecs::prelude::*;
use game_engine::core::Engine;
use game_engine::ecs::{Sprite, Transform};

fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== Game Engine Hello World Example ===");

    // 创建引擎实例
    let mut engine = Engine::new();

    // 初始化引擎
    if let Err(e) = engine.initialize() {
        eprintln!("Failed to initialize engine: {}", e);
        return;
    }

    println!("Engine initialized successfully!");

    // 创建一些实体
    let world = engine.world_mut();

    // 创建一个简单的实体
    world.spawn((
        Transform::default(),
        Sprite {
            color: [1.0, 0.0, 0.0, 1.0],
            size: [100.0, 100.0],
        },
    ));

    println!("Created a sprite entity");

    // 运行几帧
    println!("Running engine for 5 frames...");
    for i in 0..5 {
        if let Err(e) = engine.update() {
            eprintln!("Error during update: {}", e);
            break;
        }
        println!("Frame {} completed", i + 1);
    }

    println!("Example completed successfully!");
}
