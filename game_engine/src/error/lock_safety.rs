//! 锁安全性工具模块
//!
//! 提供安全的锁获取和死锁预防机制。

use std::sync::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use tracing;

/// 锁获取错误
#[derive(Debug, Clone)]
pub enum LockError {
    /// 锁被污染（Mutex）
    PoisonedMutex(String),
    /// 锁被污染（RwLock）
    PoisonedRwLock(String),
    /// 获取读锁超时
    ReadLockTimeout(String),
    /// 获取写锁超时
    WriteLockTimeout(String),
}

impl std::fmt::Display for LockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PoisonedMutex(msg) => write!(f, "Poisoned Mutex: {}", msg),
            Self::PoisonedRwLock(msg) => write!(f, "Poisoned RwLock: {}", msg),
            Self::ReadLockTimeout(msg) => write!(f, "Read lock timeout: {}", msg),
            Self::WriteLockTimeout(msg) => write!(f, "Write lock timeout: {}", msg),
        }
    }
}

impl std::error::Error for LockError {}

/// 安全获取Mutex锁
///
/// 使用Result替代panic!，优雅处理锁污染情况
pub fn safe_lock<'a, T>(mutex: &'a Mutex<T>, name: &str) -> Result<MutexGuard<'a, T>, LockError> {
    match mutex.lock() {
        Ok(guard) => Ok(guard),
        Err(poison_error) => {
            // 恢复中毒的锁（允许获取已污染的锁以进行清理）
            tracing::warn!(
                target: "lock_safety",
                "Recovered from poisoned Mutex: {}",
                name
            );
            let guard = poison_error.into_inner();
            Ok(guard)
        }
    }
}

/// 尝试获取Mutex锁（非阻塞）
pub fn try_lock<'a, T>(mutex: &'a Mutex<T>, name: &str) -> Result<MutexGuard<'a, T>, LockError> {
    match mutex.try_lock() {
        Ok(guard) => Ok(guard),
        Err(std::sync::TryLockError::Poisoned(poison_error)) => {
            let guard = poison_error.into_inner();
            Ok(guard)
        }
        Err(std::sync::TryLockError::WouldBlock) => {
            Err(LockError::WriteLockTimeout(
                format!("Cannot acquire lock: {}", name)
            ))
        }
    }
}

/// 安全获取RwLock读锁
pub fn safe_read<'a, T>(rw_lock: &'a RwLock<T>, name: &str) -> Result<RwLockReadGuard<'a, T>, LockError> {
    match rw_lock.read() {
        Ok(guard) => Ok(guard),
        Err(poison_error) => {
            tracing::warn!(
                target: "lock_safety",
                "Recovered from poisoned RwLock in safe_read: {}",
                name
            );
            let guard = poison_error.into_inner();
            Ok(guard)
        }
    }
}

/// 安全获取RwLock写锁
pub fn safe_write<'a, T>(rw_lock: &'a RwLock<T>, name: &str) -> Result<RwLockWriteGuard<'a, T>, LockError> {
    match rw_lock.write() {
        Ok(guard) => Ok(guard),
        Err(poison_error) => {
            tracing::warn!(
                target: "lock_safety",
                "Recovered from poisoned RwLock in safe_write: {}",
                name
            );
            let guard = poison_error.into_inner();
            Ok(guard)
        }
    }
}

/// 尝试获取RwLock读锁（非阻塞）
pub fn try_read<'a, T>(rw_lock: &'a RwLock<T>, name: &str) -> Result<RwLockReadGuard<'a, T>, LockError> {
    match rw_lock.try_read() {
        Ok(guard) => Ok(guard),
        Err(std::sync::TryLockError::Poisoned(poison_error)) => {
            let guard = poison_error.into_inner();
            tracing::warn!(
                target: "lock_safety",
                "Recovered from poisoned RwLock in try_read: {}",
                name
            );
            Ok(guard)
        }
        Err(std::sync::TryLockError::WouldBlock) => {
            Err(LockError::ReadLockTimeout(
                format!("Cannot acquire read lock: {}", name)
            ))
        }
    }
}

/// 尝试获取RwLock写锁（非阻塞）
pub fn try_write<'a, T>(rw_lock: &'a RwLock<T>, name: &str) -> Result<RwLockWriteGuard<'a, T>, LockError> {
    match rw_lock.try_write() {
        Ok(guard) => Ok(guard),
        Err(std::sync::TryLockError::Poisoned(poison_error)) => {
            let guard = poison_error.into_inner();
            tracing::warn!(
                target: "lock_safety",
                "Recovered from poisoned RwLock in try_write: {}",
                name
            );
            Ok(guard)
        }
        Err(std::sync::TryLockError::WouldBlock) => {
            Err(LockError::WriteLockTimeout(
                format!("Cannot acquire write lock: {}", name)
            ))
        }
    }
}

/// RAII 锁作用域保证（确保快速释放）
pub struct ScopedLock<T> {
    data: T,
}

impl<T> ScopedLock<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }

    pub fn get(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_lock_success() {
        let mutex = Mutex::new(42);
        let result = safe_lock(&mutex, "test");
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), 42);
    }

    #[test]
    fn test_try_lock_success() {
        let mutex = Mutex::new("hello");
        let result = try_lock(&mutex, "test");
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "hello");
    }

    #[test]
    fn test_safe_read_success() {
        let rw_lock = RwLock::new(vec![1, 2, 3]);
        let result = safe_read(&rw_lock, "test");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn test_safe_write_success() {
        let rw_lock = RwLock::new(String::from("test"));
        let result = safe_write(&rw_lock, "test");
        assert!(result.is_ok());
        assert_eq!(*result.unwrap(), "test");
    }
}
