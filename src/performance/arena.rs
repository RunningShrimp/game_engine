use std::cell::RefCell;
use std::ptr::NonNull;
use std::alloc::{alloc, dealloc, Layout};
use std::time::Duration;
use thiserror;

#[derive(Debug, thiserror::Error)]
pub enum ArenaError {
    #[error("Memory allocation failed: size={size}, align={align}")]
    AllocationFailed { size: usize, align: usize },

    #[error("Out of memory")]
    OutOfMemory,
}

/// Arena分配器
pub struct Arena {
    /// 内存块
    chunks: RefCell<Vec<Chunk>>,
    /// 当前块索引
    current_chunk: RefCell<usize>,
    /// 块大小
    chunk_size: usize,
}

struct Chunk {
    /// 内存起始地址
    ptr: NonNull<u8>,
    /// 已使用大小
    used: usize,
    /// 总大小
    size: usize,
}

impl Arena {
    /// 带重试机制的内存分配函数
    pub fn alloc_with_retry(layout: Layout, max_retries: usize) -> Result<NonNull<u8>, ArenaError> {
        for attempt in 0..max_retries {
            let ptr = unsafe { alloc(layout) };
            if !ptr.is_null() {
                return NonNull::new(ptr).ok_or(ArenaError::OutOfMemory);
            }
            if attempt == max_retries - 1 {
                return Err(ArenaError::AllocationFailed {
                    size: layout.size(),
                    align: layout.align(),
                });
            }
            // 短暂延迟后重试
            std::thread::sleep(Duration::from_millis(10));
        }
        unreachable!()
    }

    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: RefCell::new(Vec::new()),
            current_chunk: RefCell::new(0),
            chunk_size,
        }
    }
    
    /// 分配内存
    pub fn alloc(&self, size: usize, align: usize) -> Result<NonNull<u8>, ArenaError> {
        let mut chunks = self.chunks.borrow_mut();
        let mut current_chunk = self.current_chunk.borrow_mut();

        // 如果没有块或当前块空间不足,分配新块
        if chunks.is_empty() || !chunks[*current_chunk].can_alloc(size, align) {
            let chunk_size = self.chunk_size.max(size + align);
            let chunk = Chunk::new(chunk_size)?;
            chunks.push(chunk);
            *current_chunk = chunks.len() - 1;
        }

        // 从当前块分配
        Ok(chunks[*current_chunk].alloc(size, align))
    }
    
    /// 重置Arena (保留内存块)
    pub fn reset(&self) {
        let mut chunks = self.chunks.borrow_mut();
        for chunk in chunks.iter_mut() {
            chunk.used = 0;
        }
        *self.current_chunk.borrow_mut() = 0;
    }
    
    /// 获取已分配的总大小
    pub fn allocated_size(&self) -> usize {
        self.chunks.borrow().iter().map(|c| c.used).sum()
    }
    
    /// 获取总容量
    pub fn capacity(&self) -> usize {
        self.chunks.borrow().iter().map(|c| c.size).sum()
    }
}

impl Chunk {
    fn new(size: usize) -> Result<Self, ArenaError> {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = Arena::alloc_with_retry(layout, 3)?;

        Ok(Self {
            ptr,
            used: 0,
            size,
        })
    }
    
    fn can_alloc(&self, size: usize, align: usize) -> bool {
        let current_addr = self.ptr.as_ptr() as usize + self.used;
        let aligned_addr = (current_addr + align - 1) & !(align - 1);
        let padding = aligned_addr - current_addr;
        
        self.used + padding + size <= self.size
    }
    
