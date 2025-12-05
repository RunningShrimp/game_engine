//! 动画服务层
//!
//! 遵循DDD贫血模型，将动画业务逻辑封装在Service中

use super::clip::AnimationClip;
use super::player::AnimationPlayer;
use crate::ecs::Transform;

/// 动画服务 - 封装动画业务逻辑
///
/// 遵循贫血模型设计原则：
/// - AnimationPlayer (Component): 纯数据结构
/// - AnimationService (Service): 封装业务逻辑
/// - animation_system (System): 调度编排
pub struct AnimationService;

impl AnimationService {
    /// 播放动画片段
    pub fn play(player: &mut AnimationPlayer, clip: AnimationClip) {
        player.current_clip = Some(clip);
        player.current_time = 0.0;
        player.playing = true;
    }

    /// 暂停播放
    pub fn pause(player: &mut AnimationPlayer) {
        player.playing = false;
    }

    /// 恢复播放
    pub fn resume(player: &mut AnimationPlayer) {
        player.playing = true;
    }

    /// 停止播放并重置
    pub fn stop(player: &mut AnimationPlayer) {
        player.playing = false;
        player.current_time = 0.0;
    }

    /// 设置播放速度
    pub fn set_speed(player: &mut AnimationPlayer, speed: f32) {
        player.speed = speed;
    }

    /// 跳转到指定时间
    pub fn seek(player: &mut AnimationPlayer, time: f32) {
        player.current_time = time.max(0.0);
        if let Some(clip) = &player.current_clip {
            if player.current_time > clip.duration {
                player.current_time = clip.duration;
            }
        }
    }

    /// 更新动画状态
    pub fn update(player: &mut AnimationPlayer, delta_time: f32) {
        if !player.playing {
            return;
        }

        if let Some(clip) = &player.current_clip {
            player.current_time += delta_time * player.speed;

            if player.current_time >= clip.duration {
                if clip.looping {
                    player.current_time %= clip.duration;
                } else {
                    player.current_time = clip.duration;
                    player.playing = false;
                }
            }
        }
    }

    /// 应用动画到Transform组件
    pub fn apply_to_transform(player: &AnimationPlayer, entity_id: u64, transform: &mut Transform) {
        if let Some(clip) = &player.current_clip {
            if let Some(position) = clip.sample_position(entity_id, player.current_time) {
                transform.pos = position;
            }

            if let Some(rotation) = clip.sample_rotation(entity_id, player.current_time) {
                transform.rot = rotation;
            }

            if let Some(scale) = clip.sample_scale(entity_id, player.current_time) {
                transform.scale = scale;
            }
        }
    }

    /// 获取当前播放进度 (0.0 - 1.0)
    pub fn progress(player: &AnimationPlayer) -> f32 {
        if let Some(clip) = &player.current_clip {
            if clip.duration > 0.0 {
                return player.current_time / clip.duration;
            }
        }
        0.0
    }

    /// 检查动画是否播放完成
    pub fn is_finished(player: &AnimationPlayer) -> bool {
        if let Some(clip) = &player.current_clip {
            if !clip.looping && player.current_time >= clip.duration {
                return true;
            }
        }
        false
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

        AnimationService::play(&mut player, clip);
        assert!(player.playing);
        assert_eq!(player.current_time, 0.0);

        AnimationService::pause(&mut player);
        assert!(!player.playing);

        AnimationService::resume(&mut player);
        assert!(player.playing);
    }

    #[test]
    fn test_update() {
        let mut player = AnimationPlayer::default();
        let clip = create_test_clip();

        AnimationService::play(&mut player, clip);
        AnimationService::update(&mut player, 0.5);

        assert_eq!(player.current_time, 0.5);
    }

    #[test]
    fn test_progress() {
        let mut player = AnimationPlayer::default();
        let clip = create_test_clip();

        AnimationService::play(&mut player, clip);
        player.current_time = 0.5;

        let progress = AnimationService::progress(&player);
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
