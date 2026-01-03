//! # Asset Bundler - 资源打包器
//!
//! 本模块实现资源打包功能，将多个资源打包成单一文件或虚拟文件系统。

use super::pipeline::{AssetType, OptimizationError, OptimizationResult};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::{GzEncoder, ZlibEncoder};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// 打包格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BundleFormat {
    /// 单文件打包（类似Pak格式）
    Pak,

    /// 松散文件结构
    Loose,

    /// 虚拟文件系统
    Virtual,
}

/// 压缩算法
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CompressionAlgorithm {
    /// 无压缩
    None,

    /// Zlib压缩
    Zlib,

    /// Gzip压缩
    Gzip,

    /// LZ4压缩（快速）
    LZ4,

    /// Zstd压缩（高压缩率）
    Zstd,

    /// Brotli压缩（Web优化）
    Brotli,

    /// LZMA压缩（最高压缩率）
    LZMA,

    /// 自动选择最佳算法
    Auto,
}

impl CompressionAlgorithm {
    /// 获取压缩速度等级（1-10，10最快）
    pub fn speed_level(&self) -> u8 {
        match self {
            CompressionAlgorithm::None => 10,
            CompressionAlgorithm::LZ4 => 9,
            CompressionAlgorithm::Zstd => 7,
            CompressionAlgorithm::Gzip => 6,
            CompressionAlgorithm::Zlib => 6,
            CompressionAlgorithm::Brotli => 4,
            CompressionAlgorithm::LZMA => 2,
            CompressionAlgorithm::Auto => 6,
        }
    }

    /// 获取压缩率等级（1-10，10最高）
    pub fn compression_ratio(&self) -> u8 {
        match self {
            CompressionAlgorithm::None => 1,
            CompressionAlgorithm::LZ4 => 6,
            CompressionAlgorithm::Gzip => 7,
            CompressionAlgorithm::Zlib => 7,
            CompressionAlgorithm::Zstd => 8,
            CompressionAlgorithm::Brotli => 9,
            CompressionAlgorithm::LZMA => 10,
            CompressionAlgorithm::Auto => 7,
        }
    }
}

/// 压缩级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// 最快速度（1-3）
    Fast,
    /// 平衡模式（4-6）
    Balanced,
    /// 最高压缩率（7-9）
    Best,
    /// 自定义级别（0-9）
    Custom(u8),
}

impl Default for CompressionLevel {
    fn default() -> Self {
        Self::Balanced
    }
}

/// 打包优化配置
#[derive(Debug, Clone)]
pub struct BundleOptimizationConfig {
    /// 启用去重
    pub enable_deduplication: bool,
    /// 启用资源优先级排序
    pub enable_priority_sorting: bool,
    /// 启用依赖分析
    pub enable_dependency_analysis: bool,
    /// 压缩级别
    pub compression_level: CompressionLevel,
    /// 最小压缩文件大小（字节）
    pub min_compression_size: usize,
    /// 启用增量打包
    pub enable_incremental_bundle: bool,
}

impl Default for BundleOptimizationConfig {
    fn default() -> Self {
        Self {
            enable_deduplication: true,
            enable_priority_sorting: true,
            enable_dependency_analysis: true,
            compression_level: CompressionLevel::Balanced,
            min_compression_size: 1024, // 1KB
            enable_incremental_bundle: true,
        }
    }
}

/// 压缩统计信息
#[derive(Debug, Clone)]
pub struct CompressionStatistics {
    /// 原始总大小
    pub total_original_size: u64,
    /// 压缩后总大小
    pub total_compressed_size: u64,
    /// 压缩率（百分比）
    pub compression_ratio: f32,
    /// 各算法使用统计
    pub algorithm_usage: HashMap<CompressionAlgorithm, usize>,
    /// 各类型资源压缩效果
    pub asset_type_stats: HashMap<String, AssetCompressionStats>,
    /// 压缩时间（毫秒）
    pub compression_time_ms: u64,
}

/// 单个资源类型的压缩统计
#[derive(Debug, Clone)]
pub struct AssetCompressionStats {
    /// 文件数量
    pub count: usize,
    /// 原始大小
    pub original_size: u64,
    /// 压缩后大小
    pub compressed_size: u64,
    /// 压缩率
    pub ratio: f32,
}

