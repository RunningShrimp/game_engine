//  寻路系统性能基准测试
//
//  测试A*寻路算法和并行寻路服务的性能

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use game_engine::ai::pathfinding::{NavigationMesh, PathfindingService};
use game_engine::ai::{AsyncPathfindingService, ParallelPathfindingService};
use glam::Vec3;
use tokio::runtime::Runtime;

fn bench_single_pathfinding(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_pathfinding");

    // 创建测试导航网格
    let mut mesh = NavigationMesh::new();

    // 创建网格节点（10x10x10网格）
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                let node_id = (x * 100 + y * 10 + z) as u32;
                PathfindingService::add_node_to_mesh(
                    &mut mesh,
                    Vec3::new(x as f32, y as f32, z as f32),
                    true,
                );
            }
        }
    }

    // 添加连接（每个节点连接到相邻节点）
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                let node_id = (x * 100 + y * 10 + z) as u32;

                // 连接到相邻节点
                if x > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 100,
                        1.0,
                    );
                }
                if y > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 10,
                        1.0,
                    );
                }
                if z > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 1,
                        1.0,
                    );
                }
            }
        }
    }

    let start = Vec3::new(0.0, 0.0, 0.0);
    let end = Vec3::new(9.0, 9.0, 9.0);

    group.bench_function("a_star", |b| {
        b.iter(|| {
            let path = PathfindingService::find_path(&mesh, start, end);
            black_box(path)
        });
    });

    group.finish();
}

fn bench_parallel_pathfinding(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_pathfinding");

    // 创建测试导航网格
    let mut mesh = NavigationMesh::new();

    // 创建网格节点（10x10x10网格）
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                PathfindingService::add_node_to_mesh(
                    &mut mesh,
                    Vec3::new(x as f32, y as f32, z as f32),
                    true,
                );
            }
        }
    }

    // 添加连接
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                let node_id = (x * 100 + y * 10 + z) as u32;

                if x > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 100,
                        1.0,
                    );
                }
                if y > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 10,
                        1.0,
                    );
                }
                if z > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 1,
                        1.0,
                    );
                }
            }
        }
    }

    for request_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(request_count),
            request_count,
            |b, &count| {
                // 准备寻路请求
                let requests: Vec<(Vec3, Vec3)> = (0..count)
                    .map(|i| {
                        let start = Vec3::new(
                            (i % 10) as f32,
                            ((i / 10) % 10) as f32,
                            ((i / 100) % 10) as f32,
                        );
                        let end = Vec3::new(
                            ((i + 5) % 10) as f32,
                            (((i + 5) / 10) % 10) as f32,
                            (((i + 5) / 100) % 10) as f32,
                        );
                        (start, end)
                    })
                    .collect();

                // 使用异步协程寻路服务（推荐）
                let rt = Runtime::new().unwrap();
                let async_service = AsyncPathfindingService::new(mesh.clone(), 4);

                b.iter(|| {
                    // 使用异步批量寻路
                    rt.block_on(async {
                        let results = async_service.find_paths_batch(requests.clone()).await;
                        black_box(results)
                    })
                });
            },
        );
    }

    group.finish();
}

fn bench_parallel_vs_sequential(c: &mut Criterion) {
    let mut group = c.benchmark_group("parallel_vs_sequential");

    // 创建测试导航网格
    let mut mesh = NavigationMesh::new();

    // 创建网格节点（10x10x10网格）
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                PathfindingService::add_node_to_mesh(
                    &mut mesh,
                    Vec3::new(x as f32, y as f32, z as f32),
                    true,
                );
            }
        }
    }

    // 添加连接
    for x in 0..10 {
        for y in 0..10 {
            for z in 0..10 {
                let node_id = (x * 100 + y * 10 + z) as u32;

                if x > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 100,
                        1.0,
                    );
                }
                if y > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 10,
                        1.0,
                    );
                }
                if z > 0 {
                    PathfindingService::add_connection_to_mesh(
                        &mut mesh,
                        node_id,
                        node_id - 1,
                        1.0,
                    );
                }
            }
        }
    }

    let request_count = 100;
    let requests: Vec<(Vec3, Vec3)> = (0..request_count)
        .map(|i| {
            let start = Vec3::new(
                (i % 10) as f32,
                ((i / 10) % 10) as f32,
                ((i / 100) % 10) as f32,
            );
            let end = Vec3::new(
                ((i + 5) % 10) as f32,
                (((i + 5) / 10) % 10) as f32,
                (((i + 5) / 100) % 10) as f32,
            );
            (start, end)
        })
        .collect();

    // 顺序执行基准测试
    group.bench_function("sequential", |b| {
        b.iter(|| {
            let mut results = Vec::new();
            for (start, end) in &requests {
                let path = PathfindingService::find_path(&mesh, *start, *end);
                results.push(path);
            }
            black_box(results)
        });
    });

    // 并行执行基准测试（已弃用，保留用于对比）
    #[allow(deprecated)]
    group.bench_function("parallel_4_threads_deprecated", |b| {
        let parallel_service = ParallelPathfindingService::new(mesh.clone(), 4);
        b.iter(|| {
            let request_ids = parallel_service.submit_path_requests(requests.clone());
            let mut results = Vec::new();
            while results.len() < request_ids.len() {
                let batch = parallel_service.collect_results();
                results.extend(batch);
                if results.len() < request_ids.len() {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
            black_box(results)
        });
    });

    // 异步协程执行基准测试（推荐）
    group.bench_function("async_4_concurrent", |b| {
        let rt = Runtime::new().unwrap();
        let async_service = AsyncPathfindingService::new(mesh.clone(), 4);
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::new();
                for (start, end) in &requests {
                    let service = &async_service;
                    handles.push(async move { service.find_path(*start, *end).await });
                }
                let results: Vec<_> = futures::future::join_all(handles).await;
                black_box(results)
            })
        });
    });

    // 并行执行基准测试（8线程，已弃用，保留用于对比）
    #[allow(deprecated)]
    group.bench_function("parallel_8_threads_deprecated", |b| {
        let parallel_service = ParallelPathfindingService::new(mesh.clone(), 8);
        b.iter(|| {
            let request_ids = parallel_service.submit_path_requests(requests.clone());
            let mut results = Vec::new();
            while results.len() < request_ids.len() {
                let batch = parallel_service.collect_results();
                results.extend(batch);
                if results.len() < request_ids.len() {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
            black_box(results)
        });
    });

    // 异步协程执行基准测试（8并发，推荐）
    group.bench_function("async_8_concurrent", |b| {
        let rt = Runtime::new().unwrap();
        let async_service = AsyncPathfindingService::new(mesh.clone(), 8);
        b.iter(|| {
            rt.block_on(async {
                let mut handles = Vec::new();
                for (start, end) in &requests {
                    let service = &async_service;
                    handles.push(async move { service.find_path(*start, *end).await });
                }
                let results: Vec<_> = futures::future::join_all(handles).await;
                black_box(results)
            })
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_pathfinding,
    bench_parallel_pathfinding,
    bench_parallel_vs_sequential
);
criterion_main!(benches);
