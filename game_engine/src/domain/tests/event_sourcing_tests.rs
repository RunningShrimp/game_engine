//  事件溯源测试模块
//
//  提供对事件溯源系统的全面测试覆盖，包括：
//  - 事件存储和检索
//  - 快照管理
//  - 事件重放和时间旅行
//  - 事件查询和过滤
//  - 事件投影
//  - 聚合根集成

use crate::domain::event_sourcing::*;
use crate::domain::events::{DomainEvent, EventError, SceneLoadedEvent, SceneActivatedEvent};
use crate::domain::scene::{Scene, SceneId};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};

/// 测试聚合根：计数器聚合
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterAggregate {
    id: String,
    count: i32,
    events: Vec<Box<dyn DomainEvent>>,
}

impl CounterAggregate {
    pub fn new(id: String) -> Self {
        Self {
            id,
            count: 0,
            events: Vec::new(),
        }
    }

    pub fn increment(&mut self) {
        self.count += 1;
        // 注意：实际实现中应该添加领域事件
    }

    pub fn decrement(&mut self) {
        self.count -= 1;
        // 注意：实际实现中应该添加领域事件
    }

    pub fn count(&self) -> i32 {
        self.count
    }
}

/// 计数器增加事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterIncrementedEvent {
    pub aggregate_id: String,
    pub new_value: i32,
}

impl DomainEvent for CounterIncrementedEvent {
    fn event_type(&self) -> &'static str {
        "CounterIncremented"
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// 计数器减少事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterDecrementedEvent {
    pub aggregate_id: String,
    pub new_value: i32,
}

impl DomainEvent for CounterDecrementedEvent {
    fn event_type(&self) -> &'static str {
        "CounterDecremented"
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod event_id_tests {
    use super::*;

    #[test]
    fn test_event_id_new() {
        let event_id = EventId::new(1234567890, 1);

        assert_eq!(event_id.timestamp_ns, 1234567890);
        assert_eq!(event_id.sequence, 1);
    }

    #[test]
    fn test_event_id_now() {
        let event_id = EventId::now(1);

        assert!(event_id.timestamp_ns > 0);
        assert_eq!(event_id.sequence, 1);
    }

    #[test]
    fn test_event_id_ordering() {
        let id1 = EventId::new(1000, 1);
        let id2 = EventId::new(1000, 2);
        let id3 = EventId::new(2000, 1);

        assert!(id1 < id2); // 同一时间戳，序列号大的更大
        assert!(id2 < id3); // 不同时间戳
    }

    #[test]
    fn test_event_id_equality() {
        let id1 = EventId::new(1000, 1);
        let id2 = EventId::new(1000, 1);
        let id3 = EventId::new(1000, 2);

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }
}

#[cfg(test)]
mod stored_event_tests {
    use super::*;

    #[test]
    fn test_stored_event_creation() {
        let event_id = EventId::now(1);
        let stored_event = StoredEvent {
            id: event_id,
            event_type: "TestEvent".to_string(),
            data: vec![1, 2, 3, 4],
            aggregate_id: Some("aggregate_1".to_string()),
            aggregate_version: 1,
        };

        assert_eq!(stored_event.id, event_id);
        assert_eq!(stored_event.event_type, "TestEvent");
        assert_eq!(stored_event.data, vec![1, 2, 3, 4]);
        assert_eq!(stored_event.aggregate_id, Some("aggregate_1".to_string()));
        assert_eq!(stored_event.aggregate_version, 1);
    }
}

#[cfg(test)]
mod memory_event_store_tests {
    use super::*;

    #[test]
    fn test_memory_event_store_new() {
        let store = MemoryEventStore::new();

        assert_eq!(store.events.len(), 0);
        assert_eq!(store.next_sequence, 0);
    }

    #[test]
    fn test_memory_event_store_default() {
        let store = MemoryEventStore::default();

        assert_eq!(store.events.len(), 0);
    }

