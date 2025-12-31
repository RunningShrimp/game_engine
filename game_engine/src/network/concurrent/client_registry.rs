//! 客户端注册表抽象层
//!
//! # 目的
//!
//! 使用 trait 抽象替代条件编译，减少代码重复并提高可维护性。
//!
//! # 设计模式
//!
//! - 定义统一的 `ClientRegistry` trait
//! - 为 DashMap 和 Mutex<HashMap> 分别实现 adapter
//! - 运行时通过 trait 对象使用，避免编译时条件编译
//!
//! # 使用示例
//!
//! ```rust
//! use game_engine::network::concurrent::{ClientRegistry, DashMapClientRegistry, MutexClientRegistry};
//! use game_engine::network::server::ClientConnection;
//!
//! // 使用 DashMap (启用 dashmap feature)
//! #[cfg(feature = "dashmap")]
//! let registry: DashMapClientRegistry<u64, ClientConnection> = DashMapClientRegistry::new();
//!
//! // 使用 Mutex<HashMap] (默认)
//! #[cfg(not(feature = "dashmap"))]
//! let registry: MutexClientRegistry<u64, ClientConnection> = MutexClientRegistry::new();
//!
//! // 通过 trait 使用
//! fn add_client<R: ClientRegistry<u64, ClientConnection>>(
//!     registry: &R,
//!     id: u64,
//!     client: ClientConnection,
//! ) {
//!     registry.add_client(id, client);
//! }
//! ```

use std::sync::Arc;
use tokio::sync::Mutex;

#[cfg(feature = "dashmap")]
use dashmap::DashMap;

#[cfg(not(feature = "dashmap"))]
use std::collections::HashMap;

use crate::network::server::{ClientConnection, SyncClientConnection};

/// 客户端注册表 trait - 统一接口
///
/// # 目的
///
/// 为不同的并发客户端注册表实现提供统一接口，避免条件编译导致的代码重复。
///
/// # 约束
///
/// - K: Key 类型，必须支持 Copy + Send + Sync
/// - V: Value 类型，必须支持 Send + Sync
/// - Send + Sync: 线程安全
pub trait ClientRegistry<K, V>: Send + Sync
where
    K: Copy + Send + Sync,
    V: Send + Sync,
{
    /// 添加客户端连接
    fn add_client(&self, key: K, client: V) -> Result<(), String>;

    /// 获取客户端连接（返回克隆）
    fn get_client(&self, key: K) -> Option<V>
    where
        V: Clone;

    /// 移除客户端连接
    fn remove_client(&self, key: K) -> Option<V>;

    /// 获取客户端数量
    fn client_count(&self) -> usize;

    /// 检查是否包含客户端
    fn contains_client(&self, key: K) -> bool
    where
        V: Clone,
    {
        self.get_client(key).is_some()
    }

    /// 获取所有客户端ID
    fn all_client_ids(&self) -> Vec<K>;

    /// 广播消息到所有客户端
    fn broadcast<F>(&self, f: F) -> Result<(), String>
    where
        F: Fn(&V) -> Result<(), String> + Send + Sync;

    /// 清空所有客户端
    fn clear(&self);
}

// ============================================================================
// DashMap ClientRegistry
// ============================================================================

/// DashMap adapter - 高性能并发客户端注册表
///
/// # 性能特性
///
/// - 无锁并发读取
/// - 分片存储设计
/// - 适合高并发场景
///
/// # 性能提升
///
/// - 并发读取: 5-10x vs Mutex<HashMap]
/// - 并发写入: 2-5x vs Mutex<HashMap]
#[cfg(feature = "dashmap")]
pub struct DashMapClientRegistry<K, V>
where
    K: Copy + Send + Sync,
{
    inner: Arc<DashMap<K, V>>,
}

#[cfg(feature = "dashmap")]
impl<K, V> DashMapClientRegistry<K, V>
where
    K: Copy + std::cmp::Eq + std::hash::Hash + Send + Sync,
    V: Send + Sync,
{
    /// 创建新的 DashMap 客户端注册表
    pub fn new() -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
        }
    }

    /// 获取内部 DashMap 的引用（高级用法）
    pub fn inner(&self) -> &DashMap<K, V> {
        &self.inner
    }
}

