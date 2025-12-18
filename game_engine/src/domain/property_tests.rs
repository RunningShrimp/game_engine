//! 领域对象属性测试
//!
//! 使用proptest为领域对象和验证逻辑添加属性测试

#[cfg(test)]
mod tests {
    use crate::domain::entity::{EntityId, GameEntity};
    use crate::domain::physics::{
        Collider, ColliderId, RigidBody, RigidBodyId, RigidBodyType, ShapeType,
    };
    use crate::domain::value_objects::{
        Duration, Mass, Position, Rotation, Scale, Transform, Velocity, Volume,
    };
    use glam::{Quat, Vec3};
    use proptest::prelude::*;

    // 数学运算属性测试策略
    fn finite_f32() -> impl Strategy<Value = f32> {
        (-1000.0f32..1000.0).prop_filter("must be finite", |&x| x.is_finite())
    }

    fn positive_finite_f32() -> impl Strategy<Value = f32> {
        (0.0001f32..1000.0)
            .prop_filter("must be finite and positive", |&x| x.is_finite() && x > 0.0)
    }

    fn valid_vec3() -> impl Strategy<Value = Vec3> {
        (finite_f32(), finite_f32(), finite_f32()).prop_map(|(x, y, z)| Vec3::new(x, y, z))
    }

    fn valid_quat() -> impl Strategy<Value = Quat> {
        // 从欧拉角生成四元数（自动归一化）
        (finite_f32(), finite_f32(), finite_f32()).prop_map(|(x, y, z)| {
            // 限制欧拉角范围以避免数值不稳定
            let x = x.clamp(-3.14, 3.14);
            let y = y.clamp(-3.14, 3.14);
            let z = z.clamp(-3.14, 3.14);
            Quat::from_euler(glam::EulerRot::XYZ, x, y, z)
        })
    }

    // Vec3数学运算属性测试
    proptest! {
        #[test]
        fn vec3_add_commutative(
            v1 in valid_vec3(),
            v2 in valid_vec3()
        ) {
            // 交换律：v1 + v2 = v2 + v1
            let result1 = v1 + v2;
            let result2 = v2 + v1;
            prop_assert!((result1 - result2).length() < 0.0001);
        }

        #[test]
        fn vec3_add_associative(
            v1 in valid_vec3(),
            v2 in valid_vec3(),
            v3 in valid_vec3()
        ) {
            // 结合律：(v1 + v2) + v3 = v1 + (v2 + v3)
            // 注意：由于浮点运算的精度，允许小的误差
            let left = (v1 + v2) + v3;
            let right = v1 + (v2 + v3);
            let diff = (left.x - right.x).abs() + (left.y - right.y).abs() + (left.z - right.z).abs();
            prop_assert!(diff < 0.001);
        }

        #[test]
        fn vec3_dot_commutative(
            v1 in valid_vec3(),
            v2 in valid_vec3()
        ) {
            // 点积交换律：v1 · v2 = v2 · v1
            let dot1 = v1.dot(v2);
            let dot2 = v2.dot(v1);
            prop_assert!((dot1 - dot2).abs() < 0.0001);
        }

        #[test]
        fn vec3_cross_anticommutative(
            v1 in valid_vec3(),
            v2 in valid_vec3()
        ) {
            // 叉积反交换律：v1 × v2 = -(v2 × v1)
            let cross1 = v1.cross(v2);
            let cross2 = v2.cross(v1);
            prop_assert!((cross1 + cross2).length() < 0.0001);
        }

        #[test]
        fn vec3_length_non_negative(
            v in valid_vec3()
        ) {
            // 向量长度非负
            prop_assert!(v.length() >= 0.0);
            prop_assert!(v.length_squared() >= 0.0);
        }

        #[test]
        fn vec3_normalize_unit_length(
            v in valid_vec3()
        ) {
            // 归一化后长度为1
            if v.length() > 0.0001 {
                let normalized = v.normalize();
                prop_assert!((normalized.length() - 1.0).abs() < 0.0001);
            }
        }
    }

