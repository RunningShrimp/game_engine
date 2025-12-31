// 性能优化基准测试
//
// 测试P3-2性能优化各组件的性能提升

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

// ============================================================================
// 帧时间基准测试
// ============================================================================

fn bench_frame_time_profiling(c: &mut Criterion) {
    use game_engine::profiling::unified_profiler::FrameTimeProfiler;

    let mut group = c.benchmark_group("frame_time_profiling");

    group.bench_function("record_100_frames", |b| {
        b.iter(|| {
            let mut profiler = FrameTimeProfiler::new(1000);
            for _ in 0..100 {
                profiler.begin_frame();
                std::hint::black_box(profiler.end_frame());
            }
            profiler.statistics()
        })
    });

    group.bench_function("record_1000_frames", |b| {
        b.iter(|| {
            let mut profiler = FrameTimeProfiler::new(10000);
            for _ in 0..1000 {
                profiler.begin_frame();
                std::hint::black_box(profiler.end_frame());
            }
            profiler.statistics()
        })
    });

    group.finish();
}

// ============================================================================
// 内存分配基准测试
// ============================================================================

fn bench_memory_allocations(c: &mut Criterion) {
    use game_engine::memory::optimizations::{
        ComponentPool, EntityPool, VecBufferPool, StringInterner,
    };

    let mut group = c.benchmark_group("memory_allocations");

    // 实体池分配
    group.bench_function("entity_pool_allocate_1000", |b| {
        b.iter(|| {
            let mut pool = EntityPool::new(10000);
            for _ in 0..1000 {
                black_box(pool.allocate());
            }
        })
    });

    // 组件池分配
    group.bench_function("component_pool_allocate_1000", |b| {
        b.iter(|| {
            let mut pool = ComponentPool::<i32>::new(10000);
            for i in 0..1000 {
                let idx = pool.allocate().unwrap();
                pool.set(idx, i as i32);
            }
        })
    });

    // Vec缓冲区池
    group.bench_function("vec_buffer_pool_reuse", |b| {
        b.iter(|| {
            let mut pool = VecBufferPool::<u8>::new(10, 100);
            for _ in 0..100 {
                let mut buf = pool.acquire();
                buf.extend(0..1000);
                pool.release(buf);
            }
        })
    });

    // String interning
    group.bench_function("string_interning_1000_unique", |b| {
        b.iter(|| {
            let mut interner = StringInterner::new();
            for i in 0..1000 {
                black_box(interner.intern(&format!("string_{}", i)));
            }
        })
    });

    group.bench_function("string_interning_1000_duplicate", |b| {
        b.iter(|| {
            let mut interner = StringInterner::new();
            for _ in 0..1000 {
                for j in 0..10 {
                    black_box(interner.intern(&format!("string_{}", j)));
                }
            }
        })
    });

    group.finish();
}

// ============================================================================
// LRU缓存基准测试
// ============================================================================

fn bench_lru_cache(c: &mut Criterion) {
    use game_engine::memory::optimizations::LruCache;
    use std::time::Duration;

    let mut group = c.benchmark_group("lru_cache");

    group.bench_function("insert_1000_items", |b| {
        b.iter(|| {
            let mut cache = LruCache::new(1000, Duration::from_secs(60));
            for i in 0..1000 {
                cache.insert(format!("key_{}", i), i);
            }
        })
    });

    group.bench_function("get_1000_items", |b| {
        let mut cache = LruCache::new(1000, Duration::from_secs(60));
        for i in 0..1000 {
            cache.insert(format!("key_{}", i), i);
        }

        b.iter(|| {
            for i in 0..1000 {
                black_box(cache.get(&format!("key_{}", i)));
            }
        })
    });

    group.finish();
}

// ============================================================================
// 批处理基准测试
// ============================================================================

fn bench_render_batching(c: &mut Criterion) {
    use game_engine::render::optimizations::batching::{
        BatchingManager, BatchingConfig, BatchingStrategy, InstanceData,
    };

    let mut group = c.benchmark_group("render_batching");

    // 动态批处理
    group.bench_function("dynamic_batch_100_meshes", |b| {
        b.iter(|| {
            let mut manager = BatchingManager::new(
                BatchingConfig {
                    strategy: BatchingStrategy::Dynamic,
                    ..Default::default()
                },
                1000,
            );

            manager.begin_frame();
            for i in 0..100 {
                manager.add_dynamic_mesh(i, i % 10, 100, 150);
            }
            manager.get_dynamic_batches()
        })
    });

    // 实例化渲染
    group.bench_function("instancing_100_instances", |b| {
        b.iter(|| {
            let mut manager = BatchingManager::new(
                BatchingConfig::default(),
                1000,
            );

            let instance = InstanceData {
                transform: [[1.0, 0.0, 0.0, 0.0]; 4],
                color: [1.0, 0.0, 0.0, 1.0],
                custom_data: vec![],
            };

            for _ in 0..100 {
                manager.add_instance(1, 100, instance.clone());
            }
            manager.get_instanced_batches()
        })
    });

    group.finish();
}

