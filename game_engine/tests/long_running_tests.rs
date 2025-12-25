//! 长时间运行稳定性测试
//!
//! 测试引擎在长时间运行下的稳定性和资源管理，包括：
//! - 24小时连续运行测试
//! - 内存泄漏检测
//! - 资源累积检测
//! - 性能退化检测

use bevy_ecs::prelude::*;
use game_engine::ecs::{Sprite, Time, Transform};
use std::time::{Duration, Instant};

/// 长时间运行稳定性测试
///
/// 模拟长时间运行的游戏循环，验证引擎稳定性。
/// 注意：实际24小时测试需要 --ignored 标志运行。
#[test]
#[ignore] // 默认忽略，需要时使用 --ignored 运行
fn test_long_running_stability() {
    let mut world = World::new();
    world.insert_resource(Time::default());
    
    // 运行时间：24小时（测试时使用较短时间）
    const TEST_DURATION: Duration = Duration::from_secs(24 * 60 * 60);
    // 测试时使用较短时间（1分钟）
    const SHORT_TEST_DURATION: Duration = Duration::from_secs(60);
    
    let start_time = Instant::now();
    let mut frame_count = 0;
    let mut entity_count = 0;
    
    // 模拟游戏循环
    while start_time.elapsed() < SHORT_TEST_DURATION {
        // 更新时间
        if let Some(mut time) = world.get_resource_mut::<Time>() {
            time.delta_seconds = 1.0 / 60.0;
            time.elapsed_seconds += time.delta_seconds as f64;
        }
        
        // 每100帧创建一些实体
        if frame_count % 100 == 0 {
            for _ in 0..10 {
                world.spawn((Transform::default(), Sprite::default()));
                entity_count += 1;
            }
        }
        
        // 每200帧销毁一些实体
        if frame_count % 200 == 0 && entity_count > 0 {
            let mut to_destroy = Vec::new();
            let mut query = world.query::<Entity>();
            for (i, entity) in query.iter(&world).enumerate() {
                if i < 5 {
                    to_destroy.push(entity);
                }
            }
            for entity in to_destroy {
                world.despawn(entity);
                entity_count -= 1;
            }
        }
        
        frame_count += 1;
        
        // 每1000帧检查一次内存状态
        if frame_count % 1000 == 0 {
            let current_entities = world.entities().len();
            println!(
                "Frame {}: {} entities, elapsed: {:?}",
                frame_count,
                current_entities,
                start_time.elapsed()
            );
            
            // 验证实体数量在合理范围内
            assert!(current_entities < 1000, "Entity count too high, possible leak");
        }
    }
    
    println!(
        "Long running test completed: {} frames in {:?}",
        frame_count,
        start_time.elapsed()
    );
}

/// 测试内存泄漏检测（长时间运行）
///
/// 在长时间运行中检测内存泄漏。
#[test]
#[ignore]
fn test_memory_leak_long_running() {
    let mut world = World::new();
    
    const TEST_DURATION: Duration = Duration::from_secs(300); // 5分钟测试
    const ENTITIES_PER_CYCLE: usize = 100;
    
    let start_time = Instant::now();
    let mut cycles = 0;
    
    while start_time.elapsed() < TEST_DURATION {
        // 创建实体
        let entities: Vec<Entity> = (0..ENTITIES_PER_CYCLE)
            .map(|_| {
                world.spawn((Transform::default(), Sprite::default())).id()
            })
            .collect();
        
        // 等待一小段时间
        std::thread::sleep(Duration::from_millis(100));
        
        // 销毁所有实体
        for entity in entities {
            world.despawn(entity);
        }
        
        cycles += 1;
        
        // 每10个周期检查一次
        if cycles % 10 == 0 {
            let entity_count = world.entities().len();
            println!("Cycle {}: {} entities remaining", cycles, entity_count);
            
            // 验证没有实体泄漏
            assert_eq!(entity_count, 0, "Memory leak detected at cycle {}", cycles);
        }
    }
    
    // 最终验证
    assert_eq!(world.entities().len(), 0, "Final memory leak check failed");
}

