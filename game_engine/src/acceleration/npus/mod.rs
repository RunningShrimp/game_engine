//! # NPU (Neural Processing Unit) 加速支持
//!
//! 提供统一的NPU加速接口，支持：
//! - Apple Neural Engine (ANE)
//! - Android NNAPI
//! - 通用CPU/GPU fallback
//!
//! ## 架构概览
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │         Game Engine Layer               │
//! └─────────────────┬───────────────────────┘
//!                   │
//! ┌─────────────────▼───────────────────────┐
//! │       NPU Abstraction Layer             │
//! │  - NPURuntime                           │
//! │  - NPUModel                             │
//! │  - NPUTensor                            │
//! └─────────┬───────────────┬───────────────┘
//!           │               │
//!    ┌──────▼─────┐   ┌───▼────────┐
//!    │  Apple NE  │   │  Android   │
//!    │  (CoreML)  │   │  (NNAPI)   │
//!    └────────────┘   └────────────┘
//!           │               │
//!    ┌──────▼─────┐   ┌───▼────────┐
//!    │   Metal    │   │   Neuron   │
//!    │  MPS APIs  │   │   API      │
//!    └────────────┘   └────────────┘
//! ```
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::acceleration::npus::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建NPU运行时
//!     let runtime = NPURuntime::new().await?;
//!
//!     // 加载模型
//!     let model = runtime.load_model("models/my_model.mlmodel").await?;
//!
//!     // 准备输入
//!     let input = NPUTensor::from_vec(vec![1.0, 2.0, 3.0], &[3]);
//!
//!     // 推理
//!     let output = model.inference(&[input]).await?;
//!
//!     // 获取结果
//!     println!("Output: {:?}", output.to_vec::<f32>());
//!     Ok(())
//! }
//! ```

#[cfg(target_os = "macos")]
pub mod apple_neural_engine;

#[cfg(target_os = "android")]
pub mod android_nnapi;

pub mod common;
pub mod fallback;

pub use common::*;
pub use fallback::*;

use std::sync::Arc;

// ============================================================================
// NPU设备类型
// ============================================================================

/// NPU设备类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NPUDeviceType {
    /// Apple Neural Engine (macOS/iOS)
    AppleNeuralEngine,
    /// Android NNAPI
    AndroidNNAPI,
    /// CPU fallback
    CPU,
    /// GPU fallback
    GPU,
    /// 无NPU支持
    None,
}

impl NPUDeviceType {
    /// 检测当前平台可用的最佳NPU设备
    pub fn detect_best_device() -> Self {
        #[cfg(target_os = "macos")]
        {
            if Self::is_ane_available() {
                return NPUDeviceType::AppleNeuralEngine;
            }
        }

        #[cfg(target_os = "android")]
        {
            if Self::is_nnapi_available() {
                return NPUDeviceType::AndroidNNAPI;
            }
        }

        // Fallback to CPU
        NPUDeviceType::CPU
    }

    /// 检查Apple Neural Engine是否可用（macOS）
    #[cfg(target_os = "macos")]
    fn is_ane_available() -> bool {
        // 检查是否支持Metal Performance Shaders
        // 简化实现：假设macOS 11.0+支持ANE
        if let Ok(version) = std::env::var("MACOSX_DEPLOYMENT_TARGET") {
            return version.as_str() >= "11.0";
        }
        true // 假设支持
    }

    /// 检查NNAPI是否可用（Android）
    #[cfg(target_os = "android")]
    fn is_nnapi_available() -> bool {
        // 检查Android API版本（NNAPI需要API 27+）
        if let Ok(level) = std::env::var("ANDROID_API_LEVEL") {
            return level.as_str() >= "27";
        }
        true // 假设支持
    }

    #[cfg(not(any(target_os = "macos", target_os = "android")))]
    fn is_ane_available() -> bool {
        false
    }

    #[cfg(not(any(target_os = "macos", target_os = "android")))]
    fn is_nnapi_available() -> bool {
        false
    }

    /// 获取设备名称
    pub fn name(&self) -> &'static str {
        match self {
            NPUDeviceType::AppleNeuralEngine => "Apple Neural Engine",
            NPUDeviceType::AndroidNNAPI => "Android NNAPI",
            NPUDeviceType::CPU => "CPU (Fallback)",
            NPUDeviceType::GPU => "GPU (Fallback)",
            NPUDeviceType::None => "No NPU Support",
        }
    }

    /// 是否为硬件加速设备
    pub fn is_hardware_accelerated(&self) -> bool {
        matches!(
            self,
            NPUDeviceType::AppleNeuralEngine | NPUDeviceType::AndroidNNAPI
        )
    }
}

