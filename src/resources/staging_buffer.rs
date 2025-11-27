//! Staging Buffer 管理模块
//!
//! 用于高效地将 CPU 数据上传到 GPU，通过 Staging Buffer 实现异步上传，
//! 避免主线程阻塞。
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  Staging Buffer Pool                     │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. 分配策略                                              │
//! │     - 小数据 (<64KB): 从共享池分配                        │
//! │     - 大数据 (>64KB): 独立缓冲区                          │
//! │                                                          │
//! │  2. 上传流程                                              │
//! │     - CPU写入 Staging Buffer (MAP_WRITE)                 │
//! │     - GPU复制 Staging -> Target (COPY_SRC -> COPY_DST)   │
//! │                                                          │
//! │  3. 回收机制                                              │
//! │     - 帧结束后回收已完成的 Staging Buffer                  │
//! └─────────────────────────────────────────────────────────┘
//! ```

use std::collections::VecDeque;
use wgpu::util::DeviceExt;

// ============================================================================
// 常量配置
// ============================================================================

/// 小数据阈值 (64KB)
const SMALL_BUFFER_THRESHOLD: u64 = 64 * 1024;

/// 共享 Staging Buffer 大小 (4MB)
const SHARED_STAGING_SIZE: u64 = 4 * 1024 * 1024;

/// 最大保留的空闲缓冲区数量
const MAX_FREE_BUFFERS: usize = 8;

// ============================================================================
// Staging Buffer
// ============================================================================

/// 单个 Staging Buffer
pub struct StagingBuffer {
    /// GPU 缓冲区
    pub buffer: wgpu::Buffer,
    /// 缓冲区大小
    pub size: u64,
    /// 当前写入偏移
    pub offset: u64,
}

impl StagingBuffer {
    /// 创建新的 Staging Buffer
    pub fn new(device: &wgpu::Device, size: u64, label: Option<&str>) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: true,
        });

        Self {
            buffer,
            size,
            offset: 0,
        }
    }

    /// 检查是否有足够空间
    pub fn can_fit(&self, size: u64, alignment: u64) -> bool {
        let aligned_offset = align_to(self.offset, alignment);
        aligned_offset + size <= self.size
    }

    /// 分配空间并写入数据
    /// 
    /// 返回写入的偏移量
    pub fn write(&mut self, data: &[u8], alignment: u64) -> Option<u64> {
        let aligned_offset = align_to(self.offset, alignment);
        let end = aligned_offset + data.len() as u64;

        if end > self.size {
            return None;
        }

        // 写入数据
        {
            let slice = self.buffer.slice(aligned_offset..end);
            let mut view = slice.get_mapped_range_mut();
            view.copy_from_slice(data);
        }

        self.offset = end;
        Some(aligned_offset)
    }

    /// 重置偏移（用于复用）
    pub fn reset(&mut self) {
        self.offset = 0;
    }

    /// 解除映射以供 GPU 使用
    pub fn unmap(&self) {
        self.buffer.unmap();
    }

    /// 剩余可用空间
    pub fn remaining(&self) -> u64 {
        self.size - self.offset
    }
}

// ============================================================================
// Staging Buffer 池
// ============================================================================

/// Staging Buffer 池 - 管理多个 Staging Buffer
pub struct StagingBufferPool {
    /// 共享 Staging Buffer（用于小数据）
    shared_buffer: Option<StagingBuffer>,
    /// 独立缓冲区（用于大数据）
    dedicated_buffers: Vec<StagingBuffer>,
    /// 空闲缓冲区池
    free_buffers: VecDeque<StagingBuffer>,
    /// 统计信息
    stats: PoolStats,
}

/// 池统计信息
#[derive(Default, Clone, Copy, Debug)]
pub struct PoolStats {
    /// 总分配次数
    pub total_allocations: u64,
    /// 总上传字节数
    pub total_bytes_uploaded: u64,
    /// 当前活跃缓冲区数
    pub active_buffers: u32,
    /// 复用次数
    pub reuse_count: u64,
}

impl Default for StagingBufferPool {
    fn default() -> Self {
        Self::new()
    }
}

impl StagingBufferPool {
    pub fn new() -> Self {
        Self {
            shared_buffer: None,
            dedicated_buffers: Vec::new(),
            free_buffers: VecDeque::new(),
            stats: PoolStats::default(),
        }
    }

