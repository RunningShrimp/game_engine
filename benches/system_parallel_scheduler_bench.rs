// System并行调度器性能基准测试
//
// 验证优化后的并行调度相比串行调度的性能提升
//
// 运行: cargo bench --bench system_parallel_scheduler_bench

use bevy_ecs::prelude::*;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use game_engine::core::system_parallel_scheduler::{
    ParallelSchedulerConfig, ResourceAccess, SmartParallelScheduler, SystemCharacteristics,
    WorkStealingExecutor,
};

/// 创建测试World
fn create_test_world() -> World {
    World::new()
}

/// 创建模拟系统函数
fn create_mock_system(_system_id: usize) -> impl Fn(&mut World) + Send + Sync {
    move |_world: &mut World| {
        // 模拟工作负载
        let mut sum = 0u64;
        for i in 0..1000 {
            sum = sum.wrapping_add(i as u64);
        }
        // 防止编译器优化掉
        std::hint::black_box(sum);
    }
}

/// 基准测试场景1: 串行执行 vs 并行执行
fn bench_serial_vs_parallel(c: &mut Criterion) {
    let mut group = c.benchmark_group("serial_vs_parallel");

    for system_count in [4, 8, 16, 32].iter() {
        // 串行执行
        group.bench_with_input(
            BenchmarkId::new("serial_execution", system_count),
            system_count,
            |bencher, count| {
                bencher.iter(|| {
                    let mut world = create_test_world();
                    for i in 0..*count {
                        let system_fn = create_mock_system(i);
                        system_fn(&mut world);
                    }
                });
            },
        );

        // 并行执行（无冲突）
        group.bench_with_input(
            BenchmarkId::new("parallel_no_conflicts", system_count),
            system_count,
            |bencher, count| {
                let mut scheduler = SmartParallelScheduler::new(ParallelSchedulerConfig {
                    max_parallelism: 0,
                    enable_work_stealing: true,
                    enable_dynamic_parallelism: true,
                    min_parallel_threshold: 2,
                    history_window_size: 10,
                });

                // 注册只读系统（无冲突）
                for i in 0..*count {
                    scheduler.register_system(SystemCharacteristics {
                        name: format!("system_{}", i),
                        resource_access: vec![ResourceAccess::Read(format!("Resource_{}", i))],
                        expected_duration_us: 100,
                        cpu_bound: true,
                        parallel_safe: true,
                    });
                }

                bencher.iter(|| {
                    let mut world = create_test_world();
                    let systems: Vec<(String, _)> = (0..*count)
                        .map(|i| (format!("system_{}", i), create_mock_system(i)))
                        .collect();

                    let results = scheduler.schedule_and_execute(systems, &mut world);
                    std::hint::black_box(results);
                });
            },
        );

        // 并行执行（有冲突）
        group.bench_with_input(
            BenchmarkId::new("parallel_with_conflicts", system_count),
            system_count,
            |bencher, count| {
                let mut scheduler = SmartParallelScheduler::new(ParallelSchedulerConfig {
                    max_parallelism: 0,
                    enable_work_stealing: true,
                    enable_dynamic_parallelism: true,
                    min_parallel_threshold: 2,
                    history_window_size: 10,
                });

                // 注册混合读写系统（有冲突）
                for i in 0..*count {
                    let resource_idx = i % 4; // 4个共享资源
                    scheduler.register_system(SystemCharacteristics {
                        name: format!("system_{}", i),
                        resource_access: vec![if i % 2 == 0 {
                            ResourceAccess::Write(format!("Resource_{}", resource_idx))
                        } else {
                            ResourceAccess::Read(format!("Resource_{}", resource_idx))
                        }],
                        expected_duration_us: 100,
                        cpu_bound: true,
                        parallel_safe: false,
                    });
                }

                bencher.iter(|| {
                    let mut world = create_test_world();
                    let systems: Vec<(String, _)> = (0..*count)
                        .map(|i| (format!("system_{}", i), create_mock_system(i)))
                        .collect();

                    let results = scheduler.schedule_and_execute(systems, &mut world);
                    std::hint::black_box(results);
                });
            },
        );
    }

    group.finish();
}

/// 基准测试场景2: 冲突检测性能
fn bench_conflict_detection(c: &mut Criterion) {
    let mut group = c.benchmark_group("conflict_detection");

    for system_count in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("analyze_conflicts", system_count),
            system_count,
            |bencher, count| {
                let mut scheduler = SmartParallelScheduler::new(ParallelSchedulerConfig::default());

                // 注册系统
                for i in 0..*count {
                    scheduler.register_system(SystemCharacteristics {
                        name: format!("system_{}", i),
                        resource_access: vec![
                            ResourceAccess::Read("Transform".to_string()),
                            ResourceAccess::Write("Velocity".to_string()),
                        ],
                        expected_duration_us: 100,
                        cpu_bound: true,
                        parallel_safe: false,
                    });
                }

                bencher.iter(|| {
                    let system_names: Vec<String> =
                        (0..*count).map(|i| format!("system_{}", i)).collect();

                    for (i, name_a) in system_names.iter().enumerate() {
                        for name_b in system_names.iter().skip(i + 1) {
                            let has_conflict = scheduler.analyze_conflicts(name_a, name_b);
                            std::hint::black_box(has_conflict);
                        }
                    }
                });
            },
        );
    }

    group.finish();
}

