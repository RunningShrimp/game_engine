//! 渲染系统错误类型
//!
//! 定义了渲染系统相关的所有错误类型，包括GPU操作、着色器编译、纹理创建等。

use crate::error::{ErrorSeverity, ErrorCategory};
use thiserror::Error;

/// 渲染系统错误
///
/// 涵盖了渲染管线中的所有可能的错误情况，
/// 从GPU初始化到具体的渲染操作。
#[derive(Error, Debug, Clone)]
pub enum RenderError {
    /// GPU适配器错误
    #[error("GPU adapter error: {message}")]
    Adapter {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 设备创建错误
    #[error("Device creation failed: {message}")]
    DeviceCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 表面创建错误
    #[error("Surface creation failed: {message}")]
    SurfaceCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 着色器编译错误
    #[error("Shader compilation failed: {shader} - {message}")]
    ShaderCompilation {
        /// 着色器名称或路径
        shader: String,
        /// 编译错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 管线创建错误
    #[error("Pipeline creation failed: {message}")]
    PipelineCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 纹理创建错误
    #[error("Texture creation failed: {texture} - {message}")]
    TextureCreation {
        /// 纹理名称或路径
        texture: String,
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 缓冲区创建错误
    #[error("Buffer creation failed: {message}")]
    BufferCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 绑定组创建错误
    #[error("Bind group creation failed: {message}")]
    BindGroupCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 渲染通道错误
    #[error("Render pass error: {message}")]
    RenderPass {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 帧提交错误
    #[error("Frame submission failed: {message}")]
    FrameSubmission {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 顶点缓冲区错误
    #[error("Vertex buffer error: {message}")]
    VertexBuffer {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 索引缓冲区错误
    #[error("Index buffer error: {message}")]
    IndexBuffer {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 统一缓冲区错误
    #[error("Uniform buffer error: {message}")]
    UniformBuffer {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 采样器创建错误
    #[error("Sampler creation failed: {message}")]
    SamplerCreation {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 渲染状态错误
    #[error("Invalid render state: {message}")]
    InvalidState {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// GPU内存不足
    #[error("GPU out of memory: {message}")]
    OutOfMemory {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// GPU超时
    #[error("GPU timeout: {message}")]
    Timeout {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 交换链错误
    #[error("Swap chain error: {message}")]
    SwapChain {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 视窗错误
    #[error("Viewport error: {message}")]
    Viewport {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 剔除错误
    #[error("Culling error: {message}")]
    Culling {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// LOD错误
    #[error("LOD error: {message}")]
    Lod {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// PBR渲染错误
    #[error("PBR rendering error: {message}")]
    PbrRendering {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 阴影渲染错误
    #[error("Shadow rendering error: {message}")]
    ShadowRendering {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 后处理错误
    #[error("Post-processing error: {message}")]
    PostProcessing {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },

    /// 通用渲染错误
    #[error("Render error: {message}")]
    General {
        /// 错误消息
        message: String,
        /// 错误严重级别
        severity: ErrorSeverity,
    },
}

impl RenderError {
    /// 创建GPU适配器错误
    pub fn adapter(message: impl Into<String>) -> Self {
        Self::Adapter {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建设备创建错误
    pub fn device_creation(message: impl Into<String>) -> Self {
        Self::DeviceCreation {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建表面创建错误
    pub fn surface_creation(message: impl Into<String>) -> Self {
        Self::SurfaceCreation {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建着色器编译错误
    pub fn shader_compilation(
        shader: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::ShaderCompilation {
            shader: shader.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建管线创建错误
    pub fn pipeline_creation(message: impl Into<String>) -> Self {
        Self::PipelineCreation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建纹理创建错误
    pub fn texture_creation(
        texture: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::TextureCreation {
            texture: texture.into(),
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建缓冲区创建错误
    pub fn buffer_creation(message: impl Into<String>) -> Self {
        Self::BufferCreation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建绑定组创建错误
    pub fn bind_group_creation(message: impl Into<String>) -> Self {
        Self::BindGroupCreation {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建渲染通道错误
    pub fn render_pass(message: impl Into<String>) -> Self {
        Self::RenderPass {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建帧提交错误
    pub fn frame_submission(message: impl Into<String>) -> Self {
        Self::FrameSubmission {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建GPU内存不足错误
    pub fn out_of_memory(message: impl Into<String>) -> Self {
        Self::OutOfMemory {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建GPU超时错误
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
            severity: ErrorSeverity::Critical,
        }
    }

    /// 创建通用渲染错误
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            severity: ErrorSeverity::Error,
        }
    }

    /// 创建带有严重级别的通用渲染错误
    pub fn general_with_severity(
        message: impl Into<String>,
        severity: ErrorSeverity,
    ) -> Self {
        Self::General {
            message: message.into(),
            severity,
        }
    }

    /// 获取错误的严重级别
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            RenderError::Adapter { severity, .. }
            | RenderError::DeviceCreation { severity, .. }
            | RenderError::SurfaceCreation { severity, .. }
            | RenderError::ShaderCompilation { severity, .. }
            | RenderError::PipelineCreation { severity, .. }
            | RenderError::TextureCreation { severity, .. }
            | RenderError::BufferCreation { severity, .. }
            | RenderError::BindGroupCreation { severity, .. }
            | RenderError::RenderPass { severity, .. }
            | RenderError::FrameSubmission { severity, .. }
            | RenderError::VertexBuffer { severity, .. }
            | RenderError::IndexBuffer { severity, .. }
            | RenderError::UniformBuffer { severity, .. }
            | RenderError::SamplerCreation { severity, .. }
            | RenderError::InvalidState { severity, .. }
            | RenderError::OutOfMemory { severity, .. }
            | RenderError::Timeout { severity, .. }
            | RenderError::SwapChain { severity, .. }
            | RenderError::Viewport { severity, .. }
            | RenderError::Culling { severity, .. }
            | RenderError::Lod { severity, .. }
            | RenderError::PbrRendering { severity, .. }
            | RenderError::ShadowRendering { severity, .. }
            | RenderError::PostProcessing { severity, .. }
            | RenderError::General { severity, .. } => *severity,
        }
    }

    /// 检查错误是否可恢复
    pub fn is_recoverable(&self) -> bool {
        match self {
            // 严重错误通常不可恢复
            RenderError::Adapter { severity, .. }
            | RenderError::DeviceCreation { severity, .. }
            | RenderError::SurfaceCreation { severity, .. }
            | RenderError::OutOfMemory { severity, .. }
            | RenderError::Timeout { severity, .. } => *severity < ErrorSeverity::Critical,

            // 编译错误通常不可恢复
            RenderError::ShaderCompilation { .. } => false,

            // 其他错误通常可恢复
            _ => true,
        }
    }

    /// 获取错误分类
    pub fn category(&self) -> ErrorCategory {
        ErrorCategory::Render
    }

    /// 检查是否为GPU相关错误
    pub fn is_gpu_related(&self) -> bool {
        matches!(
            self,
            RenderError::Adapter { .. }
                | RenderError::DeviceCreation { .. }
                | RenderError::OutOfMemory { .. }
                | RenderError::Timeout { .. }
                | RenderError::BufferCreation { .. }
                | RenderError::TextureCreation { .. }
                | RenderError::PipelineCreation { .. }
                | RenderError::BindGroupCreation { .. }
                | RenderError::RenderPass { .. }
                | RenderError::FrameSubmission { .. }
        )
    }

    /// 检查是否为资源相关错误
    pub fn is_resource_related(&self) -> bool {
        matches!(
            self,
            RenderError::TextureCreation { .. }
                | RenderError::BufferCreation { .. }
                | RenderError::BindGroupCreation { .. }
                | RenderError::SamplerCreation { .. }
                | RenderError::VertexBuffer { .. }
                | RenderError::IndexBuffer { .. }
                | RenderError::UniformBuffer { .. }
        )
    }

    /// 检查是否为着色器相关错误
    pub fn is_shader_related(&self) -> bool {
        matches!(self, RenderError::ShaderCompilation { .. })
    }
}

// 从wgpu错误转换
impl From<wgpu::Error> for RenderError {
    fn from(err: wgpu::Error) -> Self {
        match err {
            wgpu::Error::OutOfMemory { .. } => {
                RenderError::out_of_memory(err.to_string())
            }
            wgpu::Error::Validation { .. } => {
                RenderError::general_with_severity(err.to_string(), ErrorSeverity::Error)
            }
            wgpu::Error::Lost => {
                RenderError::general_with_severity(err.to_string(), ErrorSeverity::Critical)
            }
            _ => RenderError::general(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_error_creation() {
        let err = RenderError::shader_compilation("vertex.wgsl", "Compilation failed");
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.is_shader_related());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_render_error_severity() {
        let critical_err = RenderError::out_of_memory("GPU memory exhausted");
        assert_eq!(critical_err.severity(), ErrorSeverity::Critical);
        assert!(!critical_err.is_recoverable());

        let normal_err = RenderError::general("Temporary render issue");
        assert_eq!(normal_err.severity(), ErrorSeverity::Error);
        assert!(normal_err.is_recoverable());
    }

    #[test]
    fn test_render_error_categories() {
        let texture_err = RenderError::texture_creation("diffuse.png", "Invalid format");
        assert!(texture_err.is_resource_related());

        let shader_err = RenderError::shader_compilation("frag.wgsl", "Syntax error");
        assert!(shader_err.is_shader_related());

        let gpu_err = RenderError::out_of_memory("GPU memory full");
        assert!(gpu_err.is_gpu_related());
    }

    #[test]
    fn test_from_wgpu_error() {
        let wgpu_err = wgpu::Error::OutOfMemory {
            source: Box::new(std::io::Error::new(std::io::ErrorKind::OutOfMemory, "OOM")),
        };
        let render_err: RenderError = wgpu_err.into();
        
        assert!(matches!(render_err, RenderError::OutOfMemory { .. }));
        assert_eq!(render_err.severity(), ErrorSeverity::Critical);
    }
}