//  动画系统模块
//
//  提供关键帧动画、骨骼动画和动画播放功能。
//
//  ## 功能特性
//
//  - 关键帧动画系统
//  - 骨骼动画支持
//  - 动画剪辑管理
//  - 动画播放器
//
//  ## 使用示例
//
//  ### 关键帧动画示例
//
//  ```rust
//  use game_engine::animation::{AnimationClip, Keyframe, KeyframeTrack, InterpolationMode};
//  use glam::Vec3;
//
//  // 创建动画剪辑
//  let mut clip = AnimationClip::new("walk".to_string(), 1.0);
//
//  // 添加位置关键帧
//  let mut position_track = KeyframeTrack::new(InterpolationMode::Linear);
//  position_track.add_keyframe(Keyframe::new(0.0, Vec3::ZERO));
//  position_track.add_keyframe(Keyframe::new(1.0, Vec3::new(10.0, 0.0, 0.0)));
//  clip.add_track("position".to_string(), position_track);
//
//  // 播放动画
//  let mut player = AnimationPlayer::new();
//  player.play_clip(clip);
//  player.update(0.5); // 更新到0.5秒位置
//  ```
//
//  ### 骨骼动画示例
//
//  ```rust
//  use game_engine::animation::{Skeleton, Bone, BoneTransform, SkeletonAnimationPlayer};
//
//  // 创建骨骼
//  let mut skeleton = Skeleton::new();
//  let root_bone = Bone::new(0, "root".to_string(), BoneTransform::identity());
//  skeleton.add_bone(root_bone);
//
//  // 创建骨骼动画播放器
//  let mut player = SkeletonAnimationPlayer::new(skeleton);
//  player.update(0.016); // 更新一帧
//  ```

/// 动画剪辑 - 包含多条动画轨道的完整动画
pub mod clip;
/// 关键帧系统 - 定义动画关键帧和插值模式
pub mod keyframe;
/// 并行动画系统 - 并行处理多个动画实体
/// 并行功能默认启用，使用rayon进行并行处理
pub mod parallel;
/// 动画播放器 - 播放和控制动画的系统
pub mod player;
/// 动画服务 - 管理所有动画资源的服务
pub mod service;
/// 骨骼系统 - 用于骨骼动画的骨骼和骨骼变换
pub mod skeleton;
/// 皮肤网格 - 支持骨骼蒙皮的网格系统
pub mod skinned_mesh;

pub use clip::AnimationClip;
pub use keyframe::{InterpolationMode, Keyframe, KeyframeTrack};
pub use player::{
    AnimationPlayer, SkeletonAnimationPlayer, animation_system, skeleton_update_system,
};
pub use service::AnimationService;
pub use skeleton::{Bone, BoneTransform, Skeleton, SkeletonPose};
pub use skinned_mesh::{SkinnedMesh, SkinnedMeshPipeline, SkinnedVertex3D};

// GLTF 骨骼加载（需要启用 gltf feature）
#[cfg(feature = "gltf")]
pub use skeleton::build_skeleton_from_gltf;
