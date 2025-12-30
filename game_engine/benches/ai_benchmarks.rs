// AI系统性能基准测试
//
// 测试行为树和覆盖图系统的性能表现

use game_engine::ai::influence_map::{InfluenceGrid, TacticalInfluenceMap};
use game_engine::ai::behavior_tree::{Sequence, Selector, Status, Node, Action};

#[cfg(test)]
mod ai_benches {
    use super::*;

    // ============================================================================
    // 基准测试1: 覆盖图传播
    // ============================================================================

    #[bench]
    fn bench_influence_propagate_50x50_5_iterations(b: &mut test::Bencher) {
        let mut grid = InfluenceGrid::new(50, 50, 1.0);
        grid.add_source(25, 25, 100.0);

        b.iter(|| {
            let mut test_grid = grid.clone();
            test_grid.propagate(0.3, 5);
        });
    }

    #[bench]
    fn bench_influence_propagate_100x100_10_iterations(b: &mut test::Bencher) {
        let mut grid = InfluenceGrid::new(100, 100, 1.0);
        grid.add_source(50, 50, 100.0);

        b.iter(|| {
            let mut test_grid = grid.clone();
            test_grid.propagate(0.3, 10);
        });
    }

    // ============================================================================
    // 基准测试2: 战术覆盖图更新
    // ============================================================================

    #[bench]
    fn bench_tactical_map_update(b: &mut test::Bencher) {
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);
        tactical.territory.add_source(30, 30, 100.0);
        tactical.danger.add_source(60, 60, -90.0);
        tactical.opportunity.add_source(40, 40, 60.0);

        b.iter(|| {
            let mut test_tactical = tactical.clone();
            test_tactical.update(0.3, 5);
        });
    }

    // ============================================================================
    // 基准测试3: 位置分析
    // ============================================================================

    #[bench]
    fn bench_analyze_position(b: &mut test::Bencher) {
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);
        tactical.territory.add_source(50, 50, 100.0);
        tactical.danger.add_source(30, 30, -80.0);
        tactical.update(0.3, 5);

        b.iter(|| {
            tactical.analyze_position(50, 50);
        });
    }

    #[bench]
    fn bench_find_best_position(b: &mut test::Bencher) {
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);
        tactical.territory.add_source(30, 30, 100.0);
        tactical.danger.add_source(60, 60, -90.0);
        tactical.opportunity.add_source(40, 40, 60.0);
        tactical.update(0.3, 5);

        b.iter(|| {
            tactical.find_best_position();
        });
    }

    // ============================================================================
    // 基准测试4: 高斯平滑
    // ============================================================================

    #[bench]
    fn bench_gaussian_smooth_50x50(b: &mut test::Bencher) {
        let mut grid = InfluenceGrid::new(50, 50, 1.0);
        for i in 0..5 {
            for j in 0..5 {
                grid.add_source(i * 10, j * 10, 100.0);
            }
        }

        b.iter(|| {
            let mut test_grid = grid.clone();
            test_grid.gaussian_smooth(2.0, 3);
        });
    }

    // ============================================================================
    // 基准测试5: 行为树执行
    // ============================================================================

    struct AlwaysSuccess;
    impl Node for AlwaysSuccess {
        fn tick(&mut self) -> Status {
            Status::Success
        }
    }

    struct AlwaysFailure;
    impl Node for AlwaysFailure {
        fn tick(&mut self) -> Status {
            Status::Failure
        }
    }

    #[bench]
    fn bench_sequence_5_nodes(b: &mut test::Bencher) {
        let mut sequence = Sequence {
            children: vec![
                Box::new(AlwaysSuccess) as Box<dyn Node>,
                Box::new(AlwaysSuccess) as Box<dyn Node>,
                Box::new(AlwaysSuccess) as Box<dyn Node>,
                Box::new(AlwaysSuccess) as Box<dyn Node>,
                Box::new(AlwaysSuccess) as Box<dyn Node>,
            ],
        };

        b.iter(|| {
            sequence.tick();
        });
    }

    #[bench]
    fn bench_selector_5_nodes(b: &mut test::Bencher) {
        let mut selector = Selector {
            children: vec![
                Box::new(AlwaysFailure) as Box<dyn Node>,
                Box::new(AlwaysFailure) as Box<dyn Node>,
                Box::new(AlwaysFailure) as Box<dyn Node>,
                Box::new(AlwaysFailure) as Box<dyn Node>,
                Box::new(AlwaysSuccess) as Box<dyn Node>,
            ],
        };

        b.iter(|| {
            selector.tick();
        });
    }

    // ============================================================================
    // 基准测试6: 实时AI决策循环
    // ============================================================================

    #[bench]
    fn bench_realtime_ai_decision_10_steps(b: &mut test::Bencher) {
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);
        tactical.territory.add_source(50, 50, 100.0);

        let enemy_path = vec![(70, 70), (65, 65), (60, 60), (55, 55)];

        b.iter(|| {
            let mut test_tactical = tactical.clone();
            for enemy_pos in &enemy_path {
                test_tactical.danger.clear();
                test_tactical.danger.add_source(enemy_pos.0, enemy_pos.1, -80.0);
                test_tactical.update(0.3, 3);
                test_tactical.find_best_position();
            }
        });
    }

    // ============================================================================
    // 基准测试7: 多单位分析
    // ============================================================================

    #[bench]
    fn bench_multi_unit_analysis_10_units(b: &mut test::Bencher) {
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

        // 添加10个单位
        let units: Vec<(usize, usize)> = (0..10)
            .map(|i| ((i * 10) % 100, (i * 10) % 100))
            .collect();

        for &(x, y) in &units {
            tactical.territory.add_source(x, y, 50.0);
        }

        // 添加敌人
        tactical.danger.add_source(50, 50, -100.0);
        tactical.update(0.3, 5);

        b.iter(|| {
            for &(x, y) in &units {
                tactical.analyze_position(x, y);
            }
        });
    }

    // ============================================================================
    // 基准测试8: 覆盖图操作
    // ============================================================================

    #[bench]
    fn bench_add_source(b: &mut test::Bencher) {
        let mut grid = InfluenceGrid::new(100, 100, 1.0);

        b.iter(|| {
            let mut test_grid = InfluenceGrid::new(100, 100, 1.0);
            for i in 0..10 {
                test_grid.add_source(i * 10, i * 10, 100.0);
            }
        });
    }

    #[bench]
    fn bench_get_value(b: &mut test::Bencher) {
        let grid = InfluenceGrid::new(100, 100, 1.0);
        grid.add_source(50, 50, 100.0);
        grid.propagate(0.3, 5);

        b.iter(|| {
            for i in 0..100 {
                grid.get(i, i);
            }
        });
    }

    #[bench]
    fn bench_find_max(b: &mut test::Bencher) {
        let grid = InfluenceGrid::new(100, 100, 1.0);
        grid.add_source(50, 50, 100.0);
        grid.propagate(0.3, 5);

        b.iter(|| {
            grid.find_max();
        });
    }

    #[bench]
    fn bench_find_min(b: &mut test::Bencher) {
        let grid = InfluenceGrid::new(100, 100, 1.0);
        grid.add_source(50, 50, -100.0);
        grid.propagate(0.3, 5);

        b.iter(|| {
            grid.find_min();
        });
    }
}