/// 基准测试场景3: 动态并行度调整
fn bench_dynamic_parallelism(c: &mut Criterion) {
    let mut group = c.benchmark_group("dynamic_parallelism");

    // 固定并行度
    group.bench_function("fixed_parallelism", |bencher| {
        let config = ParallelSchedulerConfig {
            max_parallelism: 8,
            enable_work_stealing: true,
            enable_dynamic_parallelism: false, // 禁用动态调整
            min_parallel_threshold: 2,
            history_window_size: 10,
        };
        let executor = WorkStealingExecutor::new(config);

        bencher.iter(|| {
            let mut world = create_test_world();
            let systems: Vec<(String, _)> =
                (0..16).map(|i| (format!("system_{}", i), create_mock_system(i))).collect();
            let results = executor.execute_systems(systems, &mut world);
            std::hint::black_box(results);
        });
    });

    // 动态并行度
    group.bench_function("dynamic_parallelism", |bencher| {
        let config = ParallelSchedulerConfig {
            max_parallelism: 0, // 自动检测
            enable_work_stealing: true,
            enable_dynamic_parallelism: true, // 启用动态调整
            min_parallel_threshold: 2,
            history_window_size: 10,
        };
        let executor = WorkStealingExecutor::new(config);

        bencher.iter(|| {
            let mut world = create_test_world();
            let systems: Vec<(String, _)> =
                (0..16).map(|i| (format!("system_{}", i), create_mock_system(i))).collect();
            let results = executor.execute_systems(systems, &mut world);
            std::hint::black_box(results);
        });
    });

    group.finish();
}

/// 基准测试场景4: Work-stealing vs 固定分区
fn bench_work_stealing(c: &mut Criterion) {
    let mut group = c.benchmark_group("work_stealing");

    for system_count in [8, 16, 32].iter() {
        // 无Work-stealing（使用Rayon的默认调度）
        group.bench_with_input(
            BenchmarkId::new("no_work_stealing", system_count),
            system_count,
            |bencher, count| {
                let config = ParallelSchedulerConfig {
                    max_parallelism: 0,
                    enable_work_stealing: false,
                    enable_dynamic_parallelism: true,
                    min_parallel_threshold: 2,
                    history_window_size: 10,
                };
                let executor = WorkStealingExecutor::new(config);

                bencher.iter(|| {
                    let mut world = create_test_world();
                    let systems: Vec<(String, _)> = (0..*count)
                        .map(|i| (format!("system_{}", i), create_mock_system(i)))
                        .collect();
                    let results = executor.execute_systems(systems, &mut world);
                    std::hint::black_box(results);
                });
            },
        );

        // 启用Work-stealing
        group.bench_with_input(
            BenchmarkId::new("with_work_stealing", system_count),
            system_count,
            |bencher, count| {
                let config = ParallelSchedulerConfig {
                    max_parallelism: 0,
                    enable_work_stealing: true,
                    enable_dynamic_parallelism: true,
                    min_parallel_threshold: 2,
                    history_window_size: 10,
                };
                let executor = WorkStealingExecutor::new(config);

                bencher.iter(|| {
                    let mut world = create_test_world();
                    let systems: Vec<(String, _)> = (0..*count)
                        .map(|i| (format!("system_{}", i), create_mock_system(i)))
                        .collect();
                    let results = executor.execute_systems(systems, &mut world);
                    std::hint::black_box(results);
                });
            },
        );
    }

    group.finish();
}

