# 聚合边界定义文档

本文档明确定义了游戏引擎领域层中所有聚合根的边界、业务规则和不变性约束。

## 聚合边界原则

1. **通过聚合根访问**：所有对聚合内实体的访问必须通过聚合根的方法
2. **事务边界**：聚合是事务边界，一个事务只能修改一个聚合
3. **引用其他聚合**：只能通过ID引用其他聚合，不能直接持有引用
4. **业务规则封装**：所有业务规则必须在聚合边界内执行
5. **不变性保证**：聚合根负责维护聚合内的不变性

---

## 1. Scene（场景聚合根）

### 边界定义

**包含**：
- `SceneId`：场景唯一标识符
- `name`：场景名称
- `state`：场景状态（Unloaded → Loading → Loaded → Active/Inactive → Unloaded）
- `entities`：场景中的实体集合（`HashMap<EntityId, GameEntity>`）
- `metadata`：场景元数据
- `recovery_strategy`：错误恢复策略

**不包含**：
- 渲染管线（基础设施层）
- 物理世界（基础设施层）
- ECS World（基础设施层）

### 业务规则

1. **场景名称规则**：场景名称不能为空
2. **状态转换规则**：状态转换必须遵循生命周期：Unloaded → Loading → Loaded → Active/Inactive → Unloaded
3. **实体ID唯一性**：场景内实体ID必须唯一
4. **相机数量限制**：活跃场景最多只能有一个活跃相机
5. **实体激活规则**：场景激活时，所有实体必须激活
6. **场景卸载规则**：场景卸载时，所有实体必须清除

### 不变性约束

- `SceneId`：创建后不可变
- `name`：创建后不可变
- `entities`：只能通过聚合根方法修改（`add_entity`, `remove_entity`）
- `state`：只能通过聚合根方法修改（`load`, `activate`, `deactivate`, `unload`）

### 访问模式

**正确**：
```rust
// 通过聚合根添加实体
scene.add_entity(entity)?;

// 通过聚合根获取实体
if let Some(entity) = scene.get_entity_mut(entity_id) {
    entity.set_position(position)?;
}
```

**错误**：
```rust
// ❌ 直接访问聚合内部
scene.entities.insert(entity_id, entity);

// ❌ 绕过业务规则
scene.state = SceneState::Active;
```

---

## 2. GameEntity（实体聚合根）

### 边界定义

**包含**：
- `EntityId`：实体唯一标识符
- `name`：实体名称（可选）
- `transform`：变换组件（可选）
- `sprite`：精灵渲染组件（可选）
- `point_light`：点光源组件（可选）
- `camera`：相机组件（可选）
- `properties`：自定义属性
- `state`：实体状态（Active, Inactive, PendingDeletion）

**不包含**：
- 渲染管线（基础设施层）
- 物理引擎（基础设施层）
- ECS组件（基础设施层）

### 业务规则

1. **组件冲突规则**：实体不能同时拥有`Sprite`和`Camera`组件
2. **缩放值规则**：`Transform`的缩放值必须为正数
3. **状态规则**：待删除的实体不能激活
4. **实体ID规则**：实体必须有ID
5. **变换一致性规则**：如果实体有`Transform`组件，缩放值必须为正数

### 不变性约束

- `EntityId`：创建后不可变
- `state`：只能通过聚合根方法修改（`activate`, `deactivate`, `mark_for_deletion`）
- 组件：只能通过聚合根方法修改（`with_transform`, `with_sprite`, `with_camera`等）

### 访问模式

**正确**：
```rust
// 通过聚合根方法修改
entity.set_position(position)?;
entity.move_by(delta)?;
entity.activate()?;
```

**错误**：
```rust
// ❌ 直接访问内部状态
entity.state = EntityState::Active;

// ❌ 绕过验证
entity.transform.as_mut().unwrap().scale = Vec3::ZERO;
```

---

## 3. RenderScene（渲染场景聚合根）

### 边界定义

**包含**：
- `objects`：渲染对象集合（`Vec<RenderObject>`）
- `lod_selector`：LOD选择器（可选）
- `frustum`：视锥体（可选）
- `frame_count`：帧计数器

**不包含**：
- GPU资源（基础设施层）
- 渲染管线（基础设施层）
- WGPU设备（基础设施层）

### 业务规则

1. **渲染对象ID唯一性**：渲染对象ID必须唯一
2. **视锥体规则**：视锥体必须设置后才能更新场景
3. **LOD选择器规则**：LOD选择器必须配置后才能更新场景
4. **可见性规则**：只有可见的渲染对象才会被LOD处理
5. **渲染对象状态规则**：渲染对象必须通过聚合根方法添加/移除