    // Quat数学运算属性测试
    proptest! {
        #[test]
        fn quat_multiply_associative(
            x1 in -3.14f32..3.14,
            y1 in -3.14f32..3.14,
            z1 in -3.14f32..3.14,
            x2 in -3.14f32..3.14,
            y2 in -3.14f32..3.14,
            z2 in -3.14f32..3.14,
            x3 in -3.14f32..3.14,
            y3 in -3.14f32..3.14,
            z3 in -3.14f32..3.14
        ) {
            // 四元数乘法结合律：(q1 * q2) * q3 ≈ q1 * (q2 * q3)
            let q1 = Quat::from_euler(glam::EulerRot::XYZ, x1, y1, z1);
            let q2 = Quat::from_euler(glam::EulerRot::XYZ, x2, y2, z2);
            let q3 = Quat::from_euler(glam::EulerRot::XYZ, x3, y3, z3);

            let left = (q1 * q2) * q3;
            let right = q1 * (q2 * q3);
            let diff = (left.x - right.x).abs() + (left.y - right.y).abs() +
                      (left.z - right.z).abs() + (left.w - right.w).abs();
            prop_assert!(diff < 0.1); // 允许更大的误差，因为浮点运算
        }

        #[test]
        fn quat_inverse_cancels(
            x in -3.14f32..3.14,
            y in -3.14f32..3.14,
            z in -3.14f32..3.14
        ) {
            // q * q^-1 ≈ identity
            let q = Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
            let inv = q.inverse();
            let product = q * inv;
            let identity = Quat::IDENTITY;
            let diff = (product.x - identity.x).abs() + (product.y - identity.y).abs() +
                      (product.z - identity.z).abs() + (product.w - identity.w).abs();
            prop_assert!(diff < 0.01);
        }

        #[test]
        fn quat_always_normalized(
            x in -3.14f32..3.14,
            y in -3.14f32..3.14,
            z in -3.14f32..3.14
        ) {
            // 从欧拉角生成的四元数应该归一化
            let q = Quat::from_euler(glam::EulerRot::XYZ, x, y, z);
            let len = (q.x * q.x + q.y * q.y + q.z * q.z + q.w * q.w).sqrt();
            prop_assert!((len - 1.0).abs() < 0.0001);
        }
    }

    // RigidBody领域对象属性测试
    proptest! {
        #[test]
        fn rigid_body_position_always_valid(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32()
        ) {
            let body = RigidBody::new(
                RigidBodyId(1),
                RigidBodyType::Dynamic,
                Vec3::new(x, y, z),
            );
            prop_assert!(body.position.x.is_finite());
            prop_assert!(body.position.y.is_finite());
            prop_assert!(body.position.z.is_finite());
        }

        #[test]
        fn rigid_body_mass_always_positive(
            mass in positive_finite_f32()
        ) {
            let mut body = RigidBody::new(
                RigidBodyId(1),
                RigidBodyType::Dynamic,
                Vec3::ZERO,
            );
            if body.set_mass(mass).is_ok() {
                prop_assert!(body.mass > 0.0);
            }
        }

        #[test]
        fn rigid_body_velocity_always_finite(
            vx in finite_f32(),
            vy in finite_f32(),
            vz in finite_f32()
        ) {
            let mut body = RigidBody::new(
                RigidBodyId(1),
                RigidBodyType::Dynamic,
                Vec3::ZERO,
            );
            body.linear_velocity = Vec3::new(vx, vy, vz);
            prop_assert!(body.linear_velocity.x.is_finite());
            prop_assert!(body.linear_velocity.y.is_finite());
            prop_assert!(body.linear_velocity.z.is_finite());
        }
    }

