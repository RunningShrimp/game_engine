//! # 并发抽象层 (Concurrency Abstraction Layer)
//!
//! 本模块提供统一的并发数据结构抽象，用于减少条件编译的使用。
//!
//! ## 设计目标
//!
//! - 统一不同并发实现的接口
//! - 减少条件编译的分散使用
//! - 提供类型安全的并发操作
//! - 支持运行时性能优化
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::sync::{ConcurrentMap, DefaultConcurrentMap};
//!
//! // 使用类型别名，自动选择最优实现
//! let map: DefaultConcurrentMap<String, u32> = DefaultConcurrentMap::new();
//! map.insert("key".to_string(), 42);
//!
//! // 或直接使用trait
//! fn process_data<M>(map: &M) -> u32
//! where
//!     M: ConcurrentMap<String, u32>,
//! {
//!     map.get(&"key".to_string()).unwrap_or(0)
//! }
//! ```

use std::collections::HashMap;
use std::sync::RwLock;

// ============================================================================
// Trait 定义
// ============================================================================

/// 并发映射表trait
///
/// 提供线程安全的键值存储接口，支持不同的并发实现策略。
pub trait ConcurrentMap<K, V> {
    /// 获取值
    fn get(&self, key: &K) -> Option<V>;

    /// 插入键值对，返回旧值
    fn insert(&self, key: K, value: V) -> Option<V>;

    /// 移除键，返回旧值
    fn remove(&self, key: &K) -> Option<V>;

    /// 清空所有条目
    fn clear(&self);

    /// 获取条目数量
    fn len(&self) -> usize;

    /// 检查是否为空
    fn is_empty(&self) -> bool;

    /// 检查是否包含某个键
    fn contains_key(&self, key: &K) -> bool;

    /// 迭代所有键
    fn keys(&self) -> Vec<K>;

    /// 迭代所有值
    fn values(&self) -> Vec<V>;
}

/// 并发集合trait
pub trait ConcurrentSet<T> {
    fn insert(&self, value: T) -> bool;
    fn remove(&self, value: &T) -> bool;
    fn contains(&self, value: &T) -> bool;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&self);
}

/// 并发队列trait
pub trait ConcurrentQueue<T> {
    fn push(&self, value: T);
    fn pop(&self) -> Option<T>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn clear(&self);
}

// ============================================================================
// DashMap 实现 (当启用 dashmap feature 时)
// ============================================================================

#[cfg(feature = "dashmap")]
pub use dashmap::DashMap;

#[cfg(feature = "dashmap")]
pub use dashmap::DashSet as DashMapSet;

#[cfg(feature = "dashmap")]
impl<K, V> ConcurrentMap<K, V> for DashMap<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        self.get(key).map(|v| v.clone())
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        self.insert(key, value)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.remove(key).map(|(_k, v)| v)
    }

    fn clear(&self) {
        self.clear();
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn contains_key(&self, key: &K) -> bool {
        self.contains_key(key)
    }

    fn keys(&self) -> Vec<K> {
        self.iter().map(|r| r.key().clone()).collect()
    }

    fn values(&self) -> Vec<V> {
        self.iter().map(|r| r.value().clone()).collect()
    }
}

// ============================================================================
// RwLock<HashMap> 实现 (默认/备用实现)
// ============================================================================

/// 基于RwLock<HashMap>的并发映射表
pub struct RwLockHashMap<K, V> {
    inner: RwLock<HashMap<K, V>>,
}

impl<K, V> RwLockHashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RwLock::new(HashMap::with_capacity(capacity)),
        }
    }
}

impl<K, V> Default for RwLockHashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> ConcurrentMap<K, V> for RwLockHashMap<K, V>
where
    K: std::hash::Hash + Eq + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<V> {
        self.inner.read().ok()?.get(key).cloned()
    }

    fn insert(&self, key: K, value: V) -> Option<V> {
        self.inner.write().ok()?.insert(key, value)
    }

    fn remove(&self, key: &K) -> Option<V> {
        self.inner.write().ok()?.remove(key)
    }

    fn clear(&self) {
        let _ = self.inner.write().map(|mut guard| guard.clear());
    }

    fn len(&self) -> usize {
        self.inner.read().ok().map_or(0, |guard| guard.len())
    }

    fn is_empty(&self) -> bool {
        self.inner.read().ok().map_or(true, |guard| guard.is_empty())
    }

    fn contains_key(&self, key: &K) -> bool {
        self.inner.read().ok().map_or(false, |guard| guard.contains_key(key))
    }

    fn keys(&self) -> Vec<K> {
        self.inner
            .read()
            .ok()
            .map_or_else(Vec::new, |guard| guard.keys().cloned().collect())
    }

    fn values(&self) -> Vec<V> {
        self.inner
            .read()
            .ok()
            .map_or_else(Vec::new, |guard| guard.values().cloned().collect())
    }
}

