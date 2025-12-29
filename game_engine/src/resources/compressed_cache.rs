//! 压缩资源缓存
//!
//! 提供资源压缩和磁盘缓存功能，减少磁盘占用并提高加载速度。

use super::resource_trait::ResourceError;
use flate2::Compression;
use flate2::read::{GzDecoder, ZlibDecoder};
use flate2::write::{GzEncoder, ZlibEncoder};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::SystemTime;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// 压缩算法
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionAlgorithm {
    /// Gzip压缩
    Gzip {
        /// 压缩级别（0-9，9为最高压缩率）
        level: u32,
    },
    /// Zlib压缩
    Zlib {
        /// 压缩级别（0-9，9为最高压缩率）
        level: u32,
    },
    /// 无压缩（直接存储）
    None,
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        Self::Gzip { level: 6 }
    }
}

impl CompressionAlgorithm {
    /// 压缩数据
    pub fn compress(&self, data: &[u8]) -> Result<Vec<u8>, ResourceError> {
        match self {
            Self::Gzip { level } => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::new(*level));
                encoder.write_all(data)?;
                encoder.finish().map_err(|e| ResourceError::Other(e.to_string()))
            }
            Self::Zlib { level } => {
                let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(*level));
                encoder.write_all(data)?;
                encoder.finish().map_err(|e| ResourceError::Other(e.to_string()))
            }
            Self::None => Ok(data.to_vec()),
        }
    }

    /// 解压数据
    pub fn decompress(&self, compressed: &[u8]) -> Result<Vec<u8>, ResourceError> {
        match self {
            Self::Gzip { .. } => {
                let mut decoder = GzDecoder::new(compressed);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            Self::Zlib { .. } => {
                let mut decoder = ZlibDecoder::new(compressed);
                let mut decompressed = Vec::new();
                decoder.read_to_end(&mut decompressed)?;
                Ok(decompressed)
            }
            Self::None => Ok(compressed.to_vec()),
        }
    }

    /// 获取文件扩展名
    pub fn file_extension(&self) -> &'static str {
        match self {
            Self::Gzip { .. } => ".gz",
            Self::Zlib { .. } => ".zlib",
            Self::None => "",
        }
    }
}

/// 缓存条目元数据
#[derive(Debug, Clone)]
struct CacheEntry {
    /// 原始文件路径
    original_path: PathBuf,
    /// 压缩文件路径
    compressed_path: PathBuf,
    /// 压缩算法
    algorithm: CompressionAlgorithm,
    /// 原始大小
    original_size: usize,
    /// 压缩后大小
    compressed_size: usize,
    /// 创建时间
    created_at: SystemTime,
    /// 最后访问时间
    last_accessed: SystemTime,
}

/// 压缩资源缓存
///
/// 提供资源压缩和磁盘缓存功能。
pub struct CompressedResourceCache {
    /// 缓存目录
    cache_dir: PathBuf,
    /// 压缩算法
    algorithm: CompressionAlgorithm,
    /// 缓存条目映射
    entries: Arc<parking_lot::RwLock<HashMap<PathBuf, CacheEntry>>>,
    /// 最大缓存大小（字节），0表示无限制
    max_cache_size: usize,
    /// 当前缓存大小（字节）
    current_cache_size: Arc<std::sync::atomic::AtomicUsize>,
}

impl CompressedResourceCache {
    /// 创建新的压缩资源缓存
    ///
    /// # 参数
    /// - `cache_dir`: 缓存目录路径
    /// - `algorithm`: 压缩算法
    /// - `max_cache_size`: 最大缓存大小（字节），0表示无限制
    ///
    /// # 返回
    /// 新的压缩资源缓存实例
    pub fn new(
        cache_dir: impl AsRef<Path>,
        algorithm: CompressionAlgorithm,
        max_cache_size: usize,
    ) -> Result<Self, ResourceError> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // 创建缓存目录
        fs::create_dir_all(&cache_dir)?;

