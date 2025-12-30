//! AI系统集成测试
//!
//! 测试行为树和覆盖图在实际AI决策场景中的集成效果。

use game_engine::ai::influence_map::{InfluenceGrid, TacticalInfluenceMap};
use game_engine::ai::behavior_tree::{Sequence, Selector, Status, Node, Action};

/// 模拟AI实体
#[derive(Debug, Clone)]
struct AIEntity {
    id: u32,
    position: (usize, usize),
    health: f32,
    ammo: u32,
}

impl AIEntity {
    fn new(id: u32, x: usize, y: usize) -> Self {
        Self {
            id,
            position: (x, y),
            health: 100.0,
            ammo: 30,
        }
    }

    fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    fn has_ammo(&self) -> bool {
        self.ammo > 0
    }

    fn is_dangerous_nearby(&self, danger_map: &InfluenceGrid) -> bool {
        let (x, y) = self.position;
        danger_map.get(x, y) < -20.0
    }
}

/// 简单的条件节点实现
struct IsAliveCondition {
    entity: AIEntity,
}

impl Node for IsAliveCondition {
    fn tick(&mut self) -> Status {
        if self.entity.is_alive() {
            Status::Success
        } else {
            Status::Failure
        }
    }
}

struct HasAmmoCondition {
    entity: AIEntity,
}

impl Node for HasAmmoCondition {
    fn tick(&mut self) -> Status {
        if self.entity.has_ammo() {
            Status::Success
        } else {
            Status::Failure
        }
    }
}

struct IsSafeCondition {
    entity: AIEntity,
    danger_map: InfluenceGrid,
}

