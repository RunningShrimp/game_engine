//! 渲染后端抽象
//!
//! 提供渲染后端 trait 定义，支持未来多后端扩展。
//!
//! ## 设计目标
//!
//! - 统一不同渲染 API 的接口
//! - 支持 wgpu、Vulkan、Metal 等后端
//! - 易于扩展和测试

/// 缓冲区描述符
#[derive(Debug, Clone)]
pub struct BufferDescriptor {
    /// 标签
    pub label: Option<String>,
    /// 大小（字节）
    pub size: wgpu::BufferAddress,
    /// 用途
    pub usage: BufferUsage,
    /// 是否映射创建
    pub mapped_at_creation: bool,
}

/// 缓冲区用途
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferUsage(pub u32);

impl BufferUsage {
    pub const VERTEX: Self = Self(1);
    pub const INDEX: Self = Self(2);
    pub const UNIFORM: Self = Self(4);
    pub const STORAGE: Self = Self(8);
    pub const COPY_SRC: Self = Self(16);
    pub const COPY_DST: Self = Self(32);

    pub fn contains(&self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }
}

impl std::ops::BitOr for BufferUsage {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// 纹理描述符
#[derive(Debug, Clone)]
pub struct TextureDescriptor {
    /// 标签
    pub label: Option<String>,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 深度/数组层数
    pub depth_or_array_layers: u32,
    /// Mip 级别数
    pub mip_level_count: u32,
    /// 采样数
    pub sample_count: u32,
    /// 格式
    pub format: TextureFormat,
    /// 用途
    pub usage: TextureUsage,
}

/// 纹理格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureFormat {
    Rgba8Unorm,
    Rgba8UnormSrgb,
    Rgba16Float,
    Rgba32Float,
    Depth32Float,
    Depth24PlusStencil8,
}

/// 纹理用途
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureUsage(pub u32);

impl TextureUsage {
    pub const COPY_SRC: Self = Self(1);
    pub const COPY_DST: Self = Self(2);
    pub const TEXTURE_BINDING: Self = Self(4);
    pub const STORAGE_BINDING: Self = Self(8);
    pub const RENDER_ATTACHMENT: Self = Self(16);
}

impl std::ops::BitOr for TextureUsage {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

/// 渲染命令
#[derive(Debug, Clone)]
pub enum RenderCommand {
    /// 开始渲染通道
    BeginRenderPass {
        color_attachments: Vec<ColorAttachment>,
        depth_attachment: Option<DepthAttachment>,
    },
    /// 结束渲染通道
    EndRenderPass,
    /// 设置管线
    SetPipeline { pipeline_id: u64 },
    /// 设置绑定组
    SetBindGroup { index: u32, bind_group_id: u64 },
    /// 设置顶点缓冲区
    SetVertexBuffer { slot: u32, buffer_id: u64 },
    /// 设置索引缓冲区
    SetIndexBuffer { buffer_id: u64, format: IndexFormat },
    /// 绘制
    Draw {
        vertex_count: u32,
        instance_count: u32,
    },
    /// 索引绘制
    DrawIndexed {
        index_count: u32,
        instance_count: u32,
    },
    /// 设置裁剪区域
    SetScissorRect {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
    /// 设置视口
    SetViewport {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        min_depth: f32,
        max_depth: f32,
    },
}

/// 颜色附件
#[derive(Debug, Clone)]
pub struct ColorAttachment {
    pub texture_id: u64,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub clear_color: [f32; 4],
}

/// 深度附件
#[derive(Debug, Clone)]
pub struct DepthAttachment {
    pub texture_id: u64,
    pub load_op: LoadOp,
    pub store_op: StoreOp,
    pub clear_depth: f32,
}

/// 加载操作
#[derive(Debug, Clone, Copy)]
pub enum LoadOp {
    Load,
    Clear,
}

/// 存储操作
#[derive(Debug, Clone, Copy)]
pub enum StoreOp {
    Store,
    Discard,
}

/// 索引格式
#[derive(Debug, Clone, Copy)]
pub enum IndexFormat {
    Uint16,
    Uint32,
}

/// 抽象缓冲区句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferHandle(pub u64);

/// 抽象纹理句柄
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u64);

/// 渲染后端 Trait
///
/// 定义渲染后端需要实现的接口。
///
/// # 示例
///
/// ```ignore
/// struct MyBackend { ... }
///
/// impl RenderBackend for MyBackend {
///     fn create_buffer(&self, desc: &BufferDescriptor) -> BufferHandle { ... }
///     fn create_texture(&self, desc: &TextureDescriptor) -> TextureHandle { ... }
///     fn submit(&mut self, commands: &[RenderCommand]) { ... }
/// }
/// ```
pub trait RenderBackend: Send + Sync {
    /// 创建缓冲区
    fn create_buffer(&self, desc: &BufferDescriptor) -> BufferHandle;

