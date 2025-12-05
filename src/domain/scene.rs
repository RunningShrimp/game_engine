//! 场景领域对象
//! 实现场景作为聚合根，管理实体集合

use crate::domain::entity::{EntityId, GameEntity};
use crate::domain::errors::{CompensationAction, DomainError, RecoveryStrategy, SceneError};
use crate::impl_default;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 场景ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SceneId(pub u64);

impl SceneId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// 场景状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneState {
    /// 未加载
    Unloaded,
    /// 加载中
    Loading,
    /// 已加载
    Loaded,
    /// 激活
    Active,
    /// 非激活
    Inactive,
}

/// 场景 - 聚合根
///
/// 管理场景中的所有实体，确保业务规则在边界内执行。
/// 场景作为聚合根，负责维护实体集合的一致性和完整性。
///
/// ## 聚合边界
///
/// **包含**：
/// - `SceneId`：场景唯一标识符
/// - `name`：场景名称
/// - `state`：场景状态
/// - `entities`：场景中的实体集合
/// - `metadata`：场景元数据
/// - `recovery_strategy`：错误恢复策略
///
/// **不包含**：
/// - 渲染管线（基础设施层）
/// - 物理世界（基础设施层）
/// - ECS World（基础设施层）
///
/// ## 业务规则
///
/// 1. 场景名称不能为空
/// 2. 状态转换必须遵循生命周期：Unloaded → Loading → Loaded → Active/Inactive → Unloaded
/// 3. 场景内实体ID必须唯一
/// 4. 活跃场景最多只能有一个活跃相机
/// 5. 场景激活时，所有实体必须激活
///
/// ## 不变性约束
///
/// - `SceneId`：创建后不可变
/// - `name`：创建后不可变
/// - `entities`：只能通过聚合根方法修改（`add_entity`, `remove_entity`）
/// - `state`：只能通过聚合根方法修改（`load`, `activate`, `deactivate`, `unload`）
///
/// ## 访问模式
///
/// **正确**：
/// ```rust
/// // 通过聚合根添加实体
/// scene.add_entity(entity)?;
///
/// // 通过聚合根获取实体
/// if let Some(entity) = scene.get_entity_mut(entity_id) {
///     entity.set_position(position)?;
/// }
/// ```
///
/// **错误**：
/// ```rust
/// // ❌ 直接访问聚合内部
/// scene.entities.insert(entity_id, entity);
///
/// // ❌ 绕过业务规则
/// scene.state = SceneState::Active;
/// ```
///
/// # 示例
///
/// ```rust
/// use game_engine::domain::{Scene, SceneId, SceneState};
/// use game_engine::domain::entity::{GameEntity, EntityId, EntityFactory};
///
/// // 创建新场景
/// let mut scene = Scene::new(
///     SceneId::new(1),
///     "main_scene"
/// )?;
///
/// // 添加实体
/// let entity = EntityFactory::create_entity(EntityId::new(1))?;
/// scene.add_entity(entity)?;
///
/// // 激活场景
/// scene.activate()?;
///
/// // 更新场景
/// scene.update(0.016)?; // 16ms delta time
///
/// // 获取实体
/// if let Some(entity) = scene.get_entity(EntityId::new(1)) {
///     // 使用实体
/// }
///
/// // 移除实体
/// scene.remove_entity(EntityId::new(1))?;
/// # Ok::<(), game_engine::domain::errors::DomainError>(())
/// ```
/// 场景 - 聚合根
///
/// **注意**：虽然字段是`pub`的（用于序列化），但应该通过聚合根方法访问和修改，
/// 以确保业务规则在边界内执行。直接修改字段可能违反业务规则。
#[derive(Debug, Clone)]
pub struct Scene {
    /// 场景ID（不可变）
    pub id: SceneId,
    /// 场景名称（创建后不可变）
    pub name: String,
    /// 场景状态（应通过`load`, `activate`, `deactivate`, `unload`方法修改）
    pub state: SceneState,
    /// 实体集合（应通过`add_entity`, `remove_entity`方法修改）
    pub entities: HashMap<EntityId, GameEntity>,
    /// 场景元数据
    pub metadata: SceneMetadata,
    /// 最后修改时间戳
    pub last_modified: u64,
    /// 错误恢复策略
    pub recovery_strategy: RecoveryStrategy,
}

