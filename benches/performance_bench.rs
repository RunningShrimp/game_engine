use std::hint::black_box;
use criterion::{criterion_group, criterion_main, Criterion};
use bevy_ecs::prelude::World;
use game_engine::ecs::*;
use game_engine::performance::*;
use game_engine::performance::batch_renderer::{BatchRenderer, BatchKey};

fn bench_ecs_spawn(c: &mut Criterion) {
    c.bench_function("ecs_spawn_1000_entities", |b| {
        b.iter(|| {
            let mut world = World::default();
            for _ in 0..1000 {
                world.spawn((
                    Transform::default(),
                    Sprite::default(),
                ));
            }
            black_box(world);
        });
    });
}

fn bench_batch_rendering(c: &mut Criterion) {
    c.bench_function("batch_render_1000_sprites", |b| {
        let mut batch_renderer = BatchRenderer::new(100);
        
        b.iter(|| {
            for i in 0..1000 {
                let key = BatchKey { material_id: 1, texture_id: 1, shader_id: 1 };
                let vertex_offset = i * 4;
                let index_offset = i * 6;
                let index_count = 6u32;
                batch_renderer.add_draw_call(key, vertex_offset, index_offset, index_count);
            }
            black_box(&batch_renderer);
        });
    });
}

fn bench_object_pool(c: &mut Criterion) {
    c.bench_function("object_pool_acquire_release", |b| {
        let mut pool = ObjectPool::<Vec<u8>>::new(|| Vec::with_capacity(1024), 100, 200);
        
        b.iter(|| {
            let mut objects = Vec::new();
            for _ in 0..100 {
                objects.push(pool.acquire());
            }
            black_box(objects);
        });
    });
}

fn bench_profiler(c: &mut Criterion) {
    c.bench_function("profiler_begin_end", |b| {
        let mut profiler = Profiler::new();
        
        b.iter(|| {
            profiler.begin_scope("test");
            // 模拟一些工作
            for _ in 0..100 {
                black_box(1 + 1);
            }
            profiler.end_scope();
        });
    });
}

criterion_group!(
    benches,
    bench_ecs_spawn,
    bench_batch_rendering,
    bench_object_pool,
    bench_profiler
);
criterion_main!(benches);
