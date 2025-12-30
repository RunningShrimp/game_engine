// 状态同步优化性能基准测试
//
// 验证优化后的状态同步相比原始实现的性能提升
//
// 运行: cargo bench --bench state_sync_optimization_bench

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use glam::{Quat, Vec3};
use game_engine::network::state_sync_optimized::{
    DirtyFlag, EntityPriority, EntityState, NetworkQuality, OptimizedStateSyncManager,
};
use std::collections::HashSet;

/// 创建测试实体状态
fn create_test_state(entity_id: u64) -> EntityState {
    EntityState::new(
        Vec3::new(
            entity_id as f32,
            (entity_id * 2) as f32,
            (entity_id * 3) as f32,
        ),
        Quat::IDENTITY,
        Vec3::ONE,
        Vec3::ZERO,
    )
}

/// 基准测试场景1: 优化后的同步 vs 全量同步
fn bench_optimized_vs_full_sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimized_vs_full_sync");

    for entity_count in [10, 50, 100].iter() {
        // 优化后的同步（只同步脏组件）
        group.bench_with_input(
            BenchmarkId::new("optimized_dirty_tracking", entity_count),
            entity_count,
            |bencher, count| {
                let mut manager = OptimizedStateSyncManager::new(16, *count);

                // 注册实体
                for i in 0..*count {
                    let entity_id = i as u64;
                    let priority = match i % 5 {
                        0 => EntityPriority::Critical,
                        1 => EntityPriority::High,
                        2 => EntityPriority::Medium,
                        3 => EntityPriority::Low,
                        _ => EntityPriority::Background,
                    };
                    manager.register_entity(entity_id, priority);
                }

                // 更新实体状态（只标记部分组件为脏）
                for i in 0..*count {
                    let entity_id = i as u64;
                    let state = create_test_state(entity_id);
                    let mut dirty_flags = HashSet::new();
                    dirty_flags.insert(DirtyFlag::Position);
                    manager.update_entity_state(entity_id, state, dirty_flags).unwrap();
                }

                bencher.iter(|| {
                    let packet = manager.generate_optimized_sync_data(0);
                    std::hint::black_box(packet);
                });
            },
        );

        // 全量同步（所有组件都变更）
        group.bench_with_input(
            BenchmarkId::new("full_sync_all_dirty", entity_count),
            entity_count,
            |bencher, count| {
                let mut manager = OptimizedStateSyncManager::new(16, *count);

                // 注册实体
                for i in 0..*count {
                    let entity_id = i as u64;
                    let priority = match i % 5 {
                        0 => EntityPriority::Critical,
                        1 => EntityPriority::High,
                        2 => EntityPriority::Medium,
                        3 => EntityPriority::Low,
                        _ => EntityPriority::Background,
                    };
                    manager.register_entity(entity_id, priority);
                }

                // 更新实体状态（标记所有组件为脏）
                for i in 0..*count {
                    let entity_id = i as u64;
                    let state = create_test_state(entity_id);
                    let dirty_flags = DirtyFlag::all();
                    manager.update_entity_state(entity_id, state, dirty_flags).unwrap();
                }

                bencher.iter(|| {
                    let packet = manager.generate_optimized_sync_data(0);
                    std::hint::black_box(packet);
                });
            },
        );
    }

    group.finish();
}

/// 基准测试场景2: 优先级队列性能
fn bench_priority_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("priority_queue");

    for entity_count in [100, 500].iter() {
        // 无优先级（所有实体同等重要）
        group.bench_with_input(
            BenchmarkId::new("no_priority", entity_count),
            entity_count,
            |bencher, count| {
                let mut manager = OptimizedStateSyncManager::new(16, *count);

                // 所有实体都是中等优先级
                for i in 0..*count {
                    let entity_id = i as u64;
                    manager.register_entity(entity_id, EntityPriority::Medium);
                    let state = create_test_state(entity_id);
                    let mut dirty_flags = HashSet::new();
                    dirty_flags.insert(DirtyFlag::Position);
                    manager.update_entity_state(entity_id, state, dirty_flags).unwrap();
                }

                bencher.iter(|| {
                    let packet = manager.generate_optimized_sync_data(0);
                    std::hint::black_box(packet);
                });
            },
        );

        // 有优先级（混合优先级）
        group.bench_with_input(
            BenchmarkId::new("with_priority", entity_count),
            entity_count,
            |bencher, count| {
                let mut manager = OptimizedStateSyncManager::new(16, *count);

                // 混合优先级
                for i in 0..*count {
                    let entity_id = i as u64;
                    let priority = match i % 10 {
                        0 => EntityPriority::Critical,
                        1..=2 => EntityPriority::High,
                        3..=5 => EntityPriority::Medium,
                        6..=8 => EntityPriority::Low,
                        _ => EntityPriority::Background,
                    };
                    manager.register_entity(entity_id, priority);
                    let state = create_test_state(entity_id);
                    let mut dirty_flags = HashSet::new();
                    dirty_flags.insert(DirtyFlag::Position);
                    manager.update_entity_state(entity_id, state, dirty_flags).unwrap();
                }

                bencher.iter(|| {
                    let packet = manager.generate_optimized_sync_data(0);
                    std::hint::black_box(packet);
                });
            },
        );
    }

    group.finish();
}

