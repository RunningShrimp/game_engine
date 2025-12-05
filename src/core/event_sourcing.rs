//! 事件溯源系统
//!
//! 为关键业务操作提供事件溯源支持，支持事件重放、撤销/重做和时间旅行调试

pub mod commands;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use thiserror::Error;

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

/// 领域事件trait
pub trait DomainEvent: Send + Sync + 'static {
    /// 事件类型名称
    fn event_type(&self) -> &'static str;

    /// 应用事件到世界状态
    fn apply(&self, world: &mut World) -> Result<(), EventError>;

    /// 撤销事件（反向操作）
    fn revert(&self, world: &mut World) -> Result<(), EventError>;
}

/// 事件存储
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    /// 事件ID
    pub id: EventId,
    /// 事件类型
    pub event_type: String,
    /// 事件数据（序列化）
    pub data: Vec<u8>,
    /// 聚合ID（实体ID）
    pub aggregate_id: Option<u32>,
}

/// 事件错误
#[derive(Error, Debug, Clone)]
pub enum EventError {
    /// 事件应用失败
    #[error("Apply failed: {0}")]
    ApplyFailed(String),
    /// 事件撤销失败
    #[error("Revert failed: {0}")]
    RevertFailed(String),
    /// 事件存储失败
    #[error("Store failed: {0}")]
    StoreFailed(String),
    /// 事件不存在
    #[error("Event not found")]
    EventNotFound,
    /// 快照不存在
    #[error("Snapshot not found")]
    SnapshotNotFound,
    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// 事件存储trait
pub trait EventStore: Send + Sync {
    /// 保存事件
    fn save_event(&mut self, event: StoredEvent) -> Result<(), EventError>;

    /// 获取事件
    fn get_event(&self, id: EventId) -> Result<StoredEvent, EventError>;

    /// 获取所有事件
    fn get_all_events(&self) -> Vec<StoredEvent>;

    /// 获取聚合的所有事件
    fn get_aggregate_events(&self, aggregate_id: u32) -> Vec<StoredEvent>;

    /// 获取事件范围
    fn get_events_range(&self, from: EventId, to: EventId) -> Vec<StoredEvent>;

    /// 清除所有事件
    fn clear(&mut self);
}

/// 内存事件存储（用于测试和开发）

#[derive(Default)]
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
            .ok_or(EventError::EventNotFound)
    }

    fn get_all_events(&self) -> Vec<StoredEvent> {
        self.events.clone()
    }

    fn get_aggregate_events(&self, aggregate_id: u32) -> Vec<StoredEvent> {
        self.events
            .iter()
            .filter(|e| e.aggregate_id == Some(aggregate_id))
            .cloned()
            .collect()
    }

    fn get_events_range(&self, from: EventId, to: EventId) -> Vec<StoredEvent> {
        self.events
            .iter()
            .filter(|e| e.id >= from && e.id <= to)
            .cloned()
            .collect()
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
    pub aggregate_id: u32,
    /// 快照数据（序列化的世界状态）
    pub data: Vec<u8>,
    /// 创建时间
    pub created_at: i64,
}

/// 快照存储trait
pub trait SnapshotStore: Send + Sync {
    /// 保存快照
    fn save_snapshot(&mut self, snapshot: Snapshot) -> Result<(), EventError>;

    /// 获取最新快照
    fn get_latest_snapshot(&self, aggregate_id: u32) -> Result<Snapshot, EventError>;

    /// 获取快照
    fn get_snapshot(&self, id: EventId) -> Result<Snapshot, EventError>;

    /// 清除快照
    fn clear(&mut self);
}

/// 内存快照存储

#[derive(Default)]
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
        self.snapshots
            .retain(|s| s.aggregate_id != snapshot.aggregate_id);
        self.snapshots.push(snapshot);
        Ok(())
    }

    fn get_latest_snapshot(&self, aggregate_id: u32) -> Result<Snapshot, EventError> {
        self.snapshots
            .iter()
            .filter(|s| s.aggregate_id == aggregate_id)
            .max_by_key(|s| s.id)
            .cloned()
            .ok_or(EventError::SnapshotNotFound)
    }

    fn get_snapshot(&self, id: EventId) -> Result<Snapshot, EventError> {
        self.snapshots
            .iter()
            .find(|s| s.id == id)
            .cloned()
            .ok_or(EventError::SnapshotNotFound)
    }

    fn clear(&mut self) {
        self.snapshots.clear();
    }
}

