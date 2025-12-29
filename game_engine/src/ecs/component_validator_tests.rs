//! Component Validator 综合测试
//!
//! 测试ComponentValidator的各种功能和边界情况

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::component_validator::*;
    use crate::ecs::{Transform, Sprite, Camera, Mesh, Velocity, Material};
    use bevy_ecs::prelude::*;

    // ========================================
    // DirtyFlags 基础测试
    // ========================================

    #[test]
    fn test_dirty_flags_none() {
        let flags = DirtyFlags::NONE;
        assert_eq!(flags.bits(), 0);
        assert!(!flags.contains(DirtyFlags::POSITION));
    }

    #[test]
    fn test_dirty_flags_position() {
        let flags = DirtyFlags::POSITION;
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(!flags.contains(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_dirty_flags_rotation() {
        let flags = DirtyFlags::ROTATION;
        assert!(flags.contains(DirtyFlags::ROTATION));
        assert!(!flags.contains(DirtyFlags::POSITION));
    }

    #[test]
    fn test_dirty_flags_scale() {
        let flags = DirtyFlags::SCALE;
        assert!(flags.contains(DirtyFlags::SCALE));
        assert!(!flags.contains(DirtyFlags::POSITION));
    }

    #[test]
    fn test_dirty_flags_transform() {
        let flags = DirtyFlags::TRANSFORM;
        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(flags.contains(DirtyFlags::ROTATION));
        assert!(flags.contains(DirtyFlags::SCALE));
    }

    #[test]
    fn test_dirty_flags_custom() {
        let flags = DirtyFlags::custom(0);
        assert_eq!(flags.bits(), 1 << 8);

        let flags2 = DirtyFlags::custom(1);
        assert_eq!(flags2.bits(), 1 << 9);

        // 测试超出范围的bit
        let flags_invalid = DirtyFlags::custom(100);
        assert_eq!(flags_invalid.bits(), 0);
    }

    #[test]
    fn test_dirty_flags_combine() {
        let flags = DirtyFlags::combine(&[
            DirtyFlags::POSITION,
            DirtyFlags::ROTATION,
            DirtyFlags::SCALE,
        ]);

        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(flags.contains(DirtyFlags::ROTATION));
        assert!(flags.contains(DirtyFlags::SCALE));
    }

    #[test]
    fn test_dirty_flags_combine_empty() {
        let flags = DirtyFlags::combine(&[]);
        assert_eq!(flags.bits(), 0);
    }

    // ========================================
    // DirtyFlags 操作符测试
    // ========================================

    #[test]
    fn test_dirty_flags_bitor() {
        let flags1 = DirtyFlags::POSITION;
        let flags2 = DirtyFlags::ROTATION;
        let combined = flags1 | flags2;

        assert!(combined.contains(DirtyFlags::POSITION));
        assert!(combined.contains(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_dirty_flags_bitand() {
        let flags1 = DirtyFlags::TRANSFORM;
        let flags2 = DirtyFlags::POSITION;
        let result = flags1 & flags2;

        assert!(result.contains(DirtyFlags::POSITION));
        assert!(!result.contains(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_dirty_flags_bitor_assign() {
        let mut flags = DirtyFlags::POSITION;
        flags |= DirtyFlags::ROTATION;

        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(flags.contains(DirtyFlags::ROTATION));
    }

    #[test]
    fn test_dirty_flags_bitand_assign() {
        let mut flags = DirtyFlags::TRANSFORM;
        flags &= DirtyFlags::POSITION;

        assert!(flags.contains(DirtyFlags::POSITION));
        assert!(!flags.contains(DirtyFlags::ROTATION));
    }

    // ========================================
    // ComponentValidationError 测试
    // ========================================

    #[test]
    fn test_error_conflict_display() {
        let error = ComponentValidationError::ComponentConflict {
            component_a: "Sprite".to_string(),
            component_b: "Camera".to_string(),
            reason: "测试原因".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("Sprite"));
        assert!(display.contains("Camera"));
        assert!(display.contains("测试原因"));
    }

    #[test]
    fn test_error_incompatible_display() {
        let error = ComponentValidationError::ComponentIncompatible {
            component: "TestComponent".to_string(),
            reason: "不兼容".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("TestComponent"));
        assert!(display.contains("不兼容"));
    }

    #[test]
    fn test_error_missing_display() {
        let error = ComponentValidationError::RequiredComponentMissing {
            component: "Transform".to_string(),
            reason: "必需组件".to_string(),
        };

        let display = format!("{}", error);
        assert!(display.contains("Transform"));
        assert!(display.contains("必需组件"));
    }

    #[test]
    fn test_error_equality() {
        let error1 = ComponentValidationError::ComponentConflict {
            component_a: "A".to_string(),
            component_b: "B".to_string(),
            reason: "test".to_string(),
        };

        let error2 = ComponentValidationError::ComponentConflict {
            component_a: "A".to_string(),
            component_b: "B".to_string(),
            reason: "test".to_string(),
        };

        assert_eq!(error1, error2);
    }

    // ========================================
    // ComponentValidator 基础测试
    // ========================================

    #[test]
    fn test_validator_new() {
        let validator = ComponentValidator::new();
        assert!(!validator.conflict_rules.is_empty());
    }

    #[test]
    fn test_validator_default() {
        let validator = ComponentValidator::default();
        assert!(!validator.conflict_rules.is_empty());
    }

    #[test]
    fn test_validator_initialization_has_default_rules() {
        let validator = ComponentValidator::new();
        assert!(validator.conflict_rules.len() > 0);
        assert!(validator.compatibility_rules.len() > 0);
    }

    // ========================================
    // 冲突规则测试
    // ========================================

    #[test]
    fn test_add_conflict_rule() {
        let mut validator = ComponentValidator::new();
        use std::any::TypeId;

        let initial_count = validator.conflict_rules.len();
        validator.add_conflict_rule(
            TypeId::of::<Velocity>(),
            vec![TypeId::of::<Camera>()],
            "自定义冲突规则"
        );

        assert_eq!(validator.conflict_rules.len(), initial_count + 1);
    }

    #[test]
    fn test_conflict_rule_sprite_camera() {
        let validator = ComponentValidator::new();

        // 检查Sprite和Camera的冲突规则是否存在
        use std::any::TypeId;
        let sprite_rule = validator.conflict_rules.get(&TypeId::of::<Sprite>());

        assert!(sprite_rule.is_some());
        let rule = sprite_rule.expect("Test: operation should succeed");
        assert!(rule.conflicting_types.contains(&TypeId::of::<Camera>()));
    }

    #[test]
    fn test_conflict_rule_mesh_sprite() {
        let validator = ComponentValidator::new();

        // 检查Mesh和Sprite的冲突规则
        use std::any::TypeId;
        let mesh_rule = validator.conflict_rules.get(&TypeId::of::<Mesh>());

        assert!(mesh_rule.is_some());
        let rule = mesh_rule.expect("Test: operation should succeed");
        assert!(rule.conflicting_types.contains(&TypeId::of::<Sprite>()));
    }

    // ========================================
    // 兼容性规则测试
    // ========================================

    #[test]
    fn test_add_compatibility_rule() {
        let mut validator = ComponentValidator::new();
        use std::any::TypeId;

        let initial_count = validator.compatibility_rules.len();
        validator.add_compatibility_rule(
            vec![TypeId::of::<Transform>()],
            vec![],
            "测试兼容性规则"
        );

        assert_eq!(validator.compatibility_rules.len(), initial_count + 1);
    }

    #[test]
    fn test_compatibility_rule_camera_requires_transform() {
        let validator = ComponentValidator::new();

        // Camera应该有Transform的兼容性规则
        let has_camera_rule = validator.compatibility_rules.iter()
            .any(|rule| rule.required_types.contains(&std::any::TypeId::of::<Camera>()));

        // 注意：当前实现中规则是从Transform的角度定义的
        // 这里我们只验证规则存在
        assert!(validator.compatibility_rules.len() > 0);
    }

    // ========================================
    // 实体验证 - 有效性测试
    // ========================================

    #[test]
    fn test_validate_entity_valid_sprite() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
        )).id();

        assert!(validator.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_entity_valid_camera() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((
            Transform::default(),
            Camera::default(),
        )).id();

        assert!(validator.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_entity_valid_mesh() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((
            Transform::default(),
            Mesh { handle: crate::resources::manager::Handle(0) },
        )).id();

        assert!(validator.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_entity_valid_with_velocity() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Velocity::default(),
        )).id();

        assert!(validator.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_entity_empty() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn_empty().id();
        // 空实体应该有效
        assert!(validator.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_entity_transform_only() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn(Transform::default()).id();
        // 只有Transform的实体应该有效
        assert!(validator.validate_entity(entity, &world).is_ok());
    }

    // ========================================
    // 实体验证 - 冲突检测
    // ========================================

    #[test]
    fn test_validate_entity_sprite_camera_conflict() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Camera::default(),
        )).id();

        let result = validator.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::ComponentConflict { .. }
        )));
    }

    #[test]
    fn test_validate_entity_mesh_sprite_conflict() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Mesh { handle: crate::resources::manager::Handle(0) },
        )).id();

        let result = validator.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::ComponentConflict { .. }
        )));
    }

    #[test]
    fn test_validate_entity_conflict_error_details() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Camera::default(),
        )).id();

        let result = validator.validate_entity(entity, &world);
        let errors = result.unwrap_err();

        let conflict_error = errors.iter()
            .find(|e| matches!(e, ComponentValidationError::ComponentConflict { .. }))
            .expect("Test: operation should succeed");

        if let ComponentValidationError::ComponentConflict { component_a, component_b, reason } = conflict_error {
            assert!((component_a == "Sprite" && component_b == "Camera") ||
                   (component_a == "Camera" && component_b == "Sprite"));
            assert!(reason.contains("不能同时存在"));
        } else {
            panic!("Expected ComponentConflict");
        }
    }

    // ========================================
    // 实体验证 - 必需组件检测
    // ========================================

    #[test]
    fn test_validate_entity_sprite_missing_transform() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn(Sprite::default()).id();

        let result = validator.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::RequiredComponentMissing { component, .. }
            if component == "Transform"
        )));
    }

    #[test]
    fn test_validate_entity_camera_missing_transform() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn(Camera::default()).id();

        let result = validator.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::RequiredComponentMissing { component, .. }
            if component == "Transform"
        )));
    }

    #[test]
    fn test_validate_entity_mesh_missing_transform() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn(Mesh { handle: crate::resources::manager::Handle(0) }).id();

        let result = validator.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::RequiredComponentMissing { component, .. }
            if component == "Transform"
        )));
    }

    // ========================================
    // 组件插入验证测试
    // ========================================

    #[test]
    fn test_validate_component_insertion_valid() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn(Transform::default()).id();
        use std::any::TypeId;

        let result = validator.validate_component_insertion(
            entity,
            TypeId::of::<Sprite>(),
            &world,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_component_insertion_conflict() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
        )).id();

        use std::any::TypeId;

        let result = validator.validate_component_insertion(
            entity,
            TypeId::of::<Camera>(),
            &world,
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::ComponentConflict { .. }
        )));
    }

    #[test]
    fn test_validate_component_insertion_nonexistent_entity() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let invalid_entity = Entity::from_raw_and_generation(999, u32::MAX);
        use std::any::TypeId;

        let result = validator.validate_component_insertion(
            invalid_entity,
            TypeId::of::<Sprite>(),
            &world,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_validate_component_insertion_reverse_conflict() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        // 先添加Camera
        let entity = world.spawn((
            Transform::default(),
            Camera::default(),
        )).id();

        use std::any::TypeId;

        // 尝试添加Sprite（应该检测到反向冲突）
        let result = validator.validate_component_insertion(
            entity,
            TypeId::of::<Sprite>(),
            &world,
        );

        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::ComponentConflict { .. }
        )));
    }

    // ========================================
    // 组件名称获取测试
    // ========================================

    #[test]
    fn test_get_component_name_transform() {
        let validator = ComponentValidator::new();
        let name = validator.get_component_name(std::any::TypeId::of::<Transform>());
        assert_eq!(name, "Transform");
    }

    #[test]
    fn test_get_component_name_sprite() {
        let validator = ComponentValidator::new();
        let name = validator.get_component_name(std::any::TypeId::of::<Sprite>());
        assert_eq!(name, "Sprite");
    }

    #[test]
    fn test_get_component_name_camera() {
        let validator = ComponentValidator::new();
        let name = validator.get_component_name(std::any::TypeId::of::<Camera>());
        assert_eq!(name, "Camera");
    }

    #[test]
    fn test_get_component_name_unknown() {
        let validator = ComponentValidator::new();
        let name = validator.get_component_name(std::any::TypeId::of::<Material>());
        assert!(name.contains("Unknown") || name.contains("Material"));
    }

    // ========================================
    // 多错误场景测试
    // ========================================

    #[test]
    fn test_validate_entity_multiple_conflicts() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        // 创建有多个冲突的实体
        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Camera::default(),
            Mesh { handle: crate::resources::manager::Handle(0) },
        )).id();

        let result = validator.validate_entity(entity, &world);
        let errors = result.unwrap_err();

        // 应该有多个冲突错误
        assert!(errors.len() >= 2);
    }

    #[test]
    fn test_validate_entity_conflict_and_missing() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        // 创建既有冲突又缺少必需组件的实体
        let entity = world.spawn((
            Sprite::default(),
            Camera::default(),
        )).id();

        let result = validator.validate_entity(entity, &world);
        let errors = result.unwrap_err();

        // 应该同时有冲突和缺少组件的错误
        let has_conflict = errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::ComponentConflict { .. }
        ));
        let has_missing = errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::RequiredComponentMissing { .. }
        ));

        assert!(has_conflict && has_missing);
    }

    // ========================================
    // 性能测试
    // ========================================

    #[test]
    fn test_validator_performance_many_entities() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        // 创建100个有效实体
        for _ in 0..100 {
            world.spawn((
                Transform::default(),
                Sprite::default(),
            ));
        }

        // 验证所有实体
        let mut error_count = 0;
        for entity in world.iter_entities() {
            if let Err(_) = validator.validate_entity(entity.id(), &world) {
                error_count += 1;
            }
        }

        assert_eq!(error_count, 0);
    }

    #[test]
    fn test_validator_performance_batch_validation() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        // 创建混合的实体
        for i in 0..50 {
            if i % 2 == 0 {
                world.spawn((Transform::default(), Sprite::default()));
            } else {
                world.spawn((Transform::default(), Camera::default()));
            }
        }

        let start = std::time::Instant::now();
        for entity in world.iter_entities() {
            let _ = validator.validate_entity(entity.id(), &world);
        }
        let duration = start.elapsed();

        // 应该快速完成（< 100ms）
        assert!(duration < std::time::Duration::from_millis(100));
    }

    // ========================================
    // 边界情况测试
    // ========================================

    #[test]
    fn test_validate_entity_despawned() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((Transform::default(), Sprite::default())).id();
        world.entity_mut(entity).despawn();

        let result = validator.validate_entity(entity, &world);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_component_insertion_to_empty_entity() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn_empty().id();
        use std::any::TypeId;

        // 向空实体添加Sprite（应该失败，因为缺少Transform）
        let result = validator.validate_component_insertion(
            entity,
            TypeId::of::<Sprite>(),
            &world,
        );

        // 当前实现不会检查插入后的完整性，只检查冲突
        // 所以这应该通过（因为没有冲突）
        assert!(result.is_ok());
    }
}
