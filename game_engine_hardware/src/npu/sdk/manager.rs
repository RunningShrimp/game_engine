/// 统一的SDK管理器
/// 
/// 提供统一的接口来管理和使用各种NPU SDK

use crate::error::{HardwareError, HardwareResult};
use super::super::sdk::{NpuInferenceEngine, NpuBackend};
use crate::gpu::detect::{detect_gpu, GpuVendor};
use crate::npu::detect::{detect_npu, NpuVendor};
use crate::soc::detect::{detect_soc, SocVendor};
use std::path::Path;

/// SDK管理器
/// 
/// 自动检测硬件并选择最优的SDK
pub struct SdkManager {
    available_backends: Vec<NpuBackend>,
    preferred_backend: Option<NpuBackend>,
}

impl SdkManager {
    /// 创建新的SDK管理器
    /// 
    /// 自动检测可用的硬件和SDK
    pub fn new() -> HardwareResult<Self> {
        let mut manager = Self {
            available_backends: Vec::new(),
            preferred_backend: None,
        };
        
        manager.detect_available_backends()?;
        manager.select_preferred_backend();
        
        Ok(manager)
    }
    
    /// 检测可用的后端
    fn detect_available_backends(&mut self) -> HardwareResult<()> {
        // 检测GPU
        let gpu_info = detect_gpu();
        {
            match gpu_info.vendor {
                GpuVendor::Nvidia => {
                    // NVIDIA GPU可以使用ONNX Runtime with CUDA或TensorRT
                    self.available_backends.push(NpuBackend::OnnxRuntime);
                }
                GpuVendor::Amd => {
                    // AMD GPU可以使用ROCm
                    self.available_backends.push(NpuBackend::ROCm);
                }
                GpuVendor::Intel => {
                    // Intel GPU可以使用OpenVINO
                    self.available_backends.push(NpuBackend::OpenVINO);
                }
                GpuVendor::Apple => {
                    // Apple GPU可以使用Core ML
                    self.available_backends.push(NpuBackend::CoreML);
                }
                _ => {}
            }
        }
        
        // 检测NPU
        if let Some(npu_info) = detect_npu() {
            match npu_info.vendor {
                NpuVendor::QualcommHexagon => {
                    self.available_backends.push(NpuBackend::SNPE);
                }
                NpuVendor::MediaTekApu => {
                    self.available_backends.push(NpuBackend::NeuroPilot);
                }
                NpuVendor::HuaweiAscend => {
                    self.available_backends.push(NpuBackend::Ascend);
                }
                NpuVendor::AppleNeuralEngine => {
                    self.available_backends.push(NpuBackend::CoreML);
                }
                _ => {}
            }
        }
        
        // 检测SoC
        if let Some(soc_info) = detect_soc() {
            match soc_info.vendor {
                SocVendor::Qualcomm => {
                    if !self.available_backends.contains(&NpuBackend::SNPE) {
                        self.available_backends.push(NpuBackend::SNPE);
                    }
                }
                SocVendor::MediaTek => {
                    if !self.available_backends.contains(&NpuBackend::NeuroPilot) {
                        self.available_backends.push(NpuBackend::NeuroPilot);
                    }
                }
                SocVendor::Apple => {
                    if !self.available_backends.contains(&NpuBackend::CoreML) {
                        self.available_backends.push(NpuBackend::CoreML);
                    }
                }
                SocVendor::HiSilicon => {
                    if !self.available_backends.contains(&NpuBackend::Ascend) {
                        self.available_backends.push(NpuBackend::Ascend);
                    }
                }
                _ => {}
            }
        }
        
        // ONNX Runtime作为通用后备
        if !self.available_backends.contains(&NpuBackend::OnnxRuntime) {
            self.available_backends.push(NpuBackend::OnnxRuntime);
        }
        
        Ok(())
    }
    
    /// 选择首选后端
    fn select_preferred_backend(&mut self) {
        // 优先级：专用NPU > GPU加速 > 通用ONNX Runtime
        let priority_order = [
            NpuBackend::CoreML,          // Apple Neural Engine
            NpuBackend::Ascend,           // 华为昇腾NPU
            NpuBackend::SNPE,         // 高通Hexagon DSP
            NpuBackend::NeuroPilot,   // 联发科APU
            NpuBackend::OpenVINO,        // Intel VPU/GPU
            NpuBackend::ROCm,              // AMD GPU
            NpuBackend::OnnxRuntime,          // 通用后备
        ];
        
        for backend in &priority_order {
            if self.available_backends.contains(backend) {
                self.preferred_backend = Some(*backend);
                break;
            }
        }
    }
    
    /// 获取可用的后端列表
    pub fn available_backends(&self) -> &[NpuBackend] {
        &self.available_backends
    }
    
    /// 获取首选后端
    pub fn preferred_backend(&self) -> Option<NpuBackend> {
        self.preferred_backend
    }
    
