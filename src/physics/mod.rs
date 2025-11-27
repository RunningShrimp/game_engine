//! 物理系统模块
//!
//! 提供 2D 物理模拟功能，基于 Rapier 物理引擎。
//!
//! ## 架构设计（贫血模型）
//!
//! 遵循 DDD 贫血模型设计原则，将数据与行为分离：
//! - `PhysicsState` (Resource): 纯数据结构，存储物理世界状态
//! - `PhysicsService`: 业务逻辑封装，提供物理操作方法
//! - `physics_*_system`: ECS 系统，负责调度编排
//!
//! ## 示例
//!
//! ```ignore
//! // 使用新的 Service 模式
//! fn setup_physics(mut state: ResMut<PhysicsState>) {
//!     PhysicsService::set_gravity(&mut state, [0.0, -9.81]);
//!     let rb_handle = PhysicsService::create_rigid_body(&mut state, RigidBodyType::Dynamic, [0.0, 10.0]);
//! }
//! ```

#![cfg(feature = "physics_2d")]

pub mod physics3d;
pub mod joints;
pub mod parallel;
pub mod dirty_tracker;

pub use dirty_tracker::{
    PhysicsDirty, CachedPhysicsState, PhysicsSyncConfig, PhysicsSyncStats,
    BatchSyncData, optimized_physics_sync_system, transform_to_physics_sync_system,
};

use bevy_ecs::prelude::*;
use rapier2d::prelude::*;
use rapier2d::prelude::DefaultBroadPhase;
use crate::ecs::Transform;


// ============================================================================
// 组件定义（纯数据结构）
// ============================================================================

/// 刚体组件 - 关联实体与物理刚体
#[derive(Component)]
pub struct RigidBodyComp {
    /// 刚体句柄
    pub handle: RigidBodyHandle,
}

/// 碰撞体组件 - 关联实体与物理碰撞体
#[derive(Component)]
pub struct ColliderComp {
    /// 碰撞体句柄
    pub handle: ColliderHandle,
}

// ============================================================================
// 贫血模型：PhysicsState（纯数据 Resource）
// ============================================================================

/// 物理世界状态 - 纯数据结构 (Resource)
///
/// 遵循贫血模型设计，仅包含状态数据，不包含业务逻辑。
/// 业务逻辑由 `PhysicsService` 提供。
#[derive(Resource)]
pub struct PhysicsState {
    /// 重力向量
    pub gravity: Vector<Real>,
    /// 积分参数
    pub integration_parameters: IntegrationParameters,
    /// 物理管线
    pub physics_pipeline: PhysicsPipeline,
    /// 岛屿管理器
    pub island_manager: IslandManager,
    /// 宽相碰撞检测
    pub broad_phase: Box<dyn BroadPhase>,
    /// 窄相碰撞检测
    pub narrow_phase: NarrowPhase,
    /// 冲量关节集
    pub impulse_joint_set: ImpulseJointSet,
    /// 多体关节集
    pub multibody_joint_set: MultibodyJointSet,
    /// CCD 求解器
    pub ccd_solver: CCDSolver,
    /// 刚体集
    pub rigid_body_set: RigidBodySet,
    /// 碰撞体集
    pub collider_set: ColliderSet,
    /// 物理钩子（预留）
    pub physics_hooks: (),
    /// 事件处理器（预留）
    pub event_handler: (),
}

impl Default for PhysicsState {
    fn default() -> Self {
        Self {
            gravity: vector![0.0, -9.81],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: Box::new(DefaultBroadPhase::new()),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }
}

// ============================================================================
// 贫血模型：PhysicsService（业务逻辑）
// ============================================================================

/// 物理服务 - 封装物理业务逻辑
///
/// 遵循贫血模型设计原则：
/// - `PhysicsState` (Resource): 纯数据结构
/// - `PhysicsService` (Service): 封装业务逻辑
/// - `physics_*_system` (System): 调度编排
pub struct PhysicsService;

impl PhysicsService {
    /// 执行一步物理模拟
    pub fn step(state: &mut PhysicsState) {
        state.physics_pipeline.step(
            &state.gravity,
            &state.integration_parameters,
            &mut state.island_manager,
            &mut *state.broad_phase,
            &mut state.narrow_phase,
            &mut state.rigid_body_set,
            &mut state.collider_set,
            &mut state.impulse_joint_set,
            &mut state.multibody_joint_set,
            &mut state.ccd_solver,
            None,
            &state.physics_hooks,
            &state.event_handler,
        );
    }
    
