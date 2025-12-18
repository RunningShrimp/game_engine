#![cfg(feature = "use_bevy_ecs")]
use bevy_ecs::prelude::*;

#[derive(Component, Clone, Copy)]
pub struct Transform { pub pos: [f32;2], pub scale: [f32;2], pub rot: f32 }

#[derive(Component, Clone, Copy)]
pub struct Sprite { pub color: [f32;4], pub layer: f32, pub tex_index: u32, pub uv_off: [f32;2], pub uv_scale: [f32;2] }

#[derive(Component, Clone, Copy)]
pub struct Camera { pub ortho: [f32;4] }

#[derive(Resource, Clone, Copy)]
pub struct Time { pub delta: f32 }

#[derive(Resource, Default)]
pub struct Input { pub mouse: [f32;2], pub mouse_down: bool }

pub struct EcsApp { pub world: World, pub schedule: Schedule }

impl EcsApp {
    pub fn new() -> Self {
        let mut world = World::new();
        world.insert_resource(Time { delta: 0.0 });
        world.insert_resource(Input::default());
        let mut schedule = Schedule::default();
        Self { world, schedule }
    }

    pub fn update(&mut self, dt: f32) { self.world.resource_mut::<Time>().delta = dt; self.schedule.run(&mut self.world); }
}
