use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::any::TypeId;

/// 组件验证错误
#[derive(Debug, Clone, PartialEq)]
pub enum ComponentValidationError {
    /// 组件冲突错误
    ComponentConflict {
        component_a: String,
        component_b: String,
        reason: String,
    },
    /// 组件不兼容错误
    ComponentIncompatible {
        component: String,
        reason: String,
    },
    /// 必需组件缺失
    RequiredComponentMissing {
        component: String,
        reason: String,
    },
}

impl std::fmt::Display for ComponentValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentValidationError::ComponentConflict { component_a, component_b, reason } => {
                write!(f, "组件冲突: {} 和 {} 不能同时存在 - {}", component_a, component_b, reason)
            }
            ComponentValidationError::ComponentIncompatible { component, reason } => {
                write!(f, "组件不兼容: {} - {}", component, reason)
            }
            ComponentValidationError::RequiredComponentMissing { component, reason } => {
                write!(f, "必需组件缺失: {} - {}", component, reason)
            }
        }
    }
}

impl std::error::Error for ComponentValidationError {}

/// 组件冲突规则
#[derive(Debug, Clone)]
pub struct ConflictRule {
    /// 冲突的组件类型ID
    pub conflicting_types: Vec<TypeId>,
    /// 冲突原因描述
    pub reason: String,
}

/// 组件兼容性规则
#[derive(Debug, Clone)]
pub struct CompatibilityRule {
    /// 必需的组件类型ID
    pub required_types: Vec<TypeId>,
    /// 不兼容的组件类型ID
    pub incompatible_types: Vec<TypeId>,
    /// 规则描述
    pub description: String,
}

/// 组件验证器
#[derive(Debug, Resource)]
pub struct ComponentValidator {
    /// 组件冲突规则映射
    conflict_rules: HashMap<TypeId, ConflictRule>,
    /// 组件兼容性规则
    compatibility_rules: Vec<CompatibilityRule>,
}

impl Default for ComponentValidator {
    fn default() -> Self {
        let mut validator = Self {
            conflict_rules: HashMap::new(),
            compatibility_rules: Vec::new(),
        };

        // 初始化默认冲突规则
        validator.initialize_default_rules();
        validator
    }
}

impl ComponentValidator {
    /// 创建新的组件验证器
    pub fn new() -> Self {
        Self::default()
    }

    /// 初始化默认的组件冲突和兼容性规则
    fn initialize_default_rules(&mut self) {
        use crate::ecs::{Camera, Sprite, Mesh, Transform};

        // Sprite 和 Camera 冲突：2D精灵和3D相机不应同时存在
        self.add_conflict_rule(
            TypeId::of::<Sprite>(),
            vec![TypeId::of::<Camera>()],
            "2D精灵组件和3D相机组件不能同时存在于同一实体",
        );

        // Mesh 和 Sprite 冲突：3D网格和2D精灵不应同时存在
        self.add_conflict_rule(
            TypeId::of::<Mesh>(),
            vec![TypeId::of::<Sprite>()],
            "3D网格组件和2D精灵组件不能同时存在于同一实体",
        );

        // Camera 需要 Transform 组件
        self.add_compatibility_rule(
            vec![TypeId::of::<Transform>()],
            vec![],
            "相机组件需要变换组件来定义位置和方向",
        );

        // Sprite 需要 Transform 组件
        self.add_compatibility_rule(
            vec![TypeId::of::<Transform>()],
            vec![],
            "精灵组件需要变换组件来定义位置",
        );

        // Mesh 需要 Transform 组件
        self.add_compatibility_rule(
            vec![TypeId::of::<Transform>()],
            vec![],
            "网格组件需要变换组件来定义位置和方向",
        );
    }

    /// 添加组件冲突规则
    pub fn add_conflict_rule(&mut self, component_type: TypeId, conflicting_types: Vec<TypeId>, reason: &str) {
        self.conflict_rules.insert(
            component_type,
            ConflictRule {
                conflicting_types,
                reason: reason.to_string(),
            },
        );
    }

    /// 添加组件兼容性规则
    pub fn add_compatibility_rule(&mut self, required_types: Vec<TypeId>, incompatible_types: Vec<TypeId>, description: &str) {
        self.compatibility_rules.push(CompatibilityRule {
            required_types,
            incompatible_types,
            description: description.to_string(),
        });
    }