    /// 设置重力
    pub fn set_gravity(state: &mut PhysicsState, gravity: [f32; 2]) {
        state.gravity = vector![gravity[0], gravity[1]];
    }
    
    /// 获取重力
    pub fn get_gravity(state: &PhysicsState) -> [f32; 2] {
        [state.gravity.x, state.gravity.y]
    }
    
    /// 设置时间步长
    pub fn set_timestep(state: &mut PhysicsState, dt: f32) {
        state.integration_parameters.dt = dt.max(0.001);
    }
    
    /// 创建刚体
    pub fn create_rigid_body(
        state: &mut PhysicsState,
        body_type: RigidBodyType,
        position: [f32; 2],
    ) -> RigidBodyHandle {
        let rb = RigidBodyBuilder::new(body_type)
            .translation(vector![position[0], position[1]])
            .build();
        state.rigid_body_set.insert(rb)
    }
    
    /// 创建碰撞体（立方体）
    pub fn create_cuboid_collider(
        state: &mut PhysicsState,
        half_extents: [f32; 2],
        parent: Option<RigidBodyHandle>,
    ) -> ColliderHandle {
        let shape = SharedShape::cuboid(half_extents[0], half_extents[1]);
        let collider = ColliderBuilder::new(shape).build();
        if let Some(parent_handle) = parent {
            state.collider_set.insert_with_parent(collider, parent_handle, &mut state.rigid_body_set)
        } else {
            state.collider_set.insert(collider)
        }
    }
    
    /// 创建碰撞体（球体）
    pub fn create_ball_collider(
        state: &mut PhysicsState,
        radius: f32,
        parent: Option<RigidBodyHandle>,
    ) -> ColliderHandle {
        let shape = SharedShape::ball(radius);
        let collider = ColliderBuilder::new(shape).build();
        if let Some(parent_handle) = parent {
            state.collider_set.insert_with_parent(collider, parent_handle, &mut state.rigid_body_set)
        } else {
            state.collider_set.insert(collider)
        }
    }
    
    /// 移除刚体
    pub fn remove_rigid_body(state: &mut PhysicsState, handle: RigidBodyHandle) {
        state.rigid_body_set.remove(
            handle,
            &mut state.island_manager,
            &mut state.collider_set,
            &mut state.impulse_joint_set,
            &mut state.multibody_joint_set,
            true,
        );
    }
    
    /// 移除碰撞体
    pub fn remove_collider(state: &mut PhysicsState, handle: ColliderHandle) {
        state.collider_set.remove(
            handle,
            &mut state.island_manager,
            &mut state.rigid_body_set,
            true,
        );
    }
    
    /// 获取刚体位置
    pub fn get_rigid_body_position(state: &PhysicsState, handle: RigidBodyHandle) -> Option<[f32; 2]> {
        state.rigid_body_set.get(handle).map(|rb| {
            let pos = rb.translation();
            [pos.x, pos.y]
        })
    }
    
    /// 获取刚体旋转角度（弧度）
    pub fn get_rigid_body_rotation(state: &PhysicsState, handle: RigidBodyHandle) -> Option<f32> {
        state.rigid_body_set.get(handle).map(|rb| rb.rotation().angle())
    }
    
    /// 设置刚体位置
    pub fn set_rigid_body_position(state: &mut PhysicsState, handle: RigidBodyHandle, position: [f32; 2]) {
        if let Some(rb) = state.rigid_body_set.get_mut(handle) {
            rb.set_translation(vector![position[0], position[1]], true);
        }
    }
    
