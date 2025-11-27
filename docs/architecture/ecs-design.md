# ECS设计

引擎采用Entity-Component-System架构，提供高性能的数据导向游戏开发模式。

## ECS基本概念

### 实体 (Entity)

实体是游戏对象的唯一标识符，没有固有数据：

```rust
// 创建实体
let entity = world.spawn_empty().id();

// 使用bevy_ecs的实体
use bevy_ecs::prelude::*;

let entity = commands.spawn_empty().id();
```

### 组件 (Component)

组件是纯数据结构，附加到实体上。引擎提供丰富的内置组件，并支持自定义组件：

```rust
#[derive(Component, Clone, Copy, Debug)]
pub struct Transform {
    pub pos: Vec3,
    pub rot: Quat,
    pub scale: Vec3,
}

#[derive(Component, Clone, Debug)]
pub struct Sprite {
    pub color: [f32; 4],
    pub tex_index: u32,
    pub uv_off: [f32; 2],
    pub uv_scale: [f32; 2],
    pub layer: f32,
}
```

### 系统 (System)

系统是操作组件数据的逻辑函数，按顺序执行：

```rust
pub fn movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity)>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.pos += velocity.lin * time.delta_seconds;
    }
}
```

## 架构优势

### 数据导向设计

- **纯数据组件**: 组件只包含数据，不含逻辑
- **组合灵活性**: 同类型组件可组合出不同行为
- **内存连续**: 同类型组件存储在连续内存中
- **缓存友好**: 提升CPU缓存命中率

### 高性能查询

ECS使用Archetype模式管理实体，一个Archetype包含具有相同组件集合的所有实体：

```rust
// 查询特定组件组合
Query<&Transform, (&mut Velocity, &Sprite)>

// 查询实体ID
Query<Entity, &Transform>

// 可变查询
Query<&mut Transform>

// 排除查询
Query<&Transform, Without<StaticBody>>
```

### 系统执行模型

系统按调度计划顺序执行，支持依赖关系和并行执行：

```rust
let mut schedule = Schedule::default();

// 链式执行 (顺序)
schedule.add_systems((
    input_system,
    physics_system,
    render_system,
).chain());

// 并行执行 (同一阶段)
schedule.add_systems((
    audio_system,
    animation_system,
).in_set(MySet));

// 条件执行
schedule.add_systems(physics_system.run_if(in_state(GameState::Running)));
```

## 组件设计模式

### 标签组件

零尺寸组件，用于分类实体：

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Bullet;
```

### 资源组件

全局单例数据：

```rust
#[derive(Resource)]
struct Time {
    delta_seconds: f32,
    elapsed_seconds: f64,
}

#[derive(Resource)]
struct AudioContext {
    device: AudioDevice,
    mixer: AudioMixer,
}
```

### 事件系统

组件级别的事件通信：

```rust
#[derive(Event)]
struct CollisionEvent {
    entity_a: Entity,
    entity_b: Entity,
    normal: Vec3,
}

// 发送事件
collision_events.send(CollisionEvent { ... });

// 监听事件
fn handle_collisions(mut events: EventReader<CollisionEvent>) {
    for event in events.iter() {
        // 处理碰撞
    }
}
```

## 系统优化模式

### 命令缓冲

延迟实体修改以避免集合修改：

```rust
fn spawn_bullets_system(
    mut commands: Commands,
    inputs: Res<ButtonInput<KeyCode>>,
) {
    if inputs.just_pressed(KeyCode::Space) {
        commands.spawn((
            Transform::default(),
            Velocity { lin: Vec3::new(0.0, 10.0, 0.0), ang: Vec3::ZERO },
            Bullet,
        ));
    }
}
```

### 变更检测

仅在组件发生变化时执行逻辑：

```rust
fn update_dirty_system(
    mut query: Query<&mut Sprite, Changed<Sprite>>,
) {
    for mut sprite in query.iter_mut() {
        // 只处理变更的精灵组件
        update_sprite_data(&mut sprite);
    }
}
```

### 实体过滤

使用Without和With约束查询：

```rust
// 查询有速度但无静态体的实体
Query<(&mut Transform, &Velocity), Without<StaticBody>>

// 查询玩家实体的武器
Query<&Weapon, With<Player>>
```

## 性能特性

### SIMD优化

批量处理同类型组件：

```rust
fn vectorized_update_system(
    mut query: Query<&mut Transform>,
) {
    query.par_iter_mut().for_each(|mut transform| {
        // SIMD友好的向量操作
        transform.pos += velocity * dt;
    });
}
```

### 内存布局

组件数据连续存储，支持高效访问：

```rust
// Archetype内存布局
#[repr(C)]
struct PositionComponents([f32; 3]); // 连续内存

#[repr(C)]
struct VelocityComponents([f32; 3]); // 连续内存
```

## 调试和监控

### 系统性能分析

```rust
#[derive(SystemSet)]
struct PhysicsSet;

app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
   .register_diagnostic(
       Diagnostic::new(FrameTimeDiagnosticsPlugin::FRAME_TIME)
   );
```

### 实体检查器

```rust
fn debug_entity_inspector_system(
    query: Query<Entity, With<Transform>>,
    world: &World,
) {
    for entity in query.iter() {
        let components = world.inspect_entity(entity);
        println!("Entity {:?} has components: {:?}", entity, components);
    }
}
```

## 最佳实践

### 组件设计

1. **小而专注**: 组件应该有单一职责
2. **数据导向**: 阐明业务逻辑在系统中
3. **复制友好**: 支持高效克隆和复制
4. **序列化友好**: 易于保存和加载

### 系统设计

1. **无副作用**: 系统不应修改无关数据
2. **模块化**: 单一系统做一件事情
3. **可重用**: 系统可以在不同应用中使用

### 查询优化

1. **缩小查询范围**: 使用具体组件组合
2. **利用变更检测**: 只处理需要更新的数据
3. **避免大型组合**: 分解过于复杂的查询

## 高级模式

### 层次实体

使用父子关系：

```rust
#[derive(Component)]
struct Parent(Entity);

#[derive(Component)]
struct Children(Vec<Entity>);

// 层次遍历系统
fn hierarchy_system(
    query: Query<(Entity, &Children, &Transform)>,
) {
    for (entity, children, transform) in query.iter() {
        for &child in children.iter() {
            inherit_transform(child, transform);
        }
    }
}
```

### 状态机

使用组件实现行为状态：

```rust
#[derive(Component)]
enum EnemyState {
    Idle,
    Alert,
    Attack,
    Flee,
}

fn enemy_ai_system(
    mut query: Query<(&Transform, &mut EnemyState)>,
) {
    for (transform, mut state) in query.iter_mut() {
        *state = match *state {
            EnemyState::Idle if detect_player(transform) => EnemyState::Alert,
            EnemyState::Alert if can_attack(transform) => EnemyState::Attack,
            _ => *state,
        };
    }
}
```

## 总结

ECS架构通过数据导向设计、内存连续性和查询优化，提供卓越的性能和灵活性，是现代游戏引擎的核心设计模式。引擎基于bevy_ecs实现，提供了完整的ECS功能集，同时保持了Rust的安全性和性能优势。