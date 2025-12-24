//! Bump分配器
//!
//! 实现高性能的Bump分配器，用于快速分配和批量释放内存。
//!
//! # 设计原理
//!
//! Bump分配器是一种线性分配器，通过移动指针来分配内存，非常快速。
//! 所有分配的内存可以在O(1)时间内一次性释放。
//!
//! # 使用场景
//!
//! - 帧内临时对象分配
//! - 批量分配相同生命周期对象
//! - 减少内存分配开销

use std::alloc::{Layout, alloc, dealloc};
use std::ptr::NonNull;
use thiserror::Error;

/// Bump分配器错误类型
#[derive(Error, Debug)]
pub enum BumpError {
    /// 内存分配失败
    #[error("Memory allocation failed: size={size}, align={align}")]
    AllocationFailed { size: usize, align: usize },
    /// 内存不足
    #[error("Out of memory: requested={requested}, available={available}")]
    OutOfMemory { requested: usize, available: usize },
}

/// Bump分配器
///
/// 线性分配器，通过移动指针快速分配内存。
/// 所有分配的内存可以通过`reset()`一次性释放。
///
/// # 性能特点
///
/// - 分配：O(1)时间复杂度
/// - 释放：O(1)时间复杂度（批量释放）
/// - 内存对齐：自动处理对齐
///
/// # 示例
///
/// ```rust
/// use game_engine_performance::memory::bump::BumpAllocator;
///
/// // 创建Bump分配器（初始大小4KB）
/// let mut bump = BumpAllocator::new(4096)?;
///
/// // 分配内存
/// let ptr1 = bump.alloc(64, 8)?;
/// let ptr2 = bump.alloc(128, 16)?;
///
/// // 重置（释放所有分配的内存）
/// bump.reset();
/// # Ok::<(), game_engine_performance::memory::bump::BumpError>(())
/// ```
pub struct BumpAllocator {
    /// 内存块起始地址
    start: NonNull<u8>,
    /// 当前分配指针
    current: NonNull<u8>,
    /// 内存块结束地址
    end: NonNull<u8>,
    /// 块大小
    size: usize,
}

unsafe impl Send for BumpAllocator {}
unsafe impl Sync for BumpAllocator {}

impl BumpAllocator {
    /// 创建新的Bump分配器
    ///
    /// # 参数
    ///
    /// * `size` - 分配器大小（字节）
    ///
    /// # 返回
    ///
    /// 返回初始化的Bump分配器，如果分配失败返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine_performance::memory::bump::BumpAllocator;
    ///
    /// let bump = BumpAllocator::new(4096)?;
    /// # Ok::<(), game_engine_performance::memory::bump::BumpError>(())
    /// ```
    pub fn new(size: usize) -> Result<Self, BumpError> {
        if size == 0 {
            return Err(BumpError::AllocationFailed { size: 0, align: 1 });
        }

        let layout = Layout::from_size_align(size, 1)
            .map_err(|_| BumpError::AllocationFailed { size, align: 1 })?;

        let ptr = unsafe { alloc(layout) };
        let start = NonNull::new(ptr).ok_or(BumpError::AllocationFailed { size, align: 1 })?;

        let end = unsafe { NonNull::new_unchecked(start.as_ptr().add(size)) };

        Ok(Self {
            start,
            current: start,
            end,
            size,
        })
    }

    /// 分配内存
    ///
    /// # 参数
    ///
    /// * `size` - 要分配的大小（字节）
    /// * `align` - 对齐要求（必须是2的幂）
    ///
    /// # 返回
    ///
    /// 返回分配的内存指针，如果内存不足返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine_performance::memory::bump::BumpAllocator;
    ///
    /// let mut bump = BumpAllocator::new(4096)?;
    /// let ptr = bump.alloc(64, 8)?;
    /// # Ok::<(), game_engine_performance::memory::bump::BumpError>(())
    /// ```
    pub fn alloc(&mut self, size: usize, align: usize) -> Result<NonNull<u8>, BumpError> {
        if size == 0 {
            return Err(BumpError::AllocationFailed { size: 0, align });
        }

        // 对齐当前指针
        let current_addr = self.current.as_ptr() as usize;
        let aligned_addr = (current_addr + align - 1) & !(align - 1);
        let aligned_ptr = unsafe { NonNull::new_unchecked(aligned_addr as *mut u8) };

        // 检查是否有足够空间
        let new_current = unsafe { aligned_ptr.as_ptr().add(size) };

        if new_current > self.end.as_ptr() {
            let available =
                unsafe { self.end.as_ptr().offset_from(self.current.as_ptr()) } as usize;
            return Err(BumpError::OutOfMemory {
                requested: size,
                available,
            });
        }

        // 更新当前指针
        self.current = unsafe { NonNull::new_unchecked(new_current) };

        Ok(aligned_ptr)
    }

