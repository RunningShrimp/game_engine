use super::clip::AnimationClip;

/// 动画播放器组件 (富领域模型)
///
/// 遵循DDD富领域模型设计原则：
/// - AnimationPlayer (Component): 包含业务逻辑的领域对象
/// - AnimationService (Service): 协调层（可选，用于复杂业务场景）
/// - animation_system (System): 系统调度编排
///
/// 业务逻辑封装在 `AnimationPlayer` 中，可以直接使用：
/// ```rust
/// use crate::animation::AnimationPlayer;
///
/// let mut player = AnimationPlayer::default();
/// player.play(clip);
/// player.update(delta_time);
/// ```
#[derive(Component, Default, Clone)]
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
    /// 创建新的动画播放器
    pub fn new() -> Self {
        Self::default()
    }

    /// 播放动画片段
    ///
    /// # 参数
    /// - `clip`: 要播放的动画片段
    ///
    /// # 示例
    /// ```rust
    /// let mut player = AnimationPlayer::new();
    /// player.play(clip);
    /// ```
    pub fn play(&mut self, clip: AnimationClip) {
        self.current_clip = Some(clip);
        self.current_time = 0.0;
        self.playing = true;
    }

    /// 暂停播放
    ///
    /// 暂停当前播放的动画，保持当前时间位置。
    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// 恢复播放
    ///
    /// 从暂停位置恢复播放。
    pub fn resume(&mut self) {
        self.playing = true;
    }

    /// 停止播放并重置
    ///
    /// 停止播放并将时间重置为0。
    pub fn stop(&mut self) {
        self.playing = false;
        self.current_time = 0.0;
    }

    /// 更新动画状态 (每帧调用)
    ///
    /// # 参数
    /// - `delta_time`: 帧时间（秒）
    ///
    /// # 示例
    /// ```rust
    /// player.update(0.016); // 更新一帧（60 FPS）
    /// ```
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

    /// 设置播放速度
    ///
    /// # 参数
    /// - `speed`: 播放速度（1.0 = 正常速度，2.0 = 2倍速，0.5 = 半速）
    pub fn set_speed(&mut self, speed: f32) {
        self.speed = speed;
    }

    /// 跳转到指定时间
    ///
    /// # 参数
    /// - `time`: 目标时间（秒）
    pub fn seek(&mut self, time: f32) {
        self.current_time = time.max(0.0);
        if let Some(clip) = &self.current_clip {
            if self.current_time > clip.duration {
                self.current_time = clip.duration;
            }
        }
    }

    /// 获取当前播放进度 (0.0 - 1.0)
    ///
    /// # 返回
    /// 播放进度，范围 [0.0, 1.0]
    pub fn progress(&self) -> f32 {
        if let Some(clip) = &self.current_clip {
            if clip.duration > 0.0 {
                return self.current_time / clip.duration;
            }
        }
        0.0
    }

    /// 检查动画是否播放完成
    ///
    /// # 返回
    /// 如果动画已播放完成（非循环动画）返回 `true`
    pub fn is_finished(&self) -> bool {
        if let Some(clip) = &self.current_clip {
            if !clip.looping && self.current_time >= clip.duration {
                return true;
            }
        }
        false
    }

    /// 应用动画到Transform组件
    ///
    /// # 参数
    /// - `entity_id`: 实体ID
    /// - `transform`: 要更新的变换组件
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

use crate::ecs::Transform;
use bevy_ecs::prelude::*;

/// 动画系统 - 更新所有动画播放器
///
/// 直接使用 AnimationPlayer 的业务方法
pub fn animation_system(
    time: Res<crate::ecs::Time>,
    mut query: Query<(Entity, &mut AnimationPlayer, &mut Transform)>,
) {
    for (entity, mut player, mut transform) in query.iter_mut() {
        player.update(time.delta_seconds);
        player.apply_to_transform(entity.to_bits(), &mut transform);
    }
}

// ============================================================================
// 骨骼动画系统
// ============================================================================

/// 骨骼动画播放器组件
#[derive(Component, Clone)]
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
    /// 创建新的骨骼动画播放器
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
