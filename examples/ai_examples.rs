//! # AI系统使用示例
//!
//! 展示行为树和覆盖图在实际游戏AI中的使用

use game_engine::ai::behavior_tree::{Sequence, Selector, Status, Node, BehaviorTree};
use game_engine::ai::influence_map::{InfluenceGrid, TacticalInfluenceMap};

// ============================================================================
// 示例1: 简单的行为树AI
// ============================================================================

/// 简单的条件节点：检查是否存活
struct IsAliveCondition {
    health: f32,
}

impl Node for IsAliveCondition {
    fn tick(&mut self) -> Status {
        if self.health > 0.0 {
            Status::Success
        } else {
            Status::Failure
        }
    }
}

/// 简单的动作节点：攻击
struct AttackAction;

impl Node for AttackAction {
    fn tick(&mut self) -> Status {
        // 模拟攻击逻辑
        println!("⚔️  Attacking target...");
        Status::Success
    }
}

/// 简单的动作节点：巡逻
struct PatrolAction;

impl Node for PatrolAction {
    fn tick(&mut self) -> Status {
        // 模拟巡逻逻辑
        println!("👮 Patrolling area...");
        Status::Success
    }
}

/// 创建简单的AI行为树
fn example_1_simple_behavior_tree() {
    println!("\n=== 示例1: 简单的行为树AI ===\n");

    let mut tree = BehaviorTree {
        root: Box::new(Selector {
            children: vec![
                // 如果存活，则攻击
                Box::new(Sequence {
                    children: vec![
                        Box::new(IsAliveCondition { health: 100.0 }) as Box<dyn Node>,
                        Box::new(AttackAction) as Box<dyn Node>,
                    ],
                }),
                // 否则巡逻
                Box::new(PatrolAction) as Box<dyn Node>,
            ],
        }),
    };

    // 执行行为树
    let status = tree.tick();
    println!("Behavior tree status: {:?}", status);
}

// ============================================================================
// 示例2: 战术覆盖图分析
// ============================================================================

fn example_2_tactical_analysis() {
    println!("\n=== 示例2: 战术覆盖图分析 ===\n");

    // 创建战术覆盖图
    let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

    // 设置玩家控制的领土
    println!("🏰 Setting up territory control...");
    tactical.territory.add_source(30, 30, 100.0);
    tactical.territory.add_source(70, 70, 80.0);

    // 添加敌人威胁
    println!("⚔️ Adding enemy threats...");
    tactical.danger.add_source(50, 50, -90.0);
    tactical.danger.add_source(20, 80, -70.0);

    // 添加资源机会点
    println!("💎 Adding resource opportunities...");
    tactical.opportunity.add_source(40, 40, 60.0);
    tactical.opportunity.add_source(80, 20, 70.0);

    // 更新覆盖图
    println!("📊 Updating influence maps...");
    tactical.update(0.3, 5);

    // 分析当前玩家位置
    let player_pos = (30, 30);
    let score = tactical.analyze_position(player_pos.0, player_pos.1);
    println!("Player position score: {:.2}", score);

    // 查找最佳战术位置
    let (best_x, best_y, best_score) = tactical.find_best_position();
    println!("Best tactical position: ({}, {}) with score {:.2}", best_x, best_y, best_score);

    // 分析领土分布
    let (min_x, min_y, min_val) = tactical.danger.find_min();
    let (max_x, max_y, max_val) = tactical.territory.find_max();
    println!("Safest area: ({}, {}) score {:.2}", min_x, min_y, min_val);
    println!("Strongest territory: ({}, {}) score {:.2}", max_x, max_y, max_val);
}

// ============================================================================
// 示例3: 实时AI决策
// ============================================================================

fn example_3_realtime_ai_decision() {
    println!("\n=== 示例3: 实时AI决策 ===\n");

    let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

    // 初始状态：玩家在(50, 50)
    let player_pos = (50, 50);
    tactical.territory.add_source(player_pos.0, player_pos.1, 100.0);

    // 敌人路径
    let enemy_path = vec![
        (70, 70),
        (65, 65),
        (60, 60),
        (55, 55),
    ];

    println!("🎮 Simulating enemy movement...");

    for (step, enemy_pos) in enemy_path.iter().enumerate() {
        // 更新危险
        tactical.danger.clear();
        tactical.danger.add_source(enemy_pos.0, enemy_pos.1, -80.0);

        // 更新覆盖图
        tactical.update(0.3, 3);

        // AI决策
        let (best_x, best_y, score) = tactical.find_best_position();

        let dist_to_enemy = ((best_x as i32 - enemy_pos.0 as i32).abs()
            + (best_y as i32 - enemy_pos.1 as i32).abs()) as f32;

        println!("Step {}: Enemy at {:?}, AI recommends ({}, {}) score={:.1}, distance={:.1}",
                 step, enemy_pos, best_x, best_y, score, dist_to_enemy);
    }
}

// ============================================================================
// 示例4: 多单位战术协调
// ============================================================================

