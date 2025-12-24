//! 资源流式加载器
//!
//! 提供大型资源（如高分辨率纹理）的流式加载功能。
//! 支持分块加载、渐进式质量提升和内存优化。

use super::resource_trait::{Resource, ResourceError};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
use tokio::sync::mpsc;

/// 流式加载配置
#[derive(Debug, Clone)]
pub struct StreamingConfig {
    /// 每个块的大小（字节）
    pub chunk_size: usize,
    /// 最大并发加载数
    pub max_concurrent: usize,
    /// 预取块数（在需要之前提前加载）
    pub prefetch_chunks: usize,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1024 * 1024, // 1MB
            max_concurrent: 4,
            prefetch_chunks: 2,
        }
    }
}

/// 资源块
#[derive(Debug, Clone)]
pub struct ResourceChunk {
    /// 块索引
    pub chunk_index: usize,
    /// 块数据
    pub data: Vec<u8>,
    /// 块在文件中的偏移量
    pub offset: u64,
    /// 是否是最有一个块
    pub is_last: bool,
}

/// 流式加载句柄
///
/// 用于跟踪流式加载的进度和接收数据块。
pub struct StreamingHandle {
    /// 数据块接收器
    receiver: mpsc::Receiver<Result<ResourceChunk, ResourceError>>,
    /// 总大小（字节）
    total_size: usize,
    /// 已加载大小（字节）
    loaded_size: Arc<AtomicUsize>,
    /// 总块数
    total_chunks: usize,
    /// 已接收块数
    received_chunks: Arc<AtomicUsize>,
    /// 资源路径
    path: PathBuf,
}

impl StreamingHandle {
    /// 获取下一个数据块
    ///
    /// # 返回
    /// 如果还有数据块则返回Some，否则返回None
    pub async fn next_chunk(&mut self) -> Option<Result<ResourceChunk, ResourceError>> {
        self.receiver.recv().await
    }

    /// 获取加载进度（0.0 - 1.0）
    pub fn progress(&self) -> f32 {
        let loaded = self.loaded_size.load(Ordering::Relaxed);
        if self.total_size > 0 {
            loaded as f32 / self.total_size as f32
        } else {
            0.0
        }
    }

    /// 获取已加载大小（字节）
    pub fn loaded_bytes(&self) -> usize {
        self.loaded_size.load(Ordering::Relaxed)
    }

    /// 获取总大小（字节）
    pub fn total_bytes(&self) -> usize {
        self.total_size
    }

    /// 获取已接收块数
    pub fn received_chunks(&self) -> usize {
        self.received_chunks.load(Ordering::Relaxed)
    }

    /// 获取总块数
    pub fn total_chunks(&self) -> usize {
        self.total_chunks
    }

    /// 检查是否已完成加载
    pub fn is_complete(&self) -> bool {
        self.received_chunks.load(Ordering::Relaxed) >= self.total_chunks
    }

    /// 获取资源路径
    pub fn path(&self) -> &Path {
        &self.path
    }
}

/// 流式加载器
///
/// 提供大型资源的流式加载功能，支持分块加载和渐进式质量提升。
pub struct StreamingLoader {
    /// 配置
    config: StreamingConfig,
    /// 并发控制信号量
    semaphore: Arc<tokio::sync::Semaphore>,
}