/// 资源包
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    /// 包格式版本
    pub version: u32,

    /// 创建时间戳
    pub created_at: u64,

    /// 资源条目
    pub entries: Vec<BundleEntry>,

    /// 元数据
    pub metadata: BundleMetadata,
}

/// 资源包条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleEntry {
    /// 资源路径
    pub path: String,

    /// 资源类型
    pub asset_type: AssetType,

    /// 原始大小
    pub original_size: u64,

    /// 压缩后大小
    pub compressed_size: u64,

    /// 数据偏移量
    pub offset: u64,

    /// 数据长度
    pub length: u64,

    /// 校验和（SHA256）
    pub checksum: String,

    /// 压缩算法
    pub compression: CompressionAlgorithm,

    /// 额外元数据
    pub metadata: HashMap<String, String>,
}

/// 资源包元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    /// 包名称
    pub name: String,

    /// 包版本
    pub bundle_version: String,

    /// 目标平台
    pub platform: String,

    /// 引擎版本
    pub engine_version: String,

    /// 自定义字段
    pub custom: HashMap<String, String>,
}

/// 资源打包器
pub struct AssetBundler {
    bundle_format: BundleFormat,
    compression: Option<CompressionAlgorithm>,
    chunk_size: usize,
    /// 优化配置
    optimization_config: BundleOptimizationConfig,
    /// 去重缓存
    deduplication_cache: HashMap<Vec<u8>, String>, // checksum -> path
}

impl AssetBundler {
    /// 创建新的打包器
    pub fn new() -> Self {
        Self {
            bundle_format: BundleFormat::Pak,
            compression: Some(CompressionAlgorithm::Auto),
            chunk_size: 64 * 1024, // 64KB chunks
            optimization_config: BundleOptimizationConfig::default(),
            deduplication_cache: HashMap::new(),
        }
    }

    /// 设置打包格式
    pub fn with_format(mut self, format: BundleFormat) -> Self {
        self.bundle_format = format;
        self
    }

    /// 设置压缩算法
    pub fn with_compression(mut self, compression: CompressionAlgorithm) -> Self {
        self.compression = Some(compression);
        self
    }

    /// 设置优化配置
    pub fn with_optimization_config(mut self, config: BundleOptimizationConfig) -> Self {
        self.optimization_config = config;
        self
    }

    /// 打包资源
    pub async fn bundle_assets(
        &self,
        results: &[OptimizationResult],
        output_path: &Path,
    ) -> Result<Bundle, OptimizationError> {
        println!("Bundling assets...");

        match self.bundle_format {
            BundleFormat::Pak => self.create_pak_bundle(results, output_path).await,
            BundleFormat::Loose => self.create_loose_bundle(results, output_path).await,
            BundleFormat::Virtual => self.create_virtual_bundle(results, output_path).await,
        }
    }

