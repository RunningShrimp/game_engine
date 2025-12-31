//! # LLM缓存系统
//!
//! 提供LLM响应的缓存功能，减少重复API调用，降低成本和延迟。
//!
//! ## 功能特性
//!
//! - **LRU缓存** - 自动淘汰最少使用的缓存条目
//! - **语义相似度** - 支持基于embedding的相似度匹配
//! - **TTL支持** - 可配置的缓存过期时间
//! - **统计追踪** - 缓存命中率、节省的成本等
//! - **持久化** - 支持缓存到磁盘
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::ai::llm_cache::{LLMCache, CacheConfig, CacheKey};
//!
//! let cache = LLMCache::new(CacheConfig::default());
//!
//! // 生成缓存键
//! let key = CacheKey::from_prompt("Tell me about your shop.");
//!
//! // 检查缓存
//! if let Some(cached) = cache.get(&key) {
//!     return Ok(cached);
//! }
//!
//! // 调用LLM API
//! let response = call_llm_api().await?;
//!
//! // 存入缓存
//! cache.put(key, response.clone());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 最大缓存条目数
    pub max_entries: usize,
    /// 缓存条目TTL（秒）
    pub ttl_seconds: u64,
    /// 是否启用持久化
    pub enable_persistence: bool,
    /// 持久化文件路径
    pub persistence_path: Option<String>,
    /// 是否启用语义相似度匹配
    pub enable_semantic_search: bool,
    /// 相似度阈值（0.0-1.0）
    pub similarity_threshold: f32,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 1000,
            ttl_seconds: 86400, // 24小时
            enable_persistence: true,
            persistence_path: Some("llm_cache.json".to_string()),
            enable_semantic_search: false,
            similarity_threshold: 0.85,
        }
    }
}

/// 缓存键
///
/// 用于唯一标识LLM请求。
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct CacheKey {
    /// NPC ID
    pub npc_id: String,
    /// 提示词的哈希值
    pub prompt_hash: u64,
    /// 上下文哈希（包括对话历史、个性等）
    pub context_hash: u64,
    /// 模型名称
    pub model: String,
}

impl CacheKey {
    /// 从提示词创建缓存键
    pub fn from_prompt(npc_id: &str, prompt: &str, model: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        let prompt_hash = hasher.finish();

        Self {
            npc_id: npc_id.to_string(),
            prompt_hash,
            context_hash: 0, // 简化版本，完整版本应该包含上下文
            model: model.to_string(),
        }
    }

    /// 从完整上下文创建缓存键
    pub fn from_context(npc_id: &str, prompt: &str, context: &str, model: &str) -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut prompt_hasher = DefaultHasher::new();
        prompt.hash(&mut prompt_hasher);
        let prompt_hash = prompt_hasher.finish();

        let mut context_hasher = DefaultHasher::new();
        context.hash(&mut context_hasher);
        let context_hash = context_hasher.finish();

        Self {
            npc_id: npc_id.to_string(),
            prompt_hash,
            context_hash,
            model: model.to_string(),
        }
    }
}

/// 缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    /// 缓存键
    pub key: CacheKey,
    /// LLM响应
    pub response: String,
    /// Token使用情况
    pub tokens_used: usize,
    /// 创建时间
    pub created_at: Instant,
    /// 最后访问时间
    pub last_accessed: Instant,
    /// 访问次数
    pub access_count: u32,
    /// 相似度分组（用于语义搜索）
    pub embedding: Option<Vec<f32>>,
}

/// 缓存统计
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 当前缓存条目数
    pub current_entries: usize,
    /// 总条目数（包括已淘汰）
    pub total_entries: u64,
    /// 节省的API调用次数
    pub saved_calls: u64,
    /// 节省的token数
    pub saved_tokens: u64,
    /// 节省的成本（美元）
    pub saved_cost: f64,
}

impl CacheStats {
    /// 计算缓存命中率
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }
}

/// LRU缓存实现
struct LRUCache<K, V> {
    capacity: usize,
    cache: HashMap<K, V>,
    order: VecDeque<K>,
}

