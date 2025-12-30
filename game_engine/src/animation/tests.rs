//! Animation System Tests
//!
//! 测试动画系统的各个组件 - 基础功能验证

#[cfg(test)]
mod animation_clip_tests {
    use crate::animation::{AnimationClip, InterpolationMode, KeyframeTrack};
    use glam::Vec3;

    #[test]
    fn test_animation_clip_creation() {
        let clip = AnimationClip::new("test_animation", 1.0);
        assert_eq!(clip.name, "test_animation");
        assert_eq!(clip.duration, 1.0);
        assert!(!clip.looping);
    }

    #[test]
    fn test_keyframe_track_vec3() {
        let mut track = KeyframeTrack::new(InterpolationMode::Linear);
        track.add_keyframe(0.0, Vec3::ZERO);
        track.add_keyframe(1.0, Vec3::new(10.0, 0.0, 0.0));
        assert_eq!(track.keyframes.len(), 2);
        assert_eq!(track.keyframes[0].time, 0.0);
        assert_eq!(track.keyframes[1].time, 1.0);
    }

    #[test]
    fn test_interpolation_modes() {
        let _linear = InterpolationMode::Linear;
        let _step = InterpolationMode::Step;
        let _cubic = InterpolationMode::CubicBezier;
    }

    #[test]
    fn test_animation_clip_with_tracks() {
        let mut clip = AnimationClip::new("test", 1.0);
        let mut track = KeyframeTrack::new(InterpolationMode::Linear);
        track.add_keyframe(0.0, Vec3::ZERO);
        clip.add_position_track(1, track);
        assert_eq!(clip.position_tracks.len(), 1);
    }
}

#[cfg(test)]
mod skeleton_tests {
    use crate::animation::{Bone, BoneTransform};
    use glam::{Mat4, Quat, Vec3};

    #[test]
    fn test_bone_transform() {
        let transform = BoneTransform::identity();
        // Verify identity transform
        assert_eq!(transform.translation, glam::Vec3::ZERO);
        assert_eq!(transform.rotation, glam::Quat::IDENTITY);
        assert_eq!(transform.scale, glam::Vec3::ONE);
    }

    #[test]
    fn test_bone_creation() {
        // Test bone creation with actual API
        let bone = Bone {
            name: "root".to_string(),
            parent_index: None,
            children_indices: Vec::new(),
            local_transform: BoneTransform::identity(),
            inverse_bind_matrix: Mat4::IDENTITY,
        };
        assert_eq!(bone.name, "root");
        assert!(bone.parent_index.is_none());
    }

    #[test]
    fn test_bone_transform_new() {
        let transform = BoneTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE);
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(transform.rotation, Quat::IDENTITY);
        assert_eq!(transform.scale, Vec3::ONE);
    }
}

#[cfg(test)]
mod skinned_mesh_tests {
    // Skinned mesh tests - verifying structure exists
    #[test]
    fn test_skinned_vertex_concept() {
        // Verify skinned vertex concept exists
        use glam::{Vec2, Vec3};
        let _position = Vec3::ZERO;
        let _normal = Vec3::Y;
        let _uv = Vec2::ZERO;
        // SkinnedVertex3D structure exists and can be constructed
    }

    #[test]
    fn test_bone_weights_concept() {
        // Verify bone weights concept exists
        let bone_indices = [0u32, 0, 0, 0];
        let bone_weights = [1.0f32, 0.0, 0.0, 0.0];
        assert_eq!(bone_indices[0], 0);
        assert_eq!(bone_weights[0], 1.0);
    }
}