    /// 验证实体的组件组合
    pub fn validate_entity(&self, entity: Entity, world: &World) -> Result<(), Vec<ComponentValidationError>> {
        let mut errors = Vec::new();

        // 获取实体上的所有组件类型
        let entity_ref = world.get_entity(entity).ok_or_else(|| {
            vec![ComponentValidationError::ComponentIncompatible {
                component: "Entity".to_string(),
                reason: "实体不存在".to_string(),
            }]
        })?;

        let component_types: Vec<TypeId> = entity_ref
            .archetype()
            .components()
            .map(|component_id| world.components().get_info(component_id).unwrap().type_id().unwrap())
            .collect();

        // 检查组件冲突
        for &component_type in &component_types {
            if let Some(conflict_rule) = self.conflict_rules.get(&component_type) {
                for &conflicting_type in &conflict_rule.conflicting_types {
                    if component_types.contains(&conflicting_type) {
                        let component_a_name = self.get_component_name(component_type);
                        let component_b_name = self.get_component_name(conflicting_type);
                        errors.push(ComponentValidationError::ComponentConflict {
                            component_a: component_a_name,
                            component_b: component_b_name,
                            reason: conflict_rule.reason.clone(),
                        });
                    }
                }
            }
        }

        // 检查组件兼容性
        for rule in &self.compatibility_rules {
            let has_required = rule.required_types.iter().all(|&req_type| component_types.contains(&req_type));
            let has_incompatible = rule.incompatible_types.iter().any(|&inc_type| component_types.contains(&inc_type));

            if !has_required {
                // 检查是否有任何必需组件缺失
                for &req_type in &rule.required_types {
                    if !component_types.contains(&req_type) {
                        let component_name = self.get_component_name(req_type);
                        errors.push(ComponentValidationError::RequiredComponentMissing {
                            component: component_name,
                            reason: rule.description.clone(),
                        });
                    }
                }
            }

            if has_incompatible {
                // 这里可以扩展为更复杂的兼容性检查
                // 目前只检查必需组件的存在性
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// 获取组件类型的名称（用于错误消息）
    fn get_component_name(&self, type_id: TypeId) -> String {
        use crate::ecs::{Camera, Sprite, Mesh, Transform, Velocity, PointLight, Material, PbrMaterialComp, PointLight3D, DirectionalLightComp};

        // 这里使用宏或更通用的方式来获取类型名称
        // 暂时硬编码主要组件类型
        if type_id == TypeId::of::<Transform>() {
            "Transform".to_string()
        } else if type_id == TypeId::of::<Sprite>() {
            "Sprite".to_string()
        } else if type_id == TypeId::of::<Camera>() {
            "Camera".to_string()
        } else if type_id == TypeId::of::<Mesh>() {
            "Mesh".to_string()
        } else if type_id == TypeId::of::<Velocity>() {
            "Velocity".to_string()
        } else if type_id == TypeId::of::<PointLight>() {
            "PointLight".to_string()
        } else if type_id == TypeId::of::<Material>() {
            "Material".to_string()
        } else if type_id == TypeId::of::<PbrMaterialComp>() {
            "PbrMaterialComp".to_string()
        } else if type_id == TypeId::of::<PointLight3D>() {
            "PointLight3D".to_string()
        } else if type_id == TypeId::of::<DirectionalLightComp>() {
            "DirectionalLightComp".to_string()
        } else {
            format!("Unknown({:?})", type_id)
        }
    }

    /// 验证组件插入操作
    pub fn validate_component_insertion(&self, entity: Entity, component_type: TypeId, world: &World) -> Result<(), Vec<ComponentValidationError>> {
        let mut errors = Vec::new();

        // 获取实体当前组件类型
        let entity_ref = world.get_entity(entity).ok_or_else(|| {
            vec![ComponentValidationError::ComponentIncompatible {
                component: "Entity".to_string(),
                reason: "实体不存在".to_string(),
            }]
        })?;

        let mut component_types: Vec<TypeId> = entity_ref
            .archetype()
            .components()
            .map(|component_id| world.components().get_info(component_id).unwrap().type_id().unwrap())
            .collect();

        // 添加要插入的组件类型
        component_types.push(component_type);

        // 检查冲突
        if let Some(conflict_rule) = self.conflict_rules.get(&component_type) {
            for &conflicting_type in &conflict_rule.conflicting_types {
                if component_types.contains(&conflicting_type) {
                    let component_a_name = self.get_component_name(component_type);
                    let component_b_name = self.get_component_name(conflicting_type);
                    errors.push(ComponentValidationError::ComponentConflict {
                        component_a: component_a_name,
                        component_b: component_b_name,
                        reason: conflict_rule.reason.clone(),
                    });
                }
            }
        }

        // 检查反向冲突（其他组件是否与要插入的组件冲突）
        for &existing_type in &component_types {
            if existing_type != component_type {
                if let Some(conflict_rule) = self.conflict_rules.get(&existing_type) {
                    if conflict_rule.conflicting_types.contains(&component_type) {
                        let component_a_name = self.get_component_name(existing_type);
                        let component_b_name = self.get_component_name(component_type);
                        errors.push(ComponentValidationError::ComponentConflict {
                            component_a: component_a_name,
                            component_b: component_b_name,
                            reason: conflict_rule.reason.clone(),
                        });
                    }
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// 组件验证系统
pub fn component_validation_system(
    validator: Res<ComponentValidator>,
    query: Query<Entity>,
    world: &World,
) {
    for entity in query.iter() {
        if let Err(errors) = validator.validate_entity(entity, world) {
            for error in errors {
                log::warn!("组件验证失败 - 实体 {:?}: {}", entity, error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ecs::{Camera, Sprite, Transform};

    #[test]
    fn test_sprite_camera_conflict() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((Transform::default(), Sprite::default())).id();

        // 验证当前状态应该是有效的
        assert!(validator.validate_entity(entity, &world).is_ok());

        // 尝试添加Camera组件，这应该会产生冲突
        let validation_result = validator.validate_component_insertion(
            entity,
            TypeId::of::<Camera>(),
            &world,
        );

        assert!(validation_result.is_err());
        let errors = validation_result.unwrap_err();
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
    fn test_valid_component_combination() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn((Transform::default(), Camera::default())).id();

        // Camera + Transform 应该是有效的
        assert!(validator.validate_entity(entity, &world).is_ok());
    }

    #[test]
    fn test_missing_required_component() {
        let mut world = World::new();
        let validator = ComponentValidator::new();

        let entity = world.spawn(Camera::default()).id();

        // Camera 没有 Transform 应该无效
        let result = validator.validate_entity(entity, &world);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ComponentValidationError::RequiredComponentMissing { .. })));
    }
}