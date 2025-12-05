//! 物理领域对象
//! 实现富领域对象，将物理业务逻辑封装到对象中
//! 基于 Rapier3D 物理引擎

use crate::domain::errors::{CompensationAction, DomainError, PhysicsError, RecoveryStrategy};
use glam::{Quat, Vec3};
use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 刚体ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RigidBodyId(pub u64);

impl RigidBodyId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// 碰撞体ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColliderId(pub u64);

impl ColliderId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// 刚体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyType {
    /// 动态刚体
    Dynamic,
    /// 固定刚体
    Fixed,
    /// 运动学刚体（位置基础）
    KinematicPositionBased,
    /// 运动学刚体（速度基础）
    KinematicVelocityBased,
}

/// 形状类型
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    /// 立方体
    Cuboid,
    /// 球体
    Ball,
}

/// 刚体 - 富领域对象
///
/// 封装物理刚体的业务逻辑，包括位置、速度、力的应用等。
///
/// # 示例
///
/// ```rust
/// use game_engine::domain::{RigidBody, RigidBodyId, RigidBodyType};
/// use glam::{Vec3, Quat};
///
/// // 创建动态刚体
/// let mut body = RigidBody::new(
///     RigidBodyId::new(1),
///     RigidBodyType::Dynamic,
///     Vec3::new(0.0, 10.0, 0.0),
/// );
///
/// // 设置质量
/// body.set_mass(1.0)?;
///
/// // 应用力
/// body.apply_force(Vec3::new(0.0, -9.81, 0.0))?;
///
/// // 应用冲量
/// body.apply_impulse(Vec3::new(5.0, 0.0, 0.0))?;
///
/// // 更新位置
/// body.update_position(Vec3::new(1.0, 9.0, 0.0))?;
/// # Ok::<(), game_engine::domain::errors::DomainError>(())
/// ```
#[derive(Debug, Clone)]
pub struct RigidBody {
    /// 刚体ID
    pub id: RigidBodyId,
    /// 刚体类型
    pub body_type: RigidBodyType,
    /// 位置
    pub position: Vec3,
    /// 旋转
    pub rotation: Quat,
    /// 线速度
    pub linear_velocity: Vec3,
    /// 角速度
    pub angular_velocity: f32,
    /// 质量
    pub mass: f32,
    /// 是否休眠
    pub sleeping: bool,
    /// 最后修改时间戳
    pub last_modified: u64,
    /// 错误恢复策略
    pub recovery_strategy: RecoveryStrategy,
}

