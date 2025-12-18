use bevy_ecs::prelude::*;
use std::any::TypeId;
use crate::ecs::component_validator::{ComponentValidator, ComponentValidationError};

/// 验证模式的实体命令扩展
pub struct ValidatedCommands<'w, 's> {
    commands: Commands<'w, 's>,
    validator: &'w ComponentValidator,
    world: &'w World,
}

impl<'w, 's> ValidatedCommands<'w, 's> {
    /// 创建新的验证命令
    pub fn new(commands: Commands<'w, 's>, validator: &'w ComponentValidator, world: &'w World) -> Self {
        Self {
            commands,
            validator,
            world,
        }
    }

    /// 验证并插入组件到实体
    pub fn insert_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<(), Vec<ComponentValidationError>> {
        let component_type = TypeId::of::<T>();

        // 验证组件插入
        self.validator.validate_component_insertion(entity, component_type, self.world)?;

        // 如果验证通过，插入组件
        self.commands.entity(entity).insert(component);
        Ok(())
    }

    /// 验证并生成实体
    pub fn spawn_validated(&mut self) -> ValidatedEntityCommands {
        ValidatedEntityCommands {
            entity_commands: self.commands.spawn_empty(),
            validator: self.validator,
            world: self.world,
            has_validation_errors: false,
        }
    }

    /// 获取底层命令（用于不需验证的操作）
    pub fn commands(&mut self) -> &mut Commands<'w, 's> {
        &mut self.commands
    }
}

/// 验证模式的实体命令
pub struct ValidatedEntityCommands<'a> {
    entity_commands: bevy_ecs::system::EntityCommands<'a>,
    validator: &'a ComponentValidator,
    world: &'a World,
    has_validation_errors: bool,
}

impl<'a> ValidatedEntityCommands<'a> {
    /// 验证并插入组件
    pub fn insert<T: Component>(mut self, component: T) -> Self {
        let entity = self.entity_commands.id();
        let component_type = TypeId::of::<T>();

        // 验证组件插入
        if let Err(errors) = self.validator.validate_component_insertion(entity, component_type, self.world) {
            log::warn!("组件验证失败，跳过插入: {:?}", errors);
            self.has_validation_errors = true;
            // 不插入组件，但继续构建命令
        } else {
            self.entity_commands.insert(component);
        }

        self
    }

    /// 完成实体构建
    pub fn finish(self) -> Entity {
        self.entity_commands.id()
    }

    /// 检查是否有验证错误
    pub fn has_validation_errors(&self) -> bool {
        self.has_validation_errors
    }
}

/// 实体管理器 - 提供验证功能的实体管理接口
#[derive(Resource)]
pub struct EntityManager {
    validator: ComponentValidator,
}

impl Default for EntityManager {
    fn default() -> Self {
        Self {
            validator: ComponentValidator::new(),
        }
    }
}

impl EntityManager {
    /// 创建新的实体管理器
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建带验证的命令
    pub fn validated_commands<'w, 's>(&'w self, commands: Commands<'w, 's>, world: &'w World) -> ValidatedCommands<'w, 's> {
        ValidatedCommands::new(commands, &self.validator, world)
    }

    /// 验证现有实体
    pub fn validate_entity(&self, entity: Entity, world: &World) -> Result<(), Vec<ComponentValidationError>> {
        self.validator.validate_entity(entity, world)
    }

    /// 获取验证器引用
    pub fn validator(&self) -> &ComponentValidator {
        &self.validator
    }

    /// 获取可变验证器引用（用于配置规则）
    pub fn validator_mut(&mut self) -> &mut ComponentValidator {
        &mut self.validator
    }
}

/// 实体验证系统 - 定期验证所有实体
pub fn entity_validation_system(
    entity_manager: Res<EntityManager>,
    query: Query<Entity>,
    world: &World,
) {
    let mut error_count = 0;
    let max_errors_to_log = 10; // 避免日志泛滥

    for entity in query.iter() {
        if let Err(errors) = entity_manager.validate_entity(entity, world) {
            for error in errors {
                if error_count < max_errors_to_log {
                    log::warn!("实体验证失败 - 实体 {:?}: {}", entity, error);
                }
                error_count += 1;
            }
        }
    }

    if error_count > max_errors_to_log {
        log::warn!("... 还有 {} 个验证错误未显示", error_count - max_errors_to_log);
    }
}

/// 组件插入观察器 - 在组件插入时进行验证
pub fn component_insertion_observer(
    trigger: Trigger<OnAdd>,
    entity_manager: Res<EntityManager>,
    world: &World,
) {
    let entity = trigger.entity();

    if let Err(errors) = entity_manager.validate_entity(entity, world) {
        for error in errors {
            log::error!("组件插入验证失败 - 实体 {:?}: {}", entity, error);
            // 注意：在观察器中我们不能移除组件，因为这可能导致递归
            // 建议在系统级别处理验证失败
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{Transform, Sprite, Camera};

    #[test]
    fn test_component_validation_sprite_camera_conflict() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        // 创建实体并插入Transform和Sprite
        let entity = world.spawn((Transform::default(), Sprite::default())).id();

        // 验证当前状态应该是有效的
        assert!(entity_manager.validate_entity(entity, &world).is_ok());

        // 手动添加Camera组件（绕过验证）
        world.entity_mut(entity).insert(Camera::default());

        // 现在验证应该失败
        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            ComponentValidationError::ComponentConflict { component_a, component_b, .. } => {
                assert!((component_a == "Sprite" && component_b == "Camera") ||
                       (component_a == "Camera" && component_b == "Sprite"));
            }
            _ => panic!("Expected ComponentConflict error"),
        }
    }

    #[test]
    fn test_entity_validation_valid_combination() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        // 创建有效的实体组合
        let entity = world.spawn((Transform::default(), Sprite::default())).id();

        // 验证应该成功
        assert!(entity_manager.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_entity_manager_validation() {
        let mut world = World::new();
        let entity_manager = EntityManager::new();

        // 创建冲突的实体组合
        let entity = world.spawn((Transform::default(), Sprite::default(), Camera::default())).id();

        // 验证应该失败
        let result = entity_manager.validate_entity(entity, &world);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(!errors.is_empty());
        assert!(errors.iter().any(|e| matches!(e, ComponentValidationError::ComponentConflict { .. })));
    }
}