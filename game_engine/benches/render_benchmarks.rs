// 渲染系统性能基准测试
//
// 测试渲染管线、批处理、视锥剔除等核心渲染功能

use bevy_ecs::prelude::*;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use game_engine::ecs::{Mesh, Transform};
use game_engine::render::frustum::Frustum;
use glam::{Mat4, Quat, Vec3};
use std::f32::consts::PI;

/// 创建测试用网格数据
fn create_test_mesh() -> Mesh {
    Mesh {
        vertex_count: 1000,
        triangle_count: 500,
    }
}

/// Benchmark视锥剔除性能
fn bench_frustum_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("frustum_culling");

    for entity_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(entity_count),
            entity_count,
            |b, &count| {
                let mut world = World::new();

                // 创建视锥体
                let projection = Mat4::perspective_rh(PI / 4.0, 16.0 / 9.0, 0.1, 100.0);
                let view =
                    Mat4::look_at_rh(Vec3::new(0.0, 5.0, 10.0), Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
                let view_projection = projection * view;
                let frustum = Frustum::from_view_projection(view_projection);

                // 创建实体
                for i in 0..count {
                    let angle = (i as f32 / count as f32) * PI * 2.0;
                    let radius = 10.0 + (i % 10) as f32;

                    world.spawn((
                        Transform {
                            pos: Vec3::new(
                                angle.cos() * radius,
                                (i % 5) as f32,
                                angle.sin() * radius,
                            ),
                            rot: Quat::IDENTITY,
                            scale: Vec3::ONE,
                        },
                        create_test_mesh(),
                    ));
                }

                b.iter(|| {
                    let mut visible_count = 0;
                    let frustum = black_box(frustum);

                    let mut query = world.query::<(&Transform, &Mesh)>();
                    for (transform, _mesh) in query.iter(&world) {
                        // 简化的边界球计算
                        let center = transform.pos;
                        let radius = 1.0;

                        if frustum.contains_sphere(center, radius) {
                            visible_count += 1;
                        }
                    }

                    black_box(visible_count);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark矩阵变换计算
fn bench_transform_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform_calculations");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let transforms: Vec<_> = (0..count)
                .map(|i| Transform {
                    pos: Vec3::new(i as f32, i as f32, i as f32),
                    rot: Quat::from_euler(glam::EulerRot::XYZ, 0.1, 0.2, 0.3),
                    scale: Vec3::ONE,
                })
                .collect();

            b.iter(|| {
                let mut matrices = Vec::with_capacity(count);
                for transform in &transforms {
                    let matrix = transform.calculate_matrix();
                    matrices.push(matrix);
                }
                black_box(matrices);
            });
        });
    }

    group.finish();
}

/// Benchmark渲染对象排序
fn bench_render_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("render_sorting");

    for object_count in [100, 1000, 5000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            object_count,
            |b, &count| {
                let mut world = World::new();

                // 创建不同材质和距离的对象
                for i in 0..count {
                    let material_id = i % 10;
                    let z_depth = (i as f32 * 0.1) % 100.0;

                    world.spawn((
                        Transform {
                            pos: Vec3::new(0.0, 0.0, z_depth),
                            rot: Quat::IDENTITY,
                            scale: Vec3::ONE,
                        },
                        create_test_mesh(),
                    ));

                    // 用于排序的元数据
                    world.insert_resource(material_id);
                }

                b.iter(|| {
                    let mut objects = Vec::new();

                    let mut query = world.query::<(Entity, &Transform)>();
                    for (entity, transform) in query.iter(&world) {
                        objects.push((entity, transform.pos.z));
                    }

                    // 按深度排序（不透明物体从远到近）
                    objects.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

                    black_box(objects);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark批处理性能
fn bench_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("batching");

    for draw_call_count in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(draw_call_count),
            draw_call_count,
            |b, &count| {
                b.iter(|| {
                    let mut batches: std::collections::HashMap<u32, Vec<Vec3>> =
                        std::collections::HashMap::new();

                    for i in 0..count {
                        let material_id = (i % 10) as u32;
                        let position = Vec3::new(i as f32, 0.0, 0.0);

                        batches.entry(material_id).or_default().push(position);
                    }

                    black_box(batches);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark MVP矩阵计算
fn bench_mvp_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("mvp_calculation");

    for count in [100, 1000, 10000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let projection = Mat4::perspective_rh(PI / 4.0, 16.0 / 9.0, 0.1, 100.0);
            let view = Mat4::look_at_rh(Vec3::new(0.0, 5.0, 10.0), Vec3::ZERO, Vec3::Y);

            let transforms: Vec<_> = (0..count)
                .map(|_| Transform {
                    pos: Vec3::new(0.0, 0.0, 0.0),
                    rot: Quat::IDENTITY,
                    scale: Vec3::ONE,
                })
                .collect();

            b.iter(|| {
                let mut mvps = Vec::with_capacity(count);
                for transform in &transforms {
                    let model = transform.calculate_matrix();
                    let mvp = projection * view * model;
                    mvps.push(mvp);
                }
                black_box(mvps);
            });
        });
    }

    group.finish();
}

/// Benchmark骨骼动画计算
fn bench_skeletal_animation(c: &mut Criterion) {
    let mut group = c.benchmark_group("skeletal_animation");

    for (bone_count, vertex_count) in [(10, 100), (50, 1000), (100, 5000)].iter() {
        group.bench_with_input(
            BenchmarkId::new(
                format!("bones_{}_vertices_{}", bone_count, vertex_count),
                (bone_count, vertex_count),
            ),
            &(bone_count, vertex_count),
            |b, &(bone_count, vertex_count)| {
                // 创建骨骼变换矩阵
                let bone_transforms: Vec<Mat4> = (0..*bone_count)
                    .map(|i| {
                        let angle = i as f32 * 0.1;
                        Mat4::from_rotation_y(angle)
                    })
                    .collect();

                // 创建顶点骨骼绑定数据（简化版）
                let vertex_bone_weights: Vec<Vec<(u32, f32)>> =
                    (0..*vertex_count).map(|_| vec![(0, 1.0)]).collect();

                b.iter(|| {
                    let mut skinned_vertices = Vec::with_capacity(*vertex_count);

                    for (i, weights) in vertex_bone_weights.iter().enumerate() {
                        let mut skinned_pos = Vec3::ZERO;

                        for &(bone_index, weight) in weights {
                            let bone_transform = bone_transforms[bone_index as usize];
                            let vertex_pos = Vec3::new(i as f32, 0.0, 0.0);
                            skinned_pos += (bone_transform.transform_point3(vertex_pos)) * weight;
                        }

                        skinned_vertices.push(skinned_pos);
                    }

                    black_box(skinned_vertices);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_frustum_culling,
    bench_transform_calculations,
    bench_render_sorting,
    bench_batching,
    bench_mvp_calculation,
    bench_skeletal_animation
);
criterion_main!(benches);
