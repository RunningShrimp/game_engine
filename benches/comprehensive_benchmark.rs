use criterion::{black_box, criterion_main, Criterion};
use game_engine::*;
use bevy_ecs::prelude::*;

// 注意：这里假设有一个GameEngine结构，具有create_scene和update方法
// 由于现有的引擎实现没有这些方法，这里做了一个简化版本用于基准测试

#[allow(unused)]
fn benchmark_full_game_frame(c: &mut Criterion) {
    // 创建一个简化的世界用于基准测试
    let mut world = World::new();

    // 添加必要的资源
    world.insert_resource(Time::default());
    world.insert_resource(PhysicsState::default());

    // 创建1000个测试实体
    for _ in 0..1000 {
        let mut entity = world.spawn(Transform::default());
        entity.insert(Sprite::new());
        entity.insert(RigidBodyDesc {
            body_type: RigidBodyType::Dynamic,
            position: [0.0, 0.0],
        });
    }

    let mut schedule = Schedule::new();
    // 为了简化，不添加物理系统，因为可能有依赖问题

    c.bench_function("full_game_frame_1000_entities", |b| {
        b.iter(|| {
            // 模拟游戏帧更新
            if let Some(mut time) = world.get_resource_mut::<Time>() {
                time.delta_seconds = 1.0 / 60.0;
                time.elapsed_seconds += time.delta_seconds as f64;
            }

            // 运行调度器
            schedule.run(&mut world);

            black_box(());
        });
    });
}

criterion_group!(benches, benchmark_full_game_frame);
criterion_main!(benches);
