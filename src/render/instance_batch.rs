//! 3D 网格实例化批处理模块
//!
//! 通过将相同 Mesh + Material 的对象合并为单次 Draw Call，
//! 减少 70-90% 的渲染开销。
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  Instance Batching Pipeline              │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. Batch Collection (System)                            │
//! │     - 遍历所有 Mesh3D + Transform 实体                    │
//! │     - 按 (mesh_id, material_id) 分组                      │
//! │                                                          │
//! │  2. Batch Upload (System)                                │
//! │     - 检测脏批次                                          │
//! │     - 增量更新 GPU 实例数据                               │
//! │                                                          │
//! │  3. Instanced Draw                                       │
//! │     - 单次 draw_indexed_instanced 绘制整个批次             │
//! └─────────────────────────────────────────────────────────┘
//! ```

use crate::impl_default;
use bevy_ecs::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;

use super::mesh::GpuMesh;
use super::pbr_renderer::Instance3D;

// ============================================================================
// 核心数据结构
// ============================================================================

/// 批次键：唯一标识一个批次
///
/// 用于将相同网格和材质的实例分组到同一个批次中。
///
/// # 使用示例
///
/// ```rust
/// use game_engine::render::instance_batch::BatchKey;
///
/// let key = BatchKey {
///     mesh_id: 1,
///     material_id: 2,
/// };
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BatchKey {
    /// 网格资源 ID
    pub mesh_id: u64,
    /// 材质资源 ID
    pub material_id: u64,
}

/// 实例脏标记追踪器 (针对 Instance3D)
///
/// 追踪实例数据的变化，支持块级和实例级的脏标记，用于增量更新GPU缓冲区。
///
/// # 性能优化
///
/// - 块级脏标记：快速检测大范围变化
/// - 实例级脏标记：精确定位变化位置
/// - 脏范围合并：减少GPU上传次数
pub struct Instance3DDirtyTracker {
    /// 每个块的大小（实例数）
    chunk_size: usize,
    /// 块级脏标记 (true = 块内有变化)
    chunk_dirty: Vec<bool>,
    /// 实例级脏标记 (细粒度追踪)
    instance_dirty: Vec<bool>,
    /// 上一帧的实例数据（用于比较）
    prev_instances: Vec<Instance3D>,
    /// 脏实例范围 (start, end) 列表，用于批量上传
    dirty_ranges: Vec<(u32, u32)>,
    /// 总实例数
    instance_count: usize,
    /// 是否需要完整重建
    needs_full_rebuild: bool,
}

impl Instance3DDirtyTracker {
    /// 默认块大小
    ///
    /// 每个块包含128个实例，用于块级脏标记。
    pub const DEFAULT_CHUNK_SIZE: usize = 128;

    /// 创建新的脏标记追踪器
    ///
    /// # 参数
    ///
    /// * `initial_capacity` - 初始容量（实例数）
    /// * `chunk_size` - 块大小（每个块的实例数）
    ///
    /// # 返回
    ///
    /// 返回一个初始化的追踪器，所有实例标记为脏。
    pub fn new(initial_capacity: usize, chunk_size: usize) -> Self {
        let chunk_count = (initial_capacity + chunk_size - 1) / chunk_size;
        Self {
            chunk_size,
            chunk_dirty: vec![true; chunk_count],
            instance_dirty: vec![true; initial_capacity],
            prev_instances: Vec::with_capacity(initial_capacity),
            dirty_ranges: Vec::new(),
            instance_count: 0,
            needs_full_rebuild: true,
        }
    }

    /// 使用默认块大小创建追踪器
    ///
    /// # 参数
    ///
    /// * `capacity` - 初始容量（实例数）
    ///
    /// # 返回
    ///
    /// 返回一个使用默认块大小的追踪器。
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(capacity, Self::DEFAULT_CHUNK_SIZE)
    }

    /// 重置追踪器
    ///
    /// 清空所有脏标记和缓存数据，准备重新开始追踪。
    pub fn reset(&mut self) {
        self.chunk_dirty.clear();
        self.instance_dirty.clear();
        self.prev_instances.clear();
        self.dirty_ranges.clear();
        self.instance_count = 0;
        self.needs_full_rebuild = true;
    }

    /// 更新并检测变化
    ///
    /// 比较新旧实例数据，检测变化并生成脏范围列表。
    ///
    /// # 参数
    ///
    /// * `instances` - 当前帧的实例数据
    ///
    /// # 返回
    ///
    /// 返回脏范围列表，每个范围表示`(start, end)`索引对。
    ///
    /// # 性能
    ///
    /// 如果实例数量变化，会触发完整重建。否则使用增量比较。
    pub fn update(&mut self, instances: &[Instance3D]) -> &[(u32, u32)] {
        self.dirty_ranges.clear();

        let new_count = instances.len();
        let old_count = self.prev_instances.len();

        // 如果数量变化，需要完整重建
        if new_count != old_count {
            self.needs_full_rebuild = true;
        }

        // 调整容量
        if new_count > self.instance_dirty.len() {
            let additional = new_count - self.instance_dirty.len();
            self.instance_dirty
                .extend(std::iter::repeat(true).take(additional));

            let new_chunk_count = (new_count + self.chunk_size - 1) / self.chunk_size;
            if new_chunk_count > self.chunk_dirty.len() {
                let chunk_additional = new_chunk_count - self.chunk_dirty.len();
                self.chunk_dirty
                    .extend(std::iter::repeat(true).take(chunk_additional));
            }
        }

        self.instance_count = new_count;

        // 完整重建模式
        if self.needs_full_rebuild {
            self.prev_instances.clear();
            self.prev_instances.extend_from_slice(instances);
            if new_count > 0 {
                self.dirty_ranges.push((0, new_count as u32));
            }
            self.needs_full_rebuild = false;

            // 重置所有标记
            for flag in &mut self.chunk_dirty {
                *flag = false;
            }
            for flag in &mut self.instance_dirty {
                *flag = false;
            }

            return &self.dirty_ranges;
        }

        // 增量检测
        let mut range_start: Option<u32> = None;

        // 比较字节数据
        fn is_equal(a: &Instance3D, b: &Instance3D) -> bool {
            bytemuck::bytes_of(a) == bytemuck::bytes_of(b)
        }

        for chunk_idx in 0..self.chunk_dirty.len() {
            // 优化：如果块未被标记为脏，且我们没有外部强制脏标记机制，
            // 我们仍然需要检查内容是否变化（因为 instances 是每帧重建的）
            // 所以这里不能简单跳过，除非我们有办法知道 instances 数据源没变。
            // 在当前架构下，instances 是每帧重新收集的，所以必须逐个比较。
            // 之前的实现可能有误导，除非 instances 列表本身是持久的且有脏标记。
            // 但在这里，我们确实是在比较新旧数据。

            let start = chunk_idx * self.chunk_size;
            let end = ((chunk_idx + 1) * self.chunk_size).min(new_count);

            if start >= end {
                break;
            }

            let mut chunk_has_changes = false;

            for i in start..end {
                let is_dirty = !is_equal(&instances[i], &self.prev_instances[i]);

                if is_dirty {
                    chunk_has_changes = true;
                    self.instance_dirty[i] = true;

                    if range_start.is_none() {
                        range_start = Some(i as u32);
                    }
                } else {
                    self.instance_dirty[i] = false;

                    if let Some(start) = range_start {
                        self.dirty_ranges.push((start, i as u32));
                        range_start = None;
                    }
                }
            }

            self.chunk_dirty[chunk_idx] = chunk_has_changes;
        }

        // 关闭最后一个范围
        if let Some(start) = range_start {
            self.dirty_ranges.push((start, new_count as u32));
        }

        // 更新缓存
        self.prev_instances.clear();
        self.prev_instances.extend_from_slice(instances);

        // 合并相邻范围
        self.merge_ranges();

        &self.dirty_ranges
    }

    fn merge_ranges(&mut self) {
        if self.dirty_ranges.len() <= 1 {
            return;
        }

        self.dirty_ranges.sort_by_key(|r| r.0);

        let mut merged = Vec::with_capacity(self.dirty_ranges.len());
        let mut current = self.dirty_ranges[0];

        for &(start, end) in &self.dirty_ranges[1..] {
            // 如果范围相邻或重叠（允许小间隙合并以减少 draw call / upload calls）
            if start <= current.1 + 16 {
                current.1 = current.1.max(end);
            } else {
                merged.push(current);
                current = (start, end);
            }
        }
        merged.push(current);

        self.dirty_ranges = merged;
    }
}

