//! 领域事件溯源系统
//!
//! 与类型安全的事件系统集成，提供事件存储、重放、快照和版本控制功能。
//!
//! 注意：此模块是新的类型安全实现，与core::event_sourcing模块并行存在。
//! 新代码应该使用此模块，旧代码可以继续使用core::event_sourcing。

use crate::domain::event_registry::EventRegistry;
use crate::domain::events::{AggregateRoot, DomainEvent, EventError};
use crate::error::{safe_lock, safe_read, safe_write};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, RwLock};
use tracing;

/// 事件ID（时间戳 + 序列号）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventId {
    /// 时间戳（纳秒）
    pub timestamp_ns: i64,
    /// 序列号（同一时间戳内的顺序）
    pub sequence: u64,
}

impl EventId {
    pub fn new(timestamp_ns: i64, sequence: u64) -> Self {
        Self {
            timestamp_ns,
            sequence,
        }
    }

    pub fn now(sequence: u64) -> Self {
        Self {
            timestamp_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64,
            sequence,
        }
    }
}

/// 存储的事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// 事件ID
    pub id: EventId,
    /// 事件类型
    pub event_type: String,
    /// 事件数据（序列化）
    pub data: Vec<u8>,
    /// 聚合ID（字符串格式，支持更灵活的ID类型）
    pub aggregate_id: Option<String>,
    /// 聚合版本号
    pub aggregate_version: u64,
}

/// 领域事件存储trait（类型安全版本）
///
/// 与core::event_sourcing::EventStore不同，此trait使用String类型的aggregate_id
/// 以支持更灵活的聚合ID类型
pub trait EventStore: Send + Sync + std::fmt::Debug {
    /// 保存事件
    fn save_event(&mut self, event: StoredEvent) -> Result<(), EventError>;

    /// 获取事件
    fn get_event(&self, id: EventId) -> Result<StoredEvent, EventError>;

    /// 获取所有事件
    fn get_all_events(&self) -> Vec<StoredEvent>;

    /// 获取聚合的所有事件
    fn get_aggregate_events(&self, aggregate_id: &str) -> Vec<StoredEvent>;

    /// 获取事件范围
    fn get_events_range(&self, from: EventId, to: EventId) -> Vec<StoredEvent>;

    /// 获取聚合的事件（从指定版本开始）
    fn get_aggregate_events_from_version(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Vec<StoredEvent>;

    /// 删除指定序列号之前的事件
    fn delete_events_before(&mut self, sequence: u64);

    /// 清除所有事件
    fn clear(&mut self);
}

/// 内存事件存储（用于测试和开发）
#[derive(Debug, Default)]
pub struct MemoryEventStore {
    events: Vec<StoredEvent>,
    next_sequence: u64,
}

impl MemoryEventStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl EventStore for MemoryEventStore {
    fn save_event(&mut self, event: StoredEvent) -> Result<(), EventError> {
        self.events.push(event);
        Ok(())
    }

    fn get_event(&self, id: EventId) -> Result<StoredEvent, EventError> {
        self.events
            .iter()
            .find(|e| e.id == id)
            .cloned()
            .ok_or(EventError::UnknownEventType(format!(
                "Event {} not found",
                id.sequence
            )))
    }

    fn get_all_events(&self) -> Vec<StoredEvent> {
        self.events.clone()
    }

    fn get_aggregate_events(&self, aggregate_id: &str) -> Vec<StoredEvent> {
        self.events
            .iter()
            .filter(|e| e.aggregate_id.as_ref().map(|s| s.as_str()) == Some(aggregate_id))
            .cloned()
            .collect()
    }

    fn get_events_range(&self, from: EventId, to: EventId) -> Vec<StoredEvent> {
        self.events.iter().filter(|e| e.id >= from && e.id <= to).cloned().collect()
    }

    fn get_aggregate_events_from_version(
        &self,
        aggregate_id: &str,
        from_version: u64,
    ) -> Vec<StoredEvent> {
        self.events
            .iter()
            .filter(|e| {
                e.aggregate_id.as_ref().map(|s| s.as_str()) == Some(aggregate_id)
                    && e.aggregate_version >= from_version
            })
            .cloned()
            .collect()
    }

