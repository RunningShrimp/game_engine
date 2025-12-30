//! # 优化性能基准测试
//!
//! 本模块提供了各种优化的性能基准测试，用于验证性能提升效果。

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use game_engine::async_optimization::*;
use game_engine::core::scheduler::{Task, TaskScheduler, TaskPriority};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

// ============================================================================
// 异步vs同步基准测试
// ============================================================================

fn benchmark_calculate_physics(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_calculation");
    
    group.bench_function("sync_physics", |b| {
        b.iter(|| {
            black_box(calculate_physics(
                black_box((0.0, 0.0, 0.0)),
                black_box((1.0, 2.0, 3.0)),
                black_box(0.016),
            ))
        })
    });

    group.finish();
}

fn benchmark_vector_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("vector_operations");
    
    group.bench_function("vector_add", |b| {
        b.iter(|| {
            black_box(vector_add(
                black_box([1.0, 2.0, 3.0]),
                black_box([4.0, 5.0, 6.0]),
            ))
        })
    });

    group.bench_function("vector_dot", |b| {
        b.iter(|| {
            black_box(vector_dot(
                black_box([1.0, 2.0, 3.0]),
                black_box([4.0, 5.0, 6.0]),
            ))
        })
    });

    group.bench_function("vector_normalize", |b| {
        b.iter(|| {
            black_box(vector_normalize(
                black_box([3.0, 0.0, 0.0]),
            ))
        })
    });

    group.finish();
}

fn benchmark_distance_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("distance_calculation");
    
    group.bench_function("calculate_distance", |b| {
        b.iter(|| {
            black_box(calculate_distance(
                black_box(0.0), black_box(0.0),
                black_box(3.0), black_box(4.0),
            ))
        })
    });

    group.finish();
}

fn benchmark_entity_queries(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_queries");
    
    let entities: Vec<u32> = (0..1000).collect();
    
    group.bench_function("get_entity_count", |b| {
        b.iter(|| {
            black_box(get_entity_count(black_box(&entities)))
        })
    });

    let entity_map: std::collections::HashMap<u32, [f32; 3]> = 
        (0..1000).map(|i| (i, [i as f32; 3])).collect();
    
    group.bench_function("query_entity_state", |b| {
        b.iter(|| {
            black_box(query_entity_state(black_box(&entity_map), black_box(500)))
        })
    });

    group.finish();
}

// ============================================================================
// 批量操作基准测试
// ============================================================================

fn benchmark_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");
    
    for size in [100, 1000, 10000].iter() {
        let mut entities = vec![[0.0f32; 3]; *size];
        let offset = [1.0, 2.0, 3.0];
        
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &_size| {
            b.iter(|| {
                let mut entities_clone = entities.clone();
                black_box(batch_process_entities_rayon(
                    black_box(&mut entities_clone),
                    black_box(offset),
                ))
            })
        });
    }
    
    group.finish();
}

// ============================================================================
// 任务调度器基准测试
// ============================================================================

fn benchmark_task_scheduler(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_scheduler");
    
    // 单任务调度
    group.bench_function("schedule_single_task", |b| {
        let scheduler = TaskScheduler::new(4);
        b.iter(|| {
            let scheduler = &scheduler;
            scheduler.schedule(Task::new(
                "test_task",
                Box::new(|| {}),
                TaskPriority::Medium,
            ))
        })
    });

    // 批量任务调度
    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        
        group.bench_with_input(BenchmarkId::new("batch_schedule", size), size, |b, &size| {
            b.iter(|| {
                let scheduler = TaskScheduler::new(4);
                let tasks: Vec<_> = (0..size)
                    .map(|i| {
                        Task::new(
                            format!("task_{}", i),
                            Box::new(move || {}),
                            TaskPriority::Medium,
                        )
                    })
                    .collect();
                
                black_box(scheduler.schedule_batch(tasks));
            })
        });
    }

    group.finish();
}

fn benchmark_task_execution(c: &mut Criterion) {
    let mut group = c.benchmark_group("task_execution");
    
    // 不同优先级的任务执行
    for priority in [TaskPriority::Low, TaskPriority::Medium, TaskPriority::High].iter() {
        group.bench_with_input(
            BenchmarkId::new("execute_task", format!("{:?}", priority)),
            priority,
            |b, &priority| {
                b.iter(|| {
                    let scheduler = TaskScheduler::new(2);
                    let counter = Arc::new(AtomicUsize::new(0));
                    let counter_clone = counter.clone();
                    
                    scheduler.schedule(Task::new(
                        "increment",
                        Box::new(move || {
                            counter_clone.fetch_add(1, Ordering::SeqCst);
                        }),
                        priority,
                    ));
                    
                    scheduler.wait_for_completion();
                    black_box(counter.load(Ordering::SeqCst));
                })
            }
        );
    }

    group.finish();
}

// ============================================================================
// 内存操作基准测试
// ============================================================================

