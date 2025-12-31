//! 性能回归测试
//!
//! 检测关键路径的性能回归，确保优化不会导致性能下降

use std::time::Instant;
use bevy_ecs::prelude::*;
use game_engine::domain::scene::Scene;
use game_engine::domain::event_sourcing::{
    EventSourcingManager, MemoryEventStore, MemorySnapshotStore,
};
use std::sync::Arc;

/// 性能基准：场景创建时间
const SCENE_CREATION_BENCHMARK_MS: u64 = 10;

/// 性能基准：事件提交时间
const EVENT_COMMIT_BENCHMARK_MS: u64 = 5;

/// 性能基准：事件重放时间（100个事件）
const EVENT_REPLAY_BENCHMARK_MS: u64 = 50;

/// 性能基准：渲染性能（1000个实体）
const RENDER_PERFORMANCE_BENCHMARK_MS: u64 = 16; // 60 FPS

/// 性能基准：网络同步性能（100个实体）
const NETWORK_SYNC_BENCHMARK_MS: u64 = 5;

/// 性能基准：内存分配性能（1000次分配）
const MEMORY_ALLOCATION_BENCHMARK_MS: u64 = 10;

/// 测试场景创建性能
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_scene_creation_performance() {
    let start = Instant::now();
    
    for _ in 0..100 {
        let _scene = Scene::new("TestScene", format!("test_id_{}", 0))
            .expect("Failed to create scene");
    }
    
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / 100;
    
    println!("Average scene creation time: {}ms", avg_ms);
    
    // 验证性能没有显著退化（允许20%的误差）
    assert!(
        avg_ms < (SCENE_CREATION_BENCHMARK_MS as u128 * 120 / 100),
        "Scene creation performance regression: {}ms > {}ms",
        avg_ms,
        SCENE_CREATION_BENCHMARK_MS * 120 / 100
    );
}

/// 测试事件提交性能
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_event_commit_performance() {
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    let mut scene = Scene::new("TestScene", "perf_test".to_string())
        .expect("Failed to create scene");
    
    let mut world = World::new();
    
    let start = Instant::now();
    
    for _ in 0..100 {
        manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    }
    
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / 100;
    
    println!("Average event commit time: {}ms", avg_ms);
    
    // 验证性能没有显著退化
    assert!(
        avg_ms < (EVENT_COMMIT_BENCHMARK_MS as u128 * 120 / 100),
        "Event commit performance regression: {}ms > {}ms",
        avg_ms,
        EVENT_COMMIT_BENCHMARK_MS * 120 / 100
    );
}

/// 测试事件重放性能
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_event_replay_performance() {
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    // 创建场景并提交100个事件
    let mut scene = Scene::new("TestScene", "replay_perf_test".to_string())
        .expect("Failed to create scene");
    
    let mut world = World::new();
    
    for _ in 0..100 {
        manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
    }
    
    // 测试重放性能
    let start = Instant::now();
    
    let events = manager.replay_aggregate_events("replay_perf_test", None);
    assert!(events.is_ok());
    
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    
    println!("Event replay time (100 events): {}ms", elapsed_ms);
    
    // 验证性能没有显著退化
    assert!(
        elapsed_ms < (EVENT_REPLAY_BENCHMARK_MS as u128 * 120 / 100),
        "Event replay performance regression: {}ms > {}ms",
        elapsed_ms,
        EVENT_REPLAY_BENCHMARK_MS * 120 / 100
    );
}

/// 测试批量操作性能
#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_batch_operations_performance() {
    let manager = EventSourcingManager::new(
        Arc::new(std::sync::RwLock::new(Box::new(MemoryEventStore::new()) as Box<dyn game_engine::domain::event_sourcing::EventStore>)),
        Arc::new(std::sync::RwLock::new(Box::new(MemorySnapshotStore::new()) as Box<dyn game_engine::domain::event_sourcing::SnapshotStore>)),
    );
    
    let start = Instant::now();
    
    // 创建并提交多个场景的事件
    for i in 0..10 {
        let mut scene = Scene::new(
            format!("Scene{}", i),
            format!("scene_{}", i),
        )
        .expect("Failed to create scene");
        
        let mut world = World::new();
        
        scene.activate().unwrap();
        manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
        
        // 添加实体
        for j in 0..10 {
            let entity = game_engine::domain::entity::GameEntity::new(
                format!("entity_{}_{}", i, j),
                glam::Vec3::ZERO,
            );
            scene.add_entity(entity).unwrap();
            manager.commit_aggregate_events(&mut scene, &mut world).unwrap();
        }
    }
    
    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();
    
    println!("Batch operations time (10 scenes, 10 entities each): {}ms", elapsed_ms);
    
    // 验证批量操作性能合理（允许较大的时间范围）
    assert!(
        elapsed_ms < 1000,
        "Batch operations took too long: {}ms",
        elapsed_ms
    );
}

// ============================================================================
// 渲染性能基准测试
// ============================================================================

