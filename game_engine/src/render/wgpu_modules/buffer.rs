//! WGPU 缓冲区管理
//!
//! 包含缓冲区创建和管理相关功能。

use super::types::Instance;

/// 双缓冲实例管理器
///
/// 使用 ping-pong 缓冲实现无等待 GPU 上传。
pub struct DoubleBufferedInstances {
    /// 两个实例缓冲区 (ping-pong)
    buffers: [wgpu::Buffer; 2],
    /// 当前活动缓冲区索引
    active_idx: usize,
    /// 缓冲区容量 (实例数)
    capacity: u32,
    /// 当前实例数
    count: u32,
    /// Staging 缓冲区用于异步上传
    staging_buffer: wgpu::Buffer,
}

impl DoubleBufferedInstances {
    /// 创建双缓冲实例管理器
    pub fn new(device: &wgpu::Device, initial_capacity: u32) -> Self {
        let buffer_size =
            (initial_capacity as usize * std::mem::size_of::<Instance>()) as wgpu::BufferAddress;

        let buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer 0"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer 1"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];

        let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            buffers,
            active_idx: 0,
            capacity: initial_capacity,
            count: 0,
            staging_buffer,
        }
    }

    /// 获取当前活动缓冲区 (用于渲染)
    pub fn active_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[self.active_idx]
    }

    /// 获取后台缓冲区 (用于写入)
    pub fn back_buffer(&self) -> &wgpu::Buffer {
        &self.buffers[1 - self.active_idx]
    }

    /// 交换前后缓冲区
    pub fn swap(&mut self) {
        self.active_idx = 1 - self.active_idx;
    }

    /// 同步更新实例数据到后台缓冲区并交换
    pub fn update_sync(&mut self, queue: &wgpu::Queue, instances: &[Instance]) {
        self.count = instances.len() as u32;
        if !instances.is_empty() {
            queue.write_buffer(self.back_buffer(), 0, bytemuck::cast_slice(instances));
        }
        self.swap();
    }

    /// 异步更新实例数据 (使用staging buffer + copy命令)
    pub fn update_with_staging(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[Instance],
    ) -> Option<wgpu::CommandBuffer> {
        if instances.is_empty() {
            self.count = 0;
            return None;
        }

        self.count = instances.len() as u32;
        let byte_size = (instances.len() * std::mem::size_of::<Instance>()) as wgpu::BufferAddress;

        queue.write_buffer(&self.staging_buffer, 0, bytemuck::cast_slice(instances));

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Instance Copy Encoder"),
        });

        encoder.copy_buffer_to_buffer(&self.staging_buffer, 0, self.back_buffer(), 0, byte_size);

        self.swap();
        Some(encoder.finish())
    }

    /// 获取当前实例数
    pub fn count(&self) -> u32 {
        self.count
    }

    /// 扩展缓冲区容量
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, required: u32) {
        if required <= self.capacity {
            return;
        }

        let new_capacity = (required as f32 * 1.5) as u32;
        let buffer_size =
            (new_capacity as usize * std::mem::size_of::<Instance>()) as wgpu::BufferAddress;

        self.buffers = [
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer 0"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Instance Buffer 1"),
                size: buffer_size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
        ];

        self.staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Instance Staging Buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        self.capacity = new_capacity;
        self.active_idx = 0;
    }
}

/// 实例脏标记追踪器
///
/// 用于追踪哪些实例已更改，实现增量更新。
pub struct InstanceDirtyTracker {
    /// 每个块的大小
    chunk_size: usize,
    /// 块级脏标记
    chunk_dirty: Vec<bool>,
    /// 实例级脏标记
    instance_dirty: Vec<bool>,
    /// 上一帧的实例数据
    prev_instances: Vec<Instance>,
    /// 脏范围列表
    dirty_ranges: Vec<(u32, u32)>,
    /// 总实例数
    instance_count: usize,
    /// 是否需要完整重建
    needs_full_rebuild: bool,
}

