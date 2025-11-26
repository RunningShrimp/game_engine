use super::clip::AnimationClip;
use bevy_ecs::prelude::*;
use crate::ecs::Transform;

/// 动画播放器
#[derive(Component)]
pub struct AnimationPlayer {
    /// 当前播放的动画片段
    pub current_clip: Option<AnimationClip>,
    /// 当前播放时间 (秒)
    pub current_time: f32,
    /// 播放速度 (1.0 = 正常速度)
    pub speed: f32,
    /// 是否正在播放
    pub playing: bool,
}

impl AnimationPlayer {
    pub fn new() -> Self {
        Self {
            current_clip: None,
            current_time: 0.0,
            speed: 1.0,
            playing: false,
        }
    }
    
    /// 播放动画片段
    pub fn play(&mut self, clip: AnimationClip) {
        self.current_clip = Some(clip);
        self.current_time = 0.0;
        self.playing = true;
    }
    
    /// 暂停播放
    pub fn pause(&mut self) {
        self.playing = false;
    }
    
    /// 恢复播放
    pub fn resume(&mut self) {
        self.playing = true;
    }
    
    /// 停止播放
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
    }
    
    /// 更新动画 (每帧调用)
    pub fn update(&mut self, delta_time: f32) {
        if !self.playing {
            return;
        }
        
        if let Some(clip) = &self.current_clip {
            self.current_time += delta_time * self.speed;
            
            if self.current_time >= clip.duration {
                if clip.looping {
                    self.current_time %= clip.duration;
                } else {
                    self.current_time = clip.duration;
                    self.playing = false;
                }
            }
        }
    }
    
    /// 应用动画到Transform组件
    pub fn apply_to_transform(&self, entity_id: u64, transform: &mut Transform) {
        if let Some(clip) = &self.current_clip {
            if let Some(position) = clip.sample_position(entity_id, self.current_time) {
                transform.pos = position;
            }
            
            if let Some(rotation) = clip.sample_rotation(entity_id, self.current_time) {
                transform.rot = rotation;
            }
            
            if let Some(scale) = clip.sample_scale(entity_id, self.current_time) {
                transform.scale = scale;
            }
        }
    }
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new()
    }
}

/// 动画系统 - 更新所有动画播放器
pub fn animation_system(
    time: Res<crate::ecs::Time>,
    mut query: Query<(Entity, &mut AnimationPlayer, &mut Transform)>,
) {
    for (entity, mut player, mut transform) in query.iter_mut() {
        player.update(time.delta_seconds);
        player.apply_to_transform(entity.to_bits(), &mut transform);
    }
}
