# 值对象使用指南

本文档说明值对象的使用情况和迁移计划。

## 已实现的值对象

### 基础值对象

1. **Position** - 3D位置
   - 验证：坐标必须有限（不能是NaN或无穷大）
   - 方法：`distance_to`, `offset`, `to_vec3`, `from_vec3`

2. **Rotation** - 旋转（四元数）
   - 验证：自动归一化
   - 方法：`combine`, `inverse`, `slerp`, `rotate_vec3`

3. **Scale** - 3D缩放
   - 验证：缩放值必须为正数
   - 方法：`combine`, `uniform`, `to_vec3`, `from_vec3`

4. **Transform** - 变换组合
   - 组合：Position + Rotation + Scale
   - 方法：`combine`, `with_position`, `with_rotation`, `with_scale`

### 物理值对象

5. **Mass** - 质量
   - 验证：质量必须为正数（或零，用于静态物体）
   - 方法：`value`, `is_zero`, `zero`

6. **Velocity** - 速度
   - 验证：速度分量必须有限
   - 方法：`magnitude`, `normalized`, `to_vec3`, `from_vec3`

### 音频值对象

7. **Volume** - 音量
   - 验证：音量必须在0.0-1.0范围内
   - 方法：`value`, `muted`, `max`, `is_muted`, `lerp`

### 时间值对象

8. **Duration** - 时长
   - 验证：时长必须为非负数
   - 方法：`seconds`, `millis`, `from_seconds`, `from_millis`

## 值对象使用原则

### 1. 优先使用值对象

**正确**：
```rust
use crate::domain::value_objects::{Position, Volume, Mass};

struct AudioSource {
    volume: Volume,  // ✅ 使用值对象
}

struct RigidBody {
    position: Position,  // ✅ 使用值对象
    mass: Mass,  // ✅ 使用值对象
}
```

**错误**：
```rust
struct AudioSource {
    volume: f32,  // ❌ 使用原始类型
}

struct RigidBody {
    position: Vec3,  // ❌ 使用原始类型
    mass: f32,  // ❌ 使用原始类型
}
```

### 2. 值对象验证

值对象在创建时进行验证，确保数据有效性：

```rust
// ✅ 验证通过
let volume = Volume::new(0.5)?;

// ❌ 验证失败
let volume = Volume::new(1.5)?;  // 返回None，超出范围

// ⚠️ 仅在确定有效时使用
let volume = Volume::new_unchecked(0.5);
```

### 3. 值对象转换

值对象提供与原始类型的转换方法：

```rust
// 从Vec3创建Position
let pos = Position::from_vec3(Vec3::new(1.0, 2.0, 3.0))?;

// 转换为Vec3
let vec = pos.to_vec3();

// 从f32创建Volume
let volume = Volume::new(0.8)?;

// 获取f32值
let value = volume.value();
```

## 迁移计划

### 阶段1：音频领域对象（已完成部分）

- [x] Volume值对象已实现
- [ ] AudioSource使用Volume值对象（当前使用f32）
- [ ] AudioListener使用Volume值对象

### 阶段2：物理领域对象（部分完成）

- [x] Mass值对象已实现
- [x] Velocity值对象已实现
- [ ] RigidBody使用Position值对象（当前使用Vec3）
- [ ] RigidBody使用Velocity值对象（当前使用Vec3）
- [ ] RigidBody使用Mass值对象（当前使用f32）

### 阶段3：实体领域对象

- [x] Position、Rotation、Scale值对象已实现
- [x] Transform值对象已实现
- [ ] GameEntity使用Transform值对象（当前使用Option<Transform>，但Transform是ECS类型）

### 阶段4：渲染领域对象

- [ ] RenderObject使用Position值对象
- [ ] RenderObject使用Rotation值对象
- [ ] RenderObject使用Scale值对象

## 值对象优势

1. **类型安全**：避免将位置和速度混淆
2. **验证保证**：确保数据有效性
3. **业务逻辑封装**：将领域概念封装为对象
4. **可读性**：代码更清晰，意图更明确
5. **可测试性**：值对象易于测试

## 示例：迁移前后对比

### 迁移前

```rust
struct AudioSource {
    volume: f32,  // 可能超出范围
}

fn set_volume(source: &mut AudioSource, value: f32) {
    // 需要手动验证
    if value < 0.0 || value > 1.0 {
        return;
    }
    source.volume = value;
}
```

### 迁移后

```rust
use crate::domain::value_objects::Volume;

struct AudioSource {
    volume: Volume,  // 自动验证
}

fn set_volume(source: &mut AudioSource, value: Volume) -> Result<(), DomainError> {
    // 值对象已经验证
    source.volume = value;
    Ok(())
}
```

## 注意事项

1. **性能考虑**：值对象是`Copy`类型，性能开销很小
2. **序列化**：值对象支持`Serialize`和`Deserialize`
3. **向后兼容**：迁移时需要考虑现有代码的兼容性
4. **渐进迁移**：可以逐步迁移，不需要一次性完成

## 总结

值对象是领域驱动设计的重要组成部分，通过封装领域概念和验证逻辑，提高了代码质量和可维护性。建议逐步将原始类型替换为值对象，确保领域模型的完整性和一致性。