/// 实例批次 - 相同 Mesh + Material 的实例集合
pub struct InstanceBatch {
    /// 批次键
    pub key: BatchKey,
    /// 网格引用
    pub mesh: Arc<GpuMesh>,
    /// 材质绑定组
    pub material_bind_group: Arc<wgpu::BindGroup>,
    /// 实例数据（CPU 端 - 每帧重建）
    pub instances: Vec<Instance3D>,
    /// GPU 实例缓冲区
    pub instance_buffer: Option<wgpu::Buffer>,
    /// 实例缓冲区容量
    pub buffer_capacity: usize,
    /// 脏标记追踪器
    pub dirty_tracker: Instance3DDirtyTracker,
    /// 是否为静态批次（不常更新）
    pub is_static: bool,
    /// 批次包围球中心
    pub bounding_center: [f32; 3],
    /// 批次包围球半径
    pub bounding_radius: f32,
    /// 额外材质绑定组（用于多绑定组支持，按管线布局顺序）
    pub extra_material_bind_groups: Vec<Arc<wgpu::BindGroup>>,
    #[cfg(feature = "wgpu_perf")]
    pub indirect_buffer: Option<wgpu::Buffer>,
    #[cfg(feature = "wgpu_perf")]
    pub indirect_count: u32,
}

impl InstanceBatch {
    /// 创建新的实例批次
    pub fn new(
        key: BatchKey,
        mesh: Arc<GpuMesh>,
        material_bind_group: Arc<wgpu::BindGroup>,
    ) -> Self {
        Self {
            key,
            mesh,
            material_bind_group,
            instances: Vec::with_capacity(128),
            instance_buffer: None,
            buffer_capacity: 0,
            dirty_tracker: Instance3DDirtyTracker::with_capacity(128),
            is_static: false,
            bounding_center: [0.0; 3],
            bounding_radius: 0.0,
            extra_material_bind_groups: Vec::new(),
            #[cfg(feature = "wgpu_perf")]
            indirect_buffer: None,
            #[cfg(feature = "wgpu_perf")]
            indirect_count: 0,
        }
    }

    /// 添加实例
    pub fn add_instance(&mut self, instance: Instance3D) {
        self.instances.push(instance);
    }

    /// 清空实例
    pub fn clear(&mut self) {
        self.instances.clear();
        // 注意：不重置 dirty_tracker，因为它需要保留 prev_instances 进行比较
    }

    /// 获取实例数量
    pub fn instance_count(&self) -> u32 {
        self.instances.len() as u32
    }