// ============================================================================
// 类型别名 (统一接口)
// ============================================================================

/// 默认并发映射表类型别名
///
/// 根据feature标志选择最优实现：
/// - 启用 `dashmap` feature 时使用 DashMap（无锁，高并发性能）
/// - 否则使用 RwLockHashMap（标准库，兼容性好）
#[cfg(feature = "dashmap")]
pub type DefaultConcurrentMap<K, V> = DashMap<K, V>;

#[cfg(not(feature = "dashmap"))]
pub type DefaultConcurrentMap<K, V> = RwLockHashMap<K, V>;

/// 默认并发集合类型别名
#[cfg(feature = "dashmap")]
pub type DefaultConcurrentSet<T> = DashMapSet<T>;

#[cfg(not(feature = "dashmap"))]
pub type DefaultConcurrentSet<T> = RwLockHashSet<T>;

/// 基于RwLock的HashSet
pub struct RwLockHashSet<T> {
    inner: RwLock<std::collections::HashSet<T>>,
}

impl<T> RwLockHashSet<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(std::collections::HashSet::new()),
        }
    }
}

impl<T> Default for RwLockHashSet<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ConcurrentSet<T> for RwLockHashSet<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    fn insert(&self, value: T) -> bool {
        self.inner.write().ok().map_or(false, |mut guard| guard.insert(value))
    }

    fn remove(&self, value: &T) -> bool {
        self.inner.write().ok().map_or(false, |mut guard| guard.remove(value))
    }

    fn contains(&self, value: &T) -> bool {
        self.inner.read().ok().map_or(false, |guard| guard.contains(value))
    }

    fn len(&self) -> usize {
        self.inner.read().ok().map_or(0, |guard| guard.len())
    }

    fn is_empty(&self) -> bool {
        self.inner.read().ok().map_or(true, |guard| guard.is_empty())
    }

    fn clear(&self) {
        let _ = self.inner.write().map(|mut guard| guard.clear());
    }
}

// ============================================================================
// 便利函数
// ============================================================================

/// 创建新的默认并发映射表
pub fn new_concurrent_map<K, V>() -> DefaultConcurrentMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    DefaultConcurrentMap::new()
}

/// 创建带容量的默认并发映射表
pub fn new_concurrent_map_with_capacity<K, V>(capacity: usize) -> DefaultConcurrentMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    #[cfg(feature = "dashmap")]
    {
        DashMap::with_capacity(capacity)
    }

    #[cfg(not(feature = "dashmap"))]
    {
        RwLockHashMap::with_capacity(capacity)
    }
}

/// 创建新的默认并发集合
pub fn new_concurrent_set<T>() -> DefaultConcurrentSet<T>
where
    T: std::hash::Hash + Eq + Clone,
{
    DefaultConcurrentSet::new()
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rwlock_hashmap_basic() {
        let map = RwLockHashMap::new();

        assert_eq!(map.insert("key".to_string(), 42), None);
        assert_eq!(map.get(&"key".to_string()), Some(42));
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());

        assert_eq!(map.insert("key".to_string(), 100), Some(42));
        assert_eq!(map.remove(&"key".to_string()), Some(100));
        assert_eq!(map.get(&"key".to_string()), None);
        assert!(map.is_empty());
    }

    #[test]
    fn test_default_concurrent_map() {
        let map: DefaultConcurrentMap<String, i32> = new_concurrent_map();

        map.insert("test".to_string(), 123);
        // 使用 trait 方法，确保返回正确类型
        let value = ConcurrentMap::get(&map, &"test".to_string());
        assert_eq!(value, Some(123));
        assert!(map.contains_key(&"test".to_string()));
    }

    #[test]
    fn test_rwlock_hashset_basic() {
        let set = RwLockHashSet::new();

        assert!(set.insert(42));
        assert!(set.contains(&42));
        assert!(!set.insert(42)); // 重复插入返回false
        assert_eq!(set.len(), 1);
    }
}
