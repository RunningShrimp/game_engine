//  性能数据收集器
// 
//  提供高精度时间测量、低开销计数器和批量数据聚合功能。

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::metrics::*;
use super::ProfilingResult;

// ============================================================================
// 高精度计时器
// ============================================================================

/// 高精度计时器
/// 
/// 使用RAII模式自动记录作用域时间
#[derive(Debug)]
pub struct HighPrecisionTimer {
    /// 计时器名称
    name: String,
    /// 开始时间
    start_time: Instant,
    /// 数据收集器引用
    collector: Option<Arc<Mutex<MetricCollector>>>,
}

impl HighPrecisionTimer {
    /// 创建新的高精度计时器
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start_time: Instant::now(),
            collector: None,
        }
    }

    /// 创建绑定到收集器的计时器
    pub fn with_collector(
        name: impl Into<String>,
        collector: Arc<Mutex<MetricCollector>>,
    ) -> Self {
        Self {
            name: name.into(),
            start_time: Instant::now(),
            collector: Some(collector),
        }
    }

    /// 获取经过的时间
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// 获取经过的微秒数
    pub fn elapsed_micros(&self) -> u64 {
        self.elapsed().as_micros() as u64
    }

    /// 获取经过的纳秒数
    pub fn elapsed_nanos(&self) -> u64 {
        self.elapsed().as_nanos() as u64
    }

    /// 结束计时并记录到收集器
    pub fn finish(self) -> Duration {
        let elapsed = self.elapsed();
        
        if let Some(collector) = self.collector {
            if let Ok(mut collector) = collector.lock() {
                collector.record_timing(&self.name, elapsed);
            }
        }
        
        elapsed
    }
}

impl Drop for HighPrecisionTimer {
    fn drop(&mut self) {
        if let Some(collector) = &self.collector {
            if let Ok(mut collector) = collector.lock() {
                collector.record_timing(&self.name, self.start_time.elapsed());
            }
        }
    }
}

/// 作用域计时器宏
/// 
/// 自动创建和销毁计时器
#[macro_export]
macro_rules! timed_scope {
    ($collector:expr, $name:expr) => {
        let _timer = $crate::performance::HighPrecisionTimer::new($name);
    };
}

// ============================================================================
// 滑动窗口聚合器
// ============================================================================

/// 滑动窗口聚合器
/// 
/// 用于实时统计和趋势分析
#[derive(Debug, Clone)]
pub struct SlidingWindowAggregator {
    /// 数据点
    samples: Vec<f64>,
    /// 窗口大小
    window_size: usize,
    /// 当前索引
    current_index: usize,
    /// 是否已填满
    is_full: bool,
}

impl SlidingWindowAggregator {
    /// 创建新的滑动窗口聚合器
    pub fn new(window_size: usize) -> Self {
        Self {
            samples: vec![0.0; window_size],
            window_size,
            current_index: 0,
            is_full: false,
        }
    }

    /// 添加新数据点
    pub fn add_sample(&mut self, value: f64) {
        self.samples[self.current_index] = value;
        self.current_index = (self.current_index + 1) % self.window_size;
        
        if self.current_index == 0 {
            self.is_full = true;
        }
    }

    /// 获取平均值
    pub fn average(&self) -> f64 {
        let count = if self.is_full { self.window_size } else { self.current_index };
        if count == 0 {
            return 0.0;
        }
        
        let sum: f64 = self.samples.iter().take(count).sum();
        sum / count as f64
    }

    /// 获取最小值
    pub fn minimum(&self) -> f64 {
        let count = if self.is_full { self.window_size } else { self.current_index };
        if count == 0 {
            return f64::INFINITY;
        }
        
        self.samples.iter().take(count).fold(f64::INFINITY, |a, &b| a.min(b))
    }

