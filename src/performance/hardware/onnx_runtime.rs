/// ONNX Runtime集成
/// 
/// 提供真实的NPU推理能力，支持多种硬件后端

use super::error::{HardwareError, HardwareResult};
use super::npu_sdk::{NpuInferenceEngine, NpuBackend, InferenceHandle, ModelFormat};
use ort::{Environment, ExecutionProvider, Session, SessionBuilder, Value};
use std::path::Path;
use std::sync::Arc;

/// ONNX Runtime推理引擎
pub struct OnnxRuntimeEngine {
    environment: Arc<Environment>,
    session: Option<Session>,
    input_shape: Vec<usize>,
    output_shape: Vec<usize>,
    backend: NpuBackend,
}

impl OnnxRuntimeEngine {
    /// 创建新的ONNX Runtime引擎
    pub fn new() -> HardwareResult<Self> {
        let environment = Environment::builder()
            .with_name("game_engine")
            .build()
            .map_err(|e| HardwareError::NpuAccelerationError(format!("Failed to create ONNX Runtime environment: {}", e)))?;
        
        Ok(Self {
            environment: Arc::new(environment),
            session: None,
            input_shape: Vec::new(),
            output_shape: Vec::new(),
            backend: NpuBackend::OnnxRuntime,
        })
    }
    
    /// 使用特定执行提供者创建引擎
    pub fn with_execution_provider(provider: ExecutionProvider) -> HardwareResult<Self> {
        let environment = Environment::builder()
            .with_name("game_engine")
            .with_execution_providers([provider])
            .build()
            .map_err(|e| HardwareError::NpuAccelerationError(format!("Failed to create ONNX Runtime environment: {}", e)))?;
        
        Ok(Self {
            environment: Arc::new(environment),
            session: None,
            input_shape: Vec::new(),
            output_shape: Vec::new(),
            backend: NpuBackend::OnnxRuntime,
        })
    }
    
    /// 创建CUDA执行提供者
    pub fn with_cuda(device_id: i32) -> HardwareResult<Self> {
        Self::with_execution_provider(
            ExecutionProvider::CUDA(Default::default())
        )
    }
    
    /// 创建TensorRT执行提供者
    pub fn with_tensorrt() -> HardwareResult<Self> {
        Self::with_execution_provider(
            ExecutionProvider::TensorRT(Default::default())
        )
    }
    
    /// 创建CoreML执行提供者（Apple平台）
    #[cfg(target_os = "macos")]
    pub fn with_coreml() -> HardwareResult<Self> {
        Self::with_execution_provider(
            ExecutionProvider::CoreML(Default::default())
        )
    }
    
    /// 创建DirectML执行提供者（Windows）
    #[cfg(target_os = "windows")]
    pub fn with_directml() -> HardwareResult<Self> {
        Self::with_execution_provider(
            ExecutionProvider::DirectML(Default::default())
        )
    }
    
    /// 获取输入张量的形状
    pub fn get_input_shape(&self) -> &[usize] {
        &self.input_shape
    }
    
    /// 获取输出张量的形状
    pub fn get_output_shape(&self) -> &[usize] {
        &self.output_shape
    }
}

impl Default for OnnxRuntimeEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create ONNX Runtime engine")
    }
}

impl NpuInferenceEngine for OnnxRuntimeEngine {
    fn load_model(&mut self, model_path: &Path) -> HardwareResult<()> {
        // 创建会话
        let session = SessionBuilder::new(&self.environment)
            .map_err(|e| HardwareError::NpuAccelerationError(format!("Failed to create session builder: {}", e)))?
            .with_model_from_file(model_path)
            .map_err(|e| HardwareError::ModelLoadError(format!("Failed to load model: {}", e)))?;
        
        // 获取输入输出形状信息
        if let Some(input) = session.inputs.first() {
            if let Some(shape) = &input.dimensions {
                self.input_shape = shape.iter()
                    .map(|&dim| if dim < 0 { 1 } else { dim as usize })
                    .collect();
            }
        }
        
        if let Some(output) = session.outputs.first() {
            if let Some(shape) = &output.dimensions {
                self.output_shape = shape.iter()
                    .map(|&dim| if dim < 0 { 1 } else { dim as usize })
                    .collect();
            }
        }
        
        self.session = Some(session);
        Ok(())
    }
    