// ============================================================================
// NPU运行时
// ============================================================================

/// NPU运行时
pub struct NPURuntime {
    /// 设备类型
    device_type: NPUDeviceType,
    /// 内部实现
    inner: Arc<dyn NPURuntimeImpl>,
}

impl NPURuntime {
    /// 创建新的NPU运行时
    pub async fn new() -> Result<Self, NPUError> {
        let device_type = NPUDeviceType::detect_best_device();

        tracing::info!(
            "Initializing NPU runtime with device: {}",
            device_type.name()
        );

        let inner: Arc<dyn NPURuntimeImpl> = match device_type {
            #[cfg(target_os = "macos")]
            NPUDeviceType::AppleNeuralEngine => {
                Arc::new(apple_neural_engine::AppleNERuntime::new()?)
            }

            #[cfg(target_os = "android")]
            NPUDeviceType::AndroidNNAPI => Arc::new(android_nnapi::NNAPIRuntime::new()?),

            _ => Arc::new(fallback::CPURuntime::new()?),
        };

        Ok(Self { device_type, inner })
    }

    /// 获取设备类型
    pub fn device_type(&self) -> NPUDeviceType {
        self.device_type
    }

    /// 加载模型
    pub async fn load_model(&self, model_path: &str) -> Result<NPUModel, NPUError> {
        self.inner.load_model(model_path).await
    }

    /// 获取设备性能信息
    pub fn get_device_info(&self) -> NPUDeviceInfo {
        self.inner.get_device_info()
    }
}

// ============================================================================
// NPU模型
// ============================================================================

/// NPU模型
pub struct NPUModel {
    /// 模型名称
    pub name: String,
    /// 输入规格
    pub input_spec: Vec<TensorSpec>,
    /// 输出规格
    pub output_spec: Vec<TensorSpec>,
    /// 内部实现
    inner: Arc<dyn NPUModelImpl>,
}

impl NPUModel {
    /// 执行推理
    pub async fn inference(&self, inputs: &[NPUTensor]) -> Result<Vec<NPUTensor>, NPUError> {
        self.inner.inference(inputs).await
    }

    /// 获取输入规格
    pub fn input_spec(&self) -> &[TensorSpec] {
        &self.input_spec
    }

    /// 获取输出规格
    pub fn output_spec(&self) -> &[TensorSpec] {
        &self.output_spec
    }
}

// ============================================================================
// NPU张量
// ============================================================================

/// NPU张量
#[derive(Debug, Clone)]
pub struct NPUTensor {
    /// 数据
    pub data: TensorData,
    /// 形状
    pub shape: Vec<usize>,
    /// 数据类型
    pub dtype: TensorDType,
    /// 张量名称
    pub name: Option<String>,
}

/// 张量数据
#[derive(Debug, Clone)]
pub enum TensorData {
    /// FP32数据
    Float32(Vec<f32>),
    /// FP16数据
    Float16(Vec<u16>),
    /// INT8数据
    Int8(Vec<i8>),
    /// UINT8数据
    UInt8(Vec<u8>),
    /// INT32数据（用于LLM tokens）
    Int32(Vec<i32>),
}

/// 张量数据类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TensorDType {
    Float32,
    Float16,
    Int8,
    UInt8,
    Int32,
}

impl NPUTensor {
    /// 从Vec创建张量
    pub fn from_vec<T>(data: Vec<T>, shape: &[usize]) -> Self
    where
        Vec<T>: IntoTensorData,
        T: FromTensorData,
    {
        Self {
            data: data.into_tensor_data(),
            shape: shape.to_vec(),
            dtype: T::dtype(),
            name: None,
        }
    }

