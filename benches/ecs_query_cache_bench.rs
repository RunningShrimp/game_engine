// ECS查询缓存性能基准测试
//
// 验证查询缓存相比直接查询的性能提升
//
// 运行: cargo bench --bench ecs_query_cache_bench

use bevy_ecs::entity::Entity;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use game_engine::ecs::query_cache::QueryTypeId;
use game_engine::ecs::{QueryCache, QueryCacheConfig, Transform};
use std::time::Duration;

/// 模拟查询操作的成本 (创建实体列表)
fn simulate_query(entity_count: usize) -> Vec<Entity> {
    (0..entity_count).map(|i| Entity::from_bits(i as u64)).collect()
}

/// 基准测试场景1: 重复查询相同实体列表
fn bench_repeated_query(c: &mut Criterion) {
    let mut group = c.benchmark_group("repeated_query");

    for entity_count in [100, 1_000, 10_000].iter() {
        let query_type = QueryTypeId::from_type::<Transform>();

        // 直接查询 (无缓存)
        group.bench_with_input(
            BenchmarkId::new("no_cache", entity_count),
            &entity_count,
            |bencher, &_count| {
                bencher.iter(|| {
                    // 模拟直接查询操作
                    let entities = simulate_query(*entity_count);
                    std::hint::black_box(entities);
                });
            },
        );

        // 使用缓存查询
        group.bench_with_input(
            BenchmarkId::new("with_cache", entity_count),
            &(entity_count, query_type.clone()),
            |bencher, (count, query_type)| {
                let mut cache = QueryCache::new(QueryCacheConfig::default());

                // 预热缓存
                let entities = simulate_query(**count);
                cache.insert_query_result(query_type.clone(), entities);

                bencher.iter(|| {
                    // 从缓存返回结果 (快速)
                    let result = cache
                        .query_cached::<()>(&bevy_ecs::world::World::new(), query_type.clone());
                    std::hint::black_box(result);
                });
            },
        );
    }

    group.finish();
}

/// 基准测试场景2: 缓存命中 vs 未命中性能对比
fn bench_cache_hit_vs_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_hit_vs_miss");

    // 缓存命中场景
    group.bench_function("cache_hit", |bencher| {
        let mut cache = QueryCache::new(QueryCacheConfig::default());
        let query_type = QueryTypeId::from_type::<Transform>();
        let world = bevy_ecs::world::World::new();

        // 预热缓存
        let entities = simulate_query(1000);
        cache.insert_query_result(query_type.clone(), entities);

        bencher.iter(|| {
            // 缓存命中，快速返回
            let result = cache.query_cached::<()>(&world, query_type.clone());
            std::hint::black_box(result);
        });
    });

    // 缓存未命中场景
    group.bench_function("cache_miss", |bencher| {
        let mut cache = QueryCache::new(QueryCacheConfig::default());
        let world = bevy_ecs::world::World::new();
        let query_type = QueryTypeId::from_type::<Transform>();

        bencher.iter(|| {
            // 缓存未命中，返回空结果
            let result = cache.query_cached::<()>(&world, query_type.clone());
            std::hint::black_box(result);
        });
    });

    group.finish();
}

/// 基准测试场景3: 缓存失效性能
fn bench_cache_invalidation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_invalidation");

    // 无脏标记 (缓存一直有效)
    group.bench_function("no_dirty_flags", |bencher| {
        let mut cache = QueryCache::new(QueryCacheConfig::default());
        let query_type = QueryTypeId::from_type::<Transform>();
        let world = bevy_ecs::world::World::new();

        // 预热缓存
        let entities = simulate_query(1000);
        cache.insert_query_result(query_type.clone(), entities);

        bencher.iter(|| {
            // 缓存持续有效，快速返回
            let result = cache.query_cached::<()>(&world, query_type.clone());
            std::hint::black_box(result);
        });
    });

    // 有脏标记 (需要重新查询)
    group.bench_function("with_dirty_flags", |bencher| {
        let mut cache = QueryCache::new(QueryCacheConfig::default());
        let query_type = QueryTypeId::from_type::<Transform>();
        let world = bevy_ecs::world::World::new();

        bencher.iter(|| {
            // 组件变更导致缓存失效，需要重新查询
            cache.mark_component_dirty::<Transform>();
            let result = cache.query_cached::<()>(&world, query_type.clone());
            std::hint::black_box(result);
        });
    });

    group.finish();
}

/// 基准测试场景4: LRU淘汰性能
fn bench_lru_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("lru_eviction");

    // 测试LRU淘汰对性能的影响
    group.bench_function("lru_eviction", |bencher| {
        let mut cache = QueryCache::new(QueryCacheConfig {
            max_cache_size: 10, // 小缓存以触发淘汰
            cache_ttl: Duration::from_millis(16),
            enable_dirty_invalidation: false,
            lru_queue_size: 10,
        });

        bencher.iter(|| {
            // 插入超过缓存大小的查询类型，触发LRU淘汰
            for i in 0..15 {
                let query_type = QueryTypeId::from_id(format!("query_{}", i));
                let entities = vec![Entity::from_bits(i as u64)];
                cache.insert_query_result(query_type, entities);
            }
            std::hint::black_box(&cache);
        });
    });

    group.finish();
}

/// 基准测试场景5: 内存开销评估
fn bench_memory_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_overhead");

    // 无缓存
    group.bench_function("no_cache_memory", |bencher| {
        bencher.iter(|| {
            // 仅存储查询结果，无持久化开销
            let entities: Vec<Entity> = (0..1000).map(|i| Entity::from_bits(i)).collect();
            std::hint::black_box(entities);
        });
    });

    // 有缓存 (<20%内存开销)
    group.bench_function("with_cache_memory", |bencher| {
        bencher.iter(|| {
            // 持有缓存结构，略有内存开销
            let mut cache = QueryCache::new(QueryCacheConfig::default());
            let query_type = QueryTypeId::from_type::<Transform>();
            let entities: Vec<Entity> = (0..1000).map(|i| Entity::from_bits(i)).collect();
            cache.insert_query_result(query_type, entities);
            std::hint::black_box(cache);
        });
    });

    group.finish();
}

/// 基准测试场景6: 缓存插入性能
fn bench_cache_insertion(c: &mut Criterion) {
    let mut group = c.benchmark_group("cache_insertion");

    for entity_count in [100, 1_000, 10_000].iter() {
        group.bench_with_input(
            BenchmarkId::new("insert", entity_count),
            &entity_count,
            |bencher, count| {
                bencher.iter(|| {
                    let mut cache = QueryCache::new(QueryCacheConfig::default());
                    let query_type = QueryTypeId::from_id("test_query".to_string());
                    let entities: Vec<Entity> =
                        (0..**count).map(|i| Entity::from_bits(i as u64)).collect();
                    cache.insert_query_result(query_type, entities);
                    std::hint::black_box(&cache);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    name = ecs_query_cache_benches;
    config = Criterion::default().sample_size(100);
    targets =
        bench_repeated_query,
        bench_cache_hit_vs_miss,
        bench_cache_invalidation,
        bench_lru_eviction,
        bench_memory_overhead,
        bench_cache_insertion
);

criterion_main!(ecs_query_cache_benches);
