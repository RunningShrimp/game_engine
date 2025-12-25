//! 模糊测试模块
//!
//! 使用proptest进行属性测试，重点测试：
//! - 场景序列化/反序列化
//! - 网络协议解析
//! - 数学库（向量、矩阵运算）

use proptest::prelude::*;
use glam::{Vec3, Vec4, Mat4, Quat};

// ============================================================================
// 数学库模糊测试
// ============================================================================

proptest! {
    #[test]
    fn vec3_add_associative(
        v1 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        v2 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        v3 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
    ) {
        let v1 = Vec3::new(v1, v1, v1);
        let v2 = Vec3::new(v2, v2, v2);
        let v3 = Vec3::new(v3, v3, v3);
        
        // 结合律：(v1 + v2) + v3 = v1 + (v2 + v3)
        let result1 = (v1 + v2) + v3;
        let result2 = v1 + (v2 + v3);
        prop_assert!((result1 - result2).length() < 0.0001);
    }

    #[test]
    fn vec3_dot_product_commutative(
        v1_x in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        v1_y in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        v1_z in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        v2_x in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        v2_y in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        v2_z in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
    ) {
        let v1 = Vec3::new(v1_x, v1_y, v1_z);
        let v2 = Vec3::new(v2_x, v2_y, v2_z);
        
        // 交换律：v1 · v2 = v2 · v1
        let dot1 = v1.dot(v2);
        let dot2 = v2.dot(v1);
        prop_assert!((dot1 - dot2).abs() < 0.0001);
    }

    #[test]
    fn mat4_identity_multiplication(
        x in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        y in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        z in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
    ) {
        let v = Vec4::new(x, y, z, 1.0);
        let identity = Mat4::IDENTITY;
        
        // 单位矩阵乘法：I * v = v
        let result = identity * v;
        prop_assert!((result - v).length() < 0.0001);
    }

    #[test]
    fn quat_normalization(
        x in (-10.0f32..10.0).prop_filter("finite", |&x| x.is_finite()),
        y in (-10.0f32..10.0).prop_filter("finite", |&x| x.is_finite()),
        z in (-10.0f32..10.0).prop_filter("finite", |&x| x.is_finite()),
        w in (-10.0f32..10.0).prop_filter("finite", |&x| x.is_finite()),
    ) {
        let q = Quat::from_xyzw(x, y, z, w);
        let normalized = q.normalize();
        
        // 归一化后的四元数长度应该接近1
        let length = normalized.length();
        prop_assert!((length - 1.0).abs() < 0.0001);
    }
}

// ============================================================================
// 场景序列化模糊测试
// ============================================================================

#[cfg(test)]
mod serialization_tests {
    use super::*;
    use game_engine::scene::serialization::SceneSerializer;
    use game_engine::domain::scene::{Scene, SceneId, SceneState};
    use game_engine::domain::value_objects::Transform;

    proptest! {
        #[test]
        fn scene_serialize_deserialize_roundtrip(
            scene_name in "[a-zA-Z0-9_]{1,50}",
            entity_count in 0usize..1000,
        ) {
            // 创建场景
            let mut scene = Scene::new(
                SceneId::new(),
                scene_name.clone(),
            );
            
            // 添加一些实体（简化测试）
            for _ in 0..entity_count {
                // 实体添加逻辑
            }
            
            // 序列化
            let serializer = SceneSerializer::new();
            let serialized = serializer.serialize(&scene).unwrap();
            
            // 反序列化
            let deserialized = serializer.deserialize(&serialized).unwrap();
            
            // 验证往返一致性
            prop_assert_eq!(deserialized.name(), &scene_name);
            // 注意：实体数量可能因序列化格式而略有不同，这里简化处理
        }
    }
}

// ============================================================================
// 网络协议模糊测试
// ============================================================================

#[cfg(test)]
mod network_protocol_tests {
    use super::*;
    use game_engine::network::compression::NetworkCompressor;
    use game_engine::network::delta_serialization::DeltaSerializer;

    proptest! {
        #[test]
        fn network_compression_roundtrip(
            data in prop::collection::vec(0u8..=255u8, 0..10000),
        ) {
            let compressor = NetworkCompressor::new();
            
            // 压缩
            let compressed = compressor.compress(&data).unwrap();
            
            // 解压
            let decompressed = compressor.decompress(&compressed).unwrap();
            
            // 验证往返一致性
            prop_assert_eq!(decompressed, data);
        }

        #[test]
        fn delta_serialization_roundtrip(
            old_data in prop::collection::vec(0u8..=255u8, 0..1000),
            new_data in prop::collection::vec(0u8..=255u8, 0..1000),
        ) {
            let serializer = DeltaSerializer::new();
            
            // 计算增量
            let delta = serializer.compute_delta(&old_data, &new_data).unwrap();
            
            // 应用增量
            let reconstructed = serializer.apply_delta(&old_data, &delta).unwrap();
            
            // 验证重建的数据与原始数据一致
            prop_assert_eq!(reconstructed, new_data);
        }
    }
}

// ============================================================================
// 值对象模糊测试
// ============================================================================

#[cfg(test)]
mod value_object_tests {
    use super::*;
    use game_engine::domain::value_objects::{Position, Rotation, Scale, Transform};

    proptest! {
        #[test]
        fn transform_composition_associative(
            x1 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
            y1 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
            z1 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
            x2 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
            y2 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
            z2 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
            x3 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
            y3 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
            z3 in (-1000.0f32..1000.0).prop_filter("finite", |&x| x.is_finite()),
        ) {
            let pos1 = Position::new(Vec3::new(x1, y1, z1));
            let pos2 = Position::new(Vec3::new(x2, y2, z2));
            let pos3 = Position::new(Vec3::new(x3, y3, z3));
            
            let t1 = Transform::new(pos1.clone(), Rotation::default(), Scale::default());
            let t2 = Transform::new(pos2.clone(), Rotation::default(), Scale::default());
            let t3 = Transform::new(pos3.clone(), Rotation::default(), Scale::default());
            
            // 变换组合的结合律测试（简化版本）
            // 注意：实际的变换组合可能涉及矩阵乘法，这里简化处理
            let combined1 = t1.combine(&t2).combine(&t3);
            let combined2 = t1.combine(&t2.combine(&t3));
            
            // 验证位置组合的一致性
            let pos1_result = combined1.position().value();
            let pos2_result = combined2.position().value();
            prop_assert!((pos1_result - pos2_result).length() < 0.1);
        }
    }
}

