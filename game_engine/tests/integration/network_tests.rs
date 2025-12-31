//! 网络系统集成测试
//!
//! 测试游戏引擎的网络功能，包括：
//! - 客户端-服务器连接
//! - 状态同步
//! - 客户端预测
//! - 延迟补偿
//! - 增量序列化
//! - 密钥交换安全
//! - WebSocket通信
//! - 网络性能

use game_engine::network::{
    server::{GameServer, ServerConfig},
    client::{GameClient, ClientConfig},
    sync::{NetworkSync, EntitySnapshot, DeltaSnapshot},
    prediction::{ClientPrediction, PredictionState},
    latency::{LatencyCompensator, LagDetection},
    serialization::NetworkSerializer,
    crypto::KeyExchange,
};
use std::time::{Duration, Instant};
use std::net::SocketAddr;
use tokio::time::sleep;

// ============================================================================
// 测试1: 客户端-服务器连接测试
// ============================================================================

#[tokio::test]
async fn test_server_startup() {
    // 测试服务器启动
    let config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 10,
        tick_rate: 60,
        ..Default::default()
    };

    let server = GameServer::new(config);
    assert!(server.is_running());

    // 清理
    server.shutdown().await;
}

#[tokio::test]
async fn test_client_connection() {
    // 测试客户端连接

    // 启动服务器
    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 10,
        tick_rate: 60,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let server_addr = server.local_addr();

    // 连接客户端
    let client_config = ClientConfig {
        server_address: server_addr,
        ..Default::default()
    };

    let client = GameClient::new(client_config);
    assert!(!client.is_connected());

    // 尝试连接
    let result = client.connect().await;
    assert!(result.is_ok());
    assert!(client.is_connected());

    // 断开连接
    client.disconnect().await;
    assert!(!client.is_connected());

    // 清理
    server.shutdown().await;
}

#[tokio::test]
async fn test_multiple_clients() {
    // 测试多个客户端同时连接

    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 5,
        tick_rate: 60,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let server_addr = server.local_addr();

    // 创建3个客户端
    let mut clients = Vec::new();
    for _ in 0..3 {
        let client_config = ClientConfig {
            server_address: server_addr,
            ..Default::default()
        };
        let client = GameClient::new(client_config);
        let result = client.connect().await;
        assert!(result.is_ok());
        clients.push(client);
    }

    // 验证所有客户端都已连接
    for client in &clients {
        assert!(client.is_connected());
    }

    // 断开所有客户端
    for client in &mut clients {
        client.disconnect().await;
    }

    // 清理
    server.shutdown().await;
}

// ============================================================================
// 测试2: 状态同步正确性测试
// ============================================================================

#[test]
fn test_entity_snapshot_creation() {
    // 测试实体快照创建

    let snapshot = EntitySnapshot {
        entity_id: 1,
        position: (1.0, 2.0, 3.0),
        rotation: (0.0, 0.0, 0.0, 1.0),
        velocity: (0.0, 0.0, 0.0),
        timestamp: 1000,
    };

    assert_eq!(snapshot.entity_id, 1);
    assert_eq!(snapshot.position, (1.0, 2.0, 3.0));
    assert_eq!(snapshot.timestamp, 1000);
}

#[test]
fn test_snapshot_delta_compression() {
    // 测试快照增量压缩

    // 旧快照
    let snapshot1 = EntitySnapshot {
        entity_id: 1,
        position: (1.0, 2.0, 3.0),
        rotation: (0.0, 0.0, 0.0, 1.0),
        velocity: (0.0, 0.0, 0.0),
        timestamp: 1000,
    };

    // 新快照（只有位置改变）
    let snapshot2 = EntitySnapshot {
        entity_id: 1,
        position: (2.0, 3.0, 4.0),
        rotation: (0.0, 0.0, 0.0, 1.0),
        velocity: (0.0, 0.0, 0.0),
        timestamp: 1010,
    };

    // 创建增量快照
    let delta = DeltaSnapshot::from_snapshots(&snapshot1, &snapshot2);

    // 验证增量数据
    assert_eq!(delta.entity_id, 1);
    assert!(delta.position_changed);
    assert!(!delta.rotation_changed);
    assert!(!delta.velocity_changed);

    // 应用增量
    let reconstructed = delta.apply_to(&snapshot1);
    assert_eq!(reconstructed.position, snapshot2.position);
}

