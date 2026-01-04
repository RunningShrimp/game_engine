//! # Fallback实现（CPU/GPU）
//!
//! 当NPU不可用时的fallback实现。

use crate::acceleration::npus::*;
use async_trait::async_trait;

// ============================================================================
// CPU Fallback
// ============================================================================

/// CPU运行时
pub struct CPURuntime {
    /// 线程数
    num_threads: usize,
}

impl CPURuntime {
    /// 创建新的CPU运行时
    pub fn new() -> Result<Self, NPUError> {
        let num_threads = num_cpus::get();

        tracing::info!("CPU runtime initialized with {} threads", num_threads);

        Ok(Self { num_threads })
    }
}

#[async_trait::async_trait]
impl NPURuntimeImpl for CPURuntime {
    async fn load_model(&self, model_path: &str) -> Result<NPUModel, NPUError> {
        tracing::info!("Loading CPU fallback model: {}", model_path);

        // 简化实现：加载模型文件
        let model_url = std::path::Path::new(model_path);
        if !model_url.exists() {
            return Err(NPUError::ModelLoadFailed(format!(
                "Model not found: {model_path}"
            )));
        }

        // 模拟模型规格
        let input_spec = vec![TensorSpec {
            name: "input".to_string(),
            shape: vec![1, 3, 224, 224],
            dtype: TensorDType::Float32,
        }];

        let output_spec = vec![TensorSpec {
            name: "output".to_string(),
            shape: vec![1, 1000],
            dtype: TensorDType::Float32,
        }];

        let model = NPUModel {
            name: model_url
                .file_name()
                .unwrap_or(std::ffi::OsStr::new("unknown"))
                .to_string_lossy()
                .to_string(),
            input_spec,
            output_spec,
            inner: Arc::new(CPUModel {
                num_threads: self.num_threads,
            }),
        };

        tracing::info!("CPU fallback model loaded: {}", model.name);

        Ok(model)
    }

    fn get_device_info(&self) -> NPUDeviceInfo {
        NPUDeviceInfo {
            device_name: "CPU Fallback".to_string(),
            device_type: NPUDeviceType::CPU,
            supports_fp16: false,
            compute_units: Some(self.num_threads as u32),
            memory_size_mb: None,
        }
    }
}

// ============================================================================

/// CPU模型
pub struct CPUModel {
    /// 线程数
    num_threads: usize,
}

#[async_trait::async_trait]
impl NPUModelImpl for CPUModel {
    async fn inference(&self, inputs: &[NPUTensor]) -> Result<Vec<NPUTensor>, NPUError> {
        tracing::debug!(
            "Running CPU fallback inference with {} threads",
            self.num_threads
        );

        // 简化实现：模拟CPU推理
        // 实际实现可以使用：
        // - tch-rs (LibTorch)
        // - ndarray
        // - cudarc (CUDA)

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // 模拟输出
        let output = NPUTensor {
            data: TensorData::Float32(vec![0.0; 1000]),
            shape: vec![1, 1000],
            dtype: TensorDType::Float32,
            name: Some("output".to_string()),
        };

        tracing::debug!("CPU fallback inference completed");

        Ok(vec![output])
    }
}

// ============================================================================
// 性能基准测试
// ============================================================================

/// 性能基准测试结果
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// 设备名称
    pub device_name: String,
    /// 推理时间（毫秒）
    pub inference_time_ms: f64,
    /// 内存使用（MB）
    pub memory_mb: f64,
    /// 加速比（相比CPU）
    pub speedup: f64,
}

impl CPURuntime {
    /// 运行性能基准测试
    pub async fn benchmark(&self) -> BenchmarkResults {
        tracing::info!("Running CPU fallback benchmark...");

        let start = std::time::Instant::now();

        // 模拟推理
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        BenchmarkResults {
            device_name: "CPU".to_string(),
            inference_time_ms: elapsed,
            memory_mb: 100.0,
            speedup: 1.0,
        }
    }
}

// ============================================================================
// 依赖项（用于编译）
// ============================================================================

/// 模拟num_cpus
#[doc(hidden)]
pub mod num_cpus {
    pub fn get() -> usize {
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1)
    }
}
