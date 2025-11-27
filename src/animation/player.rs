use super::clip::AnimationClip;
use bevy_ecs::prelude::*;

/// 动画播放器组件 (贫血模型 - 纯数据结构)
/// 
/// 遵循DDD贫血模型设计原则：
/// - AnimationPlayer (Component): 纯数据结构 ← 本文件
/// - AnimationService (Service): 业务逻辑封装 → service.rs
/// - animation_system (System): 系统调度编排
/// 
/// 业务逻辑已移至 `AnimationService`，请使用：
/// ```rust
/// use crate::animation::{AnimationPlayer, AnimationService};
/// 
/// let mut player = AnimationPlayer::default();
/// AnimationService::play(&mut player, clip);
/// AnimationService::update(&mut player, delta_time);
/// ```
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

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self {
            current_clip: None,
            current_time: 0.0,
            speed: 1.0,
            playing: false,
        }
    }
}

impl AnimationPlayer {
    pub fn new() -> Self {
        Self::default()
    }
    
    // ==========================================
    // 以下方法保留用于向后兼容，建议使用 AnimationService
    // ==========================================
    
    /// 播放动画片段
    /// 
    /// **建议使用**: `AnimationService::play()`
    #[deprecated(since = "0.2.0", note = "请使用 AnimationService::play() 代替")]
    pub fn play(&mut self, clip: AnimationClip) {
        super::service::AnimationService::play(self, clip);
    }
    
    /// 暂停播放
    /// 
    /// **建议使用**: `AnimationService::pause()`
    #[deprecated(since = "0.2.0", note = "请使用 AnimationService::pause() 代替")]
    pub fn pause(&mut self) {
        super::service::AnimationService::pause(self);
    }
    
    /// 恢复播放
    /// 
    /// **建议使用**: `AnimationService::resume()`
    #[deprecated(since = "0.2.0", note = "请使用 AnimationService::resume() 代替")]
    pub fn resume(&mut self) {
        super::service::AnimationService::resume(self);
    }
    
    /// 停止播放
    /// 
    /// **建议使用**: `AnimationService::stop()`
    #[deprecated(since = "0.2.0", note = "请使用 AnimationService::stop() 代替")]
    pub fn stop(&mut self) {
        super::service::AnimationService::stop(self);
    }
    
    /// 更新动画 (每帧调用)
    /// 
    /// **建议使用**: `AnimationService::update()`
    #[deprecated(since = "0.2.0", note = "请使用 AnimationService::update() 代替")]
    pub fn update(&mut self, delta_time: f32) {
        super::service::AnimationService::update(self, delta_time);
    }
}

use crate::ecs::Transform;
use bevy_ecs::prelude::*;

/// 动画系统 - 更新所有动画播放器
/// 
/// 使用 AnimationService 执行业务逻辑
pub fn animation_system(
    time: Res<crate::ecs::Time>,
    mut query: Query<(Entity, &mut AnimationPlayer, &mut Transform)>,
) {
    for (entity, mut player, mut transform) in query.iter_mut() {
        super::service::AnimationService::update(&mut player, time.delta_seconds);
        super::service::AnimationService::apply_to_transform(&player, entity.to_bits(), &mut transform);
    }
}