    fn delete_events_before(&mut self, sequence: u64) {
        self.events.retain(|e| e.id.sequence >= sequence);
    }

    fn clear(&mut self) {
        self.events.clear();
        self.next_sequence = 0;
    }
}

/// 快照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// 快照ID
    pub id: EventId,
    /// 聚合ID
    pub aggregate_id: String,
    /// 聚合版本号
    pub aggregate_version: u64,
    /// 快照数据（序列化的聚合状态）
    pub data: Vec<u8>,
    /// 创建时间
    pub created_at: i64,
}

/// 领域快照存储trait（类型安全版本）
///
/// 与core::event_sourcing::SnapshotStore不同，此trait使用String类型的aggregate_id
pub trait SnapshotStore: Send + Sync + std::fmt::Debug {
    /// 保存快照
    fn save_snapshot(&mut self, snapshot: Snapshot) -> Result<(), EventError>;

    /// 获取最新快照
    fn get_latest_snapshot(&self, aggregate_id: &str) -> Result<Snapshot, EventError>;

    /// 获取快照
    fn get_snapshot(&self, id: EventId) -> Result<Snapshot, EventError>;

    /// 获取聚合的所有快照
    fn get_aggregate_snapshots(&self, aggregate_id: &str) -> Vec<Snapshot>;

    /// 清除快照
    fn clear(&mut self);
}

/// 内存快照存储
#[derive(Debug, Default)]
pub struct MemorySnapshotStore {
    snapshots: Vec<Snapshot>,
}

impl MemorySnapshotStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SnapshotStore for MemorySnapshotStore {
    fn save_snapshot(&mut self, snapshot: Snapshot) -> Result<(), EventError> {
        // 移除旧的快照（只保留最新的）
        self.snapshots.retain(|s| s.aggregate_id != snapshot.aggregate_id);
        self.snapshots.push(snapshot);
        Ok(())
    }

    fn get_latest_snapshot(&self, aggregate_id: &str) -> Result<Snapshot, EventError> {
        self.snapshots
            .iter()
            .filter(|s| s.aggregate_id == aggregate_id)
            .max_by_key(|s| s.id)
            .cloned()
            .ok_or(EventError::UnknownEventType(format!(
                "Snapshot for aggregate {} not found",
                aggregate_id
            )))
    }

    fn get_snapshot(&self, id: EventId) -> Result<Snapshot, EventError> {
        self.snapshots
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or(EventError::UnknownEventType(format!(
                "Snapshot {} not found",
                id.sequence
            )))
    }

    fn get_aggregate_snapshots(&self, aggregate_id: &str) -> Vec<Snapshot> {
        self.snapshots
            .iter()
            .filter(|s| s.aggregate_id == aggregate_id)
            .cloned()
            .collect()
    }

    fn clear(&mut self) {
        self.snapshots.clear();
    }
}

/// 事件溯源管理器
///
/// 与类型安全的事件系统集成，提供：
/// - 事件存储和检索
/// - 事件重放
/// - 快照管理
/// - 版本控制
/// - 与AggregateRoot集成
///
/// # 命名说明
///
/// 注意：此类型名为`EventSourcingManager`，但根据命名规范应该为`EventSourcingService`。
/// 它包含业务逻辑（事件重放、快照管理等），而不是简单的资源管理。
/// 为了保持向后兼容，暂时保留原名称。未来版本将重命名为`EventSourcingService`。
#[derive(Debug)]
pub struct EventSourcingManager {
    /// 事件存储
    event_store: Arc<RwLock<Box<dyn EventStore>>>,
    /// 快照存储
    snapshot_store: Arc<RwLock<Box<dyn SnapshotStore>>>,
    /// 事件序列号生成器
    sequence_generator: Arc<Mutex<u64>>,
    /// 快照间隔（每N个事件创建一个快照）
    snapshot_interval: usize,
    /// 最大事件历史长度
    max_history_length: usize,
    /// 事件类型注册表（用于序列化/反序列化）
    event_registry: Arc<RwLock<EventRegistry>>,
}

