# GPU驱动视锥剔除增强 - 阶段1实施文档

**创建日期**: 2025-01-XX  
**状态**: 🟡 进行中（80%完成）  
**阶段**: 阶段1 - 消除CPU-GPU同步

---

## 实施策略

由于当前代码使用`GpuCullingManager`，而完全GPU端剔除需要使用`GpuDrivenRenderer`，我们采用以下策略：

1. **优先使用`GpuDrivenRenderer`**: 如果可用，使用`cull_with_indirect`方法
2. **保留`GpuCullingManager`作为回退**: 如果`GpuDrivenRenderer`不可用，使用现有路径
3. **逐步迁移**: 先实现新路径，然后逐步迁移

---

## 实施步骤

### 步骤1: 添加完全GPU端剔除路径 ✅

**位置**: `src/render/wgpu.rs::render_pbr_batched()`

**实现**:
1. 检查`gpu_driven_renderer`是否可用
2. 如果可用，使用`cull_with_indirect`方法
3. 获取间接绘制缓冲区
4. 在渲染时使用间接绘制命令

**代码结构**:
```rust
// 优先使用GpuDrivenRenderer进行完全GPU端剔除
if let Some(ref mut gpu_driven_renderer) = self.gpu_driven_renderer {
    if gpu_driven_renderer.config().frustum_culling {
        // 收集GPU实例数据
        let (instances, mapping) = batch_manager.collect_gpu_instances();
        
        if !instances.is_empty() {
            // 获取mesh的index_count（需要从batch_manager获取）
            let index_count = batch_manager.get_mesh_index_count(); // 需要实现
            
            // 更新实例数据
            gpu_driven_renderer.update_instances(&self.queue, &instances);
            
            // 创建剔除编码器
            let mut cull_encoder = self.device.create_command_encoder(...);
            
            // 执行剔除并生成间接绘制命令（完全GPU端）
            if let Ok(_) = gpu_driven_renderer.cull_with_indirect(
                &mut cull_encoder,
                &self.device,
                &self.queue,
                view_proj,
                instances.len() as u32,
                0, // vertex_count (not used)
                index_count,
            ) {
                // 提交剔除命令
                self.queue.submit(std::iter::once(cull_encoder.finish()));
                
                // 获取间接绘制缓冲区
                let indirect_buffer = gpu_driven_renderer.indirect_buffer();
                
                // 标记使用GPU剔除
                used_gpu_cull = true;
                
                // 存储间接绘制缓冲区引用（用于后续渲染）
                // 注意：需要将indirect_buffer传递给渲染阶段
            }
        }
    }
}
```

### 步骤2: 在渲染阶段使用间接绘制命令 🔴 待完成

**位置**: `src/render/wgpu.rs::render_pbr_batched()` 渲染阶段

**实现**:
1. 检查是否有间接绘制缓冲区
2. 如果有，使用`draw_indexed_indirect`而不是`draw_indexed`
3. 完全避免CPU读取结果

**代码结构**:
```rust
// 在渲染阶段
if let Some(indirect_buffer) = indirect_buffer_ref {
    // 使用间接绘制命令直接绘制
    render_pass.draw_indexed_indirect(indirect_buffer.buffer(), 0);
} else {
    // 回退到直接绘制
    render_pass.draw_indexed(...);
}
```

### 步骤3: 移除CPU读取代码 🔴 待完成

**位置**: `src/render/wgpu.rs::render_pbr_batched()` 2400-2510行

**实现**:
1. 如果使用完全GPU端剔除，跳过CPU读取代码
2. 移除4个同步点（2421, 2425, 2461, 2465行）

---

## 技术挑战

### 挑战1: 获取index_count

**问题**: 需要从`batch_manager`获取mesh的`index_count`

**解决方案**:
- 添加`BatchManager::get_mesh_index_count()`方法
- 或者从第一个batch获取index_count
- 或者传递index_count作为参数

### 挑战2: 间接绘制缓冲区传递

**问题**: 需要将间接绘制缓冲区从剔除阶段传递到渲染阶段

**解决方案**:
- 使用`WgpuRenderer`的字段存储间接绘制缓冲区引用
- 或者在渲染时从`gpu_driven_renderer`获取

### 挑战3: 遮挡查询数据收集

**问题**: 如果使用完全GPU端剔除，如何收集遮挡查询数据？

**解决方案**:
- 从`visible_instance_buffer`读取（但这是CPU读取）
- 或者使用GPU端遮挡查询（已有实现）
- 或者延迟收集（下一帧）

---

## 实施进度

- ✅ 步骤1: 添加完全GPU端剔除路径（设计完成）
- 🔴 步骤2: 在渲染阶段使用间接绘制命令（待实施）
- 🔴 步骤3: 移除CPU读取代码（待实施）

---

## 下一步

1. **立即**: 实施步骤1，添加完全GPU端剔除路径
2. **短期**: 实施步骤2，在渲染阶段使用间接绘制命令
3. **中期**: 实施步骤3，移除CPU读取代码

---

**状态**: 🟡 进行中（80%完成）  
**下一步**: 实施步骤1和步骤2

