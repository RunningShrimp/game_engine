// 序列化性能基准测试
//
// 测试消息序列化、场景保存/加载等序列化功能

use bincode;
use criterion::{black_box, BenchmarkId, Criterion, criterion_group, criterion_main};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 测试用的网络消息结构
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct NetworkMessage {
    message_id: u64,
    timestamp: u64,
    player_id: u32,
    message_type: MessageType,
    data: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
enum MessageType {
    PlayerUpdate { position: (f32, f32, f32), rotation: (f32, f32, f32, f32) },
    GameState { score: u32, level: u32 },
    ChatMessage { text: String },
    EntitySpawn { entity_type: u32, position: (f32, f32, f32) },
}

/// 测试用的场景数据结构
#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct SceneData {
    name: String,
    entities: Vec<EntityData>,
    lights: Vec<LightData>,
    metadata: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct EntityData {
    id: u64,
    transform: TransformData,
    mesh: MeshData,
    material_id: u32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct TransformData {
    position: (f32, f32, f32),
    rotation: (f32, f32, f32, f32),
    scale: (f32, f32, f32),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MeshData {
    vertex_count: u32,
    triangle_count: u32,
    vertices: Vec<(f32, f32, f32)>,
    indices: Vec<u32>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct LightData {
    light_type: u32,
    color: (f32, f32, f32),
    intensity: f32,
    position: (f32, f32, f32),
}

/// Benchmark网络消息序列化性能
fn bench_message_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_serialization");

    for message_size in [64, 256, 1024, 4096].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(message_size), message_size, |b, &size| {
            let message = NetworkMessage {
                message_id: 12345,
                timestamp: 1234567890,
                player_id: 1,
                message_type: MessageType::PlayerUpdate {
                    position: (1.0, 2.0, 3.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                },
                data: vec![0u8; size],
            };

            b.iter(|| {
                let serialized = bincode::serialize(black_box(&message)).unwrap();
                black_box(serialized)
            });
        });
    }

    group.finish();
}

/// Benchmark网络消息反序列化性能
fn bench_message_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("message_deserialization");

    for message_size in [64, 256, 1024, 4096].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(message_size), message_size, |b, &size| {
            let message = NetworkMessage {
                message_id: 12345,
                timestamp: 1234567890,
                player_id: 1,
                message_type: MessageType::PlayerUpdate {
                    position: (1.0, 2.0, 3.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                },
                data: vec![0u8; size],
            };

            let serialized = bincode::serialize(&message).unwrap();

            b.iter(|| {
                let deserialized: NetworkMessage = bincode::deserialize(black_box(&serialized)).unwrap();
                black_box(deserialized)
            });
        });
    }

    group.finish();
}

/// Benchmark场景序列化性能
fn bench_scene_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_serialization");

    for entity_count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(entity_count), entity_count, |b, &count| {
            let scene = create_test_scene(count);

            b.iter(|| {
                let serialized = bincode::serialize(black_box(&scene)).unwrap();
                black_box(serialized)
            });
        });
    }

    group.finish();
}

/// Benchmark场景反序列化性能
fn bench_scene_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("scene_deserialization");

    for entity_count in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(entity_count), entity_count, |b, &count| {
            let scene = create_test_scene(count);
            let serialized = bincode::serialize(&scene).unwrap();

            b.iter(|| {
                let deserialized: SceneData = bincode::deserialize(black_box(&serialized)).unwrap();
                black_box(deserialized)
            });
        });
    }

    group.finish();
}

/// Benchmark JSON序列化性能（用于对比）
fn bench_json_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_serialization");

    for entity_count in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(entity_count), entity_count, |b, &count| {
            let scene = create_test_scene(count);

            b.iter(|| {
                let serialized = serde_json::to_string(black_box(&scene)).unwrap();
                black_box(serialized)
            });
        });
    }

    group.finish();
}

/// Benchmark JSON反序列化性能（用于对比）
fn bench_json_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("json_deserialization");

    for entity_count in [10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(entity_count), entity_count, |b, &count| {
            let scene = create_test_scene(count);
            let serialized = serde_json::to_string(&scene).unwrap();

            b.iter(|| {
                let deserialized: SceneData = serde_json::from_str(black_box(&serialized)).unwrap();
                black_box(deserialized)
            });
        });
    }

    group.finish();
}