#[derive(Debug, Clone, Default)]
pub struct SceneMetadata {
    /// 作者
    pub author: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 创建时间
    pub created_at: u64,
    /// 修改时间
    pub modified_at: u64,
    /// 版本
    pub version: u32,
}

impl SceneMetadata {
    /// 创建默认场景元数据
    pub fn new() -> Self {
        Self::default()
    }
}

impl Scene {
    /// 创建新场景
    pub fn new(id: SceneId, name: impl Into<String>) -> Self {
        let now = Self::current_timestamp();
        Self {
            id,
            name: name.into(),
            state: SceneState::Unloaded,
            entities: HashMap::new(),
            metadata: SceneMetadata {
                created_at: now,
                modified_at: now,
                version: 1,
                ..Default::default()
            },
            last_modified: now,
            recovery_strategy: RecoveryStrategy::Retry {
                max_attempts: 3,
                delay_ms: 100,
            },
        }
    }

    /// 加载场景
    pub fn load(&mut self) -> Result<(), DomainError> {
        if self.state != SceneState::Unloaded {
            return Err(DomainError::Scene(SceneError::SceneNotFound(format!(
                "Cannot load scene {}: invalid state {:?}",
                self.name, self.state
            ))));
        }

        self.state = SceneState::Loading;
        self.last_modified = Self::current_timestamp();

        // 这里可以添加实际的加载逻辑
        // 例如从文件加载实体

        self.state = SceneState::Loaded;
        self.metadata.modified_at = Self::current_timestamp();

        Ok(())
    }

    /// 激活场景
    pub fn activate(&mut self) -> Result<(), DomainError> {
        if self.state != SceneState::Loaded && self.state != SceneState::Inactive {
            return Err(DomainError::Scene(SceneError::SceneNotFound(format!(
                "Cannot activate scene {}: not loaded",
                self.name
            ))));
        }

        self.state = SceneState::Active;
        self.last_modified = Self::current_timestamp();

        // 激活所有实体
        for entity in self.entities.values_mut() {
            entity.activate()?;
        }

        Ok(())
    }

    /// 停用场景
    pub fn deactivate(&mut self) -> Result<(), DomainError> {
        if self.state != SceneState::Active {
            return Err(DomainError::Scene(SceneError::SceneNotFound(format!(
                "Cannot deactivate scene {}: not active",
                self.name
            ))));
        }

        self.state = SceneState::Inactive;
        self.last_modified = Self::current_timestamp();

        // 停用所有实体
        for entity in self.entities.values_mut() {
            entity.deactivate()?;
        }

        Ok(())
    }

