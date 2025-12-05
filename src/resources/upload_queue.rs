//! 异步上传队列模块
//!
//! 管理 CPU→GPU 数据传输，支持纹理、缓冲区等资源的异步上传。
//!
//! ## 使用示例
//!
//! ```ignore
//! let mut upload_queue = UploadQueue::new();
//!
//! // 队列纹理上传
//! upload_queue.queue_texture(
//!     &texture_data,
//!     &gpu_texture,
//!     TextureUploadInfo { width: 512, height: 512, format: TextureFormat::Rgba8Unorm },
//! );
//!
//! // 队列缓冲区上传
//! upload_queue.queue_buffer(&vertex_data, &gpu_buffer, 0);
//!
//! // 在渲染循环中刷新
//! upload_queue.flush(device, queue, encoder);
//! ```

use super::staging_buffer::StagingBufferPool;

// ============================================================================
// 上传请求
// ============================================================================

/// 纹理上传信息
#[derive(Clone, Debug)]
pub struct TextureUploadInfo {
    /// 纹理宽度
    pub width: u32,
    /// 纹理高度
    pub height: u32,
    /// 纹理深度（用于 3D 纹理）
    pub depth: u32,
    /// 像素格式
    pub format: wgpu::TextureFormat,
    /// Mip 层级
    pub mip_level: u32,
    /// 目标层（用于数组纹理）
    pub array_layer: u32,
    /// 目标原点
    pub origin: wgpu::Origin3d,
}

impl Default for TextureUploadInfo {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            depth: 1,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            mip_level: 0,
            array_layer: 0,
            origin: wgpu::Origin3d::ZERO,
        }
    }
}

impl TextureUploadInfo {
    pub fn new() -> Self {
        Self::default()
    }
}

/// 缓冲区上传请求
struct BufferUploadRequest {
    /// 数据
    data: Vec<u8>,
    /// 目标缓冲区
    target: wgpu::Buffer,
    /// 目标偏移
    target_offset: u64,
}

/// 纹理上传请求
struct TextureUploadRequest {
    /// 数据
    data: Vec<u8>,
    /// 目标纹理
    target: wgpu::Texture,
    /// 上传信息
    info: TextureUploadInfo,
}

/// 上传请求类型
enum UploadRequest {
    Buffer(BufferUploadRequest),
    Texture(TextureUploadRequest),
}

// ============================================================================
// 上传队列
// ============================================================================

/// 上传统计信息
#[derive(Default, Clone, Copy, Debug)]
pub struct UploadStats {
    /// 本帧缓冲区上传数
    pub buffer_uploads: u32,
    /// 本帧纹理上传数
    pub texture_uploads: u32,
    /// 本帧上传总字节数
    pub bytes_uploaded: u64,
    /// 总上传次数
    pub total_uploads: u64,
}

/// 异步上传队列
#[derive(Default)]
pub struct UploadQueue {
    /// 待处理的上传请求
    pub(crate) pending: Vec<UploadRequest>,
    /// Staging Buffer 池
    pub(crate) staging_pool: StagingBufferPool,
    /// 统计信息
    pub(crate) stats: UploadStats,
}

impl UploadQueue {
    pub fn new() -> Self {
        Self::default()
    }

    /// 初始化（需要在首次使用前调用）
    pub fn initialize(&mut self, device: &wgpu::Device) {
        self.staging_pool.initialize(device);
    }

    /// 队列缓冲区上传
    ///
    /// # 参数
    /// - `data`: 要上传的数据
    /// - `target`: 目标 GPU 缓冲区
    /// - `target_offset`: 目标缓冲区中的偏移
    pub fn queue_buffer(&mut self, data: &[u8], target: wgpu::Buffer, target_offset: u64) {
        self.pending
            .push(UploadRequest::Buffer(BufferUploadRequest {
                data: data.to_vec(),
                target,
                target_offset,
            }));

        self.stats.buffer_uploads += 1;
        self.stats.bytes_uploaded += data.len() as u64;
    }

