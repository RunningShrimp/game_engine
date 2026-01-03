//! # 计算后端管理
//!
//! 自动检测和选择最佳计算后端（CPU/GPU/NPU），提供统一的回退机制。

use crate::compute::rocm::GpuComputeBackend;
use crate::acceleration::npus::NPUDeviceType;

// ============================================================================
// 计算后端类型
// ============================================================================

/// 计算后端类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeBackend {
    /// CPU计算
    Cpu,

    /// GPU计算（wgpu/Vulkan/Metal/DX12）
    Gpu,

    /// NPU计算（Apple Neural Engine/NNAPI）
    Npu,

    /// 无后端
    None,
}

impl ComputeBackend {
    /// 获取后端名称
    pub fn name(&self) -> &'static str {
        match self {
            ComputeBackend::Cpu => "CPU",
            ComputeBackend::Gpu => "GPU",
            ComputeBackend::Npu => "NPU",
            ComputeBackend::None => "None",
        }
    }

    /// 是否为硬件加速
    pub fn is_hardware_accelerated(&self) -> bool {
        matches!(self, ComputeBackend::Gpu | ComputeBackend::Npu)
    }

    /// 获取性能等级（0-100）
    pub fn performance_tier(&self) -> u8 {
        match self {
            ComputeBackend::Cpu => 30,
            ComputeBackend::Gpu => 90,
            ComputeBackend::Npu => 100,
            ComputeBackend::None => 0,
        }
    }
}

// ============================================================================
// 计算管理器
// ============================================================================

/// 计算管理器 - 自动选择和管理计算后端
pub struct ComputeManager {
    /// 可用的后端列表
    available_backends: Vec<ComputeBackend>,

    /// 当前选中的后端
    current_backend: ComputeBackend,

    /// GPU后端
    gpu_backend: Option<GpuComputeBackend>,

    /// NPU设备类型
    npu_device: Option<NPUDeviceType>,
}

impl ComputeManager {
    /// 创建新的计算管理器（自动检测最佳后端）
    pub fn new() -> Result<Self, ComputeError> {
        tracing::info!("Initializing compute manager (auto-detecting best backend)");

        let mut available_backends = Vec::new();

        // 检测GPU
        let gpu_backend = GpuComputeBackend::auto_detect();
        if gpu_backend.is_available() {
            available_backends.push(ComputeBackend::Gpu);
            tracing::info!("GPU backend detected: {}", gpu_backend.name());
        }

        // 检测NPU
        let npu_device = NPUDeviceType::detect_best_device();
        if npu_device.is_hardware_accelerated() {
            available_backends.push(ComputeBackend::Npu);
            tracing::info!("NPU backend detected: {}", npu_device.name());
        }

        // CPU总是可用
        available_backends.push(ComputeBackend::Cpu);

        // 选择最佳后端
        let current_backend = Self::select_best_backend(&available_backends);

        tracing::info!(
            "Available backends: {:?}, selected: {}",
            available_backends,
            current_backend.name()
        );

        Ok(Self {
            available_backends,
            current_backend,
            gpu_backend: Some(gpu_backend),
            npu_device: Some(npu_device),
        })
    }

    /// 选择最佳后端
    fn select_best_backend(available: &[ComputeBackend]) -> ComputeBackend {
        // 优先级: NPU > GPU > CPU
        if available.contains(&ComputeBackend::Npu) {
            ComputeBackend::Npu
        } else if available.contains(&ComputeBackend::Gpu) {
            ComputeBackend::Gpu
        } else {
            ComputeBackend::Cpu
        }
    }

    /// 获取当前后端
    pub fn current_backend(&self) -> ComputeBackend {
        self.current_backend
    }

    /// 设置后端
    pub fn set_backend(&mut self, backend: ComputeBackend) -> Result<(), ComputeError> {
        if !self.available_backends.contains(&backend) {
            return Err(ComputeError::BackendNotAvailable {
                backend: backend.name().to_string(),
                available: self
                    .available_backends
                    .iter()
                    .map(|b| b.name().to_string())
                    .collect(),
            });
        }

        tracing::info!("Switching compute backend from {} to {}", self.current_backend.name(), backend.name());

        self.current_backend = backend;
        Ok(())
    }

    /// 回退到CPU
    pub fn fallback_to_cpu(&mut self) -> Result<(), ComputeError> {
        tracing::warn!("Falling back to CPU backend");

        self.current_backend = ComputeBackend::Cpu;
        Ok(())
    }

    /// 检查GPU是否可用
    pub fn is_gpu_available(&self) -> bool {
        self.available_backends.contains(&ComputeBackend::Gpu)
            && self.gpu_backend.as_ref().map(|b| b.is_available()).unwrap_or(false)
    }

    /// 检查NPU是否可用
    pub fn is_npu_available(&self) -> bool {
        self.available_backends
            .contains(&ComputeBackend::Npu)
            && self
                .npu_device
                .as_ref()
                .map(|d| d.is_hardware_accelerated())
                .unwrap_or(false)
    }

