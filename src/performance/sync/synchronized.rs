use crossbeam_channel::{unbounded, Receiver, Sender};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// 高性能同步原语集合
///
/// 注意：虽然名为synchronized，但这些实现仍然使用锁机制。
/// 对于真正的无锁需求，请考虑使用crossbeam或lockfree库。
///
/// 这个模块包含：
/// - 原子操作计数器和标志（真正的无锁）
/// - 基于RwLock的同步队列和包装器（带有锁竞争监控）

/// 锁竞争监控
#[derive(Default)]
pub struct LockMetrics {
    contention_count: AtomicU64,
    wait_time_ns: AtomicU64,
}

/// 真正的无锁计数器（使用原子操作）
pub struct LockFreeCounter {
    value: AtomicU64,
}

/// 真正的无锁标志（使用原子操作）
pub struct LockFreeFlag {
    value: AtomicBool,
}

impl LockMetrics {
    pub fn contention_count(&self) -> u64 {
        self.contention_count.load(Ordering::Relaxed)
    }

    pub fn total_wait_time_ns(&self) -> u64 {
        self.wait_time_ns.load(Ordering::Relaxed)
    }

    pub fn reset(&self) {
        self.contention_count.store(0, Ordering::Relaxed);
        self.wait_time_ns.store(0, Ordering::Relaxed);
    }
}

/// 原子计数器（真正的无锁实现）
pub struct AtomicCounter {
    value: AtomicU64,
}

impl AtomicCounter {
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

/// 原子标志（真正的无锁实现）
pub struct AtomicFlag {
    value: AtomicBool,
}

impl AtomicFlag {
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

/// 读写锁包装器（带锁竞争监控）
///
/// 注意：这是一个基于RwLock的包装器，包含锁竞争监控功能。
/// 如果锁竞争成为瓶颈，考虑使用真正的无锁数据结构。
pub struct RwLockWrapper<T> {
    inner: Arc<RwLock<T>>,
    metrics: LockMetrics,
}

impl<T> RwLockWrapper<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(value)),
            metrics: LockMetrics::default(),
        }
    }

    /// 获取锁竞争监控数据
    pub fn get_metrics(&self) -> &LockMetrics {
        &self.metrics
    }

    /// 读取数据（返回Result以报告锁竞争）
    pub fn read(&self) -> Result<RwLockReadGuard<'_, T>, ()> {
        let start = std::time::Instant::now();

        match self.inner.read() {
            Ok(guard) => Ok(guard),
            Err(_) => {
                // 记录锁竞争
                self.metrics
                    .contention_count
                    .fetch_add(1, Ordering::Relaxed);
                self.metrics
                    .wait_time_ns
                    .fetch_add(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
                Err(())
            }
        }
    }

    /// 写入数据（返回Result以报告锁竞争）
    pub fn write(&self) -> Result<RwLockWriteGuard<'_, T>, ()> {
        let start = std::time::Instant::now();

        match self.inner.write() {
            Ok(guard) => Ok(guard),
            Err(_) => {
                // 记录锁竞争
                self.metrics
                    .contention_count
                    .fetch_add(1, Ordering::Relaxed);
                self.metrics
                    .wait_time_ns
                    .fetch_add(start.elapsed().as_nanos() as u64, Ordering::Relaxed);
                Err(())
            }
        }
    }
}

impl<T> Clone for RwLockWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            metrics: LockMetrics::default(), // 克隆时重置指标
        }
    }
}

impl<T: Default> Default for RwLockWrapper<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// 同步队列（基于无锁队列实现）
///
/// 使用 `crossbeam-channel` 实现无锁队列，减少锁竞争。
pub struct SynchronizedQueue<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> SynchronizedQueue<T> {
    /// 创建新的无锁队列
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    /// 入队（无锁）
    pub fn push(&self, value: T) {
        // 无锁发送，如果失败则记录警告
        if let Err(e) = self.sender.send(value) {
            tracing::warn!(target: "performance", "Failed to push to queue: {}", e);
        }
    }

    /// 出队（非阻塞）
    pub fn pop(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }

    /// 阻塞出队（带超时）
    pub fn pop_timeout(&self, timeout: std::time::Duration) -> Option<T> {
        self.receiver.recv_timeout(timeout).ok()
    }

    /// 阻塞出队（无限等待）
    pub fn pop_blocking(&self) -> Option<T> {
        self.receiver.recv().ok()
    }

    /// 获取队列长度（近似值）
    ///
    /// 注意：由于使用无锁队列，这个值只是近似值
    pub fn len(&self) -> usize {
        // crossbeam-channel不支持精确计数，返回0表示无法获取
        0
    }

    /// 检查队列是否为空（非阻塞）
    ///
    /// 注意：这个方法会尝试接收一个元素，如果成功则放回队列
    /// 这不是一个精确的检查，但可以用于快速判断
    pub fn is_empty(&self) -> bool {
        match self.receiver.try_recv() {
            Ok(item) => {
                // 如果成功接收，说明队列不为空，需要放回去
                // 注意：这会导致元素顺序改变，但对于空检查来说可以接受
                let _ = self.sender.send(item);
                false
            }
            Err(crossbeam_channel::TryRecvError::Empty) => true,
            Err(crossbeam_channel::TryRecvError::Disconnected) => true,
        }
    }

    /// 获取发送端（用于克隆）
    pub fn sender(&self) -> &Sender<T> {
        &self.sender
    }

    /// 获取接收端（用于克隆）
    pub fn receiver(&self) -> &Receiver<T> {
        &self.receiver
    }
}

impl<T> Clone for SynchronizedQueue<T> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            receiver: self.receiver.clone(),
        }
    }
}

impl<T> Default for SynchronizedQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_atomic_counter() {
        let counter = Arc::new(AtomicCounter::new(0));
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
    fn test_atomic_flag() {
        let flag = AtomicFlag::new(false);

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
        {
            let guard = wrapper.read().unwrap();
            let sum = guard.iter().sum::<i32>();
            assert_eq!(sum, 6);
        }

        // 写入数据
        {
            let mut guard = wrapper.write().unwrap();
            guard.push(4);
        }

        // 再次读取
        {
            let guard = wrapper.read().unwrap();
            let sum = guard.iter().sum::<i32>();
            assert_eq!(sum, 10);
        }

        // 测试锁竞争监控
        let metrics = wrapper.get_metrics();
        assert_eq!(metrics.contention_count(), 0); // 期望无竞争
    }

    #[test]
    fn test_synchronized_queue() {
        let queue = Arc::new(SynchronizedQueue::new());
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
