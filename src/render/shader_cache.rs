//! 着色器编译缓存系统
//!
//! 提供着色器二进制缓存功能，减少启动时间和重复编译开销。
//!
//! ## 设计原则
//!
//! 1. **缓存键生成**: 基于着色器源码的SHA256哈希
//! 2. **存储格式**: 文件系统缓存（跨平台路径管理）
//! 3. **失效策略**: 源码变更时自动失效（hash不匹配）
//! 4. **验证机制**: 缓存验证（hash匹配、格式兼容性检查）
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │              Shader Cache Architecture                  │
//! ├─────────────────────────────────────────────────────────┤
//! │  1. Cache Key Generation (SHA256 hash of source)        │
//! │     - Input: Shader source code                         │
//! │     - Output: 64-character hex string                   │
//! │                                                          │
//! │  2. Cache Storage (File System)                         │
//! │     - Path: {cache_dir}/shaders/{hash}.spv              │
//! │     - Format: Binary SPIR-V (or platform-specific)      │
//! │                                                          │
//! │  3. Cache Validation                                    │
//! │     - Hash verification                                 │
//! │     - Format compatibility check                        │
//! │     - Version check                                     │
//! │                                                          │
//! │  4. Cache Invalidation                                  │
//! │     - Source code changed (hash mismatch)               │
//! │     - Cache format version mismatch                      │
//! │     - Manual invalidation                                │
//! └─────────────────────────────────────────────────────────┘
//! ```

use crate::core::error::RenderError;
use crate::impl_default;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// 缓存元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheMetadata {
    /// 着色器源码哈希（用于验证）
    source_hash: String,
    /// 缓存格式版本
    format_version: u32,
    /// 创建时间戳
    created_at: u64,
    /// 最后访问时间戳
    last_accessed: u64,
    /// 编译选项哈希（用于区分不同编译选项）
    compile_options_hash: String,
}

impl CacheMetadata {
    /// 创建新的缓存元数据
    fn new(source_hash: String, compile_options_hash: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            source_hash,
            format_version: CACHE_FORMAT_VERSION,
            created_at: now,
            last_accessed: now,
            compile_options_hash,
        }
    }

    /// 更新最后访问时间
    fn update_access_time(&mut self) {
        self.last_accessed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }
}

/// 缓存格式版本（用于兼容性检查）
const CACHE_FORMAT_VERSION: u32 = 1;

/// 着色器缓存键
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShaderCacheKey {
    /// 源码哈希（64字符hex字符串）
    source_hash: String,
    /// 编译选项哈希
    compile_options_hash: String,
}

impl ShaderCacheKey {
    /// 从着色器源码生成缓存键
    pub fn from_source(source: &str, compile_options: &str) -> Self {
        let source_hash = Self::hash_string(source);
        let compile_options_hash = Self::hash_string(compile_options);

        Self {
            source_hash,
            compile_options_hash,
        }
    }

    /// 计算字符串的SHA256哈希
    fn hash_string(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let hash = hasher.finalize();
        hex::encode(hash)
    }

    /// 获取缓存文件名
    pub fn cache_filename(&self) -> String {
        // 组合两个哈希的前32字符
        let combined = format!(
            "{}_{}",
            &self.source_hash[..32],
            &self.compile_options_hash[..32]
        );
        format!("{}.spv", combined)
    }

    /// 获取元数据文件名
    pub fn metadata_filename(&self) -> String {
        format!("{}.meta", self.cache_filename())
    }
}

/// 着色器缓存统计
#[derive(Debug, Clone, Default)]
pub struct ShaderCacheStats {
    /// 缓存命中次数
    pub hits: u64,
    /// 缓存未命中次数
    pub misses: u64,
    /// 缓存失效次数
    pub invalidations: u64,
    /// 当前缓存大小（字节）
    pub cache_size_bytes: u64,
    /// 缓存文件数量
    pub cache_file_count: usize,
}

impl ShaderCacheStats {
    /// 计算命中率
    pub fn hit_rate(&self) -> f32 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f32 / total as f32
        }
    }
}

/// 着色器缓存配置
#[derive(Debug, Clone)]
pub struct ShaderCacheConfig {
    /// 缓存目录路径
    pub cache_dir: PathBuf,
    /// 最大缓存大小（字节）
    pub max_cache_size: u64,
    /// 是否启用缓存
    pub enabled: bool,
    /// 缓存清理策略
    pub cleanup_strategy: CleanupStrategy,
}

