use bevy_ecs::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct AudioSource {
    pub name: String,
    pub path: String,
    pub volume: f32,
    pub loop_sound: bool,
    pub is_playing: bool,
}

impl Default for AudioSource {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            path: "".to_string(),
            volume: 1.0,
            loop_sound: false,
            is_playing: false,
        }
    }
}

#[derive(Resource, Default)]
pub struct AudioState {
    pub master_volume: f32,
}
