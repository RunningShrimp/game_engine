//  环形缓冲区池模块
//
//  实现高性能的环形缓冲区管理，用于优化Staging Buffer的内存分配。
//
//  ## 架构设计
//
//  ```text
//  ┌─────────────────────────────────────────────────────────┐
//  │                  Ring Buffer Pool                       │
//  ├─────────────────────────────────────────────────────────┤
//  │  1. 三重缓冲机制                                        │
//  │     - Buffer 0: CPU写入 (当前帧)                        │
//  │     - Buffer 1: GPU使用 (上一帧)                        │
//  │     - Buffer 2: 待回收 (前两帧)                          │
//  │                                                          │
//  │  2. 环形索引管理                                        │
//  │     - 写入指针: 当前写入位置                             │
//  │     - 读取指针: GPU读取位置                              │
//  │     - 安全边界: 避免覆盖未使用的内存                      │
//  │                                                          │
//  │  3. 内存块分级                                          │
//  │     - 小块: 64KB (频繁分配的小数据)                      │
//  │     - 中块: 1MB (中等大小数据)                          │
//  │     - 大块: 4MB (大块数据)                              │
//  └─────────────────────────────────────────────────────────┘
//  ```

use std::collections::VecDeque;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use parking_lot::Mutex;

// ============================================================================
// 常量配置
// ============================================================================

/// 三重缓冲数量
const TRIPLE_BUFFER_COUNT: usize = 3;

/// 小块大小 (64KB)
const SMALL_BLOCK_SIZE: u64 = 64 * 1024;

/// 中块大小 (1MB)
const MEDIUM_BLOCK_SIZE: u64 = 1024 * 1024;

/// 大块大小 (4MB)
const LARGE_BLOCK_SIZE: u64 = 4 * 1024 * 1024;

/// 默认环形缓冲区大小 (16MB)
const DEFAULT_RING_BUFFER_SIZE: u64 = 16 * 1024 * 1024;

/// GPU延迟帧数 (确保GPU完成使用)
const GPU_FRAME_DELAY: u64 = 2;

// ============================================================================
// 内存块分级
// ============================================================================

/// 内存块大小分级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockSize {
    /// 小块 (64KB)
    Small,
    /// 中块 (1MB)
    Medium,
    /// 大块 (4MB)
    Large,
    /// 自定义大小
    Custom(u64),
}

impl BlockSize {
    /// 获取块的实际大小
    pub fn size(&self) -> u64 {
        match self {
            BlockSize::Small => SMALL_BLOCK_SIZE,
            BlockSize::Medium => MEDIUM_BLOCK_SIZE,
            BlockSize::Large => LARGE_BLOCK_SIZE,
            BlockSize::Custom(size) => *size,
        }
    }

    /// 根据请求大小选择合适的块大小
    pub fn for_request(requested_size: u64) -> Self {
        if requested_size <= SMALL_BLOCK_SIZE {
            BlockSize::Small
        } else if requested_size <= MEDIUM_BLOCK_SIZE {
            BlockSize::Medium
        } else if requested_size <= LARGE_BLOCK_SIZE {
            BlockSize::Large
        } else {
            BlockSize::Custom(requested_size.next_power_of_two())
        }
    }
}

impl std::fmt::Display for BlockSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockSize::Small => write!(f, "Small(64KB)"),
            BlockSize::Medium => write!(f, "Medium(1MB)"),
            BlockSize::Large => write!(f, "Large(4MB)"),
            BlockSize::Custom(size) => write!(f, "Custom({}B)", size),
        }
    }
}

// ============================================================================
// 缓冲区状态
// ============================================================================

/// 缓冲区状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferState {
    /// 空闲 - 可用于写入
    Idle,
    /// 写入中 - CPU正在写入数据
    Writing,
    /// GPU使用中 - GPU正在读取数据
    InUse,
    /// 待回收 - 等待GPU完成使用
    Pending,
}

// ============================================================================
// 内存分配统计
// ============================================================================

/// 内存分配统计信息
#[derive(Default, Clone, Debug)]
pub struct AllocationStats {
    /// 总分配次数
    pub total_allocations: u64,
    /// 总分配字节数
    pub total_bytes_allocated: u64,
    /// 当前活跃分配数
    pub active_allocations: u64,
    /// 当前活跃字节数
    pub active_bytes: u64,
    /// 峰值使用字节数
    pub peak_bytes: u64,
    /// 碎片化程度 (0.0-1.0)
    pub fragmentation_ratio: f32,
    /// 复用次数
    pub reuse_count: u64,
    /// 分配延迟 (微秒)
    pub allocation_latency_us: f32,
}