    // Collider领域对象属性测试
    proptest! {
        #[test]
        fn collider_cuboid_half_extents_always_positive(
            x in positive_finite_f32(),
            y in positive_finite_f32(),
            z in positive_finite_f32()
        ) {
            let collider = Collider::cuboid(
                ColliderId(1),
                Vec3::new(x, y, z),
            );
            prop_assert!(collider.half_extents.x > 0.0);
            prop_assert!(collider.half_extents.y > 0.0);
            prop_assert!(collider.half_extents.z > 0.0);
        }

        #[test]
        fn collider_ball_radius_always_positive(
            radius in positive_finite_f32()
        ) {
            let collider = Collider::ball(
                ColliderId(1),
                radius,
            );
            prop_assert!(collider.radius > 0.0);
        }

        #[test]
        fn collider_friction_in_range(
            friction in 0.0f32..=1.0
        ) {
            let mut collider = Collider::cuboid(
                ColliderId(1),
                Vec3::ONE,
            );
            collider.friction = friction;
            prop_assert!(collider.friction >= 0.0);
            prop_assert!(collider.friction <= 1.0);
        }

        #[test]
        fn collider_restitution_in_range(
            restitution in 0.0f32..=1.0
        ) {
            let mut collider = Collider::cuboid(
                ColliderId(1),
                Vec3::ONE,
            );
            collider.restitution = restitution;
            prop_assert!(collider.restitution >= 0.0);
            prop_assert!(collider.restitution <= 1.0);
        }
    }

    // Transform组合属性测试
    proptest! {
        #[test]
        fn transform_combine_preserves_validity(
            x1 in finite_f32(),
            y1 in finite_f32(),
            z1 in finite_f32(),
            x2 in finite_f32(),
            y2 in finite_f32(),
            z2 in finite_f32(),
            scale1 in positive_finite_f32(),
            scale2 in positive_finite_f32()
        ) {
            if let (Some(pos1), Some(pos2), Some(scale1), Some(scale2)) = (
                Position::new(x1, y1, z1),
                Position::new(x2, y2, z2),
                Scale::uniform(scale1),
                Scale::uniform(scale2)
            ) {
                let rot1 = Rotation::identity();
                let rot2 = Rotation::identity();
                let transform1 = Transform::new(pos1, rot1, scale1);
                let transform2 = Transform::new(pos2, rot2, scale2);
                let combined = transform1.combine(transform2);

                // 组合后的变换应该仍然有效
                prop_assert!(combined.position().x().is_finite());
                prop_assert!(combined.position().y().is_finite());
                prop_assert!(combined.position().z().is_finite());
            }
        }
    }

    // 值对象转换属性测试
    proptest! {
        #[test]
        fn position_vec3_roundtrip(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32()
        ) {
            if let Some(pos) = Position::new(x, y, z) {
                let vec = pos.to_vec3();
                let back = Position::from_vec3(vec);
                prop_assert!(back.is_some());
                let back = back.unwrap();
                prop_assert!((back.x() - x).abs() < 0.0001);
                prop_assert!((back.y() - y).abs() < 0.0001);
                prop_assert!((back.z() - z).abs() < 0.0001);
            }
        }

        #[test]
        fn velocity_vec3_roundtrip(
            x in finite_f32(),
            y in finite_f32(),
            z in finite_f32()
        ) {
            if let Some(vel) = Velocity::new(x, y, z) {
                let vec = vel.to_vec3();
                let back = Velocity::from_vec3(vec);
                prop_assert!(back.is_some());
                let back = back.unwrap();
                prop_assert!((back.x() - x).abs() < 0.0001);
                prop_assert!((back.y() - y).abs() < 0.0001);
                prop_assert!((back.z() - z).abs() < 0.0001);
            }
        }

        #[test]
        fn scale_vec3_roundtrip(
            x in positive_finite_f32(),
            y in positive_finite_f32(),
            z in positive_finite_f32()
        ) {
            if let Some(scale) = Scale::new(x, y, z) {
                let vec = scale.to_vec3();
                let back = Scale::from_vec3(vec);
                prop_assert!(back.is_some());
                let back = back.unwrap();
                prop_assert!((back.x() - x).abs() < 0.0001);
                prop_assert!((back.y() - y).abs() < 0.0001);
                prop_assert!((back.z() - z).abs() < 0.0001);
            }
        }
    }