    /// 初始化共享缓冲区
    pub fn initialize(&mut self, device: &wgpu::Device) {
        if self.shared_buffer.is_none() {
            self.shared_buffer = Some(StagingBuffer::new(
                device,
                SHARED_STAGING_SIZE,
                Some("Shared Staging Buffer"),
            ));
        }
    }

    /// 分配空间用于数据上传
    /// 
    /// 返回 (buffer_index, offset)
    /// - buffer_index: 0 表示共享缓冲区，>0 表示独立缓冲区索引+1
    pub fn allocate(
        &mut self,
        device: &wgpu::Device,
        size: u64,
        alignment: u64,
    ) -> (usize, u64) {
        self.stats.total_allocations += 1;
        self.stats.total_bytes_uploaded += size;

        // 小数据尝试使用共享缓冲区
        if size < SMALL_BUFFER_THRESHOLD {
            if let Some(ref mut shared) = self.shared_buffer {
                if let Some(offset) = shared.write(&[], alignment) {
                    // 检查是否能容纳（这里只是检查，实际写入在外部）
                    if shared.can_fit(size, alignment) {
                        return (0, align_to(shared.offset, alignment));
                    }
                }
            }
        }

        // 尝试复用空闲缓冲区
        if let Some(mut buffer) = self.free_buffers.pop_front() {
            if buffer.size >= size {
                buffer.reset();
                let index = self.dedicated_buffers.len() + 1;
                self.dedicated_buffers.push(buffer);
                self.stats.reuse_count += 1;
                return (index, 0);
            } else {
                // 放回队列
                self.free_buffers.push_front(buffer);
            }
        }

        // 创建新的独立缓冲区
        let buffer_size = size.max(SMALL_BUFFER_THRESHOLD).next_power_of_two();
        let buffer = StagingBuffer::new(
            device,
            buffer_size,
            Some(&format!("Dedicated Staging Buffer {}", self.dedicated_buffers.len())),
        );
        
        let index = self.dedicated_buffers.len() + 1;
        self.dedicated_buffers.push(buffer);
        self.stats.active_buffers += 1;
        
        (index, 0)
    }

    /// 获取缓冲区引用
    pub fn get_buffer(&self, index: usize) -> Option<&StagingBuffer> {
        if index == 0 {
            self.shared_buffer.as_ref()
        } else {
            self.dedicated_buffers.get(index - 1)
        }
    }

    /// 获取可变缓冲区引用
    pub fn get_buffer_mut(&mut self, index: usize) -> Option<&mut StagingBuffer> {
        if index == 0 {
            self.shared_buffer.as_mut()
        } else {
            self.dedicated_buffers.get_mut(index - 1)
        }
    }

    /// 解除所有缓冲区的映射
    pub fn unmap_all(&self) {
        if let Some(ref shared) = self.shared_buffer {
            shared.unmap();
        }
        for buffer in &self.dedicated_buffers {
            buffer.unmap();
        }
    }

    /// 帧结束时回收缓冲区
    pub fn end_frame(&mut self, device: &wgpu::Device) {
        // 重映射共享缓冲区
        if let Some(ref mut shared) = self.shared_buffer {
            shared.reset();
            // 重新创建以获得映射
            *shared = StagingBuffer::new(device, SHARED_STAGING_SIZE, Some("Shared Staging Buffer"));
        }

        // 回收独立缓冲区
        for buffer in self.dedicated_buffers.drain(..) {
            if self.free_buffers.len() < MAX_FREE_BUFFERS {
                self.free_buffers.push_back(buffer);
            }
            // 超出限制的直接丢弃
        }

        self.stats.active_buffers = 0;
    }

    /// 获取统计信息
    pub fn stats(&self) -> PoolStats {
        self.stats
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = PoolStats::default();
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 对齐到指定边界
#[inline]
fn align_to(value: u64, alignment: u64) -> u64 {
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
    fn test_align_to() {
        assert_eq!(align_to(0, 16), 0);
        assert_eq!(align_to(1, 16), 16);
        assert_eq!(align_to(15, 16), 16);
        assert_eq!(align_to(16, 16), 16);
        assert_eq!(align_to(17, 16), 32);
        assert_eq!(align_to(100, 256), 256);
    }

    #[test]
    fn test_pool_stats_default() {
        let stats = PoolStats::default();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.total_bytes_uploaded, 0);
    }
}