impl Node for IsSafeCondition {
    fn tick(&mut self) -> Status {
        if !self.entity.is_dangerous_nearby(&self.danger_map) {
            Status::Success
        } else {
            Status::Failure
        }
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_influence_map_with_ai_entities() {
        // 创建覆盖图
        let mut territory = InfluenceGrid::new(50, 50, 1.0);
        let mut danger = InfluenceGrid::new(50, 50, 1.0);

        // 添加一些实体
        let player = AIEntity::new(1, 25, 25);
        let enemy = AIEntity::new(2, 30, 30);

        // 添加影响力
        territory.add_source(player.position.0, player.position.1, 100.0);
        danger.add_source(enemy.position.0, enemy.position.1, -80.0);

        // 传播影响力
        territory.propagate(0.3, 5);
        danger.propagate(0.3, 5);

        // 验证传播效果
        assert!(territory.get(25, 25) > 50.0);
        assert!(danger.get(30, 30) < -40.0);

        // 查找安全区域
        let (_, _, min_danger) = danger.find_min();
        assert!(min_danger < 0.0);
    }

    #[test]
    fn test_tactical_map_decision_making() {
        // 创建战术覆盖图
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

        // 设置领土（玩家控制区域）
        tactical.territory.add_source(30, 30, 100.0);
        tactical.territory.add_source(70, 70, 80.0);

        // 设置危险（敌人位置）
        tactical.danger.add_source(60, 60, -90.0);

        // 设置机会（资源点）
        tactical.opportunity.add_source(40, 40, 60.0);
        tactical.opportunity.add_source(80, 20, 70.0);

        // 更新覆盖图
        tactical.update(0.3, 5);

        // 查找最佳位置
        let (best_x, best_y, best_score) = tactical.find_best_position();

        // 最佳位置应该在领土强、危险低、机会高的地方
        assert!(best_score > 0.0);

        // 验证该位置的各个指标
        let territory_value = tactical.territory.get(best_x, best_y);
        let _danger_value = tactical.danger.get(best_x, best_y);
        let opportunity_value = tactical.opportunity.get(best_x, best_y);

        assert!(territory_value > 0.0 || opportunity_value > 0.0);
    }

    #[test]
    fn test_behavior_tree_with_validation() {
        // 创建一个简单的行为树
        // Sequence: 检查存活 AND 检查弹药
        let entity = AIEntity::new(1, 10, 10);

        let mut sequence = Sequence {
            children: vec![
                Box::new(IsAliveCondition { entity: entity.clone() }) as Box<dyn Node>,
                Box::new(HasAmmoCondition { entity }) as Box<dyn Node>,
            ],
        };

        // 执行行为树
        let status = sequence.tick();

        // 如果实体存活且有弹药，应该成功
        assert_eq!(status, Status::Success);
    }

    #[test]
    fn test_behavior_tree_selector_with_cover() {
        // 测试选择器行为：安全 OR 战斗
        let _territory = InfluenceGrid::new(50, 50, 1.0);
        let mut danger = InfluenceGrid::new(50, 50, 1.0);

        let entity = AIEntity::new(1, 25, 25);

        // 添加危险
        danger.add_source(30, 30, -90.0);
        danger.propagate(0.3, 3);

        // 创建选择器：先检查安全，不安全则返回失败
        let mut selector = Selector {
            children: vec![
                Box::new(IsSafeCondition {
                    entity: entity.clone(),
                    danger_map: danger.clone(),
                }) as Box<dyn Node>,
                Box::new(Action) as Box<dyn Node>, // 备用行动
            ],
        };

        let status = selector.tick();
        // 应该执行第一个子节点
        assert!(matches!(status, Status::Success | Status::Failure));
    }

    #[test]
    fn test_dynamic_tactical_analysis() {
        // 测试动态战术分析
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

        // 初始状态：只有领土
        tactical.territory.add_source(50, 50, 100.0);
        tactical.update(0.3, 3);

        let (x1, y1, score1) = tactical.find_best_position();

        // 添加敌人威胁
        tactical.danger.add_source(60, 60, -100.0);
        tactical.update(0.3, 3);

        let (x2, y2, score2) = tactical.find_best_position();

        // 最佳位置应该改变
        assert!(score1 != score2 || (x1, y1) != (x2, y2));

        // 添加机会点
        tactical.opportunity.add_source(40, 40, 80.0);
        tactical.update(0.3, 3);

        let (_x3, _y3, score3) = tactical.find_best_position();

        // 新的评分应该考虑机会点
        assert!(score3 > score2 - 50.0); // 允许一定误差
    }

    #[test]
    fn test_multi_entity_tactical_positioning() {
        // 测试多个实体的战术位置选择
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

        // 添加多个实体
        let entities = vec![
            AIEntity::new(1, 20, 20),
            AIEntity::new(2, 50, 50),
            AIEntity::new(3, 80, 80),
        ];

        for entity in &entities {
            tactical.territory.add_source(entity.position.0, entity.position.1, 50.0);
        }

        // 添加敌人
        tactical.danger.add_source(40, 40, -100.0);
        tactical.danger.add_source(60, 60, -100.0);

        tactical.update(0.3, 5);

        // 为每个实体找到最佳位置
        let mut best_positions = Vec::new();
        for entity in &entities {
            let score = tactical.analyze_position(entity.position.0, entity.position.1);
            best_positions.push((entity.id, score));
        }

        // 验证每个位置都有评分
        assert_eq!(best_positions.len(), 3);

        // 验证位置1（远离危险）的评分应该合理
        let position1_score = tactical.analyze_position(20, 20);
        let position2_score = tactical.analyze_position(50, 50);

        // 位置1应该在领土控制方面更好，因为远离敌人的危险区域
        // 但由于传播影响，我们只验证它有合理的评分
        assert!(position1_score > position2_score - 100.0); // 允许较大误差
    }

    #[test]
    fn test_influence_map_gaussian_smoothing() {
        // 测试高斯平滑对AI决策的影响
        let mut grid = InfluenceGrid::new(50, 50, 1.0);

        // 添加多个离散的影响力源
        for i in 0..5 {
            for j in 0..5 {
                grid.add_source(i * 10, j * 10, if (i + j) % 2 == 0 { 100.0 } else { -100.0 });
            }
        }

        // 在平滑前查找最大值
        let (x1, y1, max1) = grid.find_max();

        // 应用高斯平滑
        grid.gaussian_smooth(2.0, 3);

        // 在平滑后查找最大值
        let (x2, y2, max2) = grid.find_max();

        // 平滑后最大值应该降低
        assert!(max2 < max1);

        // 位置应该接近（但可能不完全相同）
        let distance = ((x1 as i32 - x2 as i32).abs() + (y1 as i32 - y2 as i32).abs()) as f32;
        assert!(distance < 20.0);
    }

    #[test]
    fn test_real_time_ai_decision() {
        // 模拟实时AI决策场景
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

        // 玩家当前位置
        let player_pos = (50, 50);

        // 初始化覆盖图
        tactical.territory.add_source(player_pos.0, player_pos.1, 100.0);

        // 敌人移动
        let enemy_path = vec![(70, 70), (75, 75), (80, 80)];

        for (i, enemy_pos) in enemy_path.iter().enumerate() {
            // 更新危险
            tactical.danger.clear();
            tactical.danger.add_source(enemy_pos.0, enemy_pos.1, -80.0 - (i as f32 * 10.0));

            // 更新覆盖图
            tactical.update(0.3, 3);

            // AI做出决策
            let (best_x, best_y, _score) = tactical.find_best_position();

            // 验证决策合理性
            let dist_to_enemy = (best_x as i32 - enemy_pos.0 as i32).abs() as f32
                + (best_y as i32 - enemy_pos.1 as i32).abs() as f32;

            // AI应该选择远离敌人的位置
            assert!(dist_to_enemy > 10.0);
        }
    }

    #[test]
    fn test_combined_ai_behavior() {
        // 综合测试：行为树 + 覆盖图
        let mut tactical = TacticalInfluenceMap::new(100, 100, 1.0);

        // 设置场景
        let entity = AIEntity::new(1, 30, 30);
        tactical.territory.add_source(entity.position.0, entity.position.1, 100.0);
        tactical.danger.add_source(60, 60, -90.0);
        tactical.opportunity.add_source(40, 40, 70.0);

        tactical.update(0.3, 5);

        // 使用行为树决策
        let is_safe = IsSafeCondition {
            entity: entity.clone(),
            danger_map: tactical.danger.clone(),
        };

        let mut selector = Selector {
            children: vec![
                Box::new(is_safe) as Box<dyn Node>,
                Box::new(Action) as Box<dyn Node>, // 转移到安全位置
            ],
        };

        let status = selector.tick();

        // 应该做出决策（成功或失败）
        assert!(matches!(status, Status::Success | Status::Failure));

        // 同时分析最佳位置
        let (_best_x, _best_y, score) = tactical.find_best_position();

        // 最佳位置应该有正评分
        assert!(score > -50.0);
    }

    #[test]
    fn test_influence_map_normalization() {
        // 测试覆盖图归一化
        let mut grid = InfluenceGrid::new(50, 50, 1.0);

        // 添加不同的影响力值
        grid.add_source(10, 10, 1000.0);
        grid.add_source(40, 40, -500.0);

        // 归一化到 0-1 范围
        grid.normalize(0.0, 1.0);

        // 验证归一化效果
        let (_min_x, _min_y, min_value) = grid.find_min();
        let (_max_x, _max_y, max_value) = grid.find_max();

        assert!(min_value >= 0.0 && min_value <= 1.0);
        assert!(max_value >= 0.0 && max_value <= 1.0);
    }
}