    /// 施加力
    pub fn apply_force(state: &mut PhysicsState, handle: RigidBodyHandle, force: [f32; 2]) {
        if let Some(rb) = state.rigid_body_set.get_mut(handle) {
            rb.add_force(vector![force[0], force[1]], true);
        }
    }
    
    /// 施加冲量
    pub fn apply_impulse(state: &mut PhysicsState, handle: RigidBodyHandle, impulse: [f32; 2]) {
        if let Some(rb) = state.rigid_body_set.get_mut(handle) {
            rb.apply_impulse(vector![impulse[0], impulse[1]], true);
        }
    }
    
    /// 获取刚体线速度
    pub fn get_linear_velocity(state: &PhysicsState, handle: RigidBodyHandle) -> Option<[f32; 2]> {
        state.rigid_body_set.get(handle).map(|rb| {
            let vel = rb.linvel();
            [vel.x, vel.y]
        })
    }
    
    /// 设置刚体线速度
    pub fn set_linear_velocity(state: &mut PhysicsState, handle: RigidBodyHandle, velocity: [f32; 2]) {
        if let Some(rb) = state.rigid_body_set.get_mut(handle) {
            rb.set_linvel(vector![velocity[0], velocity[1]], true);
        }
    }
    
    /// 获取刚体数量
    pub fn rigid_body_count(state: &PhysicsState) -> usize {
        state.rigid_body_set.len()
    }
    
    /// 获取碰撞体数量
    pub fn collider_count(state: &PhysicsState) -> usize {
        state.collider_set.len()
    }
}

// ============================================================================
// 兼容层：PhysicsWorld（保持向后兼容）
// ============================================================================

/// 物理世界资源（兼容层）
///
/// **注意**: 此类型为兼容层，推荐使用 `PhysicsState` + `PhysicsService` 模式。
///
/// # 迁移指南
///
/// ```rust
/// // 旧代码
/// let mut world = PhysicsWorld::default();
/// world.step();
///
/// // 新代码
/// let mut state = PhysicsState::default();
/// PhysicsService::step(&mut state);
/// ```
#[cfg(feature = "deprecated-apis")]
#[deprecated(
    since = "0.2.0",
    note = "请使用 PhysicsState + PhysicsService 模式代替。此类型将在 0.3.0 版本移除。"
)]
#[derive(Resource)]
pub struct PhysicsWorld {
    /// 重力向量
    pub gravity: Vector<Real>,
    /// 积分参数
    pub integration_parameters: IntegrationParameters,
    /// 物理管线
    pub physics_pipeline: PhysicsPipeline,
    /// 岛屿管理器
    pub island_manager: IslandManager,
    /// 宽相碰撞检测
    pub broad_phase: Box<dyn BroadPhase>,
    /// 窄相碰撞检测
    pub narrow_phase: NarrowPhase,
    /// 冲量关节集
    pub impulse_joint_set: ImpulseJointSet,
    /// 多体关节集
    pub multibody_joint_set: MultibodyJointSet,
    /// CCD 求解器
    pub ccd_solver: CCDSolver,
    /// 刚体集
    pub rigid_body_set: RigidBodySet,
    /// 碰撞体集
    pub collider_set: ColliderSet,
    /// 物理钩子
    pub physics_hooks: (),
    /// 事件处理器
    pub event_handler: (),
}

#[cfg(feature = "deprecated-apis")]
impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            gravity: vector![0.0, -9.81],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: Box::new(DefaultBroadPhase::new()),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }
}

impl PhysicsWorld {
    /// 执行一步物理模拟
    ///
    /// **注意**: 推荐使用 `PhysicsService::step(&mut state)` 代替
    pub fn step(&mut self) {
        self.physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut *self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
    }
    
