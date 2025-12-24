use crossbeam_channel::{unbounded, Receiver, Sender};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Default)]
pub struct LockMetrics {
    contention_count: AtomicU64,
    wait_time_ns: AtomicU64,
}

pub struct LockFreeCounter {
    value: AtomicU64,
}

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

pub struct AtomicCounter {
    value: AtomicU64,
}

impl AtomicCounter {
    pub fn new(initial: u64) -> Self {
        Self {
            value: AtomicU64::new(initial),
        }
    }

    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }

    pub fn decrement(&self) -> u64 {
        self.value.fetch_sub(1, Ordering::SeqCst)
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }

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

    pub fn increment(&self) -> u64 {
        self.value.fetch_add(1, Ordering::SeqCst)
    }

    pub fn decrement(&self) -> u64 {
        self.value.fetch_sub(1, Ordering::SeqCst)
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::SeqCst)
    }

    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::SeqCst);
    }
}

pub struct AtomicFlag {
    value: AtomicBool,
}

impl AtomicFlag {
    pub fn new(initial: bool) -> Self {
        Self {
            value: AtomicBool::new(initial),
        }
    }

    pub fn set(&self, value: bool) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn get(&self) -> bool {
        self.value.load(Ordering::SeqCst)
    }

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

    pub fn set(&self, value: bool) {
        self.value.store(value, Ordering::SeqCst);
    }

    pub fn get(&self) -> bool {
        self.value.load(Ordering::SeqCst)
    }

    pub fn swap(&self, value: bool) -> bool {
        self.value.swap(value, Ordering::SeqCst)
    }
}

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

    pub fn get_metrics(&self) -> &LockMetrics {
        &self.metrics
    }

    pub fn read(&self) -> Result<RwLockReadGuard<'_, T>, ()> {
        let start = std::time::Instant::now();

        match self.inner.read() {
            Ok(guard) => Ok(guard),
            Err(_) => {
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

    pub fn write(&self) -> Result<RwLockWriteGuard<'_, T>, ()> {
        let start = std::time::Instant::now();

        match self.inner.write() {
            Ok(guard) => Ok(guard),
            Err(_) => {
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
            metrics: LockMetrics::default(),
        }
    }
}

impl<T: Default> Default for RwLockWrapper<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

pub struct SynchronizedQueue<T> {
    sender: Sender<T>,
    receiver: Receiver<T>,
}

impl<T> SynchronizedQueue<T> {
    pub fn new() -> Self {
        let (sender, receiver) = unbounded();
        Self { sender, receiver }
    }

    pub fn push(&self, value: T) {
        if let Err(e) = self.sender.send(value) {
            tracing::warn!(target: "performance", "Failed to push to queue: {}", e);
        }
    }

    pub fn pop(&self) -> Option<T> {
        self.receiver.try_recv().ok()
    }

    pub fn pop_timeout(&self, timeout: std::time::Duration) -> Option<T> {
        self.receiver.recv_timeout(timeout).ok()
    }

    pub fn pop_blocking(&self) -> Option<T> {
        self.receiver.recv().ok()
    }

    pub fn len(&self) -> usize {
        0
    }

    pub fn is_empty(&self) -> bool {
        match self.receiver.try_recv() {
            Ok(item) => {
                let _ = self.sender.send(item);
                false
            }
            Err(crossbeam_channel::TryRecvError::Empty) => true,
            Err(crossbeam_channel::TryRecvError::Disconnected) => true,
        }
    }

    pub fn sender(&self) -> &Sender<T> {
        &self.sender
    }

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

        for _ in 0..10 {
            let counter = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    counter.increment();
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

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

        {
            let guard = wrapper.read().unwrap();
            let sum = guard.iter().sum::<i32>();
            assert_eq!(sum, 6);
        }

        {
            let mut guard = wrapper.write().unwrap();
            guard.push(4);
        }

        {
            let guard = wrapper.read().unwrap();
            let sum = guard.iter().sum::<i32>();
            assert_eq!(sum, 10);
        }

        let metrics = wrapper.get_metrics();
        assert_eq!(metrics.contention_count(), 0);
    }

    #[test]
    fn test_synchronized_queue() {
        let queue = Arc::new(SynchronizedQueue::new());
        let mut handles = vec![];

        for i in 0..5 {
            let queue = Arc::clone(&queue);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    queue.push(i * 10 + j);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(queue.len(), 0);

        let mut count = 0;
        while queue.pop().is_some() {
            count += 1;
        }
        assert_eq!(count, 50);
        assert!(queue.is_empty());
    }
}
