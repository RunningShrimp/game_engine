// DashMap vs RwLock<HashMap> Performance Comparison Benchmark
//
// This benchmark compares the performance of DashMap versus RwLock<HashMap>
// in concurrent scenarios typical of game engine resource management.
//
// Run:
//   cargo bench --bench dashmap_comparison --features dashmap

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::collections::HashMap;
use std::sync::{Arc, RwLock as StdRwLock};
use std::hint::black_box;
use std::thread;
use std::time::Duration;

#[cfg(feature = "dashmap")]
use dashmap::DashMap;

// ============================================================================
// Test Data Structures
// ============================================================================

/// Mock resource representing game engine asset
#[derive(Clone)]
#[allow(dead_code)]
struct MockResource {
    id: u32,
    data: Vec<u8>,
}

impl MockResource {
    fn new(id: u32) -> Self {
        Self {
            id,
            data: vec![0u8; 1024], // 1KB of data
        }
    }
}

// ============================================================================
// DashMap Implementation
// ============================================================================

#[cfg(feature = "dashmap")]
struct DashMapCache {
    map: DashMap<u32, MockResource>,
}

#[cfg(feature = "dashmap")]
impl DashMapCache {
    fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    fn insert(&self, key: u32, value: MockResource) {
        self.map.insert(key, value);
    }

    fn get(&self, key: u32) -> Option<MockResource> {
        self.map.get(&key).map(|v| v.clone())
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.map.len()
    }
}

// ============================================================================
// RwLock Implementation
// ============================================================================

struct RwLockCache {
    map: StdRwLock<HashMap<u32, MockResource>>,
}

impl RwLockCache {
    fn new() -> Self {
        Self {
            map: StdRwLock::new(HashMap::new()),
        }
    }

    fn insert(&self, key: u32, value: MockResource) {
        let mut map = self.map.write().unwrap();
        map.insert(key, value);
    }

    fn get(&self, key: u32) -> Option<MockResource> {
        let map = self.map.read().unwrap();
        map.get(&key).cloned()
    }

    #[allow(dead_code)]
    fn len(&self) -> usize {
        let map = self.map.read().unwrap();
        map.len()
    }
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_concurrent_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_read");
    group.measurement_time(Duration::from_secs(5));

    for num_threads in [2, 4, 8].iter() {
        let ops_per_thread = 10000;

        // DashMap version
        #[cfg(feature = "dashmap")]
        group.bench_with_input(
            BenchmarkId::new("dashmap", num_threads),
            num_threads,
            |b, &num_threads| {
                let cache = Arc::new(DashMapCache::new());

                // Pre-populate cache
                for i in 0..1000 {
                    cache.insert(i, MockResource::new(i));
                }

                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let cache = cache.clone();
                            thread::spawn(move || {
                                for i in 0..ops_per_thread {
                                    black_box(cache.get(i % 1000));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // RwLock version
        group.bench_with_input(
            BenchmarkId::new("rwlock_hashmap", num_threads),
            num_threads,
            |b, &num_threads| {
                let cache = Arc::new(RwLockCache::new());

                // Pre-populate cache
                for i in 0..1000 {
                    cache.insert(i, MockResource::new(i));
                }

                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|_| {
                            let cache = cache.clone();
                            thread::spawn(move || {
                                for i in 0..ops_per_thread {
                                    black_box(cache.get(i % 1000));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_concurrent_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_write");
    group.measurement_time(Duration::from_secs(5));

    for num_threads in [2, 4, 8].iter() {
        let ops_per_thread = 5000;

        // DashMap version
        #[cfg(feature = "dashmap")]
        group.bench_with_input(
            BenchmarkId::new("dashmap", num_threads),
            num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let cache = Arc::new(DashMapCache::new());

                    let handles: Vec<_> = (0..num_threads)
                        .map(|thread_id| {
                            let cache = cache.clone();
                            thread::spawn(move || {
                                for i in 0..ops_per_thread {
                                    let key = thread_id * ops_per_thread + i;
                                    cache.insert(key as u32, MockResource::new(key as u32));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );

        // RwLock version
        group.bench_with_input(
            BenchmarkId::new("rwlock_hashmap", num_threads),
            num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let cache = Arc::new(RwLockCache::new());

                    let handles: Vec<_> = (0..num_threads)
                        .map(|thread_id| {
                            let cache = cache.clone();
                            thread::spawn(move || {
                                for i in 0..ops_per_thread {
                                    let key = thread_id * ops_per_thread + i;
                                    cache.insert(key as u32, MockResource::new(key as u32));
                                }
                            })
                        })
                        .collect();

                    for handle in handles {
                        handle.join().unwrap();
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_workload");
    group.measurement_time(Duration::from_secs(5));

    let num_threads = 8;
    let ops_per_thread = 10000;

    // DashMap version
    #[cfg(feature = "dashmap")]
    group.bench_function("dashmap", |b| {
        let cache = Arc::new(DashMapCache::new());

        // Pre-populate with some data
        for i in 0..1000 {
            cache.insert(i, MockResource::new(i));
        }

        b.iter(|| {
            let handles: Vec<_> = (0..num_threads)
                .map(|thread_id| {
                    let cache = cache.clone();
                    thread::spawn(move || {
                        for i in 0..ops_per_thread {
                            if i % 10 < 7 {
                                // 70% reads
                                black_box(cache.get(i % 1000));
                            } else {
                                // 30% writes
                                let key = thread_id * ops_per_thread + i;
                                cache.insert(key as u32, MockResource::new(key as u32));
                            }
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    // RwLock version
    group.bench_function("rwlock_hashmap", |b| {
        let cache = Arc::new(RwLockCache::new());

        // Pre-populate with some data
        for i in 0..1000 {
            cache.insert(i, MockResource::new(i));
        }

        b.iter(|| {
            let handles: Vec<_> = (0..num_threads)
                .map(|thread_id| {
                    let cache = cache.clone();
                    thread::spawn(move || {
                        for i in 0..ops_per_thread {
                            if i % 10 < 7 {
                                // 70% reads
                                black_box(cache.get(i % 1000));
                            } else {
                                // 30% writes
                                let key = thread_id * ops_per_thread + i;
                                cache.insert(key as u32, MockResource::new(key as u32));
                            }
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        });
    });

    group.finish();
}

fn bench_single_threaded_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_threaded_read");

    // DashMap version
    #[cfg(feature = "dashmap")]
    group.bench_function("dashmap", |b| {
        let cache = DashMapCache::new();
        for i in 0..10000 {
            cache.insert(i, MockResource::new(i));
        }

        b.iter(|| {
            for i in 0..10000 {
                black_box(cache.get(i));
            }
        });
    });

    // RwLock version
    group.bench_function("rwlock_hashmap", |b| {
        let cache = RwLockCache::new();
        for i in 0..10000 {
            cache.insert(i, MockResource::new(i));
        }

        b.iter(|| {
            for i in 0..10000 {
                black_box(cache.get(i));
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_threaded_read,
    bench_concurrent_read,
    bench_concurrent_write,
    bench_mixed_workload
);

criterion_main!(benches);
