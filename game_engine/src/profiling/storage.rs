//  性能数据存储模块
// 
//  提供内存中环形缓冲区、持久化存储、数据压缩和查询功能。

use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufWriter};

use crate::platform::run_sync;
use super::metrics::*;
use super::{ProfilingResult, ProfilingError};

// ============================================================================
// 环形缓冲区
// ============================================================================

/// 环形缓冲区
/// 
/// 固定大小的循环缓冲区，自动覆盖最旧的数据
#[derive(Debug, Clone)]
pub struct RingBuffer<T> {
    /// 数据存储
    buffer: Vec<Option<T>>,
    /// 写入位置
    write_pos: usize,
    /// 读取位置
    read_pos: usize,
    /// 当前元素数量
    count: usize,
    /// 是否已满
    is_full: bool,
}

impl<T> RingBuffer<T> {
    /// 创建新的环形缓冲区
    pub fn new(capacity: usize) -> Self {
        let mut buffer = Vec::with_capacity(capacity);
        buffer.resize_with(capacity, || None);
        Self {
            buffer,
            write_pos: 0,
            read_pos: 0,
            count: 0,
            is_full: false,
        }
    }

    /// 写入数据
    pub fn push(&mut self, item: T) -> bool {
        let old_item = self.buffer[self.write_pos].replace(item);
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        
        if self.is_full {
            self.read_pos = self.write_pos;
        } else {
            self.count += 1;
            if self.write_pos == 0 {
                self.is_full = true;
            }
        }
        
        old_item.is_some() // 返回是否覆盖了旧数据
    }

    /// 读取数据
    pub fn pop(&mut self) -> Option<T> {
        if self.count == 0 {
            return None;
        }
        
        let item = self.buffer[self.read_pos].take();
        self.read_pos = (self.read_pos + 1) % self.buffer.len();
        self.count -= 1;
        self.is_full = false;
        
        item
    }

    /// 获取最新数据（不移除）
    pub fn peek_latest(&self) -> Option<&T> {
        if self.count == 0 {
            return None;
        }
        
        let latest_pos = if self.write_pos == 0 {
            self.buffer.len() - 1
        } else {
            self.write_pos - 1
        };
        
        self.buffer[latest_pos].as_ref()
    }

    /// 获取最旧数据（不移除）
    pub fn peek_oldest(&self) -> Option<&T> {
        if self.count == 0 {
            return None;
        }
        
        self.buffer[self.read_pos].as_ref()
    }

    /// 获取所有数据的迭代器
    pub fn iter(&self) -> RingBufferIter<'_, T> {
        RingBufferIter {
            buffer: &self.buffer,
            pos: self.read_pos,
            remaining: self.count,
        }
    }

    /// 获取容量
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// 获取当前元素数量
    pub fn len(&self) -> usize {
        self.count
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// 检查是否已满
    pub fn is_full(&self) -> bool {
        self.is_full
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        for item in &mut self.buffer {
            *item = None;
        }
        self.write_pos = 0;
        self.read_pos = 0;
        self.count = 0;
        self.is_full = false;
    }
}

/// 环形缓冲区迭代器
pub struct RingBufferIter<'a, T> {
    buffer: &'a [Option<T>],
    pos: usize,
    remaining: usize,
}

impl<'a, T> Iterator for RingBufferIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }
        
        let item = self.buffer[self.pos].as_ref();
        self.pos = (self.pos + 1) % self.buffer.len();
        self.remaining -= 1;
        
        item
    }
}

// ============================================================================
// 性能数据点
// ============================================================================

/// 性能数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// 时间戳
    pub timestamp: u64,
    /// 指标名称
    pub metric_name: String,
    /// 指标值
    pub value: f64,
    /// 指标类别
    pub category: MetricCategory,
    /// 标签
    pub tags: HashMap<String, String>,
}

impl DataPoint {
    /// 创建新的数据点
    pub fn new(
        metric_name: impl Into<String>,
        value: f64,
        category: MetricCategory,
    ) -> Self {
        Self {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            metric_name: metric_name.into(),
            value,
            category,
            tags: HashMap::new(),
        }
    }