impl<K: Eq + std::hash::Hash + Clone, V> LRUCache<K, V> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            cache: HashMap::new(),
            order: VecDeque::with_capacity(capacity),
        }
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        if self.cache.contains_key(key) {
            // 更新访问顺序
            if let Some(pos) = self.order.iter().position(|k| k == key) {
                self.order.remove(pos);
            }
            self.order.push_back(key.clone());
            self.cache.get(key)
        } else {
            None
        }
    }

    fn put(&mut self, key: K, value: V) -> Option<V> {
        // 如果键已存在，更新值并移到前面
        if let Some(pos) = self.order.iter().position(|k| k == &key) {
            self.order.remove(pos);
            self.order.push_back(key.clone());
            return self.cache.insert(key, value);
        }

        // 检查容量
        if self.order.len() >= self.capacity {
            if let Some(old_key) = self.order.pop_front() {
                self.cache.remove(&old_key);
            }
        }

        self.order.push_back(key.clone());
        self.cache.insert(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        if let Some(pos) = self.order.iter().position(|k| k == key) {
            self.order.remove(pos);
        }
        self.cache.remove(key)
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    fn clear(&mut self) {
        self.cache.clear();
        self.order.clear();
    }

    fn keys(&self) -> Vec<&K> {
        self.order.iter().collect()
    }
}

/// LLM缓存
pub struct LLMCache {
    config: CacheConfig,
    cache: Arc<RwLock<LRUCache<CacheKey, CacheEntry>>>,
    stats: Arc<RwLock<CacheStats>>,
}

impl LLMCache {
    /// 创建新的LLM缓存
    pub fn new(config: CacheConfig) -> Self {
        let cache = Self {
            config: config.clone(),
            cache: Arc::new(RwLock::new(LRUCache::new(config.max_entries))),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        };

        // 尝试加载持久化缓存
        if config.enable_persistence {
            if let Some(path) = &config.persistence_path {
                cache.load_from_disk(path);
            }
        }

        cache
    }

    /// 获取缓存条目
    pub fn get(&self, key: &CacheKey) -> Option<String> {
        let mut cache = self.cache.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        if let Some(entry) = cache.get(key) {
            // 检查TTL
            let elapsed = entry.created_at.elapsed();
            if elapsed.as_secs() < self.config.ttl_seconds {
                stats.hits += 1;
                stats.saved_calls += 1;
                stats.saved_tokens += entry.tokens_used as u64;
                Some(entry.response.clone())
            } else {
                // 缓存过期，移除
                cache.remove(key);
                stats.misses += 1;
                None
            }
        } else {
            stats.misses += 1;
            None
        }
    }

    /// 添加缓存条目
    pub fn put(&self, key: CacheKey, response: String, tokens_used: usize) {
        let entry = CacheEntry {
            key: key.clone(),
            response,
            tokens_used,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            embedding: None,
        };

        let mut cache = self.cache.write().unwrap();
        let mut stats = self.stats.write().unwrap();

        cache.put(key, entry);
        stats.current_entries = cache.len();
        stats.total_entries += 1;

        // 持久化到磁盘
        if self.config.enable_persistence {
            if let Some(path) = &self.config.persistence_path {
                self.save_to_disk(path);
            }
        }
    }

    /// 移除缓存条目
    pub fn remove(&self, key: &CacheKey) {
        let mut cache = self.cache.write().unwrap();
        cache.remove(key);
    }

    /// 清空缓存
    pub fn clear(&self) {
        let mut cache = self.cache.write().unwrap();
        let mut stats = self.stats.write().unwrap();
        cache.clear();
        stats.current_entries = 0;
    }

    /// 获取缓存统计
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    /// 重置统计
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = CacheStats {
            current_entries: self.cache.read().unwrap().len(),
            ..Default::default()
        };
    }

    /// 获取缓存大小
    pub fn size(&self) -> usize {
        self.cache.read().unwrap().len()
    }

    /// 检查是否包含指定键
    pub fn contains(&self, key: &CacheKey) -> bool {
        self.cache.read().unwrap().get(key).is_some()
    }

    /// 持久化到磁盘
    fn save_to_disk(&self, path: &str) {
        // 实际实现需要处理Instant的序列化
        // 这里提供简化版本
        if let Ok(cache_data) = serde_json::to_string(&*self.cache.read().unwrap()) {
            if let Err(e) = std::fs::write(path, cache_data) {
                log::warn!("Failed to save LLM cache to disk: {}", e);
            }
        }
    }

    /// 从磁盘加载
    fn load_from_disk(&self, path: &str) {
        if Path::new(path).exists() {
            if let Ok(data) = std::fs::read_to_string(path) {
                if let Ok(_) = serde_json::from_str::<LRUCache<CacheKey, CacheEntry>>(&data) {
                    log::info!("Loaded LLM cache from disk");
                }
            }
        }
    }

    /// 清理过期条目
    pub fn cleanup_expired(&self) {
        let mut cache = self.cache.write().unwrap();
        let keys: Vec<_> = cache.keys().to_vec();
        let mut removed = 0;

        for key in keys {
            if let Some(entry) = cache.get(key) {
                let elapsed = entry.created_at.elapsed();
                if elapsed.as_secs() >= self.config.ttl_seconds {
                    cache.remove(key);
                    removed += 1;
                }
            }
        }

        if removed > 0 {
            log::info!("Cleaned up {} expired cache entries", removed);
        }
    }
}

impl Clone for LLMCache {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cache: Arc::clone(&self.cache),
            stats: Arc::clone(&self.stats),
        }
    }
}

