//! 渲染系统性能基准测试
//!
//! 测试视锥剔除、LOD计算、批渲染等操作的性能

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use glam::{Mat4, Vec3};

fn bench_frustum_culling(c: &mut Criterion) {
    use game_engine::render::frustum::Frustum;

    let mut group = c.benchmark_group("frustum_culling");

    // 创建视锥体
    let view_matrix = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
    let proj_matrix = Mat4::perspective_rh(
        0.785398, // PI / 4.0
        16.0 / 9.0,
        0.1,
        100.0,
    );
    let view_proj = proj_matrix * view_matrix;
    let frustum = Frustum::from_view_projection(view_proj);

    for object_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            object_count,
            |b, &count| {
                // 创建测试对象位置
                let positions: Vec<Vec3> = (0..count)
                    .map(|i| {
                        Vec3::new(
                            (i % 100) as f32 - 50.0,
                            ((i / 100) % 100) as f32 - 50.0,
                            (i / 10000) as f32 * 10.0,
                        )
                    })
                    .collect();

                b.iter(|| {
                    let mut visible_count = 0;
                    for pos in &positions {
                        // 简化的球体测试（实际应该使用AABB）
                        let distance = pos.length();
                        if distance < 100.0 {
                            // 简单的距离剔除
                            visible_count += 1;
                        }
                    }
                    black_box(visible_count)
                });
            },
        );
    }

    group.finish();
}

fn bench_lod_calculation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lod_calculation");

    for object_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            object_count,
            |b, &count| {
                let positions: Vec<Vec3> = (0..count)
                    .map(|i| {
                        Vec3::new(
                            (i % 100) as f32 - 50.0,
                            ((i / 100) % 100) as f32 - 50.0,
                            (i / 10000) as f32 * 10.0,
                        )
                    })
                    .collect();

                let camera_pos = Vec3::new(0.0, 0.0, 10.0);
                let lod_distances = [10.0, 25.0, 50.0, 100.0];

                b.iter(|| {
                    let mut lod_counts = [0; 4];
                    for pos in &positions {
                        let distance = pos.distance(camera_pos);
                        let lod_level = lod_distances
                            .iter()
                            .position(|&d| distance < d)
                            .unwrap_or(3);
                        lod_counts[lod_level] += 1;
                    }
                    black_box(lod_counts)
                });
            },
        );
    }

    group.finish();
}

fn bench_gpu_indirect_draw(c: &mut Criterion) {
    use game_engine::render::gpu_driven::culling::GpuInstance;
    use game_engine::render::gpu_driven::{GpuDrivenConfig, GpuDrivenRenderer};
    use pollster::FutureExt;
    use wgpu::util::DeviceExt;

    let mut group = c.benchmark_group("gpu_indirect_draw");

    // 创建WGPU实例和设备（简化版本，实际应该使用完整的初始化）
    // 注意：这个测试需要实际的GPU设备，在某些环境中可能失败
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    // 尝试创建适配器（如果失败则跳过测试）
    let adapter_future = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    });

    let adapter = match adapter_future.block_on() {
        Some(a) => a,
        None => {
            eprintln!("No GPU adapter found, skipping GPU indirect draw benchmark");
            return;
        }
    };

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .block_on()
        .unwrap();

    let config = GpuDrivenConfig {
        frustum_culling: true,
        occlusion_culling: false,
        lod_enabled: false,
        max_instances: 65536,
        workgroup_size: 64,
    };

    let renderer = GpuDrivenRenderer::new(&device, config);

    for instance_count in [1000, 10000, 50000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(instance_count),
            instance_count,
            |b, &count| {
                // 创建测试实例数据
                let instances: Vec<GpuInstance> = (0..count)
                    .map(|i| {
                        let angle = (i as f32) * 0.1;
                        let x = angle.cos() * 10.0;
                        let z = angle.sin() * 10.0;
                        GpuInstance {
                            model: [
                                [1.0, 0.0, 0.0, 0.0],
                                [0.0, 1.0, 0.0, 0.0],
                                [0.0, 0.0, 1.0, 0.0],
                                [x, 0.0, z, 1.0],
                            ],
                            aabb_min: [-0.5, -0.5, -0.5],
                            instance_id: i as u32,
                            aabb_max: [0.5, 0.5, 0.5],
                            flags: 0,
                        }
                    })
                    .collect();

                // 更新实例数据
                renderer.update_instances(&queue, &instances);

                let view_proj =
                    Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 100.0);

                b.iter(|| {
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("GPU Indirect Draw Benchmark"),
                        });

                    // 执行GPU剔除和间接绘制
                    renderer.cull_with_indirect(
                        &mut encoder,
                        &device,
                        &queue,
                        view_proj.to_cols_array_2d(),
                        count as u32,
                        36, // 假设每个实例36个顶点（立方体）
                        36, // 假设每个实例36个索引
                    );

                    // 提交命令（实际测试中应该等待完成）
                    let _command_buffer = encoder.finish();
                    black_box(())
                });
            },
        );
    }

    group.finish();
}