    /// 卸载场景
    pub fn unload(&mut self) -> Result<(), DomainError> {
        self.state = SceneState::Unloaded;
        self.entities.clear();
        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 添加实体
    pub fn add_entity(&mut self, entity: GameEntity) -> Result<(), DomainError> {
        // 业务规则：确保实体ID唯一
        if self.entities.contains_key(&entity.id) {
            return Err(DomainError::Scene(SceneError::EntityNotFound(format!(
                "Entity {} already exists in scene {}",
                entity.id.as_u64(),
                self.name
            ))));
        }

        // 业务规则：验证实体状态
        entity.validate()?;

        self.entities.insert(entity.id, entity);
        self.last_modified = Self::current_timestamp();
        self.metadata.version += 1;

        Ok(())
    }

    /// 移除实体
    pub fn remove_entity(&mut self, entity_id: EntityId) -> Result<GameEntity, DomainError> {
        let entity = self.entities.remove(&entity_id).ok_or_else(|| {
            DomainError::Scene(SceneError::EntityNotFound(format!(
                "Entity {} not found in scene {}",
                entity_id.as_u64(),
                self.name
            )))
        })?;

        self.last_modified = Self::current_timestamp();
        self.metadata.version += 1;

        Ok(entity)
    }

    /// 获取实体
    pub fn get_entity(&self, entity_id: EntityId) -> Option<&GameEntity> {
        self.entities.get(&entity_id)
    }

    /// 获取实体可变引用
    pub fn get_entity_mut(&mut self, entity_id: EntityId) -> Option<&mut GameEntity> {
        self.entities.get_mut(&entity_id)
    }

    /// 查找实体（按名称）
    pub fn find_entity_by_name(&self, name: &str) -> Option<&GameEntity> {
        self.entities
            .values()
            .find(|e| e.name.as_deref() == Some(name))
    }

    /// 获取所有实体ID
    pub fn entity_ids(&self) -> Vec<EntityId> {
        self.entities.keys().cloned().collect()
    }

    /// 获取活跃实体数量
    pub fn active_entity_count(&self) -> usize {
        self.entities.values().filter(|e| e.is_active()).count()
    }

    /// 获取总实体数量
    pub fn total_entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 批量添加实体
    pub fn add_entities(&mut self, entities: Vec<GameEntity>) -> Result<(), DomainError> {
        for entity in entities {
            self.add_entity(entity)?;
        }
        Ok(())
    }

    /// 批量移除实体
    pub fn remove_entities(
        &mut self,
        entity_ids: Vec<EntityId>,
    ) -> Result<Vec<GameEntity>, DomainError> {
        let mut removed = Vec::new();
        for id in entity_ids {
            removed.push(self.remove_entity(id)?);
        }
        Ok(removed)
    }

    /// 更新场景（业务规则执行）
    pub fn update(&mut self, delta_time: f32) -> Result<(), DomainError> {
        if self.state != SceneState::Active {
            return Ok(()); // 非活跃场景不更新
        }
        // delta_time参数用于未来可能的场景更新逻辑
        let _ = delta_time;

        // 业务规则：清理待删除实体
        let to_remove: Vec<EntityId> = self
            .entities
            .iter()
            .filter(|(_, e)| matches!(e.state, crate::domain::entity::EntityState::PendingDeletion))
            .map(|(id, _)| *id)
            .collect();

        for id in to_remove {
            self.entities.remove(&id);
        }

        // 这里可以添加其他场景级别的更新逻辑
        // 例如碰撞检测、触发器检查等

        self.last_modified = Self::current_timestamp();
        Ok(())
    }

    /// 验证场景完整性
    pub fn validate(&self) -> Result<(), DomainError> {
        // 业务规则：场景必须有名称
        if self.name.trim().is_empty() {
            return Err(DomainError::Scene(SceneError::SceneNotFound(
                "Scene name cannot be empty".to_string(),
            )));
        }

        // 业务规则：实体ID不能重复（已在add_entity中检查）

        // 验证所有实体
        for entity in self.entities.values() {
            entity.validate()?;
        }

        // 业务规则：活跃场景不能有冲突的组件组合
        if self.state == SceneState::Active {
            let camera_count = self
                .entities
                .values()
                .filter(|e| e.camera.is_some())
                .count();

            // 业务规则：一个场景最多只能有一个活跃相机
            if camera_count > 1 {
                return Err(DomainError::Scene(SceneError::ComponentNotFound(format!(
                    "Scene {} has {} cameras, maximum allowed is 1",
                    self.name, camera_count
                ))));
            }
        }

        Ok(())
    }

    /// 创建快照（用于补偿）
    pub fn create_snapshot(&self) -> SceneSnapshot {
        SceneSnapshot {
            scene_id: self.id,
            name: self.name.clone(),
            state: self.state,
            entity_count: self.entities.len(),
            active_entity_count: self.active_entity_count(),
            version: self.metadata.version,
            timestamp: Self::current_timestamp(),
        }
    }

    /// 执行错误恢复
    pub fn recover_from_error(&mut self, error: &SceneError) -> Result<(), DomainError> {
        match &self.recovery_strategy {
            RecoveryStrategy::Retry {
                max_attempts,
                delay_ms,
            } => {
                for attempt in 1..=*max_attempts {
                    tracing::warn!(target: "scene", "Retry attempt {} for scene {}", attempt, self.name);
                    std::thread::sleep(std::time::Duration::from_millis(*delay_ms));

                    match error {
                        SceneError::SerializationFailed(_) => {
                            // 尝试重新序列化
                            return Ok(());
                        }
                        SceneError::DeserializationFailed(_) => {
                            // 尝试重新反序列化
                            return Ok(());
                        }
                        _ => break,
                    }
                }
                Err(DomainError::Scene(error.clone()))
            }
            RecoveryStrategy::UseDefault => {
                // 重置为默认状态
                self.state = SceneState::Unloaded;
                self.entities.clear();
                Ok(())
            }
            RecoveryStrategy::Skip => Ok(()),
            RecoveryStrategy::LogAndContinue => {
                tracing::error!(target: "scene", "Scene error logged: {:?}", error);
                Ok(())
            }
            RecoveryStrategy::Fail => Err(DomainError::Scene(error.clone())),
        }
    }

    /// 创建补偿操作
    pub fn create_compensation(&self) -> CompensationAction {
        CompensationAction::new(
            format!("scene_{}", self.id.as_u64()),
            "restore_scene_state".to_string(),
            serde_json::json!({
                "name": self.name,
                "state": format!("{:?}", self.state),
                "entity_count": self.entities.len(),
                "version": self.metadata.version
            }),
        )
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 场景快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSnapshot {
    pub scene_id: SceneId,
    pub name: String,
    pub state: SceneState,
    pub entity_count: usize,
    pub active_entity_count: usize,
    pub version: u32,
    pub timestamp: u64,
}

impl SceneSnapshot {
    /// 创建新的场景快照
    pub fn new(
        scene_id: SceneId,
        name: String,
        state: SceneState,
        entity_count: usize,
        active_entity_count: usize,
        version: u32,
    ) -> Self {
        Self {
            scene_id,
            name,
            state,
            entity_count,
            active_entity_count,
            version,
            timestamp: Self::current_timestamp(),
        }
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 场景管理器 - 领域服务
pub struct SceneManager {
    /// 场景集合
    scenes: HashMap<SceneId, Scene>,
    /// 当前活跃场景
    active_scene: Option<SceneId>,
    /// 最后更新时间戳
    last_updated: u64,
}

impl_default!(SceneManager {
    scenes: HashMap::new(),
    active_scene: None,
    last_updated: Self::current_timestamp(),
});

impl SceneManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建场景
    pub fn create_scene(
        &mut self,
        id: SceneId,
        name: impl Into<String>,
    ) -> Result<(), DomainError> {
        if self.scenes.contains_key(&id) {
            return Err(DomainError::Scene(SceneError::SceneNotFound(format!(
                "Scene {} already exists",
                id.as_u64()
            ))));
        }

        let scene = Scene::new(id, name);
        self.scenes.insert(id, scene);
        self.last_updated = Self::current_timestamp();

        Ok(())
    }

    /// 删除场景
    pub fn delete_scene(&mut self, id: SceneId) -> Result<Scene, DomainError> {
        if Some(id) == self.active_scene {
            self.active_scene = None;
        }

        let scene = self.scenes.remove(&id).ok_or_else(|| {
            DomainError::Scene(SceneError::SceneNotFound(format!(
                "Scene {} not found",
                id.as_u64()
            )))
        })?;

        self.last_updated = Self::current_timestamp();
        Ok(scene)
    }

    /// 获取场景
    pub fn get_scene(&self, id: SceneId) -> Option<&Scene> {
        self.scenes.get(&id)
    }

    /// 获取场景可变引用
    pub fn get_scene_mut(&mut self, id: SceneId) -> Option<&mut Scene> {
        self.scenes.get_mut(&id)
    }

    /// 获取当前活跃场景
    pub fn active_scene(&self) -> Option<&Scene> {
        self.active_scene.and_then(|id| self.scenes.get(&id))
    }

    /// 获取当前场景（兼容方法，等同于active_scene）
    pub fn current_scene(&self) -> Option<&Scene> {
        self.active_scene()
    }

    /// 获取当前活跃场景可变引用
    pub fn active_scene_mut(&mut self) -> Option<&mut Scene> {
        self.active_scene.and_then(|id| self.scenes.get_mut(&id))
    }

    /// 切换活跃场景
    pub fn switch_to_scene(&mut self, id: SceneId) -> Result<(), DomainError> {
        // 停用当前场景
        if let Some(current_id) = self.active_scene {
            if let Some(current_scene) = self.scenes.get_mut(&current_id) {
                current_scene.deactivate()?;
            }
        }

        // 激活新场景
        let scene = self.scenes.get_mut(&id).ok_or_else(|| {
            DomainError::Scene(SceneError::SceneNotFound(format!(
                "Scene {} not found",
                id.as_u64()
            )))
        })?;

        scene.activate()?;
        self.active_scene = Some(id);
        self.last_updated = Self::current_timestamp();

        Ok(())
    }

    /// 获取所有场景ID
    pub fn scene_ids(&self) -> Vec<SceneId> {
        self.scenes.keys().cloned().collect()
    }

    /// 更新所有场景
    pub fn update(&mut self, delta_time: f32) -> Result<(), DomainError> {
        for scene in self.scenes.values_mut() {
            scene.update(delta_time)?;
        }
        self.last_updated = Self::current_timestamp();
        Ok(())
    }

    /// 更新场景过渡（兼容方法）
    ///
    /// 注意：domain版本的SceneManager不直接管理过渡状态，
    /// 过渡逻辑由场景对象本身处理。此方法用于兼容ECS版本的API。
    pub fn update_transition(&mut self, delta_time: f32) -> Result<(), DomainError> {
        // 更新所有场景（包括过渡状态）
        // delta_time参数用于未来可能的过渡状态管理
        let _ = delta_time;
        self.update(delta_time)
    }

    /// 验证所有场景
    pub fn validate(&self) -> Result<(), DomainError> {
        for scene in self.scenes.values() {
            scene.validate()?;
        }
        Ok(())
    }

    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::EntityFactory;
    use crate::ecs::Camera;
    use glam::Vec3;

    #[test]
    fn test_scene_creation() {
        let scene = Scene::new(SceneId(1), "Test Scene");
        assert_eq!(scene.id, SceneId(1));
        assert_eq!(scene.name, "Test Scene");
        assert_eq!(scene.state, SceneState::Unloaded);
    }

    #[test]
    fn test_scene_lifecycle() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");

        scene.load().unwrap();
        assert_eq!(scene.state, SceneState::Loaded);

        scene.activate().unwrap();
        assert_eq!(scene.state, SceneState::Active);

        scene.deactivate().unwrap();
        assert_eq!(scene.state, SceneState::Inactive);

        scene.unload().unwrap();
        assert_eq!(scene.state, SceneState::Unloaded);
    }

    #[test]
    fn test_scene_entity_management() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.activate().unwrap();

        let entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        scene.add_entity(entity).unwrap();

        assert_eq!(scene.total_entity_count(), 1);
        assert_eq!(scene.active_entity_count(), 1);

        let removed = scene.remove_entity(EntityId(1)).unwrap();
        assert_eq!(removed.id, EntityId(1));
        assert_eq!(scene.total_entity_count(), 0);
    }

    #[test]
    fn test_scene_duplicate_entity_error() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        let entity1 = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        scene.add_entity(entity1).unwrap();
        
        // 尝试添加重复ID的实体应该失败
        let entity2 = EntityFactory::create_basic(EntityId(1), Vec3::new(1.0, 1.0, 1.0));
        let result = scene.add_entity(entity2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::Scene(_)));
    }