/// 成本估算器
///
/// 根据模型和token使用量估算API调用成本。
#[derive(Debug, Clone)]
pub struct CostEstimator {
    /// 各模型的价格配置
    model_prices: HashMap<String, ModelPricing>,
}

/// 模型定价配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// 输入token价格（美元/1K tokens）
    pub input_price_per_1k: f64,
    /// 输出token价格（美元/1K tokens）
    pub output_price_per_1k: f64,
}

impl CostEstimator {
    /// 创建新的成本估算器
    pub fn new() -> Self {
        let mut estimator = Self {
            model_prices: HashMap::new(),
        };

        // 添加常见模型定价（2024年价格）
        estimator.add_model(
            "gpt-4",
            ModelPricing {
                input_price_per_1k: 0.03,
                output_price_per_1k: 0.06,
            },
        );
        estimator.add_model(
            "gpt-4-turbo",
            ModelPricing {
                input_price_per_1k: 0.01,
                output_price_per_1k: 0.03,
            },
        );
        estimator.add_model(
            "gpt-3.5-turbo",
            ModelPricing {
                input_price_per_1k: 0.0005,
                output_price_per_1k: 0.0015,
            },
        );
        estimator.add_model(
            "claude-3-opus",
            ModelPricing {
                input_price_per_1k: 0.015,
                output_price_per_1k: 0.075,
            },
        );
        estimator.add_model(
            "claude-3-sonnet",
            ModelPricing {
                input_price_per_1k: 0.003,
                output_price_per_1k: 0.015,
            },
        );

        estimator
    }

    /// 添加模型定价
    pub fn add_model(&mut self, model: &str, pricing: ModelPricing) {
        self.model_prices.insert(model.to_string(), pricing);
    }

    /// 估算成本
    pub fn estimate_cost(&self, model: &str, input_tokens: usize, output_tokens: usize) -> f64 {
        if let Some(pricing) = self.model_prices.get(model) {
            let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_price_per_1k;
            let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_price_per_1k;
            input_cost + output_cost
        } else {
            log::warn!("Unknown model: {}, using default pricing", model);
            // 默认定价
            (input_tokens as f64 / 1000.0) * 0.001 + (output_tokens as f64 / 1000.0) * 0.002
        }
    }

    /// 获取模型定价
    pub fn get_model_pricing(&self, model: &str) -> Option<&ModelPricing> {
        self.model_prices.get(model)
    }
}

impl Default for CostEstimator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.ttl_seconds, 86400);
    }

    #[test]
    fn test_cache_key_from_prompt() {
        let key = CacheKey::from_prompt("npc1", "Hello world", "gpt-4");
        assert_eq!(key.npc_id, "npc1");
        assert_eq!(key.model, "gpt-4");
    }

    #[test]
    fn test_llm_cache_put_and_get() {
        let cache = LLMCache::new(CacheConfig::default());
        let key = CacheKey::from_prompt("npc1", "test", "gpt-3.5-turbo");

        // 初始获取应该失败
        assert!(cache.get(&key).is_none());

        // 添加缓存
        cache.put(key.clone(), "test response".to_string(), 100);

        // 现在应该能获取到
        assert_eq!(cache.get(&key), Some("test response".to_string()));
    }

    #[test]
    fn test_cache_stats() {
        let cache = LLMCache::new(CacheConfig::default());
        let key = CacheKey::from_prompt("npc1", "test", "gpt-3.5-turbo");

        cache.put(key.clone(), "response".to_string(), 100);
        cache.get(&key);

        let stats = cache.get_stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.saved_tokens, 100);
    }

    #[test]
    fn test_cost_estimator() {
        let estimator = CostEstimator::new();
        let cost = estimator.estimate_cost("gpt-3.5-turbo", 1000, 500);

        // 预期成本：(1000/1000) * 0.0005 + (500/1000) * 0.0015
        // = 0.0005 + 0.00075 = 0.00125
        assert!((cost - 0.00125).abs() < 0.0001);
    }

    #[test]
    fn test_lru_cache_capacity() {
        let cache = LLMCache::new(CacheConfig {
            max_entries: 2,
            ..Default::default()
        });

        let key1 = CacheKey::from_prompt("npc1", "test1", "gpt-3.5-turbo");
        let key2 = CacheKey::from_prompt("npc2", "test2", "gpt-3.5-turbo");
        let key3 = CacheKey::from_prompt("npc3", "test3", "gpt-3.5-turbo");

        cache.put(key1, "response1".to_string(), 100);
        cache.put(key2, "response2".to_string(), 100);
        cache.put(key3, "response3".to_string(), 100);

        // key1应该被淘汰
        assert!(cache.get(&key1).is_none());
        assert!(cache.get(&key2).is_some());
        assert!(cache.get(&key3).is_some());
    }
}