    /// 获取最大值
    pub fn maximum(&self) -> f64 {
        let count = if self.is_full { self.window_size } else { self.current_index };
        if count == 0 {
            return f64::NEG_INFINITY;
        }
        
        self.samples.iter().take(count).fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    }

    /// 获取标准差
    pub fn standard_deviation(&self) -> f64 {
        let count = if self.is_full { self.window_size } else { self.current_index };
        if count < 2 {
            return 0.0;
        }
        
        let avg = self.average();
        let variance: f64 = self.samples
            .iter()
            .take(count)
            .map(|x| (x - avg).powi(2))
            .sum::<f64>() / (count - 1) as f64;
        variance.sqrt()
    }

    /// 获取百分位数
    pub fn percentile(&self, p: f64) -> f64 {
        let count = if self.is_full { self.window_size } else { self.current_index };
        if count == 0 {
            return 0.0;
        }
        
        let mut sorted: Vec<f64> = self.samples.iter().take(count).cloned().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = ((p / 100.0) * (count - 1) as f64) as usize;
        sorted[index.min(count - 1)]
    }

    /// 获取样本数量
    pub fn sample_count(&self) -> usize {
        if self.is_full { self.window_size } else { self.current_index }
    }

    /// 重置聚合器
    pub fn reset(&mut self) {
        self.samples.fill(0.0);
        self.current_index = 0;
        self.is_full = false;
    }
}

// ============================================================================
// 异步数据传输器
// ============================================================================

/// 异步数据传输配置
#[derive(Debug, Clone)]
pub struct AsyncTransferConfig {
    /// 批处理大小
    pub batch_size: usize,
    /// 刷新间隔
    pub flush_interval: Duration,
    /// 最大队列大小
    pub max_queue_size: usize,
    /// 是否启用压缩
    pub enable_compression: bool,
}

impl Default for AsyncTransferConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            flush_interval: Duration::from_millis(100),
            max_queue_size: 10000,
            enable_compression: true,
        }
    }
}

/// 待传输的数据项
#[derive(Debug, Clone)]
pub struct DataItem {
    /// 指标名称
    pub metric_name: String,
    /// 指标值
    pub value: f64,
    /// 时间戳
    pub timestamp: Instant,
    /// 标签
    pub tags: HashMap<String, String>,
}

/// 异步数据传输器
/// 
/// 在后台线程中批量处理数据传输，减少主线程开销
#[derive(Debug)]
pub struct AsyncDataTransmitter {
    /// 配置
    config: AsyncTransferConfig,
    /// 数据队列
    queue: Arc<Mutex<Vec<DataItem>>>,
    /// 运行标志
    running: Arc<AtomicBool>,
    /// 处理线程句柄
    handle: Option<thread::JoinHandle<()>>,
    /// 已传输计数
    transmitted_count: Arc<AtomicU64>,
}

