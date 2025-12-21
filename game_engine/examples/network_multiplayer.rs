//! 多人游戏网络示例
//!
//! 展示网络同步、客户端预测、状态同步、压缩等多人游戏功能。
//!
//! # 功能特性
//!
//! - 客户端预测和回滚
//! - 状态同步
//! - 数据压缩
//! - 延迟补偿
//!
//! # 运行
//!
//! ```bash
//! cargo run --example network_multiplayer
//! ```

use game_engine::network::compression::{CompressionLevel, NetworkCompressor};
use game_engine::network::prediction::{ClientPredictionManager, InputCommand};
use game_engine::network::synchronization::{EntityState, SyncStrategy};
use glam::{Quat, Vec3};

fn main() {
    tracing_subscriber::fmt::init();

    println!("=== Network Multiplayer Example ===");
    println!();
    println!("This example demonstrates:");
    println!("- Client-side prediction and rollback");
    println!("- State synchronization");
    println!("- Data compression");
    println!("- Latency compensation");
    println!();

    // 创建客户端预测管理器
    let mut prediction_manager = ClientPredictionManager::default();
    println!("Created ClientPredictionManager");
    println!();

    // 演示输入命令提交
    println!("Submitting input commands...");
    for i in 0..10 {
        let input_data = vec![i as u8; 32];
        let sequence = prediction_manager.submit_input(input_data);
        println!("  Submitted input command with sequence: {}", sequence);
    }
    println!();

    // 获取预测统计
    let stats = prediction_manager.stats();
    println!("Prediction Stats:");
    println!("  - Confirmed Commands: {}", stats.confirmed_commands);
    println!("  - Unconfirmed Commands: {}", stats.unconfirmed_commands);
    println!("  - Rollbacks: {}", stats.rollbacks);
    println!("  - Replays: {}", stats.replays);
    println!();

    // 演示状态同步
    println!("Creating entity states for synchronization...");
    let mut states = Vec::new();
    for i in 0..5 {
        let state = EntityState::new(
            Vec3::new(i as f32, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec3::ZERO,
        );
        states.push(state);
    }
    println!("Created {} entity states", states.len());
    println!();

    // 演示数据压缩
    println!("Demonstrating data compression...");
    let compressor = NetworkCompressor::with_level(CompressionLevel::Balanced);
    
    let test_data = b"Hello, World! This is a test string for compression. ".repeat(100);
    println!("  Original data size: {} bytes", test_data.len());
    
    match compressor.compress(&test_data) {
        Ok(compressed) => {
            println!("  Compressed data size: {} bytes", compressed.len());
            let compression_ratio = (1.0 - compressed.len() as f64 / test_data.len() as f64) * 100.0;
            println!("  Compression ratio: {:.2}%", compression_ratio);
            
            // 测试解压缩
            match compressor.decompress(&compressed) {
                Ok(decompressed) => {
                    if decompressed == test_data {
                        println!("  ✓ Decompression successful, data matches");
                    } else {
                        println!("  ✗ Decompression failed, data mismatch");
                    }
                }
                Err(e) => {
                    println!("  ✗ Decompression error: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  ✗ Compression error: {}", e);
        }
    }
    println!();

    // 演示同步策略
    println!("Synchronization Strategies:");
    println!("  - ServerAuthoritative: {:?}", SyncStrategy::ServerAuthoritative);
    println!("  - ClientPrediction: {:?}", SyncStrategy::ClientPrediction);
    println!("  - Hybrid: {:?}", SyncStrategy::Hybrid);
    println!();

    println!("Example completed!");
    println!("Note: This is a demonstration of network components.");
    println!("      For real multiplayer, initialize a full network connection.");
}