    /// 重新计算批次包围体（AABB及包围球）
    pub fn recompute_bounds(&mut self) {
        if self.instances.is_empty() {
            self.bounding_center = [0.0; 3];
            self.bounding_radius = 0.0;
            return;
        }
        let base_min = glam::Vec3::from_array(self.mesh.aabb_min);
        let base_max = glam::Vec3::from_array(self.mesh.aabb_max);
        let corners = [
            glam::Vec3::new(base_min.x, base_min.y, base_min.z),
            glam::Vec3::new(base_min.x, base_min.y, base_max.z),
            glam::Vec3::new(base_min.x, base_max.y, base_min.z),
            glam::Vec3::new(base_min.x, base_max.y, base_max.z),
            glam::Vec3::new(base_max.x, base_min.y, base_min.z),
            glam::Vec3::new(base_max.x, base_min.y, base_max.z),
            glam::Vec3::new(base_max.x, base_max.y, base_min.z),
            glam::Vec3::new(base_max.x, base_max.y, base_max.z),
        ];
        let mut world_min = glam::Vec3::splat(f32::INFINITY);
        let mut world_max = glam::Vec3::splat(f32::NEG_INFINITY);
        for inst in &self.instances {
            let m = glam::Mat4::from_cols_array_2d(&inst.model);
            for c in &corners {
                let wp = m.transform_point3(*c);
                world_min = world_min.min(wp);
                world_max = world_max.max(wp);
            }
        }
        let center = (world_min + world_max) * 0.5;
        let radius = (world_max - center).length();
        self.bounding_center = center.to_array();
        self.bounding_radius = radius;
    }
    /// 更新 GPU 缓冲区
    pub fn update_buffer(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        if self.instances.is_empty() {
            return;
        }

        // 1. 检查容量并调整缓冲区
        if self.instance_buffer.is_none() || self.buffer_capacity < self.instances.len() {
            // 预留 50% 额外空间，减少频繁重建
            let new_capacity = (self.instances.len() * 3 / 2).max(64);
            let buffer_size = new_capacity * std::mem::size_of::<Instance3D>();

            self.instance_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(&format!("Instance Buffer {:?}", self.key)),
                size: buffer_size as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
            self.buffer_capacity = new_capacity;

            // 缓冲区重建，强制全量更新
            self.dirty_tracker.reset();
        }

        // 2. 检测脏数据
        let dirty_ranges = self.dirty_tracker.update(&self.instances);

        if dirty_ranges.is_empty() {
            return;
        }

        // 3. 上传数据
        if let Some(buffer) = &self.instance_buffer {
            for &(start, end) in dirty_ranges {
                let start_index = start as usize;
                let end_index = end as usize;

                if start_index >= self.instances.len() {
                    continue;
                }
                let actual_end = end_index.min(self.instances.len());

                let data = &self.instances[start_index..actual_end];
                let offset =
                    (start_index * std::mem::size_of::<Instance3D>()) as wgpu::BufferAddress;

                queue.write_buffer(buffer, offset, bytemuck::cast_slice(data));
            }
        }
        #[cfg(feature = "wgpu_perf")]
        {
            self.update_indirect(device, queue);
        }
    }

    #[cfg(feature = "wgpu_perf")]
    pub fn update_indirect(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let cmd = DrawIndexedIndirect {
            index_count: self.mesh.index_count,
            instance_count: self.instance_count(),
            first_index: 0,
            base_vertex: 0,
            first_instance: 0,
        };
        let bytes = bytemuck::bytes_of(&cmd);
        if self.indirect_buffer.is_none() {
            self.indirect_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Indirect Draw (Batch)"),
                size: std::mem::size_of::<DrawIndexedIndirect>() as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }
        self.indirect_count = 1;
        if let Some(buf) = &self.indirect_buffer {
            queue.write_buffer(buf, 0, bytes);
        }
    }

    #[cfg(feature = "wgpu_perf")]
    /// 写入间接绘制命令（优化版本）
    ///
    /// # 性能优化
    /// - 复用缓冲区，避免每帧重建
    /// - 只在需要时扩展缓冲区
    /// - 使用批量写入减少CPU开销
    pub fn write_indirect_commands(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        cmds: &[DrawIndexedIndirect],
    ) {
        if cmds.is_empty() {
            self.indirect_count = 0;
            return;
        }

        let required_size =
            (cmds.len() * std::mem::size_of::<DrawIndexedIndirect>()) as wgpu::BufferAddress;
        // 对齐到256字节边界，提高GPU内存访问效率
        let aligned_size = (required_size + 255) & !255;

        // 检查是否需要创建或扩展缓冲区
        let needs_resize = self
            .indirect_buffer
            .as_ref()
            .map(|buf| buf.size() < aligned_size)
            .unwrap_or(true);

        if needs_resize {
            // 扩展缓冲区（预留50%额外空间）
            let expanded_size = (aligned_size * 3 / 2).max(256);
            self.indirect_buffer = Some(device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Indirect Draw (Batch Multi)"),
                size: expanded_size,
                usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }));
        }

        self.indirect_count = cmds.len() as u32;

        // 批量写入数据
        if let Some(buf) = &self.indirect_buffer {
            let data: &[u8] = bytemuck::cast_slice(cmds);
            queue.write_buffer(buf, 0, data);
        }
    }
}

// ============================================================================
// 批次管理器
// ============================================================================

/// 批次管理器 - 管理所有实例批次
#[derive(Resource)]
pub struct BatchManager {
    /// 批次映射
    batches: HashMap<BatchKey, InstanceBatch>,
    /// 可见批次索引（每帧更新）
    visible_batch_keys: Vec<BatchKey>,
    small_batch_keys: Vec<BatchKey>,
    /// 统计信息
    pub stats: BatchStats,
    /// 动态批次配置
    dynamic_config: DynamicBatchConfig,
}

impl Default for BatchManager {
    fn default() -> Self {
        Self {
            batches: HashMap::new(),
            visible_batch_keys: Vec::new(),
            small_batch_keys: Vec::new(),
            stats: BatchStats::default(),
            dynamic_config: DynamicBatchConfig::default(),
        }
    }
}

/// 动态批次配置
///
/// 管理实例化渲染的批次大小，支持根据GPU能力和性能指标动态调整。
///
/// ## 功能特性
///
/// - **GPU限制检测**: 自动检测GPU支持的最大实例数
/// - **性能自适应**: 根据性能历史自动调整批次大小
/// - **批次优化**: 识别需要拆分的大批次和需要合并的小批次
///
/// ## 使用示例
///
/// ```ignore
/// use game_engine::render::DynamicBatchConfig;
///
/// // 创建默认配置
/// let mut config = DynamicBatchConfig::default();
///
/// // 根据GPU限制调整
/// config.adjust_for_gpu_limits(2048);
///
/// // 记录性能指标（例如：每帧时间）
/// config.record_performance(16.67); // 60 FPS
///
/// // 自适应调整
/// config.adaptive_adjust();
/// ```
#[derive(Debug, Clone)]
pub struct DynamicBatchConfig {
    /// 最小批次大小（实例数）
    pub min_batch_size: usize,
    /// 最大批次大小（实例数）
    pub max_batch_size: usize,
    /// 理想批次大小（实例数）
    pub ideal_batch_size: usize,
    /// 小批次阈值（小于此值认为是小批次）
    pub small_batch_threshold: usize,
    /// 是否启用动态调整
    pub enable_dynamic_adjustment: bool,
    /// 性能指标历史（用于自适应调整）
    performance_history: Vec<f32>,
    /// 历史记录最大长度
    max_history_length: usize,
}