// ============================================================================
// 剔除基准测试
// ============================================================================

fn bench_frustum_culling(c: &mut Criterion) {
    use game_engine::render::optimizations::culling::FrustumCuller;

    let mut group = c.benchmark_group("frustum_culling");

    let view_proj = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    group.bench_function("cull_1000_objects", |b| {
        b.iter(|| {
            let mut culler = FrustumCuller::new();
            culler.update_frustum(&view_proj);

            for i in 0..1000 {
                let min = [i as f32, i as f32, i as f32];
                let max = [(i + 1) as f32, (i + 1) as f32, (i + 1) as f32];
                black_box(culler.test_aabb(min, max));
            }
        })
    });

    group.finish();
}

// ============================================================================
// 排序基准测试
// ============================================================================

fn bench_render_sorting(c: &mut Criterion) {
    use game_engine::render::optimizations::sort::{
        RenderItem, RenderQueue, SortStrategy,
    };

    let mut group = c.benchmark_group("render_sorting");

    // 按材质排序
    group.bench_function("sort_by_material_100_items", |b| {
        let items: Vec<_> = (0..100)
            .map(|i| RenderItem::new(i, i % 10, i, i as f32, false))
            .collect();

        b.iter(|| {
            let mut queue = RenderQueue::new(SortStrategy::ByMaterial);
            for item in &items {
                queue.add_item(item.clone());
            }
            queue.sort()
        })
    });

    // 混合排序
    group.bench_function("hybrid_sort_100_items", |b| {
        let items: Vec<_> = (0..100)
            .map(|i| RenderItem::new(i, i % 10, i, i as f32, i % 2 == 0))
            .collect();

        b.iter(|| {
            let mut queue = RenderQueue::new(SortStrategy::Hybrid);
            for item in &items {
                queue.add_item(item.clone());
            }
            queue.sort()
        })
    });

    group.finish();
}

// ============================================================================
// SIMD基准测试
// ============================================================================

fn bench_simd_operations(c: &mut Criterion) {
    use game_engine::simd::accelerated::{SimdVecOps, SimdMatrixOps};

    let mut group = c.benchmark_group("simd_operations");

    // 向量加法
    group.bench_function("vec3_add_batch_1000", |b| {
        let ops = SimdVecOps::new();
        let a: Vec<[f32; 3]> = (0..1000).map(|i| [i as f32, i as f32, i as f32]).collect();
        let b: Vec<[f32; 3]> = (0..1000).map(|i| [1.0, 2.0, 3.0]).collect();
        let mut dest = vec![[0.0; 3]; 1000];

        b.iter(|| {
            ops.add_vec3_batch(&a, &b, &mut dest);
            black_box(&dest);
        })
    });

    // 向量点积
    group.bench_function("vec3_dot_1000", |b| {
        let ops = SimdVecOps::new();
        let a: Vec<[f32; 3]> = (0..1000).map(|i| [i as f32, i as f32, i as f32]).collect();
        let b: Vec<[f32; 3]> = (0..1000).map(|i| [1.0, 2.0, 3.0]).collect();

        b.iter(|| {
            let mut sum = 0.0;
            for i in 0..1000 {
                sum += ops.dot_vec3(a[i], b[i]);
            }
            black_box(sum)
        })
    });

    // 矩阵乘法
    group.bench_function("mat4_mul_100", |b| {
        let ops = SimdMatrixOps::new();
        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        b.iter(|| {
            for _ in 0..100 {
                black_box(ops.mul_mat4(&identity, &identity));
            }
        })
    });

    group.finish();
}

// ============================================================================
// 综合性能基准测试
// ============================================================================

fn bench_comprehensive_performance(c: &mut Criterion) {
    let mut group = c.benchmark_group("comprehensive");

    group.measurement_time(Duration::from_secs(30));

    group.bench_function("full_pipeline", |b| {
        use game_engine::{
            profiling::unified_profiler::UnifiedProfiler,
            memory::optimizations::{EntityPool, ComponentPool},
            render::optimizations::{
                batching::{BatchingManager, BatchingConfig},
                culling::FrustumCuller,
                sort::{RenderItem, RenderQueue, SortStrategy},
            },
            simd::accelerated::SimdVecOps,
        };

        b.iter(|| {
            // 1. 创建剖析器
            let mut profiler = UnifiedProfiler::new(game_engine::profiling::unified_profiler::ProfilingBackend::BuiltIn);
            profiler.begin_frame();

            // 2. 实体和组件分配
            let mut entity_pool = EntityPool::new(1000);
            let mut component_pool = ComponentPool::<f32>::new(1000);

            for i in 0..100 {
                if let Some(entity) = entity_pool.allocate() {
                    let idx = component_pool.allocate().unwrap();
                    component_pool.set(idx, i as f32);
                }
            }

            // 3. 渲染批处理
            let mut batch_manager = BatchingManager::new(BatchingConfig::default(), 1000);
            batch_manager.begin_frame();

            for i in 0..100 {
                batch_manager.add_dynamic_mesh(i, i % 10, 100, 150);
            }

            let _batches = batch_manager.get_dynamic_batches();

            // 4. 视锥体剔除
            let mut culler = FrustumCuller::new();
            let view_proj = [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ];
            culler.update_frustum(&view_proj);

            // 5. 渲染排序
            let mut queue = RenderQueue::new(SortStrategy::Hybrid);
            for i in 0..100 {
                let item = RenderItem::new(i, i % 10, i, i as f32, i % 2 == 0);
                queue.add_item(item);
            }
            let _sorted = queue.sort();

            // 6. SIMD运算
            let vec_ops = SimdVecOps::new();
            let a: Vec<[f32; 3]> = (0..100).map(|i| [i as f32, i as f32, i as f32]).collect();
            let b: Vec<[f32; 3]> = (0..100).map(|i| [1.0, 2.0, 3.0]).collect();
            let mut dest = vec![[0.0; 3]; 100];
            vec_ops.add_vec3_batch(&a, &b, &mut dest);

            profiler.end_frame();

            black_box(profiler.generate_report())
        })
    });

    group.finish();
}