impl AsyncDataTransmitter {
    /// 创建新的异步数据传输器
    pub fn new(config: AsyncTransferConfig) -> Self {
        Self {
            config,
            queue: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            transmitted_count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// 启动异步传输
    pub fn start(&mut self) -> ProfilingResult<()> {
        if self.running.load(Ordering::Relaxed) {
            return Err(super::ProfilingError::ConfigurationError(
                "异步传输器已在运行".to_string(),
            ));
        }

        self.running.store(true, Ordering::Relaxed);
        let queue = Arc::clone(&self.queue);
        let running = Arc::clone(&self.running);
        let transmitted_count = Arc::clone(&self.transmitted_count);
        let config = self.config.clone();

        let handle = thread::spawn(move || {
            let mut last_flush = Instant::now();
            
            while running.load(Ordering::Relaxed) {
                // 检查是否需要刷新
                let should_flush = last_flush.elapsed() >= config.flush_interval;
                
                // 获取待处理数据
                let mut items = Vec::new();
                {
                    if let Ok(mut queue) = queue.lock() {
                        // 检查队列大小限制
                        if queue.len() >= config.max_queue_size {
                            // 移除最旧的数据
                            let excess = queue.len() - config.max_queue_size + config.batch_size;
                            queue.drain(0..excess);
                        }
                        
                        // 批量提取数据
                        let take_count = if should_flush {
                            queue.len()
                        } else {
                            queue.len().min(config.batch_size)
                        };
                        
                        items = queue.drain(0..take_count).collect();
                    }
                }

                // 处理数据
                if !items.is_empty() {
                    Self::process_batch(&items, &config);
                    transmitted_count.fetch_add(items.len() as u64, Ordering::Relaxed);
                    last_flush = Instant::now();
                }

                // 短暂休眠避免CPU占用过高
                thread::sleep(Duration::from_millis(1));
            }
        });

        self.handle = Some(handle);
        Ok(())
    }

    /// 停止异步传输
    pub fn stop(&mut self) -> ProfilingResult<()> {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            handle.join().map_err(|_| {
                super::ProfilingError::ProcessingError(
                    "无法加入异步传输线程".to_string(),
                )
            })?;
        }
        
        Ok(())
    }

    /// 添加数据项到传输队列
    pub fn send(&self, item: DataItem) -> ProfilingResult<()> {
        if let Ok(mut queue) = self.queue.lock() {
            queue.push(item);
            Ok(())
        } else {
            Err(super::ProfilingError::CollectionError(
                "无法访问传输队列".to_string(),
            ))
        }
    }

    /// 获取已传输的数据计数
    pub fn transmitted_count(&self) -> u64 {
        self.transmitted_count.load(Ordering::Relaxed)
    }

    /// 获取队列大小
    pub fn queue_size(&self) -> usize {
        self.queue.lock().map(|q| q.len()).unwrap_or(0)
    }

    /// 处理数据批次
    fn process_batch(items: &[DataItem], config: &AsyncTransferConfig) {
        // 这里可以实现实际的传输逻辑
        // 例如发送到监控系统、写入文件等
        
        tracing::debug!(
            target: "profiling",
            "处理数据批次: {} 项, 压缩: {}",
            items.len(),
            config.enable_compression
        );
        
        // 模拟处理时间
        thread::sleep(Duration::from_micros(100));
    }
}

impl Drop for AsyncDataTransmitter {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

// ============================================================================
// 指标收集器
// ============================================================================

/// 指标收集器配置
#[derive(Debug, Clone)]
pub struct CollectorConfig {
    /// 采样频率 (Hz)
    pub sample_rate: f32,
    /// 环形缓冲区大小
    pub ring_buffer_size: usize,
    /// 是否启用异步传输
    pub enable_async_transfer: bool,
    /// 异步传输配置
    pub async_transfer_config: Option<AsyncTransferConfig>,
    /// 是否启用高精度计时
    pub enable_high_precision: bool,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            sample_rate: super::defaults::DEFAULT_SAMPLE_RATE,
            ring_buffer_size: super::defaults::DEFAULT_RING_BUFFER_SIZE,
            enable_async_transfer: true,
            async_transfer_config: Some(AsyncTransferConfig::default()),
            enable_high_precision: true,
        }
    }
}

/// 指标收集器
/// 
/// 统一收集和管理所有性能指标
#[derive(Debug)]
pub struct MetricCollector {
    /// 配置
    config: CollectorConfig,
    /// 指标注册表
    registry: Arc<Mutex<MetricRegistry>>,
    /// 滑动窗口聚合器
    aggregators: HashMap<String, SlidingWindowAggregator>,
    /// 异步传输器
    async_transmitter: Option<AsyncDataTransmitter>,
    /// 收集开始时间
    start_time: Instant,
    /// 总样本数
    total_samples: AtomicU64,
}

