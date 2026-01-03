//  增强的间接绘制优化系统
//
//  ## 主要特性
//
//  1. **批处理优化（Batching）**
//     - 自动合并相似的绘制调用
//     - 减少状态切换
//     - 优化资源绑定
//
//  2. **实例化渲染（Instancing）**
//     - GPU驱动的实例合并
//     - 动态实例筛选
//     - 批量实例更新
//
//  3. **多绘制间接（Multi-Draw Indirect）**
//     - 单次调用多个绘制
//     - 减少CPU开销
//     - 支持WebGPU后端

use crate::render::gpu_driven::culling::GpuInstance;
use crate::render::gpu_driven::indirect::DrawIndexedIndirectArgs;
use std::collections::HashMap;
use wgpu::{Buffer, Device, Queue};

// ============================================================================
// 批处理优化
// ============================================================================

/// 批处理键（用于分组相似的绘制调用）
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct BatchKey {
    /// 管线哈希
    pub pipeline_hash: u64,
    /// 材质ID
    pub material_id: u32,
    /// 纹理绑定组哈希
    pub texture_hash: u64,
}

impl BatchKey {
    /// 创建新的批处理键
    pub fn new(pipeline_hash: u64, material_id: u32, texture_hash: u64) -> Self {
        Self {
            pipeline_hash,
            material_id,
            texture_hash,
        }
    }
}

/// 批次信息
#[derive(Debug, Clone)]
pub struct BatchInfo {
    /// 批次键
    pub key: BatchKey,
    /// 实例索引列表
    pub instance_indices: Vec<u32>,
    /// 批次AABB（用于剔除）
    pub aabb_min: [f32; 3],
    pub aabb_max: [f32; 3],
}

impl BatchInfo {
    /// 创建新的批次
    pub fn new(key: BatchKey) -> Self {
        Self {
            key,
            instance_indices: Vec::new(),
            aabb_min: [f32::MAX; 3],
            aabb_max: [f32::MIN; 3],
        }
    }

    /// 添加实例到批次
    pub fn add_instance(&mut self, instance_idx: u32, instance: &GpuInstance) {
        self.instance_indices.push(instance_idx);

        // 更新批次AABB
        for i in 0..3 {
            self.aabb_min[i] = self.aabb_min[i].min(instance.aabb_min[i]);
            self.aabb_max[i] = self.aabb_max[i].max(instance.aabb_max[i]);
        }
    }

    /// 获取实例数量
    pub fn instance_count(&self) -> u32 {
        self.instance_indices.len() as u32
    }

    /// 是否为空
    pub fn is_empty(&self) -> bool {
        self.instance_indices.is_empty()
    }
}

/// 批处理优化器
///
/// 将相似的绘制调用合并为批次，减少状态切换。
pub struct BatchingOptimizer {
    /// 当前批次的映射表
    batches: HashMap<BatchKey, BatchInfo>,
    /// 批处理统计
    stats: BatchingStats,
    /// 是否启用批处理
    enabled: bool,
    /// 最大批次大小（实例数）
    max_batch_size: u32,
}

#[derive(Debug, Clone, Default)]
pub struct BatchingStats {
    /// 总绘制调用数
    pub total_draw_calls: u32,
    /// 批次数量
    pub batch_count: u32,
    /// 合并的绘制调用数
    pub merged_draw_calls: u32,
    /// 平均批次大小
    pub avg_batch_size: f32,
}

impl BatchingOptimizer {
    /// 创建批处理优化器
    pub fn new(enabled: bool, max_batch_size: u32) -> Self {
        Self {
            batches: HashMap::new(),
            stats: BatchingStats::default(),
            enabled,
            max_batch_size,
        }
    }

    /// 清空批次
    pub fn clear(&mut self) {
        self.batches.clear();
        self.stats = BatchingStats::default();
    }

    /// 添加绘制调用到批次
    ///
    /// # 参数
    ///
    /// - `key`: 批次键
    /// - `instance_idx`: 实例索引
    /// - `instance`: 实例数据
    pub fn add_draw_call(&mut self, key: BatchKey, instance_idx: u32, instance: &GpuInstance) {
        if !self.enabled {
            self.stats.total_draw_calls += 1;
            return;
        }

        self.stats.total_draw_calls += 1;

        // 查找或创建批次
        let batch = self
            .batches
            .entry(key.clone())
            .or_insert_with(|| BatchInfo::new(key));

        // 如果批次已满，创建新批次
        if batch.instance_count() >= self.max_batch_size {
            let new_key = BatchKey::new(
                key.pipeline_hash + 1, // 修改哈希以创建新批次
                key.material_id,
                key.texture_hash,
            );
            let new_batch = self
                .batches
                .entry(new_key.clone())
                .or_insert_with(|| BatchInfo::new(new_key));
            new_batch.add_instance(instance_idx, instance);
        } else {
            batch.add_instance(instance_idx, instance);
        }
    }

