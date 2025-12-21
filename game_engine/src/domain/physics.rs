//  物理领域对象
//  实现富领域对象设计模式，将物理相关的业务逻辑封装到领域对象中

// 移除未使用的EntityId导入，如果将来需要可以重新导入
use crate::domain::errors::{CompensationAction, DomainError, PhysicsError, RecoveryStrategy};
use crate::error::safe_lock;
// 移除未使用的Transform导入，如果将来需要可以重新导入
use glam::{Quat, Vec3};
use rapier3d::na::{Point3, Quaternion, UnitQuaternion, Vector3};
use rapier3d::prelude::*;
// use rapier3d::pipeline::QueryPipeline; // Temporarily disabled
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

/// 刚体类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RigidBodyType {
    /// 静态刚体，不受力影响，不能移动
    Fixed,
    /// 动态刚体，受力影响，可以移动
    Dynamic,
    /// 运动学刚体，可以被直接控制移动，但不受力影响
    Kinematic,
}

/// 形状类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShapeType {
    /// 球形
    Sphere { radius: f32 },
    /// 球体
    Ball { radius: f32 },
    /// 立方体
    Cuboid { half_extents: Vec3 },
    /// 胶囊体
    Capsule { radius: f32, height: f32 },
    /// 圆柱体
    Cylinder { radius: f32, height: f32 },
    /// 锥体
    Cone { radius: f32, height: f32 },
    /// 凸多边形
    ConvexHull { points: Vec<Vec3> },
    /// 三角网格
    TriMesh {
        vertices: Vec<Vec3>,
        indices: Vec<[u32; 3]>,
    },
}

/// 刚体状态
#[derive(Debug, Clone)]
pub struct RigidBodyState {
    /// 位置
    pub position: Vec3,
    /// 旋转
    pub rotation: Quat,
    /// 线性速度
    pub linear_velocity: Vec3,
    /// 角速度
    pub angular_velocity: Vec3,
    /// 是否休眠
    pub sleeping: bool,
}

/// 刚体ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RigidBodyId(pub u64);

impl RigidBodyId {
    /// 创建新的刚体ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取ID值
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// 碰撞体ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColliderId(pub u64);

impl ColliderId {
    /// 创建新的碰撞体ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取ID值
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// 刚体领域对象
#[derive(Debug, Clone)]
pub struct RigidBody {
    /// 刚体ID
    pub(crate) id: RigidBodyId,
    /// 刚体类型
    pub(crate) body_type: RigidBodyType,
    /// 位置
    pub(crate) position: Vec3,
    /// 旋转
    pub(crate) rotation: Quat,
    /// 线性速度
    pub(crate) linear_velocity: Vec3,
    /// 角速度
    pub(crate) angular_velocity: Vec3,
    /// 质量
    pub(crate) mass: f32,
    /// 摩擦系数
    pub(crate) friction: f32,
    /// 弹性系数
    pub(crate) restitution: f32,
    /// 是否处于休眠状态
    pub(crate) sleeping: bool,
    /// 错误恢复策略
    pub(crate) recovery_strategy: RecoveryStrategy,
}

impl RigidBody {
    /// 创建新的刚体（默认旋转和质量）
    pub fn new(
        id: RigidBodyId,
        body_type: RigidBodyType,
        position: Vec3,
    ) -> Self {
        Self::with_all(id, body_type, position, Quat::IDENTITY, 1.0)
    }

    /// 创建新的刚体（完整参数）
    pub fn with_all(
        id: RigidBodyId,
        body_type: RigidBodyType,
        position: Vec3,
        rotation: Quat,
        mass: f32,
    ) -> Self {
        Self {
            id,
            body_type,
            position,
            rotation,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass,
            friction: 0.5,
            restitution: 0.3,
            sleeping: false,
            recovery_strategy: RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 100,
            },
        }
    }

    /// 创建动态刚体（为了兼容测试代码）
    pub fn dynamic(id: RigidBodyId, position: Vec3) -> Self {
        Self::new(id, RigidBodyType::Dynamic, position)
    }

    /// 获取刚体ID
    pub fn id(&self) -> RigidBodyId {
        self.id
    }

    /// 获取刚体类型
    pub fn body_type(&self) -> RigidBodyType {
        self.body_type
    }

    /// 获取位置
    pub fn position(&self) -> Vec3 {
        self.position
    }

