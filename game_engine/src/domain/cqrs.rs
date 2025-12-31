//! CQRS（Command Query Responsibility Segregation）模式实现
//!
//! 将命令（写操作）和查询（读操作）分离，提供清晰的职责划分和更好的可扩展性。
//!
//! ## 核心概念
//!
//! - **Command（命令）**: 修改系统状态的操作，不返回值（或返回事件ID）
//! - **Query（查询）**: 只读操作，返回数据但不修改状态
//! - **Command Handler（命令处理器）**: 处理命令，可能产生领域事件
//! - **Query Handler（查询处理器）**: 处理查询，返回只读数据
//! - **Command Bus（命令总线）**: 路由命令到相应的处理器
//! - **Query Bus（查询总线）**: 路由查询到相应的处理器
//!
//! ## 示例
//!
//! ```ignore
//! use game_engine::domain::cqrs::*;
//!
//! // 创建CQRS管理器
//! let cqrs = CqrsManager::new(event_sourcing_manager);
//!
//! // 执行命令
//! let command = MovePlayerCommand { player_id: 1, position: [10.0, 0.0, 0.0] };
//! cqrs.execute_command(command, &mut world)?;
//!
//! // 执行查询
//! let query = GetPlayerPositionQuery { player_id: 1 };
//! let position = cqrs.execute_query(query, &world)?;
//! ```

use crate::domain::event_sourcing::{EventId, EventSourcingManager};
use crate::domain::events::EventError;
use crate::error::{safe_read, safe_write};
use bevy_ecs::prelude::*;
use std::any::TypeId;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// 命令trait
///
/// 命令表示修改系统状态的意图，不返回值（或返回事件ID）
pub trait Command: Send + Sync + 'static {
    /// 命令类型名称
    fn command_type(&self) -> &'static str;
}

/// 命令结果
#[derive(Debug, Clone)]
pub struct CommandResult {
    /// 是否成功
    pub success: bool,
    /// 事件ID（如果命令产生了事件）
    pub event_id: Option<EventId>,
    /// 错误消息（如果失败）
    pub error: Option<String>,
}

impl CommandResult {
    pub fn success(event_id: Option<EventId>) -> Self {
        Self {
            success: true,
            event_id,
            error: None,
        }
    }

    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            event_id: None,
            error: Some(error),
        }
    }
}

/// 命令处理器trait
///
/// 处理特定类型的命令，可能产生领域事件
pub trait CommandHandler<C: Command>: Send + Sync {
    /// 处理命令
    fn handle(&self, command: C, world: &mut World) -> Result<CommandResult, EventError>;
}

/// 查询trait
///
/// 查询表示只读操作，返回数据但不修改状态
pub trait Query: Send + Sync + 'static {
    /// 查询类型名称
    fn query_type(&self) -> &'static str;
}

/// 查询结果
pub type QueryResult<T> = Result<T, QueryError>;

/// 查询错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum QueryError {
    /// 查询执行失败
    #[error("Query execution failed: {0}")]
    ExecutionFailed(String),
    /// 未找到数据
    #[error("Data not found: {0}")]
    NotFound(String),
    /// 无效参数
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// 查询处理器trait
///
/// 处理特定类型的查询，返回只读数据
pub trait QueryHandler<Q: Query>: Send + Sync {
    /// 查询结果类型
    type Result: Send + Sync;

    /// 处理查询
    fn handle(&self, query: Q, world: &World) -> QueryResult<Self::Result>;
}

/// 命令总线
///
/// 路由命令到相应的处理器
pub struct CommandBus {
    /// 命令处理器映射：命令类型ID -> 处理器
    handlers: Arc<RwLock<HashMap<TypeId, Box<dyn CommandHandlerTrait>>>>,
}

/// 类型擦除的命令处理器trait
trait CommandHandlerTrait: Send + Sync {
    fn handle_boxed(
        &self,
        command: Box<dyn std::any::Any>,
        world: &mut World,
    ) -> Result<CommandResult, EventError>;
}

/// 命令处理器包装器
struct CommandHandlerWrapper<C: Command, H: CommandHandler<C> + 'static> {
    handler: Arc<H>,
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Command, H: CommandHandler<C> + 'static> CommandHandlerTrait
    for CommandHandlerWrapper<C, H>
{
    fn handle_boxed(
        &self,
        command: Box<dyn std::any::Any>,
        world: &mut World,
    ) -> Result<CommandResult, EventError> {
        let command = command
            .downcast::<C>()
            .map_err(|_| EventError::ApplyFailed("Invalid command type".to_string()))?;
        self.handler.handle(*command, world)
    }
}

