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
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
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
        let pos1 = Position::new(1.0, 2.0, 3.0).unwrap();
        let pos2 = Position::new(1.0, 2.0, 3.0).unwrap();
        let pos3 = Position::new(1.0, 2.0, 3.1).unwrap();
        
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_position_offset_validation() {
        let pos = Position::new(f32::MAX - 1.0, 0.0, 0.0).unwrap();
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
        
        let pos = pos.unwrap();
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
        let scale = Scale::new(2.0, 3.0, 4.0).unwrap();
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
        let uniform_scale = Scale::uniform(2.0).unwrap();
        assert_eq!(uniform_scale.x(), 2.0);
        assert_eq!(uniform_scale.y(), 2.0);
        assert_eq!(uniform_scale.z(), 2.0);
    }

    #[test]
    fn test_scale_combine_properties() {
        let scale1 = Scale::new(2.0, 3.0, 4.0).unwrap();
        let scale2 = Scale::new(0.5, 2.0, 1.5).unwrap();
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
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
        let rot = Rotation::identity();
        let scale = Scale::uniform(2.0).unwrap();
        
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
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
        let rot = Rotation::from_euler(0.0, 1.0, 0.0);
        let scale = Scale::uniform(2.0).unwrap();
        
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
            .with_position(Position::new(1.0, 0.0, 0.0).unwrap())
            .with_scale(Scale::uniform(2.0).unwrap());
            
        let transform2 = Transform::identity()
            .with_position(Position::new(0.0, 1.0, 0.0).unwrap())
            .with_scale(Scale::uniform(0.5).unwrap());
            
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
        let volume = Volume::new(0.75).unwrap();
        let value = volume.value();
        
        assert_eq!(value, 0.75);
        assert_eq!(volume.value(), 0.75); // 确保未被修改
    }

    #[test]
    fn test_volume_lerp_boundary() {
        let vol1 = Volume::new(0.0).unwrap();
        let vol2 = Volume::new(1.0).unwrap();
        
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
        let mass = Mass::new(10.0).unwrap();
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
        let vel = Velocity::new(1.0, 2.0, 3.0).unwrap();
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
        let vel = Velocity::new(3.0, 4.0, 0.0).unwrap();
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
        
        let vel = Velocity::new(1.0, 0.0, 0.0).unwrap();
        let normalized = vel.normalized().unwrap();
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
        let duration = Duration::new(5.5).unwrap();
        assert_eq!(duration.seconds(), 5.5);
        assert_eq!(duration.millis(), 5500.0);
        
        let from_millis = Duration::from_millis(1000.0).unwrap();
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
        let pos = Position::new(1.0, 2.0, 3.0).unwrap();
        let serialized = serde_json::to_string(&pos).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(pos, deserialized);
    }

    #[test]
    fn test_rotation_serialization() {
        let rot = Rotation::from_euler(0.0, 1.0, 0.0);
        let serialized = serde_json::to_string(&rot).unwrap();
        let deserialized: Rotation = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(rot.to_quat(), deserialized.to_quat());
    }

    #[test]
    fn test_scale_serialization() {
        let scale = Scale::new(2.0, 3.0, 4.0).unwrap();
        let serialized = serde_json::to_string(&scale).unwrap();
        let deserialized: Scale = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(scale, deserialized);
    }

    #[test]
    fn test_transform_serialization() {
        let transform = Transform::identity()
            .with_position(Position::new(1.0, 2.0, 3.0).unwrap())
            .with_scale(Scale::uniform(2.0).unwrap());
            
        let serialized = serde_json::to_string(&transform).unwrap();
        let deserialized: Transform = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(transform, deserialized);
    }
}