fn example_4_multi_unit_coordination() {
    println!("\n=== 示例4: 多单位战术协调 ===\n");

    let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

    // 定义多个单位的位置
    let units = vec![
        ("Soldier1", 20usize, 20usize),
        ("Soldier2", 30usize, 30usize),
        ("Sniper", 80usize, 20usize),
        ("Medic", 70usize, 70usize),
    ];

    // 添加单位的领土控制
    println!("👥 Deploying units...");
    for (name, x, y) in &units {
        tactical.territory.add_source(*x, *y, 40.0);
        println!("  {} at ({}, {})", name, x, y);
    }

    // 添加敌人
    println!("\n⚔️ Enemy positions:");
    let enemies = vec![(50, 50), (25, 75)];
    for (x, y) in &enemies {
        tactical.danger.add_source(*x, *y, -70.0);
        println!("  Enemy at ({}, {})", x, y);
    }

    // 更新覆盖图
    println!("\n📊 Analyzing tactical situation...");
    tactical.update(0.3, 5);

    // 为每个单位评估当前位置
    println!("\n📋 Unit position assessments:");
    for (name, x, y) in &units {
        let score = tactical.analyze_position(*x, *y);
        let territory = tactical.territory.get(*x, *y);
        let danger = tactical.danger.get(*x, *y);

        println!("  {}: ({}, {}) score={:.1}, territory={:.1}, danger={:.1}",
                 name, x, y, score, territory, danger);
    }

    // 找到最佳集体位置
    let (best_x, best_y, best_score) = tactical.find_best_position();
    println!("\n🎯 Best strategic position: ({}, {}) score={:.1}", best_x, best_y, best_score);
}

// ============================================================================
// 示例5: 覆盖图传播算法演示
// ============================================================================

fn example_5_influence_propagation() {
    println!("\n=== 示例5: 覆盖图传播算法演示 ===\n");

    let mut grid = InfluenceGrid::new(20, 20, 1.0);

    // 在中心添加强影响力源
    let center = (10, 10);
    grid.add_source(center.0, center.1, 100.0);

    println!("Initial influence at center: {:.2}", grid.get(center.0, center.1));

    // 传播影响力
    let decay = 0.5;
    let iterations = 5;

    println!("\nPropagating with decay={}, iterations={}...", decay, iterations);

    for i in 0..iterations {
        grid.propagate(decay, 1);

        // 每次迭代后检查几个关键点
        let points = vec![(10, 10), (9, 10), (11, 10), (10, 9), (10, 11)];
        println!("After iteration {}:", i + 1);
        for (x, y) in &points {
            println!("  ({}, {}): {:.2}", x, y, grid.get(*x, *y));
        }
    }

    // 找到最大影响力位置
    let (max_x, max_y, max_val) = grid.find_max();
    println!("\nMaximum influence: ({}, {}) = {:.2}", max_x, max_y, max_val);
}

// ============================================================================
// 示例6: 高斯平滑效果
// ============================================================================

fn example_6_gaussian_smoothing() {
    println!("\n=== 示例6: 高斯平滑效果 ===\n");

    let mut grid = InfluenceGrid::new(20, 20, 1.0);

    // 添加多个离散的影响力源
    println!("Adding discrete influence sources...");
    let sources = vec![(5, 5), (15, 5), (10, 15)];
    for (x, y) in &sources {
        grid.add_source(*x, *y, 100.0);
        println!("  Source at ({}, {})", x, y);
    }

    // 平滑前
    let (max_x, max_y, max_before) = grid.find_max();
    println!("\nBefore smoothing: max at ({}, {}) = {:.2}", max_x, max_y, max_before);

    // 应用高斯平滑
    println!("\nApplying Gaussian smoothing (sigma=2.0, radius=3)...");
    grid.gaussian_smooth(2.0, 3);

    // 平滑后
    let (max_x2, max_y2, max_after) = grid.find_max();
    println!("After smoothing: max at ({}, {}) = {:.2}", max_x2, max_y2, max_after);
    println!("Reduction: {:.2} ({:.1}%)", max_before - max_after,
             (max_before - max_after) / max_before * 100.0);
}

// ============================================================================
// 主函数
// ============================================================================

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 游戏引擎AI系统使用示例\n");
    println!("====================================\n");

    example_1_simple_behavior_tree();
    example_2_tactical_analysis();
    example_3_realtime_ai_decision();
    example_4_multi_unit_coordination();
    example_5_influence_propagation();
    example_6_gaussian_smoothing();

    println!("\n====================================");
    println!("🎉 所有AI示例执行完成！");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_1() {
        example_1_simple_behavior_tree();
    }

    #[test]
    fn test_example_2() {
        example_2_tactical_analysis();
    }

    #[test]
    fn test_example_3() {
        example_3_realtime_ai_decision();
    }

    #[test]
    fn test_example_4() {
        example_4_multi_unit_coordination();
    }

    #[test]
    fn test_example_5() {
        example_5_influence_propagation();
    }

    #[test]
    fn test_example_6() {
        example_6_gaussian_smoothing();
    }
}