impl CommandBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册命令处理器
    pub fn register_handler<C: Command, H: CommandHandler<C> + 'static>(
        &self,
        handler: Arc<H>,
    ) -> Result<(), EventError> {
        let type_id = TypeId::of::<C>();
        let wrapper = CommandHandlerWrapper {
            handler,
            _phantom: std::marker::PhantomData,
        };

        let mut handlers = safe_write(&self.handlers, "command_handlers")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {e}")))?;
        handlers.insert(type_id, Box::new(wrapper));
        Ok(())
    }

    /// 执行命令
    pub fn execute<C: Command>(
        &self,
        command: C,
        world: &mut World,
    ) -> Result<CommandResult, EventError> {
        let type_id = TypeId::of::<C>();
        let handlers = safe_read(&self.handlers, "command_handlers")
            .map_err(|e| EventError::ApplyFailed(format!("Failed to acquire lock: {e}")))?;

        let handler = handlers.get(&type_id).ok_or_else(|| {
            EventError::ApplyFailed(format!(
                "No handler registered for command type: {}",
                command.command_type()
            ))
        })?;

        handler.handle_boxed(Box::new(command), world)
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

/// 查询总线
///
/// 路由查询到相应的处理器
pub struct QueryBus {
    /// 查询处理器映射：查询类型ID -> 处理器
    handlers: Arc<RwLock<HashMap<TypeId, Box<dyn QueryHandlerTrait>>>>,
}

/// 类型擦除的查询处理器trait
trait QueryHandlerTrait: Send + Sync {
    fn handle_boxed(
        &self,
        query: Box<dyn std::any::Any>,
        world: &World,
    ) -> Result<Box<dyn std::any::Any>, QueryError>;
}

/// 查询处理器包装器
struct QueryHandlerWrapper<Q: Query, H: QueryHandler<Q> + 'static>
where
    <H as QueryHandler<Q>>::Result: 'static,
{
    handler: Arc<H>,
    _phantom: std::marker::PhantomData<Q>,
}

impl<Q: Query, H: QueryHandler<Q> + 'static> QueryHandlerTrait for QueryHandlerWrapper<Q, H>
where
    <H as QueryHandler<Q>>::Result: 'static,
{
    fn handle_boxed(
        &self,
        query: Box<dyn std::any::Any>,
        world: &World,
    ) -> Result<Box<dyn std::any::Any>, QueryError> {
        let query = query
            .downcast::<Q>()
            .map_err(|_| QueryError::ExecutionFailed("Invalid query type".to_string()))?;
        let result = self.handler.handle(*query, world)?;
        Ok(Box::new(result))
    }
}

impl QueryBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 注册查询处理器
    pub fn register_handler<Q: Query, H: QueryHandler<Q> + 'static>(
        &self,
        handler: Arc<H>,
    ) -> Result<(), QueryError>
    where
        <H as QueryHandler<Q>>::Result: 'static,
    {
        let type_id = TypeId::of::<Q>();
        let wrapper = QueryHandlerWrapper {
            handler,
            _phantom: std::marker::PhantomData,
        };

        let mut handlers = safe_write(&self.handlers, "query_handlers")
            .map_err(|e| QueryError::ExecutionFailed(format!("Failed to acquire lock: {e}")))?;
        handlers.insert(type_id, Box::new(wrapper));
        Ok(())
    }

    /// 执行查询
    pub fn execute<Q: Query, R: 'static>(&self, query: Q, world: &World) -> QueryResult<R> {
        let type_id = TypeId::of::<Q>();
        let handlers = safe_read(&self.handlers, "query_handlers")
            .map_err(|e| QueryError::ExecutionFailed(format!("Failed to acquire lock: {e}")))?;

        let handler = handlers.get(&type_id).ok_or_else(|| {
            QueryError::ExecutionFailed(format!(
                "No handler registered for query type: {}",
                query.query_type()
            ))
        })?;

        let result = handler.handle_boxed(Box::new(query), world)?;
        let result = result
            .downcast::<R>()
            .map_err(|_| QueryError::ExecutionFailed("Invalid result type".to_string()))?;
        Ok(*result)
    }
}

impl Default for QueryBus {
    fn default() -> Self {
        Self::new()
    }
}

/// CQRS管理器
///
/// 统一管理命令和查询总线，与事件溯源系统集成
pub struct CqrsManager {
    /// 命令总线
    command_bus: Arc<CommandBus>,
    /// 查询总线
    query_bus: Arc<QueryBus>,
    /// 事件溯源管理器（可选，用于命令产生的事件）
    event_sourcing: Option<Arc<EventSourcingManager>>,
}

impl CqrsManager {
    /// 创建新的CQRS管理器
    pub fn new() -> Self {
        Self {
            command_bus: Arc::new(CommandBus::new()),
            query_bus: Arc::new(QueryBus::new()),
            event_sourcing: None,
        }
    }

    /// 使用事件溯源管理器创建
    pub fn with_event_sourcing(event_sourcing: Arc<EventSourcingManager>) -> Self {
        Self {
            command_bus: Arc::new(CommandBus::new()),
            query_bus: Arc::new(QueryBus::new()),
            event_sourcing: Some(event_sourcing),
        }
    }

