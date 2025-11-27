/// 批量数据处理SIMD优化模块
/// 
/// 用于渲染管线中的大批量数据处理

mod transform;
mod skinning;
mod particle;

pub use transform::*;
pub use skinning::*;
pub use particle::*;

use crate::performance::simd::SimdBackend;

/// 批量处理配置
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// 批量大小
    pub batch_size: usize,
    /// 使用的SIMD后端
    pub backend: SimdBackend,
    /// 是否启用多线程
    pub use_threading: bool,
    /// 线程数（0表示自动检测）
    pub num_threads: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 1024,
            backend: SimdBackend::best_available(),
            use_threading: true,
            num_threads: 0,
        }
    }
}

impl BatchConfig {
    /// 获取实际使用的线程数
    pub fn get_num_threads(&self) -> usize {
        if self.num_threads == 0 {
            num_cpus::get()
        } else {
            self.num_threads
        }
    }
}

/// 批量处理统计信息
#[derive(Debug, Default, Clone)]
pub struct BatchStats {
    /// 处理的元素数量
    pub elements_processed: usize,
    /// 处理时间（微秒）
    pub processing_time_us: u64,
    /// 使用的SIMD后端
    pub backend_used: Option<SimdBackend>,
}

impl BatchStats {
    /// 计算吞吐量（元素/秒）
    pub fn throughput(&self) -> f64 {
        if self.processing_time_us == 0 {
            return 0.0;
        }
        (self.elements_processed as f64) / (self.processing_time_us as f64 / 1_000_000.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_config() {
        let config = BatchConfig::default();
        assert!(config.batch_size > 0);
        assert!(config.get_num_threads() > 0);
    }
}