impl MetricCollector {
    /// 创建新的指标收集器
    pub fn new(config: CollectorConfig) -> ProfilingResult<Self> {
        let registry = Arc::new(Mutex::new(MetricRegistry::new()));
        let mut collector = Self {
            config,
            registry,
            aggregators: HashMap::new(),
            async_transmitter: None,
            start_time: Instant::now(),
            total_samples: AtomicU64::new(0),
        };

        // 初始化异步传输器
        if collector.config.enable_async_transfer {
            if let Some(async_config) = &collector.config.async_transfer_config {
                let mut transmitter = AsyncDataTransmitter::new(async_config.clone());
                transmitter.start()?;
                collector.async_transmitter = Some(transmitter);
            }
        }

        Ok(collector)
    }

    /// 记录计时数据
    pub fn record_timing(&mut self, name: &str, duration: Duration) {
        let duration_ms = duration.as_secs_f64() * 1000.0;
        self.record_value(name, duration_ms);
    }

    /// 记录数值数据
    pub fn record_value(&mut self, name: &str, value: f64) {
        // 更新计数器
        if let Ok(mut registry) = self.registry.lock() {
            if let Some(counter) = registry.get_counter(name) {
                counter.set(value as u64);
            }
        }

        // 更新滑动窗口
        let aggregator = self.aggregators.entry(name.to_string())
            .or_insert_with(|| SlidingWindowAggregator::new(60)); // 60个样本的窗口
        aggregator.add_sample(value);

        // 异步传输
        if let Some(transmitter) = &self.async_transmitter {
            let item = DataItem {
                metric_name: name.to_string(),
                value,
                timestamp: Instant::now(),
                tags: HashMap::new(),
            };
            let _ = transmitter.send(item);
        }

        // 更新样本计数
        self.total_samples.fetch_add(1, Ordering::Relaxed);
    }

    /// 获取指标的滑动窗口统计
    pub fn get_window_stats(&self, name: &str) -> Option<WindowStats> {
        self.aggregators.get(name).map(|aggregator| WindowStats {
            average: aggregator.average(),
            minimum: aggregator.minimum(),
            maximum: aggregator.maximum(),
            standard_deviation: aggregator.standard_deviation(),
            percentile_95: aggregator.percentile(95.0),
            percentile_99: aggregator.percentile(99.0),
            sample_count: aggregator.sample_count(),
        })
    }

    /// 获取所有指标的当前值
    pub fn get_current_values(&self) -> HashMap<String, u64> {
        let mut values = HashMap::new();
        
        if let Ok(registry) = self.registry.lock() {
            for (name, counter) in registry.get_all_counters() {
                values.insert(name.clone(), counter.value());
            }
        }
        
        values
    }

    /// 获取收集器统计信息
    pub fn get_collector_stats(&self) -> CollectorStats {
        CollectorStats {
            uptime: self.start_time.elapsed(),
            total_samples: self.total_samples.load(Ordering::Relaxed),
            metrics_count: self.registry.lock()
                .map(|r| r.get_all_definitions().len())
                .unwrap_or(0),
            queue_size: self.async_transmitter
                .as_ref()
                .map(|t| t.queue_size())
                .unwrap_or(0),
            transmitted_count: self.async_transmitter
                .as_ref()
                .map(|t| t.transmitted_count())
                .unwrap_or(0),
        }
    }

    /// 重置所有数据
    pub fn reset(&mut self) {
        if let Ok(mut registry) = self.registry.lock() {
            registry.reset_all();
        }
        
        for aggregator in self.aggregators.values_mut() {
            aggregator.reset();
        }
        
        self.start_time = Instant::now();
        self.total_samples.store(0, Ordering::Relaxed);
    }

    /// 创建高精度计时器
    pub fn create_timer(&self, name: &str) -> HighPrecisionTimer {
        // 简化：直接创建不绑定收集器的计时器，避免跨类型引用问题
        HighPrecisionTimer::new(name)
    }
}

