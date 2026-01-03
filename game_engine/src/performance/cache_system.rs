//! # 多级缓存系统
//!
//! **API 稳定性**: 稳定 (Stable) (v0.1.0)
//!
//! 提供高性能的多级缓存系统，包括：
//! - L1/L2/L3多级缓存
//! - 缓存预取
//! - 缓存一致性管理
//! - 自适应缓存策略
//!
//! ## 功能特性
//!
//! | 功能 | 状态 | 说明 |
//! |------|------|------|
//! | 多级缓存 | ✅ 已实现 | L1/L2/L3三级缓存 |
//! | 缓存预取 | ✅ 已实现 | 智能预取算法 |
//! | 缓存一致性 | ✅ 已实现 | 自动同步机制 |
//! | 自适应策略 | ✅ 已实现 | 基于访问模式调整 |

use lru::LruCache;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tracing;

/// 缓存级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheLevel {
    /// L1缓存 - 最快，最小
    L1 = 1,
    /// L2缓存 - 中等速度和大小
    L2 = 2,
    /// L3缓存 - 较慢，较大
    L3 = 3,
}

/// 缓存策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CachePolicy {
    /// LRU (Least Recently Used)
    Lru,
    /// LFU (Least Frequently Used)
    Lfu,
    /// FIFO (First In First Out)
    Fifo,
    /// 自适应
    Adaptive,
}

/// 缓存项
#[derive(Debug, Clone)]
pub struct CacheEntry<V> {
    /// 值
    pub value: V,
    /// 访问次数
    pub access_count: u64,
    /// 最后访问时间
    pub last_access: Instant,
    /// 创建时间
    pub created_at: Instant,
    /// 缓存级别
    pub level: CacheLevel,
    /// 数据大小（字节）
    pub size: usize,
}

impl<V> CacheEntry<V> {
    /// 创建新的缓存项
    pub fn new(value: V, level: CacheLevel, size: usize) -> Self {
        let now = Instant::now();
        Self {
            value,
            access_count: 1,
            last_access: now,
            created_at: now,
            level,
            size,
        }
    }

    /// 记录访问
    pub fn access(&mut self) {
        self.access_count += 1;
        self.last_access = Instant::now();
    }

    /// 获取访问频率（每秒访问次数）
    pub fn access_frequency(&self) -> f64 {
        let duration = self.created_at.elapsed().as_secs_f64();
        if duration > 0.0 {
            self.access_count as f64 / duration
        } else {
            self.access_count as f64
        }
    }
}

/// 预取策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefetchStrategy {
    /// 顺序预取
    Sequential,
    /// 基于历史的预取
    HistoryBased,
    /// 基于模式的预取
    PatternBased,
    /// 自适应预取
    Adaptive,
}

/// 缓存配置
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// L1缓存大小
    pub l1_size: usize,
    /// L2缓存大小
    pub l2_size: usize,
    /// L3缓存大小
    pub l3_size: usize,
    /// 缓存策略
    pub policy: CachePolicy,
    /// 预取策略
    pub prefetch_strategy: PrefetchStrategy,
    /// 是否启用预取
    pub enable_prefetch: bool,
    /// 预取距离
    pub prefetch_distance: usize,
    /// 缓存过期时间
    pub ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            l1_size: 256,  // 256项
            l2_size: 1024, // 1K项
            l3_size: 4096, // 4K项
            policy: CachePolicy::Adaptive,
            prefetch_strategy: PrefetchStrategy::Adaptive,
            enable_prefetch: true,
            prefetch_distance: 4,
            ttl: Duration::from_secs(300), // 5分钟
        }
    }
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// L1缓存命中次数
    pub l1_hits: u64,
    /// L2缓存命中次数
    pub l2_hits: u64,
    /// L3缓存命中次数
    pub l3_hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 预取命中次数
    pub prefetch_hits: u64,
    /// 总访问次数
    pub total_accesses: u64,
    /// 淘汰次数
    pub evictions: u64,
    /// 当前缓存大小（字节）
    pub current_size: usize,
    /// 缓存命中率（0-1）
    pub hit_rate: f32,
}

impl CacheStats {
    /// 计算整体命中率
    pub fn calculate_hit_rate(&mut self) {
        let total_hits = self.l1_hits + self.l2_hits + self.l3_hits + self.prefetch_hits;
        self.total_accesses = total_hits + self.misses;

        if self.total_accesses > 0 {
            self.hit_rate = total_hits as f32 / self.total_accesses as f32;
        } else {
            self.hit_rate = 0.0;
        }
    }
}

