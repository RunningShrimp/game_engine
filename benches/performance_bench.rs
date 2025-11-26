use criterion::{black_box, criterion_group, criterion_main, Criterion};
use game_engine::ecs::*;
use game_engine::performance::*;

fn bench_ecs_spawn(c: &mut Criterion) {
    c.bench_function("ecs_spawn_1000_entities", |b| {
        b.iter(|| {
            let mut world = World::default();
            for _ in 0..1000 {
                world.spawn((
                    Transform::default(),
                    Sprite {
                        color: [1.0, 1.0, 1.0, 1.0],
                        size: [10.0, 10.0],
                        texture: None,
                    },
                ));
            }
            black_box(world);
        });
    });
}

fn bench_batch_rendering(c: &mut Criterion) {
    c.bench_function("batch_render_1000_sprites", |b| {
        let mut batch_renderer = BatchRenderer::new();
        
        b.iter(|| {
            for i in 0..1000 {
                batch_renderer.add_sprite(
                    [i as f32, 0.0],
                    [10.0, 10.0],
                    [1.0, 1.0, 1.0, 1.0],
                    None,
                );
            }
            black_box(&batch_renderer);
        });
    });
}

fn bench_object_pool(c: &mut Criterion) {
    c.bench_function("object_pool_acquire_release", |b| {
        let pool = ObjectPool::<Vec<u8>>::new(100, || Vec::with_capacity(1024));
        
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
        let profiler = Profiler::new();
        
        b.iter(|| {
            profiler.begin_frame();
            profiler.begin_section("test");
            // 模拟一些工作
            for _ in 0..100 {
                black_box(1 + 1);
            }
            profiler.end_section("test");
            profiler.end_frame();
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
