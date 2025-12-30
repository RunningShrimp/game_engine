// A*搜索算法 - GOAP规划器优化版本
//
// 使用启发式搜索的高效GOAP规划

use std::collections::{HashMap, HashSet, BinaryHeap};
use std::cmp::Ordering;
use std::f32::MAX;

use crate::ai::goap::{WorldState, Action, Goal, StateValue};

/// A*搜索节点
#[derive(Clone)]
struct AStarNode {
    /// 当前世界状态
    state: WorldState,
    /// 从起点到当前节点的实际代价
    g_cost: f32,
    /// 启发式估计到目标的代价
    h_cost: f32,
    /// 总代价 f = g + h
    f_cost: f32,
    /// 达到此状态的动作序列
    actions: Vec<String>,
}

impl PartialEq for AStarNode {
    fn eq(&self, other: &Self) -> bool {
        self.f_cost == other.f_cost
    }
}

impl Eq for AStarNode {}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap是最大堆，但我们想要最小堆，所以反转比较
        other.f_cost.partial_cmp(&self.f_cost).unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(other.f_cost.partial_cmp(&self.f_cost).unwrap_or(Ordering::Equal))
    }
}

/// A*搜索规划器
pub struct AStarPlanner {
    max_search_depth: usize,
    max_nodes_expanded: usize,
    heuristic_weight: f32,
}

impl AStarPlanner {
    /// 创建新的A*规划器
    pub fn new() -> Self {
        Self {
            max_search_depth: 50,
            max_nodes_expanded: 1000,
            heuristic_weight: 1.0,
        }
    }

    /// 使用A*搜索寻找最佳动作序列
    pub fn plan(
        &self,
        current_state: &WorldState,
        actions: &[Box<dyn Action>],
        goal: &Box<dyn Goal>,
    ) -> Option<Vec<String>> {
        // 初始化
        let mut open_set: BinaryHeap<AStarNode> = BinaryHeap::new();
        let mut closed_set: HashSet<u64> = HashSet::new();
        let mut nodes_expanded = 0;

        // 初始节点
        let h = self.heuristic(current_state, goal);
        let initial_node = AStarNode {
            state: current_state.clone(),
            g_cost: 0.0,
            h_cost: h,
            f_cost: h,
            actions: Vec::new(),
        };

        open_set.push(initial_node);

        // A*主循环
        while let Some(current) = open_set.pop() {
            // 检查是否达到目标
            if goal.is_satisfied(&current.state) {
                return Some(current.actions);
            }

            // 检查搜索限制
            if current.actions.len() >= self.max_search_depth {
                continue;
            }

            nodes_expanded += 1;
            if nodes_expanded > self.max_nodes_expanded {
                break; // 搜索空间太大
            }

            // 生成状态哈希用于closed set
            let state_hash = self.hash_state(&current.state);
            if closed_set.contains(&state_hash) {
                continue;
            }
            closed_set.insert(state_hash);

            // 扩展节点 - 尝试所有动作
            for action in actions {
                // 检查前置条件
                if !self.check_preconditions(action, &current.state) {
                    continue;
                }

                // 应用动作
                let mut new_state = current.state.clone();
                action.apply(&mut new_state);

                // 计算新节点的代价
                let g = current.g_cost + action.cost();
                let h = self.heuristic(&new_state, goal);
                let f = g + self.heuristic_weight * h;

                // 创建新节点
                let mut new_actions = current.actions.clone();
                new_actions.push(action.name().to_string());

                let new_node = AStarNode {
                    state: new_state,
                    g_cost: g,
                    h_cost: h,
                    f_cost: f,
                    actions: new_actions,
                };

                open_set.push(new_node);
            }
        }

        // 未找到解决方案
        None
    }

    /// 启发式函数 - 估计到目标的代价
    fn heuristic(&self, state: &WorldState, goal: &Box<dyn Goal>) -> f32 {
        // 如果目标已满足，代价为0
        if goal.is_satisfied(state) {
            return 0.0;
        }

        // 否则，使用目标优先级的倒数作为启发式
        // 高优先级目标 = 低启发式代价
        let priority = goal.priority(state);
        if priority > 0.0 {
            1.0 / priority
        } else {
            MAX // 无穷大
        }
    }

    /// 检查动作的前置条件
    fn check_preconditions(&self, action: &Box<dyn Action>, state: &WorldState) -> bool {
        let preconditions = action.preconditions();

        // 简化版本：只检查主要状态
        // 实际实现需要遍历preconditions的所有键
        if let Some(has_weapon) = preconditions.get("has_weapon") {
            if let Some(current) = state.get("has_weapon") {
                // 比较StateValue需要克隆
                if current != has_weapon {
                    return false;
                }
            }
        }

        true
    }

    /// 生成世界状态的哈希值
    fn hash_state(&self, state: &WorldState) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::Hasher;

        let mut hasher = DefaultHasher::new();

        // 简单哈希：基于主要状态
        if let Some(&health) = state.get("health") {
            if let StateValue::Float(h) = health {
                hasher.write_u32(h.to_bits());
            }
        }

        hasher.finish()
    }

    /// 设置最大搜索深度
    pub fn set_max_search_depth(&mut self, depth: usize) {
        self.max_search_depth = depth;
    }

    /// 设置最大扩展节点数
    pub fn set_max_nodes_expanded(&mut self, nodes: usize) {
        self.max_nodes_expanded = nodes;
    }

    /// 设置启发式权重
    pub fn set_heuristic_weight(&mut self, weight: f32) {
        self.heuristic_weight = weight;
    }
}

impl Default for AStarPlanner {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::goap::*;

    #[test]
    fn test_astar_planner_creation() {
        let planner = AStarPlanner::new();
        assert_eq!(planner.max_search_depth, 50);
    }

    #[test]
    fn test_astar_search() {
        let planner = AStarPlanner::new();

        // 创建测试状态
        let mut current_state = WorldState::new();
        current_state.set("has_weapon", StateValue::Bool(true));
        current_state.set("target_alive", StateValue::Bool(true));

        // 创建测试动作
        let attack_action = Box::new(AttackAction {
            target_id: 1,
            damage: 25.0,
        }) as Box<dyn Action>;

        // 创建测试目标
        let goal = Box::new(EliminateTargetGoal) as Box<dyn Goal>;

        // 执行搜索
        let plan = planner.plan(&current_state, &[attack_action], &goal);

        // 应该找到计划
        assert!(plan.is_some());
    }

    #[test]
    fn test_heuristic() {
        let planner = AStarPlanner::new();

        let mut satisfied_state = WorldState::new();
        satisfied_state.set("target_alive", StateValue::Bool(false));

        let mut unsatisfied_state = WorldState::new();
        unsatisfied_state.set("target_alive", StateValue::Bool(true));

        let goal = Box::new(EliminateTargetGoal) as Box<dyn Goal>;

        // 满足状态的启发式应该更低
        let h_satisfied = planner.heuristic(&satisfied_state, &goal);
        let h_unsatisfied = planner.heuristic(&unsatisfied_state, &goal);

        assert!(h_satisfied < h_unsatisfied);
    }

    #[test]
    fn test_search_limits() {
        let mut planner = AStarPlanner::new();
        planner.set_max_search_depth(3);
        planner.set_max_nodes_expanded(10);

        assert_eq!(planner.max_search_depth, 3);
        assert_eq!(planner.max_nodes_expanded, 10);
    }
}