impl EventSourcingManager {
    /// 创建新的事件溯源管理器
    pub fn new(
        event_store: Arc<RwLock<Box<dyn EventStore>>>,
        snapshot_store: Arc<RwLock<Box<dyn SnapshotStore>>>,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
            sequence_generator: Arc::new(Mutex::new(0)),
            snapshot_interval: 100,    // 默认每100个事件创建一个快照
            max_history_length: 10000, // 默认保留10000个事件
            event_registry: Arc::new(RwLock::new(EventRegistry::new())),
        }
    }

    /// 使用自定义事件注册表创建
    pub fn with_registry(
        event_store: Arc<RwLock<Box<dyn EventStore>>>,
        snapshot_store: Arc<RwLock<Box<dyn SnapshotStore>>>,
        event_registry: Arc<RwLock<EventRegistry>>,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
            sequence_generator: Arc::new(Mutex::new(0)),
            snapshot_interval: 100,
            max_history_length: 10000,
            event_registry,
        }
    }

    /// 获取事件注册表引用
    pub fn event_registry(&self) -> Arc<RwLock<EventRegistry>> {
        self.event_registry.clone()
    }

    /// 设置快照间隔
    pub fn set_snapshot_interval(&mut self, interval: usize) {
        self.snapshot_interval = interval;
    }

    /// 设置最大历史长度
    pub fn set_max_history_length(&mut self, max_length: usize) {
        self.max_history_length = max_length;
    }

    /// 提交聚合根的事件（从AggregateRoot获取未提交的事件）
    ///
    /// 注意：由于DomainEvent trait object不能直接序列化，此方法需要事件类型信息
    /// 实际使用中，应该通过事件类型注册表来处理
    pub fn commit_aggregate_events<A: AggregateRoot>(
        &self,
        aggregate: &mut A,
        _world: &mut World,
    ) -> Result<EventId, EventError> {
        let aggregate_id = aggregate.aggregate_id();
        let events = aggregate.take_uncommitted_events();

        if events.is_empty() {
            return Err(EventError::ApplyFailed("No uncommitted events".to_string()));
        }

        // 获取当前聚合版本
        let current_version = self.get_aggregate_version(&aggregate_id)?;
        let mut new_version = current_version;

        // 保存所有事件（通过序列化/反序列化）
        // 注意：这里需要事件类型注册表来正确反序列化
        let mut last_event_id = None;
        for event in events {
            new_version += 1;
            // 由于DomainEvent trait object的限制，我们需要通过事件类型来处理
            // 这里简化处理，实际应该使用事件类型注册表
            let event_type = event.event_type();
            let mut sequence = safe_lock(&self.sequence_generator, "sequence_generator")
                .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
            *sequence += 1;
            let event_id = EventId::now(*sequence);

            // 使用事件注册表序列化事件
            let registry = safe_read(&self.event_registry, "event_registry")
                .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;

            // 注意：由于DomainEvent trait object的限制，我们需要通过类型ID来序列化
            // 这里简化处理：如果事件类型已注册，尝试序列化；否则使用空数据
            let data = if registry.is_registered(event_type) {
                // 尝试通过事件注册表序列化（需要事件实现Serialize）
                // 由于trait object的限制，这里暂时使用空数据
                // 实际使用中，应该在事件发布时使用save_event方法
                Vec::new()
            } else {
                Vec::new()
            };

            let stored_event = StoredEvent {
                id: event_id,
                event_type: event_type.to_string(),
                data,
                aggregate_id: Some(aggregate_id.clone()),
                aggregate_version: new_version,
            };

            let mut store = safe_write(&self.event_store, "event_store")
                .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
            store.save_event(stored_event)?;
            last_event_id = Some(event_id);
        }

        // 标记事件为已提交
        aggregate.mark_events_committed();

        // 检查是否需要创建快照
        // 注意：快照创建需要Serialize trait，这里暂时跳过
        // 实际使用中，应该通过事件类型注册表来处理
        if new_version % (self.snapshot_interval as u64) == 0 {
            tracing::debug!(
                "Snapshot interval reached for aggregate {}, but snapshot creation requires Serialize trait",
                aggregate_id
            );
        }

        // 清理旧事件
        self.cleanup_old_events()?;

        last_event_id.ok_or_else(|| EventError::ApplyFailed("No events saved".to_string()))
    }

    /// 保存单个事件
    pub fn save_event<E: DomainEvent + Serialize>(
        &self,
        event: &E,
        aggregate_id: Option<&str>,
        aggregate_version: u64,
        _world: &World,
    ) -> Result<EventId, EventError> {
        let mut sequence = safe_lock(&self.sequence_generator, "sequence_generator")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
        *sequence += 1;
        let event_id = EventId::now(*sequence);

        // 序列化事件
        let event_type = event.event_type();
        let data =
            bincode::serialize(event).map_err(|e| EventError::SerializationError(e.to_string()))?;

        let stored_event = StoredEvent {
            id: event_id,
            event_type: event_type.to_string(),
            data,
            aggregate_id: aggregate_id.map(|s| s.to_string()),
            aggregate_version,
        };

        // 保存事件
        let mut store = safe_write(&self.event_store, "event_store")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
        store.save_event(stored_event)?;

        Ok(event_id)
    }

    /// 获取聚合的当前版本号
    fn get_aggregate_version(&self, aggregate_id: &str) -> Result<u64, EventError> {
        let store = safe_read(&self.event_store, "event_store")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
        let events = store.get_aggregate_events(aggregate_id);
        Ok(events.iter().map(|e| e.aggregate_version).max().unwrap_or(0))
    }

    /// 创建快照（需要聚合实现Serialize）
    ///
    /// 注意：如果聚合包含不支持序列化的组件，应该使用其他快照机制
    pub fn create_snapshot<A: AggregateRoot + Serialize>(
        &self,
        aggregate: &A,
        aggregate_id: &str,
        version: u64,
    ) -> Result<EventId, EventError> {
        // 序列化聚合状态
        let data = bincode::serialize(aggregate)
            .map_err(|e| EventError::SerializationError(e.to_string()))?;

        let mut sequence = safe_lock(&self.sequence_generator, "sequence_generator")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
        *sequence += 1;
        let snapshot_id = EventId::now(*sequence);

        let snapshot = Snapshot {
            id: snapshot_id,
            aggregate_id: aggregate_id.to_string(),
            aggregate_version: version,
            data,
            created_at: snapshot_id.timestamp_ns,
        };

        let mut store = safe_write(&self.snapshot_store, "snapshot_store")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
        store.save_snapshot(snapshot)?;

        Ok(snapshot_id)
    }

    /// 清理旧事件
    fn cleanup_old_events(&self) -> Result<(), EventError> {
        let store = safe_read(&self.event_store, "event_store")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
        let all_events = store.get_all_events();

        if all_events.len() > self.max_history_length {
            let cutoff_sequence = all_events
                .iter()
                .rev()
                .nth(self.max_history_length)
                .map(|e| e.id.sequence)
                .unwrap_or(0);

            let mut store = safe_write(&self.event_store, "event_store")
                .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
            store.delete_events_before(cutoff_sequence);
        }

        Ok(())
    }

    /// 重放聚合事件（获取事件列表，不进行反序列化）
    ///
    /// 注意：实际的事件重放需要事件类型注册表来反序列化事件
    pub fn replay_aggregate_events(
        &self,
        aggregate_id: &str,
        from_version: Option<u64>,
    ) -> Result<Vec<StoredEvent>, EventError> {
        let store = safe_read(&self.event_store, "event_store")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;

        let events = if let Some(from_ver) = from_version {
            store.get_aggregate_events_from_version(aggregate_id, from_ver)
        } else {
            store.get_aggregate_events(aggregate_id)
        };

        Ok(events)
    }

    /// 重放并反序列化聚合事件
    ///
    /// 使用事件注册表将存储的事件反序列化为DomainEvent trait objects
    pub fn replay_and_deserialize_events(
        &self,
        aggregate_id: &str,
        from_version: Option<u64>,
    ) -> Result<Vec<Box<dyn DomainEvent>>, EventError> {
        let stored_events = self.replay_aggregate_events(aggregate_id, from_version)?;
        let registry = safe_read(&self.event_registry, "event_registry")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;

        let mut deserialized_events = Vec::new();
        for stored_event in stored_events {
            if let Ok(event) = registry.deserialize(&stored_event.event_type, &stored_event.data) {
                deserialized_events.push(event);
            } else {
                tracing::warn!(
                    target: "domain",
                    "Failed to deserialize event type '{}' for aggregate '{}'",
                    stored_event.event_type,
                    aggregate_id
                );
            }
        }

        Ok(deserialized_events)
    }

    /// 从快照恢复聚合
    ///
    /// 注意：此方法需要聚合类型实现Deserialize trait
    /// 如果聚合包含不支持序列化的组件（如ECS组件），需要使用其他恢复机制
    pub fn restore_from_snapshot<A: AggregateRoot + for<'de> Deserialize<'de>>(
        &self,
        aggregate_id: &str,
    ) -> Result<(A, u64), EventError> {
        let store = safe_read(&self.snapshot_store, "snapshot_store")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
        let snapshot = store.get_latest_snapshot(aggregate_id)?;

        let aggregate: A = bincode::deserialize(&snapshot.data)
            .map_err(|e| EventError::SerializationError(e.to_string()))?;

        Ok((aggregate, snapshot.aggregate_version))
    }

    /// 获取快照数据（不反序列化）
    ///
    /// 用于需要自定义反序列化逻辑的场景
    pub fn get_snapshot_data(&self, aggregate_id: &str) -> Result<(Vec<u8>, u64), EventError> {
        let store = safe_read(&self.snapshot_store, "snapshot_store")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {}", e)))?;
        let snapshot = store.get_latest_snapshot(aggregate_id)?;
        Ok((snapshot.data, snapshot.aggregate_version))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::events::SceneLoadedEvent;
    use crate::domain::scene::{Scene, SceneId};

    #[test]
    fn test_event_store_save_and_retrieve() {
        let store: Arc<RwLock<Box<dyn EventStore>>> =
            Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store: Arc<RwLock<Box<dyn SnapshotStore>>> =
            Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));
        let manager = EventSourcingManager::new(store.clone(), snapshot_store);

        let event = SceneLoadedEvent {
            scene_id: 1,
            scene_name: "Test Scene".to_string(),
        };

        let event_id = manager.save_event(&event, Some("Scene_1"), 1, &World::default()).unwrap();

        let stored = safe_read(&store, "event_store").unwrap().get_event(event_id).unwrap();

        assert_eq!(stored.event_type, "SceneLoaded");
        assert_eq!(stored.aggregate_id, Some("Scene_1".to_string()));
        assert_eq!(stored.aggregate_version, 1);
    }

    #[test]
    fn test_commit_aggregate_events() {
        let store: Arc<RwLock<Box<dyn EventStore>>> =
            Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store: Arc<RwLock<Box<dyn SnapshotStore>>> =
            Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));
        let manager = EventSourcingManager::new(store.clone(), snapshot_store);

        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap(); // 这会添加SceneLoadedEvent

        let event_id = manager.commit_aggregate_events(&mut scene, &mut World::default()).unwrap();

        // 验证事件已保存
        let stored = safe_read(&store, "event_store").unwrap().get_event(event_id).unwrap();
        assert_eq!(stored.event_type, "SceneLoaded");

        // 验证事件已清除
        assert_eq!(scene.uncommitted_event_count(), 0);
    }

    #[test]
    fn test_replay_aggregate_events() {
        let store: Arc<RwLock<Box<dyn EventStore>>> =
            Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store: Arc<RwLock<Box<dyn SnapshotStore>>> =
            Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));
        let manager = EventSourcingManager::new(store.clone(), snapshot_store);

        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().unwrap();
        scene.activate().unwrap();

        manager.commit_aggregate_events(&mut scene, &mut World::default()).unwrap();

        let events = manager.replay_aggregate_events("Scene_1", None).unwrap();

        assert_eq!(events.len(), 2); // SceneLoaded + SceneActivated
    }
}
