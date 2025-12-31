//! CUDA/ROCm计算加速模块
//!
//! GPU计算加速的统一接口，支持CUDA和ROCm。
//!
//! ## GPU加速架构
//!
//! 游戏引擎使用多层GPU加速架构：
//!
//! 1. **wgpu (WebGPU)** - 跨平台计算着色器（主要实现）
//!    - 支持Vulkan、Metal、DX12、WebGL后端
//!    - 完整的物理模拟、粒子系统、碰撞检测
//!    - WGSL计算着色器
//!
//! 2. **CUDA (NVIDIA)** - NVIDIA特定优化（可选）
//!    - 需要CUDA工具包
//!    - 提供额外10-30%性能提升
//!    - Tensor Cores优化
//!
//! 3. **ROCm (AMD)** - AMD特定优化（可选）
//!    - 需要ROCm工具包
//!    - HIP兼容层
//!    - CDNA/RDNA优化
//!
//! ## 推荐使用方式
//!
//! - **大多数情况**: 使用wgpu（默认启用）
//! - **NVIDIA GPU**: 可选启用CUDA以获得额外性能
//! - **AMD GPU**: 可选启用ROCm以获得额外性能
//!
//! ## 示例
//!
//! ```rust
//! use game_engine::compute::{GpuCapabilities, GpuVendor};
//!
//! // 检测GPU能力
//! let caps = GpuCapabilities::detect();
//! println!("{}", caps);
//!
//! // 查看优化建议
//! for hint in &caps.physics_optimizations {
//!     println!("{}: {:.1}%提升", hint.description, hint.estimated_improvement);
//! }
//! ```

pub mod cuda;
pub mod rocm;
pub mod gpu_capabilities;

pub use cuda::{CudaContext, CudaPhysicsSystem, CudaParticleSystem, CudaMeshProcessor, CudaError};
pub use rocm::{RocmContext, RocmPhysicsSystem, GpuComputeBackend};
pub use gpu_capabilities::{GpuCapabilities, GpuVendor, GpuArchitecture, OptimizationHint, OptimizationType};

use bevy_ecs::prelude::*;

/// GPU计算系统 - 自动选择最佳后端
#[derive(Component)]
pub struct GpuComputeSystem {
    /// GPU后端
    backend: rocm::GpuComputeBackend,

    /// CUDA系统
    cuda_system: Option<cuda::CudaPhysicsSystem>,

    /// ROCm系统
    rocm_system: Option<rocm::RocmPhysicsSystem>,
}

impl GpuComputeSystem {
    /// 创建新的GPU计算系统
    pub fn new() -> Self {
        let backend = rocm::GpuComputeBackend::auto_detect();

        let (cuda_system, rocm_system) = match &backend {
            rocm::GpuComputeBackend::Cuda(_) => (
                Some(cuda::CudaPhysicsSystem::new()),
                None,
            ),
            rocm::GpuComputeBackend::Rocm(_) => (
                None,
                Some(rocm::RocmPhysicsSystem::new()),
            ),
            rocm::GpuComputeBackend::None => (
                Some(cuda::CudaPhysicsSystem::new()),
                Some(rocm::RocmPhysicsSystem::new()),
            ),
        };

        Self {
            backend,
            cuda_system,
            rocm_system,
        }
    }

    /// 获取设备属性
    pub fn get_device_properties(&self) -> Option<cuda::CudaDeviceProperties> {
        self.backend.get_device_properties()
    }

    /// 是否可用
    pub fn is_available(&self) -> bool {
        self.backend.is_available()
    }
}

impl Default for GpuComputeSystem {
    fn default() -> Self {
        Self::new()
    }
}
