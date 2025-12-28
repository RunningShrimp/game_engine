//  网络系统性能基准测试
//
//  测试消息序列化、网络消息处理等操作的性能

use bincode;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use game_engine::network::security::MessageEncryptor;
use game_engine::network::{NetworkMessage, key_exchange};

fn bench_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");

    // 创建测试消息
    let messages = vec![
        NetworkMessage::Connect {
            client_id: 1,
            name: "TestClient".to_string(),
        },
        NetworkMessage::Disconnect { client_id: 1 },
        NetworkMessage::StateSync {
            tick: 1,
            data: vec![1, 2, 3, 4],
        },
        NetworkMessage::Rpc {
            id: 1,
            method: "test".to_string(),
            params: vec![1, 2, 3],
        },
        NetworkMessage::Heartbeat {
            timestamp: 1234567890,
        },
    ];

    for message in messages {
        let message_name = match &message {
            NetworkMessage::Connect { .. } => "connect",
            NetworkMessage::Disconnect { .. } => "disconnect",
            NetworkMessage::StateSync { .. } => "state_sync",
            NetworkMessage::Rpc { .. } => "rpc",
            NetworkMessage::Heartbeat { .. } => "heartbeat",
            _ => "unknown",
        };

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_serialize", message_name)),
            &message,
            |b, msg| {
                b.iter(|| std::hint::black_box(bincode::serialize(msg).unwrap()));
            },
        );

        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}_deserialize", message_name)),
            &message,
            |b, msg| {
                let serialized = bincode::serialize(msg).unwrap();
                b.iter(|| std::hint::black_box(bincode::deserialize::<NetworkMessage>(&serialized).unwrap()));
            },
        );
    }

    group.finish();
}

fn bench_key_exchange(c: &mut Criterion) {
    let mut group = c.benchmark_group("key_exchange");

    group.bench_function("generate_keypair", |b| {
        b.iter(|| std::hint::black_box(key_exchange::KeyPair::generate()));
    });

    group.finish();
}

fn bench_message_encryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_encryption");

    let key = [42u8; 32];
    let mut encryptor = MessageEncryptor::new(key);
    let test_data = b"Hello, this is a test message for encryption benchmarking!";

    group.bench_function("encrypt_message", |b| {
        b.iter(|| std::hint::black_box(encryptor.encrypt(test_data).unwrap()));
    });

    // 预加密一个消息用于解密测试
    let encrypted = encryptor.encrypt(test_data).unwrap();

    group.bench_function("decrypt_message", |b| {
        b.iter(|| std::hint::black_box(encryptor.decrypt(&encrypted).unwrap()));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_message_serialization,
    bench_key_exchange,
    bench_message_encryption
);
criterion_main!(benches);