impl Default for ShaderCacheConfig {
    fn default() -> Self {
        Self {
            cache_dir: Self::default_cache_dir(),
            max_cache_size: 100 * 1024 * 1024, // 100 MB
            enabled: true,
            cleanup_strategy: CleanupStrategy::LRU,
        }
    }
}

impl ShaderCacheConfig {
    /// 获取默认缓存目录
    fn default_cache_dir() -> PathBuf {
        // 使用平台特定的缓存目录
        if let Some(cache_dir) = dirs::cache_dir() {
            cache_dir.join("game_engine").join("shader_cache")
        } else {
            // 回退到临时目录
            std::env::temp_dir().join("game_engine_shader_cache")
        }
    }
}

/// 缓存清理策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CleanupStrategy {
    /// LRU（最近最少使用）
    LRU,
    /// FIFO（先进先出）
    FIFO,
    /// 基于大小（删除最大的文件）
    SizeBased,
}

/// 着色器缓存管理器
pub struct ShaderCache {
    config: ShaderCacheConfig,
    stats: ShaderCacheStats,
}

impl ShaderCache {
    /// 创建新的着色器缓存
    pub fn new(config: ShaderCacheConfig) -> Result<Self, RenderError> {
        // 确保缓存目录存在
        if config.enabled {
            fs::create_dir_all(&config.cache_dir).map_err(|e| {
                RenderError::InvalidState(format!("Failed to create shader cache directory: {}", e))
            })?;
        }

        let mut cache = Self {
            config,
            stats: ShaderCacheStats::default(),
        };

        // 初始化统计信息
        cache.update_stats();

        Ok(cache)
    }

    /// 使用默认配置创建着色器缓存
    ///
    /// 这是 `new(ShaderCacheConfig::default())` 的便捷方法。
    ///
    /// # 返回
    ///
    /// 返回使用默认配置创建的着色器缓存实例。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::render::shader_cache::ShaderCache;
    ///
    /// let cache = ShaderCache::with_default_config()?;
    /// ```
    pub fn with_default_config() -> Result<Self, RenderError> {
        Self::new(ShaderCacheConfig::default())
    }

    /// 获取缓存的着色器二进制数据
    ///
    /// 如果缓存命中，返回缓存的二进制数据（SPIR-V格式）
    /// 如果缓存未命中或失效，返回None
    ///
    /// 注意：当前实现缓存WGSL源码hash用于验证，实际二进制缓存需要naga支持
    pub fn get(&mut self, key: &ShaderCacheKey) -> Result<Option<Vec<u8>>, RenderError> {
        if !self.config.enabled {
            self.stats.misses += 1;
            return Ok(None);
        }

        let cache_path = self.config.cache_dir.join(key.cache_filename());
        let metadata_path = self.config.cache_dir.join(key.metadata_filename());

        // 检查缓存文件是否存在
        if !cache_path.exists() || !metadata_path.exists() {
            self.stats.misses += 1;
            return Ok(None);
        }

        // 读取并验证元数据
        let metadata: CacheMetadata = match self.load_metadata(&metadata_path) {
            Ok(m) => m,
            Err(_) => {
                // 元数据损坏，删除缓存
                let _ = fs::remove_file(&cache_path);
                let _ = fs::remove_file(&metadata_path);
                self.stats.invalidations += 1;
                self.stats.misses += 1;
                return Ok(None);
            }
        };

        // 验证哈希匹配
        if metadata.source_hash != key.source_hash
            || metadata.compile_options_hash != key.compile_options_hash
        {
            // 源码或编译选项已更改，缓存失效
            let _ = fs::remove_file(&cache_path);
            let _ = fs::remove_file(&metadata_path);
            self.stats.invalidations += 1;
            self.stats.misses += 1;
            return Ok(None);
        }

        // 验证格式版本
        if metadata.format_version != CACHE_FORMAT_VERSION {
            // 格式版本不匹配，删除旧缓存
            let _ = fs::remove_file(&cache_path);
            let _ = fs::remove_file(&metadata_path);
            self.stats.invalidations += 1;
            self.stats.misses += 1;
            return Ok(None);
        }

        // 读取缓存文件
        // 注意：当前缓存的是WGSL源码（用于验证），未来可以缓存SPIR-V二进制
        let cached_data = fs::read(&cache_path).map_err(|e| {
            RenderError::InvalidState(format!("Failed to read shader cache file: {}", e))
        })?;

        // 更新访问时间
        let mut updated_metadata = metadata;
        updated_metadata.update_access_time();
        if let Err(e) = self.save_metadata(&metadata_path, &updated_metadata) {
            tracing::warn!(
                target: "render",
                "Failed to update cache metadata access time: {}",
                e
            );
        }

        self.stats.hits += 1;

        // 注意：当前实现返回缓存的WGSL源码
        // 未来优化：集成naga库编译WGSL到SPIR-V二进制，提升加载性能
        // 相关任务：需要评估naga集成方案和性能收益
        Ok(Some(cached_data))
    }

