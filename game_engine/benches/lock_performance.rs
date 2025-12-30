// 性能基准测试 - parking_lot vs std::sync
//
// 运行: cargo bench --bench lock_performance

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use game_engine::resources::{
    dashmap_optimizations::{ConcurrentEntityManager, ConcurrentResourceCache},
    optimized_manager::OptimizedAssetManager,
};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

/// 基准测试：parking_lot::RwLock vs std::sync::RwLock
fn bench_rwlock_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("rwlock_read");

    // parking_lot::RwLock
    group.bench_function("parking_lot", |b| {
        let lock = parking_lot::RwLock::new(42);
        b.iter(|| {
            let _guard = lock.read();
            black_box(&_guard);
        });
    });

    // std::sync::RwLock
    group.bench_function("std_sync", |b| {
        let lock = RwLock::new(42);
        b.iter(|| {
            let _guard = lock.read().unwrap();
            black_box(&_guard);
        });
    });

    group.finish();
}

/// 基准测试：parking_lot::RwLock vs std::sync::RwLock（写操作）
fn bench_rwlock_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("rwlock_write");

    // parking_lot::RwLock
    group.bench_function("parking_lot", |b| {
        let lock = parking_lot::RwLock::new(42);
        b.iter(|| {
            let mut _guard = lock.write();
            *_guard = black_box(100);
        });
    });

    // std::sync::RwLock
    group.bench_function("std_sync", |b| {
        let lock = RwLock::new(42);
        b.iter(|| {
            let mut _guard = lock.write().unwrap();
            *_guard = black_box(100);
        });
    });

    group.finish();
}

