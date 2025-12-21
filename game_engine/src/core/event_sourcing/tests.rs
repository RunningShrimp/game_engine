//  事件溯源系统测试

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manager() -> Arc<EventSourcingManager> {
        let event_store = Arc::new(Mutex::new(MemoryEventStore::new()));
        let snapshot_store = Arc::new(Mutex::new(MemorySnapshotStore::new()));
        Arc::new(EventSourcingManager::new(event_store, snapshot_store))
    }

    #[test]
    fn test_event_id_creation() {
        let event_id = EventId::now(1);
        assert!(event_id.timestamp_ns > 0);
        assert_eq!(event_id.sequence, 1);
    }

    #[test]
    fn test_entity_created_event() {
        let event = EntityCreatedEvent {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
        };

        assert_eq!(event.event_type(), "EntityCreated");
    }

    #[test]
    fn test_entity_deleted_event() {
        let event = EntityDeletedEvent {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
        };

        assert_eq!(event.event_type(), "EntityDeleted");
    }

    #[test]
    fn test_entity_updated_event() {
        let event = EntityUpdatedEvent {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
            old_data: vec![1, 2, 3],
            new_data: vec![4, 5, 6],
        };

        assert_eq!(event.event_type(), "EntityUpdated");
    }

    #[test]
    fn test_create_entity_command() {
        let command = CreateEntityCommand {
            entity_type: "TestEntity".to_string(),
            initial_data: vec![1, 2, 3],
        };

        assert_eq!(command.command_type(), "CreateEntity");
    }

    #[test]
    fn test_delete_entity_command() {
        let command = DeleteEntityCommand {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
        };

        assert_eq!(command.command_type(), "DeleteEntity");
    }

    #[test]
    fn test_update_entity_command() {
        let command = UpdateEntityCommand {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
            old_data: vec![1, 2, 3],
            new_data: vec![4, 5, 6],
        };

        assert_eq!(command.command_type(), "UpdateEntity");
    }

    #[test]
    fn test_memory_event_store() {
        let mut store = MemoryEventStore::new();

        let event = StoredEvent {
            id: EventId::now(1),
            event_type: "TestEvent".to_string(),
            data: vec![1, 2, 3],
            aggregate_id: Some(123),
        };

        store.save_event(event.clone()).unwrap();

        let retrieved = store.get_event(event.id).unwrap();
        assert_eq!(retrieved.event_type, "TestEvent");
        assert_eq!(retrieved.aggregate_id, Some(123));
    }

    #[test]
    fn test_memory_snapshot_store() {
        let mut store = MemorySnapshotStore::new();

        let snapshot = Snapshot {
            id: EventId::now(1),
            aggregate_id: 123,
            data: vec![1, 2, 3],
            created_at: 1234567890,
        };

        store.save_snapshot(snapshot.clone()).unwrap();

        let retrieved = store.get_latest_snapshot(123).unwrap();
        assert_eq!(retrieved.aggregate_id, 123);
        assert_eq!(retrieved.created_at, 1234567890);
    }

    #[test]
    fn test_event_recording() {
        let manager = create_test_manager();
        let mut world = World::new();

        let event = EntityCreatedEvent {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
        };

        let event_id = manager.record_event(event, &world, Some(123)).unwrap();

        let history = manager.get_event_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, event_id);
    }

    #[test]
    fn test_aggregate_history() {
        let manager = create_test_manager();
        let mut world = World::new();

        let event1 = EntityCreatedEvent {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
        };

        let event2 = EntityCreatedEvent {
            entity_id: 456,
            entity_type: "TestEntity".to_string(),
        };

        manager.record_event(event1, &world, Some(123)).unwrap();
        manager.record_event(event2, &world, Some(456)).unwrap();

        let history = manager.get_aggregate_history(123);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].aggregate_id, Some(123));
    }

    #[test]
    fn test_event_registry() {
        let mut registry = super::registry::EventTypeRegistry::new();

        // 注册事件类型
        registry
            .register_event_type::<EntityCreatedEvent>()
            .unwrap();

        // 测试创建事件
        let event = EntityCreatedEvent {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
        };

        let serialized = bincode::serialize(&event).unwrap();
        let created_event = registry.create_event("EntityCreated", &serialized).unwrap();

        assert_eq!(created_event.event_type(), "EntityCreated");
    }

    #[test]
    fn test_command_handler() {
        let manager = create_test_manager();
        let handler = CommandHandler::new(manager.clone());
        let mut world = World::new();

        let command = CreateEntityCommand {
            entity_type: "TestEntity".to_string(),
            initial_data: vec![1, 2, 3],
        };

        let event_id = handler
            .execute_command(command, &mut world, Some(123))
            .unwrap();

        let history = manager.get_event_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].id, event_id);
    }

    #[test]
    fn test_snapshot_creation() {
        let manager = create_test_manager();
        let mut world = World::new();

        // 设置快照间隔为1，确保每个事件都创建快照
        let mut manager_mut = Arc::try_unwrap(manager).unwrap();
        manager_mut.set_snapshot_interval(1);
        let manager = Arc::new(manager_mut);

        let event = EntityCreatedEvent {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
        };

        let mut world = World::new();
        manager.record_event(event, &world, Some(123)).unwrap();

        // 验证快照已创建
        let snapshots = manager.get_aggregate_snapshots(123);
        assert_eq!(snapshots.len(), 1);
    }

    #[test]
    fn test_time_travel_debugger() {
        let manager = create_test_manager();
        let mut debugger = TimeTravelDebugger::new(manager.clone());
        let mut world = World::new();

        let event = EntityCreatedEvent {
            entity_id: 123,
            entity_type: "TestEntity".to_string(),
        };

        let event_id = manager.record_event(event, &world, Some(123)).unwrap();

        // 测试跳转到时间点
        debugger.jump_to_time(&mut world, event_id).unwrap();
        assert_eq!(debugger.current_time(), Some(event_id));
    }
}
