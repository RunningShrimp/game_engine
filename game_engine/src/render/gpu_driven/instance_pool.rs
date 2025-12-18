//! 实例数据池模块
//!
//! 实现持久化的GPU实例数据管理，支持增量更新和脏标记系统。
//!
//! ## 性能优化
//!
//! - **数据持久化**: 实例数据在GPU上持久化，避免每帧重新上传
//! - **增量更新**: 只上传变化的实例数据，减少CPU-GPU带宽占用
//! - **脏标记系统**: 使用块级和实例级脏标记，快速检测变化
//! - **批量更新**: 合并脏范围，减少API调用开销

use crate::render::gpu_driven::culling::GpuInstance;
use crate::render::gpu_driven::indirect::IndirectDrawError;
use wgpu::{Buffer, Device, Queue};

/// 实例数据池
///
/// 管理GPU上的实例数据，支持增量更新和持久化存储。
///
/// ## 设计要点
///
/// - 使用GPU存储缓冲区持久化实例数据
/// - 实现脏标记系统，只更新变化的实例
/// - 支持批量增量更新
/// - 自动扩展缓冲区容量
///
/// # 使用示例
///
/// ```rust
/// use game_engine::render::gpu_driven::instance_pool::InstanceDataPool;
///
/// // 创建实例数据池
/// let mut pool = InstanceDataPool::new(device, 10000);
///
/// // 更新实例数据（增量更新）
/// pool.update_instances(queue, &instances);
///
/// // 标记特定实例为脏
/// pool.mark_dirty(&[0, 1, 2]);
///
/// // 获取GPU缓冲区
/// let buffer = pool.buffer();
/// ```
pub struct InstanceDataPool {
    /// GPU存储缓冲区（持久化）
    instance_buffer: Buffer,
    /// 脏标记位图（CPU端）
    dirty_bits: Vec<bool>,
    /// 块级脏标记（用于快速检测）
    chunk_dirty: Vec<bool>,
    /// 块大小（每个块的实例数）
    chunk_size: usize,
    /// 最大实例数
    max_instances: u32,
    /// 当前实例数
    current_count: u32,
    /// 上一帧的实例数据（用于比较）
    prev_instances: Vec<GpuInstance>,
}

impl InstanceDataPool {
    /// 默认块大小
    ///
    /// 每个块包含128个实例，用于块级脏标记。
    pub const DEFAULT_CHUNK_SIZE: usize = 128;

    /// 创建新的实例数据池
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `max_instances` - 最大实例数
    ///
    /// # 返回
    ///
    /// 返回一个初始化的实例数据池。
    pub fn new(device: &Device, max_instances: u32) -> Self {
        Self::with_chunk_size(device, max_instances, Self::DEFAULT_CHUNK_SIZE)
    }

    /// 使用自定义块大小创建实例数据池
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `max_instances` - 最大实例数
    /// * `chunk_size` - 块大小（每个块的实例数）
    ///
    /// # 返回
    ///
    /// 返回一个初始化的实例数据池。
    pub fn with_chunk_size(device: &Device, max_instances: u32, chunk_size: usize) -> Self {
        let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;
        let buffer_size = instance_size * max_instances as wgpu::BufferAddress;
        // 对齐到256字节边界，提高GPU内存访问效率
        let aligned_size = (buffer_size + 255) & !255;

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Data Pool"),
            size: aligned_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let chunk_count = (max_instances as usize + chunk_size - 1) / chunk_size;

        Self {
            instance_buffer,
            dirty_bits: vec![true; max_instances as usize], // 初始全部标记为脏
            chunk_dirty: vec![true; chunk_count],
            chunk_size,
            max_instances,
            current_count: 0,
            prev_instances: Vec::with_capacity(max_instances as usize),
        }
    }