/// 基准测试：并发读取场景
fn bench_concurrent_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_read");

    // parking_lot::RwLock (10线程)
    group.bench_function("parking_lot_10_threads", |b| {
        let lock = Arc::new(parking_lot::RwLock::new(vec![1u32; 1000]));
        b.iter(|| {
            let lock = lock.clone();
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let lock = lock.clone();
                    std::thread::spawn(move || {
                        for _ in 0..100 {
                            let _data = lock.read();
                            black_box(&_data);
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });
    });

    // std::sync::RwLock (10线程)
    group.bench_function("std_sync_10_threads", |b| {
        let lock = Arc::new(RwLock::new(vec![1u32; 1000]));
        b.iter(|| {
            let lock = lock.clone();
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let lock = lock.clone();
                    std::thread::spawn(move || {
                        for _ in 0..100 {
                            let _data = lock.read().unwrap();
                            black_box(&_data);
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });
    });

    group.finish();
}

/// 基准测试：DashMap vs Mutex<HashMap>
fn bench_map_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_concurrent");

    // DashMap
    group.bench_function("dashmap", |b| {
        let map = Arc::new(dashmap::DashMap::new());
        b.iter(|| {
            let map = map.clone();
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let map = map.clone();
                    std::thread::spawn(move || {
                        for j in 0..100 {
                            map.insert(i * 100 + j, j);
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });
    });

    // Mutex<HashMap>
    group.bench_function("mutex_hashmap", |b| {
        let map = Arc::new(Mutex::new(std::collections::HashMap::new()));
        b.iter(|| {
            let map = map.clone();
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let map = map.clone();
                    std::thread::spawn(move || {
                        for j in 0..100 {
                            let mut m = map.lock().unwrap();
                            m.insert(i * 100 + j, j);
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });
    });

    group.finish();
}

/// 基准测试：资源管理器并发访问
fn bench_asset_manager_concurrent(c: &mut Criterion) {
    let mut group = c.benchmark_group("asset_manager_concurrent");

    // 优化的资源管理器 (parking_lot)
    group.bench_function("optimized_parking_lot", |b| {
        let manager = Arc::new(OptimizedAssetManager::new());

        // 预先添加一些资源
        for i in 0..100 {
            let path = format!("texture_{}.png", i);
            manager.load_texture(&path).ok();
        }

        b.iter(|| {
            let manager = manager.clone();
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let manager = manager.clone();
                    std::thread::spawn(move || {
                        for i in 0..100 {
                            let path = format!("texture_{}.png", i);
                            let _ = manager.get_texture(&path);
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });
    });

    group.finish();
}

/// 基准测试：实体管理器
fn bench_entity_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_manager");

    // DashMap实体管理器
    group.bench_function("dashmap_concurrent", |b| {
        let manager = Arc::new(ConcurrentEntityManager::new());

        // 预先添加实体
        for i in 0..1000 {
            manager.add_entity(game_engine::resources::dashmap_optimizations::EntityData {
                id: i,
                position: (0.0, 0.0, 0.0),
                rotation: (0.0, 0.0, 0.0),
                scale: (1.0, 1.0, 1.0),
                active: true,
            });
        }

        b.iter(|| {
            for i in 0..1000 {
                let _ = manager.get_entity(i);
            }
        });
    });

    group.finish();
}

/// 基准测试：资源缓存
fn bench_resource_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_cache");

    // DashMap资源缓存
    group.bench_function("dashmap_cache", |b| {
        let cache = Arc::new(ConcurrentResourceCache::new());

        // 预先添加资源
        for i in 0..1000 {
            cache.insert(format!("resource_{}", i), i);
        }

        b.iter(|| {
            for i in 0..1000 {
                let _ = cache.get(&format!("resource_{}", i));
            }
        });
    });

    group.finish();
}

/// 基准测试：不同锁粒度
fn bench_lock_granularity(c: &mut Criterion) {
    let mut group = c.benchmark_group("lock_granularity");

    // 细粒度锁（每个资源一个锁）
    group.bench_function("fine_grained", |b| {
        use std::collections::HashMap;
        let locks: Arc<Mutex<HashMap<String, parking_lot::RwLock<u32>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // 初始化
        {
            let mut locks = locks.lock().unwrap();
            for i in 0..100 {
                locks.insert(format!("resource_{}", i), parking_lot::RwLock::new(i));
            }
        }

        b.iter(|| {
            let locks = locks.clone();
            let handles: Vec<_> = (0..10)
                .map(|i| {
                    let locks = locks.clone();
                    std::thread::spawn(move || {
                        for j in 0..100 {
                            let locks = locks.lock().unwrap();
                            if let Some(lock) = locks.get(&format!("resource_{}", j % 100)) {
                                let _data = lock.read();
                                black_box(&_data);
                            }
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });
    });

    // 粗粒度锁（单个大锁）
    group.bench_function("coarse_grained", |b| {
        use std::collections::HashMap;
        let lock: Arc<parking_lot::RwLock<HashMap<String, u32>>> =
            Arc::new(parking_lot::RwLock::new(HashMap::new()));

        // 初始化
        {
            let mut data = lock.write();
            for i in 0..100 {
                data.insert(format!("resource_{}", i), i);
            }
        }

        b.iter(|| {
            let lock = lock.clone();
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let lock = lock.clone();
                    std::thread::spawn(move || {
                        for j in 0..100 {
                            let _data = lock.read();
                            black_box(&_data);
                        }
                    })
                })
                .collect();

            for h in handles {
                h.join().unwrap();
            }
        });
    });

    group.finish();
}

/// 基准测试：批量操作
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    // 批量加载（优化版本）
    group.bench_function("batch_load_optimized", |b| {
        let manager = OptimizedAssetManager::new();
        let names: Vec<String> = (0..100).map(|i| format!("texture_{}.png", i)).collect();

        b.iter(|| {
            let names: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
            let _ = manager.load_textures_batch(&names);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_rwlock_read,
    bench_rwlock_write,
    bench_concurrent_read,
    bench_map_concurrent,
    bench_asset_manager_concurrent,
    bench_entity_manager,
    bench_resource_cache,
    bench_lock_granularity,
    bench_batch_operations
);

criterion_main!(benches);