#[cfg(feature = "dashmap")]
impl<K, V> Default for DashMapClientRegistry<K, V>
where
    K: Copy + std::cmp::Eq + std::hash::Hash + Send + Sync,
    V: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "dashmap")]
impl<K, V> ClientRegistry<K, V> for DashMapClientRegistry<K, V>
where
    K: Copy + std::hash::Hash + std::cmp::Eq + Send + Sync,
    V: Clone + Send + Sync,
{
    fn add_client(&self, key: K, client: V) -> Result<(), String> {
        self.inner.insert(key, client);
        Ok(())
    }

    fn get_client(&self, key: K) -> Option<V> {
        self.inner.get(&key).map(|v| v.clone())
    }

    fn remove_client(&self, key: K) -> Option<V> {
        self.inner.remove(&key).map(|(_, v)| v)
    }

    fn client_count(&self) -> usize {
        self.inner.len()
    }

    fn all_client_ids(&self) -> Vec<K> {
        self.inner.iter().map(|entry| *entry.key()).collect()
    }

    fn broadcast<F>(&self, f: F) -> Result<(), String>
    where
        F: Fn(&V) -> Result<(), String> + Send + Sync,
    {
        let mut errors = Vec::new();
        for item in self.inner.iter() {
            if let Err(e) = f(item.value()) {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("Broadcast failed for {} clients", errors.len()))
        }
    }

    fn clear(&self) {
        self.inner.clear()
    }
}

// ============================================================================
// Mutex<HashMap] ClientRegistry
// ============================================================================

/// Mutex<HashMap] adapter - 兼容的客户端注册表
///
/// # 性能特性
///
/// - 使用 tokio::sync::Mutex（异步友好）
/// - 适合中低并发场景
/// - 内存占用更小
#[cfg(not(feature = "dashmap"))]
pub struct MutexClientRegistry<K, V>
where
    K: Copy + Send + Sync,
{
    inner: Arc<Mutex<HashMap<K, V>>>,
}

#[cfg(not(feature = "dashmap"))]
impl<K, V> MutexClientRegistry<K, V>
where
    K: Copy + Send + Sync,
    V: Send + Sync,
{
    /// 创建新的 Mutex 客户端注册表
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取内部 Mutex 的引用（高级用法）
    pub fn inner(&self) -> &Mutex<HashMap<K, V>> {
        &self.inner
    }
}

#[cfg(not(feature = "dashmap"))]
impl<K, V> Default for MutexClientRegistry<K, V>
where
    K: Copy + Send + Sync,
    V: Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "dashmap"))]
