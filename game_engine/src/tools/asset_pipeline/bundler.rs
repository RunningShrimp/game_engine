//! # Asset Bundler - 资源打包器
//!
//! 本模块实现资源打包功能，将多个资源打包成单一文件或虚拟文件系统。

use super::pipeline::{OptimizationError, OptimizationResult, AssetType};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use flate2::write::GzEncoder;
use flate2::Compression;
use flate2::read::GzDecoder;
use sha2::{Sha256, Digest};
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
}

impl AssetBundler {
    /// 创建新的打包器
    pub fn new() -> Self {
        Self {
            bundle_format: BundleFormat::Pak,
            compression: Some(CompressionAlgorithm::Gzip),
            chunk_size: 64 * 1024, // 64KB chunks
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
        let mut entries = Vec::new();
        let mut bundle_data = Vec::new();
        let mut current_offset = 0u64;

        for result in results {
            // 读取资源数据
            let mut data = Vec::new();
            let mut file = File::open(&result.asset_path)
                .map_err(|e| OptimizationError::BundleError(format!("Failed to open asset: {}", e)))?;

            file.read_to_end(&mut data)
                .map_err(|e| OptimizationError::BundleError(format!("Failed to read asset: {}", e)))?;

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
                    meta.insert("processing_time".to_string(), format!("{}", result.processing_time));
                    if result.compressed {
                        meta.insert("compressed".to_string(), "true".to_string());
                    }
                    if result.optimized {
                        meta.insert("optimized".to_string(), "true".to_string());
                    }
                    if result.lods_generated > 0 {
                        meta.insert("lods".to_string(), format!("{}", result.lods_generated));
                    }
                    meta
                },
            };

            entries.push(entry);
            bundle_data.extend_from_slice(&compressed_data);
            current_offset += compressed_data.len() as u64;

            println!("  Bundled: {}", result.asset_path.display());
        }

        // 创建资源包
        let bundle = Bundle {
            version: 1,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            entries,
            metadata: BundleMetadata {
                name: output_path
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                bundle_version: "1.0".to_string(),
                platform: "unknown".to_string(),
                engine_version: env!("CARGO_PKG_VERSION").to_string(),
                custom: HashMap::new(),
            },
        };

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
        let file = File::create(output_path)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to create bundle: {}", e)))?;

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
        let metadata_json = serde_json::to_string_pretty(bundle)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to serialize metadata: {}", e)))?;
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
            writer
                .write_all(header_slice)
                .map_err(|e| OptimizationError::BundleError(format!("Failed to write header: {}", e)))?;
        }

        writer
            .write_all(bundle_data)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to write data: {}", e)))?;

        writer
            .write_all(metadata_bytes)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to write metadata: {}", e)))?;

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
                std::fs::create_dir_all(&dest_dir)
                    .map_err(|e| OptimizationError::BundleError(format!("Failed to create directory: {}", e)))?;
            }

            let dest_path = output_dir.join(&result.asset_path);
            std::fs::copy(&result.asset_path, &dest_path)
                .map_err(|e| OptimizationError::BundleError(format!("Failed to copy file: {}", e)))?;

            println!("  Copied: {}", result.asset_path.display());
        }

        // 创建元数据文件
        let bundle = Bundle {
            version: 1,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
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
        let metadata_json = serde_json::to_string_pretty(&bundle)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to serialize metadata: {}", e)))?;

        std::fs::write(&metadata_path, metadata_json)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to write metadata: {}", e)))?;

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
            let mut file = File::open(&result.asset_path)
                .map_err(|e| OptimizationError::BundleError(format!("Failed to open asset: {}", e)))?;

            file.read_to_end(&mut data)
                .map_err(|e| OptimizationError::BundleError(format!("Failed to read asset: {}", e)))?;

            vfs.add_file(&result.asset_path, data);
        }

        // 序列化VFS
        let vfs_data = bincode::serialize(&vfs)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to serialize VFS: {}", e)))?;

        std::fs::write(output_path, vfs_data)
            .map_err(|e| OptimizationError::BundleError(format!("Failed to write VFS: {}", e)))?;

        println!("Virtual bundle created: {}", output_path.display());

        Ok(Bundle {
            version: 1,
            created_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
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
        match compression {
            CompressionAlgorithm::None => Ok(data.to_vec()),
            CompressionAlgorithm::Gzip => {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(data)
                    .map_err(|e| OptimizationError::BundleError(format!("Compression failed: {}", e)))?;
                encoder
                    .finish()
                    .map_err(|e| OptimizationError::BundleError(format!("Compression failed: {}", e)))
            }
            CompressionAlgorithm::Zlib => {
                // 简化实现：使用Gzip代替
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(data)
                    .map_err(|e| OptimizationError::BundleError(format!("Compression failed: {}", e)))?;
                encoder
                    .finish()
                    .map_err(|e| OptimizationError::BundleError(format!("Compression failed: {}", e)))
            }
            CompressionAlgorithm::LZ4 => {
                // TODO: 实现LZ4压缩
                Ok(data.to_vec())
            }
        }
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
        self.files
            .insert(path.to_string_lossy().to_string(), data);
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