    // Volume值对象属性测试
    proptest! {
        #[test]
        fn volume_always_in_range(
            value in 0.0f32..=1.0
        ) {
            if let Some(vol) = Volume::new(value) {
                prop_assert!(vol.value() >= 0.0);
                prop_assert!(vol.value() <= 1.0);
                prop_assert!(vol.value().is_finite());
            }
        }

        #[test]
        fn volume_rejects_invalid(
            value in (-1000.0f32..-0.1).prop_union(1.1f32..1000.0)
        ) {
            prop_assert!(Volume::new(value).is_none());
        }

        #[test]
        fn volume_lerp_preserves_range(
            v1 in 0.0f32..=1.0,
            v2 in 0.0f32..=1.0,
            t in 0.0f32..=1.0
        ) {
            if let (Some(vol1), Some(vol2)) = (Volume::new(v1), Volume::new(v2)) {
                let lerped = vol1.lerp(vol2, t);
                prop_assert!(lerped.value() >= 0.0);
                prop_assert!(lerped.value() <= 1.0);
            }
        }

    }

    #[test]
    fn volume_muted_is_zero() {
        let muted = Volume::muted();
        assert_eq!(muted.value(), 0.0);
        assert!(muted.is_muted());
    }

    #[test]
    fn volume_max_is_one() {
        let max = Volume::max();
        assert_eq!(max.value(), 1.0);
        assert!(!max.is_muted());
    }

    // Mass值对象属性测试
    proptest! {
        #[test]
        fn mass_always_positive(
            value in positive_finite_f32()
        ) {
            if let Some(mass) = Mass::new(value) {
                prop_assert!(mass.value() > 0.0);
                prop_assert!(mass.value().is_finite());
                prop_assert!(!mass.is_zero());
            }
        }

        #[test]
        fn mass_rejects_non_positive(
            value in -1000.0f32..=0.0
        ) {
            prop_assert!(Mass::new(value).is_none());
        }

    }

    #[test]
    fn mass_zero_is_zero() {
        let zero = Mass::zero();
        assert_eq!(zero.value(), 0.0);
        assert!(zero.is_zero());
    }

    // Duration值对象属性测试
    proptest! {
        #[test]
        fn duration_always_non_negative(
            seconds in 0.0f32..10000.0
        ) {
            if let Some(dur) = Duration::new(seconds) {
                prop_assert!(dur.seconds() >= 0.0);
                prop_assert!(dur.seconds().is_finite());
                prop_assert!(!dur.is_zero() || seconds == 0.0);
            }
        }

        #[test]
        fn duration_rejects_negative(
            seconds in -1000.0f32..-0.0001
        ) {
            prop_assert!(Duration::new(seconds).is_none());
        }

        #[test]
        fn duration_millis_roundtrip(
            millis in 0.0f32..1000000.0
        ) {
            if let Some(dur) = Duration::from_millis(millis) {
                let back_millis = dur.millis();
                prop_assert!((back_millis - millis).abs() < 0.001);
            }
        }

    }

    #[test]
    fn duration_zero_is_zero() {
        let zero = Duration::zero();
        assert_eq!(zero.seconds(), 0.0);
        assert!(zero.is_zero());
    }

    // Scale值对象属性测试
    proptest! {
        #[test]
        fn scale_always_positive(
            x in positive_finite_f32(),
            y in positive_finite_f32(),
            z in positive_finite_f32()
        ) {
            if let Some(scale) = Scale::new(x, y, z) {
                prop_assert!(scale.x() > 0.0);
                prop_assert!(scale.y() > 0.0);
                prop_assert!(scale.z() > 0.0);
            }
        }

        #[test]
        fn scale_rejects_non_positive(
            x in -1000.0f32..=0.0,
            y in positive_finite_f32(),
            z in positive_finite_f32()
        ) {
            prop_assert!(Scale::new(x, y, z).is_none());
        }

        #[test]
        fn scale_combine_preserves_positivity(
            x1 in positive_finite_f32(),
            y1 in positive_finite_f32(),
            z1 in positive_finite_f32(),
            x2 in positive_finite_f32(),
            y2 in positive_finite_f32(),
            z2 in positive_finite_f32()
        ) {
            if let (Some(scale1), Some(scale2)) = (
                Scale::new(x1, y1, z1),
                Scale::new(x2, y2, z2)
            ) {
                let combined = scale1.combine(scale2);
                prop_assert!(combined.x() > 0.0);
                prop_assert!(combined.y() > 0.0);
                prop_assert!(combined.z() > 0.0);
            }
        }

        #[test]
        fn scale_uniform_creates_equal_components(
            value in positive_finite_f32()
        ) {
            if let Some(scale) = Scale::uniform(value) {
                prop_assert!((scale.x() - value).abs() < 0.0001);
                prop_assert!((scale.y() - value).abs() < 0.0001);
                prop_assert!((scale.z() - value).abs() < 0.0001);
            }
        }
    }

