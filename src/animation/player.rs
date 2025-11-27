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

// ============================================================================
// 骨骼动画系统
// ============================================================================

/// 骨骼动画播放器组件
#[derive(Component)]
pub struct SkeletonAnimationPlayer {
    /// 当前播放的动画片段
    pub current_clip: Option<super::clip::AnimationClip>,
    /// 当前播放时间 (秒)
    pub current_time: f32,
    /// 播放速度 (1.0 = 正常速度)
    pub speed: f32,
    /// 是否正在播放
    pub playing: bool,
    /// 是否循环播放
    pub looping: bool,
}

impl Default for SkeletonAnimationPlayer {
    fn default() -> Self {
        Self {
            current_clip: None,
            current_time: 0.0,
            speed: 1.0,
            playing: false,
            looping: true,
        }
    }
}

impl SkeletonAnimationPlayer {
    pub fn new() -> Self {
        Self::default()
    }

    /// 播放动画
    pub fn play(&mut self, clip: super::clip::AnimationClip) {
        self.current_clip = Some(clip);
        self.current_time = 0.0;
        self.playing = true;
    }

    /// 暂停
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// 恢复
    pub fn resume(&mut self) {
        self.playing = true;
    }

    /// 停止
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
    }

    /// 设置循环
    pub fn set_looping(&mut self, looping: bool) {
        self.looping = looping;
    }
}

/// 骨骼姿态更新系统
/// 
/// 更新所有骨骼动画，采样当前时间的骨骼变换，
/// 计算世界空间矩阵并更新 GPU 缓冲区。
pub fn skeleton_update_system(
    time: Res<crate::ecs::Time>,
    mut query: Query<(&mut super::skeleton::Skeleton, &mut SkeletonAnimationPlayer)>,
) {
    let delta = time.delta_seconds;

    for (mut skeleton, mut player) in query.iter_mut() {
        if !player.playing {
            continue;
        }

        // 更新播放时间
        player.current_time += delta * player.speed;

        // 处理动画片段采样
        let (should_stop, sample_time) = if let Some(ref clip) = player.current_clip {
            // 处理循环/结束
            if player.current_time >= clip.duration {
                if player.looping {
                    (false, player.current_time % clip.duration)
                } else {
                    (true, clip.duration)
                }
            } else {
                (false, player.current_time)
            }
        } else {
            continue;
        };

        if should_stop {
            player.current_time = sample_time;
            player.playing = false;
        } else {
            player.current_time = sample_time;
        }

        // 采样骨骼变换 - 使用分离的函数避免借用冲突
        if let Some(ref clip) = player.current_clip {
            sample_skeleton_pose_from_clip(&mut skeleton, clip, player.current_time);
        }

        // 更新骨骼矩阵
        skeleton.update_pose();
    }
}

/// 从动画片段采样骨骼姿态
/// 
/// 使用骨骼索引作为 entity_id 来查找轨道
fn sample_skeleton_pose_from_clip(
    skeleton: &mut super::skeleton::Skeleton,
    clip: &super::clip::AnimationClip,
    time: f32,
) {
    // 遍历每个骨骼，查找对应的动画轨道
    for bone_index in 0..skeleton.bone_count() {
        let bone_id = bone_index as u64;
        
        // 采样位置
        if let Some(position) = clip.sample_position(bone_id, time) {
            if let Some(bone) = skeleton.bones.get_mut(bone_index) {
                bone.local_transform.translation = position;
            }
        }
        
        // 采样旋转
        if let Some(rotation) = clip.sample_rotation(bone_id, time) {
            if let Some(bone) = skeleton.bones.get_mut(bone_index) {
                bone.local_transform.rotation = rotation;
            }
        }
        
        // 采样缩放
        if let Some(scale) = clip.sample_scale(bone_id, time) {
            if let Some(bone) = skeleton.bones.get_mut(bone_index) {
                bone.local_transform.scale = scale;
            }
        }
    }

    skeleton.dirty = true;
}

// GPU 上传需要在渲染系统中手动调用，因为 wgpu::Device 不是 ECS Resource
// 使用方式:
// ```
// for mut skeleton in skeleton_query.iter_mut() {
//     if skeleton.dirty {
//         skeleton.update_gpu_buffer(&device, &queue);
//     }
// }
// ```
