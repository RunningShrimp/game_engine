// Test Helpers
// 提供测试工具函数

use std::time::{Duration, Instant};
use std::sync::{Arc, Mutex};
use std::thread;

/// 性能计时器
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.elapsed().as_secs_f64() * 1000.0
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}

/// 重试机制
pub fn retry<F, R, E>(mut attempts: u32, mut delay: Duration, mut operation: F) -> Result<R, E>
where
    F: FnMut() -> Result<R, E>,
    E: std::fmt::Debug,
{
    loop {
        match operation() {
            Ok(result) => return Ok(result),
            Err(err) if attempts > 1 => {
                attempts -= 1;
                thread::sleep(delay);
                delay *= 2; // 指数退避
            }
            Err(err) => return Err(err),
        }
    }
}

/// 并发测试辅助工具
pub struct ConcurrentTestRunner {
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl ConcurrentTestRunner {
    pub fn new() -> Self {
        Self { threads: Vec::new() }
    }

    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.threads.push(thread::spawn(f));
    }

    pub fn join_all(self) {
        for handle in self.threads {
            handle.join().expect("Thread panicked");
        }
    }
}

impl Default for ConcurrentTestRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// 共享状态测试辅助工具
#[derive(Clone)]
pub struct SharedState<T>(Arc<Mutex<T>>);

impl<T> SharedState<T>
where
    T: Default,
{
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(T::default())))
    }

    pub fn with_value(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    pub fn read<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.0.lock().unwrap();
        f(&*guard)
    }

    pub fn write<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.0.lock().unwrap();
        f(&mut *guard)
    }

    pub fn update(&self, value: T) {
        let mut guard = self.0.lock().unwrap();
        *guard = value;
    }
}

/// 测试数据生成器
pub struct TestDataGenerator {
    seed: u64,
}

impl TestDataGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    pub fn with_seed(seed: u64) -> Self {
        Self::new(seed)
    }

    pub fn random_range(&mut self, min: u64, max: u64) -> u64 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        min + (self.seed % (max - min + 1))
    }

    pub fn random_bool(&mut self) -> bool {
        self.random_range(0, 1) == 1
    }

    pub fn random_string(&mut self, length: usize) -> String {
        (0..length)
            .map(|_| {
                let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
                chars[self.random_range(0, (chars.len() - 1) as u64) as usize] as char
            })
            .collect()
    }
}

impl Default for TestDataGenerator {
    fn default() -> Self {
        Self::new(42)
    }
}

/// 内存使用监控器
#[cfg(target_os = "linux")]
pub struct MemoryMonitor {
    initial_usage: usize,
}

#[cfg(target_os = "linux")]
impl MemoryMonitor {
    pub fn new() -> std::io::Result<Self> {
        let initial_usage = Self::get_current_memory_usage()?;
        Ok(Self { initial_usage })
    }

    fn get_current_memory_usage() -> std::io::Result<usize> {
        use std::fs;
        let status = fs::read_to_string("/proc/self/status")?;
        let line = status
            .lines()
            .find(|line| line.starts_with("VmRSS:"))
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "VmRSS not found"))?;

        let parts: Vec<&str> = line.split_whitespace().collect();
        let kb: usize = parts[1].parse().map_err(|_| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid memory value")
        })?;

        Ok(kb * 1024) // 转换为字节
    }

    pub fn current_usage(&self) -> std::io::Result<usize> {
        Self::get_current_memory_usage()
    }

    pub fn delta(&self) -> std::io::Result<isize> {
        let current = Self::get_current_memory_usage()? as isize;
        Ok(current - self.initial_usage as isize)
    }
}

#[cfg(not(target_os = "linux"))]
pub struct MemoryMonitor;

#[cfg(not(target_os = "linux"))]
impl MemoryMonitor {
    pub fn new() -> std::io::Result<Self> {
        Ok(Self)
    }

    pub fn current_usage(&self) -> std::io::Result<usize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Memory monitoring only supported on Linux",
        ))
    }

    pub fn delta(&self) -> std::io::Result<isize> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Memory monitoring only supported on Linux",
        ))
    }
}

/// 测试日志收集器
pub struct TestLogCollector {
    logs: Arc<Mutex<Vec<String>>>,
}

impl TestLogCollector {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn log(&self, message: String) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(message);
    }

    pub fn get_logs(&self) -> Vec<String> {
        let logs = self.logs.lock().unwrap();
        logs.clone()
    }

    pub fn clear(&self) {
        let mut logs = self.logs.lock().unwrap();
        logs.clear();
    }

    pub fn contains(&self, pattern: &str) -> bool {
        let logs = self.logs.lock().unwrap();
        logs.iter().any(|log| log.contains(pattern))
    }

    pub fn count(&self, pattern: &str) -> usize {
        let logs = self.logs.lock().unwrap();
        logs.iter().filter(|log| log.contains(pattern)).count()
    }
}

impl Clone for TestLogCollector {
    fn clone(&self) -> Self {
        Self {
            logs: Arc::clone(&self.logs),
        }
    }
}

impl Default for TestLogCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let timer = Timer::new();
        thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed_ms() >= 10.0);
    }

    #[test]
    fn test_retry_success() {
        let mut attempts = 0;
        let result = retry(3, Duration::from_millis(10), || {
            attempts += 1;
            if attempts < 3 {
                Err("not yet")
            } else {
                Ok("success")
            }
        });
        assert_eq!(result.unwrap(), "success");
    }

    #[test]
    fn test_shared_state() {
        let state = SharedState::new();
        state.write(|v| *v = 42);
        assert_eq!(state.read(|v| *v), 42);
    }

    #[test]
    fn test_data_generator() {
        let mut gen = TestDataGenerator::new(12345);
        let value1 = gen.random_range(1, 100);
        let value2 = gen.random_range(1, 100);
        // 相同种子应该产生相同序列
        let mut gen2 = TestDataGenerator::new(12345);
        let value3 = gen2.random_range(1, 100);
        assert_eq!(value1, value3);
    }

    #[test]
    fn test_log_collector() {
        let logger = TestLogCollector::new();
        logger.log("test message".to_string());
        assert!(logger.contains("test"));
        assert_eq!(logger.count("test"), 1);
        logger.clear();
        assert_eq!(logger.count("test"), 0);
    }
}