        Ok(Self {
            cache_dir,
            algorithm,
            entries: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            max_cache_size,
            current_cache_size: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }

    /// 使用默认配置创建压缩资源缓存
    ///
    /// # 参数
    /// - `cache_dir`: 缓存目录路径
    ///
    /// # 返回
    /// 新的压缩资源缓存实例
    pub fn with_default_config(cache_dir: impl AsRef<Path>) -> Result<Self, ResourceError> {
        Self::new(cache_dir, CompressionAlgorithm::default(), 0)
    }

    /// 获取或创建压缩缓存
    ///
    /// 如果缓存已存在且有效，则直接返回缓存路径。
    /// 否则，压缩资源并保存到缓存。
    ///
    /// # 参数
    /// - `resource_path`: 资源路径
    /// - `data`: 资源数据
    ///
    /// # 返回
    /// 压缩后的文件路径
    pub async fn get_or_create(
        &self,
        resource_path: impl AsRef<Path>,
        data: &[u8],
    ) -> Result<PathBuf, ResourceError> {
        let resource_path = resource_path.as_ref();
        let cache_key = self.get_cache_key(resource_path);

        // 检查缓存是否存在且有效
        {
            let entries = self.entries.read();
            if let Some(entry) = entries.get(&cache_key) {
                // 检查原始文件是否已修改
                if let Ok(metadata) = fs::metadata(resource_path)
                    && let Ok(modified) = metadata.modified()
                {
                    let created_at = entry.created_at;
                    let compressed_path = entry.compressed_path.clone();
                    drop(entries);
                    if modified <= created_at {
                        // 缓存有效，更新访问时间
                        let mut entries = self.entries.write();
                        if let Some(entry) = entries.get_mut(&cache_key) {
                            entry.last_accessed = SystemTime::now();
                        }
                        return Ok(compressed_path);
                    }
                }
            }
        }

        // 压缩数据
        let compressed = self.algorithm.compress(data)?;

        // 生成缓存文件路径
        let cache_file = self.cache_dir.join(format!(
            "{}{}",
            self.hash_path(resource_path),
            self.algorithm.file_extension()
        ));

        // 写入缓存文件
        let mut file = File::create(&cache_file).await?;
        file.write_all(&compressed).await?;
        file.sync_all().await?;

        // 更新缓存条目
        {
            let mut entries = self.entries.write();
            let old_size = entries.get(&cache_key).map(|e| e.compressed_size).unwrap_or(0);
            let new_size = compressed.len();

            let entry = CacheEntry {
                original_path: resource_path.to_path_buf(),
                compressed_path: cache_file.clone(),
                algorithm: self.algorithm,
                original_size: data.len(),
                compressed_size: new_size,
                created_at: SystemTime::now(),
                last_accessed: SystemTime::now(),
            };

            entries.insert(cache_key, entry);

            // 更新缓存大小
            let current = self.current_cache_size.load(std::sync::atomic::Ordering::Relaxed);
            let new_current = current + new_size - old_size;
            self.current_cache_size.store(new_current, std::sync::atomic::Ordering::Relaxed);

            // 检查是否需要清理缓存
            if self.max_cache_size > 0 && new_current > self.max_cache_size {
                drop(entries);
                self.cleanup_cache().await?;
            }
        }

        Ok(cache_file)
    }

    /// 从缓存加载资源
    ///
    /// # 参数
    /// - `resource_path`: 资源路径
    ///
    /// # 返回
    /// 解压后的资源数据
    pub async fn load(&self, resource_path: impl AsRef<Path>) -> Result<Vec<u8>, ResourceError> {
        let resource_path = resource_path.as_ref();
        let cache_key = self.get_cache_key(resource_path);

        // 查找缓存条目
        let compressed_path = {
            let entries = self.entries.read();
            entries
                .get(&cache_key)
                .map(|e| e.compressed_path.clone())
                .ok_or_else(|| ResourceError::NotFound(cache_key.display().to_string()))?
        };

        // 读取压缩文件
        let mut file = File::open(&compressed_path).await?;
        let mut compressed = Vec::new();
        file.read_to_end(&mut compressed).await?;

        // 解压数据
        let decompressed = self.algorithm.decompress(&compressed)?;

        // 更新访问时间
        {
            let mut entries = self.entries.write();
            if let Some(entry) = entries.get_mut(&cache_key) {
                entry.last_accessed = SystemTime::now();
            }
        }

        Ok(decompressed)
    }

    /// 检查缓存是否存在
    pub fn exists(&self, resource_path: impl AsRef<Path>) -> bool {
        let cache_key = self.get_cache_key(resource_path);
        let entries = self.entries.read();
        entries.contains_key(&cache_key)
    }

    /// 删除缓存条目
    pub async fn remove(&self, resource_path: impl AsRef<Path>) -> Result<(), ResourceError> {
        let resource_path = resource_path.as_ref();
        let cache_key = self.get_cache_key(resource_path);

        let compressed_path = {
            let mut entries = self.entries.write();
            if let Some(entry) = entries.remove(&cache_key) {
                let size = entry.compressed_size;
                self.current_cache_size.fetch_sub(size, std::sync::atomic::Ordering::Relaxed);
                entry.compressed_path
            } else {
                return Ok(());
            }
        };

        // 删除文件
        if compressed_path.exists() {
            fs::remove_file(&compressed_path)?;
        }

        Ok(())
    }

    /// 清空所有缓存
    pub async fn clear(&self) -> Result<(), ResourceError> {
        let paths: Vec<PathBuf> = {
            let mut entries = self.entries.write();
            let paths: Vec<PathBuf> = entries.values().map(|e| e.compressed_path.clone()).collect();
            entries.clear();
            self.current_cache_size.store(0, std::sync::atomic::Ordering::Relaxed);
            paths
        };

        // 删除所有缓存文件
        for path in paths {
            if path.exists() {
                let _ = fs::remove_file(&path);
            }
        }

        Ok(())
    }

    /// 获取缓存统计信息
    pub fn stats(&self) -> CompressedCacheStats {
        let entries = self.entries.read();
        let mut total_original_size = 0;
        let mut total_compressed_size = 0;

        for entry in entries.values() {
            total_original_size += entry.original_size;
            total_compressed_size += entry.compressed_size;
        }

        CompressedCacheStats {
            entry_count: entries.len(),
            total_original_size,
            total_compressed_size,
            current_cache_size: self.current_cache_size.load(std::sync::atomic::Ordering::Relaxed),
            max_cache_size: self.max_cache_size,
            compression_ratio: if total_original_size > 0 {
                total_compressed_size as f32 / total_original_size as f32
            } else {
                0.0
            },
        }
    }

    /// 清理缓存（删除最久未访问的条目）
    async fn cleanup_cache(&self) -> Result<(), ResourceError> {
        let mut entries = self.entries.write();

        // 按最后访问时间排序
        let mut sorted_entries: Vec<_> =
            entries.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        sorted_entries.sort_by_key(|(_, e)| e.last_accessed);

        // 删除最久未访问的条目，直到缓存大小在限制内
        while self.current_cache_size.load(std::sync::atomic::Ordering::Relaxed)
            > self.max_cache_size
            && !sorted_entries.is_empty()
        {
            let (key, entry) = sorted_entries.remove(0);
            entries.remove(&key);
            self.current_cache_size
                .fetch_sub(entry.compressed_size, std::sync::atomic::Ordering::Relaxed);

            // 删除文件
            if entry.compressed_path.exists() {
                let _ = fs::remove_file(&entry.compressed_path);
            }
        }

        Ok(())
    }

    /// 生成缓存键
    fn get_cache_key(&self, path: impl AsRef<Path>) -> PathBuf {
        path.as_ref().to_path_buf()
    }

    /// 哈希路径（用于生成缓存文件名）
    fn hash_path(&self, path: impl AsRef<Path>) -> String {
        use sha2::{Digest, Sha256};
        let path_str = path.as_ref().to_string_lossy();
        let mut hasher = Sha256::new();
        hasher.update(path_str.as_bytes());
        let hash = hasher.finalize();
        hex::encode(&hash[..16]) // 使用前16字节
    }
}

/// 压缩缓存统计信息
#[derive(Debug, Clone)]
pub struct CompressedCacheStats {
    /// 缓存条目数
    pub entry_count: usize,
    /// 总原始大小（字节）
    pub total_original_size: usize,
    /// 总压缩后大小（字节）
    pub total_compressed_size: usize,
    /// 当前缓存大小（字节）
    pub current_cache_size: usize,
    /// 最大缓存大小（字节）
    pub max_cache_size: usize,
    /// 压缩比（压缩后/原始）
    pub compression_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_compression_decompression() {
        let data = b"Hello, World! This is a test string for compression.";

        // 测试Gzip
        let gzip = CompressionAlgorithm::Gzip { level: 6 };
        let compressed = gzip.compress(data).expect("Test: operation should succeed");
        let decompressed = gzip.decompress(&compressed).expect("Test: operation should succeed");
        assert_eq!(decompressed, data);

        // 测试Zlib
        let zlib = CompressionAlgorithm::Zlib { level: 6 };
        let compressed = zlib.compress(data).expect("Test: operation should succeed");
        let decompressed = zlib.decompress(&compressed).expect("Test: operation should succeed");
        assert_eq!(decompressed, data);

        // 测试无压缩
        let none = CompressionAlgorithm::None;
        let compressed = none.compress(data).expect("Test: operation should succeed");
        let decompressed = none.decompress(&compressed).expect("Test: operation should succeed");
        assert_eq!(decompressed, data);
    }

