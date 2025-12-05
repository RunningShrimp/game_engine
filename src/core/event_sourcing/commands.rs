//! 命令模式实现
//!
//! 为事件溯源提供命令接口

use super::*;
use bevy_ecs::prelude::*;

/// 命令trait
pub trait Command: Send + Sync + 'static {
    /// 执行命令
    /// 返回事件类型名称和序列化的事件数据
    fn execute(&self, world: &mut World) -> Result<(String, Vec<u8>), EventError>;

    /// 命令类型名称
    fn command_type(&self) -> &'static str;
}

/// 创建实体命令
pub struct CreateEntityCommand {
    pub entity_type: String,
    pub initial_data: Vec<u8>,
}

impl Command for CreateEntityCommand {
    fn execute(&self, world: &mut World) -> Result<(String, Vec<u8>), EventError> {
        // 创建实体
        let entity = world.spawn_empty().id();
        let entity_id = entity.index();

        // 创建事件
        let event = EntityCreatedEvent {
            entity_id: entity_id as u32,
            entity_type: self.entity_type.clone(),
        };

        // 应用事件
        event.apply(world)?;

        // 序列化事件
        let data = bincode::serialize(&event)
            .map_err(|e| EventError::SerializationError(e.to_string()))?;

        Ok((event.event_type().to_string(), data))
    }

    fn command_type(&self) -> &'static str {
        "CreateEntity"
    }
}

/// 删除实体命令
pub struct DeleteEntityCommand {
    pub entity_id: u32,
}

impl Command for DeleteEntityCommand {
    fn execute(&self, world: &mut World) -> Result<(String, Vec<u8>), EventError> {
        // 删除实体（简化处理）
        // 实际实现需要记录删除前的状态以便撤销

        let event = EntityCreatedEvent {
            entity_id: self.entity_id,
            entity_type: "Deleted".to_string(),
        };

        // 应用事件
        event.apply(world)?;

        // 序列化事件
        let data = bincode::serialize(&event)
            .map_err(|e| EventError::SerializationError(e.to_string()))?;

        Ok((event.event_type().to_string(), data))
    }

    fn command_type(&self) -> &'static str {
        "DeleteEntity"
    }
}

/// 命令处理器
pub struct CommandHandler {
    manager: Arc<EventSourcingManager>,
}

impl CommandHandler {
    pub fn new(manager: Arc<EventSourcingManager>) -> Self {
        Self { manager }
    }

    /// 执行命令并记录事件
    pub fn execute_command<C: Command>(
        &self,
        command: C,
        world: &mut World,
        aggregate_id: Option<u32>,
    ) -> Result<EventId, EventError> {
        // 执行命令（命令内部已经应用事件）
        let (event_type, event_data) = command.execute(world)?;

        // 创建存储事件
        let mut sequence = self.manager.sequence_generator.lock().unwrap();
        *sequence += 1;
        let event_id = EventId::now(*sequence);
        drop(sequence);

        let stored_event = StoredEvent {
            id: event_id,
            event_type,
            data: event_data,
            aggregate_id,
        };

        // 保存事件
        self.manager
            .event_store
            .lock()
            .unwrap()
            .save_event(stored_event)?;

        Ok(event_id)
    }
}
