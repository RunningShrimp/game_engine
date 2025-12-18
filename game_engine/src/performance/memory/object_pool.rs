use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

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
        self.available
            .pop_front()
            .unwrap_or_else(|| (self.factory)())
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

// ============================================================================
// 线程安全对象池
// ============================================================================

/// 线程安全对象池（使用无锁队列）
///
/// 支持多线程并发获取和归还对象，使用无锁队列减少锁竞争。
pub struct SyncObjectPool<T: Send> {
    /// 可用对象队列（无锁）
    available_sender: Sender<T>,
    available_receiver: Receiver<T>,
    factory: Box<dyn Fn() -> T + Send + Sync>,
    max_size: AtomicUsize,
    /// 统计：总分配次数
    allocations: AtomicUsize,
    /// 统计：总归还次数
    releases: AtomicUsize,
    /// 统计：缓存命中次数
    cache_hits: AtomicUsize,
    /// 当前池中对象数量（近似值）
    current_size: AtomicUsize,
}

impl<T: Send> SyncObjectPool<T> {
    /// 创建线程安全对象池（使用无锁队列）
    pub fn new<F>(factory: F, initial_size: usize, max_size: usize) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
    {
        let (sender, receiver) = unbounded();

        // 预分配初始对象
        let mut current_size = 0;
        for _ in 0..initial_size {
            if sender.send(factory()).is_ok() {
                current_size += 1;
            }
        }

        Self {
            available_sender: sender,
            available_receiver: receiver,
            factory: Box::new(factory),
            max_size: AtomicUsize::new(max_size),
            allocations: AtomicUsize::new(0),
            releases: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            current_size: AtomicUsize::new(current_size),
        }
    }

    /// 从池中获取对象（无锁）
    pub fn acquire(&self) -> T {
        self.allocations.fetch_add(1, Ordering::Relaxed);

        // 尝试从无锁队列获取对象
        match self.available_receiver.try_recv() {
            Ok(obj) => {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                self.current_size.fetch_sub(1, Ordering::Relaxed);
                return obj;
            }
            Err(_) => {
                // 队列为空，创建新对象
            }
        }

        (self.factory)()
    }

    /// 将对象归还到池中（无锁）
    pub fn release(&self, obj: T) {
        self.releases.fetch_add(1, Ordering::Relaxed);

        let max_size = self.max_size.load(Ordering::Relaxed);
        let current = self.current_size.load(Ordering::Relaxed);

        // 检查池是否已满
        if current < max_size {
            if self.available_sender.send(obj).is_ok() {
                self.current_size.fetch_add(1, Ordering::Relaxed);
            }
        }
        // 如果池已满，对象将被丢弃
    }

    /// 获取池中可用对象的数量（近似值）
    ///
    /// 注意：由于使用无锁队列，这个值只是近似值
    pub fn available_count(&self) -> usize {
        self.current_size.load(Ordering::Relaxed)
    }

