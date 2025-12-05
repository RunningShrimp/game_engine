use crate::ecs::{Sprite, Time};
use crate::impl_default;
use bevy_ecs::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct Flipbook {
    pub frames: Vec<u32>, // Texture indices
    pub speed: f32,       // Frames per second
    pub current_frame: usize,
    pub timer: f32,
    pub playing: bool,
    pub loop_anim: bool,
}

impl_default!(Flipbook {
    frames: vec![],
    speed: 10.0,
    current_frame: 0,
    timer: 0.0,
    playing: true,
    loop_anim: true,
});

pub fn animation_system(mut query: Query<(&mut Flipbook, &mut Sprite)>, time: Res<Time>) {
    for (mut anim, mut sprite) in query.iter_mut() {
        if !anim.playing || anim.frames.is_empty() {
            continue;
        }

        anim.timer += time.delta_seconds;
        let frame_duration = 1.0 / anim.speed;

        if anim.timer >= frame_duration {
            anim.timer -= frame_duration;
            anim.current_frame += 1;

            if anim.current_frame >= anim.frames.len() {
                if anim.loop_anim {
                    anim.current_frame = 0;
                } else {
                    anim.current_frame = anim.frames.len() - 1;
                    anim.playing = false;
                }
            }

            sprite.tex_index = anim.frames[anim.current_frame];
        }
    }
}
