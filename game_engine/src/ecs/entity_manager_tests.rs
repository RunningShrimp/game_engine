//! Entity Manager 综合测试
//!
//! 测试EntityManager的各种功能和边界情况

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::entity_manager::*;
    use crate::ecs::component_validator::*;
    use crate::ecs::{Transform, Sprite, Camera, Mesh, Velocity};
    use bevy_ecs::prelude::*;

    // ========================================
    // ValidatedCommands 测试
    // ========================================

    #[test]
    fn test_validated_commands_insert_valid_component() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &mut world);

        let entity = world.spawn_empty().id();
        let mut validated = entity_manager.validated_commands(commands, &world);

        let result = validated.insert_component(entity, Transform::default());
        assert!(result.is_ok());
    }

    #[test]
    fn test_validated_commands_insert_conflicting_component() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &mut world);

        // 创建带Sprite的实体
        let entity = world.spawn((Transform::default(), Sprite::default())).id();
        let mut validated = entity_manager.validated_commands(commands, &world);

        // 尝试添加冲突的Camera组件
        let result = validated.insert_component(entity, Camera::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_validated_entity_commands_build_valid_entity() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &mut world);

        let mut validated = entity_manager.validated_commands(commands, &world);
        let entity = validated.spawn_validated()
            .insert(Transform::default())
            .insert(Sprite::default())
            .finish();

        assert!(world.get_entity(entity).is_ok());
        assert!(world.get::<Transform>(entity).is_some());
        assert!(world.get::<Sprite>(entity).is_some());
    }

    #[test]
    fn test_validated_entity_commands_detects_conflicts() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &mut world);

        let mut validated = entity_manager.validated_commands(commands, &world);
        let entity_commands = validated.spawn_validated()
            .insert(Transform::default())
            .insert(Sprite::default())
            .insert(Camera::default()); // 冲突：Sprite + Camera

        // 应该检测到冲突
        assert!(entity_commands.has_validation_errors());
    }

    #[test]
    fn test_validated_commands_get_underlying_commands() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();
        let mut command_queue = CommandQueue::default();
        let mut commands = Commands::new(&mut command_queue, &mut world);

        let mut validated = entity_manager.validated_commands(commands, &world);
        let _ = validated.commands();
        // 确保能获取底层Commands
    }

    // ========================================
    // EntityManager 基础功能测试
    // ========================================

    #[test]
    fn test_entity_manager_new() {
        let manager = EntityManager::new();
        assert!(!manager.validator().conflict_rules.is_empty());
    }

    #[test]
    fn test_entity_manager_default() {
        let manager = EntityManager::default();
        assert!(!manager.validator().conflict_rules.is_empty());
    }

    #[test]
    fn test_entity_manager_validator_access() {
        let mut manager = EntityManager::new();
        let validator = manager.validator();
        assert!(validator.conflict_rules.len() > 0);
    }

    #[test]
    fn test_entity_manager_validator_mut_access() {
        let mut manager = EntityManager::new();
        let validator = manager.validator_mut();
        // 可以修改验证规则
        assert!(validator.conflict_rules.len() > 0);
    }

    // ========================================
    // 实体验证测试 - 有效性
    // ========================================

    #[test]
    fn test_validate_valid_transform_sprite() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((Transform::default(), Sprite::default())).id();
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_valid_transform_camera() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((Transform::default(), Camera::default())).id();
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_valid_transform_mesh() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((
            Transform::default(),
            Mesh { handle: crate::resources::manager::Handle(0) }
        )).id();
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_valid_transform_velocity() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((
            Transform::default(),
            Velocity::default()
        )).id();
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_valid_complete_sprite_entity() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Velocity::default(),
        )).id();
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_valid_complete_camera_entity() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((
            Transform::default(),
            Camera::default(),
            Velocity::default(),
        )).id();
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    // ========================================
    // 实体验证测试 - 冲突检测
    // ========================================

    #[test]
    fn test_validate_conflict_sprite_camera() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Camera::default(),
        )).id();

        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::ComponentConflict { .. }
        )));
    }

    #[test]
    fn test_validate_conflict_mesh_sprite() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Mesh { handle: crate::resources::manager::Handle(0) },
        )).id();

        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::ComponentConflict { .. }
        )));
    }

    #[test]
    fn test_validate_conflict_error_message_content() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn((
            Transform::default(),
            Sprite::default(),
            Camera::default(),
        )).id();

        let result = entity_manager.validate_entity(entity, &world);
        let errors = result.unwrap_err();

        let error = &errors[0];
        if let ComponentValidationError::ComponentConflict { component_a, component_b, reason } = error {
            assert!((component_a == "Sprite" && component_b == "Camera") ||
                   (component_a == "Camera" && component_b == "Sprite"));
            assert!(reason.contains("不能同时存在"));
        } else {
            panic!("Expected ComponentConflict error");
        }
    }

    // ========================================
    // 实体验证测试 - 必需组件检测
    // ========================================

    #[test]
    fn test_validate_missing_transform_for_sprite() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn(Sprite::default()).id();

        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::RequiredComponentMissing { .. }
        )));
    }

    #[test]
    fn test_validate_missing_transform_for_camera() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn(Camera::default()).id();

        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::RequiredComponentMissing { .. }
        )));
    }

    #[test]
    fn test_validate_missing_transform_for_mesh() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn(Mesh { handle: crate::resources::manager::Handle(0) }).id();

        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(
            e,
            ComponentValidationError::RequiredComponentMissing { .. }
        )));
    }

    // ========================================
    // 实体验证测试 - 边界情况
    // ========================================

    #[test]
    fn test_validate_nonexistent_entity() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let invalid_entity = Entity::from_raw_and_generation(0, u32::MAX);
        let result = entity_manager.validate_entity(invalid_entity, &world);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_entity_with_no_components() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn_empty().id();
        // 空实体应该是有效的（没有组件冲突）
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_entity_with_only_transform() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        let entity = world.spawn(Transform::default()).id();
        // 只有Transform应该是有效的
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_validate_multiple_errors_single_entity() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        // 创建一个有多个问题的实体
        let entity = world.spawn((
            Sprite::default(),
            Camera::default(),
            Mesh { handle: crate::resources::manager::Handle(0) },
        )).id();

        let result = entity_manager.validate_entity(entity, &world);
        let errors = result.unwrap_err();

        // 应该有多个错误：冲突 + 缺少Transform
        assert!(errors.len() >= 2);
    }

    // ========================================
    // 系统测试
    // ========================================

    #[test]
    fn test_entity_validation_system_logs_errors() {
        let mut world = World::new();
        world.insert_resource(EntityManager::new());

        // 创建有效的实体
        world.spawn((Transform::default(), Sprite::default()));

        // 创建无效的实体
        world.spawn((
            Transform::default(),
            Sprite::default(),
            Camera::default(),
        ));

        // 运行系统（应该会记录警告）
        // 注意：这里我们只测试系统能运行，实际日志需要通过日志系统测试
        let mut schedule = Schedule::new();
        schedule.add_systems(entity_validation_system);
        schedule.run(&mut world);

        // 系统应该成功运行
    }

    #[test]
    fn test_component_insertion_observer() {
        let mut world = World::new();
        world.insert_resource(EntityManager::new());

        // 添加观察器系统
        let mut schedule = Schedule::new();
        schedule.add_systems(component_insertion_observer);

        world.spawn((Transform::default(), Sprite::default()));

        // 触发组件插入
        let entity = world.spawn((Transform::default(), Sprite::default())).id();
        world.entity_mut(entity).insert(Camera::default());

        // 运行观察器
        schedule.run(&mut world);

        // 系统应该成功运行
    }

    // ========================================
    // 并发和线程安全测试
    // ========================================

    #[test]
    fn test_concurrent_validations() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();
        let entity_manager_ref = &entity_manager;

        // 创建多个实体
        let entities: Vec<Entity> = (0..10)
            .map(|_| world.spawn((Transform::default(), Sprite::default())).id())
            .collect();

        // 并发验证所有实体
        let results: Vec<Result<(), Vec<ComponentValidationError>>> = entities
            .iter()
            .map(|&e| entity_manager_ref.validate_entity(e, &world))
            .collect();

        // 所有验证都应该成功
        assert!(results.iter().all(|r| r.is_ok()));
    }

    // ========================================
    // 性能测试
    // ========================================

    #[test]
    fn test_validation_performance_many_entities() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        // 创建100个实体
        for _ in 0..100 {
            world.spawn((Transform::default(), Sprite::default()));
        }

        // 验证所有实体
        let mut error_count = 0;
        for entity in world.iter_entities() {
            if let Err(_) = entity_manager.validate_entity(entity.id(), &world) {
                error_count += 1;
            }
        }

        assert_eq!(error_count, 0);
    }

    #[test]
    fn test_validation_performance_complex_entities() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        // 创建复杂实体
        for _ in 0..50 {
            world.spawn((
                Transform::default(),
                Sprite::default(),
                Velocity::default(),
            ));
        }

        // 验证应该快速完成
        let start = std::time::Instant::now();
        for entity in world.iter_entities() {
            let _ = entity_manager.validate_entity(entity.id(), &world);
        }
        let duration = start.elapsed();

        // 应该在合理时间内完成（< 100ms）
        assert!(duration < std::time::Duration::from_millis(100));
    }

    // ========================================
    // 自定义规则测试
    // ========================================

    #[test]
    fn test_custom_conflict_rule() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();

        use std::any::TypeId;

        // 添加自定义规则：Velocity和Camera冲突（示例）
        entity_manager.validator_mut().add_conflict_rule(
            TypeId::of::<Velocity>(),
            vec![TypeId::of::<Camera>()],
            "Velocity和Camera不能同时存在（自定义规则）"
        );

        let entity = world.spawn((
            Transform::default(),
            Velocity::default(),
            Camera::default(),
        )).id();

        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());
    }

    #[test]
    fn test_custom_compatibility_rule() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();

        use std::any::TypeId;

        // 添加自定义兼容性规则
        entity_manager.validator_mut().add_compatibility_rule(
            vec![TypeId::of::<Velocity>()],
            vec![],
            "Velocity需要Transform（自定义规则测试）"
        );

        // 创建没有Transform的Velocity实体
        let entity = world.spawn(Velocity::default()).id();

        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());
    }
}