    /// 销毁缓冲区
    fn destroy_buffer(&self, buffer: BufferHandle);

    /// 写入缓冲区数据
    fn write_buffer(&self, buffer: BufferHandle, offset: u64, data: &[u8]);

    /// 创建纹理
    fn create_texture(&self, desc: &TextureDescriptor) -> TextureHandle;

    /// 销毁纹理
    fn destroy_texture(&self, texture: TextureHandle);

    /// 写入纹理数据
    fn write_texture(&self, texture: TextureHandle, data: &[u8], width: u32, height: u32);

    /// 提交渲染命令
    fn submit(&mut self, commands: &[RenderCommand]);

    /// 呈现帧
    fn present(&mut self);

    /// 获取后端名称
    fn name(&self) -> &str;

    /// 获取后端能力
    fn capabilities(&self) -> BackendCapabilities;
}

/// 后端能力
#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    /// 最大纹理尺寸
    pub max_texture_size: u32,
    /// 最大缓冲区大小
    pub max_buffer_size: wgpu::BufferAddress,
    /// 是否支持计算着色器
    pub compute_shaders: bool,
    /// 是否支持光线追踪
    pub ray_tracing: bool,
    /// 是否支持网格着色器
    pub mesh_shaders: bool,
    /// 最大绑定组数
    pub max_bind_groups: u32,
}

impl Default for BackendCapabilities {
    fn default() -> Self {
        Self {
            max_texture_size: 8192,
            max_buffer_size: 256 * 1024 * 1024,
            compute_shaders: true,
            ray_tracing: false,
            mesh_shaders: false,
            max_bind_groups: 4,
        }
    }
}

/// 空后端实现（用于测试）
pub struct NullBackend {
    next_buffer_id: std::sync::atomic::AtomicU64,
    next_texture_id: std::sync::atomic::AtomicU64,
}

impl Default for NullBackend {
    fn default() -> Self {
        Self {
            next_buffer_id: std::sync::atomic::AtomicU64::new(1),
            next_texture_id: std::sync::atomic::AtomicU64::new(1),
        }
    }
}

impl NullBackend {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RenderBackend for NullBackend {
    fn create_buffer(&self, _desc: &BufferDescriptor) -> BufferHandle {
        BufferHandle(
            self.next_buffer_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        )
    }

    fn destroy_buffer(&self, _buffer: BufferHandle) {}

    fn write_buffer(&self, _buffer: BufferHandle, _offset: u64, _data: &[u8]) {}

    fn create_texture(&self, _desc: &TextureDescriptor) -> TextureHandle {
        TextureHandle(
            self.next_texture_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        )
    }

    fn destroy_texture(&self, _texture: TextureHandle) {}

    fn write_texture(&self, _texture: TextureHandle, _data: &[u8], _width: u32, _height: u32) {}

    fn submit(&mut self, _commands: &[RenderCommand]) {}

    fn present(&mut self) {}

    fn name(&self) -> &str {
        "null"
    }

    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_usage_bitor() {
        let usage = BufferUsage::VERTEX | BufferUsage::COPY_DST;
        assert!(usage.contains(BufferUsage::VERTEX));
        assert!(usage.contains(BufferUsage::COPY_DST));
        assert!(!usage.contains(BufferUsage::UNIFORM));
    }

    #[test]
    fn test_null_backend() {
        let mut backend = NullBackend::new();

        let buffer = backend.create_buffer(&BufferDescriptor {
            label: None,
            size: 1024,
            usage: BufferUsage::VERTEX,
            mapped_at_creation: false,
        });
        assert!(buffer.0 > 0);

        let texture = backend.create_texture(&TextureDescriptor {
            label: None,
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
            mip_level_count: 1,
            sample_count: 1,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsage::TEXTURE_BINDING,
        });
        assert!(texture.0 > 0);

        backend.submit(&[]);
        backend.present();

        assert_eq!(backend.name(), "null");
    }
}
