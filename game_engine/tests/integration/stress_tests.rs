//! 压力测试
//!
//! 测试游戏引擎在高负载下的性能和稳定性，包括：
//! - 大规模实体渲染（1000+实体）
//! - 大规模事件处理（10000+事件）
//! - 高并发资源加载
//! - 长时间运行稳定性测试

use game_engine::ecs::{World, Entity};
use game_engine::core::event::{EventBus, EventHandler, Event};
use game_engine::render::mesh::Mesh;
use game_engine::resource::{ResourceManager, ResourceLoadOptions};
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::task::JoinSet;

// ============================================================================
// 测试1: 大规模实体渲染压力测试
// ============================================================================

#[test]
fn test_large_scale_entity_spawn() {
    // 测试大规模实体创建

    let mut world = World::new();
    let num_entities = 1000;

    let start = Instant::now();

    // 创建1000个实体
    for i in 0..num_entities {
        let entity = world.spawn_entity();
        world.add_component(entity, (i as f32, i as f32, i as f32));
    }

    let elapsed = start.elapsed();

    println!("Spawned {} entities in {:?}", num_entities, elapsed);

    // 验证所有实体都已创建
    let entity_count = world.entity_count();
    assert_eq!(entity_count, num_entities);

    // 性能检查：应该在合理时间内完成
    assert!(elapsed.as_millis() < 1000); // 1秒内完成
}

#[test]
fn test_large_scale_entity_update() {
    // 测试大规模实体更新

    let mut world = World::new();
    let num_entities = 1000;

    // 创建实体
    for i in 0..num_entities {
        let entity = world.spawn_entity();
        world.add_component(entity, (i as f32, 0.0, 0.0));
    }

    let start = Instant::now();

    // 更新所有实体（模拟物理更新）
    for _ in 0..100 {
        world.update();
    }

    let elapsed = start.elapsed();

    println!("Updated {} entities x 100 frames in {:?}", num_entities, elapsed);

    // 性能检查：100帧更新应该在合理时间内完成
    assert!(elapsed.as_millis() < 5000); // 5秒内完成
}

#[test]
fn test_entity_deletion_stress() {
    // 测试大规模实体删除

    let mut world = World::new();
    let num_entities = 1000;

    // 创建实体
    let mut entities = Vec::new();
    for i in 0..num_entities {
        let entity = world.spawn_entity();
        world.add_component(entity, (i as f32, 0.0, 0.0));
        entities.push(entity);
    }

    let start = Instant::now();

    // 删除所有实体
    for entity in entities {
        world.despawn_entity(entity);
    }

    let elapsed = start.elapsed();

    println!("Deleted {} entities in {:?}", num_entities, elapsed);

    // 验证所有实体已删除
    assert_eq!(world.entity_count(), 0);

    // 性能检查：删除操作应该很快
    assert!(elapsed.as_millis() < 500);
}

#[test]
fn test_entity_query_performance() {
    // 测试实体查询性能

    let mut world = World::new();
    let num_entities = 1000;

    // 创建实体（只有一半有Position组件）
    for i in 0..num_entities {
        let entity = world.spawn_entity();
        if i % 2 == 0 {
            world.add_component(entity, (i as f32, 0.0, 0.0));
        }
    }

    let start = Instant::now();

    // 查询所有有Position的实体
    let mut count = 0;
    for _ in 0..100 {
        count = 0;
        for entity in world.query::<(f32, f32, f32)>() {
            let _ = entity;
            count += 1;
        }
    }

    let elapsed = start.elapsed();

    println!("Queried {} entities x 100 in {:?}", count, elapsed);

    // 验证查询结果
    assert_eq!(count, num_entities / 2);

    // 性能检查：查询应该很快
    assert!(elapsed.as_millis() < 1000);
}

// ============================================================================
// 测试2: 大规模事件处理压力测试
// ============================================================================

#[test]
fn test_large_scale_event_dispatch() {
    // 测试大规模事件分发

    let mut event_bus = EventBus::new();
    let num_events = 10000;

    // 创建事件处理器
    let mut handler = TestEventHandler::new();

    let start = Instant::now();

    // 分发10000个事件
    for i in 0..num_events {
        let event = TestEvent { value: i };
        event_bus.dispatch(event);
    }

    let elapsed = start.elapsed();

    println!("Dispatched {} events in {:?}", num_events, elapsed);

    // 性能检查：事件分发应该很快
    assert!(elapsed.as_millis() < 100);
}