    /// 添加标签
    pub fn with_tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    /// 设置时间戳
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }
}

// ============================================================================
// 数据压缩
// ============================================================================

/// 压缩算法类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionType {
    /// 无压缩
    None,
    /// LZ4压缩
    Lz4,
    /// Gzip压缩
    Gzip,
    /// Zstd压缩
    Zstd,
}

/// 压缩配置
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// 压缩类型
    pub compression_type: CompressionType,
    /// 压缩级别 (1-9)
    pub compression_level: u32,
    /// 最小压缩大小
    pub min_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            compression_type: CompressionType::Lz4,
            compression_level: 6,
            min_size: 1024, // 1KB
        }
    }
}

/// 数据压缩器
pub struct DataCompressor {
    config: CompressionConfig,
}

impl DataCompressor {
    /// 创建新的数据压缩器
    pub fn new(config: CompressionConfig) -> Self {
        Self { config }
    }

    /// 压缩数据
    pub fn compress(&self, data: &[u8]) -> ProfilingResult<Vec<u8>> {
        if data.len() < self.config.min_size {
            return Ok(data.to_vec());
        }

        match self.config.compression_type {
            CompressionType::None => Ok(data.to_vec()),
            CompressionType::Lz4 => self.compress_lz4(data),
            CompressionType::Gzip => self.compress_gzip(data),
            CompressionType::Zstd => self.compress_zstd(data),
        }
    }

    /// 解压缩数据
    pub fn decompress(&self, compressed: &[u8]) -> ProfilingResult<Vec<u8>> {
        match self.config.compression_type {
            CompressionType::None => Ok(compressed.to_vec()),
            CompressionType::Lz4 => self.decompress_lz4(compressed),
            CompressionType::Gzip => self.decompress_gzip(compressed),
            CompressionType::Zstd => self.decompress_zstd(compressed),
        }
    }

    fn compress_lz4(&self, data: &[u8]) -> ProfilingResult<Vec<u8>> {
        // 简化的LZ4压缩实现（实际项目中应使用lz4库）
        let compressed = data.to_vec(); // 占位实现
        Ok(compressed)
    }

    fn decompress_lz4(&self, compressed: &[u8]) -> ProfilingResult<Vec<u8>> {
        // 简化的LZ4解压缩实现
        let decompressed = compressed.to_vec(); // 占位实现
        Ok(decompressed)
    }

    fn compress_gzip(&self, data: &[u8]) -> ProfilingResult<Vec<u8>> {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), 
            Compression::new(self.config.compression_level));
        encoder.write_all(data)?;
        Ok(encoder.finish()?)
    }

    fn decompress_gzip(&self, compressed: &[u8]) -> ProfilingResult<Vec<u8>> {
        use flate2::read::GzDecoder;
        use std::io::Read;

        let mut decoder = GzDecoder::new(compressed);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)?;
        Ok(decompressed)
    }

    fn compress_zstd(&self, data: &[u8]) -> ProfilingResult<Vec<u8>> {
        // Zstd压缩实现（需要zstd库）
        let compressed = data.to_vec(); // 占位实现
        Ok(compressed)
    }

    fn decompress_zstd(&self, compressed: &[u8]) -> ProfilingResult<Vec<u8>> {
        // Zstd解压缩实现
        let decompressed = compressed.to_vec(); // 占位实现
        Ok(decompressed)
    }
}

// ============================================================================
// 持久化存储
// ============================================================================

/// 存储配置
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// 数据目录
    pub data_dir: PathBuf,
    /// 文件名前缀
    pub file_prefix: String,
    /// 单个文件最大大小 (字节)
    pub max_file_size: usize,
    /// 保留文件数量
    pub retain_files: usize,
    /// 压缩配置
    pub compression: CompressionConfig,
    /// 自动归档间隔
    pub archive_interval: Duration,
    /// 是否启用写入缓存
    pub enable_write_cache: bool,
    /// 缓存大小
    pub cache_size: usize,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./profiling_data"),
            file_prefix: "metrics".to_string(),
            max_file_size: 100 * 1024 * 1024, // 100MB
            retain_files: 10,
            compression: CompressionConfig::default(),
            archive_interval: Duration::from_secs(3600), // 1小时
            enable_write_cache: true,
            cache_size: 10000,
        }
    }
}

