//  CQRS模式测试模块
//
//  提供对CQRS（命令查询职责分离）实现的全面测试覆盖，包括：
//  - 命令处理器和命令总线
//  - 查询处理器和查询总线
//  - CQRS管理器集成
//  - 错误处理和边界情况
//  - 与事件溯源的集成

use crate::domain::cqrs::*;
use crate::domain::event_sourcing::{EventId, EventSourcingManager, MemoryEventStore, MemorySnapshotStore};
use crate::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// 测试命令：创建实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityCommand {
    pub entity_id: u64,
    pub entity_type: String,
}

impl Command for CreateEntityCommand {
    fn command_type(&self) -> &'static str {
        "CreateEntity"
    }
}

/// 测试命令：移动实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveEntityCommand {
    pub entity_id: u64,
    pub position: [f32; 3],
}

impl Command for MoveEntityCommand {
    fn command_type(&self) -> &'static str {
        "MoveEntity"
    }
}

/// 测试命令：删除实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteEntityCommand {
    pub entity_id: u64,
}

impl Command for DeleteEntityCommand {
    fn command_type(&self) -> &'static str {
        "DeleteEntity"
    }
}

/// 实体创建事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCreatedEvent {
    pub entity_id: u64,
    pub entity_type: String,
}

impl DomainEvent for EntityCreatedEvent {
    fn event_type(&self) -> &'static str {
        "EntityCreated"
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

/// 实体移动事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMovedEvent {
    pub entity_id: u64,
    pub old_position: [f32; 3],
    pub new_position: [f32; 3],
}