    /// 获取旋转
    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    /// 获取线性速度
    pub fn linear_velocity(&self) -> Vec3 {
        self.linear_velocity
    }

    /// 获取角速度
    pub fn angular_velocity(&self) -> Vec3 {
        self.angular_velocity
    }

    /// 获取质量
    pub fn mass(&self) -> f32 {
        self.mass
    }

    /// 设置质量
    pub fn set_mass(&mut self, mass: f32) -> Result<(), DomainError> {
        if mass <= 0.0 {
            return Err(DomainError::Physics(PhysicsError::InvalidParameter(
                "Mass must be positive".to_string(),
            )));
        }
        self.mass = mass;
        Ok(())
    }

    /// 获取摩擦系数
    pub fn friction(&self) -> f32 {
        self.friction
    }

    /// 获取弹性系数
    pub fn restitution(&self) -> f32 {
        self.restitution
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
                self.angular_velocity = Vec3::ZERO;
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
                "angular_velocity": [self.angular_velocity.x, self.angular_velocity.y, self.angular_velocity.z],
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

        if let Some(rot) = action.data.get("rotation").and_then(|v| v.as_array()) {
            if rot.len() == 4 {
                self.rotation = Quat::from_xyzw(
                    rot[0].as_f64().unwrap_or(0.0) as f32,
                    rot[1].as_f64().unwrap_or(0.0) as f32,
                    rot[2].as_f64().unwrap_or(0.0) as f32,
                    rot[3].as_f64().unwrap_or(1.0) as f32,
                );
            }
        }

        if let Some(lv) = action.data.get("linear_velocity").and_then(|v| v.as_array()) {
            if lv.len() == 3 {
                self.linear_velocity = Vec3::new(
                    lv[0].as_f64().unwrap_or(0.0) as f32,
                    lv[1].as_f64().unwrap_or(0.0) as f32,
                    lv[2].as_f64().unwrap_or(0.0) as f32,
                );
            }
        }

        if let Some(av) = action.data.get("angular_velocity").and_then(|v| v.as_array()) {
            if av.len() == 3 {
                self.angular_velocity = Vec3::new(
                    av[0].as_f64().unwrap_or(0.0) as f32,
                    av[1].as_f64().unwrap_or(0.0) as f32,
                    av[2].as_f64().unwrap_or(0.0) as f32,
                );
            }
        }

        if let Some(mass) = action.data.get("mass").and_then(|v| v.as_f64()) {
            self.mass = mass as f32;
        }

        if let Some(sleeping) = action.data.get("sleeping").and_then(|v| v.as_bool()) {
            self.sleeping = sleeping;
        }

        Ok(())
    }

    /// 设置位置
    pub fn set_position(&mut self, position: Vec3) {
        self.position = position;
    }

    /// 设置旋转
    pub fn set_rotation(&mut self, rotation: Quat) {
        self.rotation = rotation;
    }

    /// 设置线性速度
    pub fn set_linear_velocity(&mut self, velocity: Vec3) {
        self.linear_velocity = velocity;
    }

    /// 设置角速度
    pub fn set_angular_velocity(&mut self, velocity: Vec3) {
        self.angular_velocity = velocity;
    }

    /// 设置摩擦系数
    pub fn set_friction(&mut self, friction: f32) {
        self.friction = friction;
    }

    /// 设置弹性系数
    pub fn set_restitution(&mut self, restitution: f32) {
        self.restitution = restitution;
    }
}

/// 碰撞体领域对象
#[derive(Debug, Clone)]
pub struct Collider {
    /// 碰撞体ID
    pub(crate) id: ColliderId,
    /// 关联的刚体ID
    pub(crate) body_id: RigidBodyId,
    /// 形状类型
    pub(crate) shape_type: ShapeType,
    /// 密度
    pub(crate) density: f32,
    /// 摩擦系数
    pub(crate) friction: f32,
    /// 弹性系数
    pub(crate) restitution: f32,
}

impl Collider {
    /// 创建新的碰撞体
    pub fn new(id: ColliderId, body_id: RigidBodyId, shape_type: ShapeType, density: f32) -> Self {
        Self {
            id,
            body_id,
            shape_type,
            density,
            friction: 0.5,
            restitution: 0.3,
        }
    }

    /// 创建立方体碰撞体
    pub fn cuboid(id: ColliderId, half_extents: Vec3) -> Self {
        Self {
            id,
            body_id: RigidBodyId::new(0), // 临时ID，实际使用时会被替换
            shape_type: ShapeType::Cuboid { half_extents },
            density: 1.0,
            friction: 0.5,
            restitution: 0.3,
        }
    }