    /// 创建推理引擎
    /// 
    /// 使用首选后端创建引擎
    pub fn create_engine(&self) -> HardwareResult<Box<dyn NpuInferenceEngine>> {
        let backend = self.preferred_backend.ok_or_else(|| {
            HardwareError::NpuAccelerationError {
                operation: "create_engine".to_string(),
                reason: "No available backend".to_string(),
            }
        })?;
        
        self.create_engine_with_backend(backend)
    }
    
    /// 使用指定后端创建推理引擎
    pub fn create_engine_with_backend(
        &self,
        backend: NpuBackend,
    ) -> HardwareResult<Box<dyn NpuInferenceEngine>> {
        if !self.available_backends.contains(&backend) {
            return Err(HardwareError::NpuAccelerationError {
                operation: "create_engine_with_backend".to_string(),
                reason: format!("Backend {:?} is not available", backend),
            });
        }
        
        match backend {
            NpuBackend::OnnxRuntime => {
                use super::onnx_runtime_real::OnnxRuntimeEngineReal;
                Ok(Box::new(OnnxRuntimeEngineReal::new()?))
            }
            NpuBackend::OpenVINO => {
                use crate::npu::sdk::extended::OpenVINOEngine as OpenVinoEngine;
                Ok(Box::new(OpenVinoEngine::new()?))
            }
            NpuBackend::ROCm => {
                use crate::npu::sdk::extended::ROCmEngine as RocmEngine;
                Ok(Box::new(RocmEngine::new()?))
            }
            NpuBackend::Ascend => {
                use crate::npu::sdk::extended::AscendEngine as CannEngine;
                Ok(Box::new(CannEngine::new()?))
            }
            NpuBackend::SNPE => {
                use crate::npu::sdk::extended::SNPEEngine as SnpeEngine;
                Ok(Box::new(SnpeEngine::new()?))
            }
            NpuBackend::NeuroPilot => {
                use crate::npu::sdk::extended::NeuroPilotEngine;
                Ok(Box::new(NeuroPilotEngine::new()?))
            }
            NpuBackend::CoreML => {
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                {
                    use super::CoreMLEngine;
                    Ok(Box::new(CoreMLEngine::new()?))
                }
                #[cfg(not(any(target_os = "macos", target_os = "ios")))]
                {
                    Err(HardwareError::UnsupportedPlatform {
                        platform: std::env::consts::OS.to_string(),
                        feature: "Core ML".to_string(),
                    })
                }
            }
            _ => Err(HardwareError::NpuAccelerationError {
                operation: "create_engine_with_backend".to_string(),
                reason: format!("Backend {:?} is not yet implemented", backend),
            }),
        }
    }
    
    /// 打印可用后端信息
    pub fn print_info(&self) {
        println!("=== SDK Manager Info ===");
        println!();
        println!("Available backends:");
        for (i, backend) in self.available_backends.iter().enumerate() {
            let marker = if Some(*backend) == self.preferred_backend {
                "✓ (preferred)"
            } else {
                " "
            };
            println!("  {}. {:?} {}", i + 1, backend, marker);
        }
        println!();
        if let Some(backend) = self.preferred_backend {
            println!("Preferred backend: {:?}", backend);
        } else {
            println!("No preferred backend selected");
        }
        println!();
    }
}

impl Default for SdkManager {
    fn default() -> Self {
        Self::new().expect("Failed to create SDK manager")
    }
}

/// 便捷函数：自动选择并创建推理引擎
/// 
/// # 示例
/// 
/// ```rust,ignore
/// use crate::sdk_manager::auto_create_engine;
/// 
/// let mut engine = auto_create_engine()?;
/// engine.load_model("model.onnx")?;
/// let output = engine.infer(&input)?;
/// ```
pub fn auto_create_engine() -> HardwareResult<Box<dyn NpuInferenceEngine>> {
    let manager = SdkManager::new()?;
    manager.create_engine()
}

/// 便捷函数：加载模型并创建引擎
/// 
/// # 示例
/// 
/// ```rust,ignore
/// use crate::sdk_manager::load_model;
/// 
/// let engine = load_model("model.onnx")?;
/// let output = engine.infer(&input)?;
/// ```
pub fn load_model(model_path: impl AsRef<Path>) -> HardwareResult<Box<dyn NpuInferenceEngine>> {
    let mut engine = auto_create_engine()?;
    engine.load_model(model_path.as_ref())?;
    Ok(engine)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_manager_creation() {
        let result = SdkManager::new();
        assert!(result.is_ok());
        
        let manager = result.unwrap();
        assert!(!manager.available_backends().is_empty());
    }
    
    #[test]
    fn test_preferred_backend() {
        let manager = SdkManager::new().unwrap();
        assert!(manager.preferred_backend().is_some());
    }
}
