//! 压力测试 - 极限场景下的系统表现
//!
//! 测试系统在极限负载下的表现和稳定性

use game_engine::prelude::*;
use std::thread;
use std::time::{Duration, Instant};

#[cfg(test)]
mod stress_tests {
    use super::*;

    #[test]
    fn test_massive_entity_count() {
        println!("🧪 测试：大量实体处理");

        let manager = ConcurrentEntityManager::new();
        let entity_count = 100_000;

        let start = Instant::now();

        // 添加10万个实体
        for i in 0..entity_count {
            manager.add_entity(EntityData {
                id: i,
                position: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                scale: (1.0, 1.0, 1.0),
                active: true,
            });
        }

        let duration = start.elapsed();

        assert_eq!(manager.len(), entity_count);
        println!("✅ 添加 {} 个实体耗时: {:?}", entity_count, duration);
        println!("   平均: {:.2}μs/实体", duration.as_micros() as f64 / entity_count as f64);
    }

    #[test]
    fn test_extreme_concurrency() {
        println!("🧪 测试：极限并发");

        let manager = ConcurrentEntityManager::new();
        let num_threads = 100;
        let operations_per_thread = 1000;

        // 预先添加实体
        for i in 0..num_threads * operations_per_thread {
            manager.add_entity(EntityData {
                id: i,
                position: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                scale: (1.0, 1.0, 1.0),
                active: true,
            });
        }

        let start = Instant::now();

        // 100个线程同时操作
        let handles: Vec<_> = (0..num_threads)
            .map(|thread_id| {
                let manager = manager.clone();
                thread::spawn(move || {
                    let start = Instant::now();

                    // 每个线程执行1000次混合操作
                    for i in 0..operations_per_thread {
                        let id = thread_id * operations_per_thread + i;

                        // 读取
                        let _ = manager.get_entity(id);

                        // 更新
                        manager.update_entity(id, |entity| {
                            entity.position.0 = thread_id as f32;
                        });
                    }

                    start.elapsed()
                })
            })
            .collect();

        // 统计每个线程的耗时
        let mut durations = vec![];
        for handle in handles {
            durations.push(handle.join().unwrap());
        }

        let total_duration = start.elapsed();

        println!("✅ {} 个线程并发操作完成:", num_threads);
        println!("   总耗时: {:?}", total_duration);
        println!("   最快线程: {:?}",
            durations.iter().min().unwrap());
        println!("   最慢线程: {:?}",
            durations.iter().max().unwrap());
        println!("   平均线程耗时: {:?}",
            Duration::from_nanos(
                (durations.iter().map(|d| d.as_nanos()).sum::<u128>() / num_threads as u128) as u64
            ));
    }

    #[test]
    fn test_rapid_add_remove() {
        println!("🧪 测试：快速添加删除");

        let manager = ConcurrentEntityManager::new();
        let iterations = 10_000;

        let start = Instant::now();

        // 快速添加和删除
        for i in 0..iterations {
            manager.add_entity(EntityData {
                id: i,
                position: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                scale: (1.0, 1.0, 1.0),
                active: true,
            });

            if i % 2 == 0 {
                manager.remove_entity(i);
            }
        }

        let duration = start.elapsed();

        println!("✅ {} 次添加/删除操作耗时: {:?}", iterations, duration);
        println!("   平均: {:.2}μs/操作", duration.as_micros() as f64 / iterations as f64);
    }

    #[test]
    fn test_cache_memory_pressure() {
        println!("🧪 测试：缓存内存压力");

        let cache = ConcurrentResourceCache::<Vec<u8>>::new();
        let large_data_size = 1024 * 1024; // 1MB
        let num_items = 100;

        let start = Instant::now();

        // 添加100个1MB的数据项
        for i in 0..num_items {
            let data: Vec<u8> = vec![0; large_data_size];
            cache.insert(format!("large_{}", i), data);
        }

        let insert_duration = start.elapsed();

        // 验证插入成功
        assert_eq!(cache.len(), num_items);

        // 测试访问
        let start = Instant::now();
        for i in 0..num_items {
            let _ = cache.get(&format!("large_{}", i));
        }
        let access_duration = start.elapsed();

        println!("✅ 缓存内存压力测试:");
        println!("   插入 {} 个1MB项耗时: {:?}", num_items, insert_duration);
        println!("   访问 {} 个项耗时: {:?}", num_items, access_duration);
        println!("   总内存: ~{} MB", large_data_size * num_items / (1024 * 1024));
    }

