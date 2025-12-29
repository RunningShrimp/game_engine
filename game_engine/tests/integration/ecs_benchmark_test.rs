//! ECS系统性能基准测试
//!
//! 测试ECS系统的性能指标，包括：
//! - 实体创建性能
//! - 组件添加性能
//! - 查询性能
//! - 系统执行性能

use bevy_ecs::prelude::*;
use std::time::Instant;

#[derive(Component)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct Rotation {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_entity_creation() {
    let iterations = 10000;
    let mut world = World::new();
    let start = Instant::now();
    
    for i in 0..iterations {
        world.spawn(Position {
            x: i as f32,
            y: 0.0,
            z: 0.0,
        });
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("实体创建性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 10000.0, "实体创建应该小于10000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_component_addition() {
    let iterations = 10000;
    let mut world = World::new();
    let mut entities = Vec::with_capacity(iterations);
    
    for i in 0..iterations {
        entities.push(world.spawn(Position {
            x: i as f32,
            y: 0.0,
            z: 0.0,
        }).id());
    }
    
    let start = Instant::now();
    
    for entity in entities {
        world.entity_mut(entity).insert(Velocity {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        });
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("组件添加性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 5000.0, "组件添加应该小于5000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_query_single_component() {
    let iterations = 10000;
    let mut world = World::new();
    
    for i in 0..iterations {
        world.spawn(Position {
            x: i as f32,
            y: 0.0,
            z: 0.0,
        });
    }
    
    let start = Instant::now();
    
    for _ in 0..100 {
        let mut query = world.query::<&Position>();
        let count = query.iter(&world).count();
        assert_eq!(count, iterations);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / (iterations * 100) as f64;
    
    println!("单组件查询性能: {} 次迭代, 100次查询", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/实体", avg_time);
    
    assert!(avg_time < 100.0, "单组件查询应该小于100ns/实体");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_query_multiple_components() {
    let iterations = 10000;
    let mut world = World::new();
    
    for i in 0..iterations {
        world.spawn((
            Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            Velocity {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ));
    }
    
    let start = Instant::now();
    
    for _ in 0..100 {
        let mut query = world.query::<(&Position, &Velocity, &Rotation)>();
        let count = query.iter(&world).count();
        assert_eq!(count, iterations);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / (iterations * 100) as f64;
    
    println!("多组件查询性能: {} 次迭代, 100次查询", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/实体", avg_time);
    
    assert!(avg_time < 200.0, "多组件查询应该小于200ns/实体");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_query_mutation() {
    let iterations = 10000;
    let mut world = World::new();
    
    for i in 0..iterations {
        world.spawn((
            Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            Velocity {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        ));
    }
    
    let start = Instant::now();
    
    for _ in 0..100 {
        let mut query = world.query::<(&mut Position, &Velocity)>();
        for (pos, vel) in query.iter_mut(&mut world) {
            pos.x += vel.x;
            pos.y += vel.y;
            pos.z += vel.z;
        }
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / (iterations * 100) as f64;
    
    println!("查询变异性能: {} 次迭代, 100次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/实体", avg_time);
    
    assert!(avg_time < 300.0, "查询变异应该小于300ns/实体");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_entity_despawn() {
    let iterations = 10000;
    let mut world = World::new();
    let mut entities = Vec::with_capacity(iterations);
    
    for i in 0..iterations {
        entities.push(world.spawn(Position {
            x: i as f32,
            y: 0.0,
            z: 0.0,
        }).id());
    }
    
    let start = Instant::now();
    
    for entity in entities {
        world.despawn(entity);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("实体销毁性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 5000.0, "实体销毁应该小于5000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_entity_with_many_components() {
    let iterations = 1000;
    let mut world = World::new();
    let start = Instant::now();
    
    for i in 0..iterations {
        world.spawn((
            Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            Velocity {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Health {
                current: 100.0,
                max: 100.0,
            },
        ));
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("多组件实体创建性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 50000.0, "多组件实体创建应该小于50000ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_resource_access() {
    let iterations = 100000;
    let mut world = World::new();
    world.insert_resource(Counter { count: 0 });
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        let counter = world.resource::<Counter>();
        let _ = counter.count;
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("资源访问性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 100.0, "资源访问应该小于100ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_resource_mutation() {
    let iterations = 100000;
    let mut world = World::new();
    world.insert_resource(Counter { count: 0 });
    
    let start = Instant::now();
    
    for _ in 0..iterations {
        let mut counter = world.resource_mut::<Counter>();
        counter.count += 1;
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("资源变异性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 200.0, "资源变异应该小于200ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_query_filtering() {
    let iterations = 10000;
    let mut world = World::new();
    
    for i in 0..iterations {
        if i % 2 == 0 {
            world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Velocity {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
            ));
        } else {
            world.spawn(Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            });
        }
    }
    
    let start = Instant::now();
    
    for _ in 0..100 {
        let mut query = world.query::<(&Position, &Velocity)>();
        let count = query.iter(&world).count();
        assert_eq!(count, iterations / 2);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / (iterations * 100) as f64;
    
    println!("查询过滤性能: {} 次迭代, 100次查询", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/实体", avg_time);
    
    assert!(avg_time < 150.0, "查询过滤应该小于150ns/实体");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_entity_lookup() {
    let iterations = 10000;
    let mut world = World::new();
    let mut entities = Vec::with_capacity(iterations);
    
    for i in 0..iterations {
        entities.push(world.spawn(Position {
            x: i as f32,
            y: 0.0,
            z: 0.0,
        }).id());
    }
    
    let start = Instant::now();
    
    for entity in &entities {
        let _ = world.get::<Position>(*entity);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / iterations as f64;
    
    println!("实体查找性能: {} 次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns", avg_time);
    
    assert!(avg_time < 500.0, "实体查找应该小于500ns");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_batch_entity_creation() {
    let batch_size = 1000;
    let batches = 10;
    let mut world = World::new();
    let start = Instant::now();
    
    for batch in 0..batches {
        for i in 0..batch_size {
            world.spawn(Position {
                x: (batch * batch_size + i) as f32,
                y: 0.0,
                z: 0.0,
            });
        }
    }
    
    let duration = start.elapsed();
    let total_entities = batch_size * batches;
    let avg_time = duration.as_nanos() as f64 / total_entities as f64;
    
    println!("批量实体创建性能: {} 批次, {} 实体/批次", batches, batch_size);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/实体", avg_time);
    
    assert!(avg_time < 10000.0, "批量实体创建应该小于10000ns/实体");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_system_execution() {
    let iterations = 10000;
    let mut world = World::new();
    
    for i in 0..iterations {
        world.spawn((
            Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            Velocity {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
        ));
    }
    
    let start = Instant::now();
    
    for _ in 0..100 {
        let mut query = world.query::<(&mut Position, &Velocity)>();
        for (pos, vel) in query.iter_mut(&mut world) {
            pos.x += vel.x * 0.016;
            pos.y += vel.y * 0.016;
            pos.z += vel.z * 0.016;
        }
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / (iterations * 100) as f64;
    
    println!("系统执行性能: {} 次迭代, 100次执行", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/实体", avg_time);
    
    assert!(avg_time < 500.0, "系统执行应该小于500ns/实体");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_archetype_iteration() {
    let iterations = 10000;
    let mut world = World::new();
    
    for i in 0..iterations {
        world.spawn((
            Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            },
            Velocity {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            Rotation {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
        ));
    }
    
    let start = Instant::now();
    
    for _ in 0..100 {
        let mut query = world.query::<(&Position, &Velocity, &Rotation)>();
        let mut count = 0;
        for _ in query.iter(&world) {
            count += 1;
        }
        assert_eq!(count, iterations);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / (iterations * 100) as f64;
    
    println!("原型迭代性能: {} 次迭代, 100次迭代", iterations);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/实体", avg_time);
    
    assert!(avg_time < 200.0, "原型迭代应该小于200ns/实体");
}

#[test]
#[ignore]  // TODO: Fix compilation errors
fn benchmark_sparse_query() {
    let iterations = 10000;
    let sparse_count = 100;
    let mut world = World::new();
    
    for i in 0..iterations {
        if i < sparse_count {
            world.spawn((
                Position {
                    x: i as f32,
                    y: 0.0,
                    z: 0.0,
                },
                Velocity {
                    x: 1.0,
                    y: 0.0,
                    z: 0.0,
                },
                Health {
                    current: 100.0,
                    max: 100.0,
                },
            ));
        } else {
            world.spawn(Position {
                x: i as f32,
                y: 0.0,
                z: 0.0,
            });
        }
    }
    
    let start = Instant::now();
    
    for _ in 0..100 {
        let mut query = world.query::<(&Position, &Velocity, &Health)>();
        let count = query.iter(&world).count();
        assert_eq!(count, sparse_count);
    }
    
    let duration = start.elapsed();
    let avg_time = duration.as_nanos() as f64 / (iterations * 100) as f64;
    
    println!("稀疏查询性能: {} 次迭代, {} 稀疏实体, 100次查询", iterations, sparse_count);
    println!("总耗时: {:?}", duration);
    println!("平均耗时: {:.2} ns/实体", avg_time);
    
    assert!(avg_time < 200.0, "稀疏查询应该小于200ns/实体");
}

#[derive(Resource)]
struct Counter {
    count: u64,
}