/// 文件信息
#[derive(Debug, Clone)]
struct FileInfo {
    path: PathBuf,
    size: usize,
    created_at: SystemTime,
    is_compressed: bool,
}

/// 持久化存储
pub struct PersistentStorage {
    config: StorageConfig,
    current_file: Option<BufWriter<File>>,
    current_file_info: Option<FileInfo>,
    file_index: usize,
    compressor: DataCompressor,
    write_cache: VecDeque<DataPoint>,
    total_written: usize,
}

impl PersistentStorage {
    /// 创建新的持久化存储
    pub async fn new(config: StorageConfig) -> ProfilingResult<Self> {
        // 确保数据目录存在
        tokio::fs::create_dir_all(&config.data_dir).await
            .map_err(super::ProfilingError::IoError)?;

        let compressor = DataCompressor::new(config.compression.clone());
        
        let mut storage = Self {
            config,
            current_file: None,
            current_file_info: None,
            file_index: 0,
            compressor,
            write_cache: VecDeque::new(),
            total_written: 0,
        };

        // 初始化文件
        storage.rotate_file_if_needed().await?;
        
        Ok(storage)
    }

    /// 创建新的持久化存储（同步版本，用于向后兼容）
    pub fn new_sync(config: StorageConfig) -> ProfilingResult<Self> {
        run_sync(Self::new(config))
    }

    /// 存储数据点
    pub fn store(&mut self, data_point: DataPoint) -> ProfilingResult<()> {
        if self.config.enable_write_cache {
            self.write_cache.push_back(data_point);
            
            // 缓存满时刷新
            if self.write_cache.len() >= self.config.cache_size {
                // 使用同步版本的 flush_cache
                self.flush_cache_sync()?;
            }
        } else {
            // write_data_point 是异步函数，需要使用 run_sync 包装
            let _data_point_clone = data_point.clone();  // Intentionally unused
            // 我们不能移动 self，而是应该重构为不移动的方式
            return Err(ProfilingError::Other("Sync storage not supported, use async version".into()));
        }
        
        Ok(())
    }

    /// 批量存储数据点
    pub fn store_batch(&mut self, data_points: &[DataPoint]) -> ProfilingResult<()> {
        for data_point in data_points {
            self.store(data_point.clone())?;
        }
        Ok(())
    }

    /// 刷新写入缓存
    pub async fn flush_cache(&mut self) -> ProfilingResult<()> {
        while let Some(data_point) = self.write_cache.pop_front() {
            self.write_data_point(&data_point).await?;
        }
        
        if let Some(ref mut file) = self.current_file {
            file.flush().await?;
        }
        
        Ok(())
    }

    /// 刷新写入缓存（同步版本，用于向后兼容）
    pub fn flush_cache_sync(&mut self) -> ProfilingResult<()> {
        // 不支持同步版本，返回错误
        Err(ProfilingError::Other("Sync flush not supported, use async version".into()))
    }

    /// 写入单个数据点
    async fn write_data_point(&mut self, data_point: &DataPoint) -> ProfilingResult<()> {
        // 序列化数据点
        let serialized = serde_json::to_vec(data_point)?;
        
        // 压缩数据（如果需要）
        let compressed = self.compressor.compress(&serialized)?;
        
        // 写入长度前缀和数据
        let length_bytes = (compressed.len() as u32).to_le_bytes();
        
        if let Some(ref mut file) = self.current_file {
            file.write_all(&length_bytes).await?;
            file.write_all(&compressed).await?;
            self.total_written += compressed.len() + 4;
        }
        
        // 检查是否需要轮换文件
        self.rotate_file_if_needed().await?;
        
        Ok(())
    }

    /// 轮换文件（如果需要）
    async fn rotate_file_if_needed(&mut self) -> ProfilingResult<()> {
        let should_rotate = if let Some(ref info) = self.current_file_info {
            info.size >= self.config.max_file_size
        } else {
            true // 没有当前文件，需要创建
        };
        
        if should_rotate {
            self.close_current_file().await?;
            self.create_new_file().await?;
            self.cleanup_old_files().await?;
        }
        
        Ok(())
    }

