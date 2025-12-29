//  动画服务层
//
//  遵循DDD贫血模型，将动画业务逻辑封装在Service中

use super::clip::AnimationClip;
use super::player::AnimationPlayer;
use crate::ecs::Transform;

/// 动画服务 - 协调层（用于复杂业务场景）
///
/// 遵循富领域模型设计原则：
/// - AnimationPlayer (Component): 包含业务逻辑的领域对象
/// - AnimationService (Service): 协调层，用于复杂业务场景（如动画混合、队列管理等）
/// - animation_system (System): 调度编排
///
/// 注意：大部分业务逻辑已移至 `AnimationPlayer`，此服务主要用于：
/// - 动画混合（blending）
/// - 动画队列管理
/// - 跨实体的动画协调
pub struct AnimationService;

impl AnimationService {
    /// 播放动画片段（委托给AnimationPlayer）
    ///
    /// 此方法保留用于向后兼容，建议直接使用 `player.play(clip)`
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::play()")]
    pub fn play(player: &mut AnimationPlayer, clip: AnimationClip) {
        player.play(clip);
    }

    /// 暂停播放（委托给AnimationPlayer）
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::pause()")]
    pub fn pause(player: &mut AnimationPlayer) {
        player.pause();
    }

    /// 恢复播放（委托给AnimationPlayer）
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::resume()")]
    pub fn resume(player: &mut AnimationPlayer) {
        player.resume();
    }

    /// 停止播放（委托给AnimationPlayer）
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::stop()")]
    pub fn stop(player: &mut AnimationPlayer) {
        player.stop();
    }

    /// 设置播放速度（委托给AnimationPlayer）
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::set_speed()")]
    pub fn set_speed(player: &mut AnimationPlayer, speed: f32) {
        player.set_speed(speed);
    }

    /// 跳转到指定时间（委托给AnimationPlayer）
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::seek()")]
    pub fn seek(player: &mut AnimationPlayer, time: f32) {
        player.seek(time);
    }

    /// 更新动画状态（委托给AnimationPlayer）
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::update()")]
    pub fn update(player: &mut AnimationPlayer, delta_time: f32) {
        player.update(delta_time);
    }

    /// 应用动画到Transform组件（委托给AnimationPlayer）
    #[deprecated(
        since = "0.3.0",
        note = "请直接使用 AnimationPlayer::apply_to_transform()"
    )]
    pub fn apply_to_transform(player: &AnimationPlayer, entity_id: u64, transform: &mut Transform) {
        player.apply_to_transform(entity_id, transform);
    }

    /// 获取当前播放进度（委托给AnimationPlayer）
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::progress()")]
    pub fn progress(player: &AnimationPlayer) -> f32 {
        player.progress()
    }

    /// 检查动画是否播放完成（委托给AnimationPlayer）
    #[deprecated(since = "0.3.0", note = "请直接使用 AnimationPlayer::is_finished()")]
    pub fn is_finished(player: &AnimationPlayer) -> bool {
        player.is_finished()
    }

    /// 混合两个动画 (线性插值)
    pub fn blend_transforms(
        transform_a: &Transform,
        transform_b: &Transform,
        blend_factor: f32,
    ) -> Transform {
        let factor = blend_factor.clamp(0.0, 1.0);
        Transform {
            pos: transform_a.pos.lerp(transform_b.pos, factor),
            rot: transform_a.rot.slerp(transform_b.rot, factor),
            scale: transform_a.scale.lerp(transform_b.scale, factor),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Quat, Vec3};

    fn create_test_clip() -> AnimationClip {
        AnimationClip {
            name: "test".to_string(),
            duration: 1.0,
            looping: false,
            position_tracks: std::collections::HashMap::new(),
            rotation_tracks: std::collections::HashMap::new(),
            scale_tracks: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_play_pause_resume() {
        let mut player = AnimationPlayer::default();
        let clip = create_test_clip();

        player.play(clip);
        assert!(player.playing);
        assert_eq!(player.current_time, 0.0);

        player.pause();
        assert!(!player.playing);

        player.resume();
        assert!(player.playing);
    }

    #[test]
    fn test_update() {
        let mut player = AnimationPlayer::default();
        let clip = create_test_clip();

        player.play(clip);
        player.update(0.5);

        assert_eq!(player.current_time, 0.5);
    }

    #[test]
    fn test_progress() {
        let mut player = AnimationPlayer::default();
        let clip = create_test_clip();

        player.play(clip);
        player.current_time = 0.5;

        let progress = player.progress();
        assert!((progress - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_blend_transforms() {
        let t1 = Transform {
            pos: Vec3::ZERO,
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let t2 = Transform {
            pos: Vec3::new(10.0, 0.0, 0.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE * 2.0,
        };

        let blended = AnimationService::blend_transforms(&t1, &t2, 0.5);

        assert!((blended.pos.x - 5.0).abs() < 0.001);
        assert!((blended.scale.x - 1.5).abs() < 0.001);
    }
}
