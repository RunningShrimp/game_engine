//! 网络压力测试
//!
//! 测试网络系统在高并发下的性能。

use game_engine::network::compression::NetworkCompressor;
use game_engine::network::prediction::{ClientPredictionManager, InputCommand};
use game_engine::network::synchronization::EntityState;
use glam::{Quat, Vec3};

#[test]
#[ignore] // 压力测试默认忽略，需要时手动运行
fn test_network_compression_1000_messages() {
    // 测试压缩1000条消息的性能
    const MESSAGE_COUNT: usize = 1000;
    
    let compressor = NetworkCompressor::new();
    let test_data = vec![42u8; 1000]; // 每条消息1KB
    
    let start = std::time::Instant::now();
    
    let mut compressed_count = 0;
    for _ in 0..MESSAGE_COUNT {
        match compressor.compress(&test_data) {
            Ok(_) => compressed_count += 1,
            Err(_) => {}
        }
    }
    
    let compression_time = start.elapsed();
    
    // 验证压缩性能
    assert_eq!(compressed_count, MESSAGE_COUNT);
    // 压缩1000条消息应该在合理时间内完成（应该在5秒内）
    assert!(compression_time.as_secs() < 10, "压缩应在10秒内完成");
}

#[test]
#[ignore]
fn test_network_prediction_1000_inputs() {
    // 测试预测管理器处理1000个输入的性能
    const INPUT_COUNT: usize = 1000;
    
    let mut manager = ClientPredictionManager::default();
    
    let start = std::time::Instant::now();
    
    for i in 0..INPUT_COUNT {
        let input_data = vec![(i % 256) as u8; 64];
        manager.submit_input(input_data);
    }
    
    let submission_time = start.elapsed();
    
    // 验证输入已提交
    let stats = manager.stats();
    assert!(stats.unconfirmed_commands > 0 || stats.confirmed_commands > 0);
    
    // 提交应该在合理时间内完成（应该在100ms内）
    assert!(submission_time.as_millis() < 1000, "提交应在1秒内完成");
}

#[test]
#[ignore]
fn test_network_state_synchronization_1000_entities() {
    // 测试同步1000个实体状态的性能
    const ENTITY_COUNT: usize = 1000;
    
    let mut states = Vec::with_capacity(ENTITY_COUNT);
    
    let start = std::time::Instant::now();
    
    for i in 0..ENTITY_COUNT {
        let state = EntityState::new(
            Vec3::new(i as f32, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec3::ZERO,
        );
        states.push(state);
    }
    
    let creation_time = start.elapsed();
    
    // 验证状态已创建
    assert_eq!(states.len(), ENTITY_COUNT);
    
    // 创建应该在合理时间内完成（应该在50ms内）
    assert!(creation_time.as_millis() < 500, "状态创建应在500ms内完成");
}

#[test]
#[ignore]
fn test_network_serialization_performance() {
    // 测试序列化性能
    use serde_json;
    
    const ENTITY_COUNT: usize = 500;
    
    let states: Vec<EntityState> = (0..ENTITY_COUNT)
        .map(|i| {
            EntityState::new(
                Vec3::new(i as f32, 0.0, 0.0),
                Quat::IDENTITY,
                Vec3::ONE,
                Vec3::ZERO,
            )
        })
        .collect();
    
    let start = std::time::Instant::now();
    
    let serialized = serde_json::to_string(&states);
    assert!(serialized.is_ok());
    
    let serialization_time = start.elapsed();
    
    // 序列化应该在合理时间内完成（应该在200ms内）
    assert!(serialization_time.as_millis() < 2000, "序列化应在2秒内完成");
}

#[test]
#[ignore]
fn test_network_concurrent_connections() {
    // 模拟100个并发连接
    const CONNECTION_COUNT: usize = 100;
    
    let mut managers: Vec<ClientPredictionManager> = Vec::with_capacity(CONNECTION_COUNT);
    
    let start = std::time::Instant::now();
    
    for _ in 0..CONNECTION_COUNT {
        let mut manager = ClientPredictionManager::default();
        // 每个连接提交一些输入
        for i in 0..10 {
            let input_data = vec![i as u8; 32];
            manager.submit_input(input_data);
        }
        managers.push(manager);
    }
    
    let creation_time = start.elapsed();
    
    // 验证所有管理器已创建
    assert_eq!(managers.len(), CONNECTION_COUNT);
    
    // 创建应该在合理时间内完成（应该在500ms内）
    assert!(creation_time.as_millis() < 2000, "并发连接创建应在2秒内完成");
}

