//! Physics Plugin
//!
//! A physics simulation plugin that demonstrates custom components and systems.

use game_engine::plugins::api::*;
use game_engine::ecs::{Component, ComponentRegistry, System, SystemContext, SystemScheduler};
use game_engine::error::Error;
use std::any::Any;

/// Velocity component
#[derive(Component, Debug, Clone)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Acceleration component
#[derive(Component, Debug, Clone)]
pub struct Acceleration {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Acceleration {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

/// Mass component
#[derive(Component, Debug, Clone)]
pub struct Mass {
    pub value: f32,
}

impl Mass {
    pub fn new(value: f32) -> Self {
        Self { value }
    }
}

/// Physics material component
#[derive(Component, Debug, Clone)]
pub struct PhysicsMaterial {
    pub restitution: f32,  // Bounciness (0-1)
    pub friction: f32,      // Friction coefficient
}

impl PhysicsMaterial {
    pub fn new(restitution: f32, friction: f32) -> Self {
        Self { restitution, friction }
    }
}

/// Physics system
pub struct PhysicsSystem {
    gravity: f32,
    time_step: f32,
}

impl PhysicsSystem {
    pub fn new(gravity: f32, time_step: f32) -> Self {
        Self { gravity, time_step }
    }
}

impl System for PhysicsSystem {
    fn name(&self) -> &str {
        "PhysicsSystem"
    }

    fn run(&mut self, context: &SystemContext, delta: f32) {
        let world = context.world();
        let entities = world.entities();

        let mut total_processed = 0;

        for entity in entities {
            // Get velocity component
            if let Some(velocity) = world.get_component::<Velocity>(entity) {
                // Get acceleration component (optional)
                let acceleration = world.get_component::<Acceleration>(entity);

                // Apply physics
                let mut vel = velocity.clone();

                if let Some(acc) = acceleration {
                    vel.x += acc.x * delta;
                    vel.y += acc.y * delta;
                    vel.z += acc.z * delta;
                }

                // Apply gravity
                vel.y -= self.gravity * delta;

                // Update velocity (in a real system, this would be written back)
                let _ = vel;
                total_processed += 1;
            }
        }

        if total_processed > 0 {
            println!("[PhysicsSystem] Updated {} entities", total_processed);
        }
    }
}

/// Physics Plugin
pub struct PhysicsPlugin {
    metadata: PluginMetadata,
    gravity: f32,
    time_step: f32,
}

impl PhysicsPlugin {
    /// Create a new Physics plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata::new("physics".to_string(), "0.1.0".to_string())
            .with_author("Game Engine Team".to_string())
            .with_description("Physics simulation plugin".to_string())
            .with_engine_version(">=0.1.0".to_string());

        Self {
            metadata,
            gravity: -9.81,
            time_step: 1.0 / 60.0,
        }
    }

    /// Set gravity
    pub fn with_gravity(mut self, gravity: f32) -> Self {
        self.gravity = gravity;
        self
    }

    /// Set time step
    pub fn with_time_step(mut self, time_step: f32) -> Self {
        self.time_step = time_step;
        self
    }
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for PhysicsPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, context: &PluginContext) -> Result<(), Error> {
        println!("[PhysicsPlugin] Loaded successfully!");
        println!("[PhysicsPlugin] Gravity: {} m/s²", self.gravity);
        println!("[PhysicsPlugin] Time step: {} s", self.time_step);
        println!("[PhysicsPlugin] Engine version: {}", context.engine_version());

        Ok(())
    }

    fn on_unload(&mut self, _context: &PluginContext) -> Result<(), Error> {
        println!("[PhysicsPlugin] Unloaded successfully!");
        Ok(())
    }

    fn on_update(&mut self, _context: &PluginContext, delta: f32) {
        // Physics updates are handled by the physics system
    }

    fn register_components(&self, registry: &mut ComponentRegistry) {
        println!("[PhysicsPlugin] Registering physics components...");

        // Register custom components
        registry.register::<Velocity>("Velocity");
        registry.register::<Acceleration>("Acceleration");
        registry.register::<Mass>("Mass");
        registry.register::<PhysicsMaterial>("PhysicsMaterial");

        println!("[PhysicsPlugin] Registered 4 component types");
    }

    fn register_systems(&self, scheduler: &mut SystemScheduler) {
        println!("[PhysicsPlugin] Registering physics systems...");

        let physics_system = PhysicsSystem::new(self.gravity, self.time_step);
        scheduler.register_system(Box::new(physics_system));

        println!("[PhysicsPlugin] Registered physics system");
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Plugin creation function (exported for dynamic loading)
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(PhysicsPlugin::new())
}