impl AllocationStats {
    /// 更新峰值使用量
    pub fn update_peak(&mut self) {
        self.peak_bytes = self.peak_bytes.max(self.active_bytes);
    }

    /// 计算碎片化程度
    pub fn calculate_fragmentation(&mut self, total_capacity: u64) {
        if total_capacity == 0 {
            self.fragmentation_ratio = 0.0;
        } else {
            // 碎片化 = (总容量 - 活跃字节数) / 总容量
            self.fragmentation_ratio =
                (total_capacity - self.active_bytes) as f32 / total_capacity as f32;
        }
    }
}

// ============================================================================
// 内存块
// ============================================================================

/// 内存块分配记录
#[derive(Debug, Clone)]
pub struct MemoryBlock {
    /// 块ID
    pub id: u64,
    /// 块大小
    pub size: u64,
    /// 在环形缓冲区中的偏移
    pub offset: u64,
    /// 对齐要求
    pub alignment: u64,
    /// 块状态
    pub state: BufferState,
    /// 创建帧号
    pub created_frame: u64,
    /// 完成帧号 (GPU完成使用时设置)
    pub completed_frame: Option<u64>,
    /// GPU 使用帧号 (用于帧计数同步)
    pub gpu_usage_frame: Option<u64>,
}

impl MemoryBlock {
    /// 创建新的内存块
    pub fn new(id: u64, size: u64, offset: u64, alignment: u64, frame: u64) -> Self {
        Self {
            id,
            size,
            offset,
            alignment,
            state: BufferState::Idle,
            created_frame: frame,
            completed_frame: None,
            gpu_usage_frame: None,
        }
    }

