#![cfg(feature = "physics_2d")]

pub mod physics3d;
pub mod joints;
use bevy_ecs::prelude::*;
use rapier2d::prelude::*;
use crate::ecs::Transform;


// --- Components ---

#[derive(Component)]
pub struct RigidBodyComp {
    pub handle: RigidBodyHandle,
}

#[derive(Component)]
pub struct ColliderComp {
    pub handle: ColliderHandle,
}

// --- Resources ---

#[derive(Resource)]
pub struct PhysicsWorld {
    pub gravity: Vector<Real>,
    pub integration_parameters: IntegrationParameters,
    pub physics_pipeline: PhysicsPipeline,
    pub island_manager: IslandManager,
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub impulse_joint_set: ImpulseJointSet,
    pub multibody_joint_set: MultibodyJointSet,
    pub ccd_solver: CCDSolver,
    pub rigid_body_set: RigidBodySet,
    pub collider_set: ColliderSet,
    pub physics_hooks: (),
    pub event_handler: (),
}

impl Default for PhysicsWorld {
    fn default() -> Self {
        Self {
            gravity: vector![0.0, -9.81],
            integration_parameters: IntegrationParameters::default(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: BroadPhase::new(),
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
            None,
            &self.physics_hooks,
            &self.event_handler,
        );
    }
}

// --- Builders ---

#[derive(Component, Clone)]
pub struct RigidBodyDesc {
    pub body_type: RigidBodyType,
    pub position: [f32; 2],
}

#[derive(Component, Clone)]
pub struct ColliderDesc {
    pub shape_type: ShapeType, // Simplified enum for now
    pub half_extents: [f32; 2],
    pub radius: f32,
}

#[derive(Clone, Copy)]
pub enum ShapeType {
    Cuboid,
    Ball,
}

// --- Systems ---

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

pub fn physics_step_system(mut physics: ResMut<PhysicsWorld>, time: Res<crate::ecs::Time>) {
    // Rapier usually expects a fixed timestep, but for simplicity we use delta here or assume fixed update
    // In a real engine, we should accumulate time and step fixed amounts.
    physics.integration_parameters.dt = time.delta_seconds.max(0.001); 
    physics.step();
}

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
            // 2D rotation around Z axis
            transform.rot = glam::Quat::from_rotation_z(rot.angle());
        }
    }
}