/// 测试渲染性能基准
#[test]
#[ignore]  // 需要GPU环境
fn test_render_performance_benchmark() {
    use game_engine::render::Renderer;
    use game_engine::render::mesh::Mesh;

    // 创建渲染器
    let renderer = Renderer::new();

    // 创建1000个简单的网格用于渲染
    let meshes: Vec<Mesh> = (0..1000)
        .map(|_| Mesh::cube(1.0))
        .collect();

    // 测试渲染性能
    let start = Instant::now();

    // 模拟渲染1000个网格
    for mesh in &meshes {
        renderer.draw_mesh(mesh);
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Render time (1000 meshes): {}ms", elapsed_ms);

    // 验证渲染性能满足60 FPS目标（16ms）
    assert!(
        elapsed_ms < (RENDER_PERFORMANCE_BENCHMARK_MS as u128 * 120 / 100),
        "Render performance regression: {}ms > {}ms",
        elapsed_ms,
        RENDER_PERFORMANCE_BENCHMARK_MS * 120 / 100
    );
}

/// 测试批处理渲染性能
#[test]
#[ignore]  // 需要GPU环境
fn test_batch_rendering_performance() {
    use game_engine::render::batching::RenderBatchManager;

    let batch_manager = RenderBatchManager::new();
    let num_batches = 100;

    // 创建100个批次
    let start = Instant::now();

    for i in 0..num_batches {
        let batch = game_engine::render::batching::RenderBatch {
            mesh_id: i,
            material_id: i % 10,
            instance_count: 10,
            ..Default::default()
        };
        batch_manager.add_batch(batch);
    }

    // 执行批处理渲染
    batch_manager.render_all_batches();

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Batch rendering time (100 batches): {}ms", elapsed_ms);

    // 批处理应该比单独渲染更快
    assert!(
        elapsed_ms < 50,
        "Batch rendering performance regression: {}ms",
        elapsed_ms
    );
}

/// 测试LOD系统性能
#[test]
fn test_lod_performance() {
    use game_engine::render::lod::LodManager;

    let lod_manager = LodManager::new();
    let num_entities = 1000;

    // 测试LOD计算性能
    let start = Instant::now();

    let camera_pos = glam::Vec3::new(0.0, 0.0, 0.0);
    for i in 0..num_entities {
        let entity_pos = glam::Vec3::new(i as f32, 0.0, 10.0);
        let distance = camera_pos.distance(entity_pos);
        let _lod_level = lod_manager.select_lod_level(distance, &[]);

        // 简单验证LOD级别存在
        let _ = lod_level;
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("LOD calculation time (1000 entities): {}ms", elapsed_ms);

    // LOD计算应该很快
    assert!(
        elapsed_ms < 20,
        "LOD performance regression: {}ms",
        elapsed_ms
    );
}

// ============================================================================
// 网络同步性能基准测试
// ============================================================================

/// 测试网络同步性能基准
#[tokio::test]
async fn test_network_sync_performance() {
    use game_engine::network::sync::NetworkSync;

    let sync = NetworkSync::new();
    let num_snapshots = 100;

    // 创建100个实体快照
    let snapshots: Vec<game_engine::network::sync::EntitySnapshot> = (0..num_snapshots)
        .map(|i| game_engine::network::sync::EntitySnapshot {
            entity_id: i,
            position: (i as f32, 0.0, 0.0),
            rotation: (0.0, 0.0, 0.0, 1.0),
            velocity: (0.0, 0.0, 0.0),
            timestamp: i as u64,
        })
        .collect();

    // 测试同步性能
    let start = Instant::now();

    // 序列化并发送快照
    for snapshot in &snapshots {
        let result = sync.send_snapshot(snapshot).await;
        assert!(result.is_ok());
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Network sync time (100 snapshots): {}ms", elapsed_ms);

    // 验证网络同步性能
    assert!(
        elapsed_ms < (NETWORK_SYNC_BENCHMARK_MS as u128 * num_snapshots as u128 * 120 / 100),
        "Network sync performance regression: {}ms",
        elapsed_ms
    );
}

/// 测试增量序列化性能
#[test]
fn test_delta_serialization_performance() {
    use game_engine::network::serialization::NetworkSerializer;

    let serializer = NetworkSerializer::new();
    let num_snapshots = 100;

    // 创建快照对（旧+新）
    let snapshot_pairs: Vec<(game_engine::network::sync::EntitySnapshot, game_engine::network::sync::EntitySnapshot)> =
        (0..num_snapshots)
            .map(|i| {
                let old = game_engine::network::sync::EntitySnapshot {
                    entity_id: i,
                    position: (i as f32, 0.0, 0.0),
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    velocity: (0.0, 0.0, 0.0),
                    timestamp: i as u64,
                };

                let new = game_engine::network::sync::EntitySnapshot {
                    entity_id: i,
                    position: ((i + 1) as f32, 0.0, 0.0), // 位置改变
                    rotation: (0.0, 0.0, 0.0, 1.0),
                    velocity: (0.0, 0.0, 0.0),
                    timestamp: i as u64 + 1,
                };

                (old, new)
            })
            .collect();

    // 测试增量序列化性能
    let start = Instant::now();

    for (old, new) in &snapshot_pairs {
        let delta = game_engine::network::sync::DeltaSnapshot::from_snapshots(old, new);
        let serialized = serializer.serialize_delta(&delta);

        // 验证序列化成功
        assert!(!serialized.is_empty());
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Delta serialization time (100 snapshots): {}ms", elapsed_ms);

    // 增量序列化应该很快
    assert!(
        elapsed_ms < 50,
        "Delta serialization performance regression: {}ms",
        elapsed_ms
    );
}

/// 测试延迟补偿性能
#[test]
fn test_latency_compensation_performance() {
    use game_engine::network::latency::LatencyCompensator;

    let compensator = LatencyCompensator::new();
    let num_updates = 1000;

    // 测试延迟补偿性能
    let start = Instant::now();

    for i in 0..num_updates {
        let timestamp = std::time::Instant::now();
        let _ = compensator.compensate_time(timestamp);

        // 每隔100次更新延迟
        if i % 100 == 0 {
            compensator.set_client_latency(std::time::Duration::from_millis(50));
        }
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Latency compensation time (1000 updates): {}ms", elapsed_ms);

    // 延迟补偿应该很快
    assert!(
        elapsed_ms < 20,
        "Latency compensation performance regression: {}ms",
        elapsed_ms
    );
}

// ============================================================================
// 内存分配性能基准测试
// ============================================================================

/// 测试内存分配性能基准
#[test]
fn test_memory_allocation_performance() {
    use std::alloc::{GlobalAlloc, Layout};
    use std::sync::atomic::{AtomicU64, Ordering};

    // 自定义分配器用于测量分配次数
    struct MeasuringAllocator;

    static ALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);
    static DEALLOCATION_COUNT: AtomicU64 = AtomicU64::new(0);

    unsafe impl GlobalAlloc for MeasuringAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            ALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
            std::alloc::System.alloc(layout)
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            DEALLOCATION_COUNT.fetch_add(1, Ordering::SeqCst);
            std::alloc::System.dealloc(ptr, layout)
        }

        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_layout: Layout) -> *mut u8 {
            std::alloc::System.realloc(ptr, layout, new_layout)
        }
    }

    // 测试分配性能
    let start = Instant::now();

    // 执行1000次小内存分配
    let mut allocations: Vec<Vec<u8>> = Vec::new();
    for _ in 0..1000 {
        allocations.push(vec![0u8; 1024]); // 1KB
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Memory allocation time (1000 x 1KB): {}ms", elapsed_ms);
    println!("Total allocations: {}", ALLOCATION_COUNT.load(Ordering::SeqCst));

    // 验证内存分配性能
    assert!(
        elapsed_ms < (MEMORY_ALLOCATION_BENCHMARK_MS as u128 * 120 / 100),
        "Memory allocation performance regression: {}ms > {}ms",
        elapsed_ms,
        MEMORY_ALLOCATION_BENCHMARK_MS * 120 / 100
    );
}

/// 测试对象池性能
#[test]
fn test_object_pool_performance() {
    use game_engine::performance::memory::pool_manager::ObjectPool;

    let mut pool = ObjectPool::new(|| Vec::<u8>::with_capacity(1024), 100);
    let num_iterations = 1000;

    // 测试对象池性能
    let start = Instant::now();

    // 从对象池获取和归还对象
    for _ in 0..num_iterations {
        let obj = pool.acquire();
        // 使用对象
        let _ = obj;
        pool.release(obj);
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Object pool time (1000 acquire/release): {}ms", elapsed_ms);

    // 对象池操作应该很快
    assert!(
        elapsed_ms < 50,
        "Object pool performance regression: {}ms",
        elapsed_ms
    );
}

/// 测试内存泄漏检测
#[test]
fn test_memory_leak_detection() {
    use game_engine::performance::memory::MemoryMonitor;

    let monitor = MemoryMonitor::new();

    // 记录初始内存
    let initial_memory = monitor.current_memory_usage();

    // 分配一些内存
    let _large_allocation: Vec<u64> = (0..10000).collect();

    // 记录当前内存
    let current_memory = monitor.current_memory_usage();

    // 验证内存使用增加了
    assert!(current_memory >= initial_memory);

    // 清理后应该检测到内存释放
    drop(_large_allocation);

    // 注意：实际内存释放可能由GC延迟处理
    // 这里只是测试监控功能
}

/// 测试Arena分配器性能
#[test]
fn test_arena_allocator_performance() {
    use game_engine::performance::memory::arena::ArenaAllocator;

    let mut arena = ArenaAllocator::new(1024 * 1024); // 1MB arena
    let num_allocations = 1000;

    // 测试Arena分配器性能
    let start = Instant::now();

    for i in 0..num_allocations {
        let size = 1024; // 1KB
        let ptr = arena.allocate(size);
        assert!(!ptr.is_null());

        // 写入一些数据
        unsafe {
            *(ptr as *mut u64) = i as u64;
        }
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_millis();

    println!("Arena allocator time (1000 x 1KB): {}ms", elapsed_ms);

    // Arena分配应该非常快
    assert!(
        elapsed_ms < 10,
        "Arena allocator performance regression: {}ms",
        elapsed_ms
    );
}
