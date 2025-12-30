//  Arena分配器 - 批量操作优化
//
//  为临时对象和帧生命周期对象提供高性能的Arena分配器。
//  Arena分配器一次性分配大块内存，然后从其中分配小块对象，
//  所有对象可以一次性释放，极大地减少了分配/释放开销。
//
//  ## 性能优化策略
//
//  1. **批量分配** (Bump Allocation)
//     - 指针递增分配，O(1)时间复杂度
//     - 无碎片化
//     - 批量释放
//
//  2. **内存对齐** (Memory Alignment)
//     - 自动对齐到指定边界
//     - 支持SIMD对齐要求
//
//  3. **多Arena支持** (Multiple Arenas)
//     - 每帧一个Arena
//     - 临时计算Arena
//     - 持久化Arena
//
//  ## 预期收益
//
//  - 分配速度提升 10-50倍
//  - 内存碎片减少 80-90%
//  - 缓存命中率提升 20-30%

use std::alloc::{self, Layout};
use std::ptr::NonNull;

/// Arena分配器错误
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArenaError {
    /// 内存不足
    OutOfMemory,
    /// 对齐失败
    AlignmentFailed,
    /// 大小超过限制
    SizeTooLarge,
}

/// Arena分配器
///
/// 使用bump指针分配策略，提供极快的分配性能。
/// 所有分配的生命周期绑定到Arena本身，一次性释放所有对象。
pub struct ArenaAllocator {
    /// 起始指针
    start: NonNull<u8>,
    /// 当前位置（bump指针）
    current: NonNull<u8>,
    /// 结束指针
    end: NonNull<u8>,
    /// 总容量（字节）
    capacity: usize,
    /// 已使用字节
    used: usize,
    /// 对齐要求
    alignment: usize,
}

unsafe impl Send for ArenaAllocator {}
unsafe impl Sync for ArenaAllocator {}

impl ArenaAllocator {
    /// 创建新的Arena分配器
    ///
    /// # Arguments
    /// * `capacity` - Arena容量（字节）
    /// * `alignment` - 对齐要求（默认8）
    pub fn new(capacity: usize, alignment: usize) -> Result<Self, ArenaError> {
        if capacity == 0 {
            return Err(ArenaError::SizeTooLarge);
        }

        // 确保对齐是2的幂
        let alignment = alignment.next_power_of_two();

        // 计算实际布局
        let layout = Layout::from_size_align(capacity, alignment)
            .map_err(|_| ArenaError::AlignmentFailed)?;

        // 分配内存
        let start = unsafe {
            let ptr = alloc::alloc(layout);
            if ptr.is_null() {
                return Err(ArenaError::OutOfMemory);
            }
            NonNull::new_unchecked(ptr)
        };

        // SAFETY: 计算end指针，添加溢出检查
        // start.as_ptr()保证有效（刚分配的内存）
        // capacity在之前已经验证过>0
        // 使用checked_add防止指针算术溢出
        let end_ptr = (start.as_ptr() as usize)
            .checked_add(capacity)
            .ok_or(ArenaError::SizeTooLarge)?;
        let end = unsafe { NonNull::new_unchecked(end_ptr as *mut u8) };

        Ok(Self {
            start,
            current: start,
            end,
            capacity,
            used: 0,
            alignment,
        })
    }

    /// 使用默认对齐创建（8字节）
    pub fn with_capacity(capacity: usize) -> Result<Self, ArenaError> {
        Self::new(capacity, 8)
    }

    /// 分配内存
    ///
    /// # Arguments
    /// * `size` - 分配大小（字节）
    /// * `align` - 对齐要求
    ///
    /// # Returns
    /// 返回分配的内存指针，如果空间不足返回None
    pub fn allocate(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        if size == 0 {
            return NonNull::new(self.current.as_ptr());
        }

        // 计算对齐后的偏移
        let current_addr = self.current.as_ptr() as usize;
        let alignment = align.next_power_of_two();
        let aligned_offset = (current_addr + alignment - 1) & !(alignment - 1);
        let aligned_addr = aligned_offset - current_addr;

        // 检查是否有足够空间
        let total_size = aligned_addr + size;
        let new_current = unsafe { self.current.as_ptr().add(total_size) };

        if new_current > self.end.as_ptr() {
            return None; // 空间不足
        }

        // 更新bump指针
        let aligned_ptr =
            unsafe { NonNull::new_unchecked(self.current.as_ptr().add(aligned_addr)) };
        self.current = unsafe { NonNull::new_unchecked(new_current) };
        self.used += total_size;

        Some(aligned_ptr)
    }

    /// 分配并初始化对象
    ///
    /// # Arguments
    /// * `value` - 要存储的值
    ///
    /// # Returns
    /// 返回对象的引用，如果空间不足返回None
    pub fn allocate_obj<T>(&mut self, value: T) -> Option<&mut T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();