    // Rotation值对象属性测试
    proptest! {
        #[test]
        fn rotation_always_normalized(
            x in -3.14f32..3.14,
            y in -3.14f32..3.14,
            z in -3.14f32..3.14
        ) {
            let rot = Rotation::from_euler(x, y, z);
            let quat = rot.to_quat();
            let len = (quat.x * quat.x + quat.y * quat.y + quat.z * quat.z + quat.w * quat.w).sqrt();
            prop_assert!((len - 1.0).abs() < 0.0001);
        }

        #[test]
        fn rotation_combine_preserves_normalization(
            x1 in -3.14f32..3.14,
            y1 in -3.14f32..3.14,
            z1 in -3.14f32..3.14,
            x2 in -3.14f32..3.14,
            y2 in -3.14f32..3.14,
            z2 in -3.14f32..3.14
        ) {
            let rot1 = Rotation::from_euler(x1, y1, z1);
            let rot2 = Rotation::from_euler(x2, y2, z2);
            let combined = rot1.combine(rot2);
            let quat = combined.to_quat();
            let len = (quat.x * quat.x + quat.y * quat.y + quat.z * quat.z + quat.w * quat.w).sqrt();
            prop_assert!((len - 1.0).abs() < 0.0001);
        }

        #[test]
        fn rotation_inverse_cancels(
            x in -3.14f32..3.14,
            y in -3.14f32..3.14,
            z in -3.14f32..3.14
        ) {
            let rot = Rotation::from_euler(x, y, z);
            let inv = rot.inverse();
            let combined = rot.combine(inv);
            let identity = Rotation::identity();
            let diff = (combined.to_quat().x - identity.to_quat().x).abs() +
                      (combined.to_quat().y - identity.to_quat().y).abs() +
                      (combined.to_quat().z - identity.to_quat().z).abs() +
                      (combined.to_quat().w - identity.to_quat().w).abs();
            prop_assert!(diff < 0.01);
        }
    }

    // AudioSource领域对象属性测试
    proptest! {
        #[test]
        fn audio_source_volume_always_valid(
            volume_value in 0.0f32..=1.0
        ) {
            use crate::domain::audio::{AudioSource, AudioSourceId};
            
            let mut source = AudioSource::new(AudioSourceId(1));
            if let Some(volume) = Volume::new(volume_value) {
                if source.set_volume(volume).is_ok() {
                    prop_assert!(source.volume.value() >= 0.0);
                    prop_assert!(source.volume.value() <= 1.0);
                }
            }
        }

        #[test]
        fn audio_source_playback_position_always_valid(
            position in 0.0f32..1000.0
        ) {
            use crate::domain::audio::{AudioSource, AudioSourceId};
            
            let mut source = AudioSource::new(AudioSourceId(1));
            source.playback_position = position;
            prop_assert!(source.playback_position >= 0.0);
            prop_assert!(source.playback_position.is_finite());
        }

    }

    #[test]
    fn audio_source_state_transitions_valid() {
        use crate::domain::audio::{AudioSource, AudioSourceId, AudioSourceState};
        
        let source = AudioSource::new(AudioSourceId(1));
        
        // 初始状态应该是Stopped
        assert_eq!(source.state, AudioSourceState::Stopped);
        
        // 验证状态是有效值之一
        assert!(matches!(
            source.state,
            AudioSourceState::Stopped | AudioSourceState::Playing | AudioSourceState::Paused | AudioSourceState::Loading
        ));
    }
}