    fn alloc(&mut self, size: usize, align: usize) -> NonNull<u8> {
        let current_addr = self.ptr.as_ptr() as usize + self.used;
        let aligned_addr = (current_addr + align - 1) & !(align - 1);
        let padding = aligned_addr - current_addr;
        
        // 安全检查：验证对齐是否为2的幂
        debug_assert!(align.is_power_of_two(), "对齐必须是2的幂");
        // 安全检查：验证分配不会溢出
        debug_assert!(
            self.used + padding + size <= self.size,
            "Arena 分配溢出：请求 {} 字节，剩余 {} 字节",
            padding + size,
            self.size - self.used
        );
        // 安全检查：验证对齐后的地址确实对齐
        debug_assert!(
            aligned_addr % align == 0,
            "地址对齐失败：地址 {:#x} 未按 {} 字节对齐",
            aligned_addr,
            align
        );
        
        self.used += padding + size;
        
        // SAFETY: aligned_addr 已通过 can_alloc 验证在有效范围内，
        // 且通过上述 debug_assert 验证了对齐正确性
        unsafe { NonNull::new_unchecked(aligned_addr as *mut u8) }
    }
}

impl Drop for Chunk {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.size, 8).unwrap();
        unsafe { dealloc(self.ptr.as_ptr(), layout) };
    }
}

/// 类型化Arena分配器
///
/// 提供类型安全的Arena分配，适用于需要大量分配同类型对象的场景。
///
/// # 注意
///
/// - `reset()` 不会调用已分配对象的析构函数
/// - 对于实现了 `Drop` 的类型，请使用 `TypedArenaWithDrop`
/// - 适用于 POD 类型或生命周期与 Arena 一致的类型
///
/// # 示例
///
/// ```
/// use game_engine::performance::arena::TypedArena;
///
/// let arena = TypedArena::<i32>::new();
/// let val = arena.alloc(42).unwrap();
/// assert_eq!(*val, 42);
/// ```
pub struct TypedArena<T> {
    arena: Arena,
    /// 已分配对象计数（用于调试）
    alloc_count: std::cell::Cell<usize>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TypedArena<T> {
    /// 创建新的类型化Arena
    ///
    /// 默认块大小为4096字节
    pub fn new() -> Self {
        Self {
            arena: Arena::new(4096),
            alloc_count: std::cell::Cell::new(0),
            _marker: std::marker::PhantomData,
        }
    }
    
    /// 使用指定块大小创建Arena
    pub fn with_chunk_size(chunk_size: usize) -> Self {
        Self {
            arena: Arena::new(chunk_size),
            alloc_count: std::cell::Cell::new(0),
            _marker: std::marker::PhantomData,
        }
    }
    
    /// 分配单个对象
    ///
    /// # 安全性
    ///
    /// 返回的引用生命周期与 Arena 绑定。
    /// 调用 `reset()` 后，之前返回的引用将失效。
    pub fn alloc(&self, value: T) -> Result<&mut T, ArenaError> {
        let ptr = self.arena.alloc(
            std::mem::size_of::<T>(),
            std::mem::align_of::<T>(),
        )?;

        self.alloc_count.set(self.alloc_count.get() + 1);

        // SAFETY: ptr 来自 arena.alloc，保证有效且对齐正确
        unsafe {
            let ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(ptr, value);
            Ok(&mut *ptr)
        }
    }
    
    /// 重置Arena
    ///
    /// # 警告
    ///
    /// 此方法不会调用已分配对象的析构函数。
    /// 对于需要析构的类型，请确保在调用此方法前手动清理。
    pub fn reset(&self) {
        self.alloc_count.set(0);
        self.arena.reset();
    }
    
    /// 获取已分配对象数量
    pub fn len(&self) -> usize {
        self.alloc_count.get()
    }
    
    /// 检查Arena是否为空
    pub fn is_empty(&self) -> bool {
        self.alloc_count.get() == 0
    }
}

impl<T> Default for TypedArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TypedArenaWithDrop - 支持析构的类型化Arena
// ============================================================================

/// 支持析构的类型化Arena分配器
///
/// 与 `TypedArena` 不同，此分配器会在 `Drop` 时调用所有已分配对象的析构函数。
///
/// # 示例
///
/// ```
/// use game_engine::performance::arena::TypedArenaWithDrop;
///
/// let arena = TypedArenaWithDrop::<String>::new();
/// let s = arena.alloc(String::from("Hello")).unwrap();
/// // 当 arena 被 drop 时，String 的析构函数会被调用
/// ```
pub struct TypedArenaWithDrop<T> {
    arena: Arena,
    /// 存储已分配对象的指针，用于析构
    allocated: RefCell<Vec<NonNull<T>>>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TypedArenaWithDrop<T> {
    /// 创建新的支持析构的类型化Arena
    pub fn new() -> Self {
        Self {
            arena: Arena::new(4096),
            allocated: RefCell::new(Vec::new()),
            _marker: std::marker::PhantomData,
        }
    }
    
    /// 分配单个对象
    pub fn alloc(&self, value: T) -> Result<&mut T, ArenaError> {
        let ptr = self.arena.alloc(
            std::mem::size_of::<T>(),
            std::mem::align_of::<T>(),
        )?;

        // SAFETY: ptr 来自 arena.alloc，保证有效且对齐正确
        unsafe {
            let typed_ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(typed_ptr, value);

            // 记录指针用于后续析构
            self.allocated.borrow_mut().push(NonNull::new_unchecked(typed_ptr));

            Ok(&mut *typed_ptr)
        }
    }
    
    /// 获取已分配对象数量
    pub fn len(&self) -> usize {
        self.allocated.borrow().len()
    }
    
    /// 检查Arena是否为空
    pub fn is_empty(&self) -> bool {
        self.allocated.borrow().is_empty()
    }
}

impl<T> Default for TypedArenaWithDrop<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Drop for TypedArenaWithDrop<T> {
    fn drop(&mut self) {
        // 按分配的逆序调用析构函数
        let allocated = self.allocated.borrow();
        for ptr in allocated.iter().rev() {
            // SAFETY: 这些指针都是通过 alloc 分配的有效指针
            unsafe {
                std::ptr::drop_in_place(ptr.as_ptr());
            }
        }
    }
}

/// 内存池
pub struct MemoryPool<T> {
    /// 空闲对象列表
    free_list: RefCell<Vec<Box<T>>>,
    /// 池大小
    capacity: usize,
}

impl<T: Default> MemoryPool<T> {
    pub fn new(capacity: usize) -> Self {
        let mut free_list = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            free_list.push(Box::new(T::default()));
        }
        