    /// 存储着色器源码到缓存（用于验证）
    ///
    /// 注意：当前实现存储WGSL源码，未来可以存储SPIR-V二进制
    pub fn put_source(
        &mut self,
        key: &ShaderCacheKey,
        source_code: &str,
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_path = self.config.cache_dir.join(key.cache_filename());
        let metadata_path = self.config.cache_dir.join(key.metadata_filename());

        // 写入源码（当前实现）
        // 注意：当前实现存储WGSL源码，未来可以编译为SPIR-V并缓存二进制
        // 未来优化：集成naga库，编译WGSL到SPIR-V，减少运行时编译开销
        fs::write(&cache_path, source_code.as_bytes()).map_err(|e| {
            RenderError::InvalidState(format!("Failed to write shader cache file: {}", e))
        })?;

        // 写入元数据
        let metadata =
            CacheMetadata::new(key.source_hash.clone(), key.compile_options_hash.clone());
        self.save_metadata(&metadata_path, &metadata)?;

        // 更新统计
        self.update_stats();

        // 检查是否需要清理缓存
        if self.stats.cache_size_bytes > self.config.max_cache_size {
            self.cleanup()?;
        }

        Ok(())
    }

    /// 存储着色器二进制数据到缓存（SPIR-V格式）
    ///
    /// 未来实现：当naga集成后，使用此方法缓存SPIR-V二进制
    pub fn put_binary(
        &mut self,
        key: &ShaderCacheKey,
        binary_data: &[u8],
    ) -> Result<(), RenderError> {
        if !self.config.enabled {
            return Ok(());
        }

        let cache_path = self.config.cache_dir.join(key.cache_filename());
        let metadata_path = self.config.cache_dir.join(key.metadata_filename());

        // 写入二进制数据
        fs::write(&cache_path, binary_data).map_err(|e| {
            RenderError::InvalidState(format!("Failed to write shader cache file: {}", e))
        })?;

        // 写入元数据
        let metadata =
            CacheMetadata::new(key.source_hash.clone(), key.compile_options_hash.clone());
        self.save_metadata(&metadata_path, &metadata)?;

        // 更新统计
        self.update_stats();

        // 检查是否需要清理缓存
        if self.stats.cache_size_bytes > self.config.max_cache_size {
            self.cleanup()?;
        }

        Ok(())
    }

    /// 加载元数据
    fn load_metadata(&self, path: &Path) -> Result<CacheMetadata, RenderError> {
        let data = fs::read(path)
            .map_err(|e| RenderError::InvalidState(format!("Failed to read metadata: {}", e)))?;

        let metadata: CacheMetadata = toml::from_slice(&data)
            .map_err(|e| RenderError::InvalidState(format!("Failed to parse metadata: {}", e)))?;

        Ok(metadata)
    }

    /// 保存元数据
    fn save_metadata(&self, path: &Path, metadata: &CacheMetadata) -> Result<(), RenderError> {
        let data = toml::to_string(metadata).map_err(|e| {
            RenderError::InvalidState(format!("Failed to serialize metadata: {}", e))
        })?;

        fs::write(path, data)
            .map_err(|e| RenderError::InvalidState(format!("Failed to write metadata: {}", e)))?;

        Ok(())
    }