    #[test]
    fn test_sustained_load() {
        println!("🧪 测试：持续负载");

        let manager = ConcurrentEntityManager::new();
        let duration_secs = 5;
        let ops_per_batch = 1000;

        let start = Instant::now();
        let mut total_ops = 0;

        while start.elapsed().as_secs() < duration_secs {
            let batch_start = Instant::now();

            // 每批次执行1000次操作
            for i in 0..ops_per_batch {
                let id = total_ops + i;
                manager.add_entity(EntityData {
                    id,
                    position: (0.0, 0.0, 0.0),
                    rotation: (0.0, 0.0, 0.0),
                    scale: (1.0, 1.0, 1.0),
                    active: true,
                });
            }

            total_ops += ops_per_batch;

            let batch_duration = batch_start.elapsed();
            println!("  批次完成: {} 操作, 耗时: {:?}, 平均: {:.2}μs/操作",
                ops_per_batch, batch_duration,
                batch_duration.as_micros() as f64 / ops_per_batch as f64);
        }

        let total_duration = start.elapsed();

        println!("✅ 持续负载测试:");
        println!("   运行时间: {:?}", total_duration);
        println!("   总操作数: {}", total_ops);
        println!("   吞吐量: {:.2} ops/秒",
            total_ops as f64 / total_duration.as_secs_f64());
    }
}

#[cfg(test)]
mod performance_regression_tests {
    use super::*;

    #[test]
    fn test_lock_contention_performance() {
        println!("🧪 测试：锁竞争性能");

        let manager = ConcurrentEntityManager::new();
        let entity_count = 1000;

        // 预先添加实体
        for i in 0..entity_count {
            manager.add_entity(EntityData {
                id: i,
                position: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                scale: (1.0, 1.0, 1.0),
                active: true,
            });
        }

        // 测试不同线程数下的性能
        for num_threads in [1, 2, 4, 8, 16] {
            let start = Instant::now();

            let handles: Vec<_> = (0..num_threads)
                .map(|_| {
                    let manager = manager.clone();
                    thread::spawn(move || {
                        for i in 0..entity_count {
                            let _ = manager.get_entity(i);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }

            let duration = start.elapsed();
            let total_ops = num_threads * entity_count;

            println!("  {} 线程: {} 次读取, 耗时: {:?}, 平均: {:.2}μs/操作",
                num_threads, total_ops, duration,
                duration.as_micros() as f64 / total_ops as f64);
        }
    }

    #[test]
    fn test_scaling_performance() {
        println!("🧪 测试：扩展性性能");

        let entity_counts = [100, 1000, 10000];

        for entity_count in entity_counts {
            let manager = ConcurrentEntityManager::new();

            // 添加实体
            for i in 0..entity_count {
                manager.add_entity(EntityData {
                    id: i,
                    position: (0.0, 0.0, 0.0),
                    rotation: (0.0, 0.0, 0.0),
                    scale: (1.0, 1.0, 1.0),
                    active: true,
                });
            }

            // 测试读取性能
            let start = Instant::now();
            for i in 0..entity_count {
                let _ = manager.get_entity(i);
            }
            let read_duration = start.elapsed();

            // 测试更新性能
            let start = Instant::now();
            for i in 0..entity_count {
                manager.update_entity(i, |entity| {
                    entity.position.0 += 1.0;
                });
            }
            let update_duration = start.elapsed();

            println!("  实体数: {}:", entity_count);
            println!("    读取性能: {:?}, 平均: {:.2}μs/操作",
                read_duration,
                read_duration.as_micros() as f64 / entity_count as f64);
            println!("    更新性能: {:?}, 平均: {:.2}μs/操作",
                update_duration,
                update_duration.as_micros() as f64 / entity_count as f64);
        }
    }
}
