//! CQRS模式示例
//!
//! 展示CQRS模式的使用，包括：
//! - 命令和查询的定义
//! - 命令和查询处理器的实现
//! - 命令总线和查询总线的使用
//! - 与事件溯源系统的集成

use game_engine::domain::cqrs::{
    Command, CommandHandler, CommandResult, CqrsManager, Query, QueryHandler, QueryError,
};
use game_engine::domain::events::EventError;
use bevy_ecs::prelude::*;
use std::sync::Arc;

// 示例命令：移动玩家
#[derive(Debug, Clone)]
struct MovePlayerCommand {
    player_id: u32,
    position: [f32; 3],
}

impl Command for MovePlayerCommand {
    fn command_type(&self) -> &'static str {
        "MovePlayer"
    }
}

// 命令处理器
struct MovePlayerCommandHandler;

impl CommandHandler<MovePlayerCommand> for MovePlayerCommandHandler {
    fn handle(
        &self,
        command: MovePlayerCommand,
        _world: &mut World,
    ) -> Result<CommandResult, EventError> {
        println!("Executing MovePlayer command: player {} to {:?}", 
                 command.player_id, command.position);
        
        // 这里可以产生领域事件
        // 例如：PlayerMovedEvent
        
        Ok(CommandResult::success(None))
    }
}

// 示例查询：获取玩家位置
#[derive(Debug, Clone)]
struct GetPlayerPositionQuery {
    player_id: u32,
}

impl Query for GetPlayerPositionQuery {
    fn query_type(&self) -> &'static str {
        "GetPlayerPosition"
    }
}

// 查询处理器
struct GetPlayerPositionQueryHandler;

impl QueryHandler<GetPlayerPositionQuery> for GetPlayerPositionQueryHandler {
    type Result = [f32; 3];

    fn handle(
        &self,
        query: GetPlayerPositionQuery,
        _world: &World,
    ) -> Result<Self::Result, QueryError> {
        println!("Executing GetPlayerPosition query: player {}", query.player_id);
        
        // 从世界状态读取数据
        // 这里简化处理，返回固定位置
        Ok([10.0, 0.0, 0.0])
    }
}

fn main() -> Result<(), EventError> {
    tracing_subscriber::fmt::init();

    println!("=== CQRS Pattern Example ===\n");

    // 1. 创建CQRS管理器
    println!("1. Creating CQRS manager...");
    let cqrs = CqrsManager::new();

    // 2. 注册命令处理器
    println!("2. Registering command handler...");
    let move_handler = Arc::new(MovePlayerCommandHandler);
    cqrs.register_command_handler(move_handler)?;
    println!("  Registered MovePlayerCommandHandler");

    // 3. 注册查询处理器
    println!("3. Registering query handler...");
    let position_handler = Arc::new(GetPlayerPositionQueryHandler);
    cqrs.register_query_handler(position_handler)?;
    println!("  Registered GetPlayerPositionQueryHandler");

    // 4. 执行命令
    println!("\n4. Executing commands...");
    let mut world = World::new();

    let command1 = MovePlayerCommand {
        player_id: 1,
        position: [10.0, 0.0, 0.0],
    };
    let result1 = cqrs.execute_command(command1, &mut world)?;
    println!("  Command 1 result: success = {}", result1.success);

    let command2 = MovePlayerCommand {
        player_id: 1,
        position: [20.0, 0.0, 0.0],
    };
    let result2 = cqrs.execute_command(command2, &mut world)?;
    println!("  Command 2 result: success = {}", result2.success);

    // 5. 执行查询
    println!("\n5. Executing queries...");
    let query = GetPlayerPositionQuery { player_id: 1 };
    let position: [f32; 3] = cqrs.execute_query(query, &world)?;
    println!("  Player position: {:?}", position);

    // 6. 演示错误处理
    println!("\n6. Error handling demonstration...");
    let invalid_command = MovePlayerCommand {
        player_id: 999,
        position: [0.0, 0.0, 0.0],
    };
    // 注意：这里不会失败，因为命令处理器总是成功
    // 实际应用中，命令处理器应该验证命令并可能返回失败
    let result = cqrs.execute_command(invalid_command, &mut world)?;
    println!("  Invalid command result: success = {}", result.success);

    println!("\n=== Example completed successfully! ===");
    Ok(())
}

