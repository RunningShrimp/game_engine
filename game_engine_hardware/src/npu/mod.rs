//  NPU检测和加速模块

pub mod acceleration;
pub mod detect;
pub mod sdk;
pub mod upscaling;

pub use acceleration::{BehaviorDecision, NpuAccelerator, PhysicsPrediction};
pub use detect::{NpuInfo, NpuVendor, detect_npu};
pub use sdk::extended::{AscendEngine, NeuroPilotEngine, OpenVINOEngine, ROCmEngine, SNPEEngine};
pub use upscaling::{
    AiUpscalingModel, HybridUpscalingStrategy, NpuUpscalingEngine, NpuUpscalingManager,
};