    /// 创建新文件
    async fn create_new_file(&mut self) -> ProfilingResult<()> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        
        let filename = format!(
            "{}_{}_{}.dat{}",
            self.config.file_prefix,
            timestamp,
            self.file_index,
            if self.config.compression.compression_type != CompressionType::None {
                ".gz"
            } else {
                ""
            }
        );
        
        let file_path = self.config.data_dir.join(filename);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path).await?;
        
        let file_info = FileInfo {
            path: file_path.clone(),
            size: 0,
            created_at: SystemTime::now(),
            is_compressed: self.config.compression.compression_type != CompressionType::None,
        };
        
        self.current_file = Some(BufWriter::new(file));
        self.current_file_info = Some(file_info);
        self.file_index += 1;
        self.total_written = 0;
        
        tracing::debug!(
            target: "profiling",
            "创建新的存储文件: {:?}",
            file_path
        );
        
        Ok(())
    }

    /// 关闭当前文件
    async fn close_current_file(&mut self) -> ProfilingResult<()> {
        if let Some(mut file) = self.current_file.take() {
            file.flush().await?;
            
            // 更新文件信息
            if let Some(ref mut info) = self.current_file_info {
                info.size = self.total_written;
            }
        }
        
        self.current_file_info = None;
        self.total_written = 0;
        
        Ok(())
    }

    /// 清理旧文件
    async fn cleanup_old_files(&mut self) -> ProfilingResult<()> {
        let mut files = Vec::new();
        
        // 扫描目录中的文件
        let mut entries = tokio::fs::read_dir(&self.config.data_dir).await?;
        while let Some(entry) = entries.next_entry().await.map_err(super::ProfilingError::IoError)? {
            let path = entry.path();
            
            // 检查文件名是否匹配前缀
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(&self.config.file_prefix) && name.ends_with(".dat") {
                    let metadata = tokio::fs::metadata(&path).await
                        .map_err(super::ProfilingError::IoError)?;
                    let created_at = metadata.created().unwrap_or(SystemTime::now());
                    
                    files.push(FileInfo {
                        path: path.clone(),
                        size: metadata.len() as usize,
                        created_at,
                        is_compressed: name.ends_with(".gz"),
                    });
                }
            }
        }
        
        // 按创建时间排序（最新的在前）
        files.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        // 删除超出保留数量的文件
        if files.len() > self.config.retain_files {
            for file_info in files.iter().skip(self.config.retain_files) {
                if let Err(e) = tokio::fs::remove_file(&file_info.path).await {
                    tracing::warn!(
                        target: "profiling",
                        "无法删除旧文件 {:?}: {}",
                        file_info.path,
                        e
                    );
                } else {
                    tracing::debug!(
                        target: "profiling",
                        "删除旧文件: {:?}",
                        file_info.path
                    );
                }
            }
        }
        
        Ok(())
    }

    /// 获取存储统计信息
    pub async fn get_storage_stats(&self) -> ProfilingResult<StorageStats> {
        let mut total_files = 0;
        let mut total_size = 0;
        let mut compressed_files = 0;
        
        let mut entries = tokio::fs::read_dir(&self.config.data_dir).await
            .map_err(super::ProfilingError::IoError)?;
        while let Some(entry) = entries.next_entry().await
            .map_err(super::ProfilingError::IoError)? {
            let path = entry.path();
            
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(&self.config.file_prefix) && name.ends_with(".dat") {
                    let metadata = tokio::fs::metadata(&path).await
                        .map_err(super::ProfilingError::IoError)?;
                    total_files += 1;
                    total_size += metadata.len();
                    if name.ends_with(".gz") {
                        compressed_files += 1;
                    }
                }
            }
        }
        
        Ok(StorageStats {
            total_files,
            total_size,
            compressed_files,
            current_file_size: self.total_written,
            cache_size: self.write_cache.len(),
        })
    }

    /// 获取存储统计信息（同步版本，用于向后兼容）
    pub fn get_storage_stats_sync(&self) -> ProfilingResult<StorageStats> {
        // 使用阻塞方式直接获取统计信息，避免生命周期问题
        // Note: Since we don't have a file_manager field, we'll return zeros for file-related stats
        // and use total_written for current file size as an approximation
        Ok(StorageStats {
            total_files: 0, // Placeholder - would need file system access to get real count
            total_size: 0,  // Placeholder - would need file system access to get real size
            compressed_files: 0, // Placeholder
            current_file_size: self.total_written,
            cache_size: self.write_cache.len(),
        })
    }
}