    /// 重置分配器
    ///
    /// 将所有分配的内存标记为可用，O(1)时间复杂度。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine_performance::memory::bump::BumpAllocator;
    ///
    /// let mut bump = BumpAllocator::new(4096)?;
    /// bump.alloc(64, 8)?;
    /// bump.reset(); // 释放所有分配的内存
    /// # Ok::<(), game_engine_performance::memory::bump::BumpError>(())
    /// ```
    pub fn reset(&mut self) {
        self.current = self.start;
    }

    /// 获取已使用的内存大小
    ///
    /// # 返回
    ///
    /// 返回已分配的内存大小（字节）。
    pub fn used_size(&self) -> usize {
        unsafe {
            let offset = self.current.as_ptr().offset_from(self.start.as_ptr());
            if offset < 0 { 0 } else { offset as usize }
        }
    }

    /// 获取总容量
    ///
    /// # 返回
    ///
    /// 返回分配器的总容量（字节）。
    pub fn capacity(&self) -> usize {
        self.size
    }

    /// 获取可用内存大小
    ///
    /// # 返回
    ///
    /// 返回剩余可用内存大小（字节）。
    pub fn available_size(&self) -> usize {
        self.capacity() - self.used_size()
    }

    /// 检查是否有足够空间
    ///
    /// # 参数
    ///
    /// * `size` - 需要的大小
    /// * `align` - 对齐要求
    ///
    /// # 返回
    ///
    /// 如果有足够空间返回`true`，否则返回`false`。
    pub fn can_alloc(&self, size: usize, align: usize) -> bool {
        let current_addr = self.current.as_ptr() as usize;
        let aligned_addr = (current_addr + align - 1) & !(align - 1);
        let aligned_ptr = aligned_addr as *mut u8;
        let new_current = unsafe { aligned_ptr.add(size) };
        new_current <= self.end.as_ptr()
    }
}

impl Drop for BumpAllocator {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, 1).unwrap();
        unsafe {
            dealloc(self.start.as_ptr(), layout);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_allocator_creation() {
        let bump = BumpAllocator::new(4096);
        assert!(bump.is_ok());
    }

    #[test]
    fn test_bump_allocator_allocation() {
        let mut bump = BumpAllocator::new(4096).unwrap();
        let ptr1 = bump.alloc(64, 8);
        assert!(ptr1.is_ok());

        let ptr2 = bump.alloc(128, 16);
        assert!(ptr2.is_ok());

        assert!(bump.used_size() > 0);
    }

    #[test]
    fn test_bump_allocator_reset() {
        let mut bump = BumpAllocator::new(4096).unwrap();
        bump.alloc(64, 8).unwrap();
        bump.alloc(128, 16).unwrap();

        assert!(bump.used_size() > 0);
        bump.reset();
        assert_eq!(bump.used_size(), 0);
    }

    #[test]
    fn test_bump_allocator_out_of_memory() {
        let mut bump = BumpAllocator::new(100).unwrap();
        let result = bump.alloc(200, 8);
        assert!(result.is_err());
    }

    #[test]
    fn test_bump_allocator_alignment() {
        let mut bump = BumpAllocator::new(4096).unwrap();

        // 分配一个未对齐的大小
        let ptr1 = bump.alloc(1, 1).unwrap();
        let addr1 = ptr1.as_ptr() as usize;

        // 分配一个需要对齐的大小
        let ptr2 = bump.alloc(64, 16).unwrap();
        let addr2 = ptr2.as_ptr() as usize;

        // 验证第一个分配的地址是有效的（对齐要求为1，所以任意地址都可以）
        assert!(addr1 > 0); // Verify the address is valid

        // 验证对齐
        assert_eq!(addr2 % 16, 0);
    }
}