    #[test]
    fn test_memory_event_store_save_and_get() {
        let mut store = MemoryEventStore::new();

        let event_id = EventId::now(1);
        let event = StoredEvent {
            id: event_id,
            event_type: "TestEvent".to_string(),
            data: vec![],
            aggregate_id: Some("agg_1".to_string()),
            aggregate_version: 1,
        };

        store.save_event(event.clone()).expect("Test: operation should succeed");

        let retrieved = store.get_event(event_id).expect("Test: operation should succeed");
        assert_eq!(retrieved.id, event_id);
        assert_eq!(retrieved.event_type, "TestEvent");
    }

    #[test]
    fn test_memory_event_store_get_all() {
        let mut store = MemoryEventStore::new();

        for i in 1..=3 {
            let event = StoredEvent {
                id: EventId::new(1000, i),
                event_type: format!("Event{}", i),
                data: vec![],
                aggregate_id: Some("agg_1".to_string()),
                aggregate_version: i,
            };
            store.save_event(event).expect("Test: operation should succeed");
        }

        let all_events = store.get_all_events();
        assert_eq!(all_events.len(), 3);
    }

    #[test]
    fn test_memory_event_store_get_aggregate_events() {
        let mut store = MemoryEventStore::new();

        // 添加多个聚合的事件
        for i in 1..=3 {
            let event = StoredEvent {
                id: EventId::new(1000, i),
                event_type: "Event".to_string(),
                data: vec![],
                aggregate_id: Some("agg_1".to_string()),
                aggregate_version: i,
            };
            store.save_event(event).expect("Test: operation should succeed");
        }

        for i in 4..=6 {
            let event = StoredEvent {
                id: EventId::new(1000, i),
                event_type: "Event".to_string(),
                data: vec![],
                aggregate_id: Some("agg_2".to_string()),
                aggregate_version: i - 3,
            };
            store.save_event(event).expect("Test: operation should succeed");
        }

        let agg1_events = store.get_aggregate_events("agg_1");
        assert_eq!(agg1_events.len(), 3);

        let agg2_events = store.get_aggregate_events("agg_2");
        assert_eq!(agg2_events.len(), 3);
    }

    #[test]
    fn test_memory_event_store_get_events_range() {
        let mut store = MemoryEventStore::new();

        for i in 1..=10 {
            let event = StoredEvent {
                id: EventId::new(1000 + i as i64, i),
                event_type: "Event".to_string(),
                data: vec![],
                aggregate_id: None,
                aggregate_version: i,
            };
            store.save_event(event).expect("Test: operation should succeed");
        }

        let from = EventId::new(1003, 3);
        let to = EventId::new(1007, 7);
        let range_events = store.get_events_range(from, to);

        assert_eq!(range_events.len(), 5);
    }

    #[test]
    fn test_memory_event_store_get_aggregate_events_from_version() {
        let mut store = MemoryEventStore::new();

        for i in 1..=10 {
            let event = StoredEvent {
                id: EventId::new(1000, i),
                event_type: "Event".to_string(),
                data: vec![],
                aggregate_id: Some("agg_1".to_string()),
                aggregate_version: i,
            };
            store.save_event(event).expect("Test: operation should succeed");
        }

        let from_version = 5;
        let events = store.get_aggregate_events_from_version("agg_1", from_version);

        assert_eq!(events.len(), 6);
        for event in &events {
            assert!(event.aggregate_version >= from_version);
        }
    }

    #[test]
    fn test_memory_event_store_delete_events_before() {
        let mut store = MemoryEventStore::new();

        for i in 1..=10 {
            let event = StoredEvent {
                id: EventId::new(1000, i),
                event_type: "Event".to_string(),
                data: vec![],
                aggregate_id: None,
                aggregate_version: i,
            };
            store.save_event(event).expect("Test: operation should succeed");
        }

        store.delete_events_before(5);

        let remaining = store.get_all_events();
        assert_eq!(remaining.len(), 6);
    }

