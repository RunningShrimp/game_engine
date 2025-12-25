//  AI 系统模块
//
//  提供智能代理的决策和导航功能。
//
//  ## 功能特性
//
//  - 行为树系统
//  - 状态机系统
//  - A* 寻路算法
//  - 导航网格支持
//
//  ## 使用示例
//
//  ### 寻路示例
//
//  ```rust
//  use game_engine::ai::{PathfindingService, NavigationMesh, PathNode};
//  use glam::Vec3;
//
//  // 创建导航网格
//  let mut nav_mesh = NavigationMesh::new();
//  nav_mesh.add_node(PathNode::new(0, Vec3::new(0.0, 0.0, 0.0)));
//  nav_mesh.add_node(PathNode::new(1, Vec3::new(10.0, 0.0, 0.0)));
//  nav_mesh.add_connection(0, 1, 10.0);
//
//  // 创建寻路服务
//  let mut pathfinding = PathfindingService::new(nav_mesh);
//
//  // 寻路
//  let path = pathfinding.find_path(0, 1).unwrap();
//  assert_eq!(path.len(), 2);
//  ```
//
//  ### AI组件示例
//
//  ```rust
//  use game_engine::ai::AI;
//  use bevy_ecs::prelude::*;
//
//  // 在ECS系统中使用AI组件
//  fn setup_ai_system(mut commands: Commands) {
//      commands.spawn(AI {
//          behavior_tree: None,
//          state_machine: None,
//          target_position: None,
//          current_path: Vec::new(),
//      });
//  }
//  ```

/// 行为树系统 - 用于AI决策的行为树实现
pub mod behavior_tree;
/// 群集系统 - 用于群集行为的寻路和避障
pub mod flocking;
/// 增强的群集系统 - 提供更复杂的群体行为
pub mod flocking_enhanced;
/// 导航网格 - 用于路径规划的导航网格数据结构
pub mod navmesh;
/// 增强的导航网格生成器 - 提供完整的导航网格生成功能
pub mod navmesh_enhanced;
/// 寻路系统 - 基于A*算法的路径规划服务
pub mod pathfinding;
/// 状态机 - 用于AI状态管理的状态机实现
pub mod state_machine;
/// 决策树编辑器 - 提供决策树的可视化编辑和管理
pub mod decision_tree_editor;

pub use navmesh::{
    ColliderGeometry, NavMesh, NavMeshConfig, NavMeshError, NavMeshGenerator, NavPolygon,
};
pub use navmesh_enhanced::{
    EnhancedNavMeshConfig, EnhancedNavMeshGenerator,
};

pub use flocking::{Agent, AgentId, FlockConfig, FlockManager, FlockingError, Obstacle};
pub use flocking_enhanced::{
    EnhancedFlockConfig, EnhancedFlockManager,
};

pub use decision_tree_editor::{
    DecisionNodeData, DecisionNodeType, DecisionTree, DecisionTreeEditor,
    DecisionTreeError, DecisionTreeNode, NodeUpdates,
};

// 重新导出寻路相关类型
pub use pathfinding::{
    NavigationMesh, PathConnection, PathNode, PathfindingRequest, PathfindingResult,
    PathfindingService,
};

/// 异步协程寻路服务（推荐使用）
pub mod async_pathfinding;
pub use async_pathfinding::AsyncPathfindingService;

// 向后兼容：导出已弃用的 ParallelPathfindingService
#[allow(deprecated)]
pub use pathfinding::ParallelPathfindingService;

use bevy_ecs::prelude::*;
use glam::Vec3;