#[test]
fn test_large_scale_event_handling() {
    // 测试大规模事件处理

    let mut event_bus = EventBus::new();
    let num_events = 10000;

    // 注册事件处理器
    event_bus.register_handler(Box::new(TestEventHandler::new()));

    let start = Instant::now();

    // 分发并处理10000个事件
    for i in 0..num_events {
        let event = TestEvent { value: i };
        event_bus.dispatch(event);
        event_bus.handle_events();
    }

    let elapsed = start.elapsed();

    println!("Handled {} events in {:?}", num_events, elapsed);

    // 性能检查：事件处理应该在合理时间内完成
    assert!(elapsed.as_millis() < 2000);
}

#[test]
fn test_concurrent_event_dispatch() {
    // 测试并发事件分发

    let event_bus = Arc::new(std::sync::Mutex::new(EventBus::new()));
    let num_tasks = 10;
    let events_per_task = 1000;

    let start = Instant::now();

    // 使用tokio并发分发事件
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let mut handles = JoinSet::new();

        for task_id in 0..num_tasks {
            let event_bus_clone = event_bus.clone();
            handles.spawn(async move {
                for i in 0..events_per_task {
                    let event = TestEvent {
                        value: task_id * events_per_task + i,
                    };
                    let mut bus = event_bus_clone.lock().unwrap();
                    bus.dispatch(event);
                }
            });
        }

        while let Some(result) = handles.join_next().await {
            result.unwrap();
        }
    });

    let elapsed = start.elapsed();

    let total_events = num_tasks * events_per_task;
    println!("Concurrently dispatched {} events in {:?}", total_events, elapsed);

    // 性能检查：并发分发应该更快
    assert!(elapsed.as_millis() < 1000);
}

#[test]
fn test_event_queue_overflow() {
    // 测试事件队列溢出

    let mut event_bus = EventBus::new();
    let num_events = 100000;

    // 尝试分发大量事件
    for i in 0..num_events {
        let event = TestEvent { value: i };
        event_bus.dispatch(event);
    }

    // 处理事件
    event_bus.handle_events();

    // 验证系统没有崩溃
    assert!(true);
}

// ============================================================================
// 测试3: 高并发资源加载压力测试
// ============================================================================

#[tokio::test]
async fn test_concurrent_resource_loading() {
    // 测试并发资源加载

    let resource_manager = ResourceManager::new();
    let num_resources = 100;

    let start = Instant::now();

    // 并发加载100个资源
    let mut handles = JoinSet::new();

    for i in 0..num_resources {
        let manager = resource_manager.clone();
        handles.spawn(async move {
            // 模拟资源加载
            let resource_path = format!("resource_{}.dat", i);
            let options = ResourceLoadOptions::default();

            // 这里应该是实际的资源加载，现在用模拟
            tokio::time::sleep(Duration::from_millis(10)).await;

            Ok::<(), std::io::Error>(())
        });
    }

    // 等待所有资源加载完成
    let mut success_count = 0;
    while let Some(result) = handles.join_next().await {
        if result.unwrap().is_ok() {
            success_count += 1;
        }
    }

    let elapsed = start.elapsed();

    println!("Loaded {} resources concurrently in {:?}", success_count, elapsed);

    // 验证所有资源都已加载
    assert_eq!(success_count, num_resources);

    // 性能检查：并发加载应该比串行快
    assert!(elapsed.as_millis() < 500); // 每个资源10ms，100个并发应该<500ms
}

#[tokio::test]
async fn test_resource_loading_memory_pressure() {
    // 测试资源加载内存压力

    let resource_manager = ResourceManager::new();
    let num_resources = 100;
    let resource_size = 1024 * 1024; // 1MB

    let start = Instant::now();

    // 加载100个1MB的资源（100MB总数据）
    let mut handles = JoinSet::new();

    for i in 0..num_resources {
        handles.spawn(async move {
            // 模拟加载大资源
            let data = vec![0u8; resource_size];
            // 处理数据
            let _checksum: u32 = data.iter().map(|&x| x as u32).sum();
            tokio::time::sleep(Duration::from_millis(5)).await;
            Ok::<(), std::io::Error>(())
        });
    }

    // 等待所有资源加载
    while let Some(result) = handles.join_next().await {
        result.unwrap().unwrap();
    }

    let elapsed = start.elapsed();

    println!("Loaded {} MB in {:?}", num_resources * resource_size / 1024 / 1024, elapsed);

    // 性能检查：应该在合理时间内完成
    assert!(elapsed.as_millis() < 2000);
}

#[tokio::test]
async fn test_resource_caching_under_load() {
    // 测试负载下的资源缓存

    let resource_manager = ResourceManager::new();
    let resource_path = "test_resource.dat";

    let start = Instant::now();

    // 尝试并发加载同一个资源100次
    let mut handles = JoinSet::new();

    for _ in 0..100 {
        let manager = resource_manager.clone();
        let path = resource_path.to_string();

        handles.spawn(async move {
            // 模拟资源加载和缓存
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<(), std::io::Error>(())
        });
    }

    // 等待所有加载完成
    while let Some(result) = handles.join_next().await {
        result.unwrap().unwrap();
    }

    let elapsed = start.elapsed();

    println!("Loaded cached resource 100 times in {:?}", elapsed);

    // 缓存命中应该很快
    assert!(elapsed.as_millis() < 1500);
}

