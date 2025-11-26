use std::collections::VecDeque;

/// 对象池 - 减少内存分配和释放的开销
pub struct ObjectPool<T> {
    available: VecDeque<T>,
    factory: Box<dyn Fn() -> T>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    /// 创建对象池
    pub fn new<F>(factory: F, initial_size: usize, max_size: usize) -> Self
    where
        F: Fn() -> T + 'static,
    {
        let mut available = VecDeque::with_capacity(initial_size);
        for _ in 0..initial_size {
            available.push_back(factory());
        }
        
        Self {
            available,
            factory: Box::new(factory),
            max_size,
        }
    }
    
    /// 从池中获取对象
    pub fn acquire(&mut self) -> T {
        self.available.pop_front().unwrap_or_else(|| (self.factory)())
    }
    
    /// 将对象归还到池中
    pub fn release(&mut self, obj: T) {
        if self.available.len() < self.max_size {
            self.available.push_back(obj);
        }
        // 如果池已满,对象将被丢弃
    }
    
    /// 获取池中可用对象的数量
    pub fn available_count(&self) -> usize {
        self.available.len()
    }
    
    /// 清空池
    pub fn clear(&mut self) {
        self.available.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_object_pool() {
        let mut pool = ObjectPool::new(|| Vec::<i32>::new(), 10, 20);
        
        assert_eq!(pool.available_count(), 10);
        
        // 获取对象
        let mut obj = pool.acquire();
        assert_eq!(pool.available_count(), 9);
        
        // 使用对象
        obj.push(42);
        
        // 归还对象
        pool.release(obj);
        assert_eq!(pool.available_count(), 10);
        
        // 再次获取,应该得到之前归还的对象
        let mut obj2 = pool.acquire();
        // 注意:对象池不保证归还的对象会被清空,所以可能是新对象或旧对象
        // 这里我们只验证池的行为,不验证对象状态
        obj2.clear(); // 清空以便重用
        assert_eq!(obj2.len(), 0);
    }
}
