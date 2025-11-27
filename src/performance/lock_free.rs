use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use std::collections::VecDeque;

/// 无锁计数器
pub struct LockFreeCounter {
    value: AtomicU64,
}

impl LockFreeCounter {
    pub fn new(initial: u64) -> Self {
        Self {
            value: AtomicU64::new(initial),
        }
    }
    
    /// 增加计数
    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }
    
    /// 减少计数
    pub fn decrement(&self) -> u64 {
        self.value.fetch_sub(1, Ordering::SeqCst)
    }
    
    /// 获取当前值
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }
    
    /// 设置值
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::SeqCst);
    }
}

/// 无锁标志
pub struct LockFreeFlag {
    value: AtomicBool,
}

impl LockFreeFlag {
    pub fn new(initial: bool) -> Self {
        Self {
            value: AtomicBool::new(initial),
        }
    }
    
    /// 设置标志
    pub fn set(&self, value: bool) {
        self.value.store(value, Ordering::SeqCst);
    }
    
    /// 获取标志
    pub fn get(&self) -> bool {
        self.value.load(Ordering::SeqCst)
    }
    
    /// 交换标志
    pub fn swap(&self, value: bool) -> bool {
        self.value.swap(value, Ordering::SeqCst)
    }
}

/// 读写锁包装器
pub struct RwLockWrapper<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> RwLockWrapper<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
        }
    }
    
    /// 读取数据
    pub fn read<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.read().unwrap();
        f(&*guard)
    }
    
    /// 写入数据
    pub fn write<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.write().unwrap();
        f(&mut *guard)
    }
    
    /// 尝试读取数据
    pub fn try_read<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        self.inner.read().ok().map(|guard| f(&*guard))
    }
    
    /// 尝试写入数据
    pub fn try_write<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.inner.write().ok().map(|mut guard| f(&mut *guard))
    }
}

impl<T> Clone for RwLockWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

/// 无锁队列 (简化版)
pub struct LockFreeQueue<T> {
    inner: Arc<RwLock<VecDeque<T>>>,
}

impl<T> LockFreeQueue<T> {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(VecDeque::new())),
        }
    }
    
    /// 入队
    pub fn push(&self, value: T) {
        let mut queue = self.inner.write().unwrap();
        queue.push_back(value);
    }
    
    /// 出队
    pub fn pop(&self) -> Option<T> {
        let mut queue = self.inner.write().unwrap();
        queue.pop_front()
    }
    
    /// 获取队列长度
    pub fn len(&self) -> usize {
        let queue = self.inner.read().unwrap();
        queue.len()
    }
    
    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        let queue = self.inner.read().unwrap();
        queue.is_empty()
    }
}

impl<T> Clone for LockFreeQueue<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<T> Default for LockFreeQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_lock_free_counter() {
        let counter = Arc::new(LockFreeCounter::new(0));
        let mut handles = vec![];
        
        // 启动10个线程,每个线程增加100次
        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    counter.increment();
                }
            });
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        // 验证结果
        assert_eq!(counter.get(), 1000);
    }
    
    #[test]
    fn test_lock_free_flag() {
        let flag = LockFreeFlag::new(false);
        
        assert_eq!(flag.get(), false);
        flag.set(true);
        assert_eq!(flag.get(), true);
        
        let old = flag.swap(false);
        assert_eq!(old, true);
        assert_eq!(flag.get(), false);
    }
    
    #[test]
    fn test_rw_lock_wrapper() {
        let wrapper = RwLockWrapper::new(vec![1, 2, 3]);
        
        // 读取数据
        let sum = wrapper.read(|v| v.iter().sum::<i32>());
        assert_eq!(sum, 6);
        
        // 写入数据
        wrapper.write(|v| v.push(4));
        
        // 再次读取
        let sum = wrapper.read(|v| v.iter().sum::<i32>());
        assert_eq!(sum, 10);
    }
    
    #[test]
    fn test_lock_free_queue() {
        let queue = Arc::new(LockFreeQueue::new());
        let mut handles = vec![];
        
        // 启动5个生产者线程
        for i in 0..5 {
            let queue = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    queue.push(i * 10 + j);
                }
            });
            handles.push(handle);
        }
        
        // 等待所有生产者完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        // 验证队列长度
        assert_eq!(queue.len(), 50);
        
        // 消费所有元素
        let mut count = 0;
        while queue.pop().is_some() {
            count += 1;
        }
        assert_eq!(count, 50);
        assert!(queue.is_empty());
    }
}