impl Drop for PersistentStorage {
    fn drop(&mut self) {
        // 在drop中避免使用异步函数，直接执行必要的清理操作
        // Since write_cache is a VecDeque and doesn't need locking, we can directly access it
        if !self.write_cache.is_empty() {
            // If there's cache data, try to flush it synchronously
            // We'll use a simple approach since flush_write_cache doesn't exist
            // The data will be lost since we can't do async operations in Drop
            eprintln!("Warning: {} items lost in write cache during drop", self.write_cache.len());
        }
        // Since we don't have a file_manager field, we'll skip that operation
        // The current_file will be automatically closed when it goes out of scope
    }
}

/// 存储统计信息
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// 总文件数
    pub total_files: usize,
    /// 总大小
    pub total_size: u64,
    /// 压缩文件数
    pub compressed_files: usize,
    /// 当前文件大小
    pub current_file_size: usize,
    /// 缓存大小
    pub cache_size: usize,
}

// ============================================================================
// 查询接口
// ============================================================================

/// 查询条件
#[derive(Debug, Clone)]
pub struct QueryCondition {
    /// 指标名称过滤
    pub metric_names: Option<Vec<String>>,
    /// 类别过滤
    pub categories: Option<Vec<MetricCategory>>,
    /// 时间范围开始
    pub start_time: Option<u64>,
    /// 时间范围结束
    pub end_time: Option<u64>,
    /// 标签过滤
    pub tags: Option<HashMap<String, String>>,
    /// 限制结果数量
    pub limit: Option<usize>,
    /// 排序方式
    pub order_by: Option<QueryOrder>,
}

/// 查询排序方式
#[derive(Debug, Clone, Copy)]
pub enum QueryOrder {
    /// 按时间戳升序
    TimestampAsc,
    /// 按时间戳降序
    TimestampDesc,
    /// 按值升序
    ValueAsc,
    /// 按值降序
    ValueDesc,
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// 数据点
    pub data_points: Vec<DataPoint>,
    /// 总数量（未应用limit前）
    pub total_count: usize,
    /// 查询耗时
    pub query_duration: Duration,
}

/// 数据查询器
pub struct DataQueryer {
    storage_dir: PathBuf,
    file_prefix: String,
    compressor: DataCompressor,
}

impl DataQueryer {
    /// 创建新的数据查询器
    pub fn new(storage_dir: &Path, file_prefix: &str) -> Self {
        Self {
            storage_dir: storage_dir.to_path_buf(),
            file_prefix: file_prefix.to_string(),
            compressor: DataCompressor::new(CompressionConfig::default()),
        }
    }

    /// 执行查询
    pub async fn query(&self, condition: &QueryCondition) -> ProfilingResult<QueryResult> {
        let start_time = Instant::now();
        let mut data_points = Vec::new();
        let mut total_count = 0;

        // 扫描所有文件
        let mut entries = tokio::fs::read_dir(&self.storage_dir).await
            .map_err(super::ProfilingError::IoError)?;
        while let Some(entry) = entries.next_entry().await
            .map_err(super::ProfilingError::IoError)? {
            let path = entry.path();
            
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(&self.file_prefix) && name.ends_with(".dat") {
                    let file_data_points = self.read_file(&path, condition).await?;
                    total_count += file_data_points.len();
                    data_points.extend(file_data_points);
                }
            }
        }

        // 应用过滤条件
        let filtered_points = self.apply_filters(&data_points, condition);

