# ECS架构设计

## 概述

本引擎采用ECS（Entity Component System）架构，这是一种数据导向的设计模式，特别适合游戏开发，因为它提供了出色的性能和灵活性。

## 为什么选择ECS？

### 传统面向对象的问题

```rust
// ❌ 传统OOP - 性能差
struct GameObject {
    position: Vec3,
    velocity: Vec3,
    mesh: Mesh,
    health: i32,
    // ... 更多字段
}

impl GameObject {
    fn update(&mut self) {
        // 只有部分对象需要更新
        // 但虚函数调用开销大
    }
}
```

**问题：**
- 缓存不友好（AoS布局）
- 虚函数开销
- 难以并行化
- 代码重复

### ECS的优势

```rust
// ✅ ECS - 高性能
// Components - 纯数据
struct Transform { pos: Vec3, rot: Quat, scale: Vec3 }
struct Velocity { lin: Vec3, ang: Vec3 }

// Systems - 纯逻辑
fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.pos += velocity.lin * dt;
    }
}
```

**优势：**
- **缓存友好** - SoA布局
- **并行友好** - 无状态系统
- **灵活组合** - 运行时添加组件
- **易于调试** - 数据和逻辑分离

## 架构设计

### 三大核心概念

```
┌─────────────────────────────────────────────┐
│                 ECS World                    │
├─────────────────────────────────────────────┤
│                                             │
│  ┌──────────────┐  ┌──────────────────┐    │
│  │  Entities    │  │   Components     │    │
│  │  (IDs only)  │  │  (Data Storage)  │    │
│  │              │  │                  │    │
│  │  Entity 1 ───┼──► Transform       │    │
│  │  Entity 2 ───┼──► Velocity        │    │
│  │  Entity 3 ───┼──► Sprite          │    │
│  │  ...         │  │  ...             │    │
│  └──────────────┘  └──────────────────┘    │
│                                             │
│  ┌──────────────────────────────────────┐  │
│  │            Systems                    │  │
│  │  (Logic & Behavior)                   │  │
│  │                                       │  │
│  │  ┌─────────────┐  ┌───────────────┐  │  │
│  │  │ Movement    │  │ Physics       │  │  │
│  │  │ System      │  │ System        │  │  │
│  │  └─────────────┘  └───────────────┘  │  │
│  └──────────────────────────────────────┘  │
└─────────────────────────────────────────────┘
```

### 1. Entity（实体）

**定义：** 唯一ID，不包含数据

```rust
#[derive(Component)]
pub struct Entity {
    // Entity只是一个ID，实际实现中由Bevy ECS管理
}
```

**特性：**
- 轻量级（仅ID）
- 动态组合
- 生命周期管理

### 2. Component（组件）

**定义：** 纯数据，无逻辑

```rust
/// 位置组件 - 存储空间位置
#[derive(Component, Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec3,   // 位置
    pub rot: Quat,   // 旋转
    pub scale: Vec3, // 缩放
}

/// 速度组件 - 存储运动信息
#[derive(Component, Clone, Copy, Debug)]
pub struct Velocity {
    pub lin: Vec3, // 线速度
    pub ang: Vec3, // 角速度
}
```

**设计原则：**
- 只包含数据，不包含方法（或仅构造方法）
- 小而专注（单一职责）
- 可序列化（支持保存/加载）

### 3. System（系统）

**定义：** 纯逻辑，操作数据

```rust
/// 移动系统 - 更新位置
fn movement_system(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds;

    for (mut transform, velocity) in query.iter_mut() {
        transform.pos += velocity.lin * dt;
    }
}
```

**特性：**
- 无状态（可并行）
- 声明式查询
- 自动并行化

## 数据布局优化

### AoS vs SoA

```rust
// ❌ AoS (Array of Structures) - 缓存不友好
struct GameObject {
    pos_x: f32, pos_y: f32, pos_z: f32,
    vel_x: f32, vel_y: f32, vel_z: f32,
}
// 内存: [pos_x, pos_y, pos_z, vel_x, vel_y, vel_z, ...]
// 读取pos时，vel也进入缓存（浪费）

// ✅ SoA (Structure of Arrays) - 缓存友好
struct TransformStorage {
    pos_x: Vec<f32>,
    pos_y: Vec<f32>,
    pos_z: Vec<f32>,
}
// 内存: [pos_x, pos_x, pos_x, ..., pos_y, pos_y, pos_y, ...]
// 读取pos时，只加载pos数据（高效）
```

### 脏标记追踪

```rust
/// 脏标记组件
#[derive(Component)]
pub struct ComponentDirty {
    pub flags: DirtyFlags,
}

/// 脏标记资源
#[derive(Resource)]
pub struct DirtyTrackingResource {
    dirty_entities: HashSet<Entity>,
}

// 仅同步变化的组件
fn sync_system(
    query: Query<&ComponentDirty>,
    mut resource: ResMut<DirtyTrackingResource>,
) {
    for (entity, dirty) in query.iter() {
        if dirty.flags.is_changed() {
            resource.dirty_entities.insert(entity);
        }
    }
}
```

## 系统调度

### 并行执行

```rust
// 自动并行化
let mut schedule = Schedule::default();

// 这些系统可以并行执行（无依赖冲突）
schedule.add_system(movement_system);
schedule.add_system(rotation_system);
schedule.add_system(animation_system);

// 这个系统依赖上面，会自动排序
schedule.add_system(render_system.after(movement_system));
```