    /// 创建Pak格式资源包
    async fn create_pak_bundle(
        &self,
        results: &[OptimizationResult],
        output_path: &Path,
    ) -> Result<Bundle, OptimizationError> {
        let start_time = std::time::Instant::now();

        // 应用优化
        let mut optimized_results = results.to_vec();
        optimized_results = self.deduplicate_assets(&optimized_results);
        self.sort_assets_by_priority(&mut optimized_results);
        let _dependencies = self.analyze_dependencies(&optimized_results);

        println!("Bundling {} assets...", optimized_results.len());

        let mut entries = Vec::new();
        let mut bundle_data = Vec::new();
        let mut current_offset = 0u64;

        for result in &optimized_results {
            // 读取资源数据
            let mut data = Vec::new();
            let mut file = File::open(&result.asset_path).map_err(|e| {
                OptimizationError::BundleError(format!("Failed to open asset: {}", e))
            })?;

            file.read_to_end(&mut data).map_err(|e| {
                OptimizationError::BundleError(format!("Failed to read asset: {}", e))
            })?;

            // 压缩数据
            let compressed_data = if let Some(compression) = self.compression {
                self.compress_data(&data, compression)?
            } else {
                data.clone()
            };

            // 计算校验和
            let checksum = self.calculate_checksum(&data);

            // 创建条目
            let entry = BundleEntry {
                path: result.asset_path.to_string_lossy().to_string(),
                asset_type: result.asset_type,
                original_size: result.optimized_size,
                compressed_size: compressed_data.len() as u64,
                offset: current_offset,
                length: compressed_data.len() as u64,
                checksum,
                compression: self.compression.unwrap_or(CompressionAlgorithm::None),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert(
                        "processing_time".to_string(),
                        format!("{}", result.processing_time),
                    );
                    if result.compressed {
                        meta.insert("compressed".to_string(), "true".to_string());
                    }
                    if result.optimized {
                        meta.insert("optimized".to_string(), "true".to_string());
                    }
                    if result.lods_generated > 0 {
                        meta.insert("lods".to_string(), format!("{}", result.lods_generated));
                    }
                    // 添加压缩元数据
                    let ratio = if data.len() > 0 {
                        (compressed_data.len() as f32 / data.len() as f32) * 100.0
                    } else {
                        100.0
                    };
                    meta.insert("compression_ratio".to_string(), format!("{:.1}%", ratio));
                    meta
                },
            };

            entries.push(entry);
            bundle_data.extend_from_slice(&compressed_data);
            current_offset += compressed_data.len() as u64;

            println!("  Bundled: {}", result.asset_path.display());
        }

        // 创建资源包
        let compression_time = start_time.elapsed().as_millis() as u64;
        let bundle = Bundle {
            version: 1,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            entries: entries.clone(),
            metadata: BundleMetadata {
                name: output_path.file_stem().unwrap_or_default().to_string_lossy().to_string(),
                bundle_version: "1.0".to_string(),
                platform: "unknown".to_string(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                custom: {
                    let mut custom = HashMap::new();
                    custom.insert(
                        "compression_time_ms".to_string(),
                        compression_time.to_string(),
                    );
                    custom.insert(
                        "deduplication_enabled".to_string(),
                        self.optimization_config.enable_deduplication.to_string(),
                    );
                    custom.insert(
                        "priority_sorting_enabled".to_string(),
                        self.optimization_config.enable_priority_sorting.to_string(),
                    );
                    custom.insert(
                        "dependency_analysis_enabled".to_string(),
                        self.optimization_config.enable_dependency_analysis.to_string(),
                    );
                    custom
                },
            },
        };

        // 生成并打印统计信息
        let stats = self.generate_compression_stats(results, &entries, compression_time);
        self.print_compression_stats(&stats);

        // 序列化并写入文件
        self.write_pak_bundle(&bundle, &bundle_data, output_path)?;

        Ok(bundle)
    }

    /// 写入Pak格式文件
    fn write_pak_bundle(
        &self,
        bundle: &Bundle,
        bundle_data: &[u8],
        output_path: &Path,
    ) -> Result<(), OptimizationError> {
        let file = File::create(output_path).map_err(|e| {
            OptimizationError::BundleError(format!("Failed to create bundle: {}", e))
        })?;

        let mut writer = BufWriter::new(file);

        // 写入文件头
        let header = PakHeader {
            magic: *b"PAK\0",
            version: bundle.version,
            metadata_offset: 0, // 稍后填充
            metadata_size: 0,
            data_offset: 0,
            data_size: bundle_data.len() as u64,
        };

        // 序列化元数据
        let metadata_json = serde_json::to_string_pretty(bundle).map_err(|e| {
            OptimizationError::BundleError(format!("Failed to serialize metadata: {}", e))
        })?;
        let metadata_bytes = metadata_json.as_bytes();

        // 计算偏移量
        let header_size = std::mem::size_of::<PakHeader>() as u64;
        let data_offset = header_size;
        let metadata_offset = data_offset + bundle_data.len() as u64;

        // 更新header
        let header = PakHeader {
            metadata_offset,
            metadata_size: metadata_bytes.len() as u64,
            ..header
        };

        // 写入文件
        unsafe {
            let header_slice = std::slice::from_raw_parts(
                &header as *const PakHeader as *const u8,
                std::mem::size_of::<PakHeader>(),
            );
            writer.write_all(header_slice).map_err(|e| {
                OptimizationError::BundleError(format!("Failed to write header: {}", e))
            })?;
        }

        writer
            .write_all(bundle_data)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to write data: {}", e)))?;

        writer.write_all(metadata_bytes).map_err(|e| {
            OptimizationError::BundleError(format!("Failed to write metadata: {}", e))
        })?;

        writer
            .flush()
            .map_err(|e| OptimizationError::BundleError(format!("Failed to flush: {}", e)))?;

        println!("Bundle created: {}", output_path.display());
        println!("  Total assets: {}", bundle.entries.len());
        println!("  Total size: {} MB", bundle_data.len() / 1024 / 1024);

        Ok(())
    }

