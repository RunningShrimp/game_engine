// 内存性能基准测试
//
// 测试内存分配、组件布局、内存池等内存相关性能

use bevy_ecs::prelude::*;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use game_engine::ecs::{Mesh, Sprite, Transform, Velocity};
use glam::{Quat, Vec3};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// 自定义内存分配器，用于测量内存分配
struct MeasuringAllocator;

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static BYTES_ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for MeasuringAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        BYTES_ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Ordering::SeqCst);
        System.dealloc(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL: MeasuringAllocator = MeasuringAllocator;

fn reset_memory_stats() {
    ALLOCATIONS.store(0, Ordering::SeqCst);
    DEALLOCATIONS.store(0, Ordering::SeqCst);
    BYTES_ALLOCATED.store(0, Ordering::SeqCst);
}

fn get_allocation_count() -> usize {
    ALLOCATIONS.load(Ordering::SeqCst)
}

fn get_bytes_allocated() -> usize {
    BYTES_ALLOCATED.load(Ordering::SeqCst)
}

/// Benchmark ECS实体创建的内存分配
fn bench_entity_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_memory_allocation");

    for entity_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            entity_count,
            |b, &count| {
                b.iter(|| {
                    reset_memory_stats();
                    let mut world = World::new();

                    for _ in 0..count {
                        world.spawn((Transform::default(), Velocity::default()));
                    }

                    let alloc_count = get_allocation_count();
                    let bytes_allocated = get_bytes_allocated();

                    black_box((world, alloc_count, bytes_allocated));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark组件添加的内存分配
fn bench_component_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_memory_allocation");

    for component_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(component_count),
            component_count,
            |b, &count| {
                b.iter(|| {
                    reset_memory_stats();
                    let mut world = World::new();

                    let entities: Vec<Entity> =
                        (0..count).map(|_| world.spawn(Transform::default()).id()).collect();

                    for entity in entities {
                        world.entity_mut(entity).insert((
                            Velocity::default(),
                            Sprite::default(),
                            Mesh {
                                vertex_count: 1000,
                                triangle_count: 500,
                            },
                        ));
                    }

                    let alloc_count = get_allocation_count();
                    black_box((world, alloc_count));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark内存重用（实体池）
fn bench_entity_pool_reuse(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_pool_reuse");

    let iterations = 1000;

    group.bench_function("spawn_despawn_reuse", |b| {
        b.iter(|| {
            reset_memory_stats();
            let mut world = World::new();

            // 第一轮：创建实体
            let entities: Vec<Entity> = (0..iterations)
                .map(|_| world.spawn((Transform::default(), Velocity::default())).id())
                .collect();

            // 第二轮：销毁并重新创建
            for entity in entities {
                world.despawn(entity);
            }

            for _ in 0..iterations {
                world.spawn((Transform::default(), Velocity::default()));
            }

            let alloc_count = get_allocation_count();
            black_box((world, alloc_count));
        });
    });

    group.finish();
}

/// Benchmark不同组件布局的内存效率
fn bench_component_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_layout");

    // 小组件
    group.bench_function("small_components", |b| {
        #[derive(Component)]
        struct SmallComponent(u8, u8, u8);

        b.iter(|| {
            reset_memory_stats();
            let mut world = World::new();

            for _ in 0..10000 {
                world.spawn(SmallComponent(1, 2, 3));
            }

            let bytes_allocated = get_bytes_allocated();
            black_box((world, bytes_allocated));
        });
    });

    // 大组件
    group.bench_function("large_components", |b| {
        #[derive(Component)]
        struct LargeComponent([f32; 32]);

        b.iter(|| {
            reset_memory_stats();
            let mut world = World::new();

            for _ in 0..10000 {
                world.spawn(LargeComponent([0.0; 32]));
            }

            let bytes_allocated = get_bytes_allocated();
            black_box((world, bytes_allocated));
        });
    });

    // 混合组件
    group.bench_function("mixed_components", |b| {
        b.iter(|| {
            reset_memory_stats();
            let mut world = World::new();

            for i in 0..10000 {
                if i % 2 == 0 {
                    world.spawn((Transform::default(), Velocity::default()));
                } else {
                    world.spawn((
                        Transform::default(),
                        Mesh {
                            vertex_count: 1000,
                            triangle_count: 500,
                        },
                    ));
                }
            }

            let bytes_allocated = get_bytes_allocated();
            black_box((world, bytes_allocated));
        });
    });

    group.finish();
}

/// Benchmark查询操作的内存访问模式
fn bench_query_memory_access(c: &mut Criterion) {
    let mut group = c.benchmark_group("query_memory_access");

    for entity_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            entity_count,
            |b, &count| {
                let mut world = World::new();

                for _ in 0..count {
                    world.spawn((
                        Transform {
                            pos: Vec3::new(0.0, 0.0, 0.0),
                            rot: Quat::IDENTITY,
                            scale: Vec3::ONE,
                        },
                        Velocity {
                            lin: Vec3::new(1.0, 0.0, 0.0),
                            ang: Vec3::ZERO,
                        },
                    ));
                }

                b.iter(|| {
                    let mut query = world.query::<(&mut Transform, &Velocity)>();
                    let mut sum = Vec3::ZERO;

                    for (mut transform, velocity) in query.iter_mut(&mut world) {
                        transform.pos += velocity.lin;
                        sum += transform.pos;
                    }

                    black_box(sum);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark批量操作的内存效率
fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    for entity_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            entity_count,
            |b, &count| {
                b.iter(|| {
                    reset_memory_stats();
                    let mut world = World::new();

                    // 批量创建
                    world.spawn_batch(
                        (0..count).map(|_| {
                            (Transform::default(), Velocity::default(), Sprite::default())
                        }),
                    );

                    let alloc_count = get_allocation_count();
                    black_box((world, alloc_count));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark资源管理的内存使用
fn bench_resource_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("resource_memory");

    for resource_size_kb in [1, 10, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(resource_size_kb),
            resource_size_kb,
            |b, &size_kb| {
                #[derive(Resource)]
                struct LargeResource {
                    data: Vec<u8>,
                }

                b.iter(|| {
                    reset_memory_stats();
                    let mut world = World::new();

                    let data_size = size_kb * 1024;
                    world.insert_resource(LargeResource {
                        data: vec![0u8; data_size],
                    });

                    let bytes_allocated = get_bytes_allocated();
                    black_box((world, bytes_allocated));
                });
            },
        );
    }

    group.finish();
}

/// Benchmark内存碎片
fn bench_memory_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_fragmentation");

    group.bench_function("fragmented_allocation", |b| {
        b.iter(|| {
            reset_memory_stats();
            let mut world = World::new();

            // 创建不同大小的分配
            for i in 0..1000 {
                if i % 3 == 0 {
                    world.spawn((Transform::default(),));
                } else if i % 3 == 1 {
                    world.spawn((Transform::default(), Velocity::default()));
                } else {
                    world.spawn((
                        Transform::default(),
                        Velocity::default(),
                        Sprite::default(),
                        Mesh {
                            vertex_count: 1000,
                            triangle_count: 500,
                        },
                    ));
                }
            }

            let alloc_count = get_allocation_count();
            let bytes_allocated = get_bytes_allocated();
            black_box((world, alloc_count, bytes_allocated));
        });
    });

    group.bench_function("uniform_allocation", |b| {
        b.iter(|| {
            reset_memory_stats();
            let mut world = World::new();

            // 创建相同大小的分配
            for _ in 0..1000 {
                world.spawn((Transform::default(), Velocity::default(), Sprite::default()));
            }

            let alloc_count = get_allocation_count();
            let bytes_allocated = get_bytes_allocated();
            black_box((world, alloc_count, bytes_allocated));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_entity_memory_allocation,
    bench_component_memory_allocation,
    bench_entity_pool_reuse,
    bench_component_layout,
    bench_query_memory_access,
    bench_batch_operations,
    bench_resource_memory,
    bench_memory_fragmentation
);
criterion_main!(benches);