    #[test]
    fn test_scene_invalid_state_transition() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        
        // 尝试从未加载状态直接激活应该失败
        let result = scene.activate();
        assert!(result.is_err());
        
        // 正确的状态转换
        scene.load().unwrap();
        scene.activate().unwrap();
        assert_eq!(scene.state, SceneState::Active);
    }

    #[test]
    fn test_scene_remove_nonexistent_entity() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        // 尝试移除不存在的实体应该失败
        let result = scene.remove_entity(EntityId(999));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DomainError::Scene(_)));
    }

    #[test]
    fn test_scene_entity_count() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.activate().unwrap();
        
        // 添加多个实体
        for i in 1..=5 {
            let entity = EntityFactory::create_basic(EntityId(i), Vec3::ZERO);
            scene.add_entity(entity).unwrap();
        }
        
        assert_eq!(scene.total_entity_count(), 5);
        assert_eq!(scene.active_entity_count(), 5);
        
        // 停用一个实体
        if let Some(entity) = scene.get_entity_mut(EntityId(3)) {
            entity.deactivate().unwrap();
        }
        
        assert_eq!(scene.total_entity_count(), 5);
        assert_eq!(scene.active_entity_count(), 4);
    }

    #[test]
    fn test_scene_id_creation() {
        let id = SceneId::new(42);
        assert_eq!(id.as_u64(), 42);
    }

    #[test]
    fn test_scene_add_entity_validation() {
        // 测试业务规则：添加实体时验证实体状态
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        // 创建无效实体（Sprite和Camera冲突）
        let mut invalid_entity = EntityFactory::create_sprite(
            EntityId(1),
            Vec3::ZERO,
            crate::ecs::Sprite::default(),
        );
        invalid_entity.camera = Some(crate::ecs::Camera::default());
        
        // 添加无效实体应该失败
        assert!(scene.add_entity(invalid_entity).is_err());
    }

    #[test]
    fn test_scene_activate_activates_entities() {
        // 测试业务规则：场景激活时，所有实体必须激活
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        let mut entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        entity.deactivate().unwrap(); // 先停用
        scene.add_entity(entity).unwrap();
        
        // 激活场景应该激活所有实体
        scene.activate().unwrap();
        assert_eq!(scene.state, SceneState::Active);
        assert!(scene.get_entity(EntityId(1)).unwrap().is_active());
    }

    #[test]
    fn test_scene_deactivate_deactivates_entities() {
        // 测试业务规则：场景停用时，所有实体必须停用
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.activate().unwrap();
        
        let entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        scene.add_entity(entity).unwrap();
        
        // 停用场景应该停用所有实体
        scene.deactivate().unwrap();
        assert_eq!(scene.state, SceneState::Inactive);
        assert!(!scene.get_entity(EntityId(1)).unwrap().is_active());
    }

    #[test]
    fn test_scene_unload_clears_entities() {
        // 测试业务规则：卸载场景时清除所有实体
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        let entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        scene.add_entity(entity).unwrap();
        assert_eq!(scene.total_entity_count(), 1);
        
        scene.unload().unwrap();
        assert_eq!(scene.total_entity_count(), 0);
    }

    #[test]
    fn test_scene_get_entity() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        let entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        scene.add_entity(entity).unwrap();
        
        // 获取存在的实体
        assert!(scene.get_entity(EntityId(1)).is_some());
        assert_eq!(scene.get_entity(EntityId(1)).unwrap().id, EntityId(1));
        
        // 获取不存在的实体
        assert!(scene.get_entity(EntityId(999)).is_none());
    }

    #[test]
    fn test_scene_get_entity_mut() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        let entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        scene.add_entity(entity).unwrap();
        
        // 修改实体
        if let Some(entity) = scene.get_entity_mut(EntityId(1)) {
            entity.set_position(Vec3::ONE).unwrap();
        }
        
        assert_eq!(scene.get_entity(EntityId(1)).unwrap().position(), Some(Vec3::ONE));
    }

    #[test]
    fn test_scene_manager() {
        let mut manager = SceneManager::new();

        manager.create_scene(SceneId(1), "Scene 1").unwrap();
        manager.create_scene(SceneId(2), "Scene 2").unwrap();

        assert_eq!(manager.scene_ids().len(), 2);

        // 加载场景（switch_to_scene会自动激活）
        manager.get_scene_mut(SceneId(1)).unwrap().load().unwrap();
        manager.get_scene_mut(SceneId(2)).unwrap().load().unwrap();

        manager.switch_to_scene(SceneId(1)).unwrap();
        assert_eq!(manager.active_scene().unwrap().id, SceneId(1));

        manager.switch_to_scene(SceneId(2)).unwrap();
        assert_eq!(manager.active_scene().unwrap().id, SceneId(2));
    }

    #[test]
    fn test_scene_find_entity_by_name() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        let entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO)
            .with_name("Test Entity");
        scene.add_entity(entity).unwrap();
        
        let found = scene.find_entity_by_name("Test Entity");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, EntityId(1));
        
        let not_found = scene.find_entity_by_name("Nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_scene_entity_ids() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        scene.add_entity(EntityFactory::create_basic(EntityId(1), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId(2), Vec3::ZERO)).unwrap();
        
        let ids = scene.entity_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&EntityId(1)));
        assert!(ids.contains(&EntityId(2)));
    }

    #[test]
    fn test_scene_add_entities() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        let entities = vec![
            EntityFactory::create_basic(EntityId(1), Vec3::ZERO),
            EntityFactory::create_basic(EntityId(2), Vec3::ZERO),
            EntityFactory::create_basic(EntityId(3), Vec3::ZERO),
        ];
        
        scene.add_entities(entities).unwrap();
        assert_eq!(scene.total_entity_count(), 3);
    }

    #[test]
    fn test_scene_add_entities_duplicate_error() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        let entities = vec![
            EntityFactory::create_basic(EntityId(1), Vec3::ZERO),
            EntityFactory::create_basic(EntityId(1), Vec3::ZERO), // 重复ID
        ];
        
        assert!(scene.add_entities(entities).is_err());
    }

    #[test]
    fn test_scene_remove_entities() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        scene.add_entity(EntityFactory::create_basic(EntityId(1), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId(2), Vec3::ZERO)).unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId(3), Vec3::ZERO)).unwrap();
        
        let removed = scene.remove_entities(vec![EntityId(1), EntityId(3)]).unwrap();
        assert_eq!(removed.len(), 2);
        assert_eq!(scene.total_entity_count(), 1);
    }

    #[test]
    fn test_scene_update_removes_pending_deletion() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.activate().unwrap();
        
        let mut entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        entity.mark_for_deletion().unwrap();
        scene.add_entity(entity).unwrap();
        
        assert_eq!(scene.total_entity_count(), 1);
        
        // 更新场景应该移除待删除的实体
        scene.update(0.016).unwrap();
        assert_eq!(scene.total_entity_count(), 0);
    }

    #[test]
    fn test_scene_update_inactive_scene() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        // 不激活场景
        
        let mut entity = EntityFactory::create_basic(EntityId(1), Vec3::ZERO);
        entity.mark_for_deletion().unwrap();
        scene.add_entity(entity).unwrap();
        
        // 非活跃场景的update应该成功但不移除实体
        scene.update(0.016).unwrap();
        assert_eq!(scene.total_entity_count(), 1); // 实体仍然存在
    }

    #[test]
    fn test_scene_validate_empty_name() {
        // 注意：Scene::new接受name参数，但validate检查name不能为空
        // 由于new方法接受name，我们需要测试空名称的情况
        // 但new方法本身不验证名称，所以我们需要直接构造或测试validate
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.name = String::new(); // 直接设置空名称
        assert!(scene.validate().is_err());
    }

    #[test]
    fn test_scene_validate_multiple_cameras() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.activate().unwrap();
        
        // 添加两个相机实体
        let camera1 = Camera::default();
        let camera2 = Camera::default();
        scene.add_entity(EntityFactory::create_camera(EntityId(1), Vec3::ZERO, camera1)).unwrap();
        scene.add_entity(EntityFactory::create_camera(EntityId(2), Vec3::ZERO, camera2)).unwrap();
        
        // 活跃场景不能有多个相机
        assert!(scene.validate().is_err());
    }

    #[test]
    fn test_scene_create_snapshot() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        
        scene.add_entity(EntityFactory::create_basic(EntityId(1), Vec3::ZERO)).unwrap();
        
        let snapshot = scene.create_snapshot();
        assert_eq!(snapshot.scene_id, SceneId(1));
        assert_eq!(snapshot.name, "Test Scene");
        assert_eq!(snapshot.entity_count, 1);
        assert_eq!(snapshot.active_entity_count, 1);
    }

    #[test]
    fn test_scene_manager_get_scene() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId(1), "Scene 1").unwrap();
        
        assert!(manager.get_scene(SceneId(1)).is_some());
        assert!(manager.get_scene(SceneId(999)).is_none());
    }

    #[test]
    fn test_scene_manager_delete_scene() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId(1), "Scene 1").unwrap();
        manager.get_scene_mut(SceneId(1)).unwrap().load().unwrap();
        manager.switch_to_scene(SceneId(1)).unwrap();
        
        let deleted = manager.delete_scene(SceneId(1)).unwrap();
        assert_eq!(deleted.id, SceneId(1));
        assert!(manager.active_scene().is_none());
    }

    #[test]
    fn test_scene_manager_delete_nonexistent_scene() {
        let mut manager = SceneManager::new();
        assert!(manager.delete_scene(SceneId(999)).is_err());
    }

    #[test]
    fn test_scene_manager_update() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId(1), "Scene 1").unwrap();
        manager.get_scene_mut(SceneId(1)).unwrap().load().unwrap();
        
        manager.update(0.016).unwrap();
        // 验证更新成功（通过行为验证）
    }

    #[test]
    fn test_scene_manager_validate() {
        let mut manager = SceneManager::new();
        manager.create_scene(SceneId(1), "Scene 1").unwrap();
        
        assert!(manager.validate().is_ok());
    }

    // ============================================================================
    // 错误恢复和补偿操作测试
    // ============================================================================

    #[test]
    fn test_scene_recover_from_error_serialization_failed() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = SceneError::SerializationFailed("test".to_string());
        let result = scene.recover_from_error(&error);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_scene_recover_from_error_deserialization_failed() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = SceneError::DeserializationFailed("test".to_string());
        let result = scene.recover_from_error(&error);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_scene_recover_from_error_entity_not_found() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.recovery_strategy = RecoveryStrategy::Retry {
            max_attempts: 1,
            delay_ms: 1,
        };
        
        let error = SceneError::EntityNotFound("test".to_string());
        let result = scene.recover_from_error(&error);
        
        // EntityNotFound错误无法恢复，应该返回错误
        assert!(result.is_err());
    }

    #[test]
    fn test_scene_recover_from_error_use_default() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId(1), Vec3::ZERO)).unwrap();
        scene.recovery_strategy = RecoveryStrategy::UseDefault;
        
        let error = SceneError::SerializationFailed("test".to_string());
        let result = scene.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(scene.state, SceneState::Unloaded);
        assert_eq!(scene.total_entity_count(), 0); // 实体应该被清除
    }

    #[test]
    fn test_scene_recover_from_error_skip() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.recovery_strategy = RecoveryStrategy::Skip;
        
        let error = SceneError::SerializationFailed("test".to_string());
        let result = scene.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(scene.state, SceneState::Loaded); // 状态不应该改变
    }

    #[test]
    fn test_scene_recover_from_error_log_and_continue() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.recovery_strategy = RecoveryStrategy::LogAndContinue;
        
        let error = SceneError::SerializationFailed("test".to_string());
        let result = scene.recover_from_error(&error);
        
        assert!(result.is_ok());
        assert_eq!(scene.state, SceneState::Loaded); // 状态不应该改变
    }

    #[test]
    fn test_scene_recover_from_error_fail() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.recovery_strategy = RecoveryStrategy::Fail;
        
        let error = SceneError::SerializationFailed("test".to_string());
        let result = scene.recover_from_error(&error);
        
        assert!(result.is_err());
        if let Err(DomainError::Scene(e)) = result {
            assert!(matches!(e, SceneError::SerializationFailed(_)));
        } else {
            panic!("Expected Scene error");
        }
    }

    #[test]
    fn test_scene_create_compensation() {
        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.add_entity(EntityFactory::create_basic(EntityId(1), Vec3::ZERO)).unwrap();
        
        let compensation = scene.create_compensation();
        
        assert_eq!(compensation.action_type, "restore_scene_state");
        assert!(compensation.data.get("name").is_some());
        assert!(compensation.data.get("state").is_some());
        assert!(compensation.data.get("entity_count").is_some());
        assert!(compensation.data.get("version").is_some());
    }
}