    /// 创建球体碰撞体
    pub fn ball(id: ColliderId, radius: f32) -> Self {
        Self {
            id,
            body_id: RigidBodyId::new(0), // 临时ID，实际使用时会被替换
            shape_type: ShapeType::Ball { radius },
            density: 1.0,
            friction: 0.5,
            restitution: 0.3,
        }
    }

    /// 获取立方体半长宽
    pub fn half_extents(&self) -> Vec3 {
        if let ShapeType::Cuboid { half_extents } = &self.shape_type {
            *half_extents
        } else {
            Vec3::ZERO
        }
    }

    /// 获取球体半径
    pub fn radius(&self) -> f32 {
        match &self.shape_type {
            ShapeType::Ball { radius } => *radius,
            ShapeType::Sphere { radius } => *radius,
            _ => 0.0,
        }
    }

    /// 获取碰撞体ID
    pub fn id(&self) -> ColliderId {
        self.id
    }

    /// 获取关联的刚体ID
    pub fn body_id(&self) -> RigidBodyId {
        self.body_id
    }

    /// 获取形状类型
    pub fn shape_type(&self) -> ShapeType {
        self.shape_type.clone()
    }

    /// 获取密度
    pub fn density(&self) -> f32 {
        self.density
    }

    /// 获取摩擦系数
    pub fn friction(&self) -> f32 {
        self.friction
    }

    /// 获取弹性系数
    pub fn restitution(&self) -> f32 {
        self.restitution
    }

    /// 设置摩擦系数
    pub fn set_friction(&mut self, friction: f32) {
        self.friction = friction;
    }

    /// 设置弹性系数
    pub fn set_restitution(&mut self, restitution: f32) {
        self.restitution = restitution;
    }
}

/// 物理世界领域对象
pub struct PhysicsWorld {
    /// 重力
    gravity: Vector<Real>,
    /// 积分参数
    integration_parameters: IntegrationParameters,
    /// 物理流水线
    physics_pipeline: Mutex<PhysicsPipeline>,
    /// 岛屿管理器
    island_manager: IslandManager,
    /// 广相位
    broad_phase: DefaultBroadPhase,
    /// 窄相位
    narrow_phase: NarrowPhase,
    /// 冲量关节集
    impulse_joint_set: ImpulseJointSet,
    /// 多体关节集
    multibody_joint_set: MultibodyJointSet,
    /// CCD求解器
    ccd_solver: CCDSolver,
    /// 刚体集
    rigid_body_set: RigidBodySet,
    /// 碰撞体集
    collider_set: ColliderSet,
    /// 查询流水线 (暂时禁用)
    // query_pipeline: QueryPipeline,
    /// 刚体句柄映射
    pub(crate) body_handles: HashMap<RigidBodyId, RigidBodyHandle>,
    /// 碰撞体句柄映射
    pub(crate) collider_handles: HashMap<ColliderId, ColliderHandle>,
}

impl PhysicsWorld {
    /// 创建新的物理世界
    pub fn new() -> Self {
        Self {
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: Mutex::new(PhysicsPipeline::new()),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            // query_pipeline: QueryPipeline::new(), // Temporarily disabled
            body_handles: HashMap::new(),
            collider_handles: HashMap::new(),
        }
    }

    /// 添加刚体
    pub fn add_body(&mut self, body: RigidBody) -> Result<RigidBodyHandle, PhysicsError> {
        let rb = rapier3d::prelude::RigidBodyBuilder::new(match body.body_type() {
            RigidBodyType::Fixed => rapier3d::prelude::RigidBodyType::Fixed,
            RigidBodyType::Dynamic => rapier3d::prelude::RigidBodyType::Dynamic,
            RigidBodyType::Kinematic => rapier3d::prelude::RigidBodyType::KinematicPositionBased,
        })
        .pose(Isometry::from_parts(
            Translation::new(body.position().x, body.position().y, body.position().z),
            UnitQuaternion::from_quaternion(Quaternion::new(
                body.rotation().w,
                body.rotation().x,
                body.rotation().y,
                body.rotation().z,
            )),
        ))
        .linvel(
            [
                body.linear_velocity().x,
                body.linear_velocity().y,
                body.linear_velocity().z,
            ]
            .into(),
        )
        .angvel(
            [
                body.angular_velocity().x,
                body.angular_velocity().y,
                body.angular_velocity().z,
            ]
            .into(),
        )
        .additional_mass(body.mass())
        .build();

        let handle = self.rigid_body_set.insert(rb);
        self.body_handles.insert(body.id(), handle);
        Ok(handle)
    }