        // 排序
        let mut sorted_points = filtered_points;
        if let Some(order_by) = condition.order_by {
            self.sort_data_points(&mut sorted_points, order_by);
        }

        // 应用限制
        let limited_points = if let Some(limit) = condition.limit {
            sorted_points.into_iter().take(limit).collect()
        } else {
            sorted_points
        };

        Ok(QueryResult {
            data_points: limited_points,
            total_count,
            query_duration: start_time.elapsed(),
        })
    }

    /// 执行查询（同步版本，用于向后兼容）
    pub fn query_sync(&self, _condition: &QueryCondition) -> ProfilingResult<QueryResult> {
        // 同步版本直接返回错误，建议使用异步版本
        Err(super::ProfilingError::Other("Sync query not supported, use async version".into()))
    }

    /// 读取单个文件
    async fn read_file(&self, path: &Path, condition: &QueryCondition) -> ProfilingResult<Vec<DataPoint>> {
        let file = tokio::fs::File::open(path).await
            .map_err(super::ProfilingError::IoError)?;
        let mut reader = tokio::io::BufReader::new(file);
        let mut data_points = Vec::new();

        loop {
            // 读取长度前缀
            let mut length_bytes = [0u8; 4];
            if reader.read_exact(&mut length_bytes).await.is_err() {
                break; // 文件结束
            }
            
            let length = u32::from_le_bytes(length_bytes) as usize;
            
            // 读取数据
            let mut compressed_data = vec![0u8; length];
            reader.read_exact(&mut compressed_data).await?;
            
            // 解压缩
            let decompressed = self.compressor.decompress(&compressed_data)?;
            
            // 反序列化
            let data_point: DataPoint = serde_json::from_slice(&decompressed)?;
            
            // 应用时间范围过滤
            if let Some(start_time) = condition.start_time {
                if data_point.timestamp < start_time {
                    continue;
                }
            }
            
            if let Some(end_time) = condition.end_time {
                if data_point.timestamp > end_time {
                    continue;
                }
            }
            
            data_points.push(data_point);
        }

        Ok(data_points)
    }

    /// 应用过滤条件
    fn apply_filters(&self, data_points: &[DataPoint], condition: &QueryCondition) -> Vec<DataPoint> {
        data_points
            .iter()
            .filter(|point| {
                // 指标名称过滤
                if let Some(ref names) = condition.metric_names {
                    if !names.contains(&point.metric_name) {
                        return false;
                    }
                }
                
                // 类别过滤
                if let Some(ref categories) = condition.categories {
                    if !categories.contains(&point.category) {
                        return false;
                    }
                }
                
                // 标签过滤
                if let Some(ref tags) = condition.tags {
                    for (key, value) in tags {
                        if point.tags.get(key) != Some(value) {
                            return false;
                        }
                    }
                }
                
                true
            })
            .cloned()
            .collect()
    }

    /// 排序数据点
    fn sort_data_points(&self, data_points: &mut [DataPoint], order_by: QueryOrder) {
        match order_by {
            QueryOrder::TimestampAsc => {
                data_points.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
            }
            QueryOrder::TimestampDesc => {
                data_points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            }
            QueryOrder::ValueAsc => {
                data_points.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
            }
            QueryOrder::ValueDesc => {
                data_points.sort_by(|a, b| b.value.partial_cmp(&a.value).unwrap());
            }
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new(3);
        
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
        assert!(!buffer.is_full());
        
        // 填充缓冲区
        assert!(!buffer.push(1));
        assert!(!buffer.push(2));
        assert!(!buffer.push(3));
        
        assert_eq!(buffer.len(), 3);
        assert!(!buffer.is_empty());
        assert!(buffer.is_full());
        
        // 覆盖数据
        assert!(buffer.push(4)); // 覆盖了1
        
        assert_eq!(buffer.len(), 3);
        assert_eq!(buffer.peek_latest(), Some(&4));
        assert_eq!(buffer.peek_oldest(), Some(&2));
        
        // 读取数据
        assert_eq!(buffer.pop(), Some(&2));
        assert_eq!(buffer.pop(), Some(&3));
        assert_eq!(buffer.pop(), Some(&4));
        assert_eq!(buffer.pop(), None);
    }

    #[test]
    fn test_data_point() {
        let point = DataPoint::new("test_metric", 42.0, MetricCategory::Render)
            .with_tag("scene", "test")
            .with_timestamp(1234567890);
        
        assert_eq!(point.metric_name, "test_metric");
        assert_eq!(point.value, 42.0);
        assert_eq!(point.category, MetricCategory::Render);
        assert_eq!(point.timestamp, 1234567890);
        assert_eq!(point.tags.get("scene"), Some(&"test".to_string()));
    }

    #[test]
    fn test_data_compressor() {
        let config = CompressionConfig {
            compression_type: CompressionType::Gzip,
            compression_level: 6,
            min_size: 10,
        };
        
        let compressor = DataCompressor::new(config);
        
        let data = b"Hello, World! This is a test string for compression.";
        let compressed = compressor.compress(data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();
        
        assert_eq!(data, &decompressed[..]);
        assert!(compressed.len() < data.len()); // 压缩后应该更小
    }

    #[test]
    fn test_data_queryer() {
        use std::fs;
        use tempfile::TempDir;
        
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let storage_dir = temp_dir.path();
        
        // 创建测试数据
        let points = vec![
            DataPoint::new("metric1", 10.0, MetricCategory::Render),
            DataPoint::new("metric2", 20.0, MetricCategory::Memory),
            DataPoint::new("metric1", 15.0, MetricCategory::Render),
        ];
        
        // 写入测试文件
        let file_path = storage_dir.join("test_metrics_0.dat");
        let mut file = fs::File::create(&file_path).unwrap();
        
        for point in &points {
            let serialized = serde_json::to_vec(point).unwrap();
            let length_bytes = (serialized.len() as u32).to_le_bytes();
            file.write_all(&length_bytes).unwrap();
            file.write_all(&serialized).unwrap();
        }
        
        // 创建查询器并测试查询
        let queryer = DataQueryer::new(storage_dir, "test_metrics");
        
        let condition = QueryCondition {
            metric_names: Some(vec!["metric1".to_string()]),
            categories: None,
            start_time: None,
            end_time: None,
            tags: None,
            limit: None,
            order_by: Some(QueryOrder::TimestampAsc),
        };
        
        let result = queryer.query(&condition).unwrap();
        
        assert_eq!(result.data_points.len(), 2);
        assert_eq!(result.data_points[0].metric_name, "metric1");
        assert_eq!(result.data_points[1].metric_name, "metric1");
        assert_eq!(result.data_points[0].value, 10.0);
        assert_eq!(result.data_points[1].value, 15.0);
    }

    #[test]
    fn test_persistent_storage_sync_io() {
        use tempfile::tempdir;
        use std::fs;

        let dir = tempdir().unwrap();
        let mut cfg = StorageConfig::default();
        cfg.data_dir = dir.path().to_path_buf();
        cfg.file_prefix = "test_metrics".to_string();
        cfg.enable_write_cache = true;
        cfg.cache_size = 2; // flush after 2

        // Create storage synchronously
        let mut storage = PersistentStorage::new_sync(cfg).expect("new_sync failed");

        // Store two points to trigger flush
        storage.store(DataPoint::new("m1", 1.0, MetricCategory::CPU)).unwrap();
        storage.store(DataPoint::new("m2", 2.0, MetricCategory::Memory)).unwrap();

        // flush cache sync
        storage.flush_cache_sync().expect("flush_cache_sync failed");

        // stats
        let stats = storage.get_storage_stats_sync().expect("get_storage_stats_sync failed");
        assert!(stats.total_files >= 1);
        assert!(stats.current_file_size > 0 || stats.total_size > 0);

        // query sync via DataQueryer
        let q = QueryCondition { metric_names: None, categories: None, start_time: None, end_time: None, tags: None, limit: None, order_by: None };
        let queryer = DataQueryer::new(&dir.path(), "test_metrics");
        let res = queryer.query_sync(&q).expect("query_sync failed");
        assert!(res.total_count >= 2);

        // cleanup
        fs::remove_dir_all(dir.path()).ok();
    }
}