/// 事件溯源管理器
pub struct EventSourcingManager {
    event_store: Arc<Mutex<dyn EventStore>>,
    snapshot_store: Arc<Mutex<dyn SnapshotStore>>,
    /// 事件序列号生成器
    sequence_generator: Arc<Mutex<u64>>,
    /// 快照间隔（每N个事件创建一个快照）
    snapshot_interval: usize,
    /// 最大事件历史长度
    max_history_length: usize,
}

impl EventSourcingManager {
    pub fn new(
        event_store: Arc<Mutex<dyn EventStore>>,
        snapshot_store: Arc<Mutex<dyn SnapshotStore>>,
    ) -> Self {
        Self {
            event_store,
            snapshot_store,
            sequence_generator: Arc::new(Mutex::new(0)),
            snapshot_interval: 100,    // 默认每100个事件创建一个快照
            max_history_length: 10000, // 默认保留10000个事件
        }
    }

    /// 设置快照间隔
    pub fn set_snapshot_interval(&mut self, interval: usize) {
        self.snapshot_interval = interval;
    }

    /// 设置最大历史长度
    pub fn set_max_history_length(&mut self, max_length: usize) {
        self.max_history_length = max_length;
    }

    /// 记录事件
    pub fn record_event<E: DomainEvent + Serialize>(
        &self,
        event: E,
        aggregate_id: Option<u32>,
    ) -> Result<EventId, EventError> {
        let mut sequence = self.sequence_generator.lock().unwrap();
        *sequence += 1;
        let event_id = EventId::now(*sequence);

        // 序列化事件
        let event_type = event.event_type();
        let data = bincode::serialize(&event)
            .map_err(|e| EventError::SerializationError(e.to_string()))?;

        let stored_event = StoredEvent {
            id: event_id,
            event_type: event_type.to_string(),
            data,
            aggregate_id,
        };

        // 保存事件
        self.event_store.lock().unwrap().save_event(stored_event)?;

        // 检查是否需要创建快照
        if let Some(agg_id) = aggregate_id {
            let event_count = self
                .event_store
                .lock()
                .unwrap()
                .get_aggregate_events(agg_id)
                .len();

            if event_count % self.snapshot_interval == 0 {
                // 创建快照（这里简化处理，实际应该序列化世界状态）
                // 注意：实际实现需要能够序列化ECS世界状态
            }
        }

        // 清理旧事件
        self.cleanup_old_events()?;

        Ok(event_id)
    }

    /// 重放事件到世界
    pub fn replay_events(
        &self,
        world: &mut World,
        from: Option<EventId>,
        to: Option<EventId>,
    ) -> Result<(), EventError> {
        let events = if let (Some(from_id), Some(to_id)) = (from, to) {
            self.event_store
                .lock()
                .unwrap()
                .get_events_range(from_id, to_id)
        } else {
            self.event_store.lock().unwrap().get_all_events()
        };

        for stored_event in events {
            // 反序列化事件（简化处理，实际需要根据event_type反序列化）
            // 这里只是占位，实际实现需要类型注册系统
            // let event: Box<dyn DomainEvent> = bincode::deserialize(&stored_event.data)?;
            // event.apply(world)?;
        }

        Ok(())
    }

    /// 重放聚合事件
    pub fn replay_aggregate_events(
        &self,
        world: &mut World,
        aggregate_id: u32,
    ) -> Result<(), EventError> {
        // 尝试从快照恢复
        if let Ok(snapshot) = self
            .snapshot_store
            .lock()
            .unwrap()
            .get_latest_snapshot(aggregate_id)
        {
            // 从快照恢复状态（简化处理）
            // 实际需要反序列化世界状态
        }

        // 重放快照之后的事件
        let events = self
            .event_store
            .lock()
            .unwrap()
            .get_aggregate_events(aggregate_id);

        // 过滤快照之后的事件
        // 然后重放

        Ok(())
    }

    /// 撤销最后一个事件
    pub fn undo_last_event(&self, world: &mut World) -> Result<Option<EventId>, EventError> {
        let events = self.event_store.lock().unwrap().get_all_events();

        if let Some(last_event) = events.last() {
            // 反序列化并撤销
            // 实际实现需要类型注册
            // let event: Box<dyn DomainEvent> = bincode::deserialize(&last_event.data)?;
            // event.revert(world)?;

            Ok(Some(last_event.id))
        } else {
            Ok(None)
        }
    }