    /// 获取刚体只读引用
    pub fn get_body(&self, id: RigidBodyId) -> Option<&rapier3d::prelude::RigidBody> {
        if let Some(handle) = self.body_handles.get(&id) {
            self.rigid_body_set.get(*handle)
        } else {
            None
        }
    }

    /// 获取刚体可变引用
    pub fn get_body_mut(&mut self, id: RigidBodyId) -> Option<&mut rapier3d::prelude::RigidBody> {
        if let Some(handle) = self.body_handles.get(&id) {
            self.rigid_body_set.get_mut(*handle)
        } else {
            None
        }
    }

    /// 移除刚体
    pub fn remove_body(&mut self, id: RigidBodyId) -> Result<RigidBodyHandle, PhysicsError> {
        if let Some(handle) = self.body_handles.remove(&id) {
            self.rigid_body_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.collider_set,
                &mut self.impulse_joint_set,
                &mut self.multibody_joint_set,
                true, // remove attached colliders
            );
            Ok(handle)
        } else {
            Err(PhysicsError::BodyNotFound(format!("Body {}", id.as_u64())))
        }
    }

    /// 添加碰撞体到刚体
    pub fn add_collider_to_body(
        &mut self,
        collider: Collider,
        body_id: RigidBodyId,
    ) -> Result<ColliderHandle, PhysicsError> {
        // 获取刚体句柄
        let body_handle = *self.body_handles.get(&body_id).ok_or_else(|| {
            PhysicsError::BodyNotFound(format!(
                "Body {} for collider {}",
                body_id.as_u64(),
                collider.id().as_u64()
            ))
        })?;

        // 创建Rapier形状
        let shape: SharedShape = match collider.shape_type() {
            ShapeType::Sphere { radius } | ShapeType::Ball { radius } => SharedShape::ball(radius),
            ShapeType::Cuboid { half_extents } => {
                SharedShape::cuboid(half_extents.x, half_extents.y, half_extents.z)
            }
            ShapeType::Capsule { radius, height } => SharedShape::capsule_y(height / 2.0, radius),
            ShapeType::Cylinder { radius, height } => SharedShape::cylinder(height / 2.0, radius),
            ShapeType::Cone { radius, height } => SharedShape::cone(height / 2.0, radius),
            ShapeType::ConvexHull { points } => {
                let points: Vec<_> = points.iter().map(|p| Point3::new(p.x, p.y, p.z)).collect();
                SharedShape::convex_hull(&points).ok_or(PhysicsError::InvalidShape(
                    "Failed to create convex hull".to_string(),
                ))?
            }
            ShapeType::TriMesh { vertices, indices } => {
                let vertices: Vec<_> = vertices
                    .iter()
                    .map(|v| Point3::new(v.x, v.y, v.z))
                    .collect();
                let indices: Vec<_> = indices.iter().map(|i| [i[0], i[1], i[2]]).collect();
                SharedShape::trimesh(vertices, indices)
                    .map_err(|e| PhysicsError::ShapeCreationError(format!("Failed to create trimesh: {}", e)))?
            }
        };

        // 创建碰撞体
        let coll = ColliderBuilder::new(shape)
            .density(collider.density())
            .friction(collider.friction())
            .restitution(collider.restitution())
            .build();

        // 添加到物理世界
        let handle =
            self.collider_set
                .insert_with_parent(coll, body_handle, &mut self.rigid_body_set);
        self.collider_handles.insert(collider.id(), handle);
        Ok(handle)
    }

    /// 移除碰撞体
    pub fn remove_collider(&mut self, id: ColliderId) -> Result<ColliderHandle, PhysicsError> {
        if let Some(handle) = self.collider_handles.remove(&id) {
            self.collider_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
            Ok(handle)
        } else {
            Err(PhysicsError::ColliderNotFound(format!(
                "Collider {}",
                id.as_u64()
            )))
        }
    }

