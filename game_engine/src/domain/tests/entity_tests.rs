//  实体测试模块
// 
//  提供对域实体的全面测试覆盖，包括身份相等性、生命周期方法和状态变化测试。

use crate::domain::entity::*;
use crate::domain::errors::{DomainError, SceneError};
use crate::ecs::{Camera, PointLight, Sprite, Transform};
use crate::error::safe_lock;
use glam::{Quat, Vec3};
use serde_json;

#[cfg(test)]
mod entity_id_tests {
    use super::*;

    #[test]
    fn test_entity_id_equality() {
        let id1 = EntityId::new(42);
        let id2 = EntityId::new(42);
        let id3 = EntityId::new(24);
        
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_entity_id_hash() {
        let id1 = EntityId::new(42);
        let id2 = EntityId::new(42);
        
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(id1);
        set.insert(id2);
        
        assert_eq!(set.len(), 1); // 相同的ID应该被视为同一个元素
    }

    #[test]
    fn test_entity_id_display() {
        let id = EntityId::new(123);
        assert_eq!(format!("{}", id), "Entity(123)");
    }

    #[test]
    fn test_entity_id_copy() {
        let id1 = EntityId::new(42);
        let id2 = id1;
        
        assert_eq!(id1, id2);
        assert_eq!(id1.as_u64(), 42);
        assert_eq!(id2.as_u64(), 42);
    }
}

#[cfg(test)]
mod entity_state_tests {
    use super::*;

    #[test]
    fn test_entity_state_transitions() {
        let mut entity = GameEntity::new(EntityId::new(1));
        
        // 初始状态应该是活跃的
        assert!(entity.is_active());
        assert_eq!(entity.state, EntityState::Active);
        
        // 停用实体
        assert!(entity.deactivate().is_ok());
        assert!(!entity.is_active());
        assert_eq!(entity.state, EntityState::Inactive);
        
        // 重新激活实体
        assert!(entity.activate().is_ok());
        assert!(entity.is_active());
        assert_eq!(entity.state, EntityState::Active);
        
        // 标记为待删除
        assert!(entity.mark_for_deletion().is_ok());
        assert_eq!(entity.state, EntityState::PendingDeletion);
        assert!(!entity.is_active());
    }

    #[test]
    fn test_entity_state_business_rules() {
        let mut entity = GameEntity::new(EntityId::new(1));
        
        // 标记为待删除
        assert!(entity.mark_for_deletion().is_ok());
        
        // 尝试激活待删除的实体应该失败
        assert!(entity.activate().is_err());
        assert_eq!(entity.state, EntityState::PendingDeletion);
        
        // 但停用应该成功
        assert!(entity.deactivate().is_ok());
        assert_eq!(entity.state, EntityState::PendingDeletion);
    }

    #[test]
    fn test_entity_state_copy() {
        let entity1 = GameEntity::new(EntityId::new(1));
        let entity2 = entity1.clone();
        
        assert_eq!(entity1.id, entity2.id);
        assert_eq!(entity1.state, entity2.state);
        assert_eq!(entity1.is_active(), entity2.is_active());
    }
}

#[cfg(test)]
mod entity_lifecycle_tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let id = EntityId::new(123);
        let entity = GameEntity::new(id);
        
        assert_eq!(entity.id, id);
        assert!(entity.is_active());
        assert_eq!(entity.state, EntityState::Active);
        assert!(entity.name.is_none());
        assert!(entity.transform.is_none());
        assert!(entity.sprite.is_none());
        assert!(entity.camera.is_none());
        assert!(entity.point_light.is_none());
        assert!(entity.properties.is_empty());
        assert!(entity.last_modified > 0);
    }

    #[test]
    fn test_entity_with_methods() {
        let id = EntityId::new(1);
        let name = "Test Entity";
        let transform = Transform {
            pos: Vec3::new(1.0, 2.0, 3.0),
            rot: Quat::IDENTITY,
            scale: Vec3::ONE,
        };
        let sprite = Sprite::default();
        
        let entity = GameEntity::new(id)
            .with_name(name)
            .with_transform(transform.clone())
            .with_sprite(sprite.clone());
        
        assert_eq!(entity.id, id);
        assert_eq!(entity.name, Some(name.to_string()));
        assert_eq!(entity.transform, Some(transform));
        assert_eq!(entity.sprite, Some(sprite));
        assert!(entity.camera.is_none());
        assert!(entity.point_light.is_none());
    }

    #[test]
    fn test_entity_timestamp_updates() {
        let mut entity = GameEntity::new(EntityId::new(1));
        let initial_timestamp = entity.last_modified;
        
        // 等待一小段时间确保时间戳不同
        std::thread::sleep(std::time::Duration::from_millis(1));
        
        entity.set_property("test", serde_json::json!(true)).expect("Test: operation should succeed");
        
        assert!(entity.last_modified > initial_timestamp);
    }

    #[test]
    fn test_entity_default() {
        let entity = GameEntity::default();
        
        assert_eq!(entity.id, EntityId(0));
        assert!(entity.is_active());
        assert!(entity.properties.is_empty());
    }
}

