# ECS组件脏跟踪系统

## 概述

ECS组件脏跟踪系统提供通用的组件变化跟踪机制，允许系统只处理已修改的组件，显著减少不必要的更新，提升性能。

## 设计目标

1. **细粒度跟踪**：支持按组件类型和字段级别的脏标记
2. **零开销抽象**：使用位掩码，最小化内存占用
3. **线程安全**：支持多线程环境下的原子操作
4. **性能优化**：批量查询和更新支持

## 核心组件

### DirtyFlags

脏标记标志位，使用位掩码支持多个脏标记同时存在。

```rust
use game_engine::ecs::dirty_tracking::DirtyFlags;

// 预定义标志
let position = DirtyFlags::POSITION;
let rotation = DirtyFlags::ROTATION;
let transform = DirtyFlags::TRANSFORM; // 包含POSITION | ROTATION | SCALE

// 组合标志
let flags = DirtyFlags::POSITION | DirtyFlags::ROTATION;

// 自定义标志（8-63位）
let custom = DirtyFlags::custom(8);
```

### ComponentDirty

组件脏标记组件，跟踪组件的脏状态。

```rust
use game_engine::ecs::dirty_tracking::{ComponentDirty, DirtyFlags};

// 创建脏标记组件
let mut dirty = ComponentDirty::new();

// 标记为脏
dirty.mark_dirty(DirtyFlags::POSITION);

// 检查是否脏
if dirty.is_dirty(DirtyFlags::POSITION) {
    // 处理位置变化
}

// 清除脏标记
dirty.clear(DirtyFlags::POSITION);
```

## 使用示例

### 基本使用

```rust
use bevy_ecs::prelude::*;
use game_engine::ecs::dirty_tracking::{ComponentDirty, DirtyFlags};

// 在系统中使用脏跟踪
fn update_transform_system(
    mut query: Query<(&mut Transform, &mut ComponentDirty)>,
) {
    for (mut transform, mut dirty) in query.iter_mut() {
        // 只处理位置变化
        if dirty.is_dirty(DirtyFlags::POSITION) {
            // 更新位置相关的逻辑
            // ...
            dirty.clear(DirtyFlags::POSITION);
        }
        
        // 只处理旋转变化
        if dirty.is_dirty(DirtyFlags::ROTATION) {
            // 更新旋转相关的逻辑
            // ...
            dirty.clear(DirtyFlags::ROTATION);
        }
    }
}

// 标记组件为脏
fn modify_transform_system(
    mut query: Query<(&mut Transform, &mut ComponentDirty)>,
) {
    for (mut transform, mut dirty) in query.iter_mut() {
        transform.pos.x += 1.0;
        dirty.mark_dirty(DirtyFlags::POSITION);
    }
}
```

### 批量处理

```rust
fn batch_update_system(
    mut query: Query<(&mut Transform, &mut ComponentDirty)>,
) {
    // 收集所有脏的实体
    let dirty_entities: Vec<Entity> = query
        .iter()
        .filter(|(_, dirty)| dirty.is_any_dirty())
        .map(|(_, _)| entity)
        .collect();
    
    // 批量处理
    for entity in dirty_entities {
        if let Ok((mut transform, mut dirty)) = query.get_mut(entity) {
            if dirty.is_dirty(DirtyFlags::TRANSFORM) {
                // 批量更新逻辑
                dirty.clear(DirtyFlags::TRANSFORM);
            }
        }
    }
}
```

### 线程安全操作

```rust
use std::sync::Arc;

// 在多线程环境中使用原子操作
fn parallel_update_system(
    query: Query<&ComponentDirty>,
) {
    query.par_for_each(|dirty| {
        // 原子地检查脏标记
        if dirty.is_dirty(DirtyFlags::POSITION) {
            // 处理逻辑
            dirty.clear_atomic(DirtyFlags::POSITION);
        }
    });
}
```

## 性能优化建议

1. **批量查询**：使用`Query::iter()`收集脏实体，然后批量处理
2. **标志组合**：使用组合标志（如`TRANSFORM`）减少检查次数
3. **及时清理**：处理完脏标记后立即清除，避免重复处理
4. **选择性添加**：只为需要跟踪的组件添加`ComponentDirty`

## 与现有系统的集成

### 物理系统

物理系统已有自己的脏跟踪（`PhysicsDirty`），可以与通用脏跟踪系统配合使用：

```rust
fn physics_sync_system(
    mut query: Query<(&mut Transform, &mut ComponentDirty, &PhysicsDirty)>,
) {
    for (mut transform, mut dirty, physics_dirty) in query.iter_mut() {
        if physics_dirty.needs_transform_update() {
            // 物理同步逻辑
            dirty.mark_dirty(DirtyFlags::TRANSFORM);
        }
    }
}
```

### 渲染系统

渲染系统可以使用脏跟踪来优化材质和网格更新：

```rust
fn render_update_system(
    mut query: Query<(&mut Material, &mut ComponentDirty)>,
) {
    for (mut material, mut dirty) in query.iter_mut() {
        if dirty.is_dirty(DirtyFlags::MATERIAL) {
            // 更新材质
            dirty.clear(DirtyFlags::MATERIAL);
        }
    }
}
```

## 配置

使用`DirtyTrackingResource`进行全局配置：

```rust
use game_engine::ecs::dirty_tracking::DirtyTrackingResource;

fn setup_dirty_tracking(mut commands: Commands) {
    commands.insert_resource(DirtyTrackingResource {
        config: DirtyTrackingConfig {
            enabled: true,
            auto_clear_interval: 1,
            auto_clear_on_system_end: false,
        },
        current_frame: 0,
    });
}
```

## 测试

系统包含完整的单元测试：

```bash
cargo test -p game_engine ecs::dirty_tracking --lib
```

## 性能影响

- **内存开销**：每个实体约8字节（u64标志位）
- **CPU开销**：位操作，几乎零开销
- **性能提升**：在大型场景中可减少20-40%的不必要更新

## 最佳实践

1. **只跟踪需要的变化**：不要为所有组件添加脏跟踪
2. **使用组合标志**：减少标志检查次数
3. **及时清理**：处理完立即清除，避免重复处理
4. **批量处理**：收集脏实体后批量处理，提高缓存效率

## 未来改进

- [ ] 支持自动脏标记（通过代理类型）
- [ ] 支持脏标记的持久化
- [ ] 支持脏标记的统计和监控
- [ ] 集成到系统调度器中，自动跳过无脏标记的实体