impl RigidBody {
    /// 创建新的刚体
    pub fn new(id: RigidBodyId, body_type: RigidBodyType, position: Vec3) -> Self {
        Self {
            id,
            body_type,
            position,
            rotation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: 0.0,
            mass: 1.0,
            sleeping: false,
            last_modified: Self::current_timestamp(),
            recovery_strategy: RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 50,
            },
        }
    }

    /// 创建动态刚体
    pub fn dynamic(id: RigidBodyId, position: Vec3) -> Self {
        Self::new(id, RigidBodyType::Dynamic, position)
    }

    /// 创建固定刚体
    pub fn fixed(id: RigidBodyId, position: Vec3) -> Self {
        Self::new(id, RigidBodyType::Fixed, position)
    }

    /// 应用力
    pub fn apply_force(&mut self, force: Vec3) -> Result<(), DomainError> {
        if self.body_type == RigidBodyType::Fixed {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                "Cannot apply force to fixed body".to_string(),
            )));
        }

        // F = ma, 所以 a = F/m
        let acceleration = force / self.mass;
        self.linear_velocity += acceleration * 0.016; // 假设16ms时间步长
        self.wake_up();
        self.last_modified = Self::current_timestamp();

        Ok(())
    }

    /// 应用冲量
    pub fn apply_impulse(&mut self, impulse: Vec3) -> Result<(), DomainError> {
        if self.body_type == RigidBodyType::Fixed {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                "Cannot apply impulse to fixed body".to_string(),
            )));
        }

        // 冲量直接改变速度
        self.linear_velocity += impulse / self.mass;
        self.wake_up();
        self.last_modified = Self::current_timestamp();

        Ok(())
    }

    /// 设置位置
    pub fn set_position(&mut self, position: Vec3) -> Result<(), DomainError> {
        self.position = position;
        self.wake_up();
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 设置线速度
    pub fn set_linear_velocity(&mut self, velocity: Vec3) -> Result<(), DomainError> {
        if self.body_type == RigidBodyType::Fixed {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                "Cannot set velocity on fixed body".to_string(),
            )));
        }

        self.linear_velocity = velocity;
        self.wake_up();
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 设置角速度
    pub fn set_angular_velocity(&mut self, velocity: f32) -> Result<(), DomainError> {
        if self.body_type == RigidBodyType::Fixed {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                "Cannot set angular velocity on fixed body".to_string(),
            )));
        }

        self.angular_velocity = velocity;
        self.wake_up();
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 设置质量
    pub fn set_mass(&mut self, mass: f32) -> Result<(), DomainError> {
        if mass <= 0.0 {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                "Mass must be positive".to_string(),
            )));
        }

        self.mass = mass;
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 设置旋转
    pub fn set_rotation(&mut self, rotation: Quat) -> Result<(), DomainError> {
        self.rotation = rotation;
        self.wake_up();
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 唤醒刚体
    pub fn wake_up(&mut self) {
        self.sleeping = false;
    }

    /// 让刚体休眠
    pub fn sleep(&mut self) {
        self.sleeping = true;
        self.linear_velocity = Vec3::ZERO;
        self.angular_velocity = 0.0;
    }

    /// 检查刚体是否可以休眠
    pub fn can_sleep(&self) -> bool {
        self.linear_velocity.length_squared() < 0.01 && self.angular_velocity.abs() < 0.01
    }

    /// 获取动量
    pub fn momentum(&self) -> Vec3 {
        self.linear_velocity * self.mass
    }

    /// 获取动能
    pub fn kinetic_energy(&self) -> f32 {
        0.5 * self.mass * self.linear_velocity.length_squared()
    }

    /// 验证刚体状态
    pub fn validate(&self) -> Result<(), DomainError> {
        if self.mass <= 0.0 {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                format!("Invalid mass for body {}: {}", self.id.as_u64(), self.mass),
            )));
        }

        if !self.position.is_finite() {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                format!("Invalid position for body {}", self.id.as_u64()),
            )));
        }

        Ok(())
    }

    /// 执行错误恢复
    pub fn recover_from_error(&mut self, error: &PhysicsError) -> Result<(), DomainError> {
        match &self.recovery_strategy {
            RecoveryStrategy::Retry {
                max_attempts,
                delay_ms,
            } => {
                for attempt in 1..=*max_attempts {
                    tracing::warn!(target: "physics", "Retry attempt {} for rigid body {}", attempt, self.id.as_u64());
                    std::thread::sleep(std::time::Duration::from_millis(*delay_ms));

                    match error {
                        PhysicsError::InvalidParameter(_) => {
                            // 尝试重置为默认值
                            self.mass = 1.0;
                            self.position = Vec3::ZERO;
                            self.linear_velocity = Vec3::ZERO;
                            self.angular_velocity = 0.0;
                            return Ok(());
                        }
                        _ => break,
                    }
                }
                Err(DomainError::Physics(error.clone()))
            }
            RecoveryStrategy::UseDefault => {
                self.mass = 1.0;
                self.linear_velocity = Vec3::ZERO;
                self.angular_velocity = 0.0;
                Ok(())
            }
            RecoveryStrategy::Skip => Ok(()),
            RecoveryStrategy::LogAndContinue => {
                tracing::error!(target: "physics", "Physics error logged: {:?}", error);
                Ok(())
            }
            RecoveryStrategy::Fail => Err(DomainError::Physics(error.clone())),
        }
    }

    /// 创建补偿操作
    pub fn create_compensation(&self) -> CompensationAction {
        CompensationAction::new(
            format!("rigid_body_{}", self.id.as_u64()),
            "restore_physics_state".to_string(),
            serde_json::json!({
                "position": [self.position.x, self.position.y, self.position.z],
                "rotation": [self.rotation.x, self.rotation.y, self.rotation.z, self.rotation.w],
                "linear_velocity": [self.linear_velocity.x, self.linear_velocity.y, self.linear_velocity.z],
                "angular_velocity": self.angular_velocity,
                "mass": self.mass,
                "sleeping": self.sleeping
            }),
        )
    }

    /// 从补偿操作恢复
    pub fn restore_from_compensation(
        &mut self,
        action: &CompensationAction,
    ) -> Result<(), DomainError> {
        if let Some(pos) = action.data.get("position").and_then(|v| v.as_array()) {
            if pos.len() == 3 {
                self.position = Vec3::new(
                    pos[0].as_f64().unwrap_or(0.0) as f32,
                    pos[1].as_f64().unwrap_or(0.0) as f32,
                    pos[2].as_f64().unwrap_or(0.0) as f32,
                );
            }
        }

        if let Some(vel) = action
            .data
            .get("linear_velocity")
            .and_then(|v| v.as_array())
        {
            if vel.len() == 3 {
                self.linear_velocity = Vec3::new(
                    vel[0].as_f64().unwrap_or(0.0) as f32,
                    vel[1].as_f64().unwrap_or(0.0) as f32,
                    vel[2].as_f64().unwrap_or(0.0) as f32,
                );
            }
        }

        if let Some(ang_vel) = action.data.get("angular_velocity").and_then(|v| v.as_f64()) {
            self.angular_velocity = ang_vel as f32;
        }

        if let Some(mass) = action.data.get("mass").and_then(|v| v.as_f64()) {
            self.set_mass(mass as f32)?;
        }

        if let Some(sleeping) = action.data.get("sleeping").and_then(|v| v.as_bool()) {
            self.sleeping = sleeping;
        }

        Ok(())
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 碰撞体 - 富领域对象
#[derive(Debug, Clone)]
pub struct Collider {
    /// 碰撞体ID
    pub id: ColliderId,
    /// 形状类型
    pub shape_type: ShapeType,
    /// 半边长（立方体）
    pub half_extents: Vec3,
    /// 半径（球体）
    pub radius: f32,
    /// 位置偏移
    pub offset: Vec3,
    /// 是否为触发器
    pub is_trigger: bool,
    /// 摩擦系数
    pub friction: f32,
    /// 恢复系数
    pub restitution: f32,
    /// 最后修改时间戳
    pub last_modified: u64,
}

impl Collider {
    /// 创建立方体碰撞体
    pub fn cuboid(id: ColliderId, half_extents: Vec3) -> Self {
        Self {
            id,
            shape_type: ShapeType::Cuboid,
            half_extents,
            radius: 0.0,
            offset: Vec3::ZERO,
            is_trigger: false,
            friction: 0.5,
            restitution: 0.0,
            last_modified: Self::current_timestamp(),
        }
    }

    /// 创建球体碰撞体
    pub fn ball(id: ColliderId, radius: f32) -> Self {
        Self {
            id,
            shape_type: ShapeType::Ball,
            half_extents: Vec3::ZERO,
            radius,
            offset: Vec3::ZERO,
            is_trigger: false,
            friction: 0.5,
            restitution: 0.0,
            last_modified: Self::current_timestamp(),
        }
    }

    /// 设置为触发器
    pub fn as_trigger(mut self) -> Self {
        self.is_trigger = true;
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 设置摩擦系数
    pub fn with_friction(mut self, friction: f32) -> Self {
        self.friction = friction.clamp(0.0, 1.0);
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 设置恢复系数
    pub fn with_restitution(mut self, restitution: f32) -> Self {
        self.restitution = restitution.clamp(0.0, 1.0);
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 设置位置偏移
    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 获取体积
    pub fn volume(&self) -> f32 {
        match self.shape_type {
            ShapeType::Cuboid => {
                8.0 * self.half_extents.x * self.half_extents.y * self.half_extents.z
            }
            ShapeType::Ball => 4.0 / 3.0 * std::f32::consts::PI * self.radius.powi(3),
        }
    }

    /// 验证碰撞体
    pub fn validate(&self) -> Result<(), DomainError> {
        match self.shape_type {
            ShapeType::Cuboid => {
                if self.half_extents.x <= 0.0
                    || self.half_extents.y <= 0.0
                    || self.half_extents.z <= 0.0
                {
                    return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                        format!(
                            "Invalid half extents for collider {}: {:?}",
                            self.id.as_u64(),
                            self.half_extents
                        ),
                    )));
                }
            }
            ShapeType::Ball => {
                if self.radius <= 0.0 {
                    return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                        format!(
                            "Invalid radius for collider {}: {}",
                            self.id.as_u64(),
                            self.radius
                        ),
                    )));
                }
            }
        }

        if !(0.0..=1.0).contains(&self.friction) {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                format!(
                    "Invalid friction for collider {}: {}",
                    self.id.as_u64(),
                    self.friction
                ),
            )));
        }

        if !(0.0..=1.0).contains(&self.restitution) {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                format!(
                    "Invalid restitution for collider {}: {}",
                    self.id.as_u64(),
                    self.restitution
                ),
            )));
        }

        Ok(())
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 物理世界 - 聚合根
/// 基于 Rapier3D 的完整物理模拟
pub struct PhysicsWorld {
    /// Rapier 物理管线
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
    /// 查询管线
    pub query_pipeline: QueryPipeline,
    /// 刚体集
    pub rigid_body_set: RigidBodySet,
    /// 碰撞体集
    pub collider_set: ColliderSet,
    /// 重力向量
    pub gravity: Vec3,
    /// 积分参数
    pub integration_parameters: IntegrationParameters,
    /// 刚体ID映射（领域对象ID -> Rapier句柄）
    pub body_handles: HashMap<RigidBodyId, RigidBodyHandle>,
    /// 碰撞体ID映射（领域对象ID -> Rapier句柄）
    pub collider_handles: HashMap<ColliderId, ColliderHandle>,
    /// 最后更新时间戳
    pub last_updated: u64,
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: Box::new(DefaultBroadPhase::new()),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            query_pipeline: QueryPipeline::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            body_handles: HashMap::new(),
            collider_handles: HashMap::new(),
            last_updated: Self::current_timestamp(),
        }
    }
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self::default()
    }

    /// 添加刚体
    pub fn add_body(&mut self, body: RigidBody) -> Result<(), DomainError> {
        body.validate()?;

        // 创建 Rapier 刚体
        let rb_builder = match body.body_type {
            RigidBodyType::Dynamic => RigidBodyBuilder::dynamic(),
            RigidBodyType::Fixed => RigidBodyBuilder::fixed(),
            RigidBodyType::KinematicPositionBased => RigidBodyBuilder::kinematic_position_based(),
            RigidBodyType::KinematicVelocityBased => RigidBodyBuilder::kinematic_velocity_based(),
        }
        .translation(vector![body.position.x, body.position.y, body.position.z])
        .linvel(vector![
            body.linear_velocity.x,
            body.linear_velocity.y,
            body.linear_velocity.z
        ])
        .angvel(vector![0.0, 0.0, body.angular_velocity]);

        let mut rb = rb_builder.build();

        // 设置旋转
        rb.set_rotation(
            UnitQuaternion::from_quaternion(Quaternion::new(
                body.rotation.w,
                body.rotation.x,
                body.rotation.y,
                body.rotation.z,
            )),
            true,
        );

        let handle = self.rigid_body_set.insert(rb);
        self.body_handles.insert(body.id, handle);

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 移除刚体
    pub fn remove_body(&mut self, id: RigidBodyId) -> Result<(), DomainError> {
        if let Some(handle) = self.body_handles.remove(&id) {
            self.rigid_body_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true,
            );
            Ok(())
        } else {
            Err(DomainError::Physics(PhysicsError::BodyNotFound(format!(
                "Body {}",
                id.as_u64()
            ))))
        }
    }

    /// 获取刚体状态（从 Rapier 获取）
    pub fn get_body_state(&self, id: RigidBodyId) -> Option<RigidBody> {
        let handle = self.body_handles.get(&id)?;
        let rb = self.rigid_body_set.get(*handle)?;

        let pos = rb.translation();
        let rot = rb.rotation();
        let linvel = rb.linvel();
        let angvel = rb.angvel();

        Some(RigidBody {
            id: id.clone(),
            body_type: match rb.body_type() {
                rapier3d::dynamics::RigidBodyType::Dynamic => RigidBodyType::Dynamic,
                rapier3d::dynamics::RigidBodyType::Fixed => RigidBodyType::Fixed,
                rapier3d::dynamics::RigidBodyType::KinematicPositionBased => {
                    RigidBodyType::KinematicPositionBased
                }
                rapier3d::dynamics::RigidBodyType::KinematicVelocityBased => {
                    RigidBodyType::KinematicVelocityBased
                }
            },
            position: Vec3::new(pos.x, pos.y, pos.z),
            rotation: Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w),
            linear_velocity: Vec3::new(linvel.x, linvel.y, linvel.z),
            angular_velocity: angvel.z, // 简化：只取z分量
            mass: rb.mass(),
            sleeping: rb.is_sleeping(),
            last_modified: Self::current_timestamp(),
            recovery_strategy: RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 50,
            },
        })
    }

    /// 更新刚体状态
    pub fn update_body(&mut self, body: &RigidBody) -> Result<(), DomainError> {
        if let Some(handle) = self.body_handles.get(&body.id) {
            if let Some(rb) = self.rigid_body_set.get_mut(*handle) {
                rb.set_translation(
                    vector![body.position.x, body.position.y, body.position.z],
                    true,
                );
                rb.set_rotation(
                    UnitQuaternion::new_normalize(Quaternion::new(
                        body.rotation.w,
                        body.rotation.x,
                        body.rotation.y,
                        body.rotation.z,
                    )),
                    true,
                );
                rb.set_linvel(
                    vector![
                        body.linear_velocity.x,
                        body.linear_velocity.y,
                        body.linear_velocity.z
                    ],
                    true,
                );
                rb.set_angvel(vector![0.0, 0.0, body.angular_velocity], true);
            }
        }
        Ok(())
    }

    /// 添加碰撞体到刚体
    pub fn add_collider_to_body(
        &mut self,
        collider: Collider,
        body_id: RigidBodyId,
    ) -> Result<(), DomainError> {
        collider.validate()?;

        let body_handle = self.body_handles.get(&body_id).ok_or_else(|| {
            DomainError::Physics(PhysicsError::BodyNotFound(format!(
                "Body {}",
                body_id.as_u64()
            )))
        })?;

        let shape = match collider.shape_type {
            ShapeType::Cuboid => SharedShape::cuboid(
                collider.half_extents.x,
                collider.half_extents.y,
                collider.half_extents.z,
            ),
            ShapeType::Ball => SharedShape::ball(collider.radius),
        };

        let collider_builder = ColliderBuilder::new(shape)
            .translation(vector![
                collider.offset.x,
                collider.offset.y,
                collider.offset.z
            ])
            .friction(collider.friction)
            .restitution(collider.restitution);

        let rapier_collider = collider_builder.build();
        let handle = self.collider_set.insert_with_parent(
            rapier_collider,
            *body_handle,
            &mut self.rigid_body_set,
        );
        self.collider_handles.insert(collider.id, handle);

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 移除碰撞体
    pub fn remove_collider(&mut self, id: ColliderId) -> Result<(), DomainError> {
        if let Some(handle) = self.collider_handles.remove(&id) {
            self.collider_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
            Ok(())
        } else {
            Err(DomainError::Physics(PhysicsError::ColliderNotFound(
                format!("Collider {}", id.as_u64()),
            )))
        }
    }

    /// 步进物理模拟（优化版本：支持并行处理）
    ///
    /// # 性能优化
    /// - Rapier内部自动并行处理物理岛屿
    /// - 宽相和窄相碰撞检测自动并行化
    /// - 优化积分参数以提升性能
    pub fn step(&mut self, delta_time: f32) -> Result<(), DomainError> {
        // 设置时间步长
        self.integration_parameters.dt = delta_time.max(0.001);

        // 优化积分参数以提升并行性能
        // Rapier内部会根据CPU核心数自动并行化
        // 减少CCD子步数以提升性能（对于大多数场景，1个子步足够）
        self.integration_parameters.max_ccd_substeps = 1;

        // 执行物理步进
        // Rapier的step()方法内部已经优化了并行处理：
        // - 岛屿管理器并行处理独立岛屿
        // - 宽相碰撞检测并行化
        // - 窄相碰撞检测并行化
        self.physics_pipeline.step(
            &vector![self.gravity.x, self.gravity.y, self.gravity.z],
            &self.integration_parameters,
            &mut self.island_manager,
            &mut *self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            Some(&mut self.query_pipeline),
            &(),
            &(),
        );

        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 获取活跃岛屿数量（性能统计，估算值）
    ///
    /// # 注意
    /// 这是通过动态刚体数量估算的近似值，实际岛屿数量可能不同
    pub fn active_island_count(&self) -> usize {
        let active_bodies = self
            .rigid_body_set
            .iter()
            .filter(|(_, rb)| rb.is_dynamic() && !rb.is_sleeping())
            .count();
        // 简化估算：假设每个岛屿平均有2-3个刚体
        (active_bodies / 2).max(1)
    }

    /// 获取碰撞对数量（性能统计，估算值）
    ///
    /// # 注意
    /// 这是通过活跃刚体数量估算的近似值，实际碰撞对数量可能不同
    pub fn collision_pair_count(&self) -> usize {
        let active_bodies = self
            .rigid_body_set
            .iter()
            .filter(|(_, rb)| rb.is_dynamic() && !rb.is_sleeping())
            .count();
        // 简化估算：假设每个动态刚体平均有1-2个碰撞对
        active_bodies
    }

    /// 验证世界状态
    pub fn validate(&self) -> Result<(), DomainError> {
        // 验证所有刚体句柄都有效
        for (id, handle) in &self.body_handles {
            if !self.rigid_body_set.contains(*handle) {
                return Err(DomainError::Physics(PhysicsError::BodyNotFound(format!(
                    "Body handle {:?} for body {} is invalid",
                    handle.0,
                    id.as_u64()
                ))));
            }
        }

        // 验证所有碰撞体句柄都有效
        for (id, handle) in &self.collider_handles {
            if !self.collider_set.contains(*handle) {
                return Err(DomainError::Physics(PhysicsError::ColliderNotFound(
                    format!(
                        "Collider handle {:?} for collider {} is invalid",
                        handle.0,
                        id.as_u64()
                    ),
                )));
            }
        }

        Ok(())
    }

    /// 射线投射
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(RigidBodyId, f32, Vec3)> {
        let ray = Ray::new(
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z],
        );

        if let Some((handle, _toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_distance,
            true,
            QueryFilter::default(),
        ) {
            let hit_point = ray.point_at(_toi);

            // 找到对应的领域对象ID
            for (body_id, body_handle) in &self.body_handles {
                if body_handle.0 == handle.0 {
                    return Some((
                        *body_id,
                        _toi,
                        Vec3::new(hit_point.x, hit_point.y, hit_point.z),
                    ));
                }
            }
        }
        None
    }

    /// 设置重力
    pub fn set_gravity(&mut self, gravity: Vec3) {
        self.gravity = gravity;
    }

    /// 获取重力
    pub fn get_gravity(&self) -> Vec3 {
        self.gravity
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rigid_body_creation() {
        let body = RigidBody::dynamic(RigidBodyId(1), Vec3::new(0.0, 0.0, 0.0));
        assert_eq!(body.id, RigidBodyId(1));
        assert_eq!(body.body_type, RigidBodyType::Dynamic);
        assert_eq!(body.mass, 1.0);
    }

    #[test]
    fn test_rigid_body_apply_force() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);

        body.apply_force(Vec3::new(10.0, 0.0, 0.0)).unwrap();
        assert!(body.linear_velocity.x > 0.0);
        assert!(!body.sleeping);
    }

    #[test]
    fn test_collider_creation() {
        let cuboid = Collider::cuboid(ColliderId(1), Vec3::new(1.0, 1.0, 1.0));
        assert_eq!(cuboid.shape_type, ShapeType::Cuboid);
        assert_eq!(cuboid.volume(), 8.0);

        let ball = Collider::ball(ColliderId(2), 1.0);
        assert_eq!(ball.shape_type, ShapeType::Ball);
        assert!(ball.volume() > 4.0);
    }

    #[test]
    fn test_physics_world() {
        let mut world = PhysicsWorld::new();

        let body = RigidBody::dynamic(RigidBodyId(1), Vec3::new(0.0, 10.0, 0.0));
        world.add_body(body).unwrap();

        let collider = Collider::ball(ColliderId(1), 0.5);
        world
            .add_collider_to_body(collider, RigidBodyId(1))
            .unwrap();

        // 步进几次
        for _ in 0..10 {
            world.step(1.0 / 60.0).unwrap();
        }

        let body_state = world.get_body_state(RigidBodyId(1)).unwrap();
        // 由于重力，Y位置应该下降
        assert!(body_state.position.y < 10.0);
    }

    #[test]
    fn test_raycast() {
        let mut world = PhysicsWorld::new();

        // 创建地面
        let ground_body = RigidBody::fixed(RigidBodyId(1), Vec3::new(0.0, 0.0, 0.0));
        world.add_body(ground_body).unwrap();

        let ground_collider = Collider::cuboid(ColliderId(1), Vec3::new(10.0, 0.1, 10.0));
        world
            .add_collider_to_body(ground_collider, RigidBodyId(1))
            .unwrap();

        // 更新查询管线
        world.query_pipeline.update(&world.collider_set);

        // 从上方投射射线
        let result = world.raycast(Vec3::new(0.0, 10.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 20.0);

        assert!(result.is_some());
        let (body_id, _toi, hit_point) = result.unwrap();
        assert_eq!(body_id, RigidBodyId(1));
        assert!(hit_point.y < 1.0); // 应该击中地面附近
    }

    #[test]
    fn test_rigid_body_apply_force_fixed_body() {
        // 测试业务规则：固定刚体不能应用力
        let mut body = RigidBody::fixed(RigidBodyId(1), Vec3::ZERO);
        assert!(body.apply_force(Vec3::ONE).is_err());
    }

    #[test]
    fn test_rigid_body_apply_impulse_fixed_body() {
        // 测试业务规则：固定刚体不能应用冲量
        let mut body = RigidBody::fixed(RigidBodyId(1), Vec3::ZERO);
        assert!(body.apply_impulse(Vec3::ONE).is_err());
    }

    #[test]
    fn test_rigid_body_set_velocity_fixed_body() {
        // 测试业务规则：固定刚体不能设置速度
        let mut body = RigidBody::fixed(RigidBodyId(1), Vec3::ZERO);
        assert!(body.set_linear_velocity(Vec3::ONE).is_err());
        assert!(body.set_angular_velocity(1.0).is_err());
    }

    #[test]
    fn test_rigid_body_set_mass_invalid() {
        // 测试业务规则：质量必须为正数
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        assert!(body.set_mass(0.0).is_err());
        assert!(body.set_mass(-1.0).is_err());
    }

    #[test]
    fn test_rigid_body_validate() {
        // 测试刚体验证
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        assert!(body.validate().is_ok());
        
        // 无效质量
        body.mass = 0.0;
        assert!(body.validate().is_err());
        
        // 无效位置
        body.mass = 1.0;
        body.position = Vec3::new(f32::NAN, 0.0, 0.0);
        assert!(body.validate().is_err());
    }

    #[test]
    fn test_rigid_body_sleep_and_wake() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        
        // 初始状态应该是醒着的
        assert!(!body.sleeping);
        
        // 让刚体休眠
        body.sleep();
        assert!(body.sleeping);
        assert_eq!(body.linear_velocity, Vec3::ZERO);
        assert_eq!(body.angular_velocity, 0.0);
        
        // 唤醒刚体
        body.wake_up();
        assert!(!body.sleeping);
    }

    #[test]
    fn test_rigid_body_can_sleep() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        
        // 速度很小，应该可以休眠（length_squared < 0.01）
        body.linear_velocity = Vec3::new(0.05, 0.0, 0.0); // length_squared = 0.0025 < 0.01
        body.angular_velocity = 0.005; // < 0.01
        assert!(body.can_sleep());
        
        // 速度较大，不应该休眠
        body.linear_velocity = Vec3::new(1.0, 0.0, 0.0); // length_squared = 1.0 > 0.01
        assert!(!body.can_sleep());
    }

    #[test]
    fn test_rigid_body_momentum() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.set_mass(2.0).unwrap();
        body.set_linear_velocity(Vec3::new(3.0, 0.0, 0.0)).unwrap();
        
        let momentum = body.momentum();
        assert_eq!(momentum, Vec3::new(6.0, 0.0, 0.0)); // mass * velocity
    }

    #[test]
    fn test_rigid_body_kinetic_energy() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.set_mass(2.0).unwrap();
        body.set_linear_velocity(Vec3::new(3.0, 0.0, 0.0)).unwrap();
        
        let energy = body.kinetic_energy();
        assert_eq!(energy, 0.5 * 2.0 * 9.0); // 0.5 * m * v^2
    }

    #[test]
    fn test_collider_validate_cuboid() {
        // 测试立方体碰撞体验证
        let collider = Collider::cuboid(ColliderId(1), Vec3::new(1.0, 1.0, 1.0));
        assert!(collider.validate().is_ok());
        
        // 无效的半尺寸
        let mut invalid = Collider::cuboid(ColliderId(2), Vec3::new(-1.0, 1.0, 1.0));
        assert!(invalid.validate().is_err());
        
        invalid.half_extents = Vec3::new(0.0, 1.0, 1.0);
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_collider_validate_ball() {
        // 测试球体碰撞体验证
        let collider = Collider::ball(ColliderId(1), 1.0);
        assert!(collider.validate().is_ok());
        
        // 无效的半径
        let mut invalid = Collider::ball(ColliderId(2), 0.0);
        assert!(invalid.validate().is_err());
        
        invalid.radius = -1.0;
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_collider_friction_clamp() {
        // 测试摩擦系数会被限制在0-1范围内
        let collider = Collider::cuboid(ColliderId(1), Vec3::ONE)
            .with_friction(1.5); // 超出范围
        
        assert_eq!(collider.friction, 1.0); // 应该被限制为1.0
    }

    #[test]
    fn test_collider_restitution_clamp() {
        // 测试恢复系数会被限制在0-1范围内
        let collider = Collider::cuboid(ColliderId(1), Vec3::ONE)
            .with_restitution(-0.5); // 超出范围
        
        assert_eq!(collider.restitution, 0.0); // 应该被限制为0.0
    }

    #[test]
    fn test_rigid_body_id_creation() {
        let id = RigidBodyId::new(42);
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn test_collider_id_creation() {
        let id = ColliderId::new(42);
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn test_rigid_body_set_position() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.set_position(Vec3::new(10.0, 20.0, 30.0)).unwrap();
        assert_eq!(body.position, Vec3::new(10.0, 20.0, 30.0));
        assert!(!body.sleeping); // 应该被唤醒
    }

    #[test]
    fn test_rigid_body_set_rotation() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        let rotation = Quat::from_euler(glam::EulerRot::XYZ, 1.0, 0.0, 0.0);
        body.set_rotation(rotation).unwrap();
        assert_eq!(body.rotation, rotation);
        assert!(!body.sleeping); // 应该被唤醒
    }

    #[test]
    fn test_rigid_body_kinematic_position_based() {
        let body = RigidBody::new(
            RigidBodyId(1),
            RigidBodyType::KinematicPositionBased,
            Vec3::ZERO,
        );
        assert_eq!(body.body_type, RigidBodyType::KinematicPositionBased);
    }

    #[test]
    fn test_rigid_body_kinematic_velocity_based() {
        let body = RigidBody::new(
            RigidBodyId(1),
            RigidBodyType::KinematicVelocityBased,
            Vec3::ZERO,
        );
        assert_eq!(body.body_type, RigidBodyType::KinematicVelocityBased);
    }

    #[test]
    fn test_collider_as_trigger() {
        let collider = Collider::cuboid(ColliderId(1), Vec3::ONE)
            .as_trigger();
        assert!(collider.is_trigger);
    }

    #[test]
    fn test_collider_with_offset() {
        let collider = Collider::cuboid(ColliderId(1), Vec3::ONE)
            .with_offset(Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(collider.offset, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_collider_volume_cuboid() {
        let collider = Collider::cuboid(ColliderId(1), Vec3::new(1.0, 1.0, 1.0));
        // 体积 = 8 * half_extents.x * half_extents.y * half_extents.z
        assert_eq!(collider.volume(), 8.0);
    }

    #[test]
    fn test_collider_volume_ball() {
        let collider = Collider::ball(ColliderId(1), 1.0);
        // 体积 = 4/3 * PI * r^3
        let expected_volume = 4.0 / 3.0 * std::f32::consts::PI;
        assert!((collider.volume() - expected_volume).abs() < 0.01);
    }

    // ============================================================================
    // 错误恢复和补偿操作测试
    // ============================================================================

    #[test]
    fn test_rigid_body_recover_from_error_invalid_parameter() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1, // 使用最小延迟以加快测试
        };
        
        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(body.mass, 1.0);
        assert_eq!(body.position, Vec3::ZERO);
    }

    #[test]
    fn test_rigid_body_recover_from_error_body_not_found() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = PhysicsError::BodyNotFound("test".to_string());
        let result = body.recover_from_error(&error);
        
        // BodyNotFound错误无法恢复，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_rigid_body_recover_from_error_use_default() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::new(10.0, 20.0, 30.0));
        body.mass = 5.0;
        body.linear_velocity = Vec3::new(1.0, 2.0, 3.0);
        body.angular_velocity = 4.0;
        body.recovery_strategy = RecoveryStrategy::UseDefault;
        
        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(body.mass, 1.0);
        assert_eq!(body.linear_velocity, Vec3::ZERO);
        assert_eq!(body.angular_velocity, 0.0);
    }

    #[test]
    fn test_rigid_body_recover_from_error_skip() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.mass = 5.0;
        body.recovery_strategy = RecoveryStrategy::Skip;
        
        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(body.mass, 5.0); // 状态不应该改变
    }

    #[test]
    fn test_rigid_body_recover_from_error_log_and_continue() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.mass = 5.0;
        body.recovery_strategy = RecoveryStrategy::LogAndContinue;
        
        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(body.mass, 5.0); // 状态不应该改变
    }

    #[test]
    fn test_rigid_body_recover_from_error_fail() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.recovery_strategy = RecoveryStrategy::Fail;
        
        let error = PhysicsError::InvalidParameter("test".to_string());
        let result = body.recover_from_error(&error);
        
        assert!(result.is_err());
        if let Err(DomainError::Physics(e)) = result {
            assert!(matches!(e, PhysicsError::InvalidParameter(_)));
        } else {
            panic!("Expected Physics error");
        }
    }

    #[test]
    fn test_rigid_body_create_compensation() {
        let body = RigidBody::dynamic(RigidBodyId(1), Vec3::new(1.0, 2.0, 3.0));
        let compensation = body.create_compensation();
        
        assert_eq!(compensation.action_type, "restore_physics_state");
        assert!(compensation.data.get("position").is_some());
        assert!(compensation.data.get("mass").is_some());
    }

    #[test]
    fn test_rigid_body_restore_from_compensation() {
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::new(1.0, 2.0, 3.0));
        body.mass = 5.0;
        body.linear_velocity = Vec3::new(4.0, 5.0, 6.0);
        body.angular_velocity = 7.0;
        body.sleeping = true;
        
        let compensation = body.create_compensation();
        
        // 修改状态
        body.position = Vec3::new(10.0, 20.0, 30.0);
        body.mass = 100.0;
        body.sleeping = false;
        
        // 恢复状态
        body.restore_from_compensation(&compensation).unwrap();
        
        assert_eq!(body.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(body.mass, 5.0);
        assert_eq!(body.sleeping, true);
    }

    #[test]
    fn test_rigid_body_restore_from_compensation_partial() {
        // 测试部分数据缺失的情况
        let mut body = RigidBody::dynamic(RigidBodyId(1), Vec3::ZERO);
        body.mass = 5.0;
        
        // 创建一个只有部分数据的补偿操作
        let compensation = CompensationAction::new(
            "test",
            "restore_physics_state",
            serde_json::json!({
                "mass": 10.0,
                // 其他字段缺失
            }),
        );
        
        body.restore_from_compensation(&compensation).unwrap();
        assert_eq!(body.mass, 10.0);
    }
}