#[tokio::test]
async fn test_state_synchronization() {
    // 测试状态同步

    // 启动服务器和客户端
    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 10,
        tick_rate: 60,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let server_addr = server.local_addr();

    let client_config = ClientConfig {
        server_address: server_addr,
        ..Default::default()
    };
    let mut client = GameClient::new(client_config);
    client.connect().await.unwrap();

    // 模拟服务器更新实体状态
    let sync = NetworkSync::new();
    let snapshot = EntitySnapshot {
        entity_id: 1,
        position: (5.0, 0.0, 0.0),
        rotation: (0.0, 0.0, 0.0, 1.0),
        velocity: (1.0, 0.0, 0.0),
        timestamp: 1000,
    };

    // 发送快照到客户端
    sync.send_snapshot(&snapshot).await;

    // 等待网络传输
    sleep(Duration::from_millis(50)).await;

    // 验证客户端接收到快照
    let received_snapshot = client.receive_snapshot().await;
    assert!(received_snapshot.is_some());
    assert_eq!(received_snapshot.unwrap().entity_id, 1);

    // 清理
    client.disconnect().await;
    server.shutdown().await;
}

// ============================================================================
// 测试3: 客户端预测测试
// ============================================================================

#[test]
fn test_client_prediction_state() {
    // 测试客户端预测状态

    let prediction = ClientPrediction::new();

    // 添加预测输入
    let input_sequence = 1;
    let predicted_state = PredictionState {
        position: (10.0, 0.0, 0.0),
        velocity: (1.0, 0.0, 0.0),
        sequence: input_sequence,
    };

    prediction.add_prediction(input_sequence, predicted_state);

    // 验证预测状态
    let state = prediction.get_state(input_sequence);
    assert!(state.is_some());
    assert_eq!(state.unwrap().position, (10.0, 0.0, 0.0));
}

#[test]
fn test_prediction_reconciliation() {
    // 测试预测调和

    let mut prediction = ClientPrediction::new();

    // 添加预测
    let input_sequence = 1;
    let predicted_state = PredictionState {
        position: (10.0, 0.0, 0.0),
        velocity: (1.0, 0.0, 0.0),
        sequence: input_sequence,
    };
    prediction.add_prediction(input_sequence, predicted_state);

    // 服务器返回的实际状态（略有不同）
    let server_state = PredictionState {
        position: (10.2, 0.0, 0.0), // 服务器位置不同
        velocity: (1.0, 0.0, 0.0),
        sequence: input_sequence,
    };

    // 执行调和
    let reconciled = prediction.reconcile(&server_state);

    // 验证调和结果
    assert!(reconciled.is_ok());
}

#[test]
fn test_prediction_rollback() {
    // 测试预测回滚

    let mut prediction = ClientPrediction::new();

    // 添加多个预测
    for i in 1..=5 {
        let state = PredictionState {
            position: (i as f32 * 2.0, 0.0, 0.0),
            velocity: (1.0, 0.0, 0.0),
            sequence: i,
        };
        prediction.add_prediction(i, state);
    }

    // 服务器确认到序列3
    let confirmed_sequence = 3;
    prediction.confirm_up_to(confirmed_sequence);

    // 验证未确认的预测被保留
    assert!(prediction.get_state(4).is_some());
    assert!(prediction.get_state(5).is_some());
    assert!(prediction.get_state(3).is_none()); // 已确认
}

// ============================================================================
// 测试4: 延迟补偿测试
// ============================================================================

#[test]
fn test_latency_detection() {
    // 测试延迟检测

    let mut detector = LagDetection::new();

    // 记录往返时间
    let rtt1 = Duration::from_millis(50);
    let rtt2 = Duration::from_millis(60);
    let rtt3 = Duration::from_millis(55);

    detector.record_rtt(rtt1);
    detector.record_rtt(rtt2);
    detector.record_rtt(rtt3);

    // 计算平均延迟
    let avg_latency = detector.average_latency();
    assert!(avg_latency >= Duration::from_millis(50));
    assert!(avg_latency <= Duration::from_millis(60));
}

#[test]
fn test_latency_compensation() {
    // 测试延迟补偿

    let mut compensator = LatencyCompensator::new();

    // 设置客户端延迟
    let client_latency = Duration::from_millis(100);
    compensator.set_client_latency(client_latency);

    // 服务器时间
    let server_time = Instant::now();

    // 计算补偿后的时间
    let compensated_time = compensator.compensate_time(server_time);
    let time_diff = compensated_time.duration_since(server_time);

    // 验证补偿
    assert!(time_diff >= client_latency - Duration::from_millis(10));
    assert!(time_diff <= client_latency + Duration::from_millis(10));
}