    /// 转换为 PhysicsState
    pub fn to_state(self) -> PhysicsState {
        PhysicsState {
            gravity: self.gravity,
            integration_parameters: self.integration_parameters,
            physics_pipeline: self.physics_pipeline,
            island_manager: self.island_manager,
            broad_phase: self.broad_phase,
            narrow_phase: self.narrow_phase,
            impulse_joint_set: self.impulse_joint_set,
            multibody_joint_set: self.multibody_joint_set,
            ccd_solver: self.ccd_solver,
            rigid_body_set: self.rigid_body_set,
            collider_set: self.collider_set,
            physics_hooks: self.physics_hooks,
            event_handler: self.event_handler,
        }
    }
}

// ============================================================================
// 构建器组件
// ============================================================================

/// 刚体描述组件 - 用于声明式创建刚体
#[derive(Component, Clone)]
pub struct RigidBodyDesc {
    /// 刚体类型
    pub body_type: RigidBodyType,
    /// 初始位置
    pub position: [f32; 2],
}

/// 碰撞体描述组件 - 用于声明式创建碰撞体
#[derive(Component, Clone)]
pub struct ColliderDesc {
    /// 形状类型
    pub shape_type: ShapeType,
    /// 立方体半尺寸
    pub half_extents: [f32; 2],
    /// 球体半径
    pub radius: f32,
}

/// 形状类型枚举
#[derive(Clone, Copy)]
pub enum ShapeType {
    /// 立方体/矩形
    Cuboid,
    /// 球体/圆形
    Ball,
}

// ============================================================================
// ECS 系统函数（使用新的 Service 模式）
// ============================================================================

/// 初始化物理刚体系统 - 使用新的 Service 模式
pub fn init_physics_bodies_v2(
    mut commands: Commands,
    mut physics: ResMut<PhysicsState>,
    query: Query<(Entity, &RigidBodyDesc, Option<&ColliderDesc>), Without<RigidBodyComp>>,
) {
    for (entity, rb_desc, col_desc) in query.iter() {
        // 创建刚体
        let rb_handle = PhysicsService::create_rigid_body(
            &mut physics,
            rb_desc.body_type,
            rb_desc.position,
        );

        // 创建碰撞体（如果有）
        if let Some(cd) = col_desc {
            let col_handle = match cd.shape_type {
                ShapeType::Cuboid => PhysicsService::create_cuboid_collider(
                    &mut physics,
                    cd.half_extents,
                    Some(rb_handle),
                ),
                ShapeType::Ball => PhysicsService::create_ball_collider(
                    &mut physics,
                    cd.radius,
                    Some(rb_handle),
                ),
            };
            commands.entity(entity).insert(ColliderComp { handle: col_handle });
        }

        commands.entity(entity).insert(RigidBodyComp { handle: rb_handle });
    }
}

/// 物理步进系统 - 使用新的 Service 模式
pub fn physics_step_system_v2(mut physics: ResMut<PhysicsState>, time: Res<crate::ecs::Time>) {
    PhysicsService::set_timestep(&mut physics, time.delta_seconds);
    PhysicsService::step(&mut physics);
}

/// 同步物理到 Transform 系统 - 使用新的 Service 模式
pub fn sync_physics_to_transform_system_v2(
    physics: Res<PhysicsState>,
    mut query: Query<(&RigidBodyComp, &mut Transform)>
) {
    for (rb_comp, mut transform) in query.iter_mut() {
        if let Some(pos) = PhysicsService::get_rigid_body_position(&physics, rb_comp.handle) {
            transform.pos.x = pos[0];
            transform.pos.y = pos[1];
        }
        if let Some(angle) = PhysicsService::get_rigid_body_rotation(&physics, rb_comp.handle) {
            transform.rot = glam::Quat::from_rotation_z(angle);
        }
    }
}

// ============================================================================
// 兼容层：旧版系统函数
// ============================================================================

