//  资源管理系统性能基准测试
//
//  测试资源加载、缓存、异步处理等操作的性能

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

fn bench_resource_manager_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_manager_creation");

    group.bench_function("create_hashmap", |b| {
        b.iter(|| std::hint::black_box(std::collections::HashMap::<String, Vec<u8>>::new()));
    });

    group.finish();
}

fn bench_texture_loading(c: &mut Criterion) {
    let mut group = c.benchmark_group("texture_loading");

    // 创建一些测试数据
    let test_data = vec![0u8; 1024 * 1024]; // 1MB 测试数据

    group.bench_function("load_texture_data", |b| {
        b.iter(|| {
            // 模拟纹理数据处理
            let mut processed = Vec::with_capacity(test_data.len());
            for &byte in &test_data {
                processed.push(byte.wrapping_add(1));
            }
            std::hint::black_box(processed)
        });
    });

    group.finish();
}

fn bench_resource_hashmap_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_hashmap_operations");

    group.bench_function("hashmap_insert", |b| {
        b.iter(|| {
            let mut map = std::collections::HashMap::new();
            for i in 0..100 {
                map.insert(format!("resource_{}", i), vec![i as u8; 1024]);
            }
            std::hint::black_box(map)
        });
    });

    group.bench_function("hashmap_lookup", |b| {
        let mut map = std::collections::HashMap::new();
        for i in 0..100 {
            map.insert(format!("resource_{}", i), vec![i as u8; 1024]);
        }

        b.iter(|| {
            for i in 0..50 {
                std::hint::black_box(map.get(&format!("resource_{}", i)));
            }
        });
    });

    group.finish();
}

fn bench_resource_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_cache");

    // 模拟资源缓存操作，测试不同缓存大小的性能
    for cache_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("cache_{}", cache_size)),
            cache_size,
            |b, &size| {
                b.iter(|| {
                    let mut cache: std::collections::HashMap<String, Vec<u8>> =
                        std::collections::HashMap::new();
                    // 模拟缓存查找和插入
                    for i in 0..size {
                        let key = format!("resource_{}", i);
                        cache.insert(key, vec![i as u8; 100]);
                    }
                    std::hint::black_box(cache.len())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_resource_manager_creation,
    bench_texture_loading,
    bench_resource_hashmap_operations,
    bench_resource_cache
);
criterion_main!(benches);