        let ptr = self.allocate(size, align)?;

        unsafe {
            ptr.as_ptr().cast::<T>().write(value);
            Some(&mut *ptr.as_ptr().cast())
        }
    }

    /// 分配数组
    ///
    /// # Arguments
    /// * `count` - 数组元素数量
    ///
    /// # Returns
    /// 返回切片的引用，如果空间不足返回None
    ///
    /// # Safety
    ///
    /// 此函数仅对满足以下条件的类型T是安全的：
    /// - T实现了Default
    /// - 使用Default::default()初始化每个元素
    ///
    /// 对于非Pod类型（如String, Vec），必须使用此方法而非原始内存操作
    pub fn allocate_array<T: Default>(&mut self, count: usize) -> Option<&mut [T]> {
        if count == 0 {
            return Some(unsafe {
                std::slice::from_raw_parts_mut(self.current.as_ptr().cast(), 0)
            });
        }

        let size = std::mem::size_of::<T>().checked_mul(count)?;
        let align = std::mem::align_of::<T>();

        let ptr = self.allocate(size, align)?;

        unsafe {
            let slice = std::slice::from_raw_parts_mut(ptr.as_ptr().cast(), count);
            // 使用Default::default()进行类型安全的初始化
            // 这对任何实现Default的类型都是安全的，包括String, Vec等
            for elem in slice.iter_mut() {
                std::ptr::write(elem, T::default());
            }
            Some(slice)
        }
    }

    /// 重置Arena（释放所有分配）
    ///
    /// 注意：这会使所有之前分配的指针失效
    pub fn reset(&mut self) {
        self.current = self.start;
        self.used = 0;
    }

    /// 获取剩余容量
    pub fn remaining(&self) -> usize {
        self.end.as_ptr() as usize - self.current.as_ptr() as usize
    }

    /// 获取已使用容量
    pub fn used(&self) -> usize {
        self.used
    }

    /// 获取总容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 获取使用率
    pub fn utilization(&self) -> f32 {
        self.used as f32 / self.capacity as f32
    }

    /// 是否需要扩容
    pub fn needs_grow(&self, threshold: f32) -> bool {
        self.utilization() > threshold
    }
}

impl Drop for ArenaAllocator {
    fn drop(&mut self) {
        // 释放内存
        unsafe {
            alloc::dealloc(
                self.start.as_ptr(),
                Layout::from_size_align_unchecked(self.capacity, self.alignment),
            );
        }
    }
}

/// 多Arena管理器
///
/// 管理多个Arena，每个Arena可以有不同的生命周期。
/// 适用于帧分配、临时计算等场景。
pub struct ArenaManager {
    /// 帧Arena（每帧重置）
    frame_arena: Option<ArenaAllocator>,
    /// 临时Arena（短生命周期）
    temp_arena: Option<ArenaAllocator>,
    /// 持久Arena（手动管理）
    persistent_arena: Option<ArenaAllocator>,
    /// 帧Arena容量
    frame_capacity: usize,
    /// 临时Arena容量
    temp_capacity: usize,
    /// 持久Arena容量
    persistent_capacity: usize,
}

impl ArenaManager {
    /// 创建新的Arena管理器
    pub fn new(
        frame_capacity: usize,
        temp_capacity: usize,
        persistent_capacity: usize,
    ) -> Result<Self, ArenaError> {
        Ok(Self {
            frame_arena: Some(ArenaAllocator::with_capacity(frame_capacity)?),
            temp_arena: Some(ArenaAllocator::with_capacity(temp_capacity)?),
            persistent_arena: Some(ArenaAllocator::with_capacity(persistent_capacity)?),
            frame_capacity,
            temp_capacity,
            persistent_capacity,
        })
    }

    /// 使用默认容量创建
    ///
    /// - 帧Arena: 10MB
    /// - 临时Arena: 5MB
    /// - 持久Arena: 20MB
    pub fn default_capacity() -> Result<Self, ArenaError> {
        Self::new(10 * 1024 * 1024, 5 * 1024 * 1024, 20 * 1024 * 1024)
    }

    /// 获取帧Arena
    pub fn frame_arena(&mut self) -> Option<&mut ArenaAllocator> {
        self.frame_arena.as_mut()
    }

    /// 获取临时Arena
    pub fn temp_arena(&mut self) -> Option<&mut ArenaAllocator> {
        self.temp_arena.as_mut()
    }

    /// 获取持久Arena
    pub fn persistent_arena(&mut self) -> Option<&mut ArenaAllocator> {
        self.persistent_arena.as_mut()
    }

    /// 重置帧Arena（每帧调用）
    pub fn reset_frame(&mut self) {
        if let Some(arena) = &mut self.frame_arena {
            arena.reset();
        }
    }

    /// 重置临时Arena
    pub fn reset_temp(&mut self) {
        if let Some(arena) = &mut self.temp_arena {
            arena.reset();
        }
    }