    /// 检查块是否可以复用
    pub fn can_reuse(&self, current_frame: u64) -> bool {
        match self.state {
            BufferState::Idle => true,
            BufferState::Pending => {
                // 检查GPU是否已完成使用
                if let Some(completed_frame) = self.completed_frame {
                    current_frame > completed_frame + GPU_FRAME_DELAY
                } else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 标记为写入状态
    pub fn mark_writing(&mut self) {
        self.state = BufferState::Writing;
    }

    /// 标记为GPU使用状态
    pub fn mark_in_use(&mut self, frame: u64) {
        self.state = BufferState::InUse;
        self.gpu_usage_frame = Some(frame);
    }

    /// 标记为待回收状态
    pub fn mark_pending(&mut self, frame: u64) {
        self.state = BufferState::Pending;
        self.completed_frame = Some(frame);
    }

    /// 重置为空闲状态
    pub fn reset(&mut self) {
        self.state = BufferState::Idle;
        self.completed_frame = None;
        self.gpu_usage_frame = None;
    }
}

// ============================================================================
// 环形缓冲区
// ============================================================================

/// 单个环形缓冲区
#[derive(Debug)]
pub struct RingBuffer {
    /// GPU缓冲区
    pub buffer: wgpu::Buffer,
    /// 缓冲区大小
    pub size: u64,
    /// 当前写入偏移
    write_offset: AtomicU64,
    /// 当前读取偏移 (GPU使用位置)
    read_offset: AtomicU64,
    /// 缓冲区状态
    state: AtomicUsize, // BufferState的原子表示
    /// 当前帧号
    current_frame: AtomicU64,
    /// 分配的内存块
    blocks: Mutex<Vec<MemoryBlock>>,
    /// 空闲块列表 (按大小排序)
    free_blocks: Mutex<VecDeque<MemoryBlock>>,
    /// 下一个块ID
    next_block_id: AtomicU64,
}

impl RingBuffer {
    /// 创建新的环形缓冲区
    pub fn new(device: &wgpu::Device, size: u64, label: Option<&str>) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            size,
            write_offset: AtomicU64::new(0),
            read_offset: AtomicU64::new(0),
            state: AtomicUsize::new(BufferState::Idle as usize),
            current_frame: AtomicU64::new(0),
            blocks: Mutex::new(Vec::new()),
            free_blocks: Mutex::new(VecDeque::new()),
            next_block_id: AtomicU64::new(1),
        }
    }

    /// 分配内存块
    pub fn allocate(&self, size: u64, alignment: u64, frame: u64) -> Option<MemoryBlock> {
        // 对齐大小和偏移
        let aligned_size = align_to(size, alignment);

        // 尝试从空闲块中复用
        if let Some(mut block) = self.try_reuse_block(aligned_size, alignment, frame) {
            block.mark_writing();
            return Some(block);
        }

        // 在环形缓冲区末尾分配新块
        let current_write = self.write_offset.load(Ordering::Acquire);
        let aligned_offset = align_to(current_write, alignment);
        let end_offset = aligned_offset + aligned_size;

        // 检查是否超出缓冲区边界
        if end_offset > self.size {
            // 尝试环绕到开头
            let current_read = self.read_offset.load(Ordering::Acquire);
            if aligned_offset < current_read {
                // 没有足够空间，需要等待GPU完成
                return None;
            }

            // 环绕到开头
            let new_write_offset = align_to(0, alignment);
            if new_write_offset + aligned_size > current_read {
                return None; // 空间不足
            }

            // 更新写入偏移
            if self
                .write_offset
                .compare_exchange(
                    current_write,
                    new_write_offset + aligned_size,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                let block_id = self.next_block_id.fetch_add(1, Ordering::Relaxed);
                let mut block =
                    MemoryBlock::new(block_id, aligned_size, new_write_offset, alignment, frame);
                block.mark_writing();
                return Some(block);
            }
        } else {
            // 正常分配
            if self
                .write_offset
                .compare_exchange(
                    current_write,
                    end_offset,
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                let block_id = self.next_block_id.fetch_add(1, Ordering::Relaxed);
                let mut block =
                    MemoryBlock::new(block_id, aligned_size, aligned_offset, alignment, frame);
                block.mark_writing();
                return Some(block);
            }
        }

        None // 分配失败
    }

    /// 尝试复用空闲块
    fn try_reuse_block(&self, size: u64, alignment: u64, frame: u64) -> Option<MemoryBlock> {
        let mut free_blocks = self.free_blocks.lock();

        // 寻找大小合适的块 (最佳适配)
        let mut best_index = None;
        let mut best_size = u64::MAX;

        for (i, block) in free_blocks.iter().enumerate() {
            if block.size >= size
                && block.offset % alignment == 0
                && block.can_reuse(frame)
                && block.size < best_size
            {
                best_index = Some(i);
                best_size = block.size;
            }
        }

        if let Some(index) = best_index {
            let mut block = free_blocks.remove(index).unwrap();
            block.mark_writing();
            return Some(block);
        }

        None
    }

    /// 释放内存块
    pub fn deallocate(&self, mut block: MemoryBlock) {
        block.mark_pending(self.current_frame.load(Ordering::Relaxed));

        let mut free_blocks = self.free_blocks.lock();

        // 按偏移排序插入，保持有序
        let insert_pos = free_blocks
            .binary_search_by_key(&block.offset, |b| b.offset)
            .unwrap_or_else(|pos| pos);

        free_blocks.insert(insert_pos, block);

        // 尝试合并相邻的空闲块
        self.coalesce_free_blocks(&mut free_blocks);
    }

    /// 合并相邻的空闲块
    fn coalesce_free_blocks(&self, free_blocks: &mut VecDeque<MemoryBlock>) {
        if free_blocks.len() < 2 {
            return;
        }

        let mut i = 0;
        while i < free_blocks.len() - 1 {
            let current = &free_blocks[i];
            let next = &free_blocks[i + 1];

            // 检查是否相邻且都可以复用
            if current.offset + current.size == next.offset
                && current.can_reuse(self.current_frame.load(Ordering::Relaxed))
                && next.can_reuse(self.current_frame.load(Ordering::Relaxed))
            {
                // 合并块
                let mut merged = current.clone();
                merged.size += next.size;

                // 移除两个块并插入合并后的块
                free_blocks.remove(i + 1);
                free_blocks[i] = merged;
            } else {
                i += 1;
            }
        }
    }

    /// 更新帧号
    pub fn update_frame(&self, frame: u64) {
        self.current_frame.store(frame, Ordering::Relaxed);
    }

    /// 获取当前写入偏移
    pub fn write_offset(&self) -> u64 {
        self.write_offset.load(Ordering::Acquire)
    }

    /// 获取当前读取偏移
    pub fn read_offset(&self) -> u64 {
        self.read_offset.load(Ordering::Acquire)
    }

    /// 获取缓冲区状态
    pub fn state(&self) -> BufferState {
        match self.state.load(Ordering::Acquire) {
            0 => BufferState::Idle,
            1 => BufferState::Writing,
            2 => BufferState::InUse,
            3 => BufferState::Pending,
            _ => BufferState::Idle,
        }
    }

    /// 重置缓冲区 (用于调试)
    pub fn reset(&self) {
        self.write_offset.store(0, Ordering::Release);
        self.read_offset.store(0, Ordering::Release);
        self.state.store(BufferState::Idle as usize, Ordering::Release);
        self.blocks.lock().clear();
        self.free_blocks.lock().clear();
        self.next_block_id.store(1, Ordering::Relaxed);
    }
}

// ============================================================================
// 环形缓冲区池
// ============================================================================

/// 环形缓冲区池 - 管理多个环形缓冲区
#[derive(Debug)]
pub struct RingBufferPool {
    /// 环形缓冲区数组 (三重缓冲)
    ring_buffers: Vec<RingBuffer>,
    /// 当前活动缓冲区索引
    current_buffer_index: usize,
    /// 当前帧号
    current_frame: u64,
    /// 分配统计
    stats: Arc<Mutex<AllocationStats>>,
    /// 设备引用
    device: Arc<wgpu::Device>,
}

impl RingBufferPool {
    /// 获取设备引用（用于调试）
    pub fn device(&self) -> &Arc<wgpu::Device> {
        &self.device
    }