    /// 获取GPU信息
    pub fn get_gpu_info(&self) -> Option<GpuInfo> {
        self.gpu_backend.as_ref().map(|backend| GpuInfo {
            name: backend.name().to_string(),
            is_available: backend.is_available(),
            performance_tier: backend.performance_tier(),
        })
    }

    /// 获取NPU信息
    pub fn get_npu_info(&self) -> Option<NpuInfo> {
        self.npu_device.as_ref().map(|device| NpuInfo {
            name: device.name().to_string(),
            is_hardware_accelerated: device.is_hardware_accelerated(),
        })
    }

    /// 获取系统信息
    pub fn get_system_info(&self) -> ComputeSystemInfo {
        ComputeSystemInfo {
            current_backend: self.current_backend,
            available_backends: self.available_backends.clone(),
            gpu_info: self.get_gpu_info(),
            npu_info: self.get_npu_info(),
        }
    }
}

impl Default for ComputeManager {
    fn default() -> Self {
        // 默认创建，如果失败则使用CPU
        Self::new().unwrap_or_else(|_| Self {
            available_backends: vec![ComputeBackend::Cpu],
            current_backend: ComputeBackend::Cpu,
            gpu_backend: None,
            npu_device: None,
        })
    }
}

// ============================================================================
// 信息结构
// ============================================================================

/// GPU信息
#[derive(Debug, Clone)]
pub struct GpuInfo {
    /// GPU名称
    pub name: String,

    /// 是否可用
    pub is_available: bool,

    /// 性能等级
    pub performance_tier: u8,
}

/// NPU信息
#[derive(Debug, Clone)]
pub struct NpuInfo {
    /// NPU名称
    pub name: String,

    /// 是否为硬件加速
    pub is_hardware_accelerated: bool,
}

/// 计算系统信息
#[derive(Debug, Clone)]
pub struct ComputeSystemInfo {
    /// 当前后端
    pub current_backend: ComputeBackend,

    /// 可用后端
    pub available_backends: Vec<ComputeBackend>,

    /// GPU信息
    pub gpu_info: Option<GpuInfo>,

    /// NPU信息
    pub npu_info: Option<NpuInfo>,
}

// ============================================================================
// 错误类型
// ============================================================================

/// 计算错误
#[derive(Debug, thiserror::Error)]
pub enum ComputeError {
    /// 后端不可用
    #[error("Backend '{backend}' not available. Available: {available:?}")]
    BackendNotAvailable {
        backend: String,
        available: Vec<String>,
    },

    /// 初始化失败
    #[error("Failed to initialize backend: {0}")]
    InitializationFailed(String),

    /// 后端切换失败
    #[error("Failed to switch backend: {0}")]
    SwitchFailed(String),
}

// ============================================================================
// 辅助trait
// ============================================================================

/// GpuComputeBackend的辅助trait（如果rocm模块没有）
pub trait GpuBackendInfo {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn performance_tier(&self) -> u8;
}

// 为GpuComputeBackend实现辅助trait（如果需要）
impl GpuBackendInfo for GpuComputeBackend {
    fn name(&self) -> &str {
        match self {
            GpuComputeBackend::Wgpu => "WebGPU (wgpu)",
            GpuComputeBackend::Cuda(_) => "CUDA (NVIDIA)",
            GpuComputeBackend::Rocm(_) => "ROCm (AMD)",
            GpuComputeBackend::None => "None",
        }
    }

    fn is_available(&self) -> bool {
        !matches!(self, GpuComputeBackend::None)
    }

    fn performance_tier(&self) -> u8 {
        match self {
            GpuComputeBackend::Wgpu => 85,
            GpuComputeBackend::Cuda(_) => 95,
            GpuComputeBackend::Rocm(_) => 90,
            GpuComputeBackend::None => 0,
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
    fn test_compute_manager_creation() {
        let manager = ComputeManager::new();

        assert!(manager.is_ok());

        if let Ok(manager) = manager {
            let info = manager.get_system_info();

            println!("Current backend: {}", info.current_backend.name());
            println!("Available backends: {:?}", info.available_backends);

            // CPU应该总是可用
            assert!(info.available_backends.contains(&ComputeBackend::Cpu));
        }
    }

    #[test]
    fn test_backend_selection() {
        let backends = vec![ComputeBackend::Cpu, ComputeBackend::Gpu, ComputeBackend::Npu];
        let selected = ComputeManager::select_best_backend(&backends);

        // 应该选择NPU
        assert_eq!(selected, ComputeBackend::Npu);
    }

    #[test]
    fn test_backend_fallback() {
        let backends = vec![ComputeBackend::Cpu];
        let selected = ComputeManager::select_best_backend(&backends);

        // 应该选择CPU
        assert_eq!(selected, ComputeBackend::Cpu);
    }

    #[test]
    fn test_performance_tier() {
        assert_eq!(ComputeBackend::Cpu.performance_tier(), 30);
        assert_eq!(ComputeBackend::Gpu.performance_tier(), 90);
        assert_eq!(ComputeBackend::Npu.performance_tier(), 100);
    }
}