    /// 完成批处理并生成间接绘制命令
    ///
    /// # 参数
    ///
    /// - `device`: WGPU设备
    /// - `queue`: WGPU队列
    /// - `index_count`: 每个实例的索引数
    ///
    /// # 返回
    ///
    /// 返回生成的间接绘制命令列表。
    pub fn finalize(
        &mut self,
        _device: &Device,
        _queue: &Queue,
        index_count: u32,
    ) -> Vec<DrawIndexedIndirectArgs> {
        let mut commands = Vec::new();

        for (_, batch) in &self.batches {
            if batch.is_empty() {
                continue;
            }

            commands.push(DrawIndexedIndirectArgs {
                index_count,
                instance_count: batch.instance_count(),
                first_index: 0,
                base_vertex: 0,
                first_instance: 0, // 将在渲染时更新
            });
        }

        // 更新统计
        self.stats.batch_count = commands.len() as u32;
        self.stats.merged_draw_calls = self.stats.total_draw_calls - commands.len() as u32;
        self.stats.avg_batch_size = if self.stats.batch_count > 0 {
            self.stats.total_draw_calls as f32 / self.stats.batch_count as f32
        } else {
            0.0
        };

        commands
    }

    /// 获取批处理统计
    pub fn stats(&self) -> &BatchingStats {
        &self.stats
    }

    /// 获取批次数量
    pub fn batch_count(&self) -> usize {
        self.batches.len()
    }
}

// ============================================================================
// 实例化渲染优化
// ============================================================================

/// 实例化渲染优化器
///
/// 优化实例化渲染，减少GPU内存带宽。
pub struct InstancingOptimizer {
    /// 实例数据缓冲区
    instance_buffers: Vec<Buffer>,
    /// 当前缓冲区容量
    buffer_capacity: usize,
    /// 实例化统计
    stats: InstancingStats,
    /// 是否启用
    enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct InstancingStats {
    /// 总实例数
    pub total_instances: u32,
    /// 实例化批次数
    pub instanced_batches: u32,
    /// 平均每批次实例数
    pub avg_instances_per_batch: f32,
    /// 节省的绘制调用数
    pub saved_draw_calls: u32,
}

impl InstancingOptimizer {
    /// 创建实例化优化器
    pub fn new(enabled: bool, initial_capacity: usize, device: &Device) -> Self {
        // 创建初始实例缓冲区
        let buffer_size =
            (std::mem::size_of::<GpuInstance>() * initial_capacity) as wgpu::BufferAddress;

        let instance_buffers = vec![device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instancing Optimizer Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        })];

        Self {
            instance_buffers,
            buffer_capacity: initial_capacity,
            stats: InstancingStats::default(),
            enabled,
        }
    }

    /// 准备实例化渲染
    ///
    /// 将实例数据分组并上传到GPU。
    ///
    /// # 参数
    ///
    /// - `device`: WGPU设备
    /// - `queue`: WGPU队列
    /// - `instances`: 实例数据切片
    /// - `batch_keys`: 批次键列表（每个实例对应的批次）
    ///
    /// # 返回
    ///
    /// 返回每个批次的实例范围。
    pub fn prepare_instancing(
        &mut self,
        device: &Device,
        queue: &Queue,
        instances: &[GpuInstance],
        batch_keys: &[BatchKey],
    ) -> Result<Vec<(usize, usize)>, &'static str> {
        if !self.enabled {
            return Err("Instancing not enabled");
        }

        if instances.len() != batch_keys.len() {
            return Err("Instance count and batch key count mismatch");
        }

        // 按批次键分组
        let mut batches: HashMap<BatchKey, Vec<usize>> = HashMap::new();
        for (idx, key) in batch_keys.iter().enumerate() {
            batches
                .entry(key.clone())
                .or_insert_with(Vec::new)
                .push(idx);
        }

        // 确保缓冲区容量足够
        let total_instances = instances.len();
        if total_instances > self.buffer_capacity {
            self.expand_buffers(device, total_instances);
        }

        // 上传实例数据
        if let Some(buffer) = self.instance_buffers.first() {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(instances));
        }

        // 更新统计
        self.stats.total_instances = instances.len() as u32;
        self.stats.instanced_batches = batches.len() as u32;
        self.stats.avg_instances_per_batch = if batches.len() > 0 {
            instances.len() as f32 / batches.len() as f32
        } else {
            0.0
        };
        self.stats.saved_draw_calls = instances.len() as u32 - batches.len() as u32;

        // 返回每个批次的实例范围
        let mut ranges = Vec::new();
        let mut offset = 0;
        for (_, indices) in &batches {
            let start = offset;
            let end = offset + indices.len();
            ranges.push((start, end));
            offset = end;
        }

        Ok(ranges)
    }

    /// 扩展缓冲区
    fn expand_buffers(&mut self, device: &Device, required_capacity: usize) {
        let new_capacity = (required_capacity * 3 / 2).max(64);
        let buffer_size =
            (std::mem::size_of::<GpuInstance>() * new_capacity) as wgpu::BufferAddress;

        self.instance_buffers = vec![device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instancing Optimizer Buffer (Expanded)"),
            size: buffer_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        })];

        self.buffer_capacity = new_capacity;
    }

    /// 获取实例缓冲区
    pub fn instance_buffer(&self) -> Option<&Buffer> {
        self.instance_buffers.first()
    }

    /// 获取统计
    pub fn stats(&self) -> &InstancingStats {
        &self.stats
    }
}

