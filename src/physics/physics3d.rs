use crate::ecs::Transform;
use crate::impl_default;
use bevy_ecs::prelude::*;
use glam::{Quat, Vec3};
use rapier3d::na::{Quaternion, UnitQuaternion};
use rapier3d::prelude::DefaultBroadPhase;
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
    pub broad_phase: Box<dyn BroadPhase>,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub query_pipeline: QueryPipeline,
}

impl_default!(PhysicsWorld3D {
    gravity: vector![0.0, -9.81, 0.0],
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
    query_pipeline: QueryPipeline::new(),
});

impl PhysicsWorld3D {
    pub fn new() -> Self {
        Self::default()
    }
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
            Some(&mut self.query_pipeline),
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
            point![origin.x, origin.y, origin.z],
            vector![direction.x, direction.y, direction.z],
        );

        if let Some((handle, toi)) = self.query_pipeline.cast_ray(
            &self.rigid_body_set,
            &self.collider_set,
            &ray,
            max_distance,
            true,
            QueryFilter::default(),
        ) {
            let hit_point = ray.point_at(toi);
            let entity = self.collider_set.get(handle)?.user_data;
            Some((
                Entity::from_bits(entity as u64),
                toi,
                Vec3::new(hit_point.x, hit_point.y, hit_point.z),
            ))
        } else {
            None
        }
    }

    /// 形状投射
    pub fn shapecast(
        &self,
        shape: &dyn Shape,
        position: Vec3,
        rotation: Quat,
        direction: Vec3,
        max_distance: f32,
    ) -> Option<(Entity, f32)> {
        let isometry = Isometry::from_parts(
            Translation::new(position.x, position.y, position.z),
            UnitQuaternion::new_normalize(Quaternion::new(
                rotation.w, rotation.x, rotation.y, rotation.z,
            )),
        );

        let dir = vector![direction.x, direction.y, direction.z];

        let options = rapier3d::parry::query::ShapeCastOptions {
            max_time_of_impact: max_distance,
            stop_at_penetration: true,
            ..Default::default()
        };
        if let Some((handle, hit)) = self.query_pipeline.cast_shape(
            &self.rigid_body_set,
            &self.collider_set,
            &isometry,
            &dir,
            shape,
            options,
            QueryFilter::default(),
        ) {
            let entity = self.collider_set.get(handle)?.user_data;
            Some((Entity::from_bits(entity as u64), hit.time_of_impact))
        } else {
            None
        }
    }

    /// 查询与AABB相交的碰撞体
    pub fn query_aabb(&self, min: Vec3, max: Vec3) -> Vec<Entity> {
        let aabb = Aabb::new(point![min.x, min.y, min.z], point![max.x, max.y, max.z]);

        let mut entities = Vec::new();
        self.query_pipeline
            .colliders_with_aabb_intersecting_aabb(&aabb, |handle| {
                if let Some(collider) = self.collider_set.get(*handle) {
                    entities.push(Entity::from_bits(collider.user_data as u64));
                }
                true
            });

        entities
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

impl_default!(RigidBodyDesc3D {
    body_type: RigidBodyType::Dynamic,
    position: Vec3::ZERO,
    rotation: Quat::IDENTITY,
    linear_velocity: Vec3::ZERO,
    angular_velocity: Vec3::ZERO,
});

#[derive(Component, Clone)]
pub struct ColliderDesc3D {
    pub shape: Shape3D,
    pub density: f32,
    pub friction: f32,
    pub restitution: f32,
}

impl_default!(ColliderDesc3D {
    shape: Shape3D::Cuboid(Vec3::ONE),
    density: 1.0,
    friction: 0.5,
    restitution: 0.0,
});

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

            // 分离借用
            let PhysicsWorld3D {
                rigid_body_set,
                collider_set,
                ..
            } = &mut *physics;
            let col_handle = collider_set.insert_with_parent(collider, rb_handle, rigid_body_set);
            commands
                .entity(entity)
                .insert(Collider3D { handle: col_handle });
        }

        commands
            .entity(entity)
            .insert(RigidBody3D { handle: rb_handle });
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
        let rb = RigidBodyBuilder::dynamic()
            .translation(vector![0.0, 10.0, 0.0])
            .build();
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
        let rb = RigidBodyBuilder::fixed()
            .translation(vector![0.0, 0.0, 0.0])
            .build();
        let rb_handle = world.rigid_body_set.insert(rb);

        let collider = ColliderBuilder::cuboid(10.0, 0.1, 10.0).build();
        world
            .collider_set
            .insert_with_parent(collider, rb_handle, &mut world.rigid_body_set);

        // 更新查询管线
        world.query_pipeline.update(&world.collider_set);

        // 从上方向下投射射线
        let result = world.raycast(Vec3::new(0.0, 10.0, 0.0), Vec3::new(0.0, -1.0, 0.0), 20.0);

        assert!(result.is_some());
    }
}