    #[test]
    fn test_memory_event_store_clear() {
        let mut store = MemoryEventStore::new();

        let event = StoredEvent {
            id: EventId::now(1),
            event_type: "Event".to_string(),
            data: vec![],
            aggregate_id: None,
            aggregate_version: 1,
        };
        store.save_event(event).expect("Test: operation should succeed");

        store.clear();

        assert_eq!(store.events.len(), 0);
        assert_eq!(store.next_sequence, 0);
    }
}

#[cfg(test)]
mod snapshot_tests {
    use super::*;

    #[test]
    fn test_snapshot_creation() {
        let snapshot = Snapshot {
            id: EventId::now(1),
            aggregate_id: "aggregate_1".to_string(),
            aggregate_version: 10,
            data: vec![1, 2, 3, 4],
            created_at: 1234567890,
        };

        assert_eq!(snapshot.aggregate_id, "aggregate_1");
        assert_eq!(snapshot.aggregate_version, 10);
        assert_eq!(snapshot.data, vec![1, 2, 3, 4]);
    }
}

#[cfg(test)]
mod memory_snapshot_store_tests {
    use super::*;

    #[test]
    fn test_memory_snapshot_store_new() {
        let store = MemorySnapshotStore::new();

        assert_eq!(store.snapshots.len(), 0);
    }

    #[test]
    fn test_memory_snapshot_store_save_and_get() {
        let mut store = MemorySnapshotStore::new();

        let snapshot = Snapshot {
            id: EventId::now(1),
            aggregate_id: "agg_1".to_string(),
            aggregate_version: 10,
            data: vec![1, 2, 3],
            created_at: 1234567890,
        };

        store.save_snapshot(snapshot.clone()).expect("Test: operation should succeed");

        let retrieved = store.get_snapshot(snapshot.id).expect("Test: operation should succeed");
        assert_eq!(retrieved.aggregate_id, "agg_1");
        assert_eq!(retrieved.aggregate_version, 10);
    }

    #[test]
    fn test_memory_snapshot_store_get_latest() {
        let mut store = MemorySnapshotStore::new();

        // 保存多个快照
        for i in 1..=3 {
            let snapshot = Snapshot {
                id: EventId::new(1000, i),
                aggregate_id: "agg_1".to_string(),
                aggregate_version: i * 10,
                data: vec![i],
                created_at: 1000 + i as i64,
            };
            store.save_snapshot(snapshot).expect("Test: operation should succeed");
        }

        let latest = store.get_latest_snapshot("agg_1").expect("Test: operation should succeed");
        assert_eq!(latest.aggregate_version, 30);
    }

    #[test]
    fn test_memory_snapshot_store_get_aggregate_snapshots() {
        let mut store = MemorySnapshotStore::new();

        // 为不同聚合添加快照
        for i in 1..=3 {
            let snapshot = Snapshot {
                id: EventId::new(1000, i),
                aggregate_id: "agg_1".to_string(),
                aggregate_version: i,
                data: vec![],
                created_at: 1000,
            };
            store.save_snapshot(snapshot).expect("Test: operation should succeed");
        }

        let snapshots = store.get_aggregate_snapshots("agg_1");
        assert_eq!(snapshots.len(), 1); // 只保留最新的
    }

    #[test]
    fn test_memory_snapshot_store_clear() {
        let mut store = MemorySnapshotStore::new();

        let snapshot = Snapshot {
            id: EventId::now(1),
            aggregate_id: "agg_1".to_string(),
            aggregate_version: 1,
            data: vec![],
            created_at: 1000,
        };
        store.save_snapshot(snapshot).expect("Test: operation should succeed");

        store.clear();

        assert_eq!(store.snapshots.len(), 0);
    }
}

#[cfg(test)]
mod event_sourcing_manager_tests {
    use super::*;

    #[test]
    fn test_event_sourcing_manager_new() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let manager = EventSourcingManager::new(event_store, snapshot_store);

        assert_eq!(manager.snapshot_interval, 100);
        assert_eq!(manager.max_history_length, 10000);
    }

    #[test]
    fn test_event_sourcing_manager_set_snapshot_interval() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let mut manager = EventSourcingManager::new(event_store, snapshot_store);
        manager.set_snapshot_interval(50);

