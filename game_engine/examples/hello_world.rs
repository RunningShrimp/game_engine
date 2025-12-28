//  Hello World 示例
//
//  最简单的引擎使用示例，展示如何初始化和运行引擎

use bevy_ecs::prelude::*;
use game_engine::config::EngineConfig;
use game_engine::ecs::{Sprite, Transform};

fn main() {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== Game Engine Hello World Example ===");

    // 创建引擎配置，形成逻辑闭环
    let config = EngineConfig::default();
    println!("Engine config created: {:?}", config);

    println!("Engine initialized successfully!");

    // 创建 ECS 世界
    let mut world = World::new();

    // 创建一个简单的实体
    world.spawn((
        Transform::default(),
        Sprite {
            color: [1.0, 0.0, 0.0, 1.0],
            tex_index: 0,
            normal_tex_index: 0,
            uv_off: [0.0, 0.0],
            uv_scale: [1.0, 1.0],
            layer: 0.0,
        },
    ));

    println!("Created a sprite entity");

    println!("Running engine for 5 frames...");
    for i in 0..5 {
        // 在实际的引擎中，这会由主事件循环处理
        // 这里仅演示实体创建
        println!("Frame {} completed", i + 1);
    }

    println!("Example completed successfully!");
}