### 不变性约束

- `objects`：只能通过聚合根方法修改（`add_object`, `remove_object`）
- `lod_selector`：只能通过聚合根方法设置（`set_lod_selector`）
- `frustum`：只能通过聚合根方法设置（`set_frustum`）

### 访问模式

**正确**：
```rust
// 通过聚合根添加对象
scene.add_object(render_object)?;

// 通过聚合根更新场景
scene.update(delta_time, camera_pos)?;

// 通过聚合根获取对象
for obj in scene.renderable_objects() {
    // 使用对象
}
```

**错误**：
```rust
// ❌ 直接访问聚合内部
scene.objects.push(render_object);

// ❌ 绕过业务规则
scene.frustum = Some(frustum);
```

---

## 4. RigidBody（刚体聚合根）

### 边界定义

**包含**：
- `RigidBodyId`：刚体唯一标识符
- `body_type`：刚体类型（Dynamic, Kinematic, Static）
- `mass`：质量
- `velocity`：速度
- `angular_velocity`：角速度
- `colliders`：碰撞器集合

**不包含**：
- Rapier物理世界（基础设施层）
- 物理引擎（基础设施层）

### 业务规则

1. **刚体ID唯一性**：刚体ID必须唯一
2. **质量规则**：动态刚体必须有正质量
3. **碰撞器规则**：碰撞器必须通过聚合根方法添加
4. **状态一致性规则**：静态刚体不能有速度

### 不变性约束

- `RigidBodyId`：创建后不可变
- `colliders`：只能通过聚合根方法修改（`add_collider`, `remove_collider`）

### 访问模式

**正确**：
```rust
// 通过聚合根添加碰撞器
rigid_body.add_collider(collider)?;

// 通过聚合根设置速度
rigid_body.set_velocity(velocity)?;
```

**错误**：
```rust
// ❌ 直接访问聚合内部
rigid_body.colliders.push(collider);

// ❌ 绕过业务规则
rigid_body.velocity = velocity;
```

---

## 5. AudioSource（音频源聚合根）

### 边界定义

**包含**：
- `AudioSourceId`：音频源唯一标识符
- `audio_clip`：音频剪辑
- `volume`：音量
- `pitch`：音调
- `looping`：是否循环
- `spatial`：空间音频配置（可选）

**不包含**：
- 音频引擎（基础设施层）
- 音频设备（基础设施层）

### 业务规则

1. **音频源ID唯一性**：音频源ID必须唯一
2. **音量规则**：音量必须在0.0-1.0范围内
3. **音调规则**：音调必须为正数
4. **状态规则**：播放状态必须通过聚合根方法管理

### 不变性约束

- `AudioSourceId`：创建后不可变
- `volume`：只能通过聚合根方法修改（`set_volume`）
- `pitch`：只能通过聚合根方法修改（`set_pitch`）

### 访问模式

**正确**：
```rust
// 通过聚合根播放
audio_source.play()?;

// 通过聚合根设置音量
audio_source.set_volume(0.5)?;
```

**错误**：
```rust
// ❌ 直接访问聚合内部
audio_source.volume = 0.5;

// ❌ 绕过业务规则
audio_source.state = AudioState::Playing;
```

---

## 聚合间通信

### 通过ID引用

聚合之间只能通过ID引用，不能直接持有引用：

```rust
// ✅ 正确：通过ID引用
struct Scene {
    entities: HashMap<EntityId, GameEntity>,
}

// ❌ 错误：直接持有引用
struct Scene {
    entities: Vec<&GameEntity>, // 违反聚合边界
}
```

### 领域事件（未来实现）

聚合间通信应通过领域事件实现：

```rust
// 未来实现
pub enum DomainEvent {
    EntityCreated { entity_id: EntityId, scene_id: SceneId },
    EntityRemoved { entity_id: EntityId, scene_id: SceneId },
    SceneActivated { scene_id: SceneId },
    // ...
}
```

---

## 验证检查清单

在实现聚合根时，确保：

- [ ] 所有内部状态通过方法访问
- [ ] 业务规则在聚合边界内执行
- [ ] 不变性约束得到保证
- [ ] 聚合间只通过ID引用
- [ ] 错误处理符合领域错误类型
- [ ] 验证方法检查所有业务规则
- [ ] 补偿操作支持错误恢复

---

## 未来改进

1. **领域事件**：添加领域事件支持，用于聚合间通信
2. **快照机制**：完善快照机制，支持聚合状态恢复
3. **版本控制**：添加聚合版本控制，支持乐观锁
4. **审计日志**：添加聚合操作审计日志



