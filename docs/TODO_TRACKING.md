# TODO 跟踪文档

本文档跟踪游戏引擎项目中的所有 TODO 项、FIXME、XXX 和 HACK 标记。

## 相关文档

- [重构总结](./REFACTORING_SUMMARY.md) - 最近重构的详细记录
- [协程迁移评估](./COROUTINE_MIGRATION_ASSESSMENT.md) - AI 寻路模块协程迁移评估
- [资源管理优化计划](./RESOURCE_OPTIMIZATION_PLAN.md) - 资源管理系统优化建议
- [性能监控增强计划](./PERFORMANCE_MONITORING_ENHANCEMENTS.md) - 性能监控系统增强建议

## 概述

| 状态 | 数量 |
|------|------|
| 待处理 | 0 |
| 已完成 | 6 |
| 高优先级 | 0 |
| 中优先级 | 0 |
| 低优先级 | 0 |

## 已完成

### 1. 实现 egui 渲染器
- **位置**: `game_engine/src/core/engine/renderer.rs:290`
- **描述**: 修复了 egui 渲染器参数传递，将 None 改为传入正确的 egui_renderer 参数
- **优先级**: 高
- **完成日期**: 2024-12-24
- **状态**: ✅ 已完成

**修复内容**:
```rust
// 修复前:
None, // TODO: Implement proper egui renderer

// 修复后:
egui_renderer,
```

### 2. 实现重绘请求处理
- **位置**: `game_engine/src/core/engine/input_handler.rs:66`
- **描述**: 实现了 RedrawRequested 事件的处理，将事件添加到输入缓冲区
- **优先级**: 高
- **完成日期**: 2024-12-24
- **状态**: ✅ 已完成

**修改内容**:
```rust
// 添加到 InputEvent 枚举:
RedrawRequested,

// 实现事件处理:
WindowEvent::RedrawRequested => {
    if let Some(mut buf) = world.get_resource_mut::<InputBuffer>() {
        buf.events.push(InputEvent::RedrawRequested);
        tracing::debug!(target: "input", "Redraw requested");
    }
}
```

### 3. 实现 QueryPipeline
- **位置**: `game_engine/src/domain/physics.rs`
- **描述**: 使用 rapier3d 的 `broad_phase.as_query_pipeline()` 实现了高效的射线投射功能
- **优先级**: 中
- **完成日期**: 2024-12-24
- **状态**: ✅ 已完成

**实现内容**:
```rust
pub fn raycast(
    &self,
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<(RigidBodyId, f32, Vec3)> {
    let ray = Ray::new(
        Point3::new(origin.x, origin.y, origin.z),
        Vector3::new(direction.x, direction.y, direction.z),
    );

    let filter = rapier3d::prelude::QueryFilter::default();
    let query_pipeline = self.broad_phase.as_query_pipeline(
        self.narrow_phase.query_dispatcher(),
        &self.rigid_body_set,
        &self.collider_set,
        filter,
    );

    let max_toi = max_distance / direction.length();

    if let Some((collider_handle, toi)) = query_pipeline.cast_ray(&ray, max_toi, true) {
        // ...
    }
}
```

### 4. 实现 GPU 检测
- **位置**: `game_engine_hardware/src/gpu/detect.rs`
- **描述**: 实现了完整的 GPU 检测功能，包括厂商识别、性能分级和特性检测
- **优先级**: 中
- **完成日期**: 2024-12-24
- **状态**: ✅ 已完成

**实现内容**:
- 使用 wgpu API 枚举适配器
- 识别 GPU 厂商（Nvidia、AMD、Intel、Apple 等）
- 根据 VRAM 和纹理大小进行性能分级
- 添加 `driver_info` 字段到 `GpuInfo`
- 实现射线追踪和网格着色器支持检测

### 5. 资源管理优化
- **位置**: `docs/RESOURCE_OPTIMIZATION_PLAN.md`
- **描述**: 创建了资源管理系统的优化计划文档，包括统一资源接口、热重载、依赖管理、流式加载和压缩缓存
- **优先级**: 低
- **完成日期**: 2024-12-24
- **状态**: ✅ 已完成

**文档内容**:
- 统一资源接口设计
- 资源热重载实现
- 资源依赖管理
- 资源流式加载
- 资源压缩和缓存
- 详细的实施计划和时间表

### 6. 性能监控增强
- **位置**: `docs/PERFORMANCE_MONITORING_ENHANCEMENTS.md`
- **描述**: 创建了性能监控系统的增强计划文档，包括实时仪表盘、性能热力图、帧时间分析、GPU 监控、回归检测和报告生成
- **优先级**: 低
- **完成日期**: 2024-12-24
- **状态**: ✅ 已完成

**文档内容**:
- 实时性能仪表盘设计
- 性能热力图实现
- 帧时间分布分析
- GPU 性能监控
- 性能回归检测
- 性能报告生成器
- 详细的实施计划和时间表

## 之前已完成的重构任务

以下任务已在之前的重构工作中完成：

- [x] 创建 game_engine_common 共享 crate
- [x] 将 optimization_validation.rs 移动到共享 crate
- [x] 将 synchronized.rs 移动到共享 crate
- [x] 更新受影响 crate 的依赖配置
- [x] 删除所有重复实现
- [x] 创建 resources/gltf_loader.rs 模块
- [x] 重构 resources/manager.rs 使用新模块
- [x] 创建统一的 KeyExchange trait
- [x] 重构 network/key_exchange.rs 使用统一接口
- [x] 运行 cargo build 和 cargo test 验证更改
- [x] 补充公共 API 文档
- [x] 建立 TODO 跟踪文档
- [x] 评估 AI 寻路模块迁移到协程

详细记录请参考 [重构总结](./REFACTORING_SUMMARY.md)。

## 创建的文档

| 文档 | 描述 |
|------|------|
| `TODO_TRACKING.md` | 本文档 - TODO 项跟踪 |
| `REFACTORING_SUMMARY.md` | 代码重构总结 |
| `COROUTINE_MIGRATION_ASSESSMENT.md` | AI 寻路模块协程迁移评估 |
| `RESOURCE_OPTIMIZATION_PLAN.md` | 资源管理优化计划 |
| `PERFORMANCE_MONITORING_ENHANCEMENTS.md` | 性能监控增强计划 |

## 清理周期

TODO 项的清理周期为：
- 每周审查一次高优先级项
- 每月审查一次中优先级项
- 每季度审查一次低优先级项

## 添加新 TODO 的指南

1. 添加 TODO 时，确保包含清晰的描述
2. 使用 `TODO: <description>` 格式
3. 如有截止日期，在描述中注明
4. 将新 TODO 添加到此文档中

## 完成标准

TODO 被认为完成的标准：
1. 相关代码已实现并经过测试
2. 文档已更新（如适用）
3. 本文档中的状态已更新为"已完成"