    fn infer(&self, input: &[f32]) -> HardwareResult<Vec<f32>> {
        let session = self.session.as_ref()
            .ok_or_else(|| HardwareError::NpuAccelerationError("Model not loaded".to_string()))?;
        
        // 创建输入张量
        let input_shape: Vec<i64> = self.input_shape.iter().map(|&x| x as i64).collect();
        let input_tensor = Value::from_array(
            session.allocator(),
            ndarray::Array::from_shape_vec(
                self.input_shape.as_slice(),
                input.to_vec()
            ).map_err(|e| HardwareError::NpuAccelerationError(format!("Failed to create input tensor: {}", e)))?
        ).map_err(|e| HardwareError::NpuAccelerationError(format!("Failed to create input value: {}", e)))?;
        
        // 运行推理
        let outputs = session.run(vec![input_tensor])
            .map_err(|e| HardwareError::NpuAccelerationError(format!("Inference failed: {}", e)))?;
        
        // 提取输出
        let output = outputs.first()
            .ok_or_else(|| HardwareError::NpuAccelerationError("No output from inference".to_string()))?;
        
        let output_tensor = output.try_extract::<f32>()
            .map_err(|e| HardwareError::NpuAccelerationError(format!("Failed to extract output: {}", e)))?;
        
        Ok(output_tensor.view().iter().copied().collect())
    }
    
    fn infer_async(&self, input: &[f32]) -> HardwareResult<InferenceHandle> {
        // ONNX Runtime当前不支持真正的异步推理
        // 这里我们同步执行并立即返回
        let _result = self.infer(input)?;
        Ok(InferenceHandle { backend: self.backend })
    }
    
    fn infer_batch(&self, inputs: &[&[f32]]) -> HardwareResult<Vec<Vec<f32>>> {
        inputs.iter().map(|input| self.infer(input)).collect()
    }
    
    fn input_shape(&self) -> &[usize] {
        &self.input_shape
    }
    
    fn output_shape(&self) -> &[usize] {
        &self.output_shape
    }
    
    fn warmup(&mut self) -> HardwareResult<()> {
        // 执行一次推理以预热
        if !self.input_shape.is_empty() {
            let dummy_input = vec![0.0f32; self.input_shape.iter().product()];
            let _ = self.infer(&dummy_input)?;
        }
        Ok(())
    }
    
    fn backend(&self) -> NpuBackend {
        self.backend
    }
}

/// ONNX Runtime管理器
pub struct OnnxRuntimeManager {
    engines: Vec<Box<dyn NpuInferenceEngine>>,
}

impl OnnxRuntimeManager {
    /// 创建新的管理器
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
        }
    }
    
    /// 自动检测并创建最优的ONNX Runtime引擎
    pub fn create_optimal_engine() -> HardwareResult<Box<dyn NpuInferenceEngine>> {
        // 尝试按优先级创建执行提供者
        
        // 1. 尝试CUDA (NVIDIA GPU)
        #[cfg(feature = "cuda")]
        if let Ok(engine) = OnnxRuntimeEngine::with_cuda(0) {
            println!("Using ONNX Runtime with CUDA");
            return Ok(Box::new(engine));
        }
        
        // 2. 尝试TensorRT (NVIDIA GPU优化)
        #[cfg(feature = "tensorrt")]
        if let Ok(engine) = OnnxRuntimeEngine::with_tensorrt() {
            println!("Using ONNX Runtime with TensorRT");
            return Ok(Box::new(engine));
        }
        
        // 3. 尝试CoreML (Apple平台)
        #[cfg(target_os = "macos")]
        if let Ok(engine) = OnnxRuntimeEngine::with_coreml() {
            println!("Using ONNX Runtime with CoreML");
            return Ok(Box::new(engine));
        }
        
        // 4. 尝试DirectML (Windows)
        #[cfg(target_os = "windows")]
        if let Ok(engine) = OnnxRuntimeEngine::with_directml() {
            println!("Using ONNX Runtime with DirectML");
            return Ok(Box::new(engine));
        }
        
        // 5. 回退到CPU
        println!("Using ONNX Runtime with CPU");
        Ok(Box::new(OnnxRuntimeEngine::new()?))
    }
    
    /// 添加引擎
    pub fn add_engine(&mut self, engine: Box<dyn NpuInferenceEngine>) {
        self.engines.push(engine);
    }
    
    /// 获取引擎数量
    pub fn engine_count(&self) -> usize {
        self.engines.len()
    }
}

impl Default for OnnxRuntimeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_onnx_runtime_creation() {
        let result = OnnxRuntimeEngine::new();
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_optimal_engine_creation() {
        let result = OnnxRuntimeManager::create_optimal_engine();
        assert!(result.is_ok());
    }
}