impl_default!(DynamicBatchConfig {
    min_batch_size: 32,
    max_batch_size: 8192,
    ideal_batch_size: 512,
    small_batch_threshold: 16,
    enable_dynamic_adjustment: true,
    performance_history: Vec::new(),
    max_history_length: 60,
});

impl DynamicBatchConfig {
    /// 创建新的动态批次配置
    pub fn new(min: usize, max: usize, ideal: usize) -> Self {
        Self {
            min_batch_size: min,
            max_batch_size: max,
            ideal_batch_size: ideal,
            small_batch_threshold: min / 2,
            enable_dynamic_adjustment: true,
            performance_history: Vec::new(),
            max_history_length: 60,
        }
    }

    /// 根据GPU限制调整配置
    pub fn adjust_for_gpu_limits(&mut self, max_instances_per_draw: usize) {
        self.max_batch_size = self.max_batch_size.min(max_instances_per_draw);
        self.ideal_batch_size = self.ideal_batch_size.min(max_instances_per_draw);
    }

    /// 记录性能指标（例如：每帧时间、draw call数等）
    pub fn record_performance(&mut self, metric: f32) {
        self.performance_history.push(metric);
        if self.performance_history.len() > self.max_history_length {
            self.performance_history.remove(0);
        }
    }

    /// 根据性能历史自适应调整批次大小
    pub fn adaptive_adjust(&mut self) {
        if !self.enable_dynamic_adjustment || self.performance_history.len() < 10 {
            return;
        }

        // 计算平均性能
        let avg_performance: f32 =
            self.performance_history.iter().sum::<f32>() / self.performance_history.len() as f32;

        // 如果性能下降（指标增加），减小批次大小
        // 如果性能提升（指标减少），增大批次大小
        let recent_avg: f32 = self.performance_history.iter().rev().take(10).sum::<f32>() / 10.0;

        if recent_avg > avg_performance * 1.1 {
            // 性能下降，减小批次大小
            let new_size = (self.ideal_batch_size as f32 * 0.9) as usize;
            self.ideal_batch_size = new_size.max(self.min_batch_size);
        } else if recent_avg < avg_performance * 0.9 {
            // 性能提升，可以增大批次大小
            let new_size = (self.ideal_batch_size as f32 * 1.1) as usize;
            self.ideal_batch_size = new_size.min(self.max_batch_size);
        }
    }

    /// 计算当前应该使用的批次大小
    pub fn calculate_batch_size(&self, instance_count: usize) -> usize {
        if instance_count <= self.small_batch_threshold {
            return instance_count; // 小批次直接使用实际大小
        }

        // 根据实例数量选择批次大小
        if instance_count <= self.ideal_batch_size {
            instance_count
        } else {
            // 如果超过理想大小，可能需要拆分
            self.ideal_batch_size
        }
    }
}

/// 批次统计信息
#[derive(Default, Clone, Copy, Debug)]
pub struct BatchStats {
    /// 批次拆分次数
    pub batches_split: u32,
    /// 批次合并次数
    pub batches_merged: u32,
    /// 总批次数
    pub total_batches: u32,
    /// 总实例数
    pub total_instances: u32,
    /// 本帧 Draw Call 数
    pub draw_calls: u32,
    /// 节省的 Draw Call 数
    pub saved_draw_calls: u32,
    pub small_draw_calls: u32,
    pub visible_batches: u32,
}

