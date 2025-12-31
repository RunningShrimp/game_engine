# ECS 系统深入指南

本指南深入介绍游戏引擎的实体组件系统（ECS）架构，帮助您理解并有效使用ECS模式开发游戏。

## 目录

1. [ECS 基础概念](#ecs-基础概念)
2. [实体 (Entities)](#实体-entities)
3. [组件 (Components)](#组件-components)
4. [系统 (Systems)](#系统-systems)
5. [查询和过滤器](#查询和过滤器)
6. [系统调度](#系统调度)
7. [高级特性](#高级特性)
8. [性能优化](#性能优化)
9. [最佳实践](#最佳实践)
10. [实战案例](#实战案例)

## ECS 基础概念

### 什么是 ECS？

ECS（Entity-Component-System）是一种数据导向的架构模式，它将游戏对象分解为三个核心概念：

```
┌─────────────────────────────────────────────────────┐
│                    ECS 架构                         │
├─────────────────────────────────────────────────────┤
│                                                      │
│  ┌──────────┐      ┌──────────┐      ┌──────────┐  │
│  │ 实体     │ ───> │ 组件     │ <─── │ 系统     │  │
│  │ Entity   │      │ Component│      │ System   │  │
│  │          │      │          │      │          │  │
│  │ • ID     │      │ • 数据   │      │ • 逻辑   │  │
│  │ • 标签   │      │ • 状态   │      │ • 行为   │  │
│  └──────────┘      └──────────┘      └──────────┘  │
│                                                      │
└─────────────────────────────────────────────────────┘
```

**为什么使用 ECS？**

1. **性能**: 数据局部性好，CPU缓存利用率高
2. **灵活性**: 通过组合组件动态构建实体
3. **可维护性**: 逻辑和数据分离，代码更清晰
4. **并行性**: 系统可以并行执行

### 与传统 OOP 的对比

#### 传统面向对象方法
```rust
// ❌ 传统 OOP - 紧耦合
struct GameObject {
    position: Vec3,
    velocity: Vec3,
    health: i32,
    sprite: Sprite,
    // ... 更多字段
}

impl GameObject {
    fn update(&mut self) {
        // 混合了移动、渲染、物理等逻辑
    }
}
```

#### ECS 方法
```rust
// ✅ ECS - 松耦合
// 1. 定义组件（纯数据）
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { dx: f32, dy: f32 }

// 2. 定义系统（纯逻辑）
fn movement_system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.x += vel.dx;
        pos.y += vel.dy;
    }
}
```

## 实体 (Entities)

### 什么是实体？

实体只是**唯一的 ID**，用于标识游戏对象。实体本身不包含数据，数据存储在组件中。

### 创建实体

```rust
use bevy_ecs::prelude::*;
use game_engine::ecs::{Transform, Sprite};

fn spawn_entities(mut commands: Commands) {
    // 创建带有组件的实体
    commands.spawn((
        Transform::default(),
        Sprite::default(),
    ));

    // 批量创建
    for i in 0..100 {
        commands.spawn((
            Transform {
                pos: Vec3::new(i as f32, 0.0, 0.0),
                ..Default::default()
            },
            Sprite::default(),
        ));
    }
}
```

### 实体操作

```rust
// 获取实体
fn get_entity(query: Query<Entity, With<Transform>>) {
    for entity in query.iter() {
        println!("Entity: {:?}", entity);
    }
}

// 删除实体
fn despawn_entities(mut commands: Commands, query: Query<Entity, With<Dead>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}
```

## 组件 (Components)

### 什么是组件？

组件是**纯数据**结构，附加到实体上，描述实体的特定属性。

### 定义组件

```rust
use bevy_ecs::component::Component;

#[derive(Component)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Component)]
struct Name(String);

// 带默认值的组件
#[derive(Component, Default)]
struct Player {
    level: u32,
    experience: u32,
}
```

### 组件生命周期

```rust
fn component_lifecycle(mut commands: Commands) {
    // 1. 创建实体并添加组件
    let entity = commands.spawn(Health {
        current: 100,
        max: 100,
    }).id();

    // 2. 动态添加组件
    commands.entity(entity)
        .insert(Name(String::from("Player")));

    // 3. 移除组件
    commands.entity(entity)
        .remove::<Health>();

    // 4. 删除实体（会移除所有组件）
    commands.entity(entity).despawn();
}
```

### 内置组件

引擎提供了许多常用组件：

| 组件 | 描述 | 字段 |
|------|------|------|
| `Transform` | 位置、旋转、缩放 | `pos`, `rot`, `scale` |
| `Velocity` | 线性和角速度 | `lin`, `ang` |
| `Sprite` | 2D 精灵渲染 | `color`, `tex_index`, `uv_off` |
| `Camera` | 相机投影 | `projection`, `viewport` |
| `PointLight` | 点光源 | `color`, `intensity`, `radius` |

## 系统 (Systems)

### 什么是系统？

系统是**纯逻辑**函数，操作具有特定组件组合的实体。

### 基础系统

```rust
use bevy_ecs::prelude::*;

// 简单系统 - 只读查询
fn print_positions(query: Query<&Transform>) {
    for transform in query.iter() {
        println!("Position: {:?}", transform.pos);
    }
}

// 可变系统 - 修改数据
fn update_positions(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        transform.pos.x += 1.0;
    }
}

// 组合查询
fn physics_update(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.pos += velocity.lin;
    }
}
```

### 系统参数

系统可以访问多种资源：

```rust
fn complex_system(
    // 查询实体
    mut query: Query<(&mut Transform, &Velocity)>,
    // 访问资源
    time: Res<Time>,
    // 访问事件
    mut events: EventWriter<CollisionEvent>,
    // 命令队列
    mut commands: Commands,
) {
    // 系统逻辑
}
```

### 系统示例

#### 移动系统
```rust
fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.pos += velocity.lin * time.delta;
    }
}
```

#### 生命周期系统
```rust
#[derive(Component)]
struct Lifetime {
    remaining: f32,
}

fn lifetime_system(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Lifetime)>
) {
    for (entity, mut lifetime) in query.iter_mut() {
        lifetime.remaining -= time.delta;
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

## 查询和过滤器

### 基础查询

```rust
// 所有具有 Transform 组件的实体
Query<&Transform>

// 可变访问
Query<&mut Transform>

// 多个组件（AND关系）
Query<(&Transform, &Velocity)>

// 混合可变和只读
Query<(&mut Transform, &Velocity)>
```

### 过滤器

使用过滤器来筛选特定的实体子集：

```rust
use bevy_ecs::prelude::*;

// 只有具有 Transform 的实体
Query<&Transform, With<Velocity>>

// 没有 Health 组件的实体
Query<&Transform, Without<Health>>

// 使用组件作为标记
#[derive(Component)]
struct Player;

Query<&Transform, With<Player>>
```

### 高级查询

```rust
// 可选组件
fn optional_query(query: Query<(&Transform, Option<&Velocity>)>) {
    for (transform, velocity) in query.iter() {
        if let Some(vel) = velocity {
            // 有速度的实体
        } else {
            // 没有速度的实体
        }
    }
}

// 实体 + 组件
fn entity_query(query: Query<(Entity, &Transform)>) {
    for (entity, transform) in query.iter() {
        println!("{:?}: {:?}", entity, transform.pos);
    }
}

// 过滤 + 可选
fn complex_query(
    query: Query<
        (Entity, &Transform, Option<&Velocity>),
        With<Player>
    >
) {
    // 查询所有玩家，包括没有速度的
}
```

### 查询性能

```rust
// ✅ 批量处理 - 高性能
fn batch_system(mut query: Query<&mut Transform>) {
    // 单次遍历，处理所有实体
    for mut transform in query.iter_mut() {
        transform.pos.x += 1.0;
    }
}

// ❌ 避免重复查询
fn bad_system(query1: Query<&Transform>, query2: Query<&Velocity>) {
    // 两次独立查询，性能差
}

// ✅ 合并查询
fn good_system(query: Query<(&Transform, &Velocity)>) {
    // 单次查询，性能好
}
```

## 系统调度

### 系统执行顺序

控制系统的执行顺序：

```rust
use game_engine::core::Scheduler;

fn setup_scheduler(scheduler: &mut Scheduler) {
    // 添加系统
    scheduler.add_system(input_system);
    scheduler.add_system(movement_system);
    scheduler.add_system(collision_system);
    scheduler.add_system(rendering_system);

    // 设置依赖关系
    scheduler.set_dependency("movement", "input");
    scheduler.set_dependency("collision", "movement");
    scheduler.set_dependency("rendering", "collision");
}
```

### 并行执行

系统可以自动并行执行（当没有数据竞争时）：

```rust
// 这些系统可以并行执行
fn system_a(mut query: Query<&mut ComponentA>) {}
fn system_b(mut query: Query<&mut ComponentB>) {}

// 这些系统必须串行执行（都访问 ComponentA）
fn system_c(mut query: Query<&mut ComponentA>) {}
fn system_d(mut query: Query<&mut ComponentA>) {}
```

### 系统集

将相关系统分组：

```rust
scheduler.create_system_set("Physics")
    .with_system(movement_system)
    .with_system(collision_system)
    .with_system(gravity_system);

scheduler.create_system_set("Rendering")
    .with_system(culling_system)
    .with_system(render_system)
    .with_system(ui_system);
```

## 高级特性

### 脏标记 (Dirty Flags)

只同步修改过的组件：

```rust
use game_engine::ecs::DirtyTrackingResource;

fn setup_dirty_tracking(world: &mut World) {
    world.insert_resource(DirtyTrackingResource::new());
}

fn only_sync_changed(
    mut query: Query<&mut Transform>,
    dirty: Res<DirtyTrackingResource>
) {
    for mut transform in query.iter_mut() {
        if dirty.is_changed::<Transform>(transform.entity()) {
            // 只处理修改过的组件
            sync_to_gpu(&transform);
        }
    }
}
```

### SoA 布局 (Structure of Arrays)

优化内存布局以提高缓存利用率：

```rust
use game_engine::ecs::SoALayoutManager;

fn setup_soa(world: &mut World) {
    let mut soa_manager = SoALayoutManager::new();

    // 为 Transform 启用 SoA
    soa_manager.register_component::<Transform>();
    soa_manager.optimize_storage::<Transform>(world);
}
```

### 查询缓存

缓存频繁使用的查询：

```rust
use game_engine::ecs::QueryCache;

fn setup_query_cache(world: &mut World) {
    let cache = QueryCache::new();
    cache.register_query::<Query<(&Transform, &Velocity)>>(
        "transform_velocity"
    );
    world.insert_resource(cache);
}
```

### 事件系统

系统之间通过事件通信：

```rust
use bevy_ecs::event::{Event, EventReader, EventWriter};

#[derive(Event)]
struct CollisionEvent {
    entity_a: Entity,
    entity_b: Entity,
}

fn detect_collisions(
    mut events: EventWriter<CollisionEvent>,
    query: Query<(Entity, &Transform)>
) {
    // 检测碰撞并发送事件
    events.send(CollisionEvent { ... });
}

fn handle_collisions(
    mut events: EventReader<CollisionEvent>
) {
    for event in events.iter() {
        // 处理碰撞事件
    }
}
```

## 性能优化

### 1. 组件设计

```rust
// ✅ 小而专注的组件
#[derive(Component)]
struct Position { x: f32, y: f32 }

#[derive(Component)]
struct Velocity { dx: f32, dy: f32 }

// ❌ 大而杂的组件
#[derive(Component)]
struct PhysicsState {
    position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    mass: f32,
    // ... 更多字段
}
```

### 2. 批量处理

```rust
// ✅ 批量创建
fn spawn_batch(mut commands: Commands) {
    commands.spawn_batch((0..1000).map(|i| (
        Transform::default(),
        Velocity::default(),
    )));
}
```

### 3. 避免分支

```rust
// ❌ 有分支 - 性能差
fn bad_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, vel) in query.iter_mut() {
        if vel.dx != 0.0 || vel.dy != 0.0 {
            transform.pos.x += vel.dx;
            transform.pos.y += vel.dy;
        }
    }
}

// ✅ 使用过滤器 - 性能好
fn good_system(
    mut moving: Query<(&mut Transform, &Velocity)>,
    static_entities: Query<&Transform, Without<Velocity>>
) {
    for (mut transform, vel) in moving.iter_mut() {
        transform.pos.x += vel.dx;
        transform.pos.y += vel.dy;
    }
}
```

### 4. 使用并行

```rust
// 启用并行特性
[dependencies]
game_engine = { path = "../game_engine", features = ["parallel"] }
```

### 5. 性能分析

```rust
use game_engine::profiling::Profiler;

fn profiled_system(query: Query<&mut Transform>) {
    let _profile = Profiler::start("movement_system");
    // 系统逻辑
}
```

## 最佳实践

### 1. 组件设计原则

- **单一职责**: 每个组件只负责一个方面
- **纯数据**: 组件不应该有方法
- **可序列化**: 使用 `#[derive(Serialize, Deserialize)]`

### 2. 系统设计原则

- **专注功能**: 每个系统做一件事
- **无副作用**: 避免系统之间的隐式依赖
- **声明式**: 通过查询和数据流表达逻辑

### 3. 架构模式

```rust
// 分层架构
// 1. 输入层
fn input_system(keyboard: Res<Input<KeyCode>>) {}

// 2. 逻辑层
fn game_logic_system(mut query: Query<(&mut Health, &Damage)>) {}

// 3. 物理层
fn physics_system(mut query: Query<(&mut Transform, &Velocity)>) {}

// 4. 渲染层
fn rendering_system(query: Query<&Transform>) {}
```

### 4. 错误处理

```rust
// 使用 Result 处理错误
fn safe_system(query: Query<&mut Health>) -> Result<(), SystemError> {
    for mut health in query.iter_mut() {
        health.current = health
            .current
            .checked_sub(10)
            .ok_or(SystemError::Underflow)?;
    }
    Ok(())
}
```

## 实战案例

### 案例 1: 2D 平台游戏

```rust
use bevy_ecs::prelude::*;

// 组件定义
#[derive(Component)]
struct Player {
    grounded: bool,
    jump_count: u32,
}

#[derive(Component)]
struct Gravity {
    force: f32,
}

#[derive(Component)]
struct JumpForce {
    strength: f32,
}

// 系统实现
fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Velocity, &Gravity), Without<Player>>
) {
    for (mut velocity, gravity) in query.iter_mut() {
        velocity.lin.y -= gravity.force * time.delta;
    }
}

fn player_jump(
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<&mut Velocity, With<Player>>
) {
    for mut velocity in query.iter_mut() {
        if keyboard.just_pressed(KeyCode::Space) {
            velocity.lin.y = 10.0;
        }
    }
}

fn ground_detection(
    mut query: Query<(&mut Player, &Transform)>
) {
    for (mut player, transform) in query.iter_mut() {
        player.grounded = transform.pos.y <= 0.0;
    }
}
```

### 案例 2: 弹幕射击游戏

```rust
// 组件
#[derive(Component)]
struct Bullet {
    damage: u32,
    lifetime: f32,
}

#[derive(Component)]
struct Enemy {
    health: u32,
}

// 系统
fn spawn_bullets(
    mut commands: Commands,
    keyboard: Res<Input<KeyCode>>,
    query: Query<&Transform, With<Player>>
) {
    if keyboard.just_pressed(KeyCode::Space) {
        for transform in query.iter() {
            commands.spawn((
                Bullet {
                    damage: 10,
                    lifetime: 2.0,
                },
                Transform {
                    pos: transform.pos,
                    ..Default::default()
                },
                Velocity {
                    lin: Vec3::new(0.0, 10.0, 0.0),
                    ..Default::default()
                },
            ));
        }
    }
}

fn bullet_collision(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &Bullet)>,
    enemies: Query<(Entity, &Transform, &mut Enemy)>,
) {
    for (bullet_entity, bullet_pos, bullet) in bullets.iter() {
        for (enemy_entity, enemy_pos, mut enemy) in enemies.iter() {
            let distance = bullet_pos.pos.distance(enemy_pos.pos);
            if distance < 1.0 {
                enemy.health -= bullet.damage;
                commands.entity(bullet_entity).despawn();

                if enemy.health == 0 {
                    commands.entity(enemy_entity).despawn();
                }
            }
        }
    }
}
```

### 案例 3: RPG 属性系统

```rust
// 组件
#[derive(Component)]
struct Character {
    name: String,
    level: u32,
}

#[derive(Component)]
struct Attributes {
    strength: u32,
    agility: u32,
    intelligence: u32,
}

#[derive(Component)]
struct Health {
    current: u32,
    max: u32,
}

#[derive(Component)]
struct Mana {
    current: u32,
    max: u32,
}

// 系统
fn level_up(
    mut query: Query<(&mut Character, &mut Attributes, &mut Health, &mut Mana)>
) {
    for (mut character, mut attributes, mut health, mut mana) in query.iter_mut() {
        character.level += 1;
        attributes.strength += 2;
        attributes.intelligence += 2;

        health.max += 10 * character.level;
        health.current = health.max;

        mana.max += 5 * character.level;
        mana.current = mana.max;
    }
}
```

## 调试和测试

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_movement_system() {
        let mut world = World::new();

        // 创建测试实体
        world.spawn((
            Transform { pos: Vec3::ZERO, ..Default::default() },
            Velocity { lin: Vec3::new(1.0, 0.0, 0.0), ..Default::default() },
        ));

        // 运行系统
        let mut schedule = SystemStage::single_threaded();
        schedule.add_system(movement_system);
        schedule.run(&mut world);

        // 验证结果
        let mut query = world.query::<&Transform>();
        for transform in query.iter(&world) {
            assert_eq!(transform.pos.x, 1.0);
        }
    }
}
```

### 调试工具

```rust
// 打印实体信息
fn debug_system(query: Query<(Entity, &Transform, &Velocity)>) {
    for (entity, transform, velocity) in query.iter() {
        println!(
            "Entity {:?}: pos={:?}, vel={:?}",
            entity, transform.pos, velocity.lin
        );
    }
}

// 统计实体数量
fn stats_system(query: Query<&Transform>) {
    println!("Active entities: {}", query.iter().count());
}
```

## 参考资源

- [Bevy ECS 官方文档](https://bevyengine.org/learn/book/getting-started/ecs/)
- [ECS 模式详解](https://github.com/SanderMertens/ecs-faq)
- [性能优化指南](../PERFORMANCE_BEST_PRACTICES.md)
- [API 文档](https://docs.rs/game_engine)

## 总结

ECS 是一个强大的架构模式，通过以下核心概念实现高性能游戏开发：

- **实体**: 唯一标识符
- **组件**: 纯数据结构
- **系统**: 纯逻辑函数
- **查询**: 灵活的数据访问
- **调度**: 系统执行管理

掌握这些概念和最佳实践，将帮助您构建高性能、可维护的游戏系统。

---

**下一步**: [渲染系统教程](./rendering_guide.md)