        assert_eq!(manager.snapshot_interval, 50);
    }

    #[test]
    fn test_event_sourcing_manager_set_max_history_length() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let mut manager = EventSourcingManager::new(event_store, snapshot_store);
        manager.set_max_history_length(5000);

        assert_eq!(manager.max_history_length, 5000);
    }

    #[test]
    fn test_event_sourcing_manager_save_event() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let manager = EventSourcingManager::new(event_store.clone(), snapshot_store);

        let event = SceneLoadedEvent {
            scene_id: 1,
            scene_name: "Test Scene".to_string(),
        };

        let world = World::new();
        let event_id = manager
            .save_event(&event, Some("Scene_1"), 1, &world)
            .expect("Test: operation should succeed");

        let stored = event_store.read().expect("Test: operation should succeed").get_event(event_id).expect("Test: operation should succeed");
        assert_eq!(stored.event_type, "SceneLoaded");
        assert_eq!(stored.aggregate_id, Some("Scene_1".to_string()));
    }

    #[test]
    fn test_event_sourcing_manager_save_multiple_events() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let manager = EventSourcingManager::new(event_store.clone(), snapshot_store);

        let world = World::new();

        for i in 1..=5 {
            let event = SceneLoadedEvent {
                scene_id: i,
                scene_name: format!("Scene {}", i),
            };

            manager
                .save_event(&event, Some(&format!("Scene_{}", i)), i, &world)
                .expect("Test: operation should succeed");
        }

        let all_events = event_store.read().expect("Test: operation should succeed").get_all_events();
        assert_eq!(all_events.len(), 5);
    }

    #[test]
    fn test_event_sourcing_manager_replay_aggregate_events() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let manager = EventSourcingManager::new(event_store.clone(), snapshot_store);

        let world = World::new();

        // 保存多个事件
        for i in 1..=5 {
            let event = SceneLoadedEvent {
                scene_id: 1,
                scene_name: "Test Scene".to_string(),
            };

            manager
                .save_event(&event, Some("Scene_1"), i, &world)
                .expect("Test: operation should succeed");
        }

        let events = manager.replay_aggregate_events("Scene_1", None).expect("Test: operation should succeed");
        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_event_sourcing_manager_replay_from_version() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let manager = EventSourcingManager::new(event_store.clone(), snapshot_store);

        let world = World::new();

        for i in 1..=10 {
            let event = SceneLoadedEvent {
                scene_id: 1,
                scene_name: "Test Scene".to_string(),
            };

            manager
                .save_event(&event, Some("Scene_1"), i, &world)
                .expect("Test: operation should succeed");
        }

        let events = manager
            .replay_aggregate_events("Scene_1", Some(5))
            .expect("Test: operation should succeed");

        assert_eq!(events.len(), 6); // 版本5-10
    }
}

#[cfg(test)]
mod event_query_tests {
    use super::*;

    #[test]
    fn test_event_query_all() {
        let query = EventQuery::all();

        assert!(query.aggregate_id.is_none());
        assert!(query.event_type.is_none());
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_event_query_by_aggregate() {
        let query = EventQuery::by_aggregate("Scene_1");

        assert_eq!(query.aggregate_id, Some("Scene_1".to_string()));
    }

    #[test]
    fn test_event_query_by_event_type() {
        let query = EventQuery::by_event_type("SceneLoaded");

        assert_eq!(query.event_type, Some("SceneLoaded".to_string()));
    }

    #[test]
    fn test_event_query_by_time_range() {
        let query = EventQuery::by_time_range(1000, 2000);

        assert_eq!(query.from_time, Some(1000));
        assert_eq!(query.to_time, Some(2000));
    }

    #[test]
    fn test_event_query_by_version_range() {
        let query = EventQuery::by_version_range(5, 10);

        assert_eq!(query.from_version, Some(5));
        assert_eq!(query.to_version, Some(10));
    }

    #[test]
    fn test_event_query_with_limit() {
        let query = EventQuery::all().with_limit(10);

        assert_eq!(query.limit, Some(10));
    }

    #[test]
    fn test_event_query_with_offset() {
        let query = EventQuery::all().with_offset(5);

        assert_eq!(query.offset, Some(5));
    }

    #[test]
    fn test_event_query_chain() {
        let query = EventQuery::by_aggregate("Scene_1")
            .with_limit(10)
            .with_offset(5);

        assert_eq!(query.aggregate_id, Some("Scene_1".to_string()));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(5));
    }
}

