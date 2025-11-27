/// 异步硬件检测
/// 
/// 在后台线程执行硬件检测，避免阻塞主线程

use super::{GpuInfo, NpuInfo, SocInfo};
use super::cache::{HardwareCache, detect_hardware_cached};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Instant;

/// 硬件检测状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionState {
    /// 未开始
    NotStarted,
    /// 检测中
    InProgress,
    /// 已完成
    Completed,
    /// 失败
    Failed(String),
}

/// 异步硬件检测器
pub struct AsyncHardwareDetector {
    state: Arc<Mutex<DetectionState>>,
    result: Arc<Mutex<Option<HardwareDetectionResult>>>,
    start_time: Option<Instant>,
}

/// 硬件检测结果
#[derive(Debug, Clone)]
pub struct HardwareDetectionResult {
    pub gpu: GpuInfo,
    pub npu: Option<NpuInfo>,
    pub soc: Option<SocInfo>,
    pub detection_time_ms: f64,
    pub from_cache: bool,
}

impl AsyncHardwareDetector {
    /// 创建新的异步检测器
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(DetectionState::NotStarted)),
            result: Arc::new(Mutex::new(None)),
            start_time: None,
        }
    }
    
    /// 开始异步检测
    pub fn start(&mut self) {
        let state = Arc::clone(&self.state);
        let result = Arc::clone(&self.result);
        
        // 更新状态
        *state.lock().unwrap() = DetectionState::InProgress;
        self.start_time = Some(Instant::now());
        
        // 在后台线程执行检测
        thread::spawn(move || {
            let start = Instant::now();
            
            // 尝试从缓存加载
            let (gpu, npu, soc, from_cache) = if let Some(cache) = HardwareCache::load() {
                if cache.is_valid() {
                    (cache.gpu, cache.npu, cache.soc, true)
                } else {
                    let (g, n, s) = detect_hardware_cached();
                    (g, n, s, false)
                }
            } else {
                let (g, n, s) = detect_hardware_cached();
                (g, n, s, false)
            };
            
            let detection_time_ms = start.elapsed().as_secs_f64() * 1000.0;
            
            // 保存结果
            let detection_result = HardwareDetectionResult {
                gpu,
                npu,
                soc,
                detection_time_ms,
                from_cache,
            };
            
            *result.lock().unwrap() = Some(detection_result);
            *state.lock().unwrap() = DetectionState::Completed;
        });
    }
    
    /// 获取当前状态
    pub fn state(&self) -> DetectionState {
        self.state.lock().unwrap().clone()
    }
    
    /// 检查是否完成
    pub fn is_completed(&self) -> bool {
        matches!(self.state(), DetectionState::Completed)
    }
    
    /// 检查是否失败
    pub fn is_failed(&self) -> bool {
        matches!(self.state(), DetectionState::Failed(_))
    }
    
    /// 获取结果（非阻塞）
    pub fn try_get_result(&self) -> Option<HardwareDetectionResult> {
        self.result.lock().unwrap().clone()
    }
    
    /// 等待结果（阻塞）
    pub fn wait_for_result(&self) -> HardwareDetectionResult {
        loop {
            if let Some(result) = self.try_get_result() {
                return result;
            }
            thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    
    /// 获取已经过的时间
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time
            .map(|t| t.elapsed().as_secs_f64() * 1000.0)
            .unwrap_or(0.0)
    }
    
    /// 获取进度百分比（估算）
    pub fn progress(&self) -> f32 {
        match self.state() {
            DetectionState::NotStarted => 0.0,
            DetectionState::InProgress => {
                // 基于经验值估算进度
                let elapsed = self.elapsed_ms();
                if elapsed < 50.0 {
                    elapsed as f32 / 50.0 * 0.5 // 0-50ms: 0-50%
                } else if elapsed < 150.0 {
                    0.5 + (elapsed as f32 - 50.0) / 100.0 * 0.3 // 50-150ms: 50-80%
                } else {
                    0.8 + (elapsed as f32 - 150.0) / 100.0 * 0.2 // 150ms+: 80-100%
                }.min(0.99)
            }
            DetectionState::Completed => 1.0,
            DetectionState::Failed(_) => 0.0,
        }
    }
}

impl Default for AsyncHardwareDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// 便捷函数：启动异步检测并返回检测器
pub fn start_async_detection() -> AsyncHardwareDetector {
    let mut detector = AsyncHardwareDetector::new();
    detector.start();
    detector
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_async_detection() {
        let mut detector = AsyncHardwareDetector::new();
        
        assert_eq!(detector.state(), DetectionState::NotStarted);
        
        detector.start();
        
        assert_eq!(detector.state(), DetectionState::InProgress);
        
        // 等待完成
        let result = detector.wait_for_result();
        
        assert_eq!(detector.state(), DetectionState::Completed);
        assert!(!result.gpu.name.is_empty());
        
        println!("检测结果: {:#?}", result);
        println!("检测耗时: {:.2}ms", result.detection_time_ms);
        println!("来自缓存: {}", result.from_cache);
    }
    
    #[test]
    fn test_progress() {
        let mut detector = AsyncHardwareDetector::new();
        
        assert_eq!(detector.progress(), 0.0);
        
        detector.start();
        
        // 检查进度
        thread::sleep(std::time::Duration::from_millis(20));
        let progress1 = detector.progress();
        println!("进度1: {:.1}%", progress1 * 100.0);
        
        thread::sleep(std::time::Duration::from_millis(50));
        let progress2 = detector.progress();
        println!("进度2: {:.1}%", progress2 * 100.0);
        
        // 等待完成
        detector.wait_for_result();
        
        assert_eq!(detector.progress(), 1.0);
    }
    
    #[test]
    fn test_convenience_function() {
        let detector = start_async_detection();
        
        // 在检测的同时可以做其他事情
        println!("正在后台检测硬件...");
        
        // 等待结果
        let result = detector.wait_for_result();
        
        println!("检测完成！");
        println!("GPU: {}", result.gpu.name);
        println!("耗时: {:.2}ms", result.detection_time_ms);
    }
}