    /// 队列纹理上传
    ///
    /// # 参数
    /// - `data`: 像素数据
    /// - `target`: 目标纹理
    /// - `info`: 上传配置信息
    pub fn queue_texture(&mut self, data: &[u8], target: wgpu::Texture, info: TextureUploadInfo) {
        self.pending
            .push(UploadRequest::Texture(TextureUploadRequest {
                data: data.to_vec(),
                target,
                info,
            }));

        self.stats.texture_uploads += 1;
        self.stats.bytes_uploaded += data.len() as u64;
    }

    /// 队列纹理上传（简化版本）
    pub fn queue_texture_simple(
        &mut self,
        data: &[u8],
        target: wgpu::Texture,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) {
        self.queue_texture(
            data,
            target,
            TextureUploadInfo {
                width,
                height,
                format,
                ..Default::default()
            },
        );
    }

    /// 检查是否有待处理的上传
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// 获取待处理上传数量
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// 刷新所有待处理的上传
    ///
    /// 将所有队列中的上传请求提交到 GPU
    pub fn flush(
        &mut self,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        if self.pending.is_empty() {
            return;
        }

        // 先取出所有请求避免借用冲突
        let requests: Vec<_> = self.pending.drain(..).collect();

        // 使用queue提交所有上传命令（通过staging buffer）
        for request in requests {
            match request {
                UploadRequest::Buffer(req) => {
                    Self::execute_buffer_upload_static(
                        &mut self.staging_pool,
                        device,
                        encoder,
                        req,
                    );
                }
                UploadRequest::Texture(req) => {
                    Self::execute_texture_upload_static(
                        &mut self.staging_pool,
                        device,
                        encoder,
                        req,
                    );
                }
            }
        }

        self.stats.total_uploads += 1;
    }

    /// 直接刷新（使用 queue.write_buffer/write_texture，无需 encoder）
    ///
    /// 适用于小量数据的即时上传
    pub fn flush_immediate(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for request in self.pending.drain(..) {
            match request {
                UploadRequest::Buffer(req) => {
                    queue.write_buffer(&req.target, req.target_offset, &req.data);
                }
                UploadRequest::Texture(req) => {
                    let bytes_per_pixel = req.info.format.block_copy_size(None).unwrap_or(4);
                    let bytes_per_row = req.info.width * bytes_per_pixel;

                    queue.write_texture(
                        wgpu::ImageCopyTexture {
                            texture: &req.target,
                            mip_level: req.info.mip_level,
                            origin: req.info.origin,
                            aspect: wgpu::TextureAspect::All,
                        },
                        &req.data,
                        wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(bytes_per_row),
                            rows_per_image: Some(req.info.height),
                        },
                        wgpu::Extent3d {
                            width: req.info.width,
                            height: req.info.height,
                            depth_or_array_layers: req.info.depth,
                        },
                    );
                }
            }
        }

