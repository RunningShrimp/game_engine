use std::cell::RefCell;
use std::ptr::NonNull;
use std::alloc::{alloc, dealloc, Layout};

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
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: RefCell::new(Vec::new()),
            current_chunk: RefCell::new(0),
            chunk_size,
        }
    }
    
    /// 分配内存
    pub fn alloc(&self, size: usize, align: usize) -> NonNull<u8> {
        let mut chunks = self.chunks.borrow_mut();
        let mut current_chunk = self.current_chunk.borrow_mut();
        
        // 如果没有块或当前块空间不足,分配新块
        if chunks.is_empty() || !chunks[*current_chunk].can_alloc(size, align) {
            let chunk_size = self.chunk_size.max(size + align);
            let chunk = Chunk::new(chunk_size);
            chunks.push(chunk);
            *current_chunk = chunks.len() - 1;
        }
        
        // 从当前块分配
        chunks[*current_chunk].alloc(size, align)
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
    fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = unsafe { alloc(layout) };
        let ptr = NonNull::new(ptr).expect("Failed to allocate memory");
        
        Self {
            ptr,
            used: 0,
            size,
        }
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
        
        self.used += padding + size;
        
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
pub struct TypedArena<T> {
    arena: Arena,
    _marker: std::marker::PhantomData<T>,
}

impl<T> TypedArena<T> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(4096),
            _marker: std::marker::PhantomData,
        }
    }
    
    /// 分配单个对象
    pub fn alloc(&self, value: T) -> &mut T {
        let ptr = self.arena.alloc(
            std::mem::size_of::<T>(),
            std::mem::align_of::<T>(),
        );
        
        unsafe {
            let ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(ptr, value);
            &mut *ptr
        }
    }
    
    /// 重置Arena
    pub fn reset(&self) {
        self.arena.reset();
    }
}

impl<T> Default for TypedArena<T> {
    fn default() -> Self {
        Self::new()
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
        let _ptr1 = arena.alloc(64, 8);
        let _ptr2 = arena.alloc(128, 8);
        let _ptr3 = arena.alloc(256, 8);
        
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
        let val1 = arena.alloc(42);
        let val2 = arena.alloc(100);
        
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