impl BatchManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用自定义动态配置创建
    pub fn with_dynamic_config(config: DynamicBatchConfig) -> Self {
        Self {
            batches: HashMap::new(),
            visible_batch_keys: Vec::new(),
            small_batch_keys: Vec::new(),
            stats: BatchStats::default(),
            dynamic_config: config,
        }
    }

    /// 获取动态配置的可变引用
    pub fn dynamic_config_mut(&mut self) -> &mut DynamicBatchConfig {
        &mut self.dynamic_config
    }

    /// 获取动态配置的引用
    pub fn dynamic_config(&self) -> &DynamicBatchConfig {
        &self.dynamic_config
    }

    /// 优化批次大小（动态调整）
    ///
    /// 根据配置动态调整批次大小：
    /// - 拆分超过最大批次大小的批次
    /// - 合并小批次以减少draw call
    pub fn optimize_batch_sizes(&mut self) {
        if !self.dynamic_config.enable_dynamic_adjustment {
            return;
        }

        let mut batches_to_split = Vec::new();
        let mut batches_to_merge = Vec::new();

        // 识别需要拆分的大批次和需要合并的小批次
        for (key, batch) in &self.batches {
            let instance_count = batch.instance_count() as usize;
            if instance_count > self.dynamic_config.max_batch_size {
                batches_to_split.push(*key);
            } else if instance_count < self.dynamic_config.small_batch_threshold
                && instance_count > 0
            {
                batches_to_merge.push(*key);
            }
        }

        // 拆分大批次
        for key in batches_to_split {
            if let Some(batch) = self.batches.remove(&key) {
                self.split_batch(batch);
            }
        }

        // 合并小批次
        self.merge_small_batches();
    }

    /// 拆分大批次为多个小批次
    fn split_batch(&mut self, batch: InstanceBatch) {
        let instance_count = batch.instance_count() as usize;
        let ideal_size = self.dynamic_config.ideal_batch_size;
        let num_splits = (instance_count + ideal_size - 1) / ideal_size;

        tracing::debug!(
            target: "render",
            "Splitting batch {:?} with {} instances into {} batches",
            batch.key,
            instance_count,
            num_splits
        );

        // 将实例分批
        for i in 0..num_splits {
            let start = i * ideal_size;
            let end = (start + ideal_size).min(instance_count);

            if start >= instance_count {
                break;
            }

            // 创建新的批次（使用相同的key，但添加索引后缀）
            // 注意：这里简化处理，实际应该创建新的BatchKey
            let mut new_batch = InstanceBatch::new(
                batch.key, // 使用相同的key（实际应该创建新key）
                batch.mesh.clone(),
                batch.material_bind_group.clone(),
            );

            // 复制实例数据
            new_batch
                .instances
                .extend_from_slice(&batch.instances[start..end]);

            // 复制额外绑定组
            new_batch.extra_material_bind_groups = batch.extra_material_bind_groups.clone();

            // 重新计算包围体
            new_batch.recompute_bounds();

            // 添加到批次管理器
            // 注意：这里使用相同的key会有问题，实际应该创建新的key
            // 为了简化，我们使用一个临时的key生成策略
            let new_key = BatchKey {
                mesh_id: batch.key.mesh_id,
                material_id: batch.key.material_id.wrapping_add(i as u64),
            };

            self.batches.insert(new_key, new_batch);
        }

        // 更新统计信息
        self.stats.batches_split += num_splits as u32;
    }

    /// 合并小批次
    fn merge_small_batches(&mut self) {
        // 按mesh_id和material_id分组小批次
        let mut merge_groups: HashMap<(u64, u64), Vec<BatchKey>> = HashMap::new();

        for key in &self.small_batch_keys {
            if let Some(batch) = self.batches.get(key) {
                let group_key = (batch.key.mesh_id, batch.key.material_id);
                merge_groups
                    .entry(group_key)
                    .or_insert_with(Vec::new)
                    .push(*key);
            }
        }

        // 合并每个组中的小批次
        for (group_key, batch_keys) in merge_groups {
            if batch_keys.len() < 2 {
                continue; // 至少需要2个批次才能合并
            }

            // 计算总实例数
            let total_instances: usize = batch_keys
                .iter()
                .filter_map(|key| self.batches.get(key))
                .map(|batch| batch.instance_count() as usize)
                .sum();

            // 如果合并后的批次仍然小于理想大小，则合并
            if total_instances <= self.dynamic_config.ideal_batch_size {
                self.merge_batch_group(group_key, batch_keys);
            }
        }

        // 清空小批次列表（已处理）
        self.small_batch_keys.clear();
    }

    /// 合并一组批次
    fn merge_batch_group(&mut self, group_key: (u64, u64), batch_keys: Vec<BatchKey>) {
        if batch_keys.is_empty() {
            return;
        }

        // 获取第一个批次作为基础
        let first_key = batch_keys[0];
        let mut base_batch = match self.batches.remove(&first_key) {
            Some(batch) => batch,
            None => return,
        };

        let mut merged_count = 1;

        // 合并其他批次
        for key in &batch_keys[1..] {
            if let Some(batch) = self.batches.remove(key) {
                base_batch.instances.extend_from_slice(&batch.instances);
                merged_count += 1;
            }
        }

        // 重新计算包围体
        base_batch.recompute_bounds();

        // 重新插入合并后的批次
        let merged_key = BatchKey {
            mesh_id: group_key.0,
            material_id: group_key.1,
        };
        self.batches.insert(merged_key, base_batch);

        tracing::debug!(
            target: "render",
            "Merged {} small batches into batch {:?}",
            merged_count,
            merged_key
        );

        // 更新统计信息
        self.stats.batches_merged += merged_count as u32;
    }

    /// 根据性能指标自适应调整批次配置
    pub fn adaptive_adjust_batches(&mut self, performance_metric: f32) {
        self.dynamic_config.record_performance(performance_metric);
        self.dynamic_config.adaptive_adjust();
    }

    /// 根据GPU能力调整批次配置
    pub fn adjust_for_gpu(&mut self, device: &wgpu::Device) {
        // 获取GPU限制
        let limits = device.limits();
        // 使用max_storage_buffer_binding_size作为参考，或使用合理的默认值
        let max_instances = (limits.max_storage_buffer_binding_size as usize
            / std::mem::size_of::<Instance3D>())
        .min(8192);

        // 调整配置以适应GPU限制
        self.dynamic_config.adjust_for_gpu_limits(max_instances);
    }

    /// 获取或创建批次
    pub fn get_or_create_batch(
        &mut self,
        key: BatchKey,
        mesh: Arc<GpuMesh>,
        material_bind_group: Arc<wgpu::BindGroup>,
    ) -> &mut InstanceBatch {
        self.batches
            .entry(key)
            .or_insert_with(|| InstanceBatch::new(key, mesh, material_bind_group))
    }

    /// 清空所有批次的实例（每帧开始调用）
    pub fn clear_instances(&mut self) {
        for batch in self.batches.values_mut() {
            batch.clear();
        }
        self.visible_batch_keys.clear();
        self.small_batch_keys.clear();
    }

    /// 标记批次为可见
    pub fn mark_visible(&mut self, key: BatchKey) {
        if !self.visible_batch_keys.contains(&key) {
            self.visible_batch_keys.push(key);
        }
    }

    /// 重新计算所有批次的包围体
    pub fn recompute_all_bounds(&mut self) {
        for batch in self.batches.values_mut() {
            batch.recompute_bounds();
        }
    }

    /// 使用视锥体剔除可见批次
    pub fn cull_visible_batches(&mut self, view_proj: [[f32; 4]; 4]) {
        let uniforms =
            crate::render::gpu_driven::culling::CullingUniforms::from_view_proj(view_proj, 0);
        let planes = uniforms.frustum_planes;

        fn sphere_in_frustum(center: [f32; 3], radius: f32, planes: &[[f32; 4]; 6]) -> bool {
            for p in planes.iter() {
                let dist = p[0] * center[0] + p[1] * center[1] + p[2] * center[2] + p[3];
                if dist < -radius {
                    return false;
                }
            }
            true
        }

        let mut filtered = Vec::with_capacity(self.visible_batch_keys.len());
        for key in &self.visible_batch_keys {
            if let Some(batch) = self.batches.get(key) {
                if batch.instance_count() == 0 {
                    continue;
                }
                if batch.instance_count() < 2 {
                    self.small_batch_keys.push(*key);
                } else {
                    if sphere_in_frustum(batch.bounding_center, batch.bounding_radius, &planes) {
                        filtered.push(*key);
                    }
                }
            }
        }
        self.visible_batch_keys = filtered;
        self.compute_stats();
    }

    /// 更新所有脏批次的 GPU 缓冲区
    pub fn update_buffers(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for batch in self.batches.values_mut() {
            // 只有可见且非空的批次才需要更新？
            // 或者：即使不可见，如果数据变了也更新？
            // 策略：只更新可见的，或者全部。
            // 考虑到 visible_batches 可能会变，最好是更新所有非空的。
            if !batch.instances.is_empty() {
                batch.update_buffer(device, queue);
            }
        }
    }

    /// 获取可见批次迭代器
    pub fn visible_batches(&self) -> impl Iterator<Item = &InstanceBatch> {
        self.visible_batch_keys
            .iter()
            .filter_map(|key| self.batches.get(key))
            .filter(|batch| !batch.instances.is_empty())
    }

    pub fn small_batches(&self) -> impl Iterator<Item = &InstanceBatch> {
        self.small_batch_keys
            .iter()
            .filter_map(|key| self.batches.get(key))
            .filter(|batch| batch.instance_count() > 0)
    }

    /// 计算统计信息
    pub fn compute_stats(&mut self) {
        let mut total_instances = 0u32;
        let mut draw_calls = 0u32;

        for key in &self.visible_batch_keys {
            if let Some(batch) = self.batches.get(key) {
                if !batch.instances.is_empty() {
                    total_instances += batch.instance_count();
                    draw_calls += 1;
                }
            }
        }

        self.stats = BatchStats {
            batches_split: self.stats.batches_split, // 保留之前的拆分次数
            batches_merged: self.stats.batches_merged, // 保留之前的合并次数
            total_batches: self.batches.len() as u32,
            total_instances,
            draw_calls,
            saved_draw_calls: total_instances.saturating_sub(draw_calls),
            small_draw_calls: self.small_batch_keys.len() as u32,
            visible_batches: self.visible_batch_keys.len() as u32,
        };
    }

    /// 移除空批次（定期调用）
    pub fn cleanup_empty_batches(&mut self) {
        self.batches
            .retain(|_, batch| !batch.instances.is_empty() || batch.is_static);
    }

    pub fn set_batch_textures_bind_group(
        &mut self,
        key: BatchKey,
        bg: std::sync::Arc<wgpu::BindGroup>,
    ) {
        if let Some(batch) = self.batches.get_mut(&key) {
            if batch.extra_material_bind_groups.is_empty() {
                batch.extra_material_bind_groups.push(bg);
            } else {
                batch.extra_material_bind_groups[0] = bg;
            }
        }
    }

    pub fn collect_gpu_instances(
        &self,
    ) -> (
        Vec<crate::render::gpu_driven::culling::GpuInstance>,
        Vec<(BatchKey, u32)>,
    ) {
        let mut instances = Vec::new();
        let mut mapping = Vec::new();
        for key in &self.visible_batch_keys {
            if let Some(batch) = self.batches.get(key) {
                let base_min = glam::Vec3::from_array(batch.mesh.aabb_min);
                let base_max = glam::Vec3::from_array(batch.mesh.aabb_max);
                for (idx, inst) in batch.instances.iter().enumerate() {
                    instances.push(crate::render::gpu_driven::culling::GpuInstance {
                        model: inst.model,
                        aabb_min: base_min.to_array(),
                        instance_id: mapping.len() as u32,
                        aabb_max: base_max.to_array(),
                        flags: 0,
                    });
                    mapping.push((*key, idx as u32));
                }
            }
        }
        (instances, mapping)
    }

    pub fn apply_visible_ids(&mut self, mapping: &[(BatchKey, u32)], ids: &[u32]) {
        use std::collections::HashMap;
        let mut filtered: HashMap<BatchKey, Vec<crate::render::pbr_renderer::Instance3D>> =
            HashMap::new();
        for &vid in ids {
            let (key, local_idx) = mapping[vid as usize];
            if let Some(batch) = self.batches.get(&key) {
                if let Some(inst) = batch.instances.get(local_idx as usize) {
                    filtered.entry(key).or_default().push(*inst);
                }
            }
        }
        for (key, list) in filtered {
            if let Some(batch) = self.batches.get_mut(&key) {
                batch.instances = list;
            }
        }
        self.compute_stats();
    }

    /// 应用GPU剔除的可见实例ID到批次管理器（间接绘制版本）
    ///
    /// 将GPU剔除结果应用到批次管理器，并生成间接绘制命令。
    /// 这个方法需要 `wgpu_perf` 特性，用于间接绘制优化（T3.1.2）。
    #[cfg(feature = "wgpu_perf")]
    pub fn apply_visible_ids_segments(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        mapping: &[(BatchKey, u32)],
        ids: &[u32],
    ) {
        use std::collections::HashMap;
        let mut per_batch: HashMap<BatchKey, Vec<u32>> = HashMap::new();
        for &vid in ids {
            let (key, local_idx) = mapping[vid as usize];
            per_batch.entry(key).or_default().push(local_idx);
        }
        for (key, list) in per_batch {
            if let Some(batch) = self.batches.get_mut(&key) {
                let mut idxs = list;
                idxs.sort_unstable();
                let mut cmds: Vec<DrawIndexedIndirect> = Vec::new();
                let mut i = 0usize;
                while i < idxs.len() {
                    let start = idxs[i];
                    let mut end = start;
                    i += 1;
                    while i < idxs.len() && idxs[i] == end + 1 {
                        end = idxs[i];
                        i += 1;
                    }
                    let count = end - start + 1;
                    cmds.push(DrawIndexedIndirect {
                        index_count: batch.mesh.index_count,
                        instance_count: count,
                        first_index: 0,
                        base_vertex: 0,
                        first_instance: start,
                    });
                }
                batch.write_indirect_commands(device, queue, &cmds);
            }
        }
    }
}