        Self {
            free_list: RefCell::new(free_list),
            capacity,
        }
    }
    
    /// 从池中获取对象
    pub fn acquire(&self) -> Option<Box<T>> {
        self.free_list.borrow_mut().pop()
    }
    
    /// 归还对象到池
    pub fn release(&self, obj: Box<T>) {
        let mut free_list = self.free_list.borrow_mut();
        if free_list.len() < self.capacity {
            free_list.push(obj);
        }
    }
    
    /// 获取可用对象数量
    pub fn available(&self) -> usize {
        self.free_list.borrow().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_arena() {
        let arena = Arena::new(1024);

        // 分配一些内存
        let _ptr1 = arena.alloc(64, 8).unwrap();
        let _ptr2 = arena.alloc(128, 8).unwrap();
        let _ptr3 = arena.alloc(256, 8).unwrap();

        // 检查已分配大小
        assert!(arena.allocated_size() > 0);
        assert!(arena.allocated_size() <= arena.capacity());

        // 重置Arena
        arena.reset();
        assert_eq!(arena.allocated_size(), 0);
    }
    
    #[test]
    fn test_typed_arena() {
        let arena = TypedArena::<i32>::new();

        // 分配一些对象
        let val1 = arena.alloc(42).unwrap();
        let val2 = arena.alloc(100).unwrap();

        assert_eq!(*val1, 42);
        assert_eq!(*val2, 100);

        // 修改值
        *val1 = 50;
        assert_eq!(*val1, 50);
    }
    
    #[test]
    fn test_memory_pool() {
        let pool = MemoryPool::<Vec<i32>>::new(10);
        
        // 获取对象
        let mut obj1 = pool.acquire().unwrap();
        let mut obj2 = pool.acquire().unwrap();
        
        assert_eq!(pool.available(), 8);
        
        // 使用对象
        obj1.push(1);
        obj2.push(2);
        
        // 归还对象
        pool.release(obj1);
        pool.release(obj2);
        
        assert_eq!(pool.available(), 10);
    }
}