    /// 更新实例数据（增量更新）
    ///
    /// 只上传变化的实例数据，减少CPU-GPU带宽占用。
    ///
    /// # 参数
    ///
    /// * `queue` - WGPU命令队列
    /// * `instances` - 实例数据切片
    ///
    /// # 错误
    ///
    /// 如果实例数量超过最大容量，返回错误。
    ///
    /// # 性能
    ///
    /// 此方法会自动检测变化，只上传脏实例数据。
    pub fn update_instances(
        &mut self,
        queue: &Queue,
        instances: &[GpuInstance],
    ) -> Result<(), IndirectDrawError> {
        if instances.len() > self.max_instances as usize {
            return Err(IndirectDrawError::InsufficientCapacity {
                required: instances.len() as u32,
                available: self.max_instances,
            });
        }

        let new_count = instances.len() as u32;

        // 如果实例数量变化，需要完整重建
        if new_count != self.current_count {
            self.current_count = new_count;
            self.mark_all_dirty();
        }

        // 检测变化并生成脏范围
        let dirty_ranges = self.detect_changes(instances);

        // 批量上传脏范围
        if !dirty_ranges.is_empty() {
            self.upload_dirty_ranges(queue, instances, &dirty_ranges)?;
        }

        // 更新上一帧数据
        self.prev_instances.clear();
        self.prev_instances.extend_from_slice(instances);

        // 清除脏标记
        self.clear_dirty_flags();

        Ok(())
    }

    /// 标记实例为脏（需要更新）
    ///
    /// # 参数
    ///
    /// * `instance_ids` - 需要标记为脏的实例ID列表
    pub fn mark_dirty(&mut self, instance_ids: &[u32]) {
        for &id in instance_ids {
            if (id as usize) < self.dirty_bits.len() {
                self.dirty_bits[id as usize] = true;
                let chunk_idx = id as usize / self.chunk_size;
                if chunk_idx < self.chunk_dirty.len() {
                    self.chunk_dirty[chunk_idx] = true;
                }
            }
        }
    }

    /// 标记实例范围为脏
    ///
    /// # 参数
    ///
    /// * `start` - 起始实例ID
    /// * `end` - 结束实例ID（不包含）
    pub fn mark_range_dirty(&mut self, start: u32, end: u32) {
        let start_idx = start as usize;
        let end_idx = (end as usize).min(self.dirty_bits.len());
        
        for i in start_idx..end_idx {
            self.dirty_bits[i] = true;
        }
        
        let chunk_start = start_idx / self.chunk_size;
        let chunk_end = (end_idx + self.chunk_size - 1) / self.chunk_size;
        for i in chunk_start..chunk_end.min(self.chunk_dirty.len()) {
            self.chunk_dirty[i] = true;
        }
    }

    /// 标记所有实例为脏
    ///
    /// 用于强制完整更新所有实例数据。
    pub fn mark_all_dirty(&mut self) {
        for flag in &mut self.dirty_bits {
            *flag = true;
        }
        for flag in &mut self.chunk_dirty {
            *flag = true;
        }
    }

    /// 检测变化并生成脏范围
    ///
    /// # 参数
    ///
    /// * `instances` - 当前帧的实例数据
    ///
    /// # 返回
    ///
    /// 返回脏范围列表，每个范围表示`(start, end)`索引对。
    fn detect_changes(&mut self, instances: &[GpuInstance]) -> Vec<(u32, u32)> {
        let mut dirty_ranges = Vec::new();
        let mut range_start: Option<usize> = None;

        for i in 0..instances.len().min(self.dirty_bits.len()) {
            let is_dirty = self.dirty_bits[i]
                || self.prev_instances.get(i).map(|prev| {
                    // 比较实例数据
                    prev.instance_id != instances[i].instance_id
                        || prev.aabb_min != instances[i].aabb_min
                        || prev.aabb_max != instances[i].aabb_max
                        || prev.model != instances[i].model
                        || prev.flags != instances[i].flags
                }).unwrap_or(true);

            if is_dirty {
                if range_start.is_none() {
                    range_start = Some(i);
                }
                self.dirty_bits[i] = true;
            } else {
                if let Some(start) = range_start {
                    dirty_ranges.push((start as u32, i as u32));
                    range_start = None;
                }
            }
        }

        // 处理最后一个范围
        if let Some(start) = range_start {
            dirty_ranges.push((start as u32, instances.len() as u32));
        }

        dirty_ranges
    }