impl DomainEvent for EntityMovedEvent {
    fn event_type(&self) -> &'static str {
        "EntityMoved"
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

/// 测试命令处理器：实体创建处理器
pub struct CreateEntityHandler {
    event_ids: Arc<Mutex<Vec<EventId>>>,
}

impl CreateEntityHandler {
    pub fn new() -> Self {
        Self {
            event_ids: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_event_ids(&self) -> Vec<EventId> {
        self.event_ids.lock().expect("Test: operation should succeed").clone()
    }
}

impl CommandHandler<CreateEntityCommand> for CreateEntityHandler {
    fn handle(
        &self,
        command: CreateEntityCommand,
        _world: &mut World,
    ) -> Result<CommandResult, EventError> {
        // 模拟实体创建逻辑
        let event_id = EventId::now(1);
        self.event_ids.lock().expect("Test: operation should succeed").push(event_id);

        Ok(CommandResult::success(Some(event_id)))
    }
}

/// 测试命令处理器：实体移动处理器
pub struct MoveEntityHandler {
    positions: Arc<Mutex<std::collections::HashMap<u64, [f32; 3]>>>,
}

impl MoveEntityHandler {
    pub fn new() -> Self {
        Self {
            positions: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn get_position(&self, entity_id: u64) -> Option<[f32; 3]> {
        self.positions.lock().expect("Test: operation should succeed").get(&entity_id).copied()
    }
}

impl CommandHandler<MoveEntityCommand> for MoveEntityHandler {
    fn handle(
        &self,
        command: MoveEntityCommand,
        _world: &mut World,
    ) -> Result<CommandResult, EventError> {
        // 更新实体位置
        self.positions
            .lock()
            .expect("Test: operation should succeed")
            .insert(command.entity_id, command.position);

        Ok(CommandResult::success(None))
    }
}

/// 测试查询：获取实体位置
#[derive(Debug, Clone)]
pub struct GetEntityPositionQuery {
    pub entity_id: u64,
}

impl Query for GetEntityPositionQuery {
    fn query_type(&self) -> &'static str {
        "GetEntityPosition"
    }
}

/// 测试查询：获取所有实体
#[derive(Debug, Clone)]
pub struct GetAllEntitiesQuery;

impl Query for GetAllEntitiesQuery {
    fn query_type(&self) -> &'static str {
        "GetAllEntities"
    }
}

/// 测试查询处理器：获取实体位置处理器
pub struct GetEntityPositionHandler {
    positions: Arc<Mutex<std::collections::HashMap<u64, [f32; 3]>>>,
}

impl GetEntityPositionHandler {
    pub fn new() -> Self {
        Self {
            positions: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn set_position(&self, entity_id: u64, position: [f32; 3]) {
        self.positions.lock().expect("Test: operation should succeed").insert(entity_id, position);
    }
}

impl QueryHandler<GetEntityPositionQuery> for GetEntityPositionHandler {
    type Result = Option<[f32; 3]>;

    fn handle(
        &self,
        query: GetEntityPositionQuery,
        _world: &World,
    ) -> QueryResult<Self::Result> {
        let positions = self.positions.lock().expect("Test: operation should succeed");
        Ok(positions.get(&query.entity_id).copied())
    }
}

/// 测试查询处理器：获取所有实体处理器
pub struct GetAllEntitiesHandler {
    entities: Arc<Mutex<Vec<u64>>>,
}

impl GetAllEntitiesHandler {
    pub fn new() -> Self {
        Self {
            entities: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn add_entity(&self, entity_id: u64) {
        self.entities.lock().expect("Test: operation should succeed").push(entity_id);
    }
}

impl QueryHandler<GetAllEntitiesQuery> for GetAllEntitiesHandler {
    type Result = Vec<u64>;

    fn handle(&self, _query: GetAllEntitiesQuery, _world: &World) -> QueryResult<Self::Result> {
        let entities = self.entities.lock().expect("Test: operation should succeed");
        Ok(entities.clone())
    }
}

#[cfg(test)]
mod command_result_tests {
    use super::*;

    #[test]
    fn test_command_result_success() {
        let event_id = EventId::now(1);
        let result = CommandResult::success(Some(event_id));

        assert!(result.success);
        assert_eq!(result.event_id, Some(event_id));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_command_result_success_without_event() {
        let result = CommandResult::success(None);

        assert!(result.success);
        assert!(result.event_id.is_none());
        assert!(result.error.is_none());
    }

    #[test]
    fn test_command_result_failure() {
        let error_msg = "Entity not found".to_string();
        let result = CommandResult::failure(error_msg.clone());

        assert!(!result.success);
        assert!(result.event_id.is_none());
        assert_eq!(result.error, Some(error_msg));
    }
}

#[cfg(test)]
mod command_bus_tests {
    use super::*;

    #[test]
    fn test_command_bus_register_handler() {
        let bus = CommandBus::new();
        let handler = Arc::new(CreateEntityHandler::new());

        let result = bus.register_handler::<CreateEntityCommand, _>(handler);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_bus_execute_command() {
        let bus = CommandBus::new();
        let handler = Arc::new(CreateEntityHandler::new());
        bus.register_handler::<CreateEntityCommand, _>(handler)
            .expect("Test: operation should succeed");

        let command = CreateEntityCommand {
            entity_id: 1,
            entity_type: "Player".to_string(),
        };

        let mut world = World::new();
        let result = bus.execute(command, &mut world);

        assert!(result.is_ok());
        let result = result.expect("Test: operation should succeed");
        assert!(result.success);
        assert!(result.event_id.is_some());
    }

    #[test]
    fn test_command_bus_execute_without_handler() {
        let bus = CommandBus::new();

        let command = CreateEntityCommand {
            entity_id: 1,
            entity_type: "Player".to_string(),
        };

        let mut world = World::new();
        let result = bus.execute(command, &mut world);

        assert!(result.is_err());
        if let Err(EventError::ApplyFailed(msg)) = result {
            assert!(msg.contains("No handler registered"));
        } else {
            panic!("Expected EventError::ApplyFailed");
        }
    }

    #[test]
    fn test_command_bus_multiple_handlers() {
        let bus = CommandBus::new();

        let create_handler = Arc::new(CreateEntityHandler::new());
        let move_handler = Arc::new(MoveEntityHandler::new());

        bus.register_handler::<CreateEntityCommand, _>(create_handler)
            .expect("Test: operation should succeed");
        bus.register_handler::<MoveEntityCommand, _>(move_handler)
            .expect("Test: operation should succeed");

        // 执行创建命令
        let create_command = CreateEntityCommand {
            entity_id: 1,
            entity_type: "Player".to_string(),
        };

        let mut world = World::new();
        let result = bus.execute(create_command, &mut world);
        assert!(result.is_ok());

        // 执行移动命令
        let move_command = MoveEntityCommand {
            entity_id: 1,
            position: [10.0, 20.0, 30.0],
        };

        let result = bus.execute(move_command, &mut world);
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_bus_default() {
        let bus = CommandBus::default();
        assert_eq!(
            bus.handlers
                .read()
                .expect("Test: operation should succeed")
                .len(),
            0
        );
    }
}

#[cfg(test)]
mod query_bus_tests {
    use super::*;

    #[test]
    fn test_query_bus_register_handler() {
        let bus = QueryBus::new();
        let handler = Arc::new(GetEntityPositionHandler::new());

        let result = bus.register_handler::<GetEntityPositionQuery, _>(handler);
        assert!(result.is_ok());
    }

    #[test]
    fn test_query_bus_execute_query() {
        let bus = QueryBus::new();
        let handler = Arc::new(GetEntityPositionHandler::new());

        // 设置一个实体位置
        handler.set_position(1, [10.0, 20.0, 30.0]);

        bus.register_handler::<GetEntityPositionQuery, _>(handler)
            .expect("Test: operation should succeed");

        let query = GetEntityPositionQuery { entity_id: 1 };
        let world = World::new();

        let result: Option<[f32; 3]> = bus.execute(query, &world).expect("Test: operation should succeed");

        assert_eq!(result, Some([10.0, 20.0, 30.0]));
    }

    #[test]
    fn test_query_bus_execute_not_found() {
        let bus = QueryBus::new();
        let handler = Arc::new(GetEntityPositionHandler::new());

        bus.register_handler::<GetEntityPositionQuery, _>(handler)
            .expect("Test: operation should succeed");

        let query = GetEntityPositionQuery { entity_id: 999 };
        let world = World::new();

        let result: Option<[f32; 3]> = bus.execute(query, &world).expect("Test: operation should succeed");

        assert_eq!(result, None);
    }

    #[test]
    fn test_query_bus_execute_without_handler() {
        let bus = QueryBus::new();

        let query = GetEntityPositionQuery { entity_id: 1 };
        let world = World::new();

        let result: Result<Option<[f32; 3]>, _> = bus.execute(query, &world);

        assert!(result.is_err());
    }

    #[test]
    fn test_query_bus_multiple_handlers() {
        let bus = QueryBus::new();

        let position_handler = Arc::new(GetEntityPositionHandler::new());
        let all_entities_handler = Arc::new(GetAllEntitiesHandler::new());

        all_entities_handler.add_entity(1);
        all_entities_handler.add_entity(2);
        all_entities_handler.add_entity(3);

        bus.register_handler::<GetEntityPositionQuery, _>(position_handler)
            .expect("Test: operation should succeed");
        bus.register_handler::<GetAllEntitiesQuery, _>(all_entities_handler)
            .expect("Test: operation should succeed");

        let world = World::new();

        // 执行位置查询
        let position_query = GetEntityPositionQuery { entity_id: 1 };
        let position_result: Option<[f32; 3]> = bus.execute(position_query, &world).expect("Test: operation should succeed");
        assert_eq!(position_result, None);

        // 执行获取所有实体查询
        let all_query = GetAllEntitiesQuery;
        let all_result: Vec<u64> = bus.execute(all_query, &world).expect("Test: operation should succeed");
        assert_eq!(all_result, vec![1, 2, 3]);
    }

    #[test]
    fn test_query_bus_default() {
        let bus = QueryBus::default();
        assert_eq!(
            bus.handlers
                .read()
                .expect("Test: operation should succeed")
                .len(),
            0
        );
    }
}

#[cfg(test)]
mod cqrs_manager_tests {
    use super::*;

    #[test]
    fn test_cqrs_manager_new() {
        let manager = CqrsManager::new();

        assert!(!manager.has_event_sourcing());
        assert!(manager.event_sourcing().is_none());
    }

    #[test]
    fn test_cqrs_manager_default() {
        let manager = CqrsManager::default();

        assert!(!manager.has_event_sourcing());
    }

    #[test]
    fn test_cqrs_manager_with_event_sourcing() {
        let event_store = Arc::new(std::sync::RwLock::new(Box::new(
            MemoryEventStore::new(),
        )));
        let snapshot_store = Arc::new(std::sync::RwLock::new(Box::new(
            MemorySnapshotStore::new(),
        )));
        let event_sourcing = Arc::new(EventSourcingManager::new(
            event_store,
            snapshot_store,
        ));

        let manager = CqrsManager::with_event_sourcing(event_sourcing);

        assert!(manager.has_event_sourcing());
        assert!(manager.event_sourcing().is_some());
    }

    #[test]
    fn test_cqrs_manager_set_event_sourcing() {
        let mut manager = CqrsManager::new();

        assert!(!manager.has_event_sourcing());

        let event_store = Arc::new(std::sync::RwLock::new(Box::new(
            MemoryEventStore::new(),
        )));
        let snapshot_store = Arc::new(std::sync::RwLock::new(Box::new(
            MemorySnapshotStore::new(),
        )));
        let event_sourcing = Arc::new(EventSourcingManager::new(
            event_store,
            snapshot_store,
        ));

        manager.set_event_sourcing(Some(event_sourcing));

        assert!(manager.has_event_sourcing());
    }

    #[test]
    fn test_cqrs_manager_register_command_handler() {
        let manager = CqrsManager::new();
        let handler = Arc::new(CreateEntityHandler::new());

        let result = manager.register_command_handler(handler);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cqrs_manager_register_query_handler() {
        let manager = CqrsManager::new();
        let handler = Arc::new(GetEntityPositionHandler::new());

        let result = manager.register_query_handler(handler);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cqrs_manager_execute_command() {
        let manager = CqrsManager::new();
        let handler = Arc::new(CreateEntityHandler::new());

        manager.register_command_handler(handler).expect("Test: operation should succeed");

        let command = CreateEntityCommand {
            entity_id: 1,
            entity_type: "Player".to_string(),
        };

        let mut world = World::new();
        let result = manager.execute_command(command, &mut world);

        assert!(result.is_ok());
        let result = result.expect("Test: operation should succeed");
        assert!(result.success);
    }

    #[test]
    fn test_cqrs_manager_execute_query() {
        let manager = CqrsManager::new();
        let handler = Arc::new(GetAllEntitiesHandler::new());

        handler.add_entity(1);
        handler.add_entity(2);

        manager.register_query_handler(handler).expect("Test: operation should succeed");

        let query = GetAllEntitiesQuery;
        let world = World::new();

        let result: Vec<u64> = manager.execute_query(query, &world).expect("Test: operation should succeed");

        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_cqrs_manager_get_buses() {
        let manager = CqrsManager::new();

        let command_bus = manager.command_bus();
        let query_bus = manager.query_bus();

        assert_eq!(
            command_bus
                .handlers
                .read()
                .expect("Test: operation should succeed")
                .len(),
            0
        );
        assert_eq!(
            query_bus
                .handlers
                .read()
                .expect("Test: operation should succeed")
                .len(),
            0
        );
    }
}

#[cfg(test)]
mod cqrs_integration_tests {
    use super::*;

    #[test]
    fn test_cqrs_command_query_separation() {
        // 验证命令和查询的分离
        let create_command = CreateEntityCommand {
            entity_id: 1,
            entity_type: "Player".to_string(),
        };

        let position_query = GetEntityPositionQuery { entity_id: 1 };

        // 命令和查询是不同的类型
        assert_eq!(create_command.command_type(), "CreateEntity");
        assert_eq!(position_query.query_type(), "GetEntityPosition");
    }

    #[test]
    fn test_cqrs_read_write_separation() {
        let manager = CqrsManager::new();

        let move_handler = Arc::new(MoveEntityHandler::new());
        let position_handler = Arc::new(GetEntityPositionHandler::new());

        manager
            .register_command_handler(move_handler.clone())
            .expect("Test: operation should succeed");
        manager
            .register_query_handler(position_handler.clone())
            .expect("Test: operation should succeed");

        let mut world = World::new();

        // 写操作：通过命令修改状态
        let move_command = MoveEntityCommand {
            entity_id: 1,
            position: [10.0, 20.0, 30.0],
        };

        let command_result = manager.execute_command(move_command, &mut world);
        assert!(command_result.is_ok());

        // 读操作：通过查询读取状态
        // 注意：由于命令和查询使用不同的handler，这里需要手动同步状态
        let query = GetEntityPositionQuery { entity_id: 1 };

        // 在实际实现中，应该通过事件来同步命令和查询的处理器
        // 这里我们直接设置position_handler的状态
        position_handler.set_position(1, [10.0, 20.0, 30.0]);

        let position: Option<[f32; 3]> = manager.execute_query(query, &world).expect("Test: operation should succeed");
        assert_eq!(position, Some([10.0, 20.0, 30.0]));
    }

    #[test]
    fn test_cqrs_multiple_commands_same_entity() {
        let manager = CqrsManager::new();
        let handler = Arc::new(MoveEntityHandler::new());

        manager.register_command_handler(handler).expect("Test: operation should succeed");

        let mut world = World::new();

        // 执行多个移动命令
        let moves = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        ];

        for position in moves {
            let command = MoveEntityCommand {
                entity_id: 1,
                position,
            };

            manager.execute_command(command, &mut world).expect("Test: operation should succeed");
        }

        // 最终位置应该是最后一个位置
        let final_position = handler.get_position(1);
        assert_eq!(final_position, Some([3.0, 0.0, 0.0]));
    }
}

#[cfg(test)]
mod query_error_tests {
    use super::*;

    #[test]
    fn test_query_error_execution_failed() {
        let error = QueryError::ExecutionFailed("Test error".to_string());
        assert_eq!(error.to_string(), "Query execution failed: Test error");
    }

    #[test]
    fn test_query_error_not_found() {
        let error = QueryError::NotFound("Entity".to_string());
        assert_eq!(error.to_string(), "Data not found: Entity");
    }

    #[test]
    fn test_query_error_invalid_parameter() {
        let error = QueryError::InvalidParameter("entity_id".to_string());
        assert_eq!(error.to_string(), "Invalid parameter: entity_id");
    }
}