    /// 创建松散文件结构
    async fn create_loose_bundle(
        &self,
        results: &[OptimizationResult],
        output_dir: &Path,
    ) -> Result<Bundle, OptimizationError> {
        // 松散文件结构：直接复制文件到输出目录
        for result in results {
            if let Some(parent) = result.asset_path.parent() {
                let dest_dir = output_dir.join(parent);
                std::fs::create_dir_all(&dest_dir).map_err(|e| {
                    OptimizationError::BundleError(format!("Failed to create directory: {}", e))
                })?;
            }

            let dest_path = output_dir.join(&result.asset_path);
            std::fs::copy(&result.asset_path, &dest_path).map_err(|e| {
                OptimizationError::BundleError(format!("Failed to copy file: {}", e))
            })?;

            println!("  Copied: {}", result.asset_path.display());
        }

        // 创建元数据文件
        let bundle = Bundle {
            version: 1,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            entries: results
                .iter()
                .map(|r| BundleEntry {
                    path: r.asset_path.to_string_lossy().to_string(),
                    asset_type: r.asset_type,
                    original_size: r.original_size,
                    compressed_size: r.optimized_size,
                    offset: 0,
                    length: r.optimized_size,
                    checksum: String::new(),
                    compression: CompressionAlgorithm::None,
                    metadata: HashMap::new(),
                })
                .collect(),
            metadata: BundleMetadata {
                name: "loose_bundle".to_string(),
                bundle_version: "1.0".to_string(),
                platform: "unknown".to_string(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                custom: HashMap::new(),
            },
        };

        let metadata_path = output_dir.join("bundle_metadata.json");
        let metadata_json = serde_json::to_string_pretty(&bundle).map_err(|e| {
            OptimizationError::BundleError(format!("Failed to serialize metadata: {}", e))
        })?;

        std::fs::write(&metadata_path, metadata_json).map_err(|e| {
            OptimizationError::BundleError(format!("Failed to write metadata: {}", e))
        })?;

        Ok(bundle)
    }

    /// 创建虚拟文件系统
    async fn create_virtual_bundle(
        &self,
        results: &[OptimizationResult],
        output_path: &Path,
    ) -> Result<Bundle, OptimizationError> {
        // 虚拟文件系统：创建内存文件系统的序列化表示
        let mut vfs = VirtualFileSystem::new();

        for result in results {
            let mut data = Vec::new();
            let mut file = File::open(&result.asset_path).map_err(|e| {
                OptimizationError::BundleError(format!("Failed to open asset: {}", e))
            })?;

            file.read_to_end(&mut data).map_err(|e| {
                OptimizationError::BundleError(format!("Failed to read asset: {}", e))
            })?;

            vfs.add_file(&result.asset_path, data);
        }

        // 序列化VFS
        let vfs_data = bincode::serialize(&vfs).map_err(|e| {
            OptimizationError::BundleError(format!("Failed to serialize VFS: {}", e))
        })?;

        std::fs::write(output_path, vfs_data)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to write VFS: {}", e)))?;

        println!("Virtual bundle created: {}", output_path.display());

        Ok(Bundle {
            version: 1,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            entries: Vec::new(),
            metadata: BundleMetadata {
                name: "virtual_bundle".to_string(),
                bundle_version: "1.0".to_string(),
                platform: "unknown".to_string(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                custom: HashMap::new(),
            },
        })
    }

    /// 压缩数据
    fn compress_data(
        &self,
        data: &[u8],
        compression: CompressionAlgorithm,
    ) -> Result<Vec<u8>, OptimizationError> {
        // 检查是否值得压缩
        if data.len() < self.optimization_config.min_compression_size {
            return Ok(data.to_vec());
        }

        let algorithm = if compression == CompressionAlgorithm::Auto {
            self.select_best_algorithm(data)?
        } else {
            compression
        };

        let start = std::time::Instant::now();
        let result = match algorithm {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Gzip => {
                let level = self.compression_level_to_flate2();
                let mut encoder = GzEncoder::new(Vec::new(), level);
                encoder.write_all(data).map_err(|e| {
                    OptimizationError::BundleError(format!("Gzip compression failed: {}", e))
                })?;
                encoder.finish().map_err(|e| {
                    OptimizationError::BundleError(format!("Gzip compression failed: {}", e))
                })
            }
            CompressionAlgorithm::Zlib => {
                let level = self.compression_level_to_flate2();
                let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), level);
                encoder.write_all(data).map_err(|e| {
                    OptimizationError::BundleError(format!("Zlib compression failed: {}", e))
                })?;
                encoder.finish().map_err(|e| {
                    OptimizationError::BundleError(format!("Zlib compression failed: {}", e))
                })
            }
            CompressionAlgorithm::LZ4 => self.compress_lz4(data),
            CompressionAlgorithm::Zstd => self.compress_zstd(data),
            CompressionAlgorithm::Brotli => self.compress_brotli(data),
            CompressionAlgorithm::LZMA => self.compress_lzma(data),
            CompressionAlgorithm::Auto => {
                // Already handled above
                Ok(data.to_vec())
            }
        };

        let duration = start.elapsed();
        if duration.as_millis() > 100 {
            println!(
                "  Compression took {}ms for {} bytes",
                duration.as_millis(),
                data.len()
            );
        }

        result
    }

