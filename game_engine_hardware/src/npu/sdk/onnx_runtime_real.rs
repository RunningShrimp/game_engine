// 占位符模块 - 实验性NPU SDK集成在未来阶段实现
use crate::error::HardwareResult;
use crate::npu::sdk::{NpuInferenceEngine, InferenceHandle, NpuBackend};
use std::path::Path;

pub struct OnnxRuntimeEngineReal {
    input_shape: Vec<usize>,
    output_shape: Vec<usize>,
}

impl OnnxRuntimeEngineReal {
    pub fn new() -> HardwareResult<Self> {
        Ok(Self {
            input_shape: vec![1, 3, 224, 224],
            output_shape: vec![1, 1000],
        })
    }
}

impl NpuInferenceEngine for OnnxRuntimeEngineReal {
    fn load_model(&mut self, _model_path: &Path) -> HardwareResult<()> { Ok(()) }
    fn infer(&self, _input: &[f32]) -> HardwareResult<Vec<f32>> { Ok(vec![]) }
    fn infer_async(&self, _input: &[f32]) -> HardwareResult<InferenceHandle> { 
        Ok(InferenceHandle { backend: NpuBackend::OnnxRuntime })
    }
    fn infer_batch(&self, _inputs: &[&[f32]]) -> HardwareResult<Vec<Vec<f32>>> { Ok(vec![]) }
    fn input_shape(&self) -> &[usize] { &self.input_shape }
    fn output_shape(&self) -> &[usize] { &self.output_shape }
    fn warmup(&mut self) -> HardwareResult<()> { Ok(()) }
    fn backend(&self) -> NpuBackend { NpuBackend::OnnxRuntime }
}