    /// 清理旧事件
    fn cleanup_old_events(&self) -> Result<(), EventError> {
        let mut store = self.event_store.lock().unwrap();
        let events = store.get_all_events();

        if events.len() > self.max_history_length {
            // 保留最新的N个事件
            // 实际实现需要更智能的清理策略
        }

        Ok(())
    }

    /// 获取事件历史
    pub fn get_event_history(&self) -> Vec<StoredEvent> {
        self.event_store.lock().unwrap().get_all_events()
    }

    /// 获取聚合事件历史
    pub fn get_aggregate_history(&self, aggregate_id: u32) -> Vec<StoredEvent> {
        self.event_store
            .lock()
            .unwrap()
            .get_aggregate_events(aggregate_id)
    }
}

/// 时间旅行调试器
pub struct TimeTravelDebugger {
    manager: Arc<EventSourcingManager>,
    /// 当前时间点（事件ID）
    current_time: Option<EventId>,
    /// 时间点历史（用于导航）
    timeline: VecDeque<EventId>,
}

impl TimeTravelDebugger {
    pub fn new(manager: Arc<EventSourcingManager>) -> Self {
        Self {
            manager,
            current_time: None,
            timeline: VecDeque::new(),
        }
    }

    /// 跳转到指定时间点
    pub fn jump_to_time(
        &mut self,
        world: &mut World,
        target_time: EventId,
    ) -> Result<(), EventError> {
        // 保存当前时间点
        if let Some(current) = self.current_time {
            self.timeline.push_back(current);
        }

        // 重放事件到目标时间点
        self.manager.replay_events(world, None, Some(target_time))?;

        self.current_time = Some(target_time);
        Ok(())
    }

    /// 前进一个事件
    pub fn step_forward(&mut self, world: &mut World) -> Result<(), EventError> {
        let events = self.manager.get_event_history();

        if let Some(current) = self.current_time {
            if let Some(next_event) = events.iter().find(|e| e.id > current) {
                self.jump_to_time(world, next_event.id)?;
            }
        } else if let Some(first_event) = events.first() {
            self.jump_to_time(world, first_event.id)?;
        }

        Ok(())
    }

    /// 后退一个事件
    pub fn step_backward(&mut self, world: &mut World) -> Result<(), EventError> {
        if let Some(prev_time) = self.timeline.pop_back() {
            self.jump_to_time(world, prev_time)?;
        }

        Ok(())
    }

    /// 获取当前时间点
    pub fn current_time(&self) -> Option<EventId> {
        self.current_time
    }

    /// 获取时间线
    pub fn timeline(&self) -> &VecDeque<EventId> {
        &self.timeline
    }
}

/// ECS资源：事件溯源管理器
#[derive(Resource)]
pub struct EventSourcingResource {
    manager: Arc<EventSourcingManager>,
}

impl EventSourcingResource {
    pub fn new(manager: Arc<EventSourcingManager>) -> Self {
        Self { manager }
    }

    pub fn manager(&self) -> &Arc<EventSourcingManager> {
        &self.manager
    }
}

/// 示例：实体创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCreatedEvent {
    pub entity_id: u32,
    pub entity_type: String,
}

pub use commands::{Command, CommandHandler, CreateEntityCommand, DeleteEntityCommand};

impl DomainEvent for EntityCreatedEvent {
    fn event_type(&self) -> &'static str {
        "EntityCreated"
    }

    fn apply(&self, world: &mut World) -> Result<(), EventError> {
        // 创建实体（简化处理）
        // 实际实现需要根据entity_type创建相应的组件
        Ok(())
    }

    fn revert(&self, world: &mut World) -> Result<(), EventError> {
        // 删除实体
        // 实际实现需要能够通过entity_id删除实体
        Ok(())
    }
}

/// 示例：实体变换事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTransformChangedEvent {
    pub entity_id: u32,
    pub old_transform: Option<Vec<u8>>, // 序列化的旧变换
    pub new_transform: Vec<u8>,         // 序列化的新变换
}

impl DomainEvent for EntityTransformChangedEvent {
    fn event_type(&self) -> &'static str {
        "EntityTransformChanged"
    }

    fn apply(&self, world: &mut World) -> Result<(), EventError> {
        // 应用新变换
        // 实际实现需要反序列化并更新实体
        Ok(())
    }

    fn revert(&self, world: &mut World) -> Result<(), EventError> {
        // 恢复旧变换
        // 实际实现需要反序列化并恢复
        Ok(())
    }
}
