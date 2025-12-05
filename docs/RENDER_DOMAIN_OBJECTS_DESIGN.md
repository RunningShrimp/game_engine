# 渲染领域对象设计文档

**设计日期**: 2025-01-XX  
**任务**: T1.1.2 - 设计渲染领域对象  
**状态**: ✅ 完成

---

## 执行摘要

本文档描述了为重构RenderService而设计的渲染领域对象。通过创建`LightSource`领域对象和增强`PbrScene`领域对象，将业务逻辑从服务层封装到领域层，符合DDD原则。

**关键成果**:
- ✅ 创建了`LightSource`领域对象，封装光源有效性验证
- ✅ 增强了`PbrScene`领域对象，添加业务逻辑方法
- ✅ 重构了`RenderService::build_pbr_scene()`，委托给领域对象
- ✅ 业务逻辑已从服务层移至领域层

---

## 1. LightSource领域对象

### 1.1 设计目标

封装光源的业务逻辑，包括：
- 光源有效性验证
- 光源类型判断
- 从ECS组件创建光源

### 1.2 业务规则

1. **点光源**:
   - 强度必须>0
   - 半径必须>0

2. **方向光**:
   - 强度必须>0
   - 方向会被归一化（不能为零向量）

3. **聚光灯**:
   - 强度必须>0
   - 半径必须>0
   - 内角必须<外角
   - 方向会被归一化（不能为零向量）

### 1.3 实现

```rust
pub enum LightSource {
    Point {
        position: Vec3,
        color: Vec3,
        intensity: f32,
        radius: f32,
    },
    Directional {
        direction: Vec3,
        color: Vec3,
        intensity: f32,
    },
    Spot {
        position: Vec3,
        direction: Vec3,
        color: Vec3,
        intensity: f32,
        inner_cutoff: f32,
        outer_cutoff: f32,
        radius: f32,
    },
}
```

### 1.4 主要方法

- `new_point_light()` - 创建点光源（带验证）
- `new_directional_light()` - 创建方向光（带验证）
- `new_spot_light()` - 创建聚光灯（带验证）
- `is_valid()` - 验证光源有效性
- `from_ecs_point_light()` - 从ECS组件创建点光源
- `from_ecs_directional_light()` - 从ECS组件创建方向光

---

## 2. PbrScene领域对象增强

### 2.1 设计目标

封装PBR渲染场景的业务逻辑，包括：
- 光源管理
- 场景验证
- 从ECS构建场景

### 2.2 业务规则

1. **允许无光源渲染**: 场景可以没有光源（可能有环境光）
2. **只添加有效光源**: 通过`LightSource`验证
3. **场景验证**: 检查所有光源的有效性

### 2.3 新增方法

- `add_light()` - 添加光源（带验证）
- `validate()` - 验证场景有效性
- `from_ecs_world()` - 从ECS世界构建场景
- `is_empty()` - 判断场景是否为空
- `light_count()` - 获取光源总数

### 2.4 实现

```rust
pub struct PbrScene {
    point_lights: Vec<PointLight3D>,
    dir_lights: Vec<DirectionalLight>,
}
```

---

## 3. RenderService重构

### 3.1 build_pbr_scene()重构

**重构前**:
```rust
pub fn build_pbr_scene(&mut self, world: &mut World) -> PbrScene {
    // 业务规则分散在服务层
    if light.intensity > 0.0 && light.radius > 0.0 {
        // 添加光源
    }
    // ...
}
```

**重构后**:
```rust
pub fn build_pbr_scene(&mut self, world: &mut World) -> PbrScene {
    // 委托给领域对象，业务逻辑已封装
    let domain_scene = DomainPbrScene::from_ecs_world(world);
    
    // 转换为基础设施层的数据结构
    PbrScene {
        point_lights: domain_scene.point_lights().to_vec(),
        dir_lights: domain_scene.dir_lights().to_vec(),
    }
}
```

### 3.2 paint_pbr()简化

**重构前**:
```rust
// 业务规则：验证场景有效性
if scene.point_lights.is_empty() && scene.dir_lights.is_empty() {
    // 允许无光源渲染
}
```

**重构后**:
```rust
// 业务规则已封装在领域对象中
// 场景在build_pbr_scene中已通过领域对象构建和验证
// 这里只需要调用基础设施层进行渲染
```

---

## 4. 架构改进

### 4.1 业务逻辑封装

**改进前**:
- ❌ 业务规则分散在`RenderService`中
- ❌ 光源有效性验证在服务层
- ❌ 场景验证逻辑在服务层

**改进后**:
- ✅ 业务规则封装在领域对象中
- ✅ 光源有效性验证在`LightSource`中
- ✅ 场景验证逻辑在`PbrScene`中

### 4.2 职责分离

**改进前**:
- `RenderService`既做数据转换，又做业务验证

**改进后**:
- `RenderService`只负责协调和调用基础设施
- 领域对象负责业务逻辑和验证

---

## 5. 使用示例

### 5.1 创建光源

```rust
use game_engine::domain::render::LightSource;
use glam::Vec3;

// 创建有效的点光源
let light = LightSource::new_point_light(
    Vec3::ONE,
    Vec3::ONE,
    1.0,
    10.0,
).unwrap();

// 验证光源有效性
assert!(light.is_valid());
```

### 5.2 构建PBR场景

```rust
use game_engine::domain::render::PbrScene;
use bevy_ecs::prelude::*;

// 从ECS构建场景
let mut world = World::new();
let scene = PbrScene::from_ecs_world(&mut world);

// 验证场景
assert!(scene.validate().is_ok());
```

### 5.3 在RenderService中使用

```rust
// RenderService现在委托给领域对象
let domain_scene = DomainPbrScene::from_ecs_world(world);

// 转换为基础设施层的数据结构
let pbr_scene = PbrScene {
    point_lights: domain_scene.point_lights().to_vec(),
    dir_lights: domain_scene.dir_lights().to_vec(),
};
```

---

## 6. 测试计划

### 6.1 LightSource测试

- [ ] 测试点光源创建（有效/无效）
- [ ] 测试方向光创建（有效/无效）
- [ ] 测试聚光灯创建（有效/无效）
- [ ] 测试从ECS组件创建光源
- [ ] 测试光源有效性验证

### 6.2 PbrScene测试

- [ ] 测试添加光源（有效/无效）
- [ ] 测试场景验证
- [ ] 测试从ECS构建场景
- [ ] 测试空场景处理
- [ ] 测试光源计数

### 6.3 RenderService集成测试

- [ ] 测试build_pbr_scene使用领域对象
- [ ] 测试paint_pbr使用领域对象
- [ ] 测试端到端渲染流程

---

## 7. 后续改进

### 7.1 短期改进

1. **添加单元测试**: 为`LightSource`和`PbrScene`添加完整的单元测试
2. **错误处理增强**: 改进错误消息，提供更多上下文
3. **文档完善**: 添加更多使用示例和最佳实践

### 7.2 长期改进

1. **聚光灯支持**: 完善聚光灯的领域对象支持
2. **光源影响范围**: 添加光源影响范围计算业务逻辑
3. **光源分组**: 添加光源分组和优先级管理

---

## 8. 结论

通过创建`LightSource`领域对象和增强`PbrScene`领域对象，成功将业务逻辑从服务层封装到领域层。`RenderService`现在符合DDD原则，只负责协调和调用基础设施，业务逻辑由领域对象负责。

**下一步**: 执行任务T1.1.3 - 重构RenderService为领域服务，添加更多业务逻辑方法。

---

**设计完成日期**: 2025-01-XX  
**下一步**: T1.1.3 - 重构RenderService为领域服务

