//  值对象测试模块
// 
//  提供对域值对象的全面测试覆盖，包括边界情况、验证逻辑和不变性测试。

use crate::domain::value_objects::*;
use glam::{Quat, Vec3};

#[cfg(test)]
mod position_tests {
    use super::*;

    #[test]
    fn test_position_immutability() {
        let pos = Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");
        let x = pos.x();
        let y = pos.y();
        let z = pos.z();
        
        // 验证值对象的不可变性
        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 3.0);
        
        // 确保原始值对象未被修改
        assert_eq!(pos.x(), 1.0);
        assert_eq!(pos.y(), 2.0);
        assert_eq!(pos.z(), 3.0);
    }

    #[test]
    fn test_position_edge_cases() {
        // 测试极大值
        let large_pos = Position::new(f32::MAX, f32::MAX, f32::MAX);
        assert!(large_pos.is_some());
        
        // 测试极小值
        let small_pos = Position::new(f32::MIN, f32::MIN, f32::MIN);
        assert!(small_pos.is_some());
        
        // 测试零值
        let zero_pos = Position::new(0.0, 0.0, 0.0);
        assert!(zero_pos.is_some());
    }

    #[test]
    fn test_position_equality() {
        let pos1 = Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");
        let pos2 = Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");
        let pos3 = Position::new(1.0, 2.0, 3.1).expect("Test: operation should succeed");
        
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_position_offset_validation() {
        let pos = Position::new(f32::MAX - 1.0, 0.0, 0.0).expect("Test: operation should succeed");
        let large_offset = Vec3::new(2.0, 0.0, 0.0); // 会导致溢出
        
        // 偏移可能导致无效值
        let result = pos.offset(large_offset);
        assert!(result.is_none());
    }

    #[test]
    fn test_position_from_vec3() {
        let vec = Vec3::new(1.5, 2.5, 3.5);
        let pos = Position::from_vec3(vec);
        assert!(pos.is_some());
        
        let pos = pos.expect("Test: operation should succeed");
        assert_eq!(pos.to_vec3(), vec);
    }
}

#[cfg(test)]
mod rotation_tests {
    use super::*;

    #[test]
    fn test_rotation_immutability() {
        let rot = Rotation::identity();
        let quat = rot.to_quat();
        
        // 验证四元数的归一化
        assert!((quat.length() - 1.0).abs() < 0.0001);
        
        // 确保原始旋转对象未被修改
        assert_eq!(rot.to_quat(), quat);
    }

    #[test]
    fn test_rotation_identity_properties() {
        let identity = Rotation::identity();
        let quat = identity.to_quat();
        
        // 单位四元数的性质
        assert_eq!(quat.x, 0.0);
        assert_eq!(quat.y, 0.0);
        assert_eq!(quat.z, 0.0);
        assert_eq!(quat.w, 1.0);
    }

