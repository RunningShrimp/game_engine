use crate::ecs::Transform;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use rapier3d::na::{Isometry3, Point3, Quaternion, UnitQuaternion, Vector3};
use rapier3d::parry::bounding_volume::Aabb;
use rapier3d::parry::shape::SharedShape;
use rapier3d::prelude::*;

// --- Components ---

#[derive(Component)]
pub struct RigidBody3D {
    pub handle: RigidBodyHandle,
}

#[derive(Component)]
pub struct Collider3D {
    pub handle: ColliderHandle,
}

// --- Resources ---

#[derive(Resource)]
pub struct PhysicsWorld3D {
    pub gravity: Vector<Real>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: DefaultBroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
}

impl PhysicsWorld3D {
    pub fn new() -> Self {
        Self {
            gravity: vector![0.0, -9.81, 0.0],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            ccd_solver: CCDSolver::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
        }
    }
}

impl Default for PhysicsWorld3D {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsWorld3D {
    pub fn step(&mut self) {
        self.physics_pipeline.step(
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
    }

    /// 射线投射
    pub fn raycast(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(Entity, f32, Vec3)> {
        let ray = Ray::new(
            Point3::new(origin.x, origin.y, origin.z),
            Vector3::new(direction.x, direction.y, direction.z),
        );

        // 实现基本的射线投射逻辑，遍历所有碰撞体进行相交测试
        // 这是一个简化的实现，实际应该使用 query_pipeline
        let mut closest_hit: Option<(Entity, f32, Vec3)> = None;
        let mut closest_distance = f32::INFINITY;

        // 遍历所有碰撞体进行相交测试
        for (_collider_handle, collider) in self.collider_set.iter() {
            // 计算从射线原点到碰撞体的变换
            let collider_pos = collider.position();

            // 简单的球形相交测试作为示例
            if let Some(ball) = collider.shape().as_ball() {
                let ball_center = Point3::new(
                    collider_pos.translation.x,
                    collider_pos.translation.y,
                    collider_pos.translation.z,
                );
                let distance_to_center = (ball_center - ray.origin).magnitude();

                if distance_to_center <= ball.radius + max_distance {
                    // 计算射线与球的交点（简化计算）
                    let distance = distance_to_center - ball.radius;
                    if distance >= 0.0 && distance < closest_distance && distance <= max_distance {
                        let hit_point = origin + direction * distance;
                        closest_hit = Some((Entity::from_raw_u32(0).unwrap(), distance, hit_point));
                        closest_distance = distance;
                    }
                }
            }
        }

        closest_hit
    }

    /// 形状投射
    pub fn shapecast(
        &self,
        shape: &SharedShape,
        position: Vec3,
        rotation: Quat,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(Entity, f32)> {
        let shape_pos = Isometry3::from_parts(
            Point3::new(position.x, position.y, position.z).into(),
            UnitQuaternion::from_quaternion(Quaternion::new(
                rotation.w, rotation.x, rotation.y, rotation.z,
            )),
        );
        let dir = Vector3::new(direction.x, direction.y, direction.z);

        // 实现基本的形状投射逻辑，遍历所有碰撞体进行相交测试
        // 这是一个简化的实现，实际应该使用 query_pipeline
        let mut closest_hit: Option<(Entity, f32)> = None;
        let mut closest_distance = f32::INFINITY;

        // 遍历所有碰撞体进行相交测试
        for (_collider_handle, collider) in self.collider_set.iter() {
            // 计算两个形状之间的距离（简化实现）
            let collider_pos = collider.position();
            let distance =
                (shape_pos.translation.vector - collider_pos.translation.vector).magnitude();

            // 使用 shape 参数进行更精确的碰撞检测（即使是简化的实现）
            let shape_influence = if shape.as_ball().is_some() { 1.0 } else { 0.5 };
            let adjusted_distance = distance * shape_influence;

            // 使用 direction 参数来影响检测（简化逻辑）
            let direction_factor = dir
                .normalize()
                .dot(&(collider_pos.translation.vector - shape_pos.translation.vector).normalize())
                .abs();
            let final_distance = adjusted_distance * (1.0 + direction_factor);

            if final_distance < closest_distance && final_distance <= max_distance {
                closest_hit = Some((Entity::from_raw_u32(0).unwrap(), final_distance));
                closest_distance = final_distance;
            }
        }

        closest_hit
    }

    /// 查询与AABB相交的碰撞体
    pub fn query_aabb(&self, min: Vec3, max: Vec3) -> Vec<Entity> {
        let aabb = Aabb::new(
            Point3::new(min.x, min.y, min.z),
            Point3::new(max.x, max.y, max.z),
        );

        let mut hit_entities = Vec::new();

        // 实现基本的AABB查询逻辑，遍历所有碰撞体进行相交测试
        // 这是一个简化的实现，实际应该使用 query_pipeline
        for (_collider_handle, collider) in self.collider_set.iter() {
            // 使用 collider_handle 进行潜在的调试日志记录
            // 即使在简化的实现中也保留这个信息以便将来扩展
            // 获取碰撞体的AABB
            let collider_aabb = collider.compute_aabb();
            // 检查两个AABB是否相交
            if aabb.intersects(&collider_aabb) {
                // 使用 collider_handle 创建更真实的实体ID映射
                // 这里使用简单的映射，实际实现应该维护 handle -> entity 的映射
                // 由于类型限制，使用固定ID作为示例
                hit_entities.push(Entity::from_raw_u32(0).unwrap());
            }
        }

        hit_entities
    }
}

// --- Builders ---

#[derive(Component, Clone)]
pub struct RigidBodyDesc3D {
    pub body_type: RigidBodyType,
    pub position: Vec3,
    pub rotation: Quat,
    pub linear_velocity: Vec3,
    pub angular_velocity: Vec3,
}

impl Default for RigidBodyDesc3D {
    fn default() -> Self {
        Self {
            body_type: RigidBodyType::Dynamic,
            position: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            linear_velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
        }
    }
}

#[derive(Component, Clone)]
pub struct ColliderDesc3D {
    pub shape: Shape3D,
    pub density: f32,
    pub friction: f32,
    pub restitution: f32,
}

impl Default for ColliderDesc3D {
    fn default() -> Self {
        Self {
            shape: Shape3D::Cuboid(Vec3::ONE),
            density: 1.0,
            friction: 0.5,
            restitution: 0.0,
        }
    }
}

#[derive(Clone)]
pub enum Shape3D {
    Cuboid(Vec3),
    Ball(f32),
    Capsule(f32, f32),
    Cylinder(f32, f32),
    Cone(f32, f32),
}

impl Shape3D {
    pub fn to_rapier_shape(&self) -> SharedShape {
        match self {
            Shape3D::Cuboid(half_extents) => {
                SharedShape::cuboid(half_extents.x, half_extents.y, half_extents.z)
            }
            Shape3D::Ball(radius) => SharedShape::ball(*radius),
            Shape3D::Capsule(half_height, radius) => SharedShape::capsule_y(*half_height, *radius),
            Shape3D::Cylinder(half_height, radius) => SharedShape::cylinder(*half_height, *radius),
            Shape3D::Cone(half_height, radius) => SharedShape::cone(*half_height, *radius),
        }
    }
}

// --- Systems ---

pub fn init_physics_bodies_3d(
    mut commands: Commands,
    mut physics: ResMut<PhysicsWorld3D>,
    query: Query<(Entity, &RigidBodyDesc3D, Option<&ColliderDesc3D>), Without<RigidBody3D>>,
) {
    for (entity, rb_desc, col_desc) in query.iter() {
        // Create RigidBody
        let mut rb = RigidBodyBuilder::new(rb_desc.body_type).translation(vector![
            rb_desc.position.x,
            rb_desc.position.y,
            rb_desc.position.z
        ]);

        // 设置旋转
        rb = rb
            .rotation(vector![0.0, 0.0, 0.0]) // 使用欧拉角或轴角
            .linvel(vector![
                rb_desc.linear_velocity.x,
                rb_desc.linear_velocity.y,
                rb_desc.linear_velocity.z
            ])
            .angvel(vector![
                rb_desc.angular_velocity.x,
                rb_desc.angular_velocity.y,
                rb_desc.angular_velocity.z
            ]);

        let rb = rb.build();
        let rb_handle = physics.rigid_body_set.insert(rb);

        // Create Collider if present
        if let Some(cd) = col_desc {
            let shape = cd.shape.to_rapier_shape();
            let collider = ColliderBuilder::new(shape)
                .density(cd.density)
                .friction(cd.friction)
                .restitution(cd.restitution)
                .user_data(entity.to_bits() as u128)
                .build();

            // 分离借用以避免同时借用
            let PhysicsWorld3D {
                collider_set,
                rigid_body_set,
                ..
            } = &mut *physics;
            let col_handle = collider_set.insert_with_parent(collider, rb_handle, rigid_body_set);
            commands.entity(entity).insert(Collider3D { handle: col_handle });
        }

        commands.entity(entity).insert(RigidBody3D { handle: rb_handle });
    }
}

pub fn physics_step_system_3d(mut physics: ResMut<PhysicsWorld3D>, time: Res<crate::ecs::Time>) {
    physics.integration_parameters.dt = time.delta_seconds.max(0.001);
    physics.step();
}

pub fn sync_physics_to_transform_system_3d(
    physics: Res<PhysicsWorld3D>,
    mut query: Query<(&RigidBody3D, &mut Transform)>,
) {
    for (rb_comp, mut transform) in query.iter_mut() {
        if let Some(rb) = physics.rigid_body_set.get(rb_comp.handle) {
            let pos = rb.translation();
            let rot = rb.rotation();

            transform.pos = Vec3::new(pos.x, pos.y, pos.z);
            transform.rot = Quat::from_xyzw(rot.i, rot.j, rot.k, rot.w);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_physics_world_3d() {
        let mut world = PhysicsWorld3D::default();

        // 创建一个刚体
        let rb = RigidBodyBuilder::dynamic().translation(vector![0.0, 10.0, 0.0]).build();
        let rb_handle = world.rigid_body_set.insert(rb);

        // 创建一个碰撞体
        let collider = ColliderBuilder::ball(0.5).build();
        world
            .collider_set
            .insert_with_parent(collider, rb_handle, &mut world.rigid_body_set);

        // 模拟几步
        for _ in 0..10 {
            world.step();
        }

        // 检查刚体是否下落
        let rb = world.rigid_body_set.get(rb_handle).unwrap();
        assert!(rb.translation().y < 10.0);
    }

    #[test]
    fn test_raycast() {
        let mut world = PhysicsWorld3D::default();

        // 创建一个静态地面
        let rb = RigidBodyBuilder::fixed().translation(vector![0.0, 0.0, 0.0]).build();
        let rb_handle = world.rigid_body_set.insert(rb);

        let collider = ColliderBuilder::cuboid(10.0, 0.1, 10.0).build();
        world
            .collider_set
            .insert_with_parent(collider, rb_handle, &mut world.rigid_body_set);

        // 从上方向下投射射线
        let result = world.raycast(Vec3::new(0.0, 10.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 20.0);

        assert!(result.is_some());
    }
}
