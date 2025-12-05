//! NPU检测和加速模块

pub mod detect;
pub mod acceleration;
pub mod upscaling;
pub mod sdk;

pub use detect::{NpuInfo, NpuVendor, detect_npu};
pub use acceleration::{NpuAccelerator, PhysicsPrediction, BehaviorDecision};
pub use upscaling::{NpuUpscalingEngine, NpuUpscalingManager, HybridUpscalingStrategy, AiUpscalingModel};
pub use sdk::extended::{OpenVINOEngine, ROCmEngine, AscendEngine, SNPEEngine, NeuroPilotEngine};