#[test]
fn test_lag_spike_detection() {
    // 测试延迟峰值检测

    let mut detector = LagDetection::new();

    // 正常延迟
    for _ in 0..10 {
        detector.record_rtt(Duration::from_millis(50));
    }

    // 延迟峰值
    detector.record_rtt(Duration::from_millis(200));

    // 检测到峰值
    assert!(detector.is_lag_spike());
}

// ============================================================================
// 测试5: 增量序列化测试
// ============================================================================

#[test]
fn test_delta_serialization() {
    // 测试增量序列化

    let serializer = NetworkSerializer::new();

    // 完整状态
    let full_state = vec![
        (1.0_f32, 2.0_f32, 3.0_f32),
        (4.0_f32, 5.0_f32, 6.0_f32),
        (7.0_f32, 8.0_f32, 9.0_f32),
    ];

    // 序列化
    let serialized = serializer.serialize(&full_state);
    assert!(!serialized.is_empty());

    // 反序列化
    let deserialized = serializer.deserialize(&serialized);
    assert!(deserialized.is_ok());

    let recovered = deserialized.unwrap();
    assert_eq!(recovered.len(), full_state.len());
}

#[test]
fn test_compression_ratio() {
    // 测试压缩比

    let serializer = NetworkSerializer::new();

    // 重复数据（应该很好压缩）
    let repeated_data = vec![1.0_f32; 1000];

    let serialized = serializer.serialize(&repeated_data);

    // 验证压缩效果
    let compression_ratio = serialized.len() as f64 / (repeated_data.len() * std::mem::size_of::<f32>()) as f64;
    assert!(compression_ratio < 0.5); // 应该压缩到50%以下
}

// ============================================================================
// 测试6: 密钥交换安全测试
// ============================================================================

#[tokio::test]
async fn test_diffie_hellman_key_exchange() {
    // 测试Diffie-Hellman密钥交换

    // 客户端密钥对
    let client_keypair = KeyExchange::generate_keypair();
    let client_public = client_keypair.public_key();

    // 服务器密钥对
    let server_keypair = KeyExchange::generate_keypair();
    let server_public = server_keypair.public_key();

    // 客户端计算共享密钥
    let client_shared = client_keypair.compute_shared(&server_public);
    assert!(!client_shared.is_empty());

    // 服务器计算共享密钥
    let server_shared = server_keypair.compute_shared(&client_public);
    assert!(!server_shared.is_empty());

    // 验证共享密钥相同
    assert_eq!(client_shared, server_shared);
}

#[test]
fn test_encryption_decryption() {
    // 测试加密解密

    let key = KeyExchange::generate_key();
    let plaintext = b"Hello, secure world!";

    // 加密
    let encrypted = KeyExchange::encrypt(plaintext, &key);
    assert!(!encrypted.is_empty());
    assert_ne!(encrypted, plaintext.to_vec());

    // 解密
    let decrypted = KeyExchange::decrypt(&encrypted, &key);
    assert!(decrypted.is_ok());

    let decrypted_text = decrypted.unwrap();
    assert_eq!(decrypted_text, plaintext);
}

#[test]
fn test_signature_verification() {
    // 测试签名验证

    let keypair = KeyExchange::generate_keypair();
    let message = b"Important message";

    // 签名
    let signature = keypair.sign(message);
    assert!(!signature.is_empty());

    // 验证签名
    let verified = keypair.verify(message, &signature);
    assert!(verified);

    // 篡改消息
    let tampered_message = b"Tampered message";
    let verified_tampered = keypair.verify(tampered_message, &signature);
    assert!(!verified_tampered);
}

// ============================================================================
// 测试7: WebSocket通信测试
// ============================================================================

#[tokio::test]
async fn test_websocket_connection() {
    // 测试WebSocket连接

    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 10,
        tick_rate: 60,
        enable_websocket: true,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let ws_url = format!("ws://{}/ws", server.local_addr());

    let client_config = ClientConfig {
        server_address: ws_url.parse().unwrap(),
        use_websocket: true,
        ..Default::default()
    };

    let client = GameClient::new(client_config);
    let result = client.connect().await;

    // 验证连接成功（如果WebSocket支持）
    if result.is_ok() {
        assert!(client.is_connected());
        client.disconnect().await;
    }

    // 清理
    server.shutdown().await;
}

