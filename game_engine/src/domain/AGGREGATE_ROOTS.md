# 聚合根设计文档

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
- 着色器（基础设施层）

### 业务规则

1. **渲染对象ID唯一性**：渲染对象ID必须唯一
2. **视锥体规则**：视锥体必须设置后才能更新场景
3. **LOD选择器规则**：LOD选择器必须配置后才能更新场景
4. **可见性规则**：只有可见的渲染对象才会被LOD处理
5. **渲染策略规则**：渲染对象必须属于有效的渲染策略

### 不变性约束

- `objects`：只能通过聚合根方法修改（`add_object`, `remove_object`, `update_object`）
- `frame_count`：只能通过聚合根方法递增（`new_frame`）

### 访问模式

**正确**：
```rust
// 通过聚合根添加渲染对象
render_scene.add_object(render_object)?;

// 通过聚合根更新场景
render_scene.update_scene(frustum, lod_selector)?;
```

**错误**：
```rust
// ❌ 直接访问聚合内部
render_scene.objects.push(render_object);

// ❌ 绕过验证
render_scene.frame_count += 1;
```

---

## 4. PhysicsWorld（物理世界聚合根）

### 边界定义

**包含**：
- `rigid_bodies`：刚体集合（`HashMap<RigidBodyId, RigidBody>`）
- `colliders`：碰撞体集合（`HashMap<ColliderId, Collider>`）
- `gravity`：重力向量
- `integration_parameters`：积分参数

**不包含**：
- Rapier物理引擎（基础设施层）
- 物理管线（基础设施层）
- 碰撞检测算法（基础设施层）

### 业务规则

1. **刚体ID唯一性**：刚体ID必须唯一
2. **碰撞体ID唯一性**：碰撞体ID必须唯一
3. **碰撞体关联规则**：碰撞体必须关联到有效的刚体
4. **质量规则**：动态刚体的质量必须为正数
5. **重力规则**：重力向量可以是任意值（包括零）
6. **物理步进规则**：物理步进必须使用有效的时间步长（> 0）

### 不变性约束

- `rigid_bodies`：只能通过聚合根方法修改（`add_rigid_body`, `remove_rigid_body`, `update_rigid_body`）
- `colliders`：只能通过聚合根方法修改（`add_collider`, `remove_collider`）
- `gravity`：可以通过聚合根方法修改（`set_gravity`）

### 访问模式

**正确**：
```rust
// 通过聚合根添加刚体
physics_world.add_rigid_body(rigid_body)?;

// 通过聚合根添加碰撞体
physics_world.add_collider(collider, rigid_body_id)?;

// 通过聚合根步进物理
physics_world.step(delta_time)?;
```

**错误**：
```rust
// ❌ 直接访问聚合内部
physics_world.rigid_bodies.insert(id, rigid_body);

// ❌ 绕过验证
physics_world.gravity = Vec3::ZERO;
```

---

## 5. AudioSource（音频源聚合根）

### 边界定义

**包含**：
- `AudioSourceId`：音频源唯一标识符
- `file_path`：音频文件路径
- `volume`：音量值对象
- `pitch`：音调
- `looping`：是否循环播放
- `state`：播放状态（Stopped, Playing, Paused）

**不包含**：
- 音频解码器（基础设施层）
- 音频设备（基础设施层）
- 音频缓冲区（基础设施层）

### 业务规则

1. **音频源ID唯一性**：音频源ID必须唯一
2. **文件路径规则**：文件路径必须指向有效的音频文件
3. **音量规则**：音量值必须在[0.0, 1.0]范围内
4. **音调规则**：音调值必须为正数
5. **状态转换规则**：状态转换必须遵循：Stopped → Playing → Paused → Stopped

### 不变性约束

- `AudioSourceId`：创建后不可变
- `file_path`：创建后不可变
- `state`：只能通过聚合根方法修改（`play`, `pause`, `stop`）
- `volume`：只能通过聚合根方法修改（`set_volume`）

### 访问模式

**正确**：
```rust
// 通过聚合根播放音频
audio_source.play()?;

// 通过聚合根设置音量
audio_source.set_volume(Volume::new(0.5))?;
```

**错误**：
```rust
// ❌ 直接访问内部状态
audio_source.state = AudioState::Playing;

// ❌ 绕过验证
audio_source.volume = Volume::new(2.0); // 超出范围
```

---

## 聚合根交互规则

### 1. 跨聚合引用

聚合根之间只能通过ID引用，不能直接持有引用：

**正确**：
```rust
// Scene通过EntityId引用GameEntity
scene.add_entity(entity)?;
let entity_id = entity.id;
```

**错误**：
```rust
// ❌ Scene直接持有GameEntity引用
struct Scene {
    entities: Vec<&GameEntity>, // ❌ 不允许
}
```

### 2. 事务边界

一个事务只能修改一个聚合：

**正确**：
```rust
// 修改Scene聚合
scene.add_entity(entity)?;

// 修改PhysicsWorld聚合（在另一个事务中）
physics_world.add_rigid_body(rigid_body)?;
```

**错误**：
```rust
// ❌ 在一个事务中修改多个聚合
fn update_scene_and_physics(scene: &mut Scene, physics: &mut PhysicsWorld) {
    scene.add_entity(entity)?;
    physics.add_rigid_body(rigid_body)?; // ❌ 违反事务边界
}
```

### 3. 业务规则执行

所有业务规则必须在聚合边界内执行：

**正确**：
```rust
impl Scene {
    pub fn add_entity(&mut self, entity: GameEntity) -> Result<(), DomainError> {
        // 业务规则验证在聚合边界内
        if self.entities.contains_key(&entity.id) {
            return Err(DomainError::Scene(SceneError::DuplicateEntity(entity.id)));
        }
        self.entities.insert(entity.id, entity);
        Ok(())
    }
}
```

**错误**：
```rust
// ❌ 业务规则在聚合外部执行
fn add_entity_to_scene(scene: &mut Scene, entity: GameEntity) {
    if scene.entities.contains_key(&entity.id) {
        return; // ❌ 业务规则应该在聚合内部
    }
    scene.entities.insert(entity.id, entity);
}
```

---

## 总结

所有聚合根都遵循以下原则：

1. **明确的边界**：每个聚合根都有明确的包含和不包含的内容
2. **业务规则封装**：所有业务规则都在聚合边界内执行
3. **不变性保证**：聚合根负责维护聚合内的不变性
4. **通过方法访问**：所有对聚合的访问都通过聚合根的方法
5. **ID引用**：聚合根之间只能通过ID引用，不能直接持有引用

这些原则确保了领域模型的一致性、可维护性和可测试性。

