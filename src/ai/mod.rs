//! AI 系统模块
//!
//! 提供智能代理的决策和导航功能。
//!
//! ## 功能特性
//!
//! - 行为树系统
//! - 状态机系统
//! - A* 寻路算法
//! - 导航网格支持

pub mod behavior_tree;
pub mod pathfinding;
pub mod state_machine;

use bevy_ecs::prelude::*;
use glam::Vec3;

/// AI状态类型
pub enum AIStatus {
    /// 空闲状态
    Idle,
    /// 移动中
    Moving,
    /// 执行动作
    Acting,
    /// 死亡
    Dead,
}

/// 行为执行状态
pub enum BehaviorStatus {
    /// 成功
    Success,
    /// 失败
    Failure,
    /// 运行中
    Running,
}

/// AI组件
#[derive(Component)]
pub struct AI {
    pub behavior_tree: Option<BehaviorTree>,
    pub state_machine: Option<StateMachine>,
    pub target: Option<Entity>,
    pub status: AIStatus,
    pub speed: f32,
}

impl Default for AI {
    fn default() -> Self {
        Self {
            behavior_tree: None,
            state_machine: None,
            target: None,
            status: AIStatus::Idle,
            speed: 1.0,
        }
    }
}

/// 行为树
pub struct BehaviorTree {
    pub root: BehaviorNode,
}

/// 行为树节点
pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Action(Box<dyn Fn(&mut World, Entity) -> BehaviorStatus + Send + Sync>),
    Condition(Box<dyn Fn(&World, Entity) -> bool + Send + Sync>),
}

/// 状态机
pub struct StateMachine {
    pub current_state: u32,
    pub states: std::collections::HashMap<u32, State>,
    pub transitions: std::collections::HashMap<(u32, String), u32>,
}

/// 状态
pub struct State {
    pub id: u32,
    pub name: String,
    pub on_enter: Option<Box<dyn Fn(&mut World, Entity) + Send + Sync>>,
    pub on_update: Option<Box<dyn Fn(&mut World, Entity) -> StateTransition + Send + Sync>>,
    pub on_exit: Option<Box<dyn Fn(&mut World, Entity) + Send + Sync>>,
}

/// 状态转换
pub enum StateTransition {
    None,
    To(u32),
    Pop,
}

/// 寻路网格
#[derive(Resource)]
pub struct NavigationMesh {
    pub nodes: Vec<NavNode>,
    pub connections: Vec<NavConnection>,
}

/// 导航节点
pub struct NavNode {
    pub id: u32,
    pub position: Vec3,
    pub traversable: bool,
}

/// 导航连接
pub struct NavConnection {
    pub from: u32,
    pub to: u32,
    pub cost: f32,
}

/// AI 服务 - 封装 AI 业务逻辑
pub struct AIService;

impl AIService {
    /// 创建行为树
    pub fn create_behavior_tree(root: BehaviorNode) -> BehaviorTree {
        BehaviorTree { root }
    }

    /// 执行行为树
    pub fn execute_behavior(&self, world: &mut World, entity: Entity, tree: &BehaviorTree) -> BehaviorStatus {
        Self::execute_node(world, entity, &tree.root)
    }

    fn execute_node(world: &mut World, entity: Entity, node: &BehaviorNode) -> BehaviorStatus {
        match node {
            BehaviorNode::Sequence(nodes) => {
                for node in nodes {
                    match Self::execute_node(world, entity, node) {
                        BehaviorStatus::Success => continue,
                        status => return status,
                    }
                }
                BehaviorStatus::Success
            }
            BehaviorNode::Selector(nodes) => {
                for node in nodes {
                    match Self::execute_node(world, entity, node) {
                        BehaviorStatus::Failure => continue,
                        status => return status,
                    }
                }
                BehaviorStatus::Failure
            }
            BehaviorNode::Action(action) => action(world, entity),
            BehaviorNode::Condition(condition) => {
                if condition(world, entity) {
                    BehaviorStatus::Success
                } else {
                    BehaviorStatus::Failure
                }
            }
        }
    }

    /// 寻找路径
    pub fn find_path(nav_mesh: &NavigationMesh, start: Vec3, end: Vec3) -> Option<Vec<Vec3>> {
        // 简单的 A* 实现
        // TODO: 实现完整的 A* 算法
        Some(vec![start, end])
    }

    /// 更新状态机
    pub fn update_state_machine(&self, world: &mut World, entity: Entity, state_machine: &mut StateMachine) {
        if let Some(state) = state_machine.states.get(&state_machine.current_state) {
            if let Some(on_update) = &state.on_update {
                match on_update(world, entity) {
                    StateTransition::To(new_state) => {
                        self.transition_to_state(world, entity, state_machine, new_state);
                    }
                    StateTransition::Pop => {
                        // TODO: 实现状态栈
                    }
                    StateTransition::None => {}
                }
            }
        }
    }

    fn transition_to_state(&self, world: &mut World, entity: Entity, state_machine: &mut StateMachine, new_state: u32) {
        if let Some(old_state) = state_machine.states.get(&state_machine.current_state) {
            if let Some(on_exit) = &old_state.on_exit {
                on_exit(world, entity);
            }
        }

        state_machine.current_state = new_state;

        if let Some(new_state) = state_machine.states.get(&new_state) {
            if let Some(on_enter) = &new_state.on_enter {
                on_enter(world, entity);
            }
        }
    }
}