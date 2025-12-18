use super::{ConfigError, ConfigResult};
use crate::impl_default;
use serde::{Deserialize, Serialize};

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 目标帧率
    pub target_fps: u32,

    /// 自动优化
    pub auto_optimize: bool,

    /// SIMD优化
    pub simd: SimdConfig,

    /// NPU加速
    pub npu: NpuConfig,

    /// 多线程配置
    pub threading: ThreadingConfig,

    /// 内存管理
    pub memory: MemoryConfig,
}

impl_default!(PerformanceConfig {
    target_fps: 60,
    auto_optimize: true,
    simd: SimdConfig::default(),
    npu: NpuConfig::default(),
    threading: ThreadingConfig::default(),
    memory: MemoryConfig::default(),
});

impl PerformanceConfig {
    /// 验证配置
    pub fn validate(&self) -> ConfigResult<()> {
        if self.target_fps == 0 || self.target_fps > 1000 {
            return Err(ConfigError::ValidationError(
                "Invalid target FPS".to_string(),
            ));
        }
        Ok(())
    }
}

/// SIMD配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimdConfig {
    /// 是否启用
    pub enabled: bool,

    /// 强制使用特定指令集
    pub force_instruction_set: Option<String>,

    /// 批量处理大小
    pub batch_size: usize,
}

impl_default!(SimdConfig {
    enabled: true,
    force_instruction_set: None,
    batch_size: 1000,
});

/// NPU配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpuConfig {
    /// 是否启用
    pub enabled: bool,

    /// 后端选择
    pub backend: NpuBackend,

    /// AI超分辨率
    pub ai_upscaling: bool,

    /// 物理预测
    pub physics_prediction: bool,
}

impl_default!(NpuConfig {
    enabled: true,
    backend: NpuBackend::Auto,
    ai_upscaling: false,
    physics_prediction: false,
});

/// NPU后端
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NpuBackend {
    /// 自动选择
    Auto,
    /// ONNX Runtime
    OnnxRuntime,
    /// TensorRT
    TensorRT,
    /// Core ML
    CoreML,
    /// DirectML
    DirectML,
}

/// 多线程配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadingConfig {
    /// 工作线程数（0表示自动）
    pub worker_threads: usize,

    /// 渲染线程数
    pub render_threads: usize,

    /// 物理线程数
    pub physics_threads: usize,
}

impl_default!(ThreadingConfig {
    worker_threads: 0, // 自动
    render_threads: 1,
    physics_threads: 1,
});

/// 内存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// 纹理缓存大小（MB）
    pub texture_cache_mb: usize,

    /// 模型缓存大小（MB）
    pub model_cache_mb: usize,

    /// 音频缓存大小（MB）
    pub audio_cache_mb: usize,

    /// 使用对象池
    pub use_object_pools: bool,
}

impl_default!(MemoryConfig {
    texture_cache_mb: 512,
    model_cache_mb: 256,
    audio_cache_mb: 128,
    use_object_pools: true,
});