// ============================================================================
// 多绘制间接优化
// ============================================================================

/// 多绘制间接优化器
///
/// 支持单次调用执行多个绘制。
pub struct MultiDrawIndirectOptimizer {
    /// 间接绘制命令列表
    commands: Vec<DrawIndexedIndirectArgs>,
    /// 最大命令数
    max_commands: usize,
    /// 多绘制统计
    stats: MultiDrawStats,
    /// 是否启用
    enabled: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MultiDrawStats {
    /// 总绘制数
    pub total_draws: u32,
    /// 多绘制调用数
    pub multi_draw_calls: u32,
    /// 平均每次调用的绘制数
    pub avg_draws_per_call: f32,
    /// 节省的API调用数
    pub saved_api_calls: u32,
}

impl MultiDrawIndirectOptimizer {
    /// 创建多绘制优化器
    pub fn new(enabled: bool, max_commands: usize) -> Self {
        Self {
            commands: Vec::with_capacity(max_commands),
            max_commands,
            stats: MultiDrawStats::default(),
            enabled,
        }
    }

    /// 清空命令
    pub fn clear(&mut self) {
        self.commands.clear();
        self.stats = MultiDrawStats::default();
    }

    /// 添加绘制命令
    ///
    /// # 参数
    ///
    /// - `command`: 间接绘制命令
    pub fn add_command(&mut self, command: DrawIndexedIndirectArgs) {
        if !self.enabled {
            return;
        }

        self.commands.push(command);
        self.stats.total_draws += 1;
    }

    /// 批量添加命令
    pub fn add_commands(&mut self, commands: &[DrawIndexedIndirectArgs]) {
        if !self.enabled {
            return;
        }

        self.commands.extend_from_slice(commands);
        self.stats.total_draws += commands.len() as u32;
    }

    /// 提交多绘制命令
    ///
    /// # 参数
    ///
    /// - `device`: WGPU设备
    /// - `queue`: WGPU队列
    /// - `indirect_buffer`: 间接绘制缓冲区
    /// - `commands_per_call`: 每次调用的命令数
    ///
    /// # 返回
    ///
    /// 返回多绘制调用次数。
    pub fn submit_multi_draw(
        &mut self,
        device: &Device,
        queue: &Queue,
        indirect_buffer: &Buffer,
        commands_per_call: usize,
    ) -> Result<usize, &'static str> {
        if !self.enabled {
            return Ok(0);
        }

        if self.commands.is_empty() {
            return Ok(0);
        }

        // 计算需要的调用次数
        let num_calls = (self.commands.len() + commands_per_call - 1) / commands_per_call;

        // 上传命令到缓冲区
        let data = bytemuck::cast_slice(&self.commands);
        queue.write_buffer(indirect_buffer, 0, data);

        // 更新统计
        self.stats.multi_draw_calls = num_calls as u32;
        self.stats.avg_draws_per_call = if num_calls > 0 {
            self.commands.len() as f32 / num_calls as f32
        } else {
            0.0
        };
        self.stats.saved_api_calls = self.commands.len() as u32 - num_calls as u32;

        // 注意：实际的multi_draw_indirect调用需要在render pass中进行
        // 这里只是准备数据和返回调用次数
        Ok(num_calls)
    }

    /// 获取统计
    pub fn stats(&self) -> &MultiDrawStats {
        &self.stats
    }

    /// 获取命令数量
    pub fn command_count(&self) -> usize {
        self.commands.len()
    }
}

// ============================================================================
// 综合优化管理器
// ============================================================================