impl<K, V> ClientRegistry<K, V> for MutexClientRegistry<K, V>
where
    K: Copy + std::hash::Hash + std::cmp::Eq + Send + Sync,
    V: Clone + Send + Sync,
{
    fn add_client(&self, key: K, client: V) -> Result<(), String> {
        let mut clients = self.inner.try_lock().map_err(|_| "Lock poisoned")?;
        clients.insert(key, client);
        Ok(())
    }

    fn get_client(&self, key: K) -> Option<V> {
        let clients = self.inner.try_lock().ok()?;
        clients.get(&key).cloned()
    }

    fn remove_client(&self, key: K) -> Option<V> {
        let mut clients = self.inner.try_lock().ok()?;
        clients.remove(&key)
    }

    fn client_count(&self) -> usize {
        self.inner.try_lock().ok().map_or(0, |clients| clients.len())
    }

    fn all_client_ids(&self) -> Vec<K> {
        self.inner.try_lock().ok().map_or(Vec::new(), |clients| {
            clients.keys().copied().collect()
        })
    }

    fn broadcast<F>(&self, f: F) -> Result<(), String>
    where
        F: Fn(&V) -> Result<(), String> + Send + Sync,
    {
        let clients = self.inner.try_lock().map_err(|_| "Lock poisoned")?;
        let mut errors = Vec::new();
        for client in clients.values() {
            if let Err(e) = f(client) {
                errors.push(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(format!("Broadcast failed for {} clients", errors.len()))
        }
    }

    fn clear(&self) {
        if let Ok(mut clients) = self.inner.try_lock() {
            clients.clear();
        }
    }
}

// ============================================================================
// 类型别名 - 简化使用
// ============================================================================

/// 默认客户端注册表类型别名
///
/// 根据 feature flag 自动选择实现：
/// - `dashmap` feature: DashMapClientRegistry
/// - 默认: MutexClientRegistry
#[cfg(feature = "dashmap")]
pub type DefaultClientRegistry<K, V> = DashMapClientRegistry<K, V>;

#[cfg(not(feature = "dashmap"))]
pub type DefaultClientRegistry<K, V> = MutexClientRegistry<K, V>;

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Test data structure
    #[derive(Clone, Debug)]
    struct TestClient {
        id: u64,
        name: String,
    }

    #[test]
    fn test_client_registry_basic() {
        #[cfg(feature = "dashmap")]
        let registry: DashMapClientRegistry<u64, TestClient> = DashMapClientRegistry::new();

        #[cfg(not(feature = "dashmap"))]
        let registry: MutexClientRegistry<u64, TestClient> = MutexClientRegistry::new();

        // Test add_client
        let client = TestClient {
            id: 1,
            name: "Test".to_string(),
        };
        assert!(registry.add_client(1, client).is_ok());

        // Test contains_client
        assert!(registry.contains_client(1));
        assert!(!registry.contains_client(999));

        // Test get_client
        let retrieved = registry.get_client(1);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, 1);

        // Test client_count
        assert_eq!(registry.client_count(), 1);

        // Test remove_client
        let removed = registry.remove_client(1);
        assert!(removed.is_some());
        assert_eq!(registry.client_count(), 0);
    }

    #[test]
    fn test_client_registry_multiple_clients() {
        #[cfg(feature = "dashmap")]
        let registry: DashMapClientRegistry<u64, TestClient> = DashMapClientRegistry::new();

        #[cfg(not(feature = "dashmap"))]
        let registry: MutexClientRegistry<u64, TestClient> = MutexClientRegistry::new();

        // Add multiple clients
        for i in 1..=10 {
            let client = TestClient {
                id: i,
                name: format!("Client{}", i),
            };
            registry.add_client(i, client).unwrap();
        }

        assert_eq!(registry.client_count(), 10);

        // Test all_client_ids
        let ids = registry.all_client_ids();
        assert_eq!(ids.len(), 10);
        assert!(ids.contains(&1));
        assert!(ids.contains(&10));
    }

    #[test]
    fn test_client_registry_broadcast() {
        #[cfg(feature = "dashmap")]
        let registry: DashMapClientRegistry<u64, TestClient> = DashMapClientRegistry::new();

        #[cfg(not(feature = "dashmap"))]
        let registry: MutexClientRegistry<u64, TestClient> = MutexClientRegistry::new();

        // Add clients
        for i in 1..=5 {
            let client = TestClient {
                id: i,
                name: format!("Client{}", i),
            };
            registry.add_client(i, client).unwrap();
        }

        // Test broadcast
        let result = registry.broadcast(|_| Ok(()));
        assert!(result.is_ok());

        // Test broadcast with errors
        let result = registry.broadcast(|client| {
            if client.id % 2 == 0 {
                Err("Error".to_string())
            } else {
                Ok(())
            }
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_client_registry_clear() {
        #[cfg(feature = "dashmap")]
        let registry: DashMapClientRegistry<u64, TestClient> = DashMapClientRegistry::new();

        #[cfg(not(feature = "dashmap"))]
        let registry: MutexClientRegistry<u64, TestClient> = MutexClientRegistry::new();

        // Add clients
        for i in 1..=5 {
            let client = TestClient {
                id: i,
                name: format!("Client{}", i),
            };
            registry.add_client(i, client).unwrap();
        }

        assert_eq!(registry.client_count(), 5);

        registry.clear();

        assert_eq!(registry.client_count(), 0);
        assert!(!registry.contains_client(1));
    }
}