fn benchmark_memory_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_operations");
    
    // 小数据克隆
    group.bench_function("clone_small_data", |b| {
        let data: Vec<u8> = vec![1, 2, 3, 4, 5];
        b.iter(|| {
            black_box(data.clone())
        })
    });

    // 中等数据克隆
    group.bench_function("clone_medium_data", |b| {
        let data: Vec<u8> = (0..1024).map(|i| i as u8).collect();
        b.iter(|| {
            black_box(data.clone())
        })
    });

    // 大数据克隆
    group.bench_function("clone_large_data", |b| {
        let data: Vec<u8> = (0..1024 * 1024).map(|i| i as u8).collect();
        b.iter(|| {
            black_box(data.clone())
        })
    });

    group.finish();
}

// ============================================================================
// 并发性能基准测试
// ============================================================================

fn benchmark_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    
    // 单线程 vs 多线程批量操作
    for size in [1000, 10000].iter() {
        let entities = vec![[0.0f32; 3]; *size];
        let offset = [1.0, 2.0, 3.0];
        
        group.bench_with_input(BenchmarkId::new("sequential", size), size, |b, &_size| {
            b.iter(|| {
                let mut entities_clone = entities.clone();
                for entity in entities_clone.iter_mut() {
                    entity[0] += offset[0];
                    entity[1] += offset[1];
                    entity[2] += offset[2];
                }
                black_box(&entities_clone)
            })
        });

        #[cfg(feature = "rayon")]
        group.bench_with_input(BenchmarkId::new("parallel_rayon", size), size, |b, &_size| {
            b.iter(|| {
                let mut entities_clone = entities.clone();
                batch_process_entities_rayon(&mut entities_clone, offset);
                black_box(&entities_clone)
            })
        });
    }

    group.finish();
}

// ============================================================================
// 锁性能基准测试
// ============================================================================

#[cfg(feature = "parking_lot")]
fn benchmark_lock_performance(c: &mut Criterion) {
    use parking_lot::Mutex as ParkingMutex;
    use std::sync::Mutex as StdMutex;
    
    let mut group = c.benchmark_group("lock_performance");
    
    // std::sync::Mutex vs parking_lot::Mutex
    group.bench_function("std_mutex_lock", |b| {
        let mutex = StdMutex::new(42i32);
        b.iter(|| {
            let guard = mutex.lock().unwrap();
            black_box(*guard);
        })
    });

    group.bench_function("parking_lot_mutex_lock", |b| {
        let mutex = ParkingMutex::new(42i32);
        b.iter(|| {
            let guard = mutex.lock();
            black_box(*guard);
        })
    });

    // 读写锁
    group.bench_function("std_rwlock_read", |b| {
        let rwlock = StdMutex::new(42i32);
        b.iter(|| {
            let guard = rwlock.lock().unwrap();
            black_box(*guard);
        })
    });

    group.bench_function("parking_lot_rwlock_read", |b| {
        use parking_lot::RwLock as ParkingRwLock;
        let rwlock = ParkingRwLock::new(42i32);
        b.iter(|| {
            let guard = rwlock.read();
            black_box(*guard);
        })
    });

    group.finish();
}

// ============================================================================
// 综合基准测试
// ============================================================================

fn benchmark_comprehensive_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive_workload");
    
    group.measurement_time(Duration::from_secs(10));
    
    group.bench_function("mixed_workload", |b| {
        b.iter(|| {
            // 物理计算
            let _pos = calculate_physics((0.0, 0.0, 0.0), (1.0, 2.0, 3.0), 0.016);
            
            // 向量运算
            let _v = vector_add([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]);
            
            // 距离计算
            let _dist = calculate_distance(0.0, 0.0, 3.0, 4.0);
            
            // 实体查询
            let entities = vec![1u32, 2, 3, 4, 5];
            let _count = get_entity_count(&entities);
            
            black_box((_pos, _v, _dist, _count))
        })
    });

    group.finish();
}

// ============================================================================
// 注册所有基准测试
// ============================================================================

criterion_group!(
    name = async_benches;
    config = Criterion::default().sample_size(100);
    targets = 
        benchmark_calculate_physics,
        benchmark_vector_operations,
        benchmark_distance_calculation,
        benchmark_entity_queries
);

criterion_group!(
    name = batch_benches;
    config = Criterion::default().sample_size(50);
    targets = benchmark_batch_operations
);

criterion_group!(
    name = scheduler_benches;
    config = Criterion::default().sample_size(50);
    targets = 
        benchmark_task_scheduler,
        benchmark_task_execution
);

criterion_group!(
    name = memory_benches;
    config = Criterion::default().sample_size(100);
    targets = benchmark_memory_operations
);

criterion_group!(
    name = concurrent_benches;
    config = Criterion::default().sample_size(50);
    targets = benchmark_concurrent_operations
);

#[cfg(feature = "parking_lot")]
criterion_group!(
    name = lock_benches;
    config = Criterion::default().sample_size(1000);
    targets = benchmark_lock_performance
);

criterion_group!(
    name = comprehensive_benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = benchmark_comprehensive_workload
);

criterion_main!(
    async_benches,
    batch_benches,
    scheduler_benches,
    memory_benches,
    concurrent_benches,
    #[cfg(feature = "parking_lot")]
    lock_benches,
    comprehensive_benches
);