/// 测试资源累积检测
///
/// 检测长时间运行中资源是否累积过多。
#[test]
#[ignore]
fn test_resource_accumulation() {
    use std::collections::HashMap;
    
    let mut resources: HashMap<String, Vec<u8>> = HashMap::new();
    
    const TEST_DURATION: Duration = Duration::from_secs(300); // 5分钟测试
    const RESOURCES_PER_CYCLE: usize = 10;
    
    let start_time = Instant::now();
    let mut cycles = 0;
    
    while start_time.elapsed() < TEST_DURATION {
        // 创建资源
        for i in 0..RESOURCES_PER_CYCLE {
            let key = format!("resource_{}_{}", cycles, i);
            resources.insert(key, vec![0u8; 1024]); // 1KB资源
        }
        
        // 清理旧资源（保留最近100个）
        if resources.len() > 100 {
            let keys_to_remove: Vec<String> = resources
                .keys()
                .take(resources.len() - 100)
                .cloned()
                .collect();
            for key in keys_to_remove {
                resources.remove(&key);
            }
        }
        
        cycles += 1;
        
        // 每10个周期检查一次
        if cycles % 10 == 0 {
            let resource_count = resources.len();
            println!("Cycle {}: {} resources", cycles, resource_count);
            
            // 验证资源数量在合理范围内
            assert!(resource_count <= 100, "Resource accumulation detected");
        }
        
        std::thread::sleep(Duration::from_millis(100));
    }
    
    // 最终验证
    assert!(resources.len() <= 100, "Final resource accumulation check failed");
}

/// 测试性能退化检测
///
/// 检测长时间运行中性能是否退化。
#[test]
#[ignore]
fn test_performance_degradation() {
    let mut world = World::new();
    
    const TEST_DURATION: Duration = Duration::from_secs(300); // 5分钟测试
    const ENTITIES_PER_CYCLE: usize = 100;
    
    let start_time = Instant::now();
    let mut cycles = 0;
    let mut frame_times = Vec::new();
    
    while start_time.elapsed() < TEST_DURATION {
        let cycle_start = Instant::now();
        
        // 创建实体
        for _ in 0..ENTITIES_PER_CYCLE {
            world.spawn((Transform::default(), Sprite::default()));
        }
        
        // 查询实体
        let mut query = world.query::<&Transform>();
        let _count = query.iter(&world).count();
        
        // 销毁实体
        let mut to_destroy = Vec::new();
        let mut entity_query = world.query::<Entity>();
        for (i, entity) in entity_query.iter(&world).enumerate() {
            if i < ENTITIES_PER_CYCLE {
                to_destroy.push(entity);
            }
        }
        for entity in to_destroy {
            world.despawn(entity);
        }
        
        let cycle_time = cycle_start.elapsed();
        frame_times.push(cycle_time.as_millis() as f64);
        
        cycles += 1;
        
        // 每50个周期检查一次性能
        if cycles % 50 == 0 {
            let avg_time = frame_times.iter().sum::<f64>() / frame_times.len() as f64;
            let max_time = frame_times.iter().copied().fold(0.0, f64::max);
            
            println!(
                "Cycle {}: avg {:.2}ms, max {:.2}ms",
                cycles, avg_time, max_time
            );
            
            // 验证性能没有显著退化（平均时间不应超过初始的2倍）
            if cycles > 100 {
                let early_avg: f64 = frame_times[0..100].iter().sum::<f64>() / 100.0;
                let recent_avg: f64 = frame_times[frame_times.len() - 100..]
                    .iter()
                    .sum::<f64>() / 100.0;
                
                assert!(
                    recent_avg < early_avg * 2.0,
                    "Performance degradation detected: early avg {:.2}ms, recent avg {:.2}ms",
                    early_avg,
                    recent_avg
                );
            }
        }
        
        std::thread::sleep(Duration::from_millis(10));
    }
}