    #[tokio::test]
    async fn test_cache_get_or_create() {
        let temp_dir = TempDir::new().expect("Test: operation should succeed");
        let cache = CompressedResourceCache::with_default_config(temp_dir.path()).expect("Test: operation should succeed");

        let resource_path = PathBuf::from("test_resource.txt");
        let data = b"Test resource data";

        // 创建缓存
        let cached_path = cache.get_or_create(&resource_path, data).await.expect("Test: operation should succeed");
        assert!(cached_path.exists());

        // 再次获取应该返回相同的路径
        let cached_path2 = cache.get_or_create(&resource_path, data).await.expect("Test: operation should succeed");
        assert_eq!(cached_path, cached_path2);
    }

    #[tokio::test]
    async fn test_cache_load() {
        let temp_dir = TempDir::new().expect("Test: operation should succeed");
        let cache = CompressedResourceCache::with_default_config(temp_dir.path()).expect("Test: operation should succeed");

        let resource_path = PathBuf::from("test_resource.txt");
        let original_data = b"Test resource data for loading";

        // 创建缓存
        cache.get_or_create(&resource_path, original_data).await.expect("Test: operation should succeed");

        // 从缓存加载
        let loaded_data = cache.load(&resource_path).await.expect("Test: operation should succeed");
        assert_eq!(loaded_data, original_data);
    }

    #[tokio::test]
    async fn test_cache_stats() {
        let temp_dir = TempDir::new().expect("Test: operation should succeed");
        let cache = CompressedResourceCache::with_default_config(temp_dir.path()).expect("Test: operation should succeed");

        let data = b"Test data";
        cache.get_or_create(PathBuf::from("test1.txt"), data).await.expect("Test: operation should succeed");
        cache.get_or_create(PathBuf::from("test2.txt"), data).await.expect("Test: operation should succeed");

        let stats = cache.stats();
        assert_eq!(stats.entry_count, 2);
        assert!(stats.compression_ratio > 0.0);
        assert!(stats.compression_ratio <= 1.0);
    }
}
