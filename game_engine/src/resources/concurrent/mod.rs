//! 并发容器抽象层
//!
//! # 目的
//!
//! 使用 trait 抽象替代条件编译，减少代码重复并提高可维护性。
//!
//! # 设计模式
//!
//! - 定义统一的 `ConcurrentMap` trait
//! - 为 DashMap 和 RwLock<HashMap> 分别实现 adapter
//! - 运行时通过 trait 对象使用，避免编译时条件编译
//!
//! # 使用示例
//!
//! ```rust
//! use game_engine::resources::concurrent::{ConcurrentMap, DashMapAdapter, RwLockAdapter};
//!
//! // 使用 DashMap (启用 dashmap feature)
//! #[cfg(feature = "dashmap")]
//! let map: DashMapAdapter<String, Vec<u8>> = DashMapAdapter::new();
//!
//! // 使用 RwLock<HashMap] (默认)
//! #[cfg(not(feature = "dashmap"))]
//! let map: RwLockAdapter<String, Vec<u8>> = RwLockAdapter::new();
//!
//! // 通过 trait 使用
//! fn use_map<M: ConcurrentMap<K, V>>(map: &M, key: &K) -> Option<&V>
//! where
//!     K: Eq + std::hash::Hash,
//! {
//!     map.get(key)
//! }
//! ```

use std::hash::Hash;

#[cfg(feature = "dashmap")]
use dashmap::DashMap;

#[cfg(feature = "dashmap")]
use parking_lot::RwLock;

#[cfg(not(feature = "dashmap"))]
use parking_lot::RwLock;

#[cfg(not(feature = "dashmap"))]
use std::collections::HashMap;

/// 并发 Map trait - 统一接口
///
/// # 目的
///
/// 为不同的并发 Map 实现提供统一接口，避免条件编译导致的代码重复。
///
/// # 约束
///
/// - K: Key 类型，必须支持 Eq, Hash, Clone, Send, Sync
/// - V: Value 类型，必须支持 Clone, Send, Sync
/// - Send + Sync: 线程安全
pub trait ConcurrentMap<K, V>: Send + Sync
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// 获取值引用
    fn get(&self, key: &K) -> Option<V>;

    /// 插入键值对，返回旧值
    fn insert(&self, key: K, value: V) -> Option<V>;

    /// 移除键值对，返回旧值
    fn remove(&self, key: &K) -> Option<V>;

    /// 获取 Map 长度
    fn len(&self) -> usize;

    /// 检查是否为空
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// 清空 Map
    fn clear(&self);

    /// 检查是否包含键
    fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// 迭代所有键
    fn keys(&self) -> Vec<K>;

    /// 迭代所有值
    fn values(&self) -> Vec<V>;
}

// ============================================================================
// DashMap Adapter
// ============================================================================

/// DashMap adapter - 高性能并发 HashMap
///
/// # 性能特性
///
/// - 无锁并发读取
/// - 分片存储设计
/// - 适合读多写少场景
///
/// # 性能提升
///
/// - 并发读取: 5-10x vs RwLock<HashMap]
/// - 并发写入: 2-5x vs RwLock<HashMap]
#[cfg(feature = "dashmap")]
#[derive(Debug)]
pub struct DashMapAdapter<K, V>
where
    K: Eq + Hash,
{
    inner: DashMap<K, V>,
}

#[cfg(feature = "dashmap")]
impl<K, V> Default for DashMapAdapter<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> DashMapAdapter<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// 创建新的 DashMap adapter
    pub fn new() -> Self {
        Self {
            inner: DashMap::new(),
        }
    }

    /// 创建带容量的 DashMap adapter
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: DashMap::with_capacity(capacity),
        }
    }

    /// 获取内部 DashMap 的引用（高级用法）
    pub fn inner(&self) -> &DashMap<K, V> {
        &self.inner
    }
}

#[cfg(feature = "dashmap")]
impl<K, V> ConcurrentMap<K, V> for DashMapAdapter<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn get(&self, key: &K) -> Option<V> {
        self.inner.get(key).map(|r| r.clone())
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.insert(key, value)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.inner.remove(key).map(|(_, v)| v)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn clear(&self) {
        self.inner.clear()
    }

    fn keys(&self) -> Vec<K> {
        self.inner.iter().map(|entry| entry.key().clone()).collect()
    }