// ============================================================================
// 比较基准测试（优化前 vs 优化后）
// ============================================================================

fn bench_optimization_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_comparison");

    // 使用对象池 vs 不使用对象池
    group.bench_function("with_pool", |b| {
        use game_engine::memory::optimizations::VecBufferPool;

        b.iter(|| {
            let mut pool = VecBufferPool::<u32>::new(10, 100);
            for _ in 0..1000 {
                let mut buf = pool.acquire();
                buf.extend(0..100);
                pool.release(buf);
            }
        })
    });

    group.bench_function("without_pool", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let mut buf = Vec::new();
                buf.extend(0..100);
                std::hint::black_box(buf);
            }
        })
    });

    // SIMD vs 标量
    group.bench_function("simd_vector_ops", |b| {
        use game_engine::simd::accelerated::SimdVecOps;

        let ops = SimdVecOps::new();
        let a: Vec<[f32; 3]> = (0..1000).map(|i| [i as f32, i as f32, i as f32]).collect();
        let b: Vec<[f32; 3]> = (0..1000).map(|i| [1.0, 2.0, 3.0]).collect();
        let mut dest = vec![[0.0; 3]; 1000];

        b.iter(|| {
            ops.add_vec3_batch(&a, &b, &mut dest);
            black_box(&dest);
        })
    });

    group.bench_function("scalar_vector_ops", |b| {
        let a: Vec<[f32; 3]> = (0..1000).map(|i| [i as f32, i as f32, i as f32]).collect();
        let b: Vec<[f32; 3]> = (0..1000).map(|i| [1.0, 2.0, 3.0]).collect();
        let mut dest = vec![[0.0; 3]; 1000];

        b.iter(|| {
            for i in 0..1000 {
                dest[i] = [
                    a[i][0] + b[i][0],
                    a[i][1] + b[i][1],
                    a[i][2] + b[i][2],
                ];
            }
            black_box(&dest);
        })
    });

    group.finish();
}

// ============================================================================
// 内存分配速率基准测试
// ============================================================================

fn bench_memory_allocation_rate(c: &mut Criterion) {
    use game_engine::profiling::unified_profiler::MemoryAllocationTracker;

    let mut group = c.benchmark_group("memory_allocation_rate");

    group.bench_function("track_10000_allocations", |b| {
        let tracker = MemoryAllocationTracker::new();

        b.iter(|| {
            for i in 0..10000 {
                tracker.record_allocation((i % 1000) + 1);
            }
        })
    });

    group.finish();
}

// ============================================================================
// Draw Call基准测试
// ============================================================================

fn bench_draw_call_reduction(c: &mut Criterion) {
    use game_engine::render::optimizations::batching::{
        BatchingConfig, BatchingManager, BatchingStrategy,
    };

    let mut group = c.benchmark_group("draw_call_reduction");

    // 测量批处理前的draw call数量
    group.bench_function("without_batching", |b| {
        b.iter(|| {
            let count = 1000; // 1000个网格
            black_box(count)
        })
    });

    // 测量批处理后的draw call数量
    group.bench_function("with_batching", |b| {
        b.iter(|| {
            let mut manager = BatchingManager::new(
                BatchingConfig {
                    strategy: BatchingStrategy::Dynamic,
                    ..Default::default()
                },
                1000,
            );

            manager.begin_frame();
            for i in 0..1000 {
                manager.add_dynamic_mesh(i, i % 20, 100, 150);
            }

            let batches = manager.get_dynamic_batches();
            black_box(batches.len())
        })
    });

    group.finish();
}

// ============================================================================
// 注册基准测试组
// ============================================================================

criterion_group!(
    benches,
    bench_frame_time_profiling,
    bench_memory_allocations,
    bench_lru_cache,
    bench_render_batching,
    bench_frustum_culling,
    bench_render_sorting,
    bench_simd_operations,
    bench_comprehensive_performance,
    bench_optimization_comparison,
    bench_memory_allocation_rate,
    bench_draw_call_reduction,
);

criterion_main!(benches);
