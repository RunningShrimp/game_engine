//! 多人游戏示例
//!
//! 展示网络同步、客户端预测、服务器权威等功能

use bevy_ecs::prelude::*;
use game_engine::core::Engine;
use game_engine::ecs::Transform;
use game_engine::network::{ConnectionState, NetworkService, NetworkState};
use glam::Vec3;

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Multiplayer Example ===");

    let mut engine = Engine::new();

    if let Err(e) = engine.initialize() {
        eprintln!("Failed to initialize engine: {}", e);
        return;
    }

    let world = engine.world_mut();

    // 初始化网络状态
    world.insert_resource(NetworkState {
        connection_state: ConnectionState::Disconnected,
        client_id: None,
        server_addr: None,
        stats: Default::default(),
        current_tick: 0,
        send_tx: None,
        recv_rx: None,
    });

    world.insert_resource(NetworkService::new());

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
    println!("Running multiplayer simulation...");

    // 运行网络模拟
    for i in 0..300 {
        if let Err(e) = engine.update() {
            eprintln!("Error during update: {}", e);
            break;
        }

        if i % 30 == 0 {
            if let Some(network_state) = world.get_resource::<NetworkState>() {
                println!(
                    "Network tick: {}, State: {:?}",
                    network_state.current_tick, network_state.connection_state
                );
            }
        }
    }

    println!("Multiplayer example completed!");
    println!("Note: This is a simulation. For real multiplayer, connect to a server.");
}