    /// 选择最佳压缩算法
    fn select_best_algorithm(
        &self,
        data: &[u8],
    ) -> Result<CompressionAlgorithm, OptimizationError> {
        // 基于数据特征选择算法
        let entropy = self.calculate_entropy(data);

        // 高熵数据（已压缩或加密）：使用快速算法
        if entropy > 7.8 {
            return Ok(CompressionAlgorithm::LZ4);
        }

        // 低熵数据（高重复性）：使用高压缩率算法
        if entropy < 5.0 {
            return if data.len() > 1024 * 1024 {
                Ok(CompressionAlgorithm::Zstd) // 大文件使用Zstd
            } else {
                Ok(CompressionAlgorithm::Brotli) // 小文件使用Brotli
            };
        }

        // 中等熵：使用平衡算法
        Ok(match self.optimization_config.compression_level {
            CompressionLevel::Fast => CompressionAlgorithm::LZ4,
            CompressionLevel::Balanced => CompressionAlgorithm::Gzip,
            CompressionLevel::Best => CompressionAlgorithm::Zstd,
            CompressionLevel::Custom(level) if level <= 3 => CompressionAlgorithm::LZ4,
            CompressionLevel::Custom(level) if level <= 6 => CompressionAlgorithm::Gzip,
            _ => CompressionAlgorithm::Zstd,
        })
    }

    /// 计算数据熵（用于判断压缩潜力）
    fn calculate_entropy(&self, data: &[u8]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        let mut freq = [0u64; 256];
        for &byte in data {
            freq[byte as usize] += 1;
        }

        let len = data.len() as f64;
        let mut entropy = 0.0;
        for &count in &freq {
            if count > 0 {
                let p = count as f64 / len;
                entropy -= p * p.log2();
            }
        }

        entropy as f32
    }

