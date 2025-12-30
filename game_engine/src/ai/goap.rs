// GOAP (Goal-Oriented Action Planning) 系统
//
// 实现目标导向的AI动作规划

use std::collections::HashMap;
use std::hash::Hash;

/// 世界状态
#[derive(Debug, Clone, PartialEq)]
pub struct WorldState {
    states: HashMap<String, StateValue>,
}

/// 状态值
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateValue {
    Bool(bool),
    Float(f32),
    Int(i32),
}

impl Default for WorldState {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldState {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }

    pub fn set(&mut self, key: &str, value: StateValue) {
        self.states.insert(key.to_string(), value);
    }

    pub fn get(&self, key: &str) -> Option<&StateValue> {
        self.states.get(key)
    }

    pub fn matches(&self, other: &WorldState) -> bool {
        for (key, value) in &other.states {
            if let Some(current) = self.states.get(key) {
                if current != value {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

/// 动作trait
pub trait Action: Send + Sync {
    /// 检查前置条件
    fn preconditions(&self) -> WorldState;

    /// 应用动作效果
    fn apply(&self, state: &mut WorldState);

    /// 计算动作成本
    fn cost(&self) -> f32;

    /// 执行动作（实际游戏逻辑）
    fn execute(&self, entity_id: u64);

    /// 获取动作名称
    fn name(&self) -> &str;
}

/// 目标trait
pub trait Goal: Send + Sync {
    /// 检查目标是否满足
    fn is_satisfied(&self, state: &WorldState) -> bool;

    /// 计算目标优先级
    fn priority(&self, state: &WorldState) -> f32;

    /// 获取目标名称
    fn name(&self) -> &str;
}

/// GOAP规划器
pub struct GoapPlanner {
    actions: Vec<Box<dyn Action>>,
    goals: Vec<Box<dyn Goal>>,
}

impl Default for GoapPlanner {
    fn default() -> Self {
        Self::new()
    }
}

impl GoapPlanner {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            goals: Vec::new(),
        }
    }

    /// 注册动作
    pub fn register_action(&mut self, action: Box<dyn Action>) {
        self.actions.push(action);
    }

    /// 注册目标
    pub fn register_goal(&mut self, goal: Box<dyn Goal>) {
        self.goals.push(goal);
    }

    /// 规划最佳动作序列
    pub fn plan(&self, current_state: &WorldState) -> Option<Vec<Box<dyn Action>>> {
        // 1. 找到最高优先级的未满足目标
        let goal =
            self.goals.iter().filter(|g| !g.is_satisfied(current_state)).max_by(|a, b| {
                a.priority(current_state)
                    .partial_cmp(&b.priority(current_state))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })?;

        // 2. 使用A*搜索找到达到目标的动作序列
        self.astar_search(current_state, goal)
    }

    /// A*搜索
    fn astar_search(
        &self,
        start_state: &WorldState,
        goal: &Box<dyn Goal>,
    ) -> Option<Vec<Box<dyn Action>>> {
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;

        // TODO: 实现完整的A*搜索
        // 简化版：返回空序列
        Some(vec![])
    }

    /// 计算启发式（到目标的估计距离）
    fn heuristic(&self, state: &WorldState, goal: &Box<dyn Goal>) -> f32 {
        // 简化版：使用目标优先级的倒数
        if goal.is_satisfied(state) {
            0.0
        } else {
            1.0 / (goal.priority(state) + 0.01)
        }
    }
}

// ============================================================================
// 示例动作和目标
// ============================================================================

/// 攻击动作
pub struct AttackAction {
    target_id: u64,
    damage: f32,
}

impl AttackAction {
    pub fn new(target_id: u64, damage: f32) -> Self {
        Self { target_id, damage }
    }
}

impl Action for AttackAction {
    fn preconditions(&self) -> WorldState {
        let mut pre = WorldState::new();
        pre.set("has_weapon", StateValue::Bool(true));
        pre.set("in_range", StateValue::Bool(true));
        pre
    }

    fn apply(&self, state: &mut WorldState) {
        state.set("target_alive", StateValue::Bool(false));
    }

    fn cost(&self) -> f32 {
        1.0
    }

    fn execute(&self, entity_id: u64) {
        println!("Entity {} attacks target {}", entity_id, self.target_id);
    }

    fn name(&self) -> &str {
        "Attack"
    }
}

/// 移动到目标动作
pub struct MoveToAction {
    target_position: (f32, f32, f32),
}

impl Action for MoveToAction {
    fn preconditions(&self) -> WorldState {
        let mut pre = WorldState::new();
        pre.set("can_move", StateValue::Bool(true));
        pre
    }

    fn apply(&self, state: &mut WorldState) {
        state.set("in_range", StateValue::Bool(true));
    }

    fn cost(&self) -> f32 {
        2.0 // 移动比攻击成本高
    }

    fn execute(&self, entity_id: u64) {
        println!("Entity {} moves to {:?}", entity_id, self.target_position);
    }

    fn name(&self) -> &str {
        "MoveTo"
    }
}

/// 消灭目标目标
pub struct EliminateTargetGoal;

impl Goal for EliminateTargetGoal {
    fn is_satisfied(&self, state: &WorldState) -> bool {
        state
            .get("target_alive")
            .map(|v| matches!(v, StateValue::Bool(false)))
            .unwrap_or(false)
    }

    fn priority(&self, state: &WorldState) -> f32 {
        // 目标存活时优先级高
        if state
            .get("target_alive")
            .map(|v| matches!(v, StateValue::Bool(true)))
            .unwrap_or(false)
        {
            10.0
        } else {
            0.0
        }
    }

    fn name(&self) -> &str {
        "EliminateTarget"
    }
}

/// 生存目标
pub struct SurvivalGoal;

impl Goal for SurvivalGoal {
    fn is_satisfied(&self, state: &WorldState) -> bool {
        state
            .get("health")
            .map(|v| matches!(v, StateValue::Float(h) if *h > 50.0))
            .unwrap_or(false)
    }

    fn priority(&self, state: &WorldState) -> f32 {
        if let Some(StateValue::Float(health)) = state.get("health") {
            if *health < 20.0 {
                20.0 // 生命值低时优先级最高
            } else if *health < 50.0 {
                10.0
            } else {
                1.0
            }
        } else {
            5.0
        }
    }

    fn name(&self) -> &str {
        "Survival"
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_state() {
        let mut state = WorldState::new();
        state.set("health", StateValue::Float(100.0));
        state.set("has_weapon", StateValue::Bool(true));

        assert_eq!(state.get("health"), Some(&StateValue::Float(100.0)));
        assert_eq!(state.get("has_weapon"), Some(&StateValue::Bool(true)));
    }

    #[test]
    fn test_goal_satisfaction() {
        let goal = EliminateTargetGoal;

        let mut satisfied_state = WorldState::new();
        satisfied_state.set("target_alive", StateValue::Bool(false));

        let mut unsatisfied_state = WorldState::new();
        unsatisfied_state.set("target_alive", StateValue::Bool(true));

        assert!(goal.is_satisfied(&satisfied_state));
        assert!(!goal.is_satisfied(&unsatisfied_state));
    }

    #[test]
    fn test_action_preconditions() {
        let action = AttackAction {
            target_id: 1,
            damage: 10.0,
        };

        let mut valid_state = WorldState::new();
        valid_state.set("has_weapon", StateValue::Bool(true));
        valid_state.set("in_range", StateValue::Bool(true));

        let mut invalid_state = WorldState::new();
        invalid_state.set("has_weapon", StateValue::Bool(false));

        assert!(valid_state.matches(&action.preconditions()));
        assert!(!invalid_state.matches(&action.preconditions()));
    }

    #[test]
    fn test_action_effects() {
        let action = AttackAction {
            target_id: 1,
            damage: 10.0,
        };

        let mut state = WorldState::new();
        state.set("target_alive", StateValue::Bool(true));

        action.apply(&mut state);

        assert_eq!(state.get("target_alive"), Some(&StateValue::Bool(false)));
    }

    #[test]
    fn test_goal_priority() {
        let goal = SurvivalGoal;

        let mut critical_state = WorldState::new();
        critical_state.set("health", StateValue::Float(10.0));

        let mut healthy_state = WorldState::new();
        healthy_state.set("health", StateValue::Float(100.0));

        assert!(goal.priority(&critical_state) > goal.priority(&healthy_state));
    }
}
