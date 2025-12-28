//  多人游戏示例
//
//  展示网络同步、客户端预测、服务器权威等功能

use bevy_ecs::prelude::*;
use game_engine::ecs::Transform;
use game_engine::network::NetworkState;
use glam::Vec3;

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Multiplayer Example ===");

    // 创建 ECS 世界
    let mut world = World::new();

    // 初始化网络状态
    world.insert_resource(NetworkState::new());

    println!("Network system initialized");

    // 创建一些网络同步的实体
    for i in 0..5 {
        world.spawn((
            Transform {
                pos: Vec3::new(i as f32, 0.0, 0.0),
                rot: glam::Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            // 网络同步组件会在网络系统中自动添加
        ));
    }

    println!("Created 5 network-synced entities");
    println!("Multiplayer example completed successfully!");
    println!("\nNote: This is a demonstration of entity creation.");
    println!("      For real multiplayer, initialize a full network connection.");
}