    /// LZ4压缩实现
    fn compress_lz4(&self, data: &[u8]) -> Result<Vec<u8>, OptimizationError> {
        // 使用简化的LZ4实现
        // 注意：实际应用中应使用 lz4-rs 或 lz4-compression crate

        let mut compressed = Vec::with_capacity(data.len() / 2);
        let mut pos = 0;
        const MIN_MATCH: usize = 4;
        const MAX_DISTANCE: usize = 65535;

        while pos < data.len() {
            // 查找匹配
            let mut best_match = 0;
            let mut best_distance = 0;

            let search_start = if pos > MAX_DISTANCE {
                pos - MAX_DISTANCE
            } else {
                0
            };

            for i in search_start..pos {
                let mut match_len = 0;
                while pos + match_len < data.len()
                    && i + match_len < pos
                    && data[pos + match_len] == data[i + match_len]
                {
                    match_len += 1;
                }

                if match_len >= MIN_MATCH && match_len > best_match {
                    best_match = match_len;
                    best_distance = pos - i;
                }
            }

            if best_match >= MIN_MATCH {
                // 写入字面量标记和距离/长度
                compressed.push(0x01); // 匹配标记
                compressed.push((best_distance >> 8) as u8);
                compressed.push((best_distance & 0xFF) as u8);
                compressed.push((best_match >> 8) as u8);
                compressed.push((best_match & 0xFF) as u8);
                pos += best_match;
            } else {
                // 写入字面量
                let mut literal_len = 1;
                while pos + literal_len < data.len() && literal_len < 15 && {
                    // 检查接下来的字节是否能找到匹配
                    let mut found = false;
                    for i in search_start..(pos + literal_len) {
                        if i + MIN_MATCH <= pos + literal_len && data[pos + literal_len] == data[i]
                        {
                            found = true;
                            break;
                        }
                    }
                    !found
                } {
                    literal_len += 1;
                }

                compressed.push(0x00); // 字面量标记
                compressed.push(literal_len as u8);
                compressed.extend_from_slice(&data[pos..pos + literal_len]);
                pos += literal_len;
            }
        }

        // 如果压缩后没有变小，返回原始数据
        if compressed.len() >= data.len() {
            Ok(data.to_vec())
        } else {
            Ok(compressed)
        }
    }

    /// Zstd压缩实现
    fn compress_zstd(&self, data: &[u8]) -> Result<Vec<u8>, OptimizationError> {
        // 简化的Zstd实现
        // 实际应用中应使用 zstd crate

        // 使用zlib作为fallback（比无压缩好）
        let level = self.compression_level_to_flate2();
        let mut encoder = GzEncoder::new(Vec::new(), level);
        encoder.write_all(data).map_err(|e| {
            OptimizationError::BundleError(format!("Zstd (fallback) compression failed: {}", e))
        })?;
        encoder.finish().map_err(|e| {
            OptimizationError::BundleError(format!("Zstd (fallback) compression failed: {}", e))
        })
    }

    /// Brotli压缩实现
    fn compress_brotli(&self, data: &[u8]) -> Result<Vec<u8>, OptimizationError> {
        // 简化的Brotli实现
        // 实际应用中应使用 brotli crate

        // 使用zlib作为fallback（比无压缩好）
        let level = self.compression_level_to_flate2();
        let mut encoder = GzEncoder::new(Vec::new(), level);
        encoder.write_all(data).map_err(|e| {
            OptimizationError::BundleError(format!("Brotli (fallback) compression failed: {}", e))
        })?;
        encoder.finish().map_err(|e| {
            OptimizationError::BundleError(format!("Brotli (fallback) compression failed: {}", e))
        })
    }

    /// LZMA压缩实现
    fn compress_lzma(&self, data: &[u8]) -> Result<Vec<u8>, OptimizationError> {
        // 简化的LZMA实现
        // 实际应用中应使用 xz2 或 lzma crate

        // 使用最佳质量gzip作为fallback
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(data).map_err(|e| {
            OptimizationError::BundleError(format!("LZMA (fallback) compression failed: {}", e))
        })?;
        encoder.finish().map_err(|e| {
            OptimizationError::BundleError(format!("LZMA (fallback) compression failed: {}", e))
        })
    }

    /// 将压缩级别转换为flate2的Compression
    fn compression_level_to_flate2(&self) -> Compression {
        match self.optimization_config.compression_level {
            CompressionLevel::Fast => Compression::fast(),
            CompressionLevel::Balanced => Compression::default(),
            CompressionLevel::Best => Compression::best(),
            CompressionLevel::Custom(level) => Compression::new((level.min(9)) as u32),
        }
    }