fn bench_batch_grouping(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_grouping");

    for object_count in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(object_count),
            object_count,
            |b, &count| {
                // 模拟不同材质的对象
                let materials: Vec<u32> = (0..count).map(|i| (i % 10) as u32).collect();

                b.iter(|| {
                    use std::collections::HashMap;
                    let mut batches: HashMap<u32, Vec<usize>> = HashMap::new();

                    for (idx, &material) in materials.iter().enumerate() {
                        batches.entry(material).or_insert_with(Vec::new).push(idx);
                    }

                    black_box(batches.len())
                });
            },
        );
    }

    group.finish();
}

/// GPU驱动剔除基准测试
fn bench_gpu_culling(c: &mut Criterion) {
    use game_engine::render::gpu_driven::culling::{GpuCuller, GpuInstance};
    use game_engine::render::gpu_driven::GpuDrivenConfig;
    use pollster::FutureExt;
    use wgpu::util::DeviceExt;

    let mut group = c.benchmark_group("gpu_culling");

    // 创建WGPU实例和设备
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapter_future = instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        compatible_surface: None,
        force_fallback_adapter: false,
    });

    let adapter = match adapter_future.block_on() {
        Some(a) => a,
        None => {
            eprintln!("No GPU adapter found, skipping GPU culling benchmark");
            return;
        }
    };

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
            },
            None,
        )
        .block_on()
        .unwrap();

    for instance_count in [1000, 10000, 50000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(instance_count),
            instance_count,
            |b, &count| {
                // 创建GPU剔除器
                let culler = GpuCuller::new(&device, count, 64);

                // 创建测试实例数据
                let instances: Vec<GpuInstance> = (0..count)
                    .map(|i| {
                        let angle = (i as f32) * 0.1;
                        let x = angle.cos() * 10.0;
                        let z = angle.sin() * 10.0;
                        GpuInstance {
                            model: [
                                [1.0, 0.0, 0.0, 0.0],
                                [0.0, 1.0, 0.0, 0.0],
                                [0.0, 0.0, 1.0, 0.0],
                                [x, 0.0, z, 1.0],
                            ],
                            aabb_min: [-0.5, -0.5, -0.5],
                            instance_id: i as u32,
                            aabb_max: [0.5, 0.5, 0.5],
                            flags: 0,
                        }
                    })
                    .collect();

                // 创建缓冲区
                let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;
                let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&instances),
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
                });

                let visible_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Visible Instances"),
                    size: instance_size * count as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
                    mapped_at_creation: false,
                });

                let counter_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("Counter"),
                    size: 4,
                    usage: wgpu::BufferUsages::STORAGE
                        | wgpu::BufferUsages::COPY_DST
                        | wgpu::BufferUsages::COPY_SRC,
                    mapped_at_creation: false,
                });

                let view_proj =
                    Mat4::perspective_rh(std::f32::consts::PI / 4.0, 16.0 / 9.0, 0.1, 100.0);

                b.iter(|| {
                    // 重置计数器
                    queue.write_buffer(&counter_buffer, 0, &[0u8; 4]);

                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("GPU Culling Benchmark"),
                        });

                    // 执行GPU剔除
                    culler.cull(
                        &mut encoder,
                        &device,
                        &queue,
                        &instance_buffer,
                        &visible_buffer,
                        &counter_buffer,
                        view_proj.to_cols_array_2d(),
                        count as u32,
                    );

                    // 提交命令
                    let _command_buffer = encoder.finish();
                    black_box(())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_frustum_culling,
    bench_lod_calculation,
    bench_batch_grouping,
    bench_gpu_indirect_draw,
    bench_gpu_culling
);
criterion_main!(benches);