    /// 获取统计信息
    pub fn stats(&self) -> ArenaManagerStats {
        ArenaManagerStats {
            frame_used: self.frame_arena.as_ref().map(|a| a.used()).unwrap_or(0),
            frame_remaining: self.frame_arena.as_ref().map(|a| a.remaining()).unwrap_or(0),
            frame_utilization: self.frame_arena.as_ref().map(|a| a.utilization()).unwrap_or(0.0),
            temp_used: self.temp_arena.as_ref().map(|a| a.used()).unwrap_or(0),
            temp_remaining: self.temp_arena.as_ref().map(|a| a.remaining()).unwrap_or(0),
            temp_utilization: self.temp_arena.as_ref().map(|a| a.utilization()).unwrap_or(0.0),
            persistent_used: self.persistent_arena.as_ref().map(|a| a.used()).unwrap_or(0),
            persistent_remaining: self
                .persistent_arena
                .as_ref()
                .map(|a| a.remaining())
                .unwrap_or(0),
            persistent_utilization: self
                .persistent_arena
                .as_ref()
                .map(|a| a.utilization())
                .unwrap_or(0.0),
        }
    }
}

/// Arena管理器统计信息
#[derive(Debug, Clone)]
pub struct ArenaManagerStats {
    pub frame_used: usize,
    pub frame_remaining: usize,
    pub frame_utilization: f32,
    pub temp_used: usize,
    pub temp_remaining: usize,
    pub temp_utilization: f32,
    pub persistent_used: usize,
    pub persistent_remaining: usize,
    pub persistent_utilization: f32,
}

impl ArenaManagerStats {
    /// 打印统计信息
    pub fn print(&self) {
        tracing::info!(target: "memory", "=== Arena Allocator Stats ===");
        tracing::info!(target: "memory",
            "Frame Arena: {:.2}% utilized ({} / {} bytes)",
            self.frame_utilization * 100.0,
            self.frame_used,
            self.frame_used + self.frame_remaining
        );
        tracing::info!(target: "memory",
            "Temp Arena: {:.2}% utilized ({} / {} bytes)",
            self.temp_utilization * 100.0,
            self.temp_used,
            self.temp_used + self.temp_remaining
        );
        tracing::info!(target: "memory",
            "Persistent Arena: {:.2}% utilized ({} / {} bytes)",
            self.persistent_utilization * 100.0,
            self.persistent_used,
            self.persistent_used + self.persistent_remaining
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_create() {
        let arena = ArenaAllocator::with_capacity(1024).expect("Test: operation should succeed");
        assert_eq!(arena.capacity(), 1024);
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.remaining(), 1024);
    }

    #[test]
    fn test_arena_allocate() {
        let mut arena =
            ArenaAllocator::with_capacity(1024).expect("Test: operation should succeed");

        let ptr1 = arena.allocate(100, 8);
        assert!(ptr1.is_some());
        assert_eq!(arena.used(), 100);

        let ptr2 = arena.allocate(200, 8);
        assert!(ptr2.is_some());
        assert_eq!(arena.used(), 300);
    }

    #[test]
    fn test_arena_allocate_obj() {
        let mut arena =
            ArenaAllocator::with_capacity(1024).expect("Test: operation should succeed");

        let obj = arena.allocate_obj(42u32);
        assert!(obj.is_some());
        assert_eq!(*obj.expect("Test: operation should succeed"), 42);
    }

    #[test]
    fn test_arena_reset() {
        let mut arena =
            ArenaAllocator::with_capacity(1024).expect("Test: operation should succeed");

        arena.allocate(500, 8).expect("Test: operation should succeed");
        assert_eq!(arena.used(), 500);

        arena.reset();
        assert_eq!(arena.used(), 0);
        assert_eq!(arena.remaining(), 1024);
    }

    #[test]
    fn test_arena_out_of_memory() {
        let mut arena = ArenaAllocator::with_capacity(100).expect("Test: operation should succeed");

        let ptr1 = arena.allocate(80, 8);
        assert!(ptr1.is_some());

        let ptr2 = arena.allocate(50, 8);
        assert!(ptr2.is_none()); // 空间不足
    }

    #[test]
    fn test_arena_manager() {
        let mut manager = ArenaManager::default_capacity().expect("Test: operation should succeed");

        let stats = manager.stats();
        assert_eq!(stats.frame_used, 0);
        assert_eq!(stats.temp_used, 0);
        assert_eq!(stats.persistent_used, 0);

        // 使用帧Arena
        if let Some(frame) = manager.frame_arena() {
            frame.allocate(1024, 8).expect("Test: operation should succeed");
        }

        let stats = manager.stats();
        assert_eq!(stats.frame_used, 1024);

        // 重置帧Arena
        manager.reset_frame();
        let stats = manager.stats();
        assert_eq!(stats.frame_used, 0);
    }
}