impl StreamingLoader {
    /// 创建新的流式加载器
    ///
    /// # 参数
    /// - `config`: 流式加载配置
    ///
    /// # 返回
    /// 新的流式加载器实例
    pub fn new(config: StreamingConfig) -> Self {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_concurrent));
        Self { config, semaphore }
    }

    /// 使用默认配置创建流式加载器
    pub fn with_default_config() -> Self {
        Self::new(StreamingConfig::default())
    }

    /// 开始流式加载资源
    ///
    /// # 参数
    /// - `path`: 资源路径
    ///
    /// # 返回
    /// 流式加载句柄
    pub async fn load_streaming(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<StreamingHandle, ResourceError> {
        let path = path.as_ref();
        let path_buf = path.to_path_buf();

        // 获取文件大小
        let metadata = tokio::fs::metadata(path).await?;
        let total_size = metadata.len() as usize;

        // 计算块数
        let total_chunks = (total_size + self.config.chunk_size - 1) / self.config.chunk_size;

        // 创建通道
        let (tx, rx) = mpsc::channel(100);

        // 创建共享状态
        let loaded_size = Arc::new(AtomicUsize::new(0));
        let received_chunks = Arc::new(AtomicUsize::new(0));

        // 启动加载任务
        let config = self.config.clone();
        let semaphore = self.semaphore.clone();
        let loaded_size_clone = loaded_size.clone();
        let received_chunks_clone = received_chunks.clone();

        tokio::spawn(async move {
            let mut file = match File::open(&path_buf).await {
                Ok(f) => f,
                Err(e) => {
                    let _ = tx.send(Err(ResourceError::Io(e))).await;
                    return;
                }
            };

            // 加载所有块
            for chunk_index in 0..total_chunks {
                // 获取信号量许可
                let _permit = match semaphore.acquire().await {
                    Ok(p) => p,
                    Err(_) => {
                        let _ = tx
                            .send(Err(ResourceError::Other("Semaphore closed".to_string())))
                            .await;
                        return;
                    }
                };

                // 计算块偏移量和大小
                let offset = (chunk_index * config.chunk_size) as u64;
                let remaining = total_size - (chunk_index * config.chunk_size);
                let chunk_size = remaining.min(config.chunk_size);
                let is_last = chunk_index == total_chunks - 1;

                // 读取块数据
                let chunk_result = async {
                    file.seek(SeekFrom::Start(offset)).await.map_err(|e| ResourceError::Io(e))?;
                    let mut buffer = vec![0u8; chunk_size];
                    file.read_exact(&mut buffer).await.map_err(|e| ResourceError::Io(e))?;
                    Ok::<ResourceChunk, ResourceError>(ResourceChunk {
                        chunk_index,
                        data: buffer,
                        offset,
                        is_last,
                    })
                }
                .await;

                match chunk_result {
                    Ok(chunk) => {
                        let chunk_size = chunk.data.len();
                        loaded_size_clone.fetch_add(chunk_size, Ordering::Relaxed);
                        received_chunks_clone.fetch_add(1, Ordering::Relaxed);

                        if tx.send(Ok(chunk)).await.is_err() {
                            // 接收端已关闭，停止加载
                            break;
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                }
            }
        });

        Ok(StreamingHandle {
            receiver: rx,
            total_size,
            loaded_size,
            total_chunks,
            received_chunks,
            path: path_buf,
        })
    }

    /// 流式加载并组装完整资源
    ///
    /// 此方法会等待所有块加载完成，然后组装成完整的资源数据。
    ///
    /// # 参数
    /// - `path`: 资源路径
    ///
    /// # 返回
    /// 完整的资源数据
    pub async fn load_complete(&self, path: impl AsRef<Path>) -> Result<Vec<u8>, ResourceError> {
        let mut handle = self.load_streaming(path).await?;
        let mut data = Vec::new();

        while let Some(result) = handle.next_chunk().await {
            match result {
                Ok(chunk) => {
                    data.extend_from_slice(&chunk.data);
                    if chunk.is_last {
                        break;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Ok(data)
    }

    /// 获取配置
    pub fn config(&self) -> &StreamingConfig {
        &self.config
    }

    /// 设置配置
    pub fn set_config(&mut self, config: StreamingConfig) {
        self.config = config;
        self.semaphore = Arc::new(tokio::sync::Semaphore::new(config.max_concurrent));
    }
}

/// 渐进式质量加载器
///
/// 支持渐进式质量提升的资源加载，先加载低质量版本，然后逐步提升质量。
pub struct ProgressiveQualityLoader {
    /// 基础流式加载器
    streaming_loader: StreamingLoader,
    /// 质量级别数
    quality_levels: usize,
}

impl ProgressiveQualityLoader {
    /// 创建新的渐进式质量加载器
    ///
    /// # 参数
    /// - `config`: 流式加载配置
    /// - `quality_levels`: 质量级别数（例如：低、中、高）
    ///
    /// # 返回
    /// 新的渐进式质量加载器实例
    pub fn new(config: StreamingConfig, quality_levels: usize) -> Self {
        Self {
            streaming_loader: StreamingLoader::new(config),
            quality_levels: quality_levels.max(1),
        }
    }

    /// 加载指定质量级别的资源
    ///
    /// # 参数
    /// - `path`: 资源路径
    /// - `quality_level`: 质量级别（0 = 最低质量，quality_levels-1 = 最高质量）
    ///
    /// # 返回
    /// 流式加载句柄
    pub async fn load_quality(
        &self,
        path: impl AsRef<Path>,
        quality_level: usize,
    ) -> Result<StreamingHandle, ResourceError> {
        let quality_level = quality_level.min(self.quality_levels - 1);

        // 根据质量级别调整块大小
        // 低质量 = 大块（快速加载），高质量 = 小块（精细加载）
        let base_chunk_size = self.streaming_loader.config().chunk_size;
        let quality_factor =
            (self.quality_levels - quality_level) as f32 / self.quality_levels as f32;
        let adjusted_chunk_size = (base_chunk_size as f32 * quality_factor) as usize;

        let mut config = self.streaming_loader.config().clone();
        config.chunk_size = adjusted_chunk_size.max(1024); // 最小1KB

        let loader = StreamingLoader::new(config);
        loader.load_streaming(path).await
    }

    /// 获取质量级别数
    pub fn quality_levels(&self) -> usize {
        self.quality_levels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_streaming_load() {
        // 创建测试文件
        let mut file = NamedTempFile::new().unwrap();
        let data = vec![0u8; 5 * 1024 * 1024]; // 5MB
        file.write_all(&data).unwrap();
        let path = file.path();

        // 创建流式加载器
        let config = StreamingConfig {
            chunk_size: 1024 * 1024, // 1MB chunks
            max_concurrent: 2,
            prefetch_chunks: 1,
        };
        let loader = StreamingLoader::new(config);

        // 开始流式加载
        let mut handle = loader.load_streaming(path).await.unwrap();

        // 接收所有块
        let mut total_received = 0;
        while let Some(result) = handle.next_chunk().await {
            match result {
                Ok(chunk) => {
                    total_received += chunk.data.len();
                    if chunk.is_last {
                        break;
                    }
                }
                Err(e) => panic!("Error loading chunk: {:?}", e),
            }
        }

        assert_eq!(total_received, data.len());
        assert!(handle.is_complete());
        assert!((handle.progress() - 1.0).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_load_complete() {
        // 创建测试文件
        let mut file = NamedTempFile::new().unwrap();
        let original_data = b"Hello, World! This is a test file for streaming loader.";
        file.write_all(original_data).unwrap();
        let path = file.path();

        // 创建流式加载器
        let loader = StreamingLoader::with_default_config();

        // 加载完整资源
        let loaded_data = loader.load_complete(path).await.unwrap();

        assert_eq!(loaded_data, original_data);
    }
}