    /// 获取刚体状态
    pub fn get_body_state(&self, id: RigidBodyId) -> Option<RigidBodyState> {
        if let Some(handle) = self.body_handles.get(&id) {
            if let Some(rb) = self.rigid_body_set.get(*handle) {
                let position =
                    Vec3::new(rb.translation().x, rb.translation().y, rb.translation().z);
                let rotation = Quat::from_xyzw(
                    rb.rotation().i,
                    rb.rotation().j,
                    rb.rotation().k,
                    rb.rotation().w,
                );
                let linear_velocity = Vec3::new(rb.linvel().x, rb.linvel().y, rb.linvel().z);
                let angular_velocity = Vec3::new(rb.angvel().x, rb.angvel().y, rb.angvel().z);
                let sleeping = rb.is_sleeping();

                Some(RigidBodyState {
                    position,
                    rotation,
                    linear_velocity,
                    angular_velocity,
                    sleeping,
                })
            } else {
                None
            }
        } else {
            None
        }
    }

    /// 步进模拟
    pub fn step(&mut self, delta_time: f32) -> Result<(), PhysicsError> {
        // 更新积分参数的时间步长
        self.integration_parameters.dt = delta_time;

        // 执行物理步进
        let mut physics_pipeline = safe_lock(&self.physics_pipeline, "PhysicsWorld.physics_pipeline")
            .map_err(|e| PhysicsError::LockError(format!("Failed to acquire physics pipeline lock: {}", e)))?;
        physics_pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &(),
            &(),
        );