/// 基准测试场景3: 脏追踪性能
fn bench_dirty_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("dirty_tracking");

    // 只有少量组件变更
    group.bench_function("minimal_changes", |bencher| {
        let mut manager = OptimizedStateSyncManager::new(16, 100);

        for i in 0..100 {
            manager.register_entity(i, EntityPriority::Medium);
        }

        // 只更新10%的实体
        for i in 0..10 {
            let state = create_test_state(i);
            let mut dirty_flags = HashSet::new();
            dirty_flags.insert(DirtyFlag::Position);
            manager.update_entity_state(i, state, dirty_flags).unwrap();
        }

        bencher.iter(|| {
            let packet = manager.generate_optimized_sync_data(0);
            std::hint::black_box(packet);
        });
    });

    // 中等变更
    group.bench_function("medium_changes", |bencher| {
        let mut manager = OptimizedStateSyncManager::new(16, 100);

        for i in 0..100 {
            manager.register_entity(i, EntityPriority::Medium);
        }

        // 更新50%的实体
        for i in 0..50 {
            let state = create_test_state(i);
            let mut dirty_flags = HashSet::new();
            dirty_flags.insert(DirtyFlag::Position);
            manager.update_entity_state(i, state, dirty_flags).unwrap();
        }

        bencher.iter(|| {
            let packet = manager.generate_optimized_sync_data(0);
            std::hint::black_box(packet);
        });
    });

    // 大量变更
    group.bench_function("heavy_changes", |bencher| {
        let mut manager = OptimizedStateSyncManager::new(16, 100);

        for i in 0..100 {
            manager.register_entity(i, EntityPriority::Medium);
        }

        // 更新所有实体
        for i in 0..100 {
            let state = create_test_state(i);
            let dirty_flags = DirtyFlag::all();
            manager.update_entity_state(i, state, dirty_flags).unwrap();
        }

        bencher.iter(|| {
            let packet = manager.generate_optimized_sync_data(0);
            std::hint::black_box(packet);
        });
    });

    group.finish();
}

/// 基准测试场景4: 网络质量自适应
fn bench_network_adaptation(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_adaptation");

    // 良好网络
    group.bench_function("good_network", |bencher| {
        let mut manager = OptimizedStateSyncManager::new(16, 100);

        for i in 0..100 {
            manager.register_entity(i, EntityPriority::Medium);
            let state = create_test_state(i);
            let mut dirty_flags = HashSet::new();
            dirty_flags.insert(DirtyFlag::Position);
            manager.update_entity_state(i, state, dirty_flags).unwrap();
        }

        // 设置良好网络质量
        let good_quality = NetworkQuality {
            latency_ms: 20,
            packet_loss: 0.001,
            bandwidth_bps: 10_000_000,
            jitter_ms: 5,
        };
        manager.update_network_quality(good_quality);

        bencher.iter(|| {
            let packet = manager.generate_optimized_sync_data(0);
            std::hint::black_box(packet);
        });
    });

    // 较差网络
    group.bench_function("poor_network", |bencher| {
        let mut manager = OptimizedStateSyncManager::new(16, 100);

        for i in 0..100 {
            manager.register_entity(i, EntityPriority::Medium);
            let state = create_test_state(i);
            let mut dirty_flags = HashSet::new();
            dirty_flags.insert(DirtyFlag::Position);
            manager.update_entity_state(i, state, dirty_flags).unwrap();
        }

        // 设置较差网络质量
        let poor_quality = NetworkQuality {
            latency_ms: 200,
            packet_loss: 0.1,
            bandwidth_bps: 500_000,
            jitter_ms: 50,
        };
        manager.update_network_quality(poor_quality);

        bencher.iter(|| {
            let packet = manager.generate_optimized_sync_data(0);
            std::hint::black_box(packet);
        });
    });

    group.finish();
}

/// 基准测试场景5: 综合性能（实际游戏场景）
fn bench_real_world_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world");

    // 典型多人游戏场景：1个玩家 + 10个敌人 + 50个环境物体
    group.bench_function("typical_multiplayer", |bencher| {
        let mut manager = OptimizedStateSyncManager::new(16, 100);

        // 1个玩家（关键优先级）
        manager.register_entity(0, EntityPriority::Critical);
        let player_state = create_test_state(0);
        let mut player_dirty = HashSet::new();
        player_dirty.insert(DirtyFlag::Position);
        player_dirty.insert(DirtyFlag::Velocity);
        manager.update_entity_state(0, player_state, player_dirty).unwrap();

        // 10个敌人（高优先级）
        for i in 1..=10 {
            manager.register_entity(i, EntityPriority::High);
            let state = create_test_state(i);
            let mut dirty_flags = HashSet::new();
            dirty_flags.insert(DirtyFlag::Position);
            manager.update_entity_state(i, state, dirty_flags).unwrap();
        }

        // 50个环境物体（低优先级）
        for i in 11..=60 {
            manager.register_entity(i, EntityPriority::Low);
            let state = create_test_state(i);
            let mut dirty_flags = HashSet::new();
            // 环境物体变更较少
            if i % 5 == 0 {
                dirty_flags.insert(DirtyFlag::Position);
            }
            manager.update_entity_state(i, state, dirty_flags).unwrap();
        }

        bencher.iter(|| {
            let packet = manager.generate_optimized_sync_data(0);
            std::hint::black_box(packet);
        });
    });

    group.finish();
}

criterion_group!(
    name = state_sync_optimization_benches;
    config = Criterion::default().sample_size(100);
    targets = bench_optimized_vs_full_sync, bench_priority_queue,
             bench_dirty_tracking, bench_network_adaptation, bench_real_world_scenario
);

criterion_main!(state_sync_optimization_benches);