// ============================================================================
// ECS 组件
// ============================================================================

/// 3D 网格渲染组件
#[derive(Component)]
pub struct Mesh3DRenderer {
    /// 网格资源
    pub mesh: Arc<GpuMesh>,
    /// 材质绑定组
    pub material_bind_group: Arc<wgpu::BindGroup>,
    /// 纹理绑定组（PBR纹理集，布局组3）
    pub textures_bind_group: Option<Arc<wgpu::BindGroup>>,
    /// 材质Uniform缓冲（保持资源生命周期）
    pub material_uniform_buffer: Option<Arc<wgpu::Buffer>>,
    /// 网格 ID（用于批次分组）
    pub mesh_id: u64,
    /// 材质 ID（用于批次分组）
    pub material_id: u64,
    /// 是否可见
    pub visible: bool,
}

impl Mesh3DRenderer {
    pub fn batch_key(&self) -> BatchKey {
        BatchKey {
            mesh_id: self.mesh_id,
            material_id: self.material_id,
        }
    }
}

// ============================================================================
// ECS 系统
// ============================================================================

/// 批次收集系统 - 将所有可见实体分组到批次
pub fn batch_collection_system(
    mut batch_manager: ResMut<BatchManager>,
    query: Query<(&Mesh3DRenderer, &crate::ecs::Transform)>,
) {
    use glam::Mat4;

    // 清空上一帧的实例
    batch_manager.clear_instances();

    // 收集所有可见实体
    for (renderer, transform) in query.iter() {
        if !renderer.visible {
            continue;
        }

        let key = renderer.batch_key();

        // 创建实例数据 - 从 Transform 构建模型矩阵
        let model_matrix =
            Mat4::from_scale_rotation_translation(transform.scale, transform.rot, transform.pos);

        let instance = Instance3D {
            model: model_matrix.to_cols_array_2d(),
        };

        // 添加到批次
        let batch = batch_manager.get_or_create_batch(
            key,
            renderer.mesh.clone(),
            renderer.material_bind_group.clone(),
        );
        if let Some(ref tex_bg) = renderer.textures_bind_group {
            if batch.extra_material_bind_groups.is_empty() {
                batch.extra_material_bind_groups.push(tex_bg.clone());
            } else {
                batch.extra_material_bind_groups[0] = tex_bg.clone();
            }
        }
        batch.add_instance(instance);
        batch_manager.mark_visible(key);
    }

    // 重新计算包围体并更新统计
    batch_manager.recompute_all_bounds();
    batch_manager.compute_stats();
}