#[cfg(test)]
mod entity_component_tests {
    use super::*;

    #[test]
    fn test_entity_transform_operations() {
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        // 测试位置设置
        let new_pos = Vec3::new(5.0, 10.0, 15.0);
        assert!(entity.set_position(new_pos).is_ok());
        assert_eq!(entity.position(), Some(new_pos));
        
        // 测试移动
        let delta = Vec3::new(1.0, 2.0, 3.0);
        assert!(entity.move_by(delta).is_ok());
        assert_eq!(entity.position(), Some(new_pos + delta));
        
        // 测试旋转
        let rotation = Quat::from_euler(glam::EulerRot::XYZ, 0.0, 1.0, 0.0);
        assert!(entity.rotate(rotation).is_ok());
        assert_eq!(entity.transform.as_ref().expect("Test: operation should succeed").rot, rotation);
        
        // 测试缩放
        let scale = Vec3::new(2.0, 2.0, 2.0);
        assert!(entity.scale(scale).is_ok());
        assert_eq!(entity.transform.as_ref().expect("Test: operation should succeed").scale, scale);
    }

    #[test]
    fn test_entity_operations_without_transform() {
        let mut entity = GameEntity::new(EntityId::new(1));
        
        // 没有Transform组件时，这些操作应该失败
        assert!(entity.set_position(Vec3::ONE).is_err());
        assert!(entity.move_by(Vec3::ONE).is_err());
        assert!(entity.rotate(Quat::IDENTITY).is_err());
        assert!(entity.scale(Vec3::ONE).is_err());
        
        // 获取位置应该返回None
        assert!(entity.position().is_none());
    }

    #[test]
    fn test_entity_component_conflicts() {
        let mut entity = EntityFactory::create_sprite(
            EntityId::new(1),
            Vec3::ZERO,
            Sprite::default(),
        );
        
        // 添加相机组件应该导致验证失败
        entity.camera = Some(Camera::default());
        assert!(entity.validate().is_err());
        
        // 移除相机组件后应该验证通过
        entity.camera = None;
        assert!(entity.validate().is_ok());
    }

    #[test]
    fn test_entity_transform_validation() {
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        // 设置负缩放值应该导致验证失败
        entity.scale(Vec3::new(-1.0, 1.0, 1.0)).expect("Test: operation should succeed");
        assert!(entity.validate().is_err());
        
        // 设置零缩放值应该导致验证失败
        entity.scale(Vec3::new(0.0, 1.0, 1.0)).expect("Test: operation should succeed");
        assert!(entity.validate().is_err());
        
        // 设置正缩放值应该验证通过
        entity.scale(Vec3::new(1.0, 1.0, 1.0)).expect("Test: operation should succeed");
        assert!(entity.validate().is_ok());
    }
}

#[cfg(test)]
mod entity_properties_tests {
    use super::*;

    #[test]
    fn test_entity_properties_management() {
        let mut entity = GameEntity::new(EntityId::new(1));
        
        // 测试设置属性
        assert!(entity.set_property("health", serde_json::json!(100)).is_ok());
        assert!(entity.set_property("name", serde_json::json!("Player")).is_ok());
        assert!(entity.set_property("position", serde_json::json!({"x": 1.0, "y": 2.0})).is_ok());
        
        // 测试获取属性
        assert_eq!(entity.get_property("health"), Some(&serde_json::json!(100)));
        assert_eq!(entity.get_property("name"), Some(&serde_json::json!("Player")));
        assert_eq!(entity.get_property("position"), Some(&serde_json::json!({"x": 1.0, "y": 2.0})));
        assert_eq!(entity.get_property("nonexistent"), None);
        
        // 测试属性数量
        assert_eq!(entity.properties.len(), 3);
    }

    #[test]
    fn test_entity_properties_overwrite() {
        let mut entity = GameEntity::new(EntityId::new(1));
        
        // 设置初始值
        assert!(entity.set_property("health", serde_json::json!(100)).is_ok());
        assert_eq!(entity.get_property("health"), Some(&serde_json::json!(100)));
        
        // 覆盖值
        assert!(entity.set_property("health", serde_json::json!(50)).is_ok());
        assert_eq!(entity.get_property("health"), Some(&serde_json::json!(50)));
        
        // 属性数量应该保持不变
        assert_eq!(entity.properties.len(), 1);
    }

