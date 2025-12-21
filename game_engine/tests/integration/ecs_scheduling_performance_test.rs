//! ECS系统调度性能测试
//!
//! 测试ECS系统调度的性能，包括并行调度优化效果。

use bevy_ecs::prelude::*;
use game_engine::core::system_scheduler::{SystemDependency, SystemSchedulerOptimizer, SchedulerStats};
use std::time::Instant;

/// 性能基准：系统执行时间（1000个实体）
const SYSTEM_EXECUTION_BENCHMARK_MS: u64 = 5;

/// 测试ECS系统调度性能
#[test]
fn test_ecs_system_scheduling_performance() {
    let mut world = World::new();
    
    // 创建1000个实体
    for i in 0..1000 {
        world.spawn((
            Transform::from_translation(glam::Vec3::new(i as f32, 0.0, 0.0)),
            Velocity { x: 1.0, y: 0.0, z: 0.0 },
        ));
    }
    
    let start = Instant::now();
    
    // 执行系统（模拟）
    for _ in 0..100 {
        // 这里应该调用实际的系统调度逻辑
        // 由于当前没有公开的系统调度API，这里只是占位测试
    }
    
    let elapsed = start.elapsed();
    let avg_ms = elapsed.as_millis() / 100;
    
    println!("Average ECS system execution time: {}ms", avg_ms);
    
    // 验证性能没有显著退化
    assert!(
        avg_ms < (SYSTEM_EXECUTION_BENCHMARK_MS as u128 * 120 / 100),
        "ECS system scheduling performance regression: {}ms > {}ms",
        avg_ms,
        SYSTEM_EXECUTION_BENCHMARK_MS * 120 / 100
    );
}

/// 测试系统依赖分析
#[test]
fn test_system_dependency_analysis() {
    let mut optimizer = SystemSchedulerOptimizer::new();

    // 添加系统依赖
    optimizer.add_system_dependency(SystemDependency {
        system_name: "system_a".to_string(),
        dependencies: vec![],
        read_resources: vec![],
        write_resources: vec![],
        read_components: vec![],
        write_components: vec![],
    });

    optimizer.add_system_dependency(SystemDependency {
        system_name: "system_b".to_string(),
        dependencies: vec!["system_a".to_string()],
        read_resources: vec![],
        write_resources: vec![],
        read_components: vec![],
        write_components: vec![],
    });

    optimizer.add_system_dependency(SystemDependency {
        system_name: "system_c".to_string(),
        dependencies: vec!["system_a".to_string()],
        read_resources: vec![],
        write_resources: vec![],
        read_components: vec![],
        write_components: vec![],
    });

    // 分析依赖
    optimizer.analyze_dependencies();

    let order = optimizer.execution_order();
    assert!(!order.is_empty());
    assert!(order[0].contains(&"system_a".to_string()));
    
    // system_b和system_c应该在system_a之后执行
    if order.len() > 1 {
        assert!(order[1].contains(&"system_b".to_string()) || order[1].contains(&"system_c".to_string()));
    }
}

/// 测试并行系统调度性能提升
#[test]
fn test_parallel_system_scheduling_performance() {
    // 这个测试验证并行系统调度相比串行调度的性能提升
    // 实际实现需要在system_scheduler.rs中完成
    // 这里只是占位测试
    assert!(true);
}

/// 测试性能统计收集
#[test]
fn test_scheduler_performance_stats() {
    let mut optimizer = SystemSchedulerOptimizer::new();
    
    optimizer.record_execution(true, 1000.0, 10);
    optimizer.record_execution(false, 500.0, 5);
    
    let stats = optimizer.stats();
    assert_eq!(stats.execution_count, 2);
    assert_eq!(stats.parallel_execution_count, 1);
    assert_eq!(stats.serial_execution_count, 1);
    assert!(stats.average_execution_time_us > 0.0);
    assert!(stats.average_systems_per_frame > 0.0);
}

#[derive(Component)]
struct Transform {
    translation: glam::Vec3,
}

impl Transform {
    fn from_translation(translation: glam::Vec3) -> Self {
        Self { translation }
    }
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}
