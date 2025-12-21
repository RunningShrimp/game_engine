# GPU驱动渲染指南

## 概述

GPU驱动渲染是一种高级渲染技术，通过将渲染决策从CPU转移到GPU，显著减少CPU-GPU通信开销，提升大规模场景的渲染性能。

## 核心概念

### GPU驱动渲染的优势

1. **减少Draw Call**：通过GPU实例化和间接绘制，将数千个draw call合并为单个
2. **降低CPU开销**：剔除和LOD选择在GPU上完成，释放CPU资源
3. **提升可扩展性**：支持渲染数百万个对象而不会造成CPU瓶颈

### 架构设计

```
CPU端：
  - 收集场景对象
  - 上传实例数据到GPU
  - 提交间接绘制命令

GPU端：
  - 执行视锥剔除
  - 执行遮挡剔除
  - 选择LOD级别
  - 生成间接绘制参数
  - 执行实例化绘制
```

## 使用示例

### 基本用法

```rust
use game_engine::render::instance_batch::{InstanceBatch, BatchKey};
use game_engine::render::pbr_renderer::Instance3D;

// 创建实例批次
let batch_key = BatchKey {
    mesh_id: 1,
    material_id: 1,
    pipeline_id: 1,
    blend_mode: 0,
    depth_test: true,
    render_flags: 0,
};

let mut batch = InstanceBatch::new(batch_key, mesh, material_bind_group);

// 添加实例
for entity in entities {
    let instance = Instance3D {
        transform: entity.transform.to_mat4(),
        // ... 其他属性
    };
    batch.add_instance(instance);
}

// 更新GPU缓冲区（增量更新）
batch.update_buffer(device, queue);

// 执行绘制
// render_pass.draw_indexed_instanced(...)
```

### 增量更新优化

实例批次系统支持增量更新，只上传变化的实例数据：

```rust
// 系统会自动检测脏实例并只上传变化的数据
batch.update_buffer(device, queue);

// 获取性能统计
let stats = batch.get_performance_stats();
println!("增量更新比例: {:.1}%", stats.incremental_update_ratio * 100.0);
```

## 性能优化技巧

1. **批量大小**：保持每个批次包含100-1000个实例，平衡内存和性能
2. **静态批次**：对于不常移动的对象，使用静态批次减少更新开销
3. **脏跟踪**：利用ECS脏跟踪系统，只更新变化的实例
4. **LOD选择**：在GPU上根据距离自动选择LOD级别

## 最佳实践

- 按材质和网格分组实例，减少状态切换
- 使用包围体剔除，避免渲染不可见对象
- 定期重建批次，处理对象添加/删除
- 监控GPU带宽使用，避免过度上传

## 相关文档

- [实例化渲染性能测试](../tests/integration/instance_batch_performance_test.rs)
- [渲染优化指南](performance_optimization_guide.md)