    #[test]
    fn test_entity_properties_complex_types() {
        let mut entity = GameEntity::new(EntityId::new(1));
        
        // 测试复杂类型
        let complex_data = serde_json::json!({
            "position": {"x": 1.0, "y": 2.0, "z": 3.0},
            "inventory": ["sword", "shield", "potion"],
            "stats": {"strength": 10, "agility": 15}
        });
        
        assert!(entity.set_property("complex", complex_data.clone()).is_ok());
        assert_eq!(entity.get_property("complex"), Some(&complex_data));
    }
}

#[cfg(test)]
mod entity_factory_tests {
    use super::*;

    #[test]
    fn test_entity_factory_basic() {
        let entity = EntityFactory::create_basic(EntityId::new(1), Vec3::new(1.0, 2.0, 3.0));
        
        assert_eq!(entity.id, EntityId::new(1));
        assert_eq!(entity.position(), Some(Vec3::new(1.0, 2.0, 3.0)));
        assert!(entity.transform.is_some());
        assert!(entity.sprite.is_none());
        assert!(entity.camera.is_none());
        assert!(entity.point_light.is_none());
        assert!(entity.validate().is_ok());
    }

    #[test]
    fn test_entity_factory_sprite() {
        let sprite = Sprite::default();
        let entity = EntityFactory::create_sprite(EntityId::new(1), Vec3::ZERO, sprite.clone());
        
        assert_eq!(entity.id, EntityId::new(1));
        assert_eq!(entity.position(), Some(Vec3::ZERO));
        assert_eq!(entity.sprite, Some(sprite));
        assert!(entity.camera.is_none());
        assert!(entity.validate().is_ok());
    }

    #[test]
    fn test_entity_factory_light() {
        let light = PointLight::default();
        let entity = EntityFactory::create_light(EntityId::new(1), Vec3::ZERO, light.clone());
        
        assert_eq!(entity.id, EntityId::new(1));
        assert_eq!(entity.position(), Some(Vec3::ZERO));
        assert_eq!(entity.point_light, Some(light));
        assert!(entity.sprite.is_none());
        assert!(entity.camera.is_none());
        assert!(entity.validate().is_ok());
    }

    #[test]
    fn test_entity_factory_camera() {
        let camera = Camera::default();
        let entity = EntityFactory::create_camera(EntityId::new(1), Vec3::ZERO, camera.clone());
        
        assert_eq!(entity.id, EntityId::new(1));
        assert_eq!(entity.position(), Some(Vec3::ZERO));
        assert_eq!(entity.camera, Some(camera));
        assert!(entity.sprite.is_none());
        assert!(entity.point_light.is_none());
        assert!(entity.validate().is_ok());
    }
}

#[cfg(test)]
mod entity_validation_tests {
    use super::*;

    #[test]
    fn test_entity_validation_valid_entities() {
        // 测试各种有效的实体配置
        let basic_entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        assert!(basic_entity.validate().is_ok());
        
        let sprite_entity = EntityFactory::create_sprite(
            EntityId::new(2),
            Vec3::ZERO,
            Sprite::default(),
        );
        assert!(sprite_entity.validate().is_ok());
        
        let light_entity = EntityFactory::create_light(
            EntityId::new(3),
            Vec3::ZERO,
            PointLight::default(),
        );
        assert!(light_entity.validate().is_ok());
        
        let camera_entity = EntityFactory::create_camera(
            EntityId::new(4),
            Vec3::ZERO,
            Camera::default(),
        );
        assert!(camera_entity.validate().is_ok());
    }

    #[test]
    fn test_entity_validation_sprite_camera_conflict() {
        let mut entity = EntityFactory::create_sprite(
            EntityId::new(1),
            Vec3::ZERO,
            Sprite::default(),
        );
        
        // 添加相机组件应该导致验证失败
        entity.camera = Some(Camera::default());
        let result = entity.validate();
        
        assert!(result.is_err());
        if let Err(DomainError::Scene(SceneError::ComponentNotFound(msg))) = result {
            assert!(msg.contains("Sprite and Camera"));
        } else {
            panic!("Expected SceneError::ComponentNotFound");
        }
    }

    #[test]
    fn test_entity_validation_negative_scale() {
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        // 测试各种负缩放情况
        let test_cases = vec![
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, -1.0),
        ];
        
        for scale in test_cases {
            entity.scale(scale).expect("Test: operation should succeed");
            assert!(entity.validate().is_err());
        }
    }

    #[test]
    fn test_entity_validation_zero_scale() {
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        // 测试各种零缩放情况
        let test_cases = vec![
            Vec3::new(0.0, 1.0, 1.0),
            Vec3::new(1.0, 0.0, 1.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 0.0),
        ];
        
        for scale in test_cases {
            entity.scale(scale).expect("Test: operation should succeed");
            assert!(entity.validate().is_err());
        }
    }

    #[test]
    fn test_entity_validation_positive_scale() {
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        // 测试正缩放值
        let test_cases = vec![
            Vec3::new(0.1, 0.1, 0.1),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(10.0, 10.0, 10.0),
            Vec3::new(100.0, 50.0, 25.0),
        ];
        
        for scale in test_cases {
            entity.scale(scale).expect("Test: operation should succeed");
            assert!(entity.validate().is_ok());
        }
    }
}