### 系统依赖

```rust
// 明确指定执行顺序
fn setup_systems(schedule: &mut Schedule) {
    // 阶段1: 输入处理
    schedule.add_stage("input", SystemStage::parallel()
        .with_system(keyboard_input)
        .with_system(mouse_input)
    );

    // 阶段2: 游戏逻辑（依赖输入）
    schedule.add_stage_after("input", "update", SystemStage::parallel()
        .with_system(movement)
        .with_system(physics)
    );

    // 阶段3: 渲染（依赖更新）
    schedule.add_stage_after("update", "render", SystemStage::parallel()
        .with_system(render_sprites)
        .with_system(render_ui)
    );
}
```

## 查询系统

### 基础查询

```rust
// 可变查询
fn move_players(mut query: Query<&mut Transform, With<Player>>) {
    for mut transform in query.iter_mut() {
        transform.pos.x += 1.0;
    }
}

// 只读查询（可以和其他可变查询并行）
fn print_positions(query: Query<&Transform>) {
    for transform in query.iter() {
        println!("{:?}", transform.pos);
    }
}
```

### 过滤器

```rust
// 复杂查询条件
fn complex_query(
    // 包含Transform和Velocity，但不包含Static
    query: Query<
        (&Transform, &Velocity),
        (With<Sprite>, Without<Static>)
    >
) {
    for (transform, velocity) in query.iter() {
        // 只查询有Sprite的动态对象
    }
}

// 可选组件
fn optional_query(
    query: Query<(&Transform, Option<&Velocity>)>
) {
    for (transform, velocity) in query.iter() {
        if let Some(vel) = velocity {
            // 有速度组件
        } else {
            // 无速度组件（静态对象）
        }
    }
}
```

## 性能优化

### 1. 批量操作

```rust
// ❌ 低效 - 逐个处理
for entity in entities {
    world.get_mut::<Transform>(entity);
}

// ✅ 高效 - 批量查询
for mut transform in query.iter_mut() {
    // 批量处理
}
```

### 2. 对象池

```rust
/// 实体池 - 复用实体
#[derive(Resource)]
pub struct TileEntityPool {
    unused: Vec<Entity>,
    capacity: usize,
}

impl TileEntityPool {
    pub fn get_or_spawn(&mut self, commands: &mut Commands) -> Entity {
        if let Some(entity) = self.unused.pop() {
            entity // 复用
        } else {
            commands.spawn_empty().id() // 新建
        }
    }

    pub fn recycle(&mut self, entity: Entity) {
        if self.unused.len() < self.capacity {
            self.unused.push(entity);
        }
    }
}
```

### 3. SIMD优化

```rust
// 使用glam的SIMD向量
pub struct SoATransformStorage {
    pub positions_x: Vec<f32>,
    pub positions_y: Vec<f32>,
    pub positions_z: Vec<f32>,
}

// SIMD向量化计算
fn update_positions(storage: &mut SoATransformStorage, dt: f32) {
    for i in 0..storage.positions_x.len() {
        storage.positions_x[i] += storage.velocities_x[i] * dt;
        // SIMD指令自动向量化
    }
}
```

## 最佳实践

### 1. 组件设计

```rust
// ✅ 好的设计 - 小而专注
#[derive(Component)]
struct Position { x: f32, y: f32, z: f32 }

#[derive(Component)]
struct Health { current: u32, max: u32 }

// ❌ 避免 - 过大
#[derive(Component)]
struct PlayerData {
    position: Vec3,
    health: u32,
    inventory: Vec<Item>,
    quests: Vec<Quest>,
    // ... 太多字段
}
```

### 2. 系统设计

```rust
// ✅ 好的设计 - 单一职责
fn movement_system(query: Query<(&mut Transform, &Velocity)>) { }

fn collision_system(query: Query<&Transform>) { }

fn render_system(query: Query<&Transform>) { }

// ❌ 避免 - 做太多事
fn super_system(query: Query<(&mut Transform, &Velocity, &Health)>) {
    // 移动 + 碰撞 + 渲染... 太多逻辑
}
```

### 3. 资源使用

```rust
// 全局单例数据使用Resource
#[derive(Resource)]
struct GameConfig {
    difficulty: u32,
    max_players: usize,
}

// 临时数据使用Local
#[derive(Default)]
struct Counter(u32);

fn count_system(mut counter: Local<Counter>) {
    counter.0 += 1;
}
```

## 调试工具

### World Inspector

```rust
// 实时查看实体和组件
use game_engine::profiling::WorldInspector;

fn inspector_system(world: &mut World) {
    let inspector = WorldInspector::new();
    inspector.inspect_world(world);
    // 显示:
    // - Entity 1: Transform, Velocity, Sprite
    // - Entity 2: Transform, Camera
    // - Entity 3: Transform, Mesh, Material
}
```

## 相关文档

- [架构概览](./overview.md)
- [渲染管线](./rendering.md)
- [物理系统](./physics.md)
- [领域层设计](./domain.md)

## 参考资源

- [Bevy ECS文档](https://bevyengine.org/learn/book/getting-started/ecs/)
- [ECS Back and Forth](https://skypjack.github.io/2019-02-14-ecs-baf-part-1/)
- [Unity DOTS](https://unity.com/dots)