/// Benchmark存档保存性能
fn bench_save_game(c: &mut Criterion) {
    let mut group = c.benchmark_group("save_game");

    let save_data = create_large_save_data();

    group.bench_function("save_large_game", |b| {
        b.iter(|| {
            let compressed = bincode::serialize(black_box(&save_data)).unwrap();
            black_box(compressed)
        });
    });

    group.finish();
}

/// Benchmark存档加载性能
fn bench_load_game(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_game");

    let save_data = create_large_save_data();
    let serialized = bincode::serialize(&save_data).unwrap();

    group.bench_function("load_large_game", |b| {
        b.iter(|| {
            let deserialized: SaveGameData = bincode::deserialize(black_box(&serialized)).unwrap();
            black_box(deserialized)
        });
    });

    group.finish();
}

/// Benchmark压缩性能
fn bench_compression(c: &mut Criterion) {
    use flate2::write::{GzEncoder, DeflateEncoder};
    use flate2::Compression;
    use std::io::Write;

    let mut group = c.benchmark_group("compression");

    let data = create_test_scene(1000);
    let serialized = bincode::serialize(&data).unwrap();

    group.bench_function("gzip_compress", |b| {
        b.iter(|| {
            let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&serialized)).unwrap();
            let compressed = encoder.finish().unwrap();
            black_box(compressed)
        });
    });

    group.bench_function("deflate_compress", |b| {
        b.iter(|| {
            let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(black_box(&serialized)).unwrap();
            let compressed = encoder.finish().unwrap();
            black_box(compressed)
        });
    });

    group.finish();
}

// Helper functions

fn create_test_scene(entity_count: usize) -> SceneData {
    let mut entities = Vec::with_capacity(entity_count);
    let mut metadata = HashMap::new();

    metadata.insert("version".to_string(), "1.0".to_string());
    metadata.insert("author".to_string(), "Benchmark".to_string());

    for i in 0..entity_count {
        entities.push(EntityData {
            id: i as u64,
            transform: TransformData {
                position: (i as f32, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0, 1.0),
                scale: (1.0, 1.0, 1.0),
            },
            mesh: MeshData {
                vertex_count: 1000,
                triangle_count: 500,
                vertices: vec![(0.0, 0.0, 0.0); 1000],
                indices: vec![0; 1500],
            },
            material_id: i as u32 % 10,
        });
    }

    let lights = vec![
        LightData {
            light_type: 0,
            color: (1.0, 1.0, 1.0),
            intensity: 1.0,
            position: (0.0, 10.0, 0.0),
        },
        LightData {
            light_type: 1,
            color: (0.8, 0.8, 1.0),
            intensity: 0.5,
            position: (5.0, 5.0, 5.0),
        },
    ];

    SceneData {
        name: "TestScene".to_string(),
        entities,
        lights,
        metadata,
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct SaveGameData {
    player_data: PlayerData,
    world_data: WorldData,
    inventory: Vec<ItemData>,
    quest_progress: Vec<QuestData>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct PlayerData {
    position: (f32, f32, f32),
    health: f32,
    max_health: f32,
    level: u32,
    experience: u64,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct WorldData {
    current_level: u32,
    visited_areas: Vec<String>,
    world_time: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ItemData {
    item_id: u32,
    quantity: u32,
    durability: f32,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct QuestData {
    quest_id: u32,
    status: u32,
    objectives_completed: u32,
    total_objectives: u32,
}

fn create_large_save_data() -> SaveGameData {
    SaveGameData {
        player_data: PlayerData {
            position: (100.0, 50.0, 200.0),
            health: 100.0,
            max_health: 100.0,
            level: 25,
            experience: 50000,
        },
        world_data: WorldData {
            current_level: 10,
            visited_areas: (0..100).map(|i| format!("Area_{}", i)).collect(),
            world_time: 12345.67,
        },
        inventory: (0..50).map(|i| ItemData {
            item_id: i,
            quantity: 10,
            durability: 100.0,
        }).collect(),
        quest_progress: (0..20).map(|i| QuestData {
            quest_id: i,
            status: 1,
            objectives_completed: 3,
            total_objectives: 5,
        }).collect(),
    }
}

criterion_group!(
    benches,
    bench_message_serialization,
    bench_message_deserialization,
    bench_scene_serialization,
    bench_scene_deserialization,
    bench_json_serialization,
    bench_json_deserialization,
    bench_save_game,
    bench_load_game,
    bench_compression
);
criterion_main!(benches);