    #[test]
    fn test_rotation_combine_properties() {
        let rot1 = Rotation::from_euler(0.0, std::f32::consts::PI / 2.0, 0.0);
        let rot2 = Rotation::from_euler(0.0, std::f32::consts::PI / 2.0, 0.0);
        let combined = rot1.combine(rot2);
        
        // 组合旋转应该保持归一化
        let quat = combined.to_quat();
        assert!((quat.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_rotation_inverse_properties() {
        let rot = Rotation::from_euler(1.0, 0.5, 0.25);
        let inv = rot.inverse();
        let combined = rot.combine(inv);
        
        // 旋转和逆旋转的组合应该接近单位旋转
        let quat = combined.to_quat();
        let identity = Rotation::identity().to_quat();
        
        let diff = (quat.x - identity.x).abs() + 
                  (quat.y - identity.y).abs() + 
                  (quat.z - identity.z).abs() + 
                  (quat.w - identity.w).abs();
        assert!(diff < 0.001);
    }
}

#[cfg(test)]
mod scale_tests {
    use super::*;

    #[test]
    fn test_scale_immutability() {
        let scale = Scale::new(2.0, 3.0, 4.0).expect("Test: operation should succeed");
        let x = scale.x();
        let y = scale.y();
        let z = scale.z();
        
        assert_eq!(x, 2.0);
        assert_eq!(y, 3.0);
        assert_eq!(z, 4.0);
        
        // 确保原始值对象未被修改
        assert_eq!(scale.x(), 2.0);
        assert_eq!(scale.y(), 3.0);
        assert_eq!(scale.z(), 4.0);
    }

    #[test]
    fn test_scale_validation_comprehensive() {
        // 测试有效缩放值
        assert!(Scale::new(0.1, 0.1, 0.1).is_some());
        assert!(Scale::new(1.0, 1.0, 1.0).is_some());
        assert!(Scale::new(100.0, 100.0, 100.0).is_some());
        
        // 测试无效缩放值
        assert!(Scale::new(0.0, 1.0, 1.0).is_none());
        assert!(Scale::new(-1.0, 1.0, 1.0).is_none());
        assert!(Scale::new(1.0, 0.0, 1.0).is_none());
        assert!(Scale::new(1.0, -1.0, 1.0).is_none());
        assert!(Scale::new(1.0, 1.0, 0.0).is_none());
        assert!(Scale::new(1.0, 1.0, -1.0).is_none());
    }

    #[test]
    fn test_scale_uniform_properties() {
        let uniform_scale = Scale::uniform(2.0).expect("Test: operation should succeed");
        assert_eq!(uniform_scale.x(), 2.0);
        assert_eq!(uniform_scale.y(), 2.0);
        assert_eq!(uniform_scale.z(), 2.0);
    }

    #[test]
    fn test_scale_combine_properties() {
        let scale1 = Scale::new(2.0, 3.0, 4.0).expect("Test: operation should succeed");
        let scale2 = Scale::new(0.5, 2.0, 1.5).expect("Test: operation should succeed");
        let combined = scale1.combine(scale2);
        
        assert_eq!(combined.x(), 1.0);  // 2.0 * 0.5
        assert_eq!(combined.y(), 6.0);  // 3.0 * 2.0
        assert_eq!(combined.z(), 6.0);  // 4.0 * 1.5
    }
}

#[cfg(test)]
mod transform_tests {
    use super::*;

    #[test]
    fn test_transform_immutability() {
        let pos = Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");
        let rot = Rotation::identity();
        let scale = Scale::uniform(2.0).expect("Test: operation should succeed");
        
        let transform = Transform::new(pos, rot, scale);
        
        assert_eq!(transform.position(), pos);
        assert_eq!(transform.rotation(), rot);
        assert_eq!(transform.scale(), scale);
    }

    #[test]
    fn test_transform_identity_properties() {
        let identity = Transform::identity();
        
        assert_eq!(identity.position(), Position::default());
        assert_eq!(identity.rotation(), Rotation::identity());
        assert_eq!(identity.scale(), Scale::default());
    }

    #[test]
    fn test_transform_with_methods() {
        let transform = Transform::identity();
        let pos = Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");
        let rot = Rotation::from_euler(0.0, 1.0, 0.0);
        let scale = Scale::uniform(2.0).expect("Test: operation should succeed");
        
        let new_transform = transform
            .with_position(pos)
            .with_rotation(rot)
            .with_scale(scale);
        
        assert_eq!(new_transform.position(), pos);
        assert_eq!(new_transform.rotation(), rot);
        assert_eq!(new_transform.scale(), scale);
    }

    #[test]
    fn test_transform_combine_properties() {
        let transform1 = Transform::identity()
            .with_position(Position::new(1.0, 0.0, 0.0).expect("Test: operation should succeed"))
            .with_scale(Scale::uniform(2.0).expect("Test: operation should succeed"));
            
        let transform2 = Transform::identity()
            .with_position(Position::new(0.0, 1.0, 0.0).expect("Test: operation should succeed"))
            .with_scale(Scale::uniform(0.5).expect("Test: operation should succeed"));
            
        let combined = transform1.combine(transform2);
        
        // 验证组合变换的性质
        assert!(combined.position().x() > 0.0);
        assert!(combined.position().y() > 0.0);
        assert_eq!(combined.scale().x(), 1.0); // 2.0 * 0.5
    }
}

#[cfg(test)]
mod volume_tests {
    use super::*;

    #[test]
    fn test_volume_boundary_values() {
        // 测试边界值
        assert!(Volume::new(0.0).is_some());
        assert!(Volume::new(1.0).is_some());
        assert!(Volume::new(0.5).is_some());
        
        // 测试超出边界的值
        assert!(Volume::new(-0.001).is_none());
        assert!(Volume::new(1.001).is_none());
    }

    #[test]
    fn test_volume_immutability() {
        let volume = Volume::new(0.75).expect("Test: operation should succeed");
        let value = volume.value();
        
        assert_eq!(value, 0.75);
        assert_eq!(volume.value(), 0.75); // 确保未被修改
    }

    #[test]
    fn test_volume_lerp_boundary() {
        let vol1 = Volume::new(0.0).expect("Test: operation should succeed");
        let vol2 = Volume::new(1.0).expect("Test: operation should succeed");
        
        // 测试插值边界
        assert_eq!(vol1.lerp(vol2, 0.0).value(), 0.0);
        assert_eq!(vol1.lerp(vol2, 1.0).value(), 1.0);
        assert_eq!(vol1.lerp(vol2, 0.5).value(), 0.5);
    }

    #[test]
    fn test_volume_new_unchecked_clamping() {
        let volume = Volume::new_unchecked(-0.5); // 应该被限制为0.0
        assert_eq!(volume.value(), 0.0);
        
        let volume = Volume::new_unchecked(1.5); // 应该被限制为1.0
        assert_eq!(volume.value(), 1.0);
    }
}

#[cfg(test)]
mod mass_tests {
    use super::*;

    #[test]
    fn test_mass_boundary_values() {
        // 测试边界值
        assert!(Mass::new(0.001).is_some());
        assert!(Mass::new(1.0).is_some());
        assert!(Mass::new(f32::MAX).is_some());
        
        // 测试无效值
        assert!(Mass::new(0.0).is_none());
        assert!(Mass::new(-0.001).is_none());
    }

    #[test]
    fn test_mass_immutability() {
        let mass = Mass::new(10.0).expect("Test: operation should succeed");
        let value = mass.value();
        
        assert_eq!(value, 10.0);
        assert_eq!(mass.value(), 10.0); // 确保未被修改
    }

    #[test]
    fn test_mass_new_unchecked_clamping() {
        let mass = Mass::new_unchecked(-5.0); // 应该被限制为0.0
        assert_eq!(mass.value(), 0.0);
        
        let mass = Mass::new_unchecked(10.0); // 应该保持不变
        assert_eq!(mass.value(), 10.0);
    }
}

#[cfg(test)]
mod velocity_tests {
    use super::*;

    #[test]
    fn test_velocity_immutability() {
        let vel = Velocity::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");
        let x = vel.x();
        let y = vel.y();
        let z = vel.z();
        
        assert_eq!(x, 1.0);
        assert_eq!(y, 2.0);
        assert_eq!(z, 3.0);
        
        // 确保原始值对象未被修改
        assert_eq!(vel.x(), 1.0);
        assert_eq!(vel.y(), 2.0);
        assert_eq!(vel.z(), 3.0);
    }

    #[test]
    fn test_velocity_magnitude_properties() {
        let vel = Velocity::new(3.0, 4.0, 0.0).expect("Test: operation should succeed");
        assert_eq!(vel.magnitude(), 5.0);
        assert_eq!(vel.magnitude_squared(), 25.0);
        
        let zero_vel = Velocity::zero();
        assert_eq!(zero_vel.magnitude(), 0.0);
        assert_eq!(zero_vel.magnitude_squared(), 0.0);
    }

    #[test]
    fn test_velocity_normalized_edge_cases() {
        let zero_vel = Velocity::zero();
        assert!(zero_vel.normalized().is_none());
        
        let vel = Velocity::new(1.0, 0.0, 0.0).expect("Test: operation should succeed");
        let normalized = vel.normalized().expect("Test: operation should succeed");
        assert!((normalized.magnitude() - 1.0).abs() < 0.0001);
    }
}

#[cfg(test)]
mod duration_tests {
    use super::*;

    #[test]
    fn test_duration_boundary_values() {
        // 测试边界值
        assert!(Duration::new(0.0).is_some());
        assert!(Duration::new(0.001).is_some());
        assert!(Duration::new(f32::MAX).is_some());
        
        // 测试无效值
        assert!(Duration::new(-0.001).is_none());
        assert!(Duration::new(f32::MIN).is_none());
    }

    #[test]
    fn test_duration_conversions() {
        let duration = Duration::new(5.5).expect("Test: operation should succeed");
        assert_eq!(duration.seconds(), 5.5);
        assert_eq!(duration.millis(), 5500.0);
        
        let from_millis = Duration::from_millis(1000.0).expect("Test: operation should succeed");
        assert_eq!(from_millis.seconds(), 1.0);
        assert_eq!(from_millis.millis(), 1000.0);
    }

    #[test]
    fn test_duration_new_unchecked_clamping() {
        let duration = Duration::new_unchecked(-1.0); // 应该被限制为0.0
        assert_eq!(duration.seconds(), 0.0);
        
        let duration = Duration::new_unchecked(10.0); // 应该保持不变
        assert_eq!(duration.seconds(), 10.0);
    }
}

#[cfg(test)]
mod value_object_serialization_tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_position_serialization() {
        let pos = Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");
        let serialized = serde_json::to_string(&pos).expect("Test: operation should succeed");
        let deserialized: Position = serde_json::from_str(&serialized).expect("Test: operation should succeed");
        
        assert_eq!(pos, deserialized);
    }

    #[test]
    fn test_rotation_serialization() {
        let rot = Rotation::from_euler(0.0, 1.0, 0.0);
        let serialized = serde_json::to_string(&rot).expect("Test: operation should succeed");
        let deserialized: Rotation = serde_json::from_str(&serialized).expect("Test: operation should succeed");
        
        assert_eq!(rot.to_quat(), deserialized.to_quat());
    }

    #[test]
    fn test_scale_serialization() {
        let scale = Scale::new(2.0, 3.0, 4.0).expect("Test: operation should succeed");
        let serialized = serde_json::to_string(&scale).expect("Test: operation should succeed");
        let deserialized: Scale = serde_json::from_str(&serialized).expect("Test: operation should succeed");
        
        assert_eq!(scale, deserialized);
    }

    #[test]
    fn test_transform_serialization() {
        let transform = Transform::identity()
            .with_position(Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed"))
            .with_scale(Scale::uniform(2.0).expect("Test: operation should succeed"));

        let serialized = serde_json::to_string(&transform).expect("Test: operation should succeed");
        let deserialized: Transform = serde_json::from_str(&serialized).expect("Test: operation should succeed");

        assert_eq!(transform, deserialized);
    }
}

#[cfg(test)]
mod value_object_edge_case_tests {
    use super::*;

    #[test]
    fn test_position_nan_handling() {
        // 测试NaN值
        let pos_with_nan = Position::new(f32::NAN, 1.0, 1.0);
        // NaN的相等性检查总是失败，所以我们只验证它能创建
        assert!(pos_with_nan.is_some());
    }

    #[test]
    fn test_position_infinity_handling() {
        // 测试无穷大值
        let pos_with_inf = Position::new(f32::INFINITY, 1.0, 1.0);
        assert!(pos_with_inf.is_some());

        let pos_with_neg_inf = Position::new(f32::NEG_INFINITY, 1.0, 1.0);
        assert!(pos_with_neg_inf.is_some());
    }

    #[test]
    fn test_rotation_gimbal_lock() {
        // 测试万向锁情况
        let rot1 = Rotation::from_euler(std::f32::consts::PI / 2.0, 0.0, 0.0);
        let rot2 = Rotation::from_euler(std::f32::consts::PI / 2.0, 1.0, 0.0);

        // 组合旋转可能遇到万向锁
        let combined = rot1.combine(rot2);
        // 验证结果仍然是归一化的四元数
        let quat = combined.to_quat();
        assert!((quat.length() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_scale_extreme_values() {
        // 测试非常小的缩放值
        let tiny_scale = Scale::new(0.0001, 0.0001, 0.0001);
        assert!(tiny_scale.is_some());

        // 测试非常大的缩放值
        let huge_scale = Scale::new(1000.0, 1000.0, 1000.0);
        assert!(huge_scale.is_some());
    }

    #[test]
    fn test_volume_exact_boundaries() {
        // 测试精确边界值
        assert!(Volume::new(0.0).is_ok());
        assert!(Volume::new(1.0).is_ok());

        // 测试浮点精度边界
        let epsilon = 0.0001;
        assert!(Volume::new(0.0 - epsilon).is_err());
        assert!(Volume::new(1.0 + epsilon).is_err());
    }

    #[test]
    fn test_mass_extreme_values() {
        // 测试极端质量值
        let tiny_mass = Mass::new(0.000001);
        assert!(tiny_mass.is_some());

        let huge_mass = Mass::new(1000000.0);
        assert!(huge_mass.is_some());
    }

    #[test]
    fn test_velocity_zero_normalization() {
        // 测试零速度向量的归一化
        let zero_vel = Velocity::zero();
        assert!(zero_vel.normalized().is_none());
    }

    #[test]
    fn test_velocity_very_small_magnitude() {
        // 测试非常小的速度
        let tiny_vel = Velocity::new(0.0001, 0.0001, 0.0001).expect("Test: operation should succeed");
        assert!(tiny_vel.magnitude() < 0.001);

        let normalized = tiny_vel.normalized();
        // 即使是非常小的向量，归一化也应该成功
        assert!(normalized.is_some());
    }

    #[test]
    fn test_duration_overflow_protection() {
        // 测试持续时间的溢出保护
        let max_duration = Duration::new(f32::MAX);
        assert!(max_duration.is_some());

        let max_millis = Duration::from_millis(f32::MAX);
        assert!(max_millis.is_some());
    }
}

#[cfg(test)]
mod value_object_composition_tests {
    use super::*;

    #[test]
    fn test_transform_decomposition() {
        let pos = Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");
        let rot = Rotation::from_euler(0.1, 0.2, 0.3);
        let scale = Scale::new(1.0, 2.0, 3.0).expect("Test: operation should succeed");

        let transform = Transform::new(pos, rot, scale);

        // 验证分解后的组件与原始组件相同
        assert_eq!(transform.position(), pos);
        assert_eq!(transform.rotation(), rot);
        assert_eq!(transform.scale(), scale);
    }

    #[test]
    fn test_multiple_transform_combinations() {
        let t1 = Transform::identity()
            .with_position(Position::new(1.0, 0.0, 0.0).expect("Test: operation should succeed"));

        let t2 = Transform::identity()
            .with_position(Position::new(0.0, 1.0, 0.0).expect("Test: operation should succeed"));

        let t3 = Transform::identity()
            .with_position(Position::new(0.0, 0.0, 1.0).expect("Test: operation should succeed"));

        let combined = t1.combine(t2).combine(t3);

        // 验证组合后的位置
        assert!((combined.position().x() - 1.0).abs() < 0.001);
        assert!((combined.position().y() - 1.0).abs() < 0.001);
        assert!((combined.position().z() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_transform_chaining_with_methods() {
        let transform = Transform::identity()
            .with_position(Position::new(1.0, 2.0, 3.0).expect("Test: operation should succeed"))
            .with_rotation(Rotation::from_euler(0.1, 0.2, 0.3))
            .with_scale(Scale::uniform(2.0).expect("Test: operation should succeed"))
            .with_position(Position::new(4.0, 5.0, 6.0).expect("Test: operation should succeed")); // 覆盖之前的位置

        // 验证链式调用的最终位置
        assert_eq!(transform.position().x(), 4.0);
        assert_eq!(transform.position().y(), 5.0);
        assert_eq!(transform.position().z(), 6.0);
    }
}

#[cfg(test)]
mod value_object_business_rule_tests {
    use super::*;

    #[test]
    fn test_scale_business_rule_positive_only() {
        // 业务规则：缩放值必须为正数
        assert!(Scale::new(1.0, 1.0, 1.0).is_some());
        assert!(Scale::new(0.0, 1.0, 1.0).is_none()); // 零值不允许
        assert!(Scale::new(-1.0, 1.0, 1.0).is_none()); // 负值不允许
    }

    #[test]
    fn test_mass_business_rule_positive_only() {
        // 业务规则：质量必须为正数
        assert!(Mass::new(1.0).is_some());
        assert!(Mass::new(0.0).is_none()); // 零质量不允许
        assert!(Mass::new(-1.0).is_none()); // 负质量不允许
    }

    #[test]
    fn test_volume_business_rule_range() {
        // 业务规则：音量必须在0.0到1.0之间
        assert!(Volume::new(0.0).is_ok());
        assert!(Volume::new(0.5).is_ok());
        assert!(Volume::new(1.0).is_ok());

        assert!(Volume::new(-0.1).is_err());
        assert!(Volume::new(1.1).is_err());
    }

    #[test]
    fn test_duration_business_rule_positive_only() {
        // 业务规则：持续时间必须为非负数
        assert!(Duration::new(0.0).is_some());
        assert!(Duration::new(1.0).is_some());

        assert!(Duration::new(-0.1).is_none());
        assert!(Duration::new(-1.0).is_none());
    }
}