        self.stats.total_uploads += 1;
    }

    /// 执行缓冲区上传（静态方法）
    fn execute_buffer_upload_static(
        staging_pool: &mut StagingBufferPool,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        request: BufferUploadRequest,
    ) {
        // 分配 Staging Buffer
        let (buffer_idx, offset) = staging_pool.allocate(
            device,
            request.data.len() as u64,
            wgpu::COPY_BUFFER_ALIGNMENT,
        );

        // 写入数据到 Staging Buffer
        if let Some(staging) = staging_pool.get_buffer_mut(buffer_idx) {
            let _ = staging.write(&request.data, wgpu::COPY_BUFFER_ALIGNMENT);
            staging.unmap();

            // 复制到目标缓冲区
            encoder.copy_buffer_to_buffer(
                &staging.buffer,
                offset,
                &request.target,
                request.target_offset,
                request.data.len() as u64,
            );
        }
    }

    /// 执行纹理上传（静态方法）
    fn execute_texture_upload_static(
        staging_pool: &mut StagingBufferPool,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        request: TextureUploadRequest,
    ) {
        let bytes_per_pixel = request.info.format.block_copy_size(None).unwrap_or(4);
        let bytes_per_row = request.info.width * bytes_per_pixel;

        // 确保行对齐到 256 字节（wgpu 要求）
        let aligned_bytes_per_row = align_to_256(bytes_per_row);

        // 分配 Staging Buffer
        let total_size = aligned_bytes_per_row as u64 * request.info.height as u64;
        let (buffer_idx, offset) = staging_pool.allocate(device, total_size, 256);

        // 写入数据到 Staging Buffer（需要处理行对齐）
        if let Some(staging) = staging_pool.get_buffer_mut(buffer_idx) {
            // 如果需要对齐，逐行复制
            if bytes_per_row != aligned_bytes_per_row {
                let mut padded_data = vec![0u8; total_size as usize];
                for y in 0..request.info.height as usize {
                    let src_start = y * bytes_per_row as usize;
                    let dst_start = y * aligned_bytes_per_row as usize;
                    let src_end = src_start + bytes_per_row as usize;
                    if src_end <= request.data.len() {
                        padded_data[dst_start..dst_start + bytes_per_row as usize]
                            .copy_from_slice(&request.data[src_start..src_end]);
                    }
                }
                let _ = staging.write(&padded_data, 256);
            } else {
                let _ = staging.write(&request.data, 256);
            }

            staging.unmap();

            // 复制到目标纹理
            encoder.copy_buffer_to_texture(
                wgpu::ImageCopyBuffer {
                    buffer: &staging.buffer,
                    layout: wgpu::ImageDataLayout {
                        offset,
                        bytes_per_row: Some(aligned_bytes_per_row),
                        rows_per_image: Some(request.info.height),
                    },
                },
                wgpu::ImageCopyTexture {
                    texture: &request.target,
                    mip_level: request.info.mip_level,
                    origin: request.info.origin,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::Extent3d {
                    width: request.info.width,
                    height: request.info.height,
                    depth_or_array_layers: request.info.depth,
                },
            );
        }
    }

    /// 帧结束时调用，回收资源
    pub fn end_frame(&mut self, device: &wgpu::Device) {
        self.staging_pool.end_frame(device);

        // 重置本帧统计
        self.stats.buffer_uploads = 0;
        self.stats.texture_uploads = 0;
        self.stats.bytes_uploaded = 0;
    }

    /// 获取统计信息
    pub fn stats(&self) -> UploadStats {
        self.stats
    }

    /// 获取 Staging 池统计
    pub fn staging_stats(&self) -> super::staging_buffer::PoolStats {
        self.staging_pool.stats()
    }
}

// ============================================================================
// 辅助函数
// ============================================================================

/// 对齐到 256 字节边界（wgpu 纹理行要求）
#[inline]
fn align_to_256(value: u32) -> u32 {
    (value + 255) & !255
}

// ============================================================================
// 便捷构建器
// ============================================================================

/// 纹理上传构建器
pub struct TextureUploadBuilder {
    data: Vec<u8>,
    target: Option<wgpu::Texture>,
    info: TextureUploadInfo,
}

impl TextureUploadBuilder {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            target: None,
            info: TextureUploadInfo::default(),
        }
    }

    pub fn target(mut self, texture: wgpu::Texture) -> Self {
        self.target = Some(texture);
        self
    }

    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.info.width = width;
        self.info.height = height;
        self
    }

    pub fn format(mut self, format: wgpu::TextureFormat) -> Self {
        self.info.format = format;
        self
    }

    pub fn mip_level(mut self, level: u32) -> Self {
        self.info.mip_level = level;
        self
    }

    pub fn array_layer(mut self, layer: u32) -> Self {
        self.info.array_layer = layer;
        self
    }

    pub fn origin(mut self, x: u32, y: u32, z: u32) -> Self {
        self.info.origin = wgpu::Origin3d { x, y, z };
        self
    }

    pub fn build(self, queue: &mut UploadQueue) {
        if let Some(target) = self.target {
            queue.queue_texture(&self.data, target, self.info);
        }
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_align_to_256() {
        assert_eq!(align_to_256(0), 0);
        assert_eq!(align_to_256(1), 256);
        assert_eq!(align_to_256(255), 256);
        assert_eq!(align_to_256(256), 256);
        assert_eq!(align_to_256(257), 512);
        assert_eq!(align_to_256(512), 512);
    }

    #[test]
    fn test_upload_queue_new() {
        let queue = UploadQueue::new();
        assert!(!queue.has_pending());
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_texture_upload_info_default() {
        let info = TextureUploadInfo::default();
        assert_eq!(info.width, 1);
        assert_eq!(info.height, 1);
        assert_eq!(info.depth, 1);
        assert_eq!(info.mip_level, 0);
    }
}