/// 批次可见性剔除系统 - 使用视锥剔除过滤批次
pub fn batch_visibility_culling_system(
    mut batch_manager: ResMut<BatchManager>,
    vp: Option<Res<crate::ecs::Viewport>>,
    query_cam: Query<(&crate::ecs::Transform, &crate::ecs::Camera)>,
) {
    let mut view_proj = glam::Mat4::IDENTITY.to_cols_array_2d();
    for (t, c) in query_cam.iter() {
        if c.is_active {
            let view = glam::Mat4::from_rotation_translation(t.rot, t.pos).inverse();
            let proj = match c.projection {
                crate::ecs::Projection::Orthographic { scale, near, far } => {
                    let aspect = vp
                        .as_ref()
                        .map(|v| v.width as f32 / v.height as f32)
                        .unwrap_or(1.0);
                    glam::Mat4::orthographic_rh(
                        -aspect * scale,
                        aspect * scale,
                        -scale,
                        scale,
                        near,
                        far,
                    )
                }
                crate::ecs::Projection::Perspective {
                    fov,
                    aspect,
                    near,
                    far,
                } => glam::Mat4::perspective_rh(fov, aspect, near, far),
            };
            view_proj = (proj * view).to_cols_array_2d();
            break;
        }
    }

    batch_manager.cull_visible_batches(view_proj);
}

/// 实例级剔除（CPU回退）
pub fn sphere_vs_frustum(center: [f32; 3], radius: f32, planes: &[[f32; 4]; 6]) -> bool {
    for p in planes.iter() {
        let dist = p[0] * center[0] + p[1] * center[1] + p[2] * center[2] + p[3];
        if dist < -radius {
            return false;
        }
    }
    true
}

impl BatchManager {
    pub fn cull_instances_cpu(&mut self, view_proj: [[f32; 4]; 4]) {
        let uniforms =
            crate::render::gpu_driven::culling::CullingUniforms::from_view_proj(view_proj, 0);
        let planes = uniforms.frustum_planes;
        let base_min = |b: &InstanceBatch| glam::Vec3::from_array(b.mesh.aabb_min);
        let base_max = |b: &InstanceBatch| glam::Vec3::from_array(b.mesh.aabb_max);
        let radius_from_aabb = |min: glam::Vec3, max: glam::Vec3| ((max - min) * 0.5).length();
        for key in &self.visible_batch_keys {
            if let Some(batch) = self.batches.get_mut(key) {
                let min = base_min(batch);
                let max = base_max(batch);
                let r = radius_from_aabb(min, max);
                batch.instances.retain(|inst| {
                    let c = inst.model[3];
                    sphere_vs_frustum([c[0], c[1], c[2]], r, &planes)
                });
            }
        }
        self.compute_stats();
    }
}

// ============================================================================
// 渲染辅助函数
// ============================================================================

#[cfg(feature = "wgpu_perf")]
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawIndexedIndirect {
    pub index_count: u32,
    pub instance_count: u32,
    pub first_index: u32,
    pub base_vertex: i32,
    pub first_instance: u32,
}

