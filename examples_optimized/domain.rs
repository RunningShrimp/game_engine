//! # Domain层示例
//!
//! 此示例展示如何使用领域驱动设计（DDD）的Domain层。
//!
//! ## 运行
//!
//! ```bash
//! cargo run --example domain
//! ```

use game_engine::domain::cqrs::*;
use game_engine::domain::entity::*;
use game_engine::domain::event_sourcing::*;
use game_engine::domain::physics::*;
use game_engine::domain::prelude::*;
use game_engine::domain::services::*;
use game_engine::domain::value_objects::*;
use glam::Vec3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Domain Layer Example ===\n");

    // 1. 实体工厂 - 创建领域实体
    println!("1. Using Entity Factory:");
    let player = EntityFactory::create("player")
        .with_position(Position::new(0.0, 0.0, 0.0))
        .with_velocity(Velocity::new(1.0, 0.0, 0.0))
        .with_mass(Mass::from_kilograms(70.0))
        .build();

    println!("   Created player entity:");
    println!("   - Position: {:?}", player.position());
    println!("   - Velocity: {:?}", player.velocity());
    println!("   - Mass: {} kg", player.mass().value());

    // 2. 领域服务 - 物理服务
    println!("\n2. Using Physics Domain Service:");
    let mut physics_service = PhysicsDomainService::new();

    // 创建刚体
    let body_id = RigidBodyId::new(1);
    let rigid_body = RigidBody::new(
        body_id,
        RigidBodyType::Dynamic,
        Position::new(0.0, 10.0, 0.0),
        Mass::from_kilograms(1.0),
    );

    physics_service.create_body(rigid_body)?;
    println!("   Created rigid body with ID: {}", body_id);

    // 应用力
    physics_service.apply_force(body_id, Velocity::new(0.0, -9.8, 0.0))?;
    println!("   Applied gravity force");

    // 更新物理状态
    physics_service.update(0.016)?;

    // 3. CQRS - 命令查询分离
    println!("\n3. Using CQRS Pattern:");

    // 命令：修改状态
    let create_command = CreateEntityCommand {
        entity_type: "enemy".to_string(),
        position: Position::new(5.0, 0.0, 0.0),
    };

    println!("   Executing command: CreateEntity");
    let command_result = physics_service.execute_command(create_command);
    println!("   Command result: {:?}", command_result);

    // 查询：读取状态（不修改状态）
    let query = EntityQuery::by_position(Position::new(5.0, 0.0, 0.0));
    println!("   Executing query: Find entities at position");

    // 4. 事件溯源 - 记录状态变更
    println!("\n4. Using Event Sourcing:");

    let mut event_store = InMemoryEventStore::new();

    // 记录事件
    let move_event = EntityMovedEvent {
        entity_id: EntityId::new(1),
        from: Position::new(0.0, 0.0, 0.0),
        to: Position::new(1.0, 0.0, 0.0),
        timestamp: std::time::SystemTime::now(),
    };

    event_store.append(move_event)?;
    println!("   Recorded event: EntityMoved");

    // 重放事件重建状态
    let events = event_store.get_events(EntityId::new(1))?;
    println!("   Event history contains {} events", events.len());

    // 5. 领域事件 - 事件总线
    println!("\n5. Using Domain Event Bus:");

    use game_engine::domain::event_bus::{DomainEvent, EnhancedEventBus};

    let mut event_bus = EnhancedEventBus::new();

    // 订阅事件
    event_bus.subscribe(Box::new(|event: &DomainEvent| match event {
        DomainEvent::EntityCreated { id } => {
            println!("   [Subscriber] Entity created: {:?}", id);
        }
        DomainEvent::EntityMoved { id, from, to } => {
            println!(
                "   [Subscriber] Entity {:?} moved: {:?} -> {:?}",
                id, from, to
            );
        }
        _ => {}
    }));

    // 发布事件
    event_bus.publish(DomainEvent::EntityCreated {
        id: EntityId::new(2),
        entity_type: "projectile".to_string(),
    });

    event_bus.publish(DomainEvent::EntityMoved {
        id: EntityId::new(2),
        from: Position::new(0.0, 0.0, 0.0),
        to: Position::new(10.0, 5.0, 0.0),
    });

    println!("   Published 2 domain events");

    // 6. 值对象 - 不可变值
    println!("\n6. Using Value Objects:");

    let pos1 = Position::new(1.0, 2.0, 3.0);
    let pos2 = Position::new(1.0, 2.0, 3.0);
    let pos3 = Position::new(4.0, 5.0, 6.0);

    println!("   pos1 == pos2: {}", pos1 == pos2); // true
    println!("   pos1 == pos3: {}", pos1 == pos3); // false

    // 值对象是不可变的
    let vel = Velocity::new(1.0, 0.0, 0.0);
    let new_vel = vel.add(Velocity::new(0.0, 1.0, 0.0));
    println!("   Original velocity: {:?}", vel);
    println!("   New velocity: {:?}", new_vel);

    // 7. 聚合根 - 业务一致性边界
    println!("\n7. Using Aggregates:");

    let mut scene = Scene::new("level1");
    scene.add_entity(player);
    println!("   Created scene with {} entities", scene.entity_count());

    // 场景作为聚合根保证内部一致性
    scene.validate()?;

    println!("\n=== Domain Layer Features Summary ===");
    println!("✓ Entity Factory - Builder pattern for entities");
    println!("✓ Domain Services - Business logic services");
    println!("✓ CQRS - Command-query separation");
    println!("✓ Event Sourcing - State change history");
    println!("✓ Domain Events - Event-driven communication");
    println!("✓ Value Objects - Immutable values");
    println!("✓ Aggregates - Consistency boundaries");

    Ok(())
}

// 伪代码展示更多CQRS用法
struct CreateEntityCommand {
    entity_type: String,
    position: Position,
}

struct EntityQuery;

impl EntityQuery {
    fn by_position(pos: Position) -> Self {
        Self
    }
}

// 伪代码展示事件溯源
struct EntityMovedEvent {
    entity_id: EntityId,
    from: Position,
    to: Position,
    timestamp: std::time::SystemTime,
}

trait EventStore {
    fn append<T>(&mut self, event: T) -> Result<(), DomainError>;
    fn get_events(&self, id: EntityId) -> Result<Vec<EntityMovedEvent>, DomainError>;
}

struct InMemoryEventStore;

impl InMemoryEventStore {
    fn new() -> Self {
        Self
    }
}

impl EventStore for InMemoryEventStore {
    fn append<T>(&mut self, _event: T) -> Result<(), DomainError> {
        Ok(())
    }

    fn get_events(&self, _id: EntityId) -> Result<Vec<EntityMovedEvent>, DomainError> {
        Ok(vec![])
    }
}