    /// 上传脏范围到GPU
    ///
    /// # 参数
    ///
    /// * `queue` - WGPU命令队列
    /// * `instances` - 实例数据
    /// * `dirty_ranges` - 脏范围列表
    ///
    /// # 错误
    ///
    /// 如果上传失败，返回错误。
    fn upload_dirty_ranges(
        &self,
        queue: &Queue,
        instances: &[GpuInstance],
        dirty_ranges: &[(u32, u32)],
    ) -> Result<(), IndirectDrawError> {
        let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;

        for &(start, end) in dirty_ranges {
            let start_idx = start as usize;
            let end_idx = (end as usize).min(instances.len());

            if start_idx >= instances.len() {
                continue;
            }

            let data = &instances[start_idx..end_idx];
            let offset = (start_idx as u32 * instance_size as u32) as wgpu::BufferAddress;

            queue.write_buffer(&self.instance_buffer, offset, bytemuck::cast_slice(data));
        }

        Ok(())
    }

    /// 清除脏标记
    ///
    /// 在更新完成后清除所有脏标记。
    fn clear_dirty_flags(&mut self) {
        for flag in &mut self.dirty_bits {
            *flag = false;
        }
        for flag in &mut self.chunk_dirty {
            *flag = false;
        }
    }

    /// 获取GPU缓冲区
    ///
    /// # 返回
    ///
    /// 返回GPU存储缓冲区的引用。
    pub fn buffer(&self) -> &Buffer {
        &self.instance_buffer
    }

    /// 获取最大实例数
    ///
    /// # 返回
    ///
    /// 返回最大实例数。
    pub fn max_instances(&self) -> u32 {
        self.max_instances
    }

    /// 获取当前实例数
    ///
    /// # 返回
    ///
    /// 返回当前实例数。
    pub fn current_count(&self) -> u32 {
        self.current_count
    }

    /// 确保缓冲区容量足够（自动扩展）
    ///
    /// 如果需要的实例数超过当前容量，自动扩展缓冲区。
    ///
    /// # 参数
    ///
    /// * `device` - WGPU设备
    /// * `required_instances` - 需要的实例数
    ///
    /// # 错误
    ///
    /// 如果扩展失败，返回错误。
    pub fn ensure_capacity(
        &mut self,
        device: &Device,
        required_instances: u32,
    ) -> Result<(), IndirectDrawError> {
        if required_instances <= self.max_instances {
            return Ok(());
        }

        // 扩展容量（预留50%额外空间）
        let new_max_instances = (required_instances * 3 / 2).max(64);
        let instance_size = std::mem::size_of::<GpuInstance>() as wgpu::BufferAddress;
        let buffer_size = instance_size * new_max_instances as wgpu::BufferAddress;
        let aligned_size = (buffer_size + 255) & !255;

        self.instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Data Pool (Resized)"),
            size: aligned_size,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        // 扩展脏标记位图
        self.dirty_bits.resize(new_max_instances as usize, false);
        let chunk_count = (new_max_instances as usize + self.chunk_size - 1) / self.chunk_size;
        self.chunk_dirty.resize(chunk_count, false);

        self.max_instances = new_max_instances;
        self.mark_all_dirty(); // 标记所有实例为脏，强制完整更新

        Ok(())
    }

    /// 检查是否有脏实例
    ///
    /// # 返回
    ///
    /// 如果有脏实例，返回`true`；否则返回`false`。
    pub fn has_dirty(&self) -> bool {
        self.chunk_dirty.iter().any(|&dirty| dirty)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_pool_creation() {
        // 注意：实际测试需要WGPU设备
        // let pool = InstanceDataPool::new(device, 1000);
        // assert_eq!(pool.max_instances(), 1000);
        // assert_eq!(pool.current_count(), 0);
    }

    #[test]
    fn test_mark_dirty() {
        // let mut pool = InstanceDataPool::new(device, 1000);
        // pool.mark_dirty(&[0, 1, 2]);
        // assert!(pool.has_dirty());
    }
}