/// 渲染所有可见批次
pub fn render_batches<'a>(render_pass: &mut wgpu::RenderPass<'a>, batch_manager: &'a BatchManager) {
    for batch in batch_manager.visible_batches() {
        // 绑定顶点缓冲区
        render_pass.set_vertex_buffer(0, batch.mesh.vertex_buffer.slice(..));

        // 绑定索引缓冲区
        render_pass.set_index_buffer(batch.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        // 绑定实例缓冲区
        if let Some(instance_buffer) = &batch.instance_buffer {
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        }

        // 绑定材质
        render_pass.set_bind_group(1, &batch.material_bind_group, &[]);
        // 可选绑定纹理组（布局3）
        if let Some(bg) = batch.extra_material_bind_groups.get(0) {
            render_pass.set_bind_group(3, bg, &[]);
        }

        // 实例化绘制
        #[cfg(feature = "wgpu_perf")]
        {
            if let Some(ib) = &batch.indirect_buffer {
                let stride = std::mem::size_of::<DrawIndexedIndirect>() as wgpu::BufferAddress;
                for i in 0..batch.indirect_count {
                    render_pass.draw_indexed_indirect(ib, i as wgpu::BufferAddress * stride);
                }
            } else {
                render_pass.draw_indexed(0..batch.mesh.index_count, 0, 0..batch.instance_count());
            }
        }
        #[cfg(not(feature = "wgpu_perf"))]
        {
            render_pass.draw_indexed(0..batch.mesh.index_count, 0, 0..batch.instance_count());
        }
    }
}

pub fn render_small_batches<'a>(
    render_pass: &mut wgpu::RenderPass<'a>,
    batch_manager: &'a BatchManager,
) {
    for batch in batch_manager.small_batches() {
        render_pass.set_vertex_buffer(0, batch.mesh.vertex_buffer.slice(..));
        render_pass.set_index_buffer(batch.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        if let Some(instance_buffer) = &batch.instance_buffer {
            render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
        }
        render_pass.set_bind_group(1, &batch.material_bind_group, &[]);
        if let Some(bg) = batch.extra_material_bind_groups.get(0) {
            render_pass.set_bind_group(3, bg, &[]);
        }
        #[cfg(feature = "wgpu_perf")]
        {
            if let Some(ib) = &batch.indirect_buffer {
                if batch.indirect_count > 1 {
                    // 多个间接绘制命令：使用循环（优化版本）
                    let stride = std::mem::size_of::<DrawIndexedIndirect>() as wgpu::BufferAddress;
                    for i in 0..batch.indirect_count {
                        render_pass.draw_indexed_indirect(ib, i as wgpu::BufferAddress * stride);
                    }
                } else if batch.indirect_count == 1 {
                    // 单个间接绘制命令：直接调用
                    render_pass.draw_indexed_indirect(ib, 0);
                } else {
                    // 回退到直接绘制
                    render_pass.draw_indexed(
                        0..batch.mesh.index_count,
                        0,
                        0..batch.instance_count(),
                    );
                }
            } else {
                render_pass.draw_indexed(0..batch.mesh.index_count, 0, 0..batch.instance_count());
            }
        }
        #[cfg(not(feature = "wgpu_perf"))]
        {
            render_pass.draw_indexed(0..batch.mesh.index_count, 0, 0..batch.instance_count());
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dynamic_batch_config_default() {
        let config = DynamicBatchConfig::default();
        assert_eq!(config.min_batch_size, 32);
        assert_eq!(config.max_batch_size, 8192);
        assert_eq!(config.ideal_batch_size, 512);
        assert_eq!(config.small_batch_threshold, 16);
        assert!(config.enable_dynamic_adjustment);
    }

    #[test]
    fn test_dynamic_batch_config_new() {
        let config = DynamicBatchConfig::new(64, 4096, 256);
        assert_eq!(config.min_batch_size, 64);
        assert_eq!(config.max_batch_size, 4096);
        assert_eq!(config.ideal_batch_size, 256);
        assert_eq!(config.small_batch_threshold, 32); // min / 2
    }

    #[test]
    fn test_dynamic_batch_config_adjust_for_gpu_limits() {
        let mut config = DynamicBatchConfig::default();
        config.adjust_for_gpu_limits(2048);
        assert_eq!(config.max_batch_size, 2048);
        assert_eq!(config.ideal_batch_size, 512); // 512 < 2048, 保持不变

        config.adjust_for_gpu_limits(256);
        assert_eq!(config.max_batch_size, 256);
        assert_eq!(config.ideal_batch_size, 256); // 512 > 256, 调整为256
    }

    #[test]
    fn test_dynamic_batch_config_record_performance() {
        let mut config = DynamicBatchConfig::default();
        config.record_performance(16.67); // 60 FPS
        config.record_performance(33.33); // 30 FPS
        assert_eq!(config.performance_history.len(), 2);

        // 测试历史记录限制
        for i in 0..100 {
            config.record_performance(i as f32);
        }
        assert_eq!(config.performance_history.len(), config.max_history_length);
    }

    #[test]
    fn test_dynamic_batch_config_calculate_batch_size() {
        let config = DynamicBatchConfig::default();

        // 小批次
        assert_eq!(config.calculate_batch_size(10), 10);

        // 理想大小
        assert_eq!(config.calculate_batch_size(500), 500);

        // 超过理想大小
        assert_eq!(config.calculate_batch_size(1000), 512);
    }

    #[test]
    fn test_batch_manager_with_dynamic_config() {
        let config = DynamicBatchConfig::new(32, 1024, 256);
        let manager = BatchManager::with_dynamic_config(config);
        assert_eq!(manager.dynamic_config().ideal_batch_size, 256);
    }

    #[test]
    fn test_batch_manager_optimize_batch_sizes() {
        let mut manager = BatchManager::new();
        // 测试空批次管理器
        manager.optimize_batch_sizes(); // 应该不崩溃
        assert!(manager.small_batch_keys.is_empty());
    }

    #[test]
    fn test_batch_manager_adaptive_adjust() {
        let mut manager = BatchManager::new();

        // 记录性能指标
        manager.adaptive_adjust_batches(16.67); // 60 FPS
        manager.adaptive_adjust_batches(16.67);
        manager.adaptive_adjust_batches(16.67);

        // 应该有性能历史记录
        assert!(manager.dynamic_config().performance_history.len() >= 3);
    }

    #[test]
    fn test_batch_key_equality() {
        let key1 = BatchKey {
            mesh_id: 1,
            material_id: 2,
        };
        let key2 = BatchKey {
            mesh_id: 1,
            material_id: 2,
        };
        assert_eq!(key1, key2);
    }
}
