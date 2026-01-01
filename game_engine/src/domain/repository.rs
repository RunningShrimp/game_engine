//! # Repository 模式实现
//!
//! 实现DDD中的Repository模式，提供聚合的持久化和查询抽象。
//!
//! ## 设计原则
//!
//! 1. **聚合根边界**: Repository只管理聚合根，不直接操作内部实体
//! 2. **生命周期管理**: 负责聚合的完整生命周期（CRUD）
//! 3. **一致性保证**: 确保聚合内的业务规则和不变式
//! 4. **领域事件**: 协调领域事件的保存和发布
//!
//! ## 核心组件
//!
//! - [`Repository`] - 通用Repository trait
//! - [`AggregateRepository`] - 支持领域事件的聚合仓储
//! - [`SceneRepositoryImpl`] - 场景聚合仓储
//! - [`RigidBodyRepository`] - 刚体聚合仓储
//! - [`EntityRepository`] - 实体仓储
//! - [`InMemoryRepository`] - 内存实现

use crate::domain::entity::{EntityId, GameEntity};
use crate::domain::errors::{DomainError, PhysicsError, SceneError};
use crate::domain::events::{AggregateRoot, DomainEvent};
use crate::domain::physics::{Collider, RigidBody, RigidBodyId};
use crate::domain::scene::{Scene, SceneId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::hash::Hash;
use std::sync::Arc;

/// 获取ID的trait（为不实现AggregateRoot的类型）
pub trait HasId<ID> {
    /// 获取实体ID
    fn id(&self) -> ID;
}

/// 为Scene实现HasId（使用显式方法调用避免字段/方法歧义）
impl HasId<SceneId> for Scene {
    fn id(&self) -> SceneId {
        Scene::id(self) // 显式调用Scene的id()方法
    }
}

/// 为RigidBody实现HasId（使用显式方法调用避免字段/方法歧义）
impl HasId<RigidBodyId> for RigidBody {
    fn id(&self) -> RigidBodyId {
        RigidBody::id(self) // 显式调用RigidBody的id()方法
    }
}

/// 为GameEntity实现HasId（使用显式方法调用避免字段/方法歧义）
impl HasId<EntityId> for GameEntity {
    fn id(&self) -> EntityId {
        GameEntity::id(self) // 显式调用GameEntity的id()方法
    }
}

// =============================================================================
// Repository Trait (通用接口)
// =============================================================================

/// Repository 通用接口（简化版，不要求AggregateRoot）
///
/// 提供聚合的CRUD操作抽象，遵循DDD Repository模式。
///
/// ## 类型参数
///
/// - `ID`: 聚合标识符类型
/// - `T`: 聚合根类型
///
/// ## 设计约束
///
/// 1. 只管理聚合根，不暴露内部实体
/// 2. 返回的聚合必须是完整的（包含所有必需值对象）
/// 3. 确保聚合的一致性和不变式
///
/// # 示例
///
/// ```rust,ignore
/// use game_engine::domain::repository::Repository;
///
/// let mut repo = SceneRepositoryImpl::new();
///
/// // 创建
/// let scene = Scene::new(SceneId::new(1), "TestScene".to_string());
/// repo.add(scene.clone()).await?;
///
/// // 查询
/// let found = repo.find_by_id(&scene.id()).await?;
/// assert!(found.is_some());
///
/// // 更新
/// repo.update(&scene).await?;
///
/// // 删除
/// repo.delete(&scene.id()).await?;
/// ```
pub trait Repository<ID, T>: Send + Sync
where
    ID: Clone + PartialEq + Eq + Hash + fmt::Debug + Send + Sync,
    T: HasId<ID> + Clone + Send + Sync,
{
    /// 添加新聚合
    ///
    /// # 错误
    ///
    /// - 如果ID已存在，返回错误
    fn add(&mut self, aggregate: T) -> Result<(), RepositoryError>;

    /// 更新现有聚合
    ///
    /// # 错误
    ///
    /// - 如果聚合不存在，返回错误
    fn update(&mut self, aggregate: &T) -> Result<(), RepositoryError>;

    /// 删除聚合
    ///
    /// # 错误
    ///
    /// - 如果聚合不存在，返回错误
    fn delete(&mut self, id: &ID) -> Result<Option<T>, RepositoryError>;

    /// 根据ID查找聚合
    fn find_by_id(&self, id: &ID) -> Result<Option<T>, RepositoryError>;

    /// 查询所有聚合
    fn find_all(&self) -> Result<Vec<T>, RepositoryError>;

    /// 检查聚合是否存在
    fn exists(&self, id: &ID) -> Result<bool, RepositoryError>;

    /// 获取聚合数量
    fn count(&self) -> Result<usize, RepositoryError>;

    /// 根据条件查询（泛型查询接口）
    fn find_by_predicate<F>(&self, predicate: F) -> Result<Vec<T>, RepositoryError>
    where
        F: Fn(&T) -> bool + Send + Sync;

    /// 保存聚合（添加或更新）
    fn save(&mut self, aggregate: &T) -> Result<(), RepositoryError> {
        if self.exists(&aggregate.id())? {
            self.update(aggregate)
        } else {
            self.add(aggregate.clone())
        }
    }
}

/// 支持领域事件的Repository（用于完整的聚合根）
pub trait AggregateRepository<ID, T>: Repository<ID, T>
where
    ID: Clone + PartialEq + Eq + Hash + fmt::Debug + Send + Sync,
    T: AggregateRoot + HasId<ID> + Clone + Send + Sync,
{
    /// 获取未提交的事件
    fn get_uncommitted_events(&self, id: &ID)
    -> Result<Vec<Box<dyn DomainEvent>>, RepositoryError>;

    /// 标记事件为已提交
    fn mark_events_committed(&mut self, id: &ID) -> Result<(), RepositoryError>;
}

// 为了让AggregateRoot更通用，添加一个id()方法
// 这需要在各个聚合根上实现

/// Repository 错误类型
#[derive(Debug, Clone, thiserror::Error)]
pub enum RepositoryError {
    /// 聚合不存在
    #[error("Aggregate not found: {0}")]
    NotFound(String),

    /// 聚合已存在
    #[error("Aggregate already exists: {0}")]
    AlreadyExists(String),

    /// 并发冲突
    #[error("Concurrency conflict: {0}")]
    Conflict(String),

    /// 验证失败
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// 持久化失败
    #[error("Persistence failed: {0}")]
    PersistenceFailed(String),

    /// 领域错误
    #[error("Domain error: {0}")]
    DomainError(String),
}

impl From<SceneError> for RepositoryError {
    fn from(err: SceneError) -> Self {
        RepositoryError::DomainError(err.to_string())
    }
}

impl From<PhysicsError> for RepositoryError {
    fn from(err: PhysicsError) -> Self {
        RepositoryError::DomainError(err.to_string())
    }
}

impl From<DomainError> for RepositoryError {
    fn from(err: DomainError) -> Self {
        RepositoryError::DomainError(err.to_string())
    }
}

// =============================================================================
// Scene Repository (场景仓储)
// =============================================================================

/// 场景仓储的完整实现
///
/// 负责场景聚合的持久化和查询。
///
/// ## 职责
///
/// - 管理场景的完整生命周期
/// - 确保场景的业务规则和不变式
/// - 协调领域事件的保存
pub struct SceneRepositoryImpl {
    /// 场景存储
    scenes: HashMap<SceneId, Scene>,
    /// 当前活跃场景
    active_scene: Option<SceneId>,
    /// 事件存储（模拟）
    event_store: Vec<StoredEvent>,
}

/// 存储的事件（不实现Clone/Debug，因为DomainEvent trait对象不支持）
pub struct StoredEvent {
    aggregate_id: String,
    event: Box<dyn DomainEvent>,
    timestamp: u64,
}

impl SceneRepositoryImpl {
    /// 创建新的场景仓储
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            active_scene: None,
            event_store: Vec::new(),
        }
    }

    /// 获取当前活跃场景
    pub fn get_active_scene(&self) -> Result<Option<Scene>, RepositoryError> {
        if let Some(id) = &self.active_scene {
            Ok(self.scenes.get(id).cloned())
        } else {
            Ok(None)
        }
    }

    /// 设置活跃场景
    pub fn set_active_scene(&mut self, id: SceneId) -> Result<(), RepositoryError> {
        if !self.scenes.contains_key(&id) {
            return Err(RepositoryError::NotFound(format!("Scene {}", id.as_u64())));
        }
        self.active_scene = Some(id);
        Ok(())
    }

    /// 查询场景名称
    pub fn find_by_name(&self, name: &str) -> Result<Option<Scene>, RepositoryError> {
        Ok(self.scenes.values().find(|s| s.name() == name).cloned())
    }

    /// 获取事件存储
    pub fn get_events(&self, aggregate_id: &str) -> Vec<&StoredEvent> {
        self.event_store.iter().filter(|e| e.aggregate_id == aggregate_id).collect()
    }
}