/// 基准测试场景5: 综合性能（实际游戏场景）
fn bench_real_world_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world");

    // 典型游戏帧的系统调度
    group.bench_function("typical_game_frame", |bencher| {
        let mut scheduler = SmartParallelScheduler::new(ParallelSchedulerConfig {
            max_parallelism: 0,
            enable_work_stealing: true,
            enable_dynamic_parallelism: true,
            min_parallel_threshold: 4,
            history_window_size: 10,
        });

        // 注册典型的游戏系统
        // 物理系统（写Transform/Velocity）
        scheduler.register_system(SystemCharacteristics {
            name: "physics_system".to_string(),
            resource_access: vec![
                ResourceAccess::Write("Transform".to_string()),
                ResourceAccess::Write("Velocity".to_string()),
            ],
            expected_duration_us: 500,
            cpu_bound: true,
            parallel_safe: false,
        });

        // AI系统（读Transform/Velocity，写AIState）
        scheduler.register_system(SystemCharacteristics {
            name: "ai_system".to_string(),
            resource_access: vec![
                ResourceAccess::Read("Transform".to_string()),
                ResourceAccess::Read("Velocity".to_string()),
                ResourceAccess::Write("AIState".to_string()),
            ],
            expected_duration_us: 300,
            cpu_bound: true,
            parallel_safe: false,
        });

        // 渲染系统（读Transform，写RenderData）
        scheduler.register_system(SystemCharacteristics {
            name: "render_system".to_string(),
            resource_access: vec![
                ResourceAccess::Read("Transform".to_string()),
                ResourceAccess::Write("RenderData".to_string()),
            ],
            expected_duration_us: 800,
            cpu_bound: false,
            parallel_safe: false,
        });

        // 音频系统（读Transform，写AudioState）
        scheduler.register_system(SystemCharacteristics {
            name: "audio_system".to_string(),
            resource_access: vec![
                ResourceAccess::Read("Transform".to_string()),
                ResourceAccess::Write("AudioState".to_string()),
            ],
            expected_duration_us: 100,
            cpu_bound: false,
            parallel_safe: true,
        });

        // 输入系统（写InputState）
        scheduler.register_system(SystemCharacteristics {
            name: "input_system".to_string(),
            resource_access: vec![ResourceAccess::Write("InputState".to_string())],
            expected_duration_us: 50,
            cpu_bound: false,
            parallel_safe: false,
        });

        // 动画系统（读Transform，写Animation）
        scheduler.register_system(SystemCharacteristics {
            name: "animation_system".to_string(),
            resource_access: vec![
                ResourceAccess::Read("Transform".to_string()),
                ResourceAccess::Write("Animation".to_string()),
            ],
            expected_duration_us: 200,
            cpu_bound: true,
            parallel_safe: true,
        });

        bencher.iter(|| {
            let mut world = create_test_world();
            let systems: Vec<(String, _)> = vec![
                ("physics_system".to_string(), create_mock_system(0)),
                ("ai_system".to_string(), create_mock_system(1)),
                ("render_system".to_string(), create_mock_system(2)),
                ("audio_system".to_string(), create_mock_system(3)),
                ("input_system".to_string(), create_mock_system(4)),
                ("animation_system".to_string(), create_mock_system(5)),
            ];

            let results = scheduler.schedule_and_execute(systems, &mut world);
            std::hint::black_box(results);
        });
    });

    group.finish();
}

/// 基准测试场景6: 负载均衡性能
fn bench_load_balance(c: &mut Criterion) {
    let mut group = c.benchmark_group("load_balance");

    // 不均衡负载（某些系统执行时间长）
    group.bench_function("unbalanced_load", |bencher| {
        let mut scheduler = SmartParallelScheduler::new(ParallelSchedulerConfig::default());

        for i in 0..8 {
            let duration = match i {
                0 | 1 => 1000, // 2个慢系统
                2..=4 => 500,  // 3个中等系统
                _ => 100,      // 3个快系统
            };

            scheduler.register_system(SystemCharacteristics {
                name: format!("system_{}", i),
                resource_access: vec![ResourceAccess::Read(format!("Resource_{}", i))],
                expected_duration_us: duration,
                cpu_bound: true,
                parallel_safe: true,
            });
        }

        bencher.iter(|| {
            let mut world = create_test_world();
            let systems: Vec<(String, _)> =
                (0..8).map(|i| (format!("system_{}", i), create_mock_system(i))).collect();

            let results = scheduler.schedule_and_execute(systems, &mut world);
            std::hint::black_box(results);
        });
    });

    // 均衡负载（所有系统执行时间相近）
    group.bench_function("balanced_load", |bencher| {
        let mut scheduler = SmartParallelScheduler::new(ParallelSchedulerConfig::default());

        for i in 0..8 {
            scheduler.register_system(SystemCharacteristics {
                name: format!("system_{}", i),
                resource_access: vec![ResourceAccess::Read(format!("Resource_{}", i))],
                expected_duration_us: 500, // 相同执行时间
                cpu_bound: true,
                parallel_safe: true,
            });
        }

        bencher.iter(|| {
            let mut world = create_test_world();
            let systems: Vec<(String, _)> =
                (0..8).map(|i| (format!("system_{}", i), create_mock_system(i))).collect();

            let results = scheduler.schedule_and_execute(systems, &mut world);
            std::hint::black_box(results);
        });
    });

    group.finish();
}

criterion_group!(
    name = system_parallel_scheduler_benches;
    config = Criterion::default().sample_size(100);
    targets = bench_serial_vs_parallel,
             bench_conflict_detection,
             bench_dynamic_parallelism,
             bench_work_stealing,
             bench_real_world_scenario,
             bench_load_balance
);

criterion_main!(system_parallel_scheduler_benches);