    /// 资源去重
    fn deduplicate_assets(&self, results: &[OptimizationResult]) -> Vec<OptimizationResult> {
        if !self.optimization_config.enable_deduplication {
            return results.to_vec();
        }

        let mut unique_assets = Vec::new();
        let mut seen_checksums = std::collections::HashSet::new();

        for result in results {
            let checksum = self.calculate_checksum_for_path(&result.asset_path);

            if seen_checksums.insert(checksum.clone()) {
                // 首次看到此资源，保留
                unique_assets.push(result.clone());
            } else {
                // 重复资源，记录到缓存
                self.deduplication_cache.insert(
                    checksum.into_bytes(),
                    result.asset_path.to_string_lossy().to_string(),
                );
                println!(
                    "  Deduplicated: {} (duplicate content)",
                    result.asset_path.display()
                );
            }
        }

        unique_assets
    }

    /// 按优先级排序资源
    fn sort_assets_by_priority(&self, results: &mut [OptimizationResult]) {
        if !self.optimization_config.enable_priority_sorting {
            return;
        }

        results.sort_by(|a, b| {
            // 关键资源优先（场景、脚本）
            let a_priority = self.get_asset_priority(a);
            let b_priority = self.get_asset_priority(b);

            b_priority.cmp(&a_priority).then_with(|| a.asset_path.cmp(&b.asset_path))
        });
    }

    /// 获取资源优先级（0-10，10最高）
    fn get_asset_priority(&self, result: &OptimizationResult) -> u8 {
        match result.asset_type {
            AssetType::Scene => 10,
            AssetType::Script => 9,
            AssetType::Shader => 8,
            AssetType::Texture => 7,
            AssetType::Model => 6,
            AssetType::Audio => 5,
            AssetType::Font => 4,
            AssetType::Video => 3,
            AssetType::Unknown => 1,
        }
    }

    /// 分析资源依赖关系
    fn analyze_dependencies(
        &self,
        results: &[OptimizationResult],
    ) -> std::collections::HashMap<String, Vec<String>> {
        if !self.optimization_config.enable_dependency_analysis {
            return std::collections::HashMap::new();
        }

        let mut dependencies: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for result in results {
            let mut deps = Vec::new();

            // 分析场景文件的依赖
            if result.asset_type == AssetType::Scene {
                if let Ok(content) = std::fs::read_to_string(&result.asset_path) {
                    // 提取引用的资源路径（简化实现）
                    for line in content.lines() {
                        if line.contains("asset=\"")
                            || line.contains("texture=\"")
                            || line.contains("model=\"")
                        {
                            if let Some(start) = line.find('"') {
                                if let Some(end) = line[start + 1..].find('"') {
                                    let asset_path = &line[start + 1..start + 1 + end];
                                    deps.push(asset_path.to_string());
                                }
                            }
                        }
                    }
                }
            }

            // 分析脚本文件的依赖
            if result.asset_type == AssetType::Script {
                if let Ok(content) = std::fs::read_to_string(&result.asset_path) {
                    // 简单的require/import语句匹配
                    for line in content.lines() {
                        if line.contains("require(") || line.contains("import ") {
                            if let Some(start) = line.find('"') {
                                if let Some(end) = line[start + 1..].find('"') {
                                    let asset_path = &line[start + 1..start + 1 + end];
                                    deps.push(asset_path.to_string());
                                }
                            } else if let Some(start) = line.find('\'') {
                                if let Some(end) = line[start + 1..].find('\'') {
                                    let asset_path = &line[start + 1..start + 1 + end];
                                    deps.push(asset_path.to_string());
                                }
                            }
                        }
                    }
                }
            }

            if !deps.is_empty() {
                dependencies.insert(result.asset_path.to_string_lossy().to_string(), deps);
            }
        }

        dependencies
    }

    /// 计算路径的校验和
    fn calculate_checksum_for_path(&self, path: &Path) -> String {
        if let Ok(data) = std::fs::read(path) {
            self.calculate_checksum(&data)
        } else {
            String::new()
        }
    }