    /// 创建新的环形缓冲区池（使用默认大小）
    pub fn new_default(device: Arc<wgpu::Device>) -> Self {
        Self::new(device, DEFAULT_RING_BUFFER_SIZE)
    }

    /// 创建新的环形缓冲区池
    pub fn new(device: Arc<wgpu::Device>, buffer_size: u64) -> Self {
        let mut ring_buffers = Vec::with_capacity(TRIPLE_BUFFER_COUNT);

        // 创建三个环形缓冲区
        for i in 0..TRIPLE_BUFFER_COUNT {
            let buffer = RingBuffer::new(&device, buffer_size, Some(&format!("Ring Buffer {}", i)));
            ring_buffers.push(buffer);
        }

        Self {
            ring_buffers,
            current_buffer_index: 0,
            current_frame: 0,
            stats: Arc::new(Mutex::new(AllocationStats::default())),
            device,
        }
    }

    /// 分配内存块
    pub fn allocate(&mut self, size: u64, alignment: u64) -> Option<MemoryBlock> {
        let start_time = std::time::Instant::now();

        // 获取当前活动缓冲区
        let current_buffer = &self.ring_buffers[self.current_buffer_index];

        // 尝试分配
        if let Some(block) = current_buffer.allocate(size, alignment, self.current_frame) {
            // 更新统计信息
            {
                let mut stats = self.stats.lock();
                stats.total_allocations += 1;
                stats.total_bytes_allocated += size;
                stats.active_allocations += 1;
                stats.active_bytes += size;
                stats.update_peak();
                stats.allocation_latency_us = start_time.elapsed().as_micros() as f32;
            }

            return Some(block);
        }

        // 当前缓冲区空间不足，尝试切换到下一个缓冲区
        self.switch_to_next_buffer();

        // 在新缓冲区中再次尝试分配
        let current_buffer = &self.ring_buffers[self.current_buffer_index];
        if let Some(block) = current_buffer.allocate(size, alignment, self.current_frame) {
            // 更新统计信息
            {
                let mut stats = self.stats.lock();
                stats.total_allocations += 1;
                stats.total_bytes_allocated += size;
                stats.active_allocations += 1;
                stats.active_bytes += size;
                stats.update_peak();
                stats.allocation_latency_us = start_time.elapsed().as_micros() as f32;
            }

            Some(block)
        } else {
            None // 分配失败
        }
    }

    /// 释放内存块
    pub fn deallocate(&mut self, block: MemoryBlock) {
        // 更新统计信息
        {
            let mut stats = self.stats.lock();
            stats.active_allocations = stats.active_allocations.saturating_sub(1);
            stats.active_bytes = stats.active_bytes.saturating_sub(block.size);

            // 计算碎片化程度
            let total_capacity = self.ring_buffers.iter().map(|b| b.size).sum::<u64>();
            stats.calculate_fragmentation(total_capacity);
        }

        // 找到对应的环形缓冲区并释放
        for ring_buffer in &self.ring_buffers {
            if block.offset < ring_buffer.size {
                ring_buffer.deallocate(block);
                break;
            }
        }
    }

    /// 切换到下一个缓冲区
    fn switch_to_next_buffer(&mut self) {
        // 标记当前缓冲区为GPU使用状态
        let _current_buffer = &self.ring_buffers[self.current_buffer_index];

        // 使用帧计数进行同步 (wgpu 0.20+ 移除了 Fence API)
        // 记录当前帧号用于后续同步检查
        let current_frame = self.current_frame;

        // 标记所有活跃块为GPU使用状态
        // 这里需要更复杂的逻辑来跟踪每个块的状态
        // 简化实现：基于帧计数进行同步

        // 切换到下一个缓冲区
        self.current_buffer_index = (self.current_buffer_index + 1) % TRIPLE_BUFFER_COUNT;

        // 更新新缓冲区的帧号
        let next_buffer = &self.ring_buffers[self.current_buffer_index];
        next_buffer.update_frame(current_frame);
    }

