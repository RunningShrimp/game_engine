//  GPU检测和优化模块

pub mod detect;
pub mod optimization;
pub mod vendor_optimization;

pub use detect::{GpuInfo, GpuTier, GpuVendor, detect_gpu};
pub use optimization::{GpuOptimization, PipelineMode, TextureCompressionFormat};
pub use vendor_optimization::{GpuOptimizationConfig, GpuOptimizer};