/// 间接绘制综合优化管理器
///
/// 整合批处理、实例化和多绘制优化。
pub struct IndirectDrawOptimizer {
    /// 批处理优化器
    batcher: BatchingOptimizer,
    /// 实例化优化器
    instancer: Option<InstancingOptimizer>,
    /// 多绘制优化器
    multi_draw: Option<MultiDrawIndirectOptimizer>,
    /// 间接缓冲区
    indirect_buffer: Option<Buffer>,
    /// 配置
    config: OptimizerConfig,
}

#[derive(Debug, Clone)]
pub struct OptimizerConfig {
    /// 是否启用批处理
    pub enable_batching: bool,
    /// 是否启用实例化
    pub enable_instancing: bool,
    /// 是否启用多绘制
    pub enable_multi_draw: bool,
    /// 最大批次大小
    pub max_batch_size: u32,
    /// 多绘制每次调用的命令数
    pub multi_draw_commands_per_call: usize,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            enable_batching: true,
            enable_instancing: true,
            enable_multi_draw: false, // WebGPU支持有限
            max_batch_size: 100,
            multi_draw_commands_per_call: 10,
        }
    }
}

impl IndirectDrawOptimizer {
    /// 创建优化管理器
    ///
    /// # 参数
    ///
    /// - `config`: 优化配置
    /// - `device`: WGPU设备
    pub fn new(config: OptimizerConfig, device: &Device) -> Self {
        let batcher = BatchingOptimizer::new(config.enable_batching, config.max_batch_size);

        let instancer = if config.enable_instancing {
            Some(InstancingOptimizer::new(true, 1024, device))
        } else {
            None
        };

        let multi_draw = if config.enable_multi_draw {
            Some(MultiDrawIndirectOptimizer::new(true, 1000))
        } else {
            None
        };

        // 创建间接缓冲区
        let indirect_buffer = if config.enable_multi_draw || config.enable_batching {
            let buffer_size = (std::mem::size_of::<DrawIndexedIndirectArgs>() * 1000)
                as wgpu::BufferAddress;

            Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Indirect Draw Optimizer Buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::INDIRECT
                    | wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }))
        } else {
            None
        };

        Self {
            batcher,
            instancer,
            multi_draw,
            indirect_buffer,
            config,
        }
    }

    /// 优化绘制调用
    ///
    /// # 参数
    ///
    /// - `device`: WGPU设备
    /// - `queue`: WGPU队列
    /// - `draw_calls`: 绘制调用列表
    ///
    /// # 返回
    ///
    /// 返回优化后的间接绘制命令列表。
    pub fn optimize_draw_calls(
        &mut self,
        device: &Device,
        queue: &Queue,
        draw_calls: &[(BatchKey, GpuInstance)],
        index_count: u32,
    ) -> Result<Vec<DrawIndexedIndirectArgs>, &'static str> {
        // 清空之前的批处理
        self.batcher.clear();

        // 添加绘制调用到批次
        for (idx, (key, instance)) in draw_calls.iter().enumerate() {
            self.batcher
                .add_draw_call(key.clone(), idx as u32, instance);
        }

        // 完成批处理
        let commands = self.batcher.finalize(device, queue, index_count);

        Ok(commands)
    }

    /// 获取批处理统计
    pub fn batching_stats(&self) -> &BatchingStats {
        self.batcher.stats()
    }

    /// 获取实例化统计
    pub fn instancing_stats(&self) -> Option<&InstancingStats> {
        self.instancer.as_ref().map(|i| i.stats())
    }

    /// 获取多绘制统计
    pub fn multi_draw_stats(&self) -> Option<&MultiDrawStats> {
        self.multi_draw.as_ref().map(|m| m.stats())
    }

    /// 获取间接缓冲区
    pub fn indirect_buffer(&self) -> Option<&Buffer> {
        self.indirect_buffer.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_key_creation() {
        let key = BatchKey::new(12345, 100, 67890);
        assert_eq!(key.pipeline_hash, 12345);
        assert_eq!(key.material_id, 100);
        assert_eq!(key.texture_hash, 67890);
    }

    #[test]
    fn test_batch_info() {
        let key = BatchKey::new(1, 2, 3);
        let mut batch = BatchInfo::new(key);
        assert!(batch.is_empty());

        let instance = GpuInstance::default();
        batch.add_instance(0, &instance);
        batch.add_instance(1, &instance);

        assert_eq!(batch.instance_count(), 2);
        assert!(!batch.is_empty());
    }

    #[test]
    fn test_batching_optimizer() {
        let mut optimizer = BatchingOptimizer::new(true, 10);

        let key = BatchKey::new(1, 2, 3);
        let instance = GpuInstance::default();

        optimizer.add_draw_call(key.clone(), 0, &instance);
        optimizer.add_draw_call(key.clone(), 1, &instance);

        assert_eq!(optimizer.stats().total_draw_calls, 2);
        assert_eq!(optimizer.batch_count(), 1);
    }
}