impl InstanceDirtyTracker {
    /// 默认块大小
    pub const DEFAULT_CHUNK_SIZE: usize = 128;

    /// 创建脏标记追踪器
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

    /// 使用默认配置创建
    pub fn with_capacity(capacity: usize) -> Self {
        Self::new(capacity, Self::DEFAULT_CHUNK_SIZE)
    }

    /// 标记所有实例为脏
    pub fn mark_all_dirty(&mut self) {
        self.needs_full_rebuild = true;
        for flag in &mut self.chunk_dirty {
            *flag = true;
        }
        for flag in &mut self.instance_dirty {
            *flag = true;
        }
    }

    /// 标记特定实例为脏
    #[inline]
    pub fn mark_instance_dirty(&mut self, index: usize) {
        if index < self.instance_dirty.len() {
            self.instance_dirty[index] = true;
            let chunk_idx = index / self.chunk_size;
            if chunk_idx < self.chunk_dirty.len() {
                self.chunk_dirty[chunk_idx] = true;
            }
        }
    }

    /// 标记实例范围为脏
    pub fn mark_range_dirty(&mut self, start: usize, end: usize) {
        let end = end.min(self.instance_dirty.len());
        for i in start..end {
            self.instance_dirty[i] = true;
        }
        let chunk_start = start / self.chunk_size;
        let chunk_end = (end + self.chunk_size - 1) / self.chunk_size;
        for i in chunk_start..chunk_end.min(self.chunk_dirty.len()) {
            self.chunk_dirty[i] = true;
        }
    }

    /// 更新并检测变化
    pub fn update(&mut self, instances: &[Instance]) -> &[(u32, u32)] {
        self.dirty_ranges.clear();

        let new_count = instances.len();
        let old_count = self.prev_instances.len();

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

        for chunk_idx in 0..self.chunk_dirty.len() {
            if !self.chunk_dirty[chunk_idx] {
                if let Some(start) = range_start {
                    let end = (chunk_idx * self.chunk_size).min(new_count) as u32;
                    self.dirty_ranges.push((start, end));
                    range_start = None;
                }
                continue;
            }

            let start = chunk_idx * self.chunk_size;
            let end = ((chunk_idx + 1) * self.chunk_size).min(new_count);

            let mut chunk_has_changes = false;
            for i in start..end {
                let is_dirty = if i < old_count {
                    !instances[i].equals(&self.prev_instances[i])
                } else {
                    true
                };

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

        if let Some(start) = range_start {
            self.dirty_ranges.push((start, new_count as u32));
        }

        self.prev_instances.clear();
        self.prev_instances.extend_from_slice(instances);

        self.merge_ranges();

        &self.dirty_ranges
    }

    /// 合并相邻范围
    fn merge_ranges(&mut self) {
        if self.dirty_ranges.len() <= 1 {
            return;
        }

        self.dirty_ranges.sort_by_key(|r| r.0);

        let mut merged = Vec::with_capacity(self.dirty_ranges.len());
        let mut current = self.dirty_ranges[0];

        for &(start, end) in &self.dirty_ranges[1..] {
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

    /// 获取脏范围数量
    pub fn dirty_range_count(&self) -> usize {
        self.dirty_ranges.len()
    }

    /// 获取脏实例总数
    pub fn dirty_instance_count(&self) -> usize {
        self.dirty_ranges
            .iter()
            .map(|(s, e)| (e - s) as usize)
            .sum()
    }

    /// 检查是否有任何脏数据
    pub fn has_dirty(&self) -> bool {
        !self.dirty_ranges.is_empty()
    }

    /// 重置追踪器
    pub fn reset(&mut self) {
        self.chunk_dirty.clear();
        self.instance_dirty.clear();
        self.prev_instances.clear();
        self.dirty_ranges.clear();
        self.instance_count = 0;
        self.needs_full_rebuild = true;
    }
}

impl Default for InstanceDirtyTracker {
    fn default() -> Self {
        Self::with_capacity(1024)
    }
}