#[cfg(test)]
mod event_stats_tests {
    use super::*;

    #[test]
    fn test_event_stats_creation() {
        let stats = EventStats {
            total_events: 10,
            events_by_type: std::collections::HashMap::new(),
            events_by_aggregate: std::collections::HashMap::new(),
            oldest_event_time: Some(1000),
            newest_event_time: Some(2000),
        };

        assert_eq!(stats.total_events, 10);
        assert_eq!(stats.oldest_event_time, Some(1000));
        assert_eq!(stats.newest_event_time, Some(2000));
    }
}

#[cfg(test)]
mod event_stream_processor_tests {
    use super::*;

    #[test]
    fn test_event_stream_processor_new() {
        let processor = EventStreamProcessor::new();

        assert_eq!(processor.filters.len(), 0);
        assert_eq!(processor.transformers.len(), 0);
    }

    #[test]
    fn test_event_stream_processor_add_filter() {
        let mut processor = EventStreamProcessor::new();

        processor.add_filter(|e| e.aggregate_version > 5);

        assert_eq!(processor.filters.len(), 1);
    }

    #[test]
    fn test_event_stream_processor_add_transformer() {
        let mut processor = EventStreamProcessor::new();

        processor.add_transformer(|e| e);

        assert_eq!(processor.transformers.len(), 1);
    }

    #[test]
    fn test_event_stream_processor_process_with_filter() {
        let mut processor = EventStreamProcessor::new();

        // 只保留版本大于5的事件
        processor.add_filter(|e| e.aggregate_version > 5);

        let events = vec![
            StoredEvent {
                id: EventId::now(1),
                event_type: "Event".to_string(),
                data: vec![],
                aggregate_id: Some("agg_1".to_string()),
                aggregate_version: 3,
            },
            StoredEvent {
                id: EventId::now(2),
                event_type: "Event".to_string(),
                data: vec![],
                aggregate_id: Some("agg_1".to_string()),
                aggregate_version: 7,
            },
        ];

        let filtered = processor.process(events);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].aggregate_version, 7);
    }
}

#[cfg(test)]
mod event_sourcing_integration_tests {
    use super::*;

    #[test]
    fn test_scene_aggregate_event_commit() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let manager = EventSourcingManager::new(event_store.clone(), snapshot_store);

        let mut scene = Scene::new(SceneId(1), "Test Scene");
        scene.load().expect("Test: operation should succeed");
        scene.activate().expect("Test: operation should succeed");

        let mut world = World::new();
        let event_id = manager
            .commit_aggregate_events(&mut scene, &mut world)
            .expect("Test: operation should succeed");

        // 验证事件已保存
        let stored = event_store.read().expect("Test: operation should succeed").get_event(event_id).expect("Test: operation should succeed");
        assert_eq!(stored.event_type, "SceneLoaded");

        // 验证场景的未提交事件已清除
        assert_eq!(scene.uncommitted_event_count(), 0);
    }

    #[test]
    fn test_multiple_scenes_event_tracking() {
        let event_store = Arc::new(RwLock::new(Box::new(MemoryEventStore::new())));
        let snapshot_store = Arc::new(RwLock::new(Box::new(MemorySnapshotStore::new())));

        let manager = EventSourcingManager::new(event_store.clone(), snapshot_store);

        let mut world = World::new();

        // 创建多个场景
        for i in 1..=3 {
            let mut scene = Scene::new(SceneId(i), format!("Scene {}", i));
            scene.load().expect("Test: operation should succeed");
            manager
                .commit_aggregate_events(&mut scene, &mut world)
                .expect("Test: operation should succeed");
        }

        let all_events = event_store.read().expect("Test: operation should succeed").get_all_events();
        assert_eq!(all_events.len(), 3);
    }
}