    /// 检查是否启用了事件溯源
    pub fn has_event_sourcing(&self) -> bool {
        self.event_sourcing.is_some()
    }

    /// 获取事件溯源管理器
    pub fn event_sourcing(&self) -> Option<&Arc<EventSourcingManager>> {
        self.event_sourcing.as_ref()
    }

    /// 设置事件溯源管理器
    pub fn set_event_sourcing(&mut self, event_sourcing: Option<Arc<EventSourcingManager>>) {
        self.event_sourcing = event_sourcing;
    }

    /// 注册命令处理器
    pub fn register_command_handler<C: Command, H: CommandHandler<C> + 'static>(
        &self,
        handler: Arc<H>,
    ) -> Result<(), EventError> {
        self.command_bus.register_handler(handler)
    }

    /// 注册查询处理器
    pub fn register_query_handler<Q: Query, H: QueryHandler<Q> + 'static>(
        &self,
        handler: Arc<H>,
    ) -> Result<(), QueryError>
    where
        <H as QueryHandler<Q>>::Result: 'static,
    {
        self.query_bus.register_handler(handler)
    }

    /// 执行命令
    pub fn execute_command<C: Command>(
        &self,
        command: C,
        world: &mut World,
    ) -> Result<CommandResult, EventError> {
        let result = self.command_bus.execute(command, world)?;

        // 如果命令成功且产生了事件，可以记录到事件溯源系统
        // 注意：这需要命令处理器返回事件信息
        // 这里简化处理，实际应该由命令处理器负责事件记录

        Ok(result)
    }

    /// 执行查询
    pub fn execute_query<Q: Query, R: 'static>(&self, query: Q, world: &World) -> QueryResult<R> {
        self.query_bus.execute(query, world)
    }

    /// 获取命令总线引用
    pub fn command_bus(&self) -> &Arc<CommandBus> {
        &self.command_bus
    }

    /// 获取查询总线引用
    pub fn query_bus(&self) -> &Arc<QueryBus> {
        &self.query_bus
    }
}

impl Default for CqrsManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试命令
    #[derive(Debug, Clone)]
    struct TestCommand {
        value: u32,
    }

    impl Command for TestCommand {
        fn command_type(&self) -> &'static str {
            "TestCommand"
        }
    }

    // 测试命令处理器
    struct TestCommandHandler;

    impl CommandHandler<TestCommand> for TestCommandHandler {
        fn handle(
            &self,
            command: TestCommand,
            _world: &mut World,
        ) -> Result<CommandResult, EventError> {
            Ok(CommandResult::success(None))
        }
    }

    // 测试查询
    #[derive(Debug, Clone)]
    struct TestQuery {
        value: u32,
    }

    impl Query for TestQuery {
        fn query_type(&self) -> &'static str {
            "TestQuery"
        }
    }

    // 测试查询处理器
    struct TestQueryHandler;

    impl QueryHandler<TestQuery> for TestQueryHandler {
        type Result = u32;

        fn handle(&self, query: TestQuery, _world: &World) -> QueryResult<Self::Result> {
            Ok(query.value * 2)
        }
    }

    #[test]
    fn test_command_bus() {
        let bus = CommandBus::new();
        let handler = Arc::new(TestCommandHandler);
        bus.register_handler::<TestCommand, _>(handler)
            .expect("Test: handler registration should succeed");

        let command = TestCommand { value: 42 };
        let mut world = World::new();
        let result = bus
            .execute(command, &mut world)
            .expect("Test: command execution should succeed");

        assert!(result.success);
    }

    #[test]
    fn test_query_bus() {
        let bus = QueryBus::new();
        let handler = Arc::new(TestQueryHandler);
        bus.register_handler::<TestQuery, _>(handler)
            .expect("Test: handler registration should succeed");

        let query = TestQuery { value: 21 };
        let mut world = World::new();
        let result: u32 = bus.execute(query, &world).expect("Test: query execution should succeed");

        assert_eq!(result, 42);
    }

    #[test]
    fn test_cqrs_manager() {
        let manager = CqrsManager::new();

        // 注册处理器
        let cmd_handler = Arc::new(TestCommandHandler);
        manager
            .register_command_handler(cmd_handler)
            .expect("Test: command handler registration should succeed");

        let query_handler = Arc::new(TestQueryHandler);
        manager
            .register_query_handler(query_handler)
            .expect("Test: query handler registration should succeed");

        // 执行命令
        let command = TestCommand { value: 42 };
        let mut world = World::new();
        let result = manager
            .execute_command(command, &mut world)
            .expect("Test: command execution should succeed");
        assert!(result.success);

        // 执行查询
        let query = TestQuery { value: 21 };
        let result: u32 = manager
            .execute_query(query, &world)
            .expect("Test: query execution should succeed");
        assert_eq!(result, 42);
    }
}