impl Default for SceneRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository<SceneId, Scene> for SceneRepositoryImpl {
    fn add(&mut self, aggregate: Scene) -> Result<(), RepositoryError> {
        let id = aggregate.id();
        if self.scenes.contains_key(&id) {
            return Err(RepositoryError::AlreadyExists(format!(
                "Scene {}",
                id.as_u64()
            )));
        }

        // Scene是AggregateRoot，提取未提交事件
        let events = aggregate.clone().take_uncommitted_events();
        self.scenes.insert(id, aggregate);

        // 保存事件
        for event in events {
            self.event_store.push(StoredEvent {
                aggregate_id: format!("Scene_{}", id.as_u64()),
                event,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }

        Ok(())
    }

    fn update(&mut self, aggregate: &Scene) -> Result<(), RepositoryError> {
        let id = aggregate.id();
        if !self.scenes.contains_key(&id) {
            return Err(RepositoryError::NotFound(format!("Scene {}", id.as_u64())));
        }

        // Scene是AggregateRoot，提取未提交事件
        let events = self.scenes.get(&id).unwrap().clone().take_uncommitted_events();
        self.scenes.insert(id, aggregate.clone());

        // 保存事件
        for event in events {
            self.event_store.push(StoredEvent {
                aggregate_id: format!("Scene_{}", id.as_u64()),
                event,
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }

        Ok(())
    }

    fn delete(&mut self, id: &SceneId) -> Result<Option<Scene>, RepositoryError> {
        Ok(self.scenes.remove(id))
    }

    fn find_by_id(&self, id: &SceneId) -> Result<Option<Scene>, RepositoryError> {
        Ok(self.scenes.get(id).cloned())
    }

    fn find_all(&self) -> Result<Vec<Scene>, RepositoryError> {
        Ok(self.scenes.values().cloned().collect())
    }

    fn exists(&self, id: &SceneId) -> Result<bool, RepositoryError> {
        Ok(self.scenes.contains_key(id))
    }

    fn count(&self) -> Result<usize, RepositoryError> {
        Ok(self.scenes.len())
    }

    fn find_by_predicate<F>(&self, predicate: F) -> Result<Vec<Scene>, RepositoryError>
    where
        F: Fn(&Scene) -> bool + Send + Sync,
    {
        Ok(self.scenes.values().filter(|s| predicate(s)).cloned().collect())
    }
}

// Scene实现AggregateRoot，所以实现AggregateRepository
impl AggregateRepository<SceneId, Scene> for SceneRepositoryImpl {
    fn get_uncommitted_events(
        &self,
        id: &SceneId,
    ) -> Result<Vec<Box<dyn DomainEvent>>, RepositoryError> {
        if let Some(scene) = self.scenes.get(id) {
            Ok(scene.clone().take_uncommitted_events())
        } else {
            Err(RepositoryError::NotFound(format!("Scene {}", id.as_u64())))
        }
    }

    fn mark_events_committed(&mut self, id: &SceneId) -> Result<(), RepositoryError> {
        // Scene的事件在add/update时已经处理，这里不需要额外操作
        Ok(())
    }
}

// =============================================================================
// RigidBody Repository (刚体仓储)
// =============================================================================

/// 刚体仓储
///
/// 管理物理刚体聚合的持久化和查询。
pub struct RigidBodyRepository {
    /// 刚体存储
    bodies: HashMap<RigidBodyId, RigidBody>,
    /// 碰撞体存储（关联到刚体）
    colliders: HashMap<RigidBodyId, Vec<Collider>>,
    /// 事件存储
    event_store: Vec<StoredEvent>,
}

impl RigidBodyRepository {
    /// 创建新的刚体仓储
    pub fn new() -> Self {
        Self {
            bodies: HashMap::new(),
            colliders: HashMap::new(),
            event_store: Vec::new(),
        }
    }

    /// 添加碰撞体到刚体
    pub fn add_collider(&mut self, body_id: RigidBodyId, collider: Collider) {
        self.colliders.entry(body_id).or_insert_with(Vec::new).push(collider);
    }

    /// 获取刚体的所有碰撞体
    pub fn get_colliders(&self, body_id: &RigidBodyId) -> Option<&[Collider]> {
        self.colliders.get(body_id).map(|v| v.as_slice())
    }

    /// 查询特定类型的刚体
    pub fn find_by_type(&self, body_type: crate::domain::physics::RigidBodyType) -> Vec<RigidBody> {
        self.bodies.values().filter(|b| b.body_type() == body_type).cloned().collect()
    }
}

impl Default for RigidBodyRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository<RigidBodyId, RigidBody> for RigidBodyRepository {
    fn add(&mut self, aggregate: RigidBody) -> Result<(), RepositoryError> {
        let id = aggregate.id();
        if self.bodies.contains_key(&id) {
            return Err(RepositoryError::AlreadyExists(format!(
                "RigidBody {:?}",
                id
            )));
        }
        self.bodies.insert(id, aggregate);
        Ok(())
    }

    fn update(&mut self, aggregate: &RigidBody) -> Result<(), RepositoryError> {
        let id = aggregate.id();
        if !self.bodies.contains_key(&id) {
            return Err(RepositoryError::NotFound(format!("RigidBody {:?}", id)));
        }
        self.bodies.insert(id, aggregate.clone());
        Ok(())
    }

    fn delete(&mut self, id: &RigidBodyId) -> Result<Option<RigidBody>, RepositoryError> {
        Ok(self.bodies.remove(id))
    }

    fn find_by_id(&self, id: &RigidBodyId) -> Result<Option<RigidBody>, RepositoryError> {
        Ok(self.bodies.get(id).cloned())
    }

    fn find_all(&self) -> Result<Vec<RigidBody>, RepositoryError> {
        Ok(self.bodies.values().cloned().collect())
    }

    fn exists(&self, id: &RigidBodyId) -> Result<bool, RepositoryError> {
        Ok(self.bodies.contains_key(id))
    }

    fn count(&self) -> Result<usize, RepositoryError> {
        Ok(self.bodies.len())
    }

    fn find_by_predicate<F>(&self, predicate: F) -> Result<Vec<RigidBody>, RepositoryError>
    where
        F: Fn(&RigidBody) -> bool + Send + Sync,
    {
        Ok(self.bodies.values().filter(|b| predicate(b)).cloned().collect())
    }
}

// =============================================================================
// Entity Repository (实体仓储)
// =============================================================================

/// 实体仓储
///
/// 管理游戏实体的持久化和查询。
pub struct EntityRepository {
    /// 实体存储
    entities: HashMap<EntityId, GameEntity>,
    /// 实体名称索引
    by_name: HashMap<String, Vec<EntityId>>,
}

impl EntityRepository {
    /// 创建新的实体仓储
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            by_name: HashMap::new(),
        }
    }

    /// 根据名称查询实体
    pub fn find_by_name(&self, name: &str) -> Result<Vec<GameEntity>, RepositoryError> {
        if let Some(ids) = self.by_name.get(name) {
            let mut result = Vec::new();
            for id in ids {
                if let Some(entity) = self.entities.get(id) {
                    result.push(entity.clone());
                }
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
}

impl Default for EntityRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository<EntityId, GameEntity> for EntityRepository {
    fn add(&mut self, aggregate: GameEntity) -> Result<(), RepositoryError> {
        let id = aggregate.id();
        if self.entities.contains_key(&id) {
            return Err(RepositoryError::AlreadyExists(format!("Entity {:?}", id)));
        }

        // 索引实体名称（如果有）
        if let Some(name) = aggregate.name() {
            self.by_name.entry(name.to_string()).or_insert_with(Vec::new).push(id);
        }

        self.entities.insert(id, aggregate);
        Ok(())
    }

    fn update(&mut self, aggregate: &GameEntity) -> Result<(), RepositoryError> {
        let id = aggregate.id();
        if !self.entities.contains_key(&id) {
            return Err(RepositoryError::NotFound(format!("Entity {:?}", id)));
        }

        // 更新名称索引
        if let Some(old) = self.entities.get(&id) {
            if let Some(old_name) = old.name() {
                if let Some(ids) = self.by_name.get_mut(old_name) {
                    ids.retain(|eid| *eid != id);
                }
            }
        }

        if let Some(name) = aggregate.name() {
            self.by_name.entry(name.to_string()).or_insert_with(Vec::new).push(id);
        }

        self.entities.insert(id, aggregate.clone());
        Ok(())
    }

    fn delete(&mut self, id: &EntityId) -> Result<Option<GameEntity>, RepositoryError> {
        let removed = self.entities.remove(id);
        if let Some(entity) = &removed {
            // 从名称索引中移除
            if let Some(name) = entity.name() {
                if let Some(ids) = self.by_name.get_mut(name) {
                    ids.retain(|eid| eid != id);
                }
            }
        }
        Ok(removed)
    }

    fn find_by_id(&self, id: &EntityId) -> Result<Option<GameEntity>, RepositoryError> {
        Ok(self.entities.get(id).cloned())
    }

    fn find_all(&self) -> Result<Vec<GameEntity>, RepositoryError> {
        Ok(self.entities.values().cloned().collect())
    }

    fn exists(&self, id: &EntityId) -> Result<bool, RepositoryError> {
        Ok(self.entities.contains_key(id))
    }

    fn count(&self) -> Result<usize, RepositoryError> {
        Ok(self.entities.len())
    }

    fn find_by_predicate<F>(&self, predicate: F) -> Result<Vec<GameEntity>, RepositoryError>
    where
        F: Fn(&GameEntity) -> bool + Send + Sync,
    {
        Ok(self.entities.values().filter(|e| predicate(e)).cloned().collect())
    }
}

// =============================================================================
// 通用内存Repository实现
// =============================================================================

/// 通用内存Repository实现
///
/// 用于快速原型和测试。
pub struct InMemoryRepository<ID, T>
where
    ID: Clone + PartialEq + Eq + Hash + fmt::Debug + Send + Sync,
    T: HasId<ID> + Clone + Send + Sync,
{
    storage: HashMap<ID, T>,
}

impl<ID, T> InMemoryRepository<ID, T>
where
    ID: Clone + PartialEq + Eq + Hash + fmt::Debug + Send + Sync,
    T: HasId<ID> + Clone + Send + Sync,
{
    /// 创建新的内存Repository
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    /// 获取ID的辅助函数（使用HasId trait）
    fn get_id(aggregate: &T) -> ID {
        aggregate.id()
    }
}

impl<ID, T> Default for InMemoryRepository<ID, T>
where
    ID: Clone + PartialEq + Eq + Hash + fmt::Debug + Send + Sync,
    T: HasId<ID> + Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

/// 实现Repository trait for InMemoryRepository
impl<ID, T> Repository<ID, T> for InMemoryRepository<ID, T>
where
    ID: Clone + PartialEq + Eq + Hash + fmt::Debug + Send + Sync,
    T: HasId<ID> + Clone + Send + Sync,
{
    fn add(&mut self, aggregate: T) -> Result<(), RepositoryError> {
        let id = Self::get_id(&aggregate);
        if self.storage.contains_key(&id) {
            return Err(RepositoryError::AlreadyExists(format!(
                "Aggregate with id {:?}",
                id
            )));
        }
        self.storage.insert(id, aggregate);
        Ok(())
    }

    fn update(&mut self, aggregate: &T) -> Result<(), RepositoryError> {
        let id = Self::get_id(aggregate);
        if !self.storage.contains_key(&id) {
            return Err(RepositoryError::NotFound(format!(
                "Aggregate with id {:?}",
                id
            )));
        }
        self.storage.insert(id, aggregate.clone());
        Ok(())
    }

    fn delete(&mut self, id: &ID) -> Result<Option<T>, RepositoryError> {
        Ok(self.storage.remove(id))
    }

    fn find_by_id(&self, id: &ID) -> Result<Option<T>, RepositoryError> {
        Ok(self.storage.get(id).cloned())
    }

    fn find_all(&self) -> Result<Vec<T>, RepositoryError> {
        Ok(self.storage.values().cloned().collect())
    }

    fn exists(&self, id: &ID) -> Result<bool, RepositoryError> {
        Ok(self.storage.contains_key(id))
    }

    fn count(&self) -> Result<usize, RepositoryError> {
        Ok(self.storage.len())
    }

    fn find_by_predicate<F>(&self, predicate: F) -> Result<Vec<T>, RepositoryError>
    where
        F: Fn(&T) -> bool + Send + Sync,
    {
        Ok(self.storage.values().filter(|e| predicate(e)).cloned().collect())
    }
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scene_repository_crud() {
        let mut repo = SceneRepositoryImpl::new();

        // 创建测试场景
        let scene = Scene::new(SceneId::new(1), "TestScene".to_string());

        // 测试 add
        repo.add(scene.clone()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        // 测试 find_by_id
        let found = repo.find_by_id(&scene.id()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "TestScene");

        // 测试 exists
        assert!(repo.exists(&scene.id()).unwrap());

        // 测试 update
        repo.update(&scene).unwrap();

        // 测试 delete
        let deleted = repo.delete(&scene.id()).unwrap();
        assert!(deleted.is_some());
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_scene_repository_find_by_name() {
        let mut repo = SceneRepositoryImpl::new();

        let scene = Scene::new(SceneId::new(1), "MyScene".to_string());
        repo.add(scene).unwrap();

        let found = repo.find_by_name("MyScene").unwrap();
        assert!(found.is_some());

        let not_found = repo.find_by_name("NonExistent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_scene_repository_active_scene() {
        let mut repo = SceneRepositoryImpl::new();

        let scene = Scene::new(SceneId::new(1), "ActiveScene".to_string());
        repo.add(scene).unwrap();

        repo.set_active_scene(SceneId::new(1)).unwrap();
        let active = repo.get_active_scene().unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().name(), "ActiveScene");
    }

    #[test]
    fn test_rigid_body_repository_crud() {
        let mut repo = RigidBodyRepository::new();

        // 创建测试刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            crate::domain::physics::RigidBodyType::Dynamic,
            glam::Vec3::ZERO,
        );

        // 测试 add
        repo.add(body.clone()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        // 测试 find_by_id
        let found = repo.find_by_id(&body.id()).unwrap();
        assert!(found.is_some());

        // 测试 find_by_type
        let dynamic_bodies = repo.find_by_type(crate::domain::physics::RigidBodyType::Dynamic);
        assert_eq!(dynamic_bodies.len(), 1);

        // 测试 delete
        repo.delete(&body.id()).unwrap();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_rigid_body_repository_colliders() {
        let mut repo = RigidBodyRepository::new();
        let body_id = RigidBodyId::new(1);

        let body = RigidBody::new(
            body_id,
            crate::domain::physics::RigidBodyType::Dynamic,
            glam::Vec3::ZERO,
        );
        repo.add(body).unwrap();

        // 添加碰撞体
        let collider = Collider::new(
            crate::domain::physics::ColliderId::new(1),
            body_id,
            crate::domain::physics::ShapeType::Sphere { radius: 1.0 },
            1.0, // density
        );
        repo.add_collider(body_id, collider);

        // 查询碰撞体
        let colliders = repo.get_colliders(&body_id);
        assert!(colliders.is_some());
        assert_eq!(colliders.unwrap().len(), 1);
    }

    // -------------------------------------------------------------------------
    // InMemoryRepository 测试
    // -------------------------------------------------------------------------

    #[test]
    fn test_inmemory_repository_scene_crud() {
        let mut repo: InMemoryRepository<SceneId, Scene> = InMemoryRepository::new();

        // 创建测试场景
        let scene = Scene::new(SceneId::new(1), "TestScene".to_string());

        // 测试 add
        repo.add(scene.clone()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        // 测试 find_by_id
        let found = repo.find_by_id(&scene.id()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name(), "TestScene");

        // 测试 exists
        assert!(repo.exists(&scene.id()).unwrap());

        // 测试 update
        repo.update(&scene).unwrap();

        // 测试 find_all
        let all = repo.find_all().unwrap();
        assert_eq!(all.len(), 1);

        // 测试 delete
        let deleted = repo.delete(&scene.id()).unwrap();
        assert!(deleted.is_some());
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_inmemory_repository_rigid_body_crud() {
        let mut repo: InMemoryRepository<RigidBodyId, RigidBody> = InMemoryRepository::new();

        // 创建测试刚体
        let body = RigidBody::new(
            RigidBodyId::new(1),
            crate::domain::physics::RigidBodyType::Dynamic,
            glam::Vec3::ZERO,
        );

        // 测试 add
        repo.add(body.clone()).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        // 测试 find_by_id
        let found = repo.find_by_id(&body.id()).unwrap();
        assert!(found.is_some());

        // 测试 exists
        assert!(repo.exists(&body.id()).unwrap());

        // 测试 update
        repo.update(&body).unwrap();

        // 测试 delete
        repo.delete(&body.id()).unwrap();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_inmemory_repository_error_handling() {
        let mut repo: InMemoryRepository<SceneId, Scene> = InMemoryRepository::new();

        let scene = Scene::new(SceneId::new(1), "TestScene".to_string());

        // 测试重复添加
        repo.add(scene.clone()).unwrap();
        let result = repo.add(scene.clone());
        assert!(result.is_err());

        // 测试更新不存在的实体
        let nonexistent_scene = Scene::new(SceneId::new(999), "NonExistent".to_string());
        let result = repo.update(&nonexistent_scene);
        assert!(result.is_err());

        // 测试查找不存在的实体
        let result = repo.find_by_id(&SceneId::new(999));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());

        // 测试删除不存在的实体
        let result = repo.delete(&SceneId::new(999));
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_inmemory_repository_find_by_predicate() {
        let mut repo: InMemoryRepository<SceneId, Scene> = InMemoryRepository::new();

        // 添加多个场景
        repo.add(Scene::new(SceneId::new(1), "Scene1".to_string())).unwrap();
        repo.add(Scene::new(SceneId::new(2), "Scene2".to_string())).unwrap();
        repo.add(Scene::new(SceneId::new(3), "Scene3".to_string())).unwrap();

        // 测试 find_by_predicate
        let scenes_starting_with_2 =
            repo.find_by_predicate(|s| s.name().starts_with("Scene2")).unwrap();
        assert_eq!(scenes_starting_with_2.len(), 1);

        // 测试查找所有包含"Scene"的场景
        let all_scenes = repo.find_by_predicate(|s| s.name().contains("Scene")).unwrap();
        assert_eq!(all_scenes.len(), 3);

        // 测试查找不匹配的场景
        let no_scenes = repo.find_by_predicate(|s| s.name().starts_with("XYZ")).unwrap();
        assert_eq!(no_scenes.len(), 0);
    }

    #[test]
    fn test_inmemory_repository_save() {
        let mut repo: InMemoryRepository<SceneId, Scene> = InMemoryRepository::new();

        let scene = Scene::new(SceneId::new(1), "TestScene".to_string());

        // 测试 save (add)
        repo.save(&scene).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
        assert!(repo.exists(&scene.id()).unwrap());

        // 测试 save (update)
        repo.save(&scene).unwrap();
        assert_eq!(repo.count().unwrap(), 1);
    }

    #[test]
    fn test_inmemory_repository_thread_safe() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let repo = Arc::new(Mutex::new(InMemoryRepository::<SceneId, Scene>::new()));
        let mut handles = vec![];

        // 创建多个线程同时操作Repository
        for i in 0..10 {
            let repo_clone = Arc::clone(&repo);
            let handle = thread::spawn(move || {
                let mut repo = repo_clone.lock().unwrap();
                let scene = Scene::new(SceneId::new(i as u64), format!("Scene{}", i));
                repo.add(scene).unwrap();
            });
            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 验证所有场景都已添加
        let repo = repo.lock().unwrap();
        assert_eq!(repo.count().unwrap(), 10);
    }
}