/// 类型别名，用于简化复杂类型
type BehaviorAction = Box<dyn Fn(&mut World, Entity) -> BehaviorStatus + Send + Sync>;
type BehaviorCondition = Box<dyn Fn(&World, Entity) -> bool + Send + Sync>;
type StateCallback = Box<dyn Fn(&mut World, Entity) + Send + Sync>;
type StateUpdateCallback = Box<dyn Fn(&mut World, Entity) -> StateTransition + Send + Sync>;

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
///
/// 附加到实体的AI组件，提供智能行为控制。
/// 可以包含行为树、状态机或两者组合来实现复杂的AI逻辑。
#[derive(Component)]
pub struct AI {
    /// 可选的行为树，用于实现基于行为的AI
    pub behavior_tree: Option<BehaviorTree>,
    /// 可选的状态机，用于实现状态驱动的AI
    pub state_machine: Option<StateMachine>,
    /// AI当前的目标实体（如果有）
    pub target: Option<Entity>,
    /// AI当前的状态
    pub status: AIStatus,
    /// AI移动速度（单位/秒）
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
///
/// 用于实现复杂AI决策逻辑的树形结构。
/// 包含根节点和执行逻辑。
pub struct BehaviorTree {
    /// 行为树的根节点
    pub root: BehaviorNode,
}

/// 行为树节点
///
/// 定义行为树中的不同节点类型。
/// 每种节点类型都有不同的执行逻辑和用途。
pub enum BehaviorNode {
    /// 序列节点：按顺序执行所有子节点，全部成功才算成功
    Sequence(Vec<BehaviorNode>),
    /// 选择节点：按顺序尝试执行子节点，任一成功就算成功
    Selector(Vec<BehaviorNode>),
    /// 动作节点：执行具体动作，返回执行状态
    Action(BehaviorAction),
    /// 条件节点：检查条件，返回布尔值
    Condition(BehaviorCondition),
}

/// 状态机
///
/// 实现有限状态机（FSM）模式的AI控制结构。
/// 管理状态集合和状态之间的转换逻辑。
pub struct StateMachine {
    /// 当前活动状态的ID
    pub current_state: u32,
    /// 所有可能状态的集合，键为状态ID
    pub states: std::collections::HashMap<u32, State>,
    /// 状态转换映射，键为(当前状态ID, 事件名)，值为目标状态ID
    pub transitions: std::collections::HashMap<(u32, String), u32>,
}

/// 状态
///
/// 定义状态机中的单个状态。
/// 包含状态标识、名称和生命周期回调函数。
pub struct State {
    /// 状态的唯一标识符
    pub id: u32,
    /// 状态的可读名称
    pub name: String,
    /// 进入状态时执行的回调函数
    pub on_enter: Option<StateCallback>,
    /// 状态更新时执行的回调函数，返回状态转换指令
    pub on_update: Option<StateUpdateCallback>,
    /// 退出状态时执行的回调函数
    pub on_exit: Option<StateCallback>,
}

/// 状态转换
///
/// 定义状态机中的状态转换指令。
/// 由状态的更新回调返回，控制状态机的流程。
pub enum StateTransition {
    /// 保持当前状态
    None,
    /// 转换到指定ID的状态
    To(u32),
    /// 弹出状态栈（用于嵌套状态机）
    Pop,
}

// NavigationMesh, NavNode, NavConnection 现在在 pathfinding 模块中定义

/// AI 服务 - 封装 AI 业务逻辑
pub struct AIService;

impl AIService {
    /// 创建行为树
    pub fn create_behavior_tree(root: BehaviorNode) -> BehaviorTree {
        BehaviorTree { root }
    }

    /// 执行行为树
    pub fn execute_behavior(
        &self,
        world: &mut World,
        entity: Entity,
        tree: &BehaviorTree,
    ) -> BehaviorStatus {
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
        nav_mesh.find_path(start, end)
    }

    /// 更新状态机
    pub fn update_state_machine(
        &self,
        world: &mut World,
        entity: Entity,
        state_machine: &mut StateMachine,
    ) {
        if let Some(state) = state_machine.states.get(&state_machine.current_state) {
            if let Some(on_update) = &state.on_update {
                match on_update(world, entity) {
                    StateTransition::To(new_state) => {
                        self.transition_to_state(world, entity, state_machine, new_state);
                    }
                    StateTransition::Pop => {
                        // NOTE: 状态栈功能待实现，当前仅支持单状态转换
                    }
                    StateTransition::None => {}
                }
            }
        }
    }

    fn transition_to_state(
        &self,
        world: &mut World,
        entity: Entity,
        state_machine: &mut StateMachine,
        new_state: u32,
    ) {
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