        // 更新查询流水线，以支持射线投射等查询，实现逻辑闭环
        // self.query_pipeline.update(&self.rigid_body_set, &self.collider_set);

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
            Point3::new(origin.x, origin.y, origin.z),
            Vector3::new(direction.x, direction.y, direction.z),
        );

        // 简化的射线投射实现，遍历所有碰撞体进行相交测试
        // 这是一个临时实现，直到 QueryPipeline 问题解决
        let mut closest_hit: Option<(RigidBodyId, f32, Vec3)> = None;
        let mut closest_distance = f32::INFINITY;

        // 遍历所有碰撞体进行相交测试
        for (_collider_handle, collider) in self.collider_set.iter() {
            if let Some(ball) = collider.shape().as_ball() {
                let collider_pos = collider.position();
                let ball_center = Point3::new(collider_pos.translation.x, collider_pos.translation.y, collider_pos.translation.z);
                let distance_to_center = (ball_center - ray.origin).magnitude();

                if distance_to_center <= ball.radius + max_distance {
                    let distance = distance_to_center - ball.radius;
                    if distance >= 0.0 && distance < closest_distance && distance <= max_distance {
                        // 通过句柄反查 RigidBodyId
                        if let Some(parent_handle) = collider.parent() {
                            for (id, &h) in self.body_handles.iter() {
                                if h == parent_handle {
                                    let hit_point = origin + direction * distance;
                                    closest_hit = Some((*id, distance, hit_point));
                                    closest_distance = distance;
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        closest_hit
    }

    /// 创建刚体（与add_body功能相同，为了兼容测试代码）
    pub fn create_body(&mut self, body: RigidBody) -> Result<RigidBodyHandle, PhysicsError> {
        self.add_body(body)
    }

    /// 创建碰撞体（与add_collider_to_body功能相同，为了兼容测试代码）
    pub fn create_collider(
        &mut self,
        collider: Collider,
        body_id: RigidBodyId,
    ) -> Result<ColliderHandle, PhysicsError> {
        self.add_collider_to_body(collider, body_id)
    }

    /// 销毁碰撞体（与remove_collider功能相同，为了兼容测试代码）
    pub fn destroy_collider(&mut self, id: ColliderId) -> Result<ColliderHandle, PhysicsError> {
        self.remove_collider(id)
    }

    /// 应用冲量到刚体
    pub fn apply_impulse(&mut self, id: RigidBodyId, impulse: Vec3) -> Result<(), PhysicsError> {
        if let Some(handle) = self.body_handles.get(&id) {
            if let Some(rb) = self.rigid_body_set.get_mut(*handle) {
                rb.apply_impulse(Vector3::new(impulse.x, impulse.y, impulse.z), true);
                Ok(())
            } else {
                Err(PhysicsError::BodyNotFound(format!("Body {}", id.as_u64())))
            }
        } else {
            Err(PhysicsError::BodyNotFound(format!("Body {}", id.as_u64())))
        }
    }

    /// 设置刚体位置
    pub fn set_body_position(
        &mut self,
        id: RigidBodyId,
        position: Vec3,
    ) -> Result<(), PhysicsError> {
        if let Some(handle) = self.body_handles.get(&id) {
            if let Some(rb) = self.rigid_body_set.get_mut(*handle) {
                let translation = Translation::new(position.x, position.y, position.z);
                let rotation = rb.rotation();
                rb.set_position(Isometry::from_parts(translation, *rotation), true);
                Ok(())
            } else {
                Err(PhysicsError::BodyNotFound(format!("Body {}", id.as_u64())))
            }
        } else {
            Err(PhysicsError::BodyNotFound(format!("Body {}", id.as_u64())))
        }
    }

    /// 获取刚体位置
    pub fn get_body_position(&self, id: RigidBodyId) -> Result<Vec3, PhysicsError> {
        if let Some(state) = self.get_body_state(id) {
            Ok(state.position)
        } else {
            Err(PhysicsError::BodyNotFound(format!("Body {}", id.as_u64())))
        }
    }

    /// 获取物理世界引用
    pub fn get_world(&self) -> &PhysicsWorld {
        self
    }

    /// 获取物理世界可变引用
    pub fn get_world_mut(&mut self) -> &mut PhysicsWorld {
        self
    }

    /// 销毁刚体（与remove_body功能相同，为了兼容测试代码）
    pub fn destroy_body(&mut self, id: RigidBodyId) -> Result<RigidBodyHandle, PhysicsError> {
        self.remove_body(id)
    }

    /// 更新刚体
    pub fn update_body(&mut self, body: &RigidBody) -> Result<(), PhysicsError> {
        if let Some(handle) = self.body_handles.get(&body.id()) {
            if let Some(rb) = self.rigid_body_set.get_mut(*handle) {
                // 更新位置和旋转
                let translation =
                    Translation::new(body.position().x, body.position().y, body.position().z);
                let rotation = UnitQuaternion::from_quaternion(Quaternion::new(
                    body.rotation().w,
                    body.rotation().x,
                    body.rotation().y,
                    body.rotation().z,
                ));
                rb.set_position(Isometry::from_parts(translation, rotation), true);

                // 更新速度
                rb.set_linvel(
                    Vector3::new(
                        body.linear_velocity().x,
                        body.linear_velocity().y,
                        body.linear_velocity().z,
                    ),
                    true,
                );
                rb.set_angvel(
                    Vector3::new(
                        body.angular_velocity().x,
                        body.angular_velocity().y,
                        body.angular_velocity().z,
                    ),
                    true,
                );

                Ok(())
            } else {
                Err(PhysicsError::BodyNotFound(format!(
                    "Body {}",
                    body.id().as_u64()
                )))
            }
        } else {
            Err(PhysicsError::BodyNotFound(format!(
                "Body {}",
                body.id().as_u64()
            )))
        }
    }
}

// 在physics.rs文件中添加一个测试模块来验证Rapier3D类型是否实现了Sync和Send

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_send_sync() {
        // 测试PhysicsWorld是否实现了Send
        fn assert_send<T: Send>() {}
        assert_send::<PhysicsWorld>();

        // 测试PhysicsWorld是否实现了Sync
        fn assert_sync<T: Sync>() {}
        assert_sync::<PhysicsWorld>();
    }

    #[test]
    fn test_rapier_types_send_sync() {
        // 测试各种Rapier3D类型是否实现了Send和Sync

        // PhysicsPipeline
        fn assert_send_physics_pipeline<T: Send>() {}
        fn assert_sync_physics_pipeline<T: Sync>() {}
        assert_send_physics_pipeline::<PhysicsPipeline>();
        assert_sync_physics_pipeline::<PhysicsPipeline>();

        // IslandManager
        fn assert_send_island_manager<T: Send>() {}
        fn assert_sync_island_manager<T: Sync>() {}
        assert_send_island_manager::<IslandManager>();
        assert_sync_island_manager::<IslandManager>();

        // DefaultBroadPhase
        fn assert_send_broad_phase<T: Send>() {}
        fn assert_sync_broad_phase<T: Sync>() {}
        assert_send_broad_phase::<DefaultBroadPhase>();
        assert_sync_broad_phase::<DefaultBroadPhase>();

        // NarrowPhase
        fn assert_send_narrow_phase<T: Send>() {}
        fn assert_sync_narrow_phase<T: Sync>() {}
        assert_send_narrow_phase::<NarrowPhase>();
        assert_sync_narrow_phase::<NarrowPhase>();

        // 其他类型可以类似测试...
    }
}
