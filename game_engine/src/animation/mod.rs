//! # Animation System
//!
//! 本模块提供完整的角色动画系统，支持关键帧动画、骨骼动画和动画混合。
//!
//! ## 功能特性
//!
//! - **关键帧动画系统** - 支持位置、旋转、缩放关键帧
//! - **骨骼动画** - 完整的骨骼层级和蒙皮网格支持
//! - **动画混合** - 多个动画层叠加和混合
//! - **动画剪辑管理** - 动画资源的加载和管理
//! - **并行动画处理** - 使用rayon并行处理大量动画实体
//!
//! ## 主要组件
//!
//! - [`AnimationClip`] - 动画剪辑，包含完整的动画数据
//! - [`AnimationPlayer`] - 动画播放器，控制动画播放
//! - [`SkeletonAnimationPlayer`] - 骨骼动画播放器
//! - [`Skeleton`] - 骨骼结构定义
//! - [`Bone`] - 单个骨骼节点
//! - [`SkinnedMesh`] - 蒙皮网格组件
//! - [`AnimationService`] - 动画资源管理服务
//!
//! ## 使用示例
//!
//! ### 关键帧动画示例
//!
//! ```rust,no_run
//! use game_engine::animation::{AnimationClip, Keyframe, KeyframeTrack, InterpolationMode};
//! use glam::Vec3;
//!
//! // 创建动画剪辑
//! let mut clip = AnimationClip::new("walk".to_string(), 1.0);
//!
//! // 添加位置关键帧
//! let mut position_track = KeyframeTrack::new(InterpolationMode::Linear);
//! position_track.add_keyframe(Keyframe::new(0.0, Vec3::ZERO));
//! position_track.add_keyframe(Keyframe::new(1.0, Vec3::new(10.0, 0.0, 0.0)));
//! clip.add_track("position".to_string(), position_track);
//!
//! // 播放动画
//! let mut player = AnimationPlayer::new();
//! player.play_clip(clip);
//! player.update(0.5); // 更新到0.5秒位置
//! ```
//!
//! ### 骨骼动画示例
//!
//! ```rust,no_run
//! use game_engine::animation::{Skeleton, Bone, BoneTransform, SkeletonAnimationPlayer};
//!
//! // 创建骨骼
//! let mut skeleton = Skeleton::new();
//! let root_bone = Bone::new(0, "root".to_string(), BoneTransform::identity());
//! skeleton.add_bone(root_bone);
//!
//! // 创建骨骼动画播放器
//! let mut player = SkeletonAnimationPlayer::new(skeleton);
//! player.update(0.016); // 更新一帧
//! ```
//!
//! ## 性能特性
//!
//! - **并行处理** - 默认启用rayon并行处理动画更新
//! - **GPU加速** - 蒙皮计算可卸载到GPU
//! - **增量更新** - 只更新活动的动画实体
//! - **内存优化** - 动画数据共享和缓存

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
/// SIMD蒙皮 - SIMD加速的骨骼蒙皮批量处理
pub mod simd_skinning;
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

// SIMD蒙皮导出
pub use simd_skinning::{BoneTransforms, SimdSkinningProcessor};
#[cfg(feature = "simd")]
pub use simd_skinning::{SimdBoneInfluence, SimdSkinnedVertex, simd_linear_blend_skinning_system};

// GLTF 骨骼加载（需要启用 gltf feature）
#[cfg(feature = "gltf")]
pub use skeleton::build_skeleton_from_gltf;

// 测试模块
#[cfg(test)]
mod tests;