    /// 更新统计信息
    fn update_stats(&mut self) {
        if !self.config.cache_dir.exists() {
            self.stats.cache_size_bytes = 0;
            self.stats.cache_file_count = 0;
            return;
        }

        let mut total_size = 0u64;
        let mut file_count = 0usize;

        if let Ok(entries) = fs::read_dir(&self.config.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        if let Some(ext) = entry.path().extension() {
                            if ext == "spv" {
                                total_size += metadata.len();
                                file_count += 1;
                            }
                        }
                    }
                }
            }
        }

        self.stats.cache_size_bytes = total_size;
        self.stats.cache_file_count = file_count;
    }

    /// 清理缓存（根据清理策略）
    fn cleanup(&mut self) -> Result<(), RenderError> {
        match self.config.cleanup_strategy {
            CleanupStrategy::LRU => self.cleanup_lru(),
            CleanupStrategy::FIFO => self.cleanup_fifo(),
            CleanupStrategy::SizeBased => self.cleanup_size_based(),
        }
    }

    /// LRU清理：删除最近最少使用的缓存
    fn cleanup_lru(&mut self) -> Result<(), RenderError> {
        // 收集所有缓存文件及其访问时间
        let mut cache_files: Vec<(PathBuf, u64)> = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.config.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                    if let Ok(metadata) = self.load_metadata(&path) {
                        // 获取对应的缓存文件路径
                        if let Some(cache_file) = path.parent().and_then(|p| {
                            path.file_stem().and_then(|stem| {
                                p.join(format!("{}.spv", stem.to_string_lossy()))
                                    .canonicalize()
                                    .ok()
                            })
                        }) {
                            cache_files.push((cache_file, metadata.last_accessed));
                        }
                    }
                }
            }
        }

        // 按访问时间排序（最早的在前）
        cache_files.sort_by_key(|(_, access_time)| *access_time);

        // 删除最旧的文件，直到缓存大小低于限制
        let target_size = self.config.max_cache_size * 8 / 10; // 清理到80%
        let mut current_size = self.stats.cache_size_bytes;

        for (cache_path, _) in cache_files {
            if current_size <= target_size {
                break;
            }

            if let Ok(metadata) = fs::metadata(&cache_path) {
                let file_size = metadata.len();
                if fs::remove_file(&cache_path).is_ok() {
                    // 删除对应的元数据文件
                    if let Some(meta_path) = cache_path.parent() {
                        let meta_filename =
                            format!("{}.meta", cache_path.file_stem().unwrap().to_string_lossy());
                        let _ = fs::remove_file(meta_path.join(meta_filename));
                    }
                    current_size -= file_size;
                }
            }
        }

        self.update_stats();
        Ok(())
    }

    /// FIFO清理：删除最旧的文件
    fn cleanup_fifo(&mut self) -> Result<(), RenderError> {
        // 类似LRU，但按创建时间排序
        let mut cache_files: Vec<(PathBuf, u64)> = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.config.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("meta") {
                    if let Ok(metadata) = self.load_metadata(&path) {
                        if let Some(cache_file) = path.parent().and_then(|p| {
                            path.file_stem().and_then(|stem| {
                                p.join(format!("{}.spv", stem.to_string_lossy()))
                                    .canonicalize()
                                    .ok()
                            })
                        }) {
                            cache_files.push((cache_file, metadata.created_at));
                        }
                    }
                }
            }
        }

        cache_files.sort_by_key(|(_, created_at)| *created_at);

        let target_size = self.config.max_cache_size * 8 / 10;
        let mut current_size = self.stats.cache_size_bytes;

        for (cache_path, _) in cache_files {
            if current_size <= target_size {
                break;
            }

            if let Ok(metadata) = fs::metadata(&cache_path) {
                let file_size = metadata.len();
                if fs::remove_file(&cache_path).is_ok() {
                    if let Some(meta_path) = cache_path.parent() {
                        let meta_filename =
                            format!("{}.meta", cache_path.file_stem().unwrap().to_string_lossy());
                        let _ = fs::remove_file(meta_path.join(meta_filename));
                    }
                    current_size -= file_size;
                }
            }
        }

        self.update_stats();
        Ok(())
    }

    /// 基于大小的清理：删除最大的文件
    fn cleanup_size_based(&mut self) -> Result<(), RenderError> {
        let mut cache_files: Vec<(PathBuf, u64)> = Vec::new();

        if let Ok(entries) = fs::read_dir(&self.config.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("spv") {
                    if let Ok(metadata) = fs::metadata(&path) {
                        cache_files.push((path, metadata.len()));
                    }
                }
            }
        }

        // 按文件大小排序（最大的在前）
        cache_files.sort_by_key(|(_, size)| std::cmp::Reverse(*size));

        let target_size = self.config.max_cache_size * 8 / 10;
        let mut current_size = self.stats.cache_size_bytes;

        for (cache_path, file_size) in cache_files {
            if current_size <= target_size {
                break;
            }

            if fs::remove_file(&cache_path).is_ok() {
                if let Some(meta_path) = cache_path.parent() {
                    let meta_filename =
                        format!("{}.meta", cache_path.file_stem().unwrap().to_string_lossy());
                    let _ = fs::remove_file(meta_path.join(meta_filename));
                }
                current_size -= file_size;
            }
        }

        self.update_stats();
        Ok(())
    }

    /// 清除所有缓存
    pub fn clear(&mut self) -> Result<(), RenderError> {
        if !self.config.cache_dir.exists() {
            return Ok(());
        }

        if let Ok(entries) = fs::read_dir(&self.config.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let _ = fs::remove_file(&path);
                }
            }
        }

        self.update_stats();
        Ok(())
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> &ShaderCacheStats {
        &self.stats
    }

    /// 获取缓存配置
    pub fn config(&self) -> &ShaderCacheConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_cache_key_generation() {
        let source1 = "fn main() {}";
        let source2 = "fn main() {}";
        let source3 = "fn main() { }"; // 不同内容

        let key1 = ShaderCacheKey::from_source(source1, "");
        let key2 = ShaderCacheKey::from_source(source2, "");
        let key3 = ShaderCacheKey::from_source(source3, "");

        // 相同源码应该生成相同的键
        assert_eq!(key1.source_hash, key2.source_hash);

        // 不同源码应该生成不同的键
        assert_ne!(key1.source_hash, key3.source_hash);
    }

    #[test]
    fn test_cache_key_filename() {
        let key = ShaderCacheKey::from_source("test", "");
        let filename = key.cache_filename();

        // 文件名应该以.spv结尾
        assert!(filename.ends_with(".spv"));
        assert_eq!(filename.len(), 65); // 64字符 + ".spv"
    }

    #[test]
    fn test_shader_cache_basic() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();

        let config = ShaderCacheConfig {
            cache_dir: cache_dir.clone(),
            max_cache_size: 1024 * 1024,
            enabled: true,
            cleanup_strategy: CleanupStrategy::LRU,
        };

        let mut cache = ShaderCache::new(config).unwrap();

        // 创建缓存键
        let key = ShaderCacheKey::from_source("fn main() {}", "");

        // 首次获取应该未命中
        let result = cache.get(&key).unwrap();
        assert!(result.is_none());
        assert_eq!(cache.stats().misses, 1);

        // 存储缓存
        let source_code = "fn main() {}";
        cache.put_source(&key, source_code).unwrap();

        // 再次获取应该命中
        let result = cache.get(&key).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), source_code.as_bytes());
        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_cache_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();

        let config = ShaderCacheConfig {
            cache_dir,
            max_cache_size: 1024 * 1024,
            enabled: true,
            cleanup_strategy: CleanupStrategy::LRU,
        };

        let mut cache = ShaderCache::new(config).unwrap();

        let key1 = ShaderCacheKey::from_source("fn main() {}", "");
        let source_code = "fn main() {}";

        // 存储缓存
        cache.put_source(&key1, source_code).unwrap();

        // 使用不同的源码创建新键（应该失效）
        let key2 = ShaderCacheKey::from_source("fn main() { }", "");

        // 使用key2获取应该未命中（因为hash不同）
        let result = cache.get(&key2).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().to_path_buf();

        let config = ShaderCacheConfig {
            cache_dir,
            max_cache_size: 1024 * 1024,
            enabled: true,
            cleanup_strategy: CleanupStrategy::LRU,
        };

        let mut cache = ShaderCache::new(config).unwrap();

        let key = ShaderCacheKey::from_source("test", "");

        // 存储一些数据
        cache.put_source(&key, "test").unwrap();

        let stats = cache.stats();
        assert!(stats.cache_size_bytes > 0);
        assert!(stats.cache_file_count > 0);
    }
}