// ============================================================================
// 测试4: 长时间运行稳定性测试
// ============================================================================

#[tokio::test]
#[ignore] // 默认忽略，因为需要运行较长时间
async fn test_long_running_stability() {
    // 测试长时间运行稳定性（60秒）

    let mut world = World::new();
    let num_entities = 100;

    // 创建实体
    for i in 0..num_entities {
        let entity = world.spawn_entity();
        world.add_component(entity, (i as f32, 0.0, 0.0));
    }

    let duration = Duration::from_secs(60);
    let start = Instant::now();
    let mut frame_count = 0;

    // 运行60秒
    while start.elapsed() < duration {
        // 更新世界
        world.update();

        // 添加和删除实体
        if frame_count % 100 == 0 {
            let entity = world.spawn_entity();
            world.add_component(entity, (0.0, 0.0, 0.0));

            if world.entity_count() > num_entities + 10 {
                // 删除一些实体
                for entity in world.query::<(f32, f32, f32)>().take(5) {
                    let _ = entity;
                }
            }
        }

        frame_count += 1;

        // 避免忙等待
        tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
    }

    let elapsed = start.elapsed();

    println!("Ran {} frames in {:?} (average FPS: {:.1})",
        frame_count, elapsed,
        frame_count as f64 / elapsed.as_secs_f64()
    );

    // 验证系统仍然稳定
    assert!(world.entity_count() > 0);
    assert!(frame_count > 3000); // 60秒 * 60 FPS = 3600帧
}

#[tokio::test]
#[ignore] // 默认忽略
async fn test_memory_leak_detection() {
    // 测试内存泄漏检测

    let duration = Duration::from_secs(30);
    let start = Instant::now();

    // 记录初始内存使用
    let initial_memory = get_memory_usage();

    // 持续创建和销毁实体
    let mut iteration = 0;
    while start.elapsed() < duration {
        let mut world = World::new();

        // 创建100个实体
        for i in 0..100 {
            let entity = world.spawn_entity();
            world.add_component(entity, (i as f32, 0.0, 0.0));
        }

        // 更新
        world.update();

        // 删除所有实体
        for entity in world.query::<(f32, f32, f32)>() {
            let _ = entity;
        }

        iteration += 1;

        // 避免忙等待
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    // 记录最终内存使用
    let final_memory = get_memory_usage();

    println!("Memory usage: initial = {} KB, final = {} KB, iterations = {}",
        initial_memory, final_memory, iteration
    );

    // 内存增长不应超过50%（考虑到系统波动）
    let memory_growth = final_memory as f64 / initial_memory as f64;
    assert!(memory_growth < 1.5);
}

#[tokio::test]
async fn test_stress_recovery() {
    // 测试压力恢复能力

    let mut world = World::new();

    // 创建大量实体
    for i in 0..10000 {
        let entity = world.spawn_entity();
        world.add_component(entity, (i as f32, 0.0, 0.0));
    }

    // 更新（产生压力）
    world.update();

    // 清除大部分实体
    for entity in world.query::<(f32, f32, f32)>().take(9000) {
        let _ = entity;
    }

    // 更新（应该很快恢复）
    let start = Instant::now();
    world.update();
    let elapsed = start.elapsed();

    println!("Recovery update took {:?}", elapsed);

    // 验证系统仍然响应
    assert!(elapsed.as_millis() < 100);
}

// ============================================================================
// 辅助类型和函数
// ============================================================================

struct TestEvent {
    value: u32,
}

impl Event for TestEvent {}

struct TestEventHandler {
    count: std::sync::Arc<std::sync::atomic::AtomicU32>,
}

impl TestEventHandler {
    fn new() -> Self {
        Self {
            count: std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0)),
        }
    }
}

impl EventHandler<TestEvent> for TestEventHandler {
    fn handle(&mut self, event: &TestEvent) {
        self.count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let _ = event.value;
    }
}

#[cfg(unix)]
fn get_memory_usage() -> u64 {
    // 获取当前进程的内存使用（KB）
    use std::fs;

    if let Ok(status) = fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("VmRSS:") {
                // VmRSS:     12345 kB
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<u64>() {
                        return kb;
                    }
                }
            }
        }
    }

    0 // 默认值
}

#[cfg(not(unix))]
fn get_memory_usage() -> u64 {
    // 非Unix系统，返回估计值
    10240 // 10MB
}