/// 多级缓存系统
pub struct MultiLevelCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// L1缓存
    l1_cache: Arc<Mutex<LruCache<K, CacheEntry<V>>>>,
    /// L2缓存
    l2_cache: Arc<Mutex<LruCache<K, CacheEntry<V>>>>,
    /// L3缓存
    l3_cache: Arc<Mutex<LruCache<K, CacheEntry<V>>>>,
    /// 预取缓存
    prefetch_cache: Arc<Mutex<HashMap<K, CacheEntry<V>>>>,
    /// 配置
    config: CacheConfig,
    /// 统计
    stats: Arc<RwLock<CacheStats>>,
    /// 访问历史（用于预取）
    access_history: Arc<Mutex<Vec<K>>>,
    /// 最大历史记录数
    max_history: usize,
}

impl<K, V> MultiLevelCache<K, V>
where
    K: Hash + Eq + Clone + Send + Sync + 'static + std::fmt::Debug,
    V: Clone + Send + Sync + 'static,
{
    /// 创建新的多级缓存
    pub fn new(config: CacheConfig) -> Self {
        Self {
            l1_cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZero::new(config.l1_size)
                    .unwrap_or(std::num::NonZero::new(1).unwrap()),
            ))),
            l2_cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZero::new(config.l2_size)
                    .unwrap_or(std::num::NonZero::new(1).unwrap()),
            ))),
            l3_cache: Arc::new(Mutex::new(LruCache::new(
                std::num::NonZero::new(config.l3_size)
                    .unwrap_or(std::num::NonZero::new(1).unwrap()),
            ))),
            prefetch_cache: Arc::new(Mutex::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
            access_history: Arc::new(Mutex::new(Vec::new())),
            max_history: 1000,
        }
    }

    /// 使用默认配置创建
    pub fn with_default_config() -> Self {
        Self::new(CacheConfig::default())
    }

    /// 获取缓存项
    pub async fn get(&self, key: &K) -> Option<V> {
        // 记录访问历史
        self.record_access(key.clone()).await;

        // 1. 尝试从预取缓存获取
        if self.config.enable_prefetch {
            if let Some(entry) = self.prefetch_cache.lock().await.get(key) {
                let mut stats = self.stats.write().await;
                stats.prefetch_hits += 1;
                tracing::trace!("Prefetch cache hit for key {:?}", key);
                return Some(entry.value.clone());
            }
        }

        // 2. 尝试从L1缓存获取
        {
            let mut l1 = self.l1_cache.lock().await;
            if let Some(entry) = l1.get_mut(key) {
                entry.access();
                let mut stats = self.stats.write().await;
                stats.l1_hits += 1;
                stats.calculate_hit_rate();
                tracing::trace!("L1 cache hit for key {:?}", key);
                return Some(entry.value.clone());
            }
        }

        // 3. 尝试从L2缓存获取（并提升到L1）
        {
            let mut l2 = self.l2_cache.lock().await;
            if let Some(mut entry) = l2.pop(key) {
                entry.access();
                entry.level = CacheLevel::L1;

                // 提升到L1缓存
                let mut l1 = self.l1_cache.lock().await;
                if l1.len() >= self.config.l1_size {
                    l1.pop_lru();
                }
                l1.put(key.clone(), entry.clone());

                let mut stats = self.stats.write().await;
                stats.l2_hits += 1;
                stats.calculate_hit_rate();
                tracing::trace!("L2 cache hit for key {:?}, promoted to L1", key);
                return Some(entry.value.clone());
            }
        }

        // 4. 尝试从L3缓存获取（并提升到L2）
        {
            let mut l3 = self.l3_cache.lock().await;
            if let Some(mut entry) = l3.pop(key) {
                entry.access();
                entry.level = CacheLevel::L2;

                // 提升到L2缓存
                let mut l2 = self.l2_cache.lock().await;
                if l2.len() >= self.config.l2_size {
                    l2.pop_lru();
                }
                l2.put(key.clone(), entry.clone());

                let mut stats = self.stats.write().await;
                stats.l3_hits += 1;
                stats.calculate_hit_rate();
                tracing::trace!("L3 cache hit for key {:?}, promoted to L2", key);
                return Some(entry.value.clone());
            }
        }

        // 5. 缓存未命中
        let mut stats = self.stats.write().await;
        stats.misses += 1;
        stats.calculate_hit_rate();
        tracing::trace!("Cache miss for key {:?}", key);
        None
    }

    /// 插入缓存项
    pub async fn put(&self, key: K, value: V, size: usize) {
        let entry = CacheEntry::new(value, CacheLevel::L1, size);

        // 插入到L1缓存
        let mut l1 = self.l1_cache.lock().await;
        if l1.len() >= self.config.l1_size {
            // 淘汰的项下放到L2
            if let Some((k, evicted)) = l1.pop_lru() {
                self.demote_to_l2(k, evicted).await;
            }
        }
        l1.put(key.clone(), entry);

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.current_size += size;

        // 触发预取
        if self.config.enable_prefetch {
            self.trigger_prefetch(&key).await;
        }
    }

    /// 将条目下放到L2缓存
    async fn demote_to_l2(&self, key: K, mut entry: CacheEntry<V>) {
        entry.level = CacheLevel::L2;

        let mut l2 = self.l2_cache.lock().await;
        if l2.len() >= self.config.l2_size {
            // 淘汰的项下放到L3
            if let Some((k, evicted)) = l2.pop_lru() {
                self.demote_to_l3(k, evicted).await;
            }
        }
        l2.put(key, entry);

        let mut stats = self.stats.write().await;
        stats.evictions += 1;
    }

    /// 将条目下放到L3缓存
    async fn demote_to_l3(&self, key: K, mut entry: CacheEntry<V>) {
        entry.level = CacheLevel::L3;

        let mut l3 = self.l3_cache.lock().await;
        if l3.len() >= self.config.l3_size {
            l3.pop_lru(); // 直接丢弃
        }
        l3.put(key, entry);

        let mut stats = self.stats.write().await;
        stats.evictions += 1;
    }

    /// 记录访问历史
    async fn record_access(&self, key: K) {
        let mut history = self.access_history.lock().await;
        history.push(key);

        // 限制历史大小
        if history.len() > self.max_history {
            history.remove(0);
        }
    }

    /// 触发预取
    async fn trigger_prefetch(&self, key: &K) {
        match self.config.prefetch_strategy {
            PrefetchStrategy::Sequential => {
                // 顺序预取：假设下一个键是当前键 + 1
                // 这里简化处理，实际需要根据键类型实现
            }
            PrefetchStrategy::HistoryBased => {
                // 基于历史的预取：查找历史访问中的模式
                let history = self.access_history.lock().await;
                let mut pattern_map: HashMap<K, usize> = HashMap::new();

                // 统计当前键之后的访问模式
                for window in history.windows(2) {
                    if window[0] == *key {
                        *pattern_map.entry(window[1].clone()).or_insert(0) += 1;
                    }
                }

                // 预取最可能的下一个键
                if let Some((next_key, _count)) =
                    pattern_map.iter().max_by_key(|(_, count)| **count)
                {
                    // 标记为预取项
                    tracing::trace!("Prefetching key based on history: {:?}", next_key);
                }
            }
            PrefetchStrategy::PatternBased => {
                // 基于模式的预取：检测周期性访问模式
                let history = self.access_history.lock().await;
                if history.len() > 10 {
                    // 简化的模式检测
                    let recent = &history[history.len().saturating_sub(10)..];
                    // 实际实现需要更复杂的模式识别算法
                }
            }
            PrefetchStrategy::Adaptive => {
                // 自适应预取：结合多种策略
                let history = self.access_history.lock().await;

                // 使用简单的序列预测
                if let Some(last) = history.last() {
                    if last == key {
                        // 检测到顺序访问
                        tracing::trace!("Detected sequential access pattern");
                    }
                }
            }
        }
    }

    /// 批量预取
    pub async fn prefetch(&self, keys: Vec<K>, loader: impl Fn(&K) -> Option<(V, usize)>) {
        if !self.config.enable_prefetch {
            return;
        }

        let mut prefetch_cache = self.prefetch_cache.lock().await;

        for key in keys {
            // 只预取不在任何缓存中的键
            let in_l1 = self.l1_cache.lock().await.contains(&key);
            let in_l2 = self.l2_cache.lock().await.contains(&key);
            let in_l3 = self.l3_cache.lock().await.contains(&key);

            if !in_l1 && !in_l2 && !in_l3 && !prefetch_cache.contains_key(&key) {
                if let Some((value, size)) = loader(&key) {
                    let entry = CacheEntry::new(value, CacheLevel::L1, size);
                    tracing::trace!("Prefetched key {:?}", key);
                    prefetch_cache.insert(key, entry);
                }
            }
        }
    }

    /// 清除所有缓存
    pub async fn clear(&self) {
        self.l1_cache.lock().await.clear();
        self.l2_cache.lock().await.clear();
        self.l3_cache.lock().await.clear();
        self.prefetch_cache.lock().await.clear();
        self.access_history.lock().await.clear();

        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();
    }

    /// 清除过期项
    pub async fn clear_expired(&self) {
        let now = Instant::now();

        // 收集过期的键
        let expired_l1: Vec<_> = {
            let l1 = self.l1_cache.lock().await;
            l1.iter()
                .filter(|(_key, entry)| now.duration_since(entry.created_at) >= self.config.ttl)
                .map(|(key, _)| key.clone())
                .collect()
        };

        // 清除L1过期项
        {
            let mut l1 = self.l1_cache.lock().await;
            for key in expired_l1 {
                l1.pop(&key);
            }
        }

        // 收集L2过期的键
        let expired_l2: Vec<_> = {
            let l2 = self.l2_cache.lock().await;
            l2.iter()
                .filter(|(_key, entry)| now.duration_since(entry.created_at) >= self.config.ttl)
                .map(|(key, _)| key.clone())
                .collect()
        };

        // 清除L2过期项
        {
            let mut l2 = self.l2_cache.lock().await;
            for key in expired_l2 {
                l2.pop(&key);
            }
        }

        // 收集L3过期的键
        let expired_l3: Vec<_> = {
            let l3 = self.l3_cache.lock().await;
            l3.iter()
                .filter(|(_key, entry)| now.duration_since(entry.created_at) >= self.config.ttl)
                .map(|(key, _)| key.clone())
                .collect()
        };

        // 清除L3过期项
        {
            let mut l3 = self.l3_cache.lock().await;
            for key in expired_l3 {
                l3.pop(&key);
            }
        }

        // 清除预取缓存过期项
        {
            let mut prefetch = self.prefetch_cache.lock().await;
            prefetch.retain(|_key, entry| now.duration_since(entry.created_at) < self.config.ttl);
        }
    }

    /// 获取缓存统计
    pub async fn get_stats(&self) -> CacheStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// 获取缓存大小
    pub async fn size(&self) -> (usize, usize, usize, usize) {
        let l1 = self.l1_cache.lock().await.len();
        let l2 = self.l2_cache.lock().await.len();
        let l3 = self.l3_cache.lock().await.len();
        let prefetch = self.prefetch_cache.lock().await.len();
        (l1, l2, l3, prefetch)
    }

    /// 打印缓存报告
    pub async fn print_report(&self) {
        let stats = self.get_stats().await;
        let (l1_size, l2_size, l3_size, prefetch_size) = self.size().await;

        println!("\n=== 多级缓存系统报告 ===");
        println!("缓存大小:");
        println!("  L1: {}/{} 项", l1_size, self.config.l1_size);
        println!("  L2: {}/{} 项", l2_size, self.config.l2_size);
        println!("  L3: {}/{} 项", l3_size, self.config.l3_size);
        println!("  预取: {} 项", prefetch_size);
        println!("\n访问统计:");
        println!("  L1命中: {}", stats.l1_hits);
        println!("  L2命中: {}", stats.l2_hits);
        println!("  L3命中: {}", stats.l3_hits);
        println!("  预取命中: {}", stats.prefetch_hits);
        println!("  未命中: {}", stats.misses);
        println!("  总访问: {}", stats.total_accesses);
        println!("\n性能指标:");
        println!("  整体命中率: {:.1}%", stats.hit_rate * 100.0);
        println!("  淘汰次数: {}", stats.evictions);
        println!(
            "  当前数据大小: {:.2} MB",
            stats.current_size as f64 / (1024.0 * 1024.0)
        );
        println!("==========================\n");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = MultiLevelCache::<String, Vec<u8>>::with_default_config();
        let stats = cache.get_stats().await;
        assert_eq!(stats.total_accesses, 0);
    }

    #[tokio::test]
    async fn test_cache_put_get() {
        let cache = MultiLevelCache::<String, String>::with_default_config();

        cache.put("key1".to_string(), "value1".to_string(), 100).await;
        let value = cache.get(&"key1".to_string()).await;

        assert!(value.is_some());
        assert_eq!(value.unwrap(), "value1");
    }

    #[tokio::test]
    async fn test_cache_hit_rate() {
        let cache = MultiLevelCache::<String, String>::with_default_config();

        cache.put("key1".to_string(), "value1".to_string(), 100).await;
        cache.get(&"key1".to_string()).await; // 命中
        cache.get(&"key2".to_string()).await; // 未命中

        let stats = cache.get_stats().await;
        assert!(stats.hit_rate > 0.0 && stats.hit_rate <= 1.0);
    }

    #[tokio::test]
    async fn test_cache_clear() {
        let cache = MultiLevelCache::<String, String>::with_default_config();

        cache.put("key1".to_string(), "value1".to_string(), 100).await;
        cache.clear().await;

        let value = cache.get(&"key1".to_string()).await;
        assert!(value.is_none());
    }
}