    /// 帧结束时调用
    pub fn end_frame(&mut self) {
        self.current_frame += 1;

        // 更新所有缓冲区的帧号
        for ring_buffer in &self.ring_buffers {
            ring_buffer.update_frame(self.current_frame);
        }

        // 检查GPU是否完成了早期帧的工作
        self.check_gpu_completion();
    }

    /// 检查GPU完成状态
    fn check_gpu_completion(&mut self) {
        // 检查是否有缓冲区可以被回收
        // 这里需要与Fence同步
        // 简化实现：假设GPU延迟为2帧
        let safe_frame = self.current_frame.saturating_sub(GPU_FRAME_DELAY);

        for ring_buffer in &self.ring_buffers {
            // 检查可以复用的块
            let mut free_blocks = ring_buffer.free_blocks.lock();
            for block in free_blocks.iter_mut() {
                if block.state == BufferState::Pending
                    && block.completed_frame.unwrap_or(0) <= safe_frame
                {
                    block.reset();
                }
            }
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> AllocationStats {
        self.stats.lock().clone()
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        let mut stats = self.stats.lock();
        *stats = AllocationStats::default();
    }

    /// 获取当前活动缓冲区
    pub fn current_buffer(&self) -> &RingBuffer {
        &self.ring_buffers[self.current_buffer_index]
    }

    /// 获取总容量
    pub fn total_capacity(&self) -> u64 {
        self.ring_buffers.iter().map(|b| b.size).sum()
    }

    /// 获取使用率
    pub fn utilization(&self) -> f32 {
        let stats = self.stats.lock();
        if self.total_capacity() == 0 {
            0.0
        } else {
            stats.active_bytes as f32 / self.total_capacity() as f32
        }
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 对齐到指定边界
#[inline]
pub fn align_to(value: u64, alignment: u64) -> u64 {
    if alignment == 0 {
        return value;
    }
    (value + alignment - 1) & !(alignment - 1)
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_size_for_request() {
        assert_eq!(BlockSize::for_request(32 * 1024), BlockSize::Small);
        assert_eq!(BlockSize::for_request(128 * 1024), BlockSize::Medium);
        assert_eq!(BlockSize::for_request(2 * 1024 * 1024), BlockSize::Large);
        assert_eq!(
            BlockSize::for_request(8 * 1024 * 1024),
            BlockSize::Custom(8 * 1024 * 1024)
        );
    }

    #[test]
    fn test_align_to() {
        assert_eq!(align_to(0, 256), 0);
        assert_eq!(align_to(100, 256), 256);
        assert_eq!(align_to(256, 256), 256);
        assert_eq!(align_to(300, 256), 512);
    }

    #[test]
    fn test_memory_block_creation() {
        let block = MemoryBlock::new(1, 1024, 0, 256, 10);
        assert_eq!(block.id, 1);
        assert_eq!(block.size, 1024);
        assert_eq!(block.offset, 0);
        assert_eq!(block.alignment, 256);
        assert_eq!(block.created_frame, 10);
        assert_eq!(block.state, BufferState::Idle);
    }

    #[test]
    fn test_memory_block_reuse() {
        let mut block = MemoryBlock::new(1, 1024, 0, 256, 10);

        // 初始状态可以复用
        assert!(block.can_reuse(15));

        // 标记为写入中不能复用
        block.mark_writing();
        assert!(!block.can_reuse(15));

        // 标记为待回收，需要等待足够帧数
        block.mark_pending(10);
        assert!(!block.can_reuse(12)); // 不足2帧延迟
        assert!(block.can_reuse(13)); // 超过2帧延迟
    }

    #[test]
    fn test_allocation_stats() {
        let mut stats = AllocationStats::default();

        stats.total_allocations = 10;
        stats.total_bytes_allocated = 1024 * 1024;
        stats.active_allocations = 5;
        stats.active_bytes = 512 * 1024;

        stats.update_peak();
        assert_eq!(stats.peak_bytes, 512 * 1024);

        stats.calculate_fragmentation(2 * 1024 * 1024);
        assert_eq!(stats.fragmentation_ratio, 0.75); // (2MB - 512KB) / 2MB
    }
}