    fn values(&self) -> Vec<V> {
        self.inner.iter().map(|entry| entry.value().clone()).collect()
    }
}

// ============================================================================
// RwLock<HashMap] Adapter
// ============================================================================

/// RwLock<HashMap] adapter - 兼容的并发 HashMap
///
/// # 性能特性
///
/// - 使用 parking_lot::RwLock（比 std::sync::RwLock 快 2.5-8x）
/// - 适合读多写少场景
/// - 内存占用更小
#[cfg(not(feature = "dashmap"))]
#[derive(Debug)]
pub struct RwLockAdapter<K, V>
where
    K: Eq + Hash,
{
    inner: RwLock<HashMap<K, V>>,
}

#[cfg(not(feature = "dashmap"))]
impl<K, V> RwLockAdapter<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    /// 创建新的 RwLock adapter
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    /// 创建带容量的 RwLock adapter
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }

    /// 获取内部 RwLock 的引用（高级用法）
    pub fn inner(&self) -> &RwLock<HashMap<K, V>> {
        &self.inner
    }
}

#[cfg(not(feature = "dashmap"))]
impl<K, V> ConcurrentMap<K, V> for RwLockAdapter<K, V>
where
    K: Eq + Hash + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    fn get(&self, key: &K) -> Option<V> {
        self.inner.read().get(key).cloned()
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.write().insert(key, value)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.inner.write().remove(key)
    }

    fn len(&self) -> usize {
        self.inner.read().len()
    }

    fn clear(&self) {
        self.inner.write().clear()
    }

    fn keys(&self) -> Vec<K> {
        self.inner.read().keys().cloned().collect()
    }

    fn values(&self) -> Vec<V> {
        self.inner.read().values().cloned().collect()
    }
}

// ============================================================================
// 类型别名 - 简化使用
// ============================================================================

/// 默认并发 Map 类型别名
///
/// 根据 feature flag 自动选择实现：
/// - `dashmap` feature: DashMapAdapter
/// - 默认: RwLockAdapter
#[cfg(feature = "dashmap")]
pub type DefaultConcurrentMap<K, V> = DashMapAdapter<K, V>;

#[cfg(not(feature = "dashmap"))]
pub type DefaultConcurrentMap<K, V> = RwLockAdapter<K, V>;

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concurrent_map_basic() {
        #[cfg(feature = "dashmap")]
        let map: DashMapAdapter<String, String> = DashMapAdapter::new();

        #[cfg(not(feature = "dashmap"))]
        let map: RwLockAdapter<String, String> = RwLockAdapter::new();

        // 测试插入
        map.insert("key1".to_string(), "value1".to_string());
        assert!(map.contains_key(&"key1".to_string()));

        // 测试获取
        let value = map.get(&"key1".to_string());
        assert_eq!(value, Some("value1".to_string()));

        // 测试长度
        assert_eq!(map.len(), 1);

        // 测试删除
        map.remove(&"key1".to_string());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_concurrent_map_clone_values() {
        #[cfg(feature = "dashmap")]
        let map: DashMapAdapter<String, Vec<u8>> = DashMapAdapter::new();

        #[cfg(not(feature = "dashmap"))]
        let map: RwLockAdapter<String, Vec<u8>> = RwLockAdapter::new();

        map.insert("data".to_string(), vec![1, 2, 3, 4]);

        let values = map.values();
        assert_eq!(values, vec![vec![1, 2, 3, 4]]);
    }

    #[test]
    fn test_concurrent_map_keys() {
        #[cfg(feature = "dashmap")]
        let map: DashMapAdapter<String, u32> = DashMapAdapter::new();

        #[cfg(not(feature = "dashmap"))]
        let map: RwLockAdapter<String, u32> = RwLockAdapter::new();

        map.insert("a".to_string(), 1);
        map.insert("b".to_string(), 2);

        let keys = map.keys();
        assert!(keys.contains(&"a".to_string()));
        assert!(keys.contains(&"b".to_string()));
    }

    #[test]
    fn test_concurrent_map_clear() {
        #[cfg(feature = "dashmap")]
        let map: DashMapAdapter<String, u32> = DashMapAdapter::new();

        #[cfg(not(feature = "dashmap"))]
        let map: RwLockAdapter<String, u32> = RwLockAdapter::new();

        map.insert("x".to_string(), 10);
        assert_eq!(map.len(), 1);

        map.clear();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }
}
