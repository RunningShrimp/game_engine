pub mod keyframe;
pub mod clip;
pub mod player;
pub mod service;
pub mod skeleton;
pub mod skinned_mesh;

pub use keyframe::{Keyframe, KeyframeTrack, InterpolationMode};
pub use clip::AnimationClip;
pub use player::{
    AnimationPlayer, 
    SkeletonAnimationPlayer,
    animation_system,
    skeleton_update_system,
};
pub use service::AnimationService;
pub use skeleton::{Bone, BoneTransform, Skeleton, SkeletonPose};
pub use skinned_mesh::{SkinnedMesh, SkinnedVertex3D, SkinnedMeshPipeline};

// GLTF 骨骼加载（需要启用 gltf feature）
#[cfg(feature = "gltf")]
pub use skeleton::build_skeleton_from_gltf;
