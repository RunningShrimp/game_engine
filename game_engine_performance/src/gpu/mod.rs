pub mod gpu_compute;
pub mod gpu_physics;
pub mod wgpu_integration;

pub use gpu_compute::{
    ComputePipeline, ComputeResourceManager, ComputeShaderConfig, ComputeShaderGenerator,
};
pub use gpu_physics::{
    GPUCollisionInfo, GPUConstraint, GPUParticleSystem, GPUPhysicsBody, GPUPhysicsConfig,
    GPUPhysicsSimulator,
};
pub use wgpu_integration::{
    ComputePipelineWGPU, GPUBuffer, GPUComputeDevice, GPUExecutionResult, GPUFeatures,
    PerformanceComparison, WGSLShader,
};