    /// 转换为Vec（仅支持相同类型）
    pub fn to_vec<T: FromTensorData + Copy + 'static>(&self) -> Option<Vec<T>> {
        match (&self.data, T::dtype()) {
            (TensorData::Float32(data), TensorDType::Float32) => {
                // T应该是f32
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<f32>() {
                    unsafe {
                        Some(data.iter().map(|&v| std::mem::transmute_copy::<f32, T>(&v)).collect())
                    }
                } else {
                    None
                }
            }
            (TensorData::Int8(data), TensorDType::Int8) => {
                // T应该是i8
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<i8>() {
                    unsafe {
                        Some(data.iter().map(|&v| std::mem::transmute_copy::<i8, T>(&v)).collect())
                    }
                } else {
                    None
                }
            }
            (TensorData::UInt8(data), TensorDType::UInt8) => {
                // T应该是u8
                if std::any::TypeId::of::<T>() == std::any::TypeId::of::<u8>() {
                    unsafe {
                        Some(data.iter().map(|&v| std::mem::transmute_copy::<u8, T>(&v)).collect())
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// 转换为张量数据的trait
pub trait IntoTensorData {
    fn into_tensor_data(self) -> TensorData;
}

impl IntoTensorData for Vec<f32> {
    fn into_tensor_data(self) -> TensorData {
        TensorData::Float32(self)
    }
}

impl IntoTensorData for Vec<i8> {
    fn into_tensor_data(self) -> TensorData {
        TensorData::Int8(self)
    }
}

impl IntoTensorData for Vec<u8> {
    fn into_tensor_data(self) -> TensorData {
        TensorData::UInt8(self)
    }
}

/// 从张量数据转换的trait
pub trait FromTensorData: Sized {
    fn dtype() -> TensorDType;
}

impl FromTensorData for f32 {
    fn dtype() -> TensorDType {
        TensorDType::Float32
    }
}

impl FromTensorData for i8 {
    fn dtype() -> TensorDType {
        TensorDType::Int8
    }
}

impl FromTensorData for u8 {
    fn dtype() -> TensorDType {
        TensorDType::UInt8
    }
}

// ============================================================================
// 张量规格
// ============================================================================

/// 张量规格
#[derive(Debug, Clone)]
pub struct TensorSpec {
    /// 名称
    pub name: String,
    /// 形状
    pub shape: Vec<usize>,
    /// 数据类型
    pub dtype: TensorDType,
}

// ============================================================================
// NPU设备信息
// ============================================================================

/// NPU设备信息
#[derive(Debug, Clone)]
pub struct NPUDeviceInfo {
    /// 设备名称
    pub device_name: String,
    /// 设备类型
    pub device_type: NPUDeviceType,
    /// 是否支持FP16
    pub supports_fp16: bool,
    /// 计算单元数量
    pub compute_units: Option<u32>,
    /// 内存大小（MB）
    pub memory_size_mb: Option<usize>,
}

// ============================================================================
// 错误类型
// ============================================================================

/// NPU错误
#[derive(Debug, thiserror::Error)]
pub enum NPUError {
    /// 模型加载失败
    #[error("Failed to load model: {0}")]
    ModelLoadFailed(String),

    /// 推理失败
    #[error("Inference failed: {0}")]
    InferenceFailed(String),

    /// 不支持的张量形状
    #[error("Unsupported tensor shape: {0:?}")]
    UnsupportedTensorShape(Vec<usize>),

    /// 不支持的数据类型
    #[error("Unsupported data type: {0:?}")]
    UnsupportedDataType(TensorDType),

    /// 设备不可用
    #[error("NPU device not available: {0}")]
    DeviceNotAvailable(String),

    /// IO错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// 其他错误
    #[error("NPU error: {0}")]
    Other(String),
}

// ============================================================================
// Trait定义
// ============================================================================

/// NPU运行时实现trait
#[async_trait::async_trait]
pub trait NPURuntimeImpl: Send + Sync {
    /// 加载模型
    async fn load_model(&self, model_path: &str) -> Result<NPUModel, NPUError>;

    /// 获取设备信息
    fn get_device_info(&self) -> NPUDeviceInfo;
}

/// NPU模型实现trait
#[async_trait::async_trait]
pub trait NPUModelImpl: Send + Sync {
    /// 执行推理
    async fn inference(&self, inputs: &[NPUTensor]) -> Result<Vec<NPUTensor>, NPUError>;
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_detection() {
        let device = NPUDeviceType::detect_best_device();
        println!("Detected NPU device: {}", device.name());

        #[cfg(target_os = "macos")]
        if matches!(device, NPUDeviceType::AppleNeuralEngine) {
            assert!(device.is_hardware_accelerated());
        }

        #[cfg(target_os = "android")]
        if matches!(device, NPUDeviceType::AndroidNNAPI) {
            assert!(device.is_hardware_accelerated());
        }
    }

    #[test]
    fn test_tensor_creation() {
        let tensor = NPUTensor::from_vec(vec![1.0, 2.0, 3.0, 4.0], &[2, 2]);
        assert_eq!(tensor.shape, vec![2, 2]);
        assert_eq!(tensor.dtype, TensorDType::Float32);
    }

    #[test]
    fn test_tensor_conversion() {
        let data = vec![1.0_f32, 2.0, 3.0];
        let tensor = NPUTensor::from_vec(data.clone(), &[3]);

        let result = tensor.to_vec::<f32>();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), data);
    }
}