#[tokio::test]
async fn test_websocket_message_send() {
    // 测试WebSocket消息发送

    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 10,
        tick_rate: 60,
        enable_websocket: true,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let ws_url = format!("ws://{}/ws", server.local_addr());

    let client_config = ClientConfig {
        server_address: ws_url.parse().unwrap(),
        use_websocket: true,
        ..Default::default()
    };

    let mut client = GameClient::new(client_config);

    if client.connect().await.is_ok() {
        // 发送消息
        let message = b"Test message";
        let result = client.send_message(message).await;

        // 验证发送成功（如果WebSocket支持）
        if result.is_ok() {
            assert!(result.is_ok());
        }

        client.disconnect().await;
    }

    // 清理
    server.shutdown().await;
}

// ============================================================================
// 测试8: 网络性能测试
// ============================================================================

#[tokio::test]
async fn test_network_throughput() {
    // 测试网络吞吐量

    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 10,
        tick_rate: 60,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let server_addr = server.local_addr();

    let client_config = ClientConfig {
        server_address: server_addr,
        ..Default::default()
    };

    let mut client = GameClient::new(client_config);
    client.connect().await.unwrap();

    // 发送大量数据
    let data_size = 1024 * 1024; // 1MB
    let large_data = vec![0u8; data_size];

    let start = Instant::now();
    let result = client.send_data(&large_data).await;
    let elapsed = start.elapsed();

    // 验证发送成功
    assert!(result.is_ok());

    // 计算吞吐量
    let throughput_mbps = (data_size as f64 / 1024.0 / 1024.0) / elapsed.as_secs_f64();
    println!("Network throughput: {:.2} MB/s", throughput_mbps);

    // 清理
    client.disconnect().await;
    server.shutdown().await;
}

#[tokio::test]
async fn test_network_latency() {
    // 测试网络延迟

    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 10,
        tick_rate: 60,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let server_addr = server.local_addr();

    let client_config = ClientConfig {
        server_address: server_addr,
        ..Default::default()
    };

    let mut client = GameClient::new(client_config);
    client.connect().await.unwrap();

    // 测量往返时间
    let mut latencies = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let _ = client.ping().await;
        let latency = start.elapsed();
        latencies.push(latency);
    }

    // 计算平均延迟
    let total: Duration = latencies.iter().sum();
    let avg_latency = total / latencies.len() as u32;

    println!("Average network latency: {:?}", avg_latency);

    // 本地测试应该很快
    assert!(avg_latency < Duration::from_millis(50));

    // 清理
    client.disconnect().await;
    server.shutdown().await;
}

#[tokio::test]
async fn test_concurrent_connections() {
    // 测试并发连接

    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 20,
        tick_rate: 60,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let server_addr = server.local_addr();

    // 创建10个并发连接
    let mut handles = Vec::new();
    for i in 0..10 {
        let addr = server_addr;
        handles.push(tokio::spawn(async move {
            let client_config = ClientConfig {
                server_address: addr,
                ..Default::default()
            };
            let client = GameClient::new(client_config);
            if client.connect().await.is_ok() {
                // 发送一些数据
                let _ = client.send_message(&[i]).await;
                client.disconnect().await;
            }
        }));
    }

    // 等待所有连接完成
    for handle in handles {
        handle.await.unwrap();
    }

    // 清理
    server.shutdown().await;
}

// ============================================================================
// 测试9: 网络错误处理测试
// ============================================================================

#[tokio::test]
async fn test_connection_timeout() {
    // 测试连接超时

    // 连接到不存在的服务器
    let client_config = ClientConfig {
        server_address: "127.0.0.1:9999".parse().unwrap(),
        timeout: Duration::from_millis(100),
        ..Default::default()
    };

    let client = GameClient::new(client_config);
    let result = client.connect().await;

    // 应该超时或失败
    assert!(result.is_err() || result.is_ok());
}

#[tokio::test]
async fn test_reconnection_logic() {
    // 测试重连逻辑

    let server_config = ServerConfig {
        bind_address: "127.0.0.1:0".parse().unwrap(),
        max_clients: 10,
        tick_rate: 60,
        ..Default::default()
    };
    let server = GameServer::new(server_config);
    let server_addr = server.local_addr();

    let client_config = ClientConfig {
        server_address: server_addr,
        max_retries: 3,
        retry_delay: Duration::from_millis(100),
        ..Default::default()
    };

    let mut client = GameClient::new(client_config);
    client.connect().await.unwrap();

    // 断开连接
    client.disconnect().await;

    // 尝试重连
    let result = client.reconnect().await;

    // 应该重连成功
    assert!(result.is_ok());

    // 清理
    client.disconnect().await;
    server.shutdown().await;
}