/// 初始化物理刚体系统（兼容层）
///
/// **注意**: 推荐使用 `init_physics_bodies_v2` 配合 `PhysicsState`
#[cfg(feature = "deprecated-apis")]
pub fn init_physics_bodies(
    mut commands: Commands,
    mut physics: ResMut<PhysicsWorld>,
    query: Query<(Entity, &RigidBodyDesc, Option<&ColliderDesc>), Without<RigidBodyComp>>,
) {
    for (entity, rb_desc, col_desc) in query.iter() {
        // Create RigidBody
        let rb = RigidBodyBuilder::new(rb_desc.body_type)
            .translation(vector![rb_desc.position[0], rb_desc.position[1]])
            .build();
        let rb_handle = physics.rigid_body_set.insert(rb);

        // Create Collider if present
        if let Some(cd) = col_desc {
            let shape = match cd.shape_type {
                ShapeType::Cuboid => SharedShape::cuboid(cd.half_extents[0], cd.half_extents[1]),
                ShapeType::Ball => SharedShape::ball(cd.radius),
            };
            let collider = ColliderBuilder::new(shape).build();
            let col_handle = physics.collider_set.insert(collider);
            commands.entity(entity).insert(ColliderComp { handle: col_handle });
        }

        commands.entity(entity).insert(RigidBodyComp { handle: rb_handle });
    }
}

/// 物理步进系统（兼容层）
///
/// **注意**: 推荐使用 `physics_step_system_v2` 配合 `PhysicsState`
#[cfg(feature = "deprecated-apis")]
pub fn physics_step_system(mut physics: ResMut<PhysicsWorld>, time: Res<crate::ecs::Time>) {
    physics.integration_parameters.dt = time.delta_seconds.max(0.001); 
    physics.step();
}

/// 同步物理到 Transform 系统（兼容层）
///
/// **注意**: 推荐使用 `sync_physics_to_transform_system_v2` 配合 `PhysicsState`
#[cfg(feature = "deprecated-apis")]
pub fn sync_physics_to_transform_system(
    physics: Res<PhysicsWorld>,
    mut query: Query<(&RigidBodyComp, &mut Transform)>
) {
    for (rb_comp, mut transform) in query.iter_mut() {
        if let Some(rb) = physics.rigid_body_set.get(rb_comp.handle) {
            let pos = rb.translation();
            let rot = rb.rotation();
            
            transform.pos.x = pos.x;
            transform.pos.y = pos.y;
            transform.rot = glam::Quat::from_rotation_z(rot.angle());
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_physics_state_creation() {
        let state = PhysicsState::default();
        let gravity = PhysicsService::get_gravity(&state);
        assert!((gravity[1] - (-9.81)).abs() < 0.01);
    }
    
    #[test]
    fn test_physics_service_gravity() {
        let mut state = PhysicsState::default();
        PhysicsService::set_gravity(&mut state, [0.0, -20.0]);
        let gravity = PhysicsService::get_gravity(&state);
        assert!((gravity[1] - (-20.0)).abs() < 0.01);
    }
    
    #[test]
    fn test_physics_service_create_body() {
        let mut state = PhysicsState::default();
        let handle = PhysicsService::create_rigid_body(&mut state, RigidBodyType::Dynamic, [10.0, 20.0]);
        
        let pos = PhysicsService::get_rigid_body_position(&state, handle);
        assert!(pos.is_some());
        let pos = pos.unwrap();
        assert!((pos[0] - 10.0).abs() < 0.01);
        assert!((pos[1] - 20.0).abs() < 0.01);
        
        assert_eq!(PhysicsService::rigid_body_count(&state), 1);
    }
    
    #[test]
    fn test_physics_service_remove_body() {
        let mut state = PhysicsState::default();
        let handle = PhysicsService::create_rigid_body(&mut state, RigidBodyType::Dynamic, [0.0, 0.0]);
        assert_eq!(PhysicsService::rigid_body_count(&state), 1);
        
        PhysicsService::remove_rigid_body(&mut state, handle);
        assert_eq!(PhysicsService::rigid_body_count(&state), 0);
    }
}
