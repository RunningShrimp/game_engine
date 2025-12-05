//! 游戏实体领域对象

use crate::domain::errors::{DomainError, SceneError};
use crate::ecs::{Camera, PointLight, Sprite, Transform};
use crate::impl_default;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 实体唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub u64);

impl EntityId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Entity({})", self.0)
    }
}

/// 游戏实体 - 聚合根
///
/// 封装实体的所有组件和行为，确保业务规则在边界内执行。
///
/// ## 聚合边界
///
/// **包含**：
/// - `EntityId`：实体唯一标识符
/// - `name`：实体名称（可选）
/// - `transform`：变换组件（可选）
/// - `sprite`：精灵渲染组件（可选）
/// - `point_light`：点光源组件（可选）
/// - `camera`：相机组件（可选）
/// - `properties`：自定义属性
/// - `state`：实体状态
///
/// **不包含**：
/// - 渲染管线（基础设施层）
/// - 物理引擎（基础设施层）
/// - ECS组件（基础设施层）
///
/// ## 业务规则
///
/// 1. 实体不能同时拥有`Sprite`和`Camera`组件
/// 2. `Transform`的缩放值必须为正数
/// 3. 待删除的实体不能激活
/// 4. 实体必须有ID
///
/// ## 不变性约束
///
/// - `EntityId`：创建后不可变
/// - `state`：只能通过聚合根方法修改（`activate`, `deactivate`, `mark_for_deletion`）
///
/// **注意**：虽然字段是`pub`的（用于序列化），但应该通过聚合根方法访问和修改。
#[derive(Debug, Clone)]
pub struct GameEntity {
    /// 实体ID
    pub id: EntityId,
    /// 实体名称
    pub name: Option<String>,
    /// 变换组件
    pub transform: Option<Transform>,
    /// 精灵渲染组件
    pub sprite: Option<Sprite>,
    /// 点光源组件
    pub point_light: Option<PointLight>,
    /// 相机组件
    pub camera: Option<Camera>,
    /// 自定义属性
    pub properties: HashMap<String, serde_json::Value>,
    /// 实体状态
    pub state: EntityState,
    /// 最后修改时间戳
    pub last_modified: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityState {
    /// 活跃状态
    Active,
    /// 非活跃状态
    Inactive,
    /// 待删除状态
    PendingDeletion,
}

impl GameEntity {
    /// 创建新实体
    pub fn new(id: EntityId) -> Self {
        Self {
            id,
            name: None,
            transform: None,
            sprite: None,
            point_light: None,
            camera: None,
            properties: HashMap::new(),
            state: EntityState::Active,
            last_modified: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// 设置实体名称
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 设置变换组件
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.transform = Some(transform);
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 设置精灵组件
    pub fn with_sprite(mut self, sprite: Sprite) -> Self {
        self.sprite = Some(sprite);
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 设置点光源组件
    pub fn with_point_light(mut self, light: PointLight) -> Self {
        self.point_light = Some(light);
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 设置相机组件
    pub fn with_camera(mut self, camera: Camera) -> Self {
        self.camera = Some(camera);
        self.last_modified = Self::current_timestamp();
        self
    }

    /// 设置自定义属性
    pub fn set_property(
        &mut self,
        key: impl Into<String>,
        value: serde_json::Value,
    ) -> Result<(), DomainError> {
        self.properties.insert(key.into(), value);
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 获取自定义属性
    pub fn get_property(&self, key: &str) -> Option<&serde_json::Value> {
        self.properties.get(key)
    }

    /// 激活实体
    pub fn activate(&mut self) -> Result<(), DomainError> {
        if self.state == EntityState::PendingDeletion {
            return Err(DomainError::Scene(SceneError::EntityNotFound(format!(
                "Cannot activate entity marked for deletion: {}",
                self.id
            ))));
        }
        self.state = EntityState::Active;
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 停用实体
    pub fn deactivate(&mut self) -> Result<(), DomainError> {
        self.state = EntityState::Inactive;
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 标记为待删除
    pub fn mark_for_deletion(&mut self) -> Result<(), DomainError> {
        self.state = EntityState::PendingDeletion;
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 检查实体是否活跃
    pub fn is_active(&self) -> bool {
        self.state == EntityState::Active
    }

    /// 获取实体位置（如果有变换组件）
    pub fn position(&self) -> Option<glam::Vec3> {
        self.transform.as_ref().map(|t| t.pos)
    }

    /// 设置实体位置
    pub fn set_position(&mut self, position: glam::Vec3) -> Result<(), DomainError> {
        if let Some(transform) = &mut self.transform {
            transform.pos = position;
            self.last_modified = Self::current_timestamp();
            Ok(())
        } else {
            Err(DomainError::Scene(SceneError::ComponentNotFound(
                "Transform component not found".to_string(),
            )))
        }
    }

    /// 移动实体
    pub fn move_by(&mut self, delta: glam::Vec3) -> Result<(), DomainError> {
        if let Some(transform) = &mut self.transform {
            transform.pos += delta;
            self.last_modified = Self::current_timestamp();
            Ok(())
        } else {
            Err(DomainError::Scene(SceneError::ComponentNotFound(
                "Transform component not found".to_string(),
            )))
        }
    }

    /// 旋转实体
    pub fn rotate(&mut self, rotation: glam::Quat) -> Result<(), DomainError> {
        if let Some(transform) = &mut self.transform {
            transform.rot = rotation;
            self.last_modified = Self::current_timestamp();
            Ok(())
        } else {
            Err(DomainError::Scene(SceneError::ComponentNotFound(
                "Transform component not found".to_string(),
            )))
        }
    }

    /// 缩放实体
    pub fn scale(&mut self, scale: glam::Vec3) -> Result<(), DomainError> {
        if let Some(transform) = &mut self.transform {
            transform.scale = scale;
            self.last_modified = Self::current_timestamp();
            Ok(())
        } else {
            Err(DomainError::Scene(SceneError::ComponentNotFound(
                "Transform component not found".to_string(),
            )))
        }
    }

    /// 验证实体状态
    pub fn validate(&self) -> Result<(), DomainError> {
        // 检查必需的组件组合
        if self.sprite.is_some() && self.camera.is_some() {
            return Err(DomainError::Scene(SceneError::ComponentNotFound(
                "Entity cannot have both Sprite and Camera components".to_string(),
            )));
        }

        // 检查变换组件的一致性
        if let Some(transform) = &self.transform {
            if transform.scale.x <= 0.0 || transform.scale.y <= 0.0 || transform.scale.z <= 0.0 {
                return Err(DomainError::Scene(SceneError::ComponentNotFound(
                    "Transform scale must be positive".to_string(),
                )));
            }
        }

        Ok(())
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

impl Default for GameEntity {
    fn default() -> Self {
        Self {
            id: EntityId(0),
            name: None,
            transform: None,
            sprite: None,
            point_light: None,
            camera: None,
            properties: HashMap::new(),
            state: EntityState::Active,
            last_modified: Self::current_timestamp(),
        }
    }
}

/// 实体工厂
pub struct EntityFactory;

impl EntityFactory {
    /// 创建基础实体
    pub fn create_basic(id: EntityId, position: glam::Vec3) -> GameEntity {
        GameEntity::new(id).with_transform(Transform {
            pos: position,
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        })
    }

    /// 创建精灵实体
    pub fn create_sprite(id: EntityId, position: glam::Vec3, sprite: Sprite) -> GameEntity {
        Self::create_basic(id, position).with_sprite(sprite)
    }

    /// 创建光源实体
    pub fn create_light(id: EntityId, position: glam::Vec3, light: PointLight) -> GameEntity {
        Self::create_basic(id, position).with_point_light(light)
    }

    /// 创建相机实体
    pub fn create_camera(id: EntityId, position: glam::Vec3, camera: Camera) -> GameEntity {
        Self::create_basic(id, position).with_camera(camera)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let entity = GameEntity::new(EntityId(1));
        assert_eq!(entity.id, EntityId(1));
        assert!(entity.is_active());
    }

    #[test]
    fn test_entity_with_components() {
        let transform = Transform {
            pos: glam::Vec3::new(1.0, 2.0, 3.0),
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        };

        let entity = GameEntity::new(EntityId(1))
            .with_name("Test Entity")
            .with_transform(transform.clone());

        assert_eq!(entity.name, Some("Test Entity".to_string()));
        assert_eq!(entity.transform, Some(transform));
    }

    #[test]
    fn test_entity_position_manipulation() {
        let mut entity = EntityFactory::create_basic(EntityId(1), glam::Vec3::ZERO);

        // 设置位置
        entity.set_position(glam::Vec3::new(1.0, 2.0, 3.0)).unwrap();
        assert_eq!(entity.position(), Some(glam::Vec3::new(1.0, 2.0, 3.0)));

        // 移动
        entity.move_by(glam::Vec3::new(0.0, 1.0, 0.0)).unwrap();
        assert_eq!(entity.position(), Some(glam::Vec3::new(1.0, 3.0, 3.0)));
    }

    #[test]
    fn test_entity_state_management() {
        let mut entity = GameEntity::new(EntityId(1));

        // 激活
        entity.activate().unwrap();
        assert!(entity.is_active());

        // 停用
        entity.deactivate().unwrap();
        assert!(!entity.is_active());

        // 标记删除
        entity.mark_for_deletion().unwrap();
        assert_eq!(entity.state, EntityState::PendingDeletion);
    }

    #[test]
    fn test_entity_validate_sprite_and_camera_conflict() {
        // 测试业务规则：实体不能同时拥有Sprite和Camera组件
        let mut entity = EntityFactory::create_sprite(
            EntityId(1),
            glam::Vec3::ZERO,
            Sprite::default(),
        );
        
        // 添加相机组件应该导致验证失败
        entity.camera = Some(Camera::default());
        
        assert!(entity.validate().is_err());
    }

    #[test]
    fn test_entity_validate_positive_scale() {
        // 测试业务规则：Transform的缩放值必须为正数
        let mut entity = EntityFactory::create_basic(EntityId(1), glam::Vec3::ZERO);
        
        // 设置负缩放值
        entity.scale(glam::Vec3::new(-1.0, 1.0, 1.0)).unwrap();
        assert!(entity.validate().is_err());
        
        // 设置零缩放值
        entity.scale(glam::Vec3::new(0.0, 1.0, 1.0)).unwrap();
        assert!(entity.validate().is_err());
        
        // 设置正缩放值
        entity.scale(glam::Vec3::ONE).unwrap();
        assert!(entity.validate().is_ok());
    }

    #[test]
    fn test_entity_activate_pending_deletion() {
        // 测试业务规则：待删除的实体不能激活
        let mut entity = GameEntity::new(EntityId(1));
        entity.mark_for_deletion().unwrap();
        
        // 尝试激活应该失败
        assert!(entity.activate().is_err());
    }

    #[test]
    fn test_entity_properties() {
        let mut entity = GameEntity::new(EntityId(1));
        
        // 设置属性
        entity.set_property("health", serde_json::json!(100)).unwrap();
        entity.set_property("name", serde_json::json!("Player")).unwrap();
        
        // 获取属性
        assert_eq!(entity.get_property("health"), Some(&serde_json::json!(100)));
        assert_eq!(entity.get_property("name"), Some(&serde_json::json!("Player")));
        assert_eq!(entity.get_property("nonexistent"), None);
    }

    #[test]
    fn test_entity_rotation() {
        let mut entity = EntityFactory::create_basic(EntityId(1), glam::Vec3::ZERO);
        
        let rotation = glam::Quat::from_euler(glam::EulerRot::XYZ, 0.0, 1.0, 0.0);
        entity.rotate(rotation).unwrap();
        
        assert_eq!(entity.transform.as_ref().unwrap().rot, rotation);
    }

    #[test]
    fn test_entity_operations_without_transform() {
        // 测试在没有Transform组件时操作应该失败
        let mut entity = GameEntity::new(EntityId(1));
        
        assert!(entity.set_position(glam::Vec3::ONE).is_err());
        assert!(entity.move_by(glam::Vec3::ONE).is_err());
        assert!(entity.rotate(glam::Quat::IDENTITY).is_err());
        assert!(entity.scale(glam::Vec3::ONE).is_err());
    }

    #[test]
    fn test_entity_id_creation() {
        let id = EntityId::new(42);
        assert_eq!(id.as_u64(), 42);
        assert_eq!(format!("{}", id), "Entity(42)");
    }

    #[test]
    fn test_entity_factory_create_basic() {
        let entity = EntityFactory::create_basic(EntityId(1), glam::Vec3::new(1.0, 2.0, 3.0));
        
        assert_eq!(entity.id, EntityId(1));
        assert_eq!(entity.position(), Some(glam::Vec3::new(1.0, 2.0, 3.0)));
        assert!(entity.transform.is_some());
    }

    #[test]
    fn test_entity_factory_create_sprite() {
        let sprite = Sprite::default();
        let entity = EntityFactory::create_sprite(EntityId(1), glam::Vec3::ZERO, sprite);
        
        assert!(entity.sprite.is_some());
        assert!(entity.validate().is_ok());
    }

    #[test]
    fn test_entity_factory_create_light() {
        let light = PointLight::default();
        let entity = EntityFactory::create_light(EntityId(1), glam::Vec3::ZERO, light);
        
        assert!(entity.point_light.is_some());
        assert!(entity.validate().is_ok());
    }

    #[test]
    fn test_entity_factory_create_camera() {
        let camera = Camera::default();
        let entity = EntityFactory::create_camera(EntityId(1), glam::Vec3::ZERO, camera);
        
        assert!(entity.camera.is_some());
        assert!(entity.validate().is_ok());
    }

    #[test]
    fn test_entity_validate_valid_entity() {
        // 测试有效的实体应该通过验证
        let entity = EntityFactory::create_basic(EntityId(1), glam::Vec3::ZERO);
        assert!(entity.validate().is_ok());
        
        let sprite_entity = EntityFactory::create_sprite(
            EntityId(2),
            glam::Vec3::ZERO,
            Sprite::default(),
        );
        assert!(sprite_entity.validate().is_ok());
    }

    #[test]
    fn test_entity_scale_without_transform() {
        // 测试在没有Transform组件时缩放应该失败
        let mut entity = GameEntity::new(EntityId(1));
        assert!(entity.scale(glam::Vec3::ONE).is_err());
    }

    #[test]
    fn test_entity_rotate_without_transform() {
        // 测试在没有Transform组件时旋转应该失败
        let mut entity = GameEntity::new(EntityId(1));
        assert!(entity.rotate(glam::Quat::IDENTITY).is_err());
    }

    #[test]
    fn test_entity_with_name() {
        let entity = GameEntity::new(EntityId(1))
            .with_name("Test Entity");
        assert_eq!(entity.name, Some("Test Entity".to_string()));
    }

    #[test]
    fn test_entity_with_transform() {
        let transform = Transform {
            pos: glam::Vec3::new(1.0, 2.0, 3.0),
            rot: glam::Quat::IDENTITY,
            scale: glam::Vec3::ONE,
        };
        let entity = GameEntity::new(EntityId(1))
            .with_transform(transform.clone());
        assert_eq!(entity.transform, Some(transform));
    }

    #[test]
    fn test_entity_position_without_transform() {
        // 测试在没有Transform组件时获取位置应该返回None
        let entity = GameEntity::new(EntityId(1));
        assert!(entity.position().is_none());
    }

    #[test]
    fn test_entity_position_with_transform() {
        let entity = EntityFactory::create_basic(
            EntityId(1),
            glam::Vec3::new(1.0, 2.0, 3.0),
        );
        assert_eq!(entity.position(), Some(glam::Vec3::new(1.0, 2.0, 3.0)));
    }
}