/// 滑动窗口统计信息
#[derive(Debug, Clone)]
pub struct WindowStats {
    /// 平均值
    pub average: f64,
    /// 最小值
    pub minimum: f64,
    /// 最大值
    pub maximum: f64,
    /// 标准差
    pub standard_deviation: f64,
    /// 95分位数
    pub percentile_95: f64,
    /// 99分位数
    pub percentile_99: f64,
    /// 样本数量
    pub sample_count: usize,
}

/// 收集器统计信息
#[derive(Debug, Clone)]
pub struct CollectorStats {
    /// 运行时间
    pub uptime: Duration,
    /// 总样本数
    pub total_samples: u64,
    /// 指标数量
    pub metrics_count: usize,
    /// 队列大小
    pub queue_size: usize,
    /// 已传输数量
    pub transmitted_count: u64,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_high_precision_timer() {
        let timer = HighPrecisionTimer::new("test");
        thread::sleep(Duration::from_millis(10));
        let elapsed = timer.finish();
        
        assert!(elapsed.as_millis() >= 10);
        assert!(elapsed.as_millis() < 20);
    }

    #[test]
    fn test_sliding_window_aggregator() {
        let mut aggregator = SlidingWindowAggregator::new(5);
        
        aggregator.add_sample(10.0);
        aggregator.add_sample(20.0);
        aggregator.add_sample(30.0);
        
        assert_eq!(aggregator.average(), 20.0);
        assert_eq!(aggregator.minimum(), 10.0);
        assert_eq!(aggregator.maximum(), 30.0);
        assert_eq!(aggregator.sample_count(), 3);
    }

    #[test]
    fn test_async_data_transmitter() {
        let config = AsyncTransferConfig {
            batch_size: 2,
            flush_interval: Duration::from_millis(50),
            max_queue_size: 10,
            enable_compression: false,
        };
        
        let mut transmitter = AsyncDataTransmitter::new(config);
        transmitter.start().unwrap();
        
        // 发送测试数据
        for i in 0..5 {
            let item = DataItem {
                metric_name: format!("test_metric_{}", i),
                value: i as f64,
                timestamp: Instant::now(),
                tags: HashMap::new(),
            };
            transmitter.send(item).unwrap();
        }
        
        // 等待处理
        thread::sleep(Duration::from_millis(100));
        
        assert!(transmitter.transmitted_count() > 0);
        transmitter.stop().unwrap();
    }

    #[test]
    fn test_metric_collector() {
        let config = CollectorConfig {
            enable_async_transfer: false, // 测试时禁用异步传输
            ..Default::default()
        };
        
        let mut collector = MetricCollector::new(config).unwrap();
        
        // 记录一些测试数据
        collector.record_value("test_metric", 42.0);
        collector.record_timing("test_timing", Duration::from_millis(16));
        
        // 检查统计信息
        let stats = collector.get_window_stats("test_metric");
        assert!(stats.is_some());
        assert_eq!(stats.unwrap().average, 42.0);
        
        let current_values = collector.get_current_values();
        assert!(current_values.contains_key("test_metric"));
    }

    #[test]
    fn test_concurrent_recording() {
        let config = CollectorConfig {
            enable_async_transfer: false,
            ..Default::default()
        };
        
        let mut collector = MetricCollector::new(config).unwrap();
        let collector = Arc::new(Mutex::new(collector));
        
        let mut handles = Vec::new();
        
        // 创建多个线程同时记录数据
        for i in 0..10 {
            let collector_clone = Arc::clone(&collector);
            let handle = thread::spawn(move || {
                if let Ok(mut collector) = collector_clone.lock() {
                    for j in 0..100 {
                        collector.record_value(&format!("metric_{}", i), j as f64);
                    }
                }
            });
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        // 验证数据已记录
        if let Ok(collector) = collector.lock() {
            let stats = collector.get_collector_stats();
            assert_eq!(stats.total_samples, 1000); // 10 threads * 100 samples
        }
    }
}