#[cfg(test)]
mod entity_serialization_tests {
    use super::*;

    #[test]
    fn test_entity_serialization() {
        let entity = EntityFactory::create_sprite(
            EntityId::new(123),
            Vec3::new(1.0, 2.0, 3.0),
            Sprite::default(),
        )
        .with_name("Test Entity");
        
        let serialized = serde_json::to_string(&entity).expect("Test: operation should succeed");
        let deserialized: GameEntity = serde_json::from_str(&serialized).expect("Test: operation should succeed");
        
        assert_eq!(entity.id, deserialized.id);
        assert_eq!(entity.name, deserialized.name);
        assert_eq!(entity.position(), deserialized.position());
        assert_eq!(entity.state, deserialized.state);
    }

    #[test]
    fn test_entity_id_serialization() {
        let id = EntityId::new(42);
        let serialized = serde_json::to_string(&id).expect("Test: operation should succeed");
        let deserialized: EntityId = serde_json::from_str(&serialized).expect("Test: operation should succeed");
        
        assert_eq!(id, deserialized);
        assert_eq!(id.as_u64(), deserialized.as_u64());
    }
}

#[cfg(test)]
mod entity_edge_cases_tests {
    use super::*;

    #[test]
    fn test_entity_extreme_positions() {
        let test_positions = vec![
            Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            Vec3::new(f32::MIN, f32::MIN, f32::MIN),
            Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            Vec3::new(f32::MIN, f32::MIN, f32::MIN),
        ];
        
        for pos in test_positions {
            let entity = EntityFactory::create_basic(EntityId::new(1), pos);
            assert_eq!(entity.position(), Some(pos));
            assert!(entity.validate().is_ok());
        }
    }

    #[test]
    fn test_entity_extreme_rotations() {
        let mut entity = EntityFactory::create_basic(EntityId::new(1), Vec3::ZERO);
        
        let test_rotations = vec![
            Quat::from_euler(glam::EulerRot::XYZ, f32::MAX, 0.0, 0.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, f32::MAX, 0.0),
            Quat::from_euler(glam::EulerRot::XYZ, 0.0, 0.0, f32::MAX),
            Quat::from_euler(glam::EulerRot::XYZ, f32::MIN, 0.0, 0.0),
        ];
        
        for rotation in test_rotations {
            assert!(entity.rotate(rotation).is_ok());
            assert!(entity.validate().is_ok());
        }
    }

    #[test]
    fn test_entity_large_properties() {
        let mut entity = GameEntity::new(EntityId::new(1));
        
        // 创建大量属性
        for i in 0..1000 {
            let key = format!("property_{}", i);
            let value = serde_json::json!({
                "index": i,
                "data": "some long string data that takes up memory",
                "nested": {
                    "level1": {
                        "level2": {
                            "value": i * 2
                        }
                    }
                }
            });
            
            assert!(entity.set_property(&key, value).is_ok());
        }
        
        assert_eq!(entity.properties.len(), 1000);
        
        // 验证一些属性
        assert_eq!(entity.get_property("property_0"), Some(&serde_json::json!({
            "index": 0,
            "data": "some long string data that takes up memory",
            "nested": {
                "level1": {
                    "level2": {
                        "value": 0
                    }
                }
            }
        })));
        
        assert_eq!(entity.get_property("property_999"), Some(&serde_json::json!({
            "index": 999,
            "data": "some long string data that takes up memory",
            "nested": {
                "level1": {
                    "level2": {
                        "value": 1998
                    }
                }
            }
        })));
    }

    #[test]
    fn test_entity_concurrent_operations() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let entity = Arc::new(Mutex::new(GameEntity::new(EntityId::new(1))));
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let entity_clone = Arc::clone(&entity);
                thread::spawn(move || {
                    let mut entity = safe_lock(&entity_clone, "entity_test_clone").expect("Test: operation should succeed");
                    let key = format!("thread_{}", i);
                    let value = serde_json::json!(i);
                    entity.set_property(&key, value).expect("Test: operation should succeed");
                })
            })
            .collect();
        
        for handle in handles {
            handle.join().expect("Test: operation should succeed");
        }
        
        let entity = safe_lock(&entity, "entity_test_final").expect("Test: operation should succeed");
        assert_eq!(entity.properties.len(), 10);
        
        for i in 0..10 {
            let key = format!("thread_{}", i);
            assert_eq!(entity.get_property(&key), Some(&serde_json::json!(i)));
        }
    }
}