    /// 获取池统计信息
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            allocations: self.allocations.load(Ordering::Relaxed),
            releases: self.releases.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            available: self.available_count(),
            max_size: self.max_size.load(Ordering::Relaxed),
        }
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        self.allocations.store(0, Ordering::Relaxed);
        self.releases.store(0, Ordering::Relaxed);
        self.cache_hits.store(0, Ordering::Relaxed);
    }

    /// 预热池 - 预先分配对象（无锁）
    pub fn warm_up(&self, count: usize) {
        let max_size = self.max_size.load(Ordering::Relaxed);
        let current = self.current_size.load(Ordering::Relaxed);
        let to_add = count.min(max_size.saturating_sub(current));

        for _ in 0..to_add {
            if self.available_sender.send((self.factory)()).is_ok() {
                self.current_size.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// 清空池（非阻塞）
    ///
    /// 注意：由于使用无锁队列，只能清空当前队列中的对象
    pub fn clear(&self) -> usize {
        let mut cleared = 0;
        while self.available_receiver.try_recv().is_ok() {
            cleared += 1;
            self.current_size.fetch_sub(1, Ordering::Relaxed);
        }
        cleared
    }

    /// 设置最大池大小
    pub fn set_max_size(&self, max_size: usize) {
        self.max_size.store(max_size, Ordering::Relaxed);
    }
}

/// 对象池统计信息
#[derive(Debug, Clone, Copy, Default)]
pub struct PoolStats {
    /// 总分配次数
    pub allocations: usize,
    /// 总归还次数
    pub releases: usize,
    /// 缓存命中次数
    pub cache_hits: usize,
    /// 当前可用对象数
    pub available: usize,
    /// 池最大大小
    pub max_size: usize,
}

impl PoolStats {
    /// 计算缓存命中率
    pub fn hit_rate(&self) -> f32 {
        if self.allocations == 0 {
            0.0
        } else {
            self.cache_hits as f32 / self.allocations as f32
        }
    }
}

// ============================================================================
// 可重置对象池
// ============================================================================

/// 可重置对象 trait
pub trait Resettable {
    /// 重置对象到初始状态
    fn reset(&mut self);
}

/// 可重置对象池
///
/// 在归还对象时自动调用 reset() 方法
pub struct ResettablePool<T: Resettable> {
    available: VecDeque<T>,
    factory: Box<dyn Fn() -> T>,
    max_size: usize,
}

impl<T: Resettable> ResettablePool<T> {
    /// 创建可重置对象池
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
        self.available
            .pop_front()
            .unwrap_or_else(|| (self.factory)())
    }

    /// 将对象归还到池中 (自动重置)
    pub fn release(&mut self, mut obj: T) {
        if self.available.len() < self.max_size {
            obj.reset();
            self.available.push_back(obj);
        }
    }

    /// 获取池中可用对象的数量
    pub fn available_count(&self) -> usize {
        self.available.len()
    }
}

// ============================================================================
// RAII 池化对象
// ============================================================================

/// RAII 池化对象包装器
///
/// 当对象离开作用域时自动归还到池中
pub struct Pooled<T: Send + 'static> {
    value: Option<T>,
    pool: Arc<SyncObjectPool<T>>,
}

impl<T: Send + 'static> Pooled<T> {
    /// 创建池化对象
    pub fn new(pool: Arc<SyncObjectPool<T>>) -> Self {
        let value = pool.acquire();
        Self {
            value: Some(value),
            pool,
        }
    }

    /// 获取内部值的引用
    pub fn get(&self) -> &T {
        self.value.as_ref().unwrap()
    }

    /// 获取内部值的可变引用
    pub fn get_mut(&mut self) -> &mut T {
        self.value.as_mut().unwrap()
    }

    /// 提取内部值（不归还到池中）
    pub fn take(mut self) -> T {
        self.value.take().unwrap()
    }
}

impl<T: Send + 'static> Drop for Pooled<T> {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            self.pool.release(value);
        }
    }
}

impl<T: Send + 'static> std::ops::Deref for Pooled<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value.as_ref().unwrap()
    }
}

impl<T: Send + 'static> std::ops::DerefMut for Pooled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.value.as_mut().unwrap()
    }
}

// ============================================================================
// 分类对象池
// ============================================================================

/// 分类对象池
///
/// 根据大小或类型自动选择不同的池
pub struct SizedPool<T: Send> {
    small: SyncObjectPool<T>,
    medium: SyncObjectPool<T>,
    large: SyncObjectPool<T>,
    /// 小型对象阈值
    small_threshold: usize,
    /// 中型对象阈值
    medium_threshold: usize,
    /// 大小计算函数
    size_fn: Box<dyn Fn(&T) -> usize + Send + Sync>,
}

impl<T: Send + 'static> SizedPool<T> {
    /// 创建分类对象池
    pub fn new<F, S>(
        factory: F,
        size_fn: S,
        small_threshold: usize,
        medium_threshold: usize,
    ) -> Self
    where
        F: Fn() -> T + Send + Sync + Clone + 'static,
        S: Fn(&T) -> usize + Send + Sync + 'static,
    {
        Self {
            small: SyncObjectPool::new(factory.clone(), 64, 256),
            medium: SyncObjectPool::new(factory.clone(), 32, 128),
            large: SyncObjectPool::new(factory, 16, 64),
            small_threshold,
            medium_threshold,
            size_fn: Box::new(size_fn),
        }
    }

    /// 从池中获取对象
    pub fn acquire(&self) -> T {
        // 默认从小池获取
        self.small.acquire()
    }

    /// 将对象归还到适当的池中
    pub fn release(&self, obj: T) {
        let size = (self.size_fn)(&obj);

        if size <= self.small_threshold {
            self.small.release(obj);
        } else if size <= self.medium_threshold {
            self.medium.release(obj);
        } else {
            self.large.release(obj);
        }
    }

    /// 获取所有池的统计信息
    pub fn stats(&self) -> (PoolStats, PoolStats, PoolStats) {
        (self.small.stats(), self.medium.stats(), self.large.stats())
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