    /// 生成压缩统计报告
    pub fn generate_compression_stats(
        &self,
        original_results: &[OptimizationResult],
        compressed_entries: &[BundleEntry],
        compression_time_ms: u64,
    ) -> CompressionStatistics {
        let mut total_original_size = 0u64;
        let mut total_compressed_size = 0u64;
        let mut algorithm_usage: std::collections::HashMap<CompressionAlgorithm, usize> =
            std::collections::HashMap::new();
        let mut asset_type_stats: std::collections::HashMap<String, AssetCompressionStats> =
            std::collections::HashMap::new();

        for result in original_results {
            total_original_size += result.original_size;
        }

        for entry in compressed_entries {
            total_compressed_size += entry.compressed_size;

            *algorithm_usage.entry(entry.compression).or_insert(0) += 1;

            let type_name = format!("{:?}", entry.asset_type);
            let stats =
                asset_type_stats.entry(type_name.clone()).or_insert(AssetCompressionStats {
                    count: 0,
                    original_size: 0,
                    compressed_size: 0,
                    ratio: 0.0,
                });

            stats.count += 1;
            stats.original_size += entry.original_size;
            stats.compressed_size += entry.compressed_size;
        }

        // 计算各类型的压缩率
        for stats in asset_type_stats.values_mut() {
            if stats.original_size > 0 {
                stats.ratio = (stats.compressed_size as f32 / stats.original_size as f32) * 100.0;
            }
        }

        let compression_ratio = if total_original_size > 0 {
            (total_compressed_size as f32 / total_original_size as f32) * 100.0
        } else {
            100.0
        };

        CompressionStatistics {
            total_original_size,
            total_compressed_size,
            compression_ratio,
            algorithm_usage,
            asset_type_stats,
            compression_time_ms,
        }
    }

    /// 打印压缩统计报告
    pub fn print_compression_stats(&self, stats: &CompressionStatistics) {
        println!("\n=== Compression Statistics ===");
        println!(
            "Original size: {} MB",
            stats.total_original_size / 1024 / 1024
        );
        println!(
            "Compressed size: {} MB",
            stats.total_compressed_size / 1024 / 1024
        );
        println!("Compression ratio: {:.1}%", stats.compression_ratio);
        println!("Space saved: {:.1}%", 100.0 - stats.compression_ratio);
        println!("Compression time: {} ms", stats.compression_time_ms);

        println!("\nAlgorithm Usage:");
        for (algorithm, count) in &stats.algorithm_usage {
            println!("  {:?}: {} files", algorithm, count);
        }

        println!("\nAsset Type Statistics:");
        for (type_name, type_stats) in &stats.asset_type_stats {
            println!(
                "  {}: {} files, {:.1}% of original",
                type_name, type_stats.count, type_stats.ratio
            );
        }
        println!("============================\n");
    }

    /// 计算校验和
    fn calculate_checksum(&self, data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

impl Default for AssetBundler {
    fn default() -> Self {
        Self::new()
    }
}

/// Pak文件头
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct PakHeader {
    /// 魔术数字
    magic: [u8; 4],

    /// 版本号
    version: u32,

    /// 元数据偏移量
    metadata_offset: u64,

    /// 元数据大小
    metadata_size: u64,

    /// 数据偏移量
    data_offset: u64,

    /// 数据大小
    data_size: u64,
}

/// 虚拟文件系统
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VirtualFileSystem {
    files: HashMap<String, Vec<u8>>,
}

impl VirtualFileSystem {
    fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    fn add_file(&mut self, path: &Path, data: Vec<u8>) {
        self.files.insert(path.to_string_lossy().to_string(), data);
    }

    fn get_file(&self, path: &str) -> Option<&[u8]> {
        self.files.get(path).map(|v| v.as_slice())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum_calculation() {
        let bundler = AssetBundler::new();
        let data = b"test data";

        let checksum1 = bundler.calculate_checksum(data);
        let checksum2 = bundler.calculate_checksum(data);

        assert_eq!(checksum1, checksum2);
        assert_eq!(checksum1.len(), 64); // SHA256 hex string
    }

    #[test]
    fn test_vfs() {
        let mut vfs = VirtualFileSystem::new();

        vfs.add_file(Path::new("test.txt"), b"hello".to_vec());

        assert_eq!(vfs.get_file("test.txt"), Some(b"hello".as_slice()));
        assert_eq!(vfs.get_file("nonexistent.txt"), None);
    }
}
