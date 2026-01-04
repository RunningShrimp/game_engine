//! C# 编译缓存管理器
//!
//! 提供编译结果的持久化缓存，避免重复编译相同的代码。
//!
//! **特性:**
//! - 基于源代码哈希的缓存键
//! - 持久化缓存（跨会话保持）
//! - LRU缓存淘汰策略
//! - 缓存命中率统计
//!
//! **性能提升:**
//! - 首次编译：~500ms
//! - 缓存命中：<1ms
//! - 预期加速比：500x

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// 编译缓存条目
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// 源代码哈希
    hash: String,
    /// 编译的DLL路径
    dll_path: PathBuf,
    /// 编译时间戳
    compiled_at: u64,
    /// 访问次数
    access_count: u64,
    /// 最后访问时间
    last_accessed: u64,
    /// 脚本名称
    script_name: String,
}

/// 编译缓存管理器
#[derive(Debug, Clone)]
pub struct CompileCache {
    /// 缓存目录
    cache_dir: PathBuf,
    /// 缓存条目（哈希 -> 条目）
    entries: Arc<Mutex<HashMap<String, CacheEntry>>>,
    /// 最大缓存大小（MB）
    max_cache_size_mb: usize,
    /// 缓存统计
    stats: Arc<Mutex<CacheStats>>,
}

/// 缓存统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 编译次数
    pub compiles: u64,
    /// 缓存淘汰次数
    pub evictions: u64,
}

impl CompileCache {
    /// 创建新的编译缓存
    ///
    /// **参数:**
    /// - `cache_dir`: 缓存目录路径
    /// - `max_cache_size_mb`: 最大缓存大小（MB）
    ///
    /// **示例:**
    /// ```ignore
    /// let cache = CompileCache::new(
    ///     PathBuf::from("./cache/csharp"),
    ///     100  // 100MB
    /// )?;
    /// ```
    pub fn new(cache_dir: PathBuf, max_cache_size_mb: usize) -> Result<Self, String> {
        // 创建缓存目录
        fs::create_dir_all(&cache_dir)
            .map_err(|e| format!("Failed to create cache directory: {e}"))?;

        let cache = Self {
            cache_dir: cache_dir.clone(),
            entries: Arc::new(Mutex::new(HashMap::new())),
            max_cache_size_mb,
            stats: Arc::new(Mutex::new(CacheStats::default())),
        };

        // 加载现有缓存索引
        cache.load_index()?;

        tracing::info!("C# compile cache initialized at: {}", cache_dir.display());
        tracing::info!("Max cache size: {} MB", max_cache_size_mb);

        Ok(cache)
    }

