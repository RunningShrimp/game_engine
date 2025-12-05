//! 物理系统性能基准测试
//!
//! 测试物理世界更新、碰撞检测等操作的性能

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

#[cfg(feature = "physics_2d")]
fn bench_physics_world_creation(c: &mut Criterion) {
    use rapier2d::prelude::*;

    let mut group = c.benchmark_group("physics_world_creation");

    for body_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(body_count),
            body_count,
            |b, &count| {
                b.iter(|| {
                    let mut physics_world = RigidBodySet::new();
                    let mut collider_set = ColliderSet::new();

                    for i in 0..count {
                        let rigid_body = RigidBodyBuilder::dynamic()
                            .translation(vector![i as f32 * 2.0, 0.0])
                            .build();
                        let handle = physics_world.insert(rigid_body);

                        let collider = ColliderBuilder::cuboid(0.5, 0.5).build();
                        collider_set.insert_with_parent(collider, handle, &mut physics_world);
                    }

                    black_box((physics_world, collider_set))
                });
            },
        );
    }

    group.finish();
}

#[cfg(feature = "physics_2d")]
fn bench_physics_step(c: &mut Criterion) {
    use rapier2d::prelude::DefaultBroadPhase;
    use rapier2d::prelude::*;

    let mut group = c.benchmark_group("physics_step");

    for body_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(body_count),
            body_count,
            |b, &count| {
                let mut physics_world = RigidBodySet::new();
                let mut collider_set = ColliderSet::new();
                let mut query_pipeline = QueryPipeline::new();

                // 创建地面
                let ground = RigidBodyBuilder::fixed()
                    .translation(vector![0.0, -10.0])
                    .build();
                let ground_handle = physics_world.insert(ground);
                let ground_collider = ColliderBuilder::cuboid(50.0, 1.0).build();
                collider_set.insert_with_parent(ground_collider, ground_handle, &mut physics_world);

                // 创建动态物体
                for i in 0..count {
                    let rigid_body = RigidBodyBuilder::dynamic()
                        .translation(vector![i as f32 * 2.0, 10.0])
                        .build();
                    let handle = physics_world.insert(rigid_body);

                    let collider = ColliderBuilder::cuboid(0.5, 0.5).build();
                    collider_set.insert_with_parent(collider, handle, &mut physics_world);
                }

                let gravity = vector![0.0, -9.81];
                let integration_parameters = IntegrationParameters::default();
                let mut island_manager = IslandManager::new();
                let mut broad_phase = DefaultBroadPhase::new();
                let mut narrow_phase = NarrowPhase::new();
                let mut impulse_joint_set = ImpulseJointSet::new();
                let mut multibody_joint_set = MultibodyJointSet::new();
                let mut ccd_solver = CCDSolver::new();
                let mut physics_pipeline = PhysicsPipeline::new();
                let physics_hooks = ();
                let event_handler = ();

                b.iter(|| {
                    physics_pipeline.step(
                        &gravity,
                        &integration_parameters,
                        &mut island_manager,
                        &mut broad_phase,
                        &mut narrow_phase,
                        &mut physics_world,
                        &mut collider_set,
                        &mut impulse_joint_set,
                        &mut multibody_joint_set,
                        &mut ccd_solver,
                        Some(&mut query_pipeline),
                        &physics_hooks,
                        &event_handler,
                    );
                });
            },
        );
    }

    group.finish();
}

#[cfg(not(feature = "physics_2d"))]
fn bench_physics_world_creation(_c: &mut Criterion) {}
#[cfg(not(feature = "physics_2d"))]
fn bench_physics_step(_c: &mut Criterion) {}

criterion_group!(benches, bench_physics_world_creation, bench_physics_step);
criterion_main!(benches);