    /// 计算源代码的SHA256哈希
    fn compute_hash(code: &str, script_name: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(code.as_bytes());
        hasher.update(script_name.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 查找缓存的编译结果
    ///
    /// **参数:**
    /// - `code`: C# 源代码
    /// - `script_name`: 脚本名称
    ///
    /// **返回:** 缓存的DLL路径（如果存在）
    pub fn get(&self, code: &str, script_name: &str) -> Option<PathBuf> {
        let hash = Self::compute_hash(code, script_name);
        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        // First, check if the entry exists and get the DLL path
        let dll_path_opt = entries
            .get(&hash)
            .map(|entry| (entry.dll_path.clone(), entry.dll_path.exists()));

        if let Some((dll_path, exists)) = dll_path_opt {
            if exists {
                // DLL exists - update access statistics
                if let Some(entry) = entries.get_mut(&hash) {
                    entry.access_count += 1;
                    entry.last_accessed = current_timestamp();
                }
                stats.hits += 1;

                tracing::debug!(
                    "Cache hit for script: {} (hash: {})",
                    script_name,
                    &hash[..8]
                );
                Some(dll_path)
            } else {
                // DLL文件丢失，移除条目
                entries.remove(&hash);
                tracing::warn!(
                    "Cached DLL missing, removing from cache: {}",
                    dll_path.display()
                );
                stats.misses += 1;
                tracing::debug!("Cache miss for script: {}", script_name);
                None
            }
        } else {
            stats.misses += 1;
            tracing::debug!("Cache miss for script: {}", script_name);
            None
        }
    }

    /// 插入编译结果到缓存
    ///
    /// **参数:**
    /// - `code`: C# 源代码
    /// - `script_name`: 脚本名称
    /// - `dll_path`: 编译的DLL路径
    pub fn insert(&self, code: &str, script_name: &str, dll_path: PathBuf) -> Result<(), String> {
        let hash = Self::compute_hash(code, script_name);
        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        // 创建缓存条目
        let entry = CacheEntry {
            hash: hash.clone(),
            dll_path: dll_path.clone(),
            compiled_at: current_timestamp(),
            access_count: 0,
            last_accessed: current_timestamp(),
            script_name: script_name.to_string(),
        };

        // 插入缓存
        entries.insert(hash.clone(), entry);
        stats.compiles += 1;

        tracing::debug!(
            "Cached compiled script: {} (hash: {})",
            script_name,
            &hash[..8]
        );

        // 检查缓存大小，必要时淘汰
        self.check_and_evict(&mut entries, &mut stats)?;

        // 保存缓存索引
        self.save_index()?;

        Ok(())
    }

    /// 清除所有缓存
    pub fn clear(&self) -> Result<(), String> {
        let mut entries = self.entries.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        // 删除所有缓存的DLL
        for entry in entries.values() {
            if entry.dll_path.exists() {
                fs::remove_file(&entry.dll_path)
                    .map_err(|e| format!("Failed to remove cached DLL: {e}"))?;
            }
        }

        // 清空条目
        entries.clear();

        // 重置统计
        *stats = CacheStats::default();

        // 保存索引
        self.save_index()?;

        tracing::info!("Cleared all compile cache");

        Ok(())
    }

    /// 获取缓存统计
    pub fn get_stats(&self) -> CacheStats {
        let stats = self.stats.lock().unwrap();
        stats.clone()
    }

    /// 计算缓存命中率
    pub fn get_hit_rate(&self) -> f64 {
        let stats = self.stats.lock().unwrap();
        let total = stats.hits + stats.misses;
        if total == 0 {
            0.0
        } else {
            stats.hits as f64 / total as f64
        }
    }

    /// 获取缓存目录路径
    pub fn get_cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// 获取缓存大小（字节）
    fn get_cache_size(&self) -> Result<u64, String> {
        let entries = self.entries.lock().unwrap();
        let mut total_size = 0u64;

        for entry in entries.values() {
            if entry.dll_path.exists() {
                let metadata = fs::metadata(&entry.dll_path)
                    .map_err(|e| format!("Failed to get DLL metadata: {e}"))?;
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    /// 检查并淘汰缓存条目
    fn check_and_evict(
        &self,
        entries: &mut HashMap<String, CacheEntry>,
        stats: &mut CacheStats,
    ) -> Result<(), String> {
        let max_size_bytes = self.max_cache_size_mb * 1024 * 1024;
        let current_size = self.get_cache_size()?;

        if current_size > max_size_bytes as u64 {
            tracing::info!("Cache size limit exceeded, evicting old entries...");

            // 按最后访问时间排序（LRU）
            let mut sorted_entries: Vec<_> = entries.iter().collect();
            sorted_entries.sort_by_key(|(_, e)| e.last_accessed);

            // 收集需要淘汰的条目信息
            let mut to_evict = Vec::new();
            for (hash, entry) in sorted_entries {
                if self.get_cache_size()? <= max_size_bytes as u64 {
                    break;
                }

                // 保存需要删除的DLL路径和脚本名称
                to_evict.push((
                    hash.clone(),
                    entry.dll_path.clone(),
                    entry.script_name.clone(),
                    entry.hash.clone(),
                ));
            }

            // 执行删除操作
            for (hash, dll_path, script_name, entry_hash) in to_evict {
                // 删除DLL文件
                if dll_path.exists() {
                    fs::remove_file(&dll_path).map_err(|e| format!("Failed to evict DLL: {e}"))?;
                }

                // 移除条目
                entries.remove(&hash);
                stats.evictions += 1;

                tracing::debug!(
                    "Evicted cache entry: {} (hash: {})",
                    script_name,
                    &entry_hash[..8]
                );
            }
        }

        Ok(())
    }

    /// 保存缓存索引到磁盘
    fn save_index(&self) -> Result<(), String> {
        let index_path = self.cache_dir.join("cache_index.json");
        let entries = self.entries.lock().unwrap();
        let stats = self.stats.lock().unwrap();

        let index_data = serde_json::json!({
            "entries": entries.values().collect::<Vec<_>>(),
            "stats": *stats,
        });

        fs::write(
            &index_path,
            serde_json::to_string_pretty(&index_data)
                .map_err(|e| format!("Failed to serialize cache index: {e}"))?,
        )
        .map_err(|e| format!("Failed to write cache index: {e}"))?;

        Ok(())
    }

    /// 从磁盘加载缓存索引
    fn load_index(&self) -> Result<(), String> {
        let index_path = self.cache_dir.join("cache_index.json");

        if !index_path.exists() {
            tracing::info!("No existing cache index found");
            return Ok(());
        }

        let index_data = fs::read_to_string(&index_path)
            .map_err(|e| format!("Failed to read cache index: {e}"))?;

        let index: serde_json::Value = serde_json::from_str(&index_data)
            .map_err(|e| format!("Failed to parse cache index: {e}"))?;

        // 加载条目
        if let Some(entries_array) = index.get("entries").and_then(|v| v.as_array()) {
            let mut entries = self.entries.lock().unwrap();
            for entry_value in entries_array {
                if let Ok(entry) = serde_json::from_value::<CacheEntry>(entry_value.clone()) {
                    // 只保留仍然存在的DLL
                    if entry.dll_path.exists() {
                        entries.insert(entry.hash.clone(), entry);
                    }
                }
            }
        }

        // 加载统计
        if let Some(stats_value) = index.get("stats") {
            if let Ok(stats) = serde_json::from_value::<CacheStats>(stats_value.clone()) {
                let mut cached_stats = self.stats.lock().unwrap();
                *cached_stats = stats;
            }
        }

        let entries = self.entries.lock().unwrap();
        tracing::info!("Loaded {} cached entries from disk", entries.len());

        Ok(())
    }
}

/// 获取当前时间戳（秒）
fn current_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "csharp")]
    fn test_cache_hash_computation() {
        let code1 = "public class Test { }";
        let code2 = "public class Test { }";
        let code3 = "public class Different { }";

        let hash1 = CompileCache::compute_hash(code1, "test");
        let hash2 = CompileCache::compute_hash(code2, "test");
        let hash3 = CompileCache::compute_hash(code3, "test");

        // 相同代码应该产生相同哈希
        assert_eq!(hash1, hash2);
        // 不同代码应该产生不同哈希
        assert_ne!(hash1, hash3);
    }

    #[test]
    #[cfg(feature = "csharp")]
    fn test_cache_stats() {
        let cache = CompileCache::new(std::env::temp_dir().join("test_cache"), 10).unwrap();

        let stats = cache.get_stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        let hit_rate = cache.get_hit_rate();
        assert_eq!(hit_rate, 0.0);
    }
}
