//  性能指标模块
// 
//  定义全面的性能指标体系，包括渲染、内存、物理、音频和系统指标。

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};

// ============================================================================
// 指标分类
// ============================================================================

//  指标类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricCategory {
    /// 渲染指标
    Render,
    /// 内存指标
    Memory,
    /// 物理指标
    Physics,
    /// 音频指标
    Audio,
    /// 系统指标
    System,
    /// 网络指标
    Network,
    /// 输入指标
    Input,
    /// 文件IO指标
    IO,
}

impl MetricCategory {
    /// 获取类别名称
    pub fn name(&self) -> &'static str {
        match self {
            MetricCategory::Render => "渲染",
            MetricCategory::Memory => "内存",
            MetricCategory::Physics => "物理",
            MetricCategory::Audio => "音频",
            MetricCategory::System => "系统",
            MetricCategory::Network => "网络",
            MetricCategory::Input => "输入",
            MetricCategory::IO => "文件IO",
        }
    }

    /// 获取类别描述
    pub fn description(&self) -> &'static str {
        match self {
            MetricCategory::Render => "渲染相关的性能指标",
            MetricCategory::Memory => "内存分配和使用相关指标",
            MetricCategory::Physics => "物理计算和碰撞检测指标",
            MetricCategory::Audio => "音频处理和延迟指标",
            MetricCategory::System => "系统资源和调度指标",
            MetricCategory::Network => "网络传输和延迟指标",
            MetricCategory::Input => "输入设备响应指标",
            MetricCategory::IO => "文件读写性能指标",
        }
    }
}

//  指标单位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricUnit {
    /// 无单位
    None,
    /// 毫秒
    Milliseconds,
    /// 微秒
    Microseconds,
    /// 纳秒
    Nanoseconds,
    /// 帧每秒
    FPS,
    /// 百分比
    Percentage,
    /// 字节
    Bytes,
    /// 兆字节
    Megabytes,
    /// 吉字节
    Gigabytes,
    /// 计数
    Count,
    /// 每秒次数
    PerSecond,
    /// 赫兹
    Hertz,
}

impl MetricUnit {
    /// 获取单位符号
    pub fn symbol(&self) -> &'static str {
        match self {
            MetricUnit::None => "",
            MetricUnit::Milliseconds => "ms",
            MetricUnit::Microseconds => "μs",
            MetricUnit::Nanoseconds => "ns",
            MetricUnit::FPS => "fps",
            MetricUnit::Percentage => "%",
            MetricUnit::Bytes => "B",
            MetricUnit::Megabytes => "MB",
            MetricUnit::Gigabytes => "GB",
            MetricUnit::Count => "",
            MetricUnit::PerSecond => "/s",
            MetricUnit::Hertz => "Hz",
        }
    }
}

// ============================================================================
// 性能计数器
// ============================================================================

//  性能计数器
//  
//  使用原子操作确保线程安全，支持高频率更新
#[derive(Debug)]
pub struct PerformanceCounter {
    /// 计数器名称
    pub name: String,
    /// 当前值
    value: AtomicU64,
    /// 峰值
    peak: AtomicU64,
    /// 指标类别
    pub category: MetricCategory,
    /// 指标单位
    pub unit: MetricUnit,
    /// 创建时间
    created_at: Instant,
    /// 最后更新时间
    last_updated: AtomicU64, // 存储时间戳
}

impl PerformanceCounter {
    /// 创建新的性能计数器
    pub fn new(
        name: impl Into<String>,
        category: MetricCategory,
        unit: MetricUnit,
    ) -> Self {
        let now = Instant::now();
        Self {
            name: name.into(),
            value: AtomicU64::new(0),
            peak: AtomicU64::new(0),
            category,
            unit,
            created_at: now,
            last_updated: AtomicU64::new(now.elapsed().as_nanos() as u64),
        }
    }

    /// 增加计数器值
    pub fn increment(&self, delta: u64) {
        let old_value = self.value.fetch_add(delta, Ordering::Relaxed);
        let new_value = old_value + delta;
        
        // 更新峰值
        let mut current_peak = self.peak.load(Ordering::Relaxed);
        while new_value > current_peak {
            match self.peak.compare_exchange_weak(
                current_peak,
                new_value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }

        // 更新最后更新时间
        self.last_updated.store(
            Instant::now().elapsed().as_nanos() as u64,
            Ordering::Relaxed,
        );
    }

    /// 设置计数器值
    pub fn set(&self, value: u64) {
        self.value.store(value, Ordering::Relaxed);
        
        // 更新峰值
        let mut current_peak = self.peak.load(Ordering::Relaxed);
        while value > current_peak {
            match self.peak.compare_exchange_weak(
                current_peak,
                value,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current_peak = actual,
            }
        }

        // 更新最后更新时间
        self.last_updated.store(
            Instant::now().elapsed().as_nanos() as u64,
            Ordering::Relaxed,
        );
    }

    /// 获取当前值
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// 获取峰值
    pub fn peak(&self) -> u64 {
        self.peak.load(Ordering::Relaxed)
    }

    /// 重置计数器
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
        self.peak.store(0, Ordering::Relaxed);
        self.last_updated.store(
            Instant::now().elapsed().as_nanos() as u64,
            Ordering::Relaxed,
        );
    }

    /// 获取最后更新时间
    pub fn last_updated(&self) -> Instant {
        let nanos = self.last_updated.load(Ordering::Relaxed);
        Instant::now() - Duration::from_nanos(nanos)
    }
}

// ============================================================================
// 预定义指标
// ============================================================================

//  渲染指标
pub mod render {
    use super::*;

    pub const FRAME_TIME: &str = "render.frame_time";
    pub const FPS: &str = "render.fps";
    pub const GPU_UTILIZATION: &str = "render.gpu_utilization";
    pub const RENDER_LATENCY: &str = "render.render_latency";
    pub const DRAW_CALLS: &str = "render.draw_calls";
    pub const TRIANGLE_COUNT: &str = "render.triangle_count";
    pub const VERTEX_COUNT: &str = "render.vertex_count";
    pub const SHADER_COMPILATION_TIME: &str = "render.shader_compilation_time";
    pub const TEXTURE_UPLOAD_TIME: &str = "render.texture_upload_time";
    pub const BUFFER_UPLOAD_TIME: &str = "render.buffer_upload_time";
}

//  内存指标
pub mod memory {
    use super::*;

    pub const ALLOCATION_COUNT: &str = "memory.allocation_count";
    pub const DEALLOCATION_COUNT: &str = "memory.deallocation_count";
    pub const USAGE_BYTES: &str = "memory.usage_bytes";
    pub const USAGE_MB: &str = "memory.usage_mb";
    pub const PEAK_USAGE: &str = "memory.peak_usage";
    pub const FRAGMENTATION_RATIO: &str = "memory.fragmentation_ratio";
    pub const ALLOCATION_RATE: &str = "memory.allocation_rate";
    pub const DEALLOCATION_RATE: &str = "memory.deallocation_rate";
    pub const LEAK_COUNT: &str = "memory.leak_count";
}

//  物理指标
pub mod physics {
    use super::*;

    pub const STEP_TIME: &str = "physics.step_time";
    pub const COLLISION_DETECTION_TIME: &str = "physics.collision_detection_time";
    pub const CONSTRAINT_SOLVING_TIME: &str = "physics.constraint_solving_time";
    pub const SYNC_OVERHEAD: &str = "physics.sync_overhead";
    pub const ACTIVE_BODIES: &str = "physics.active_bodies";
    pub const SLEEPING_BODIES: &str = "physics.sleeping_bodies";
    pub const COLLISION_CHECKS: &str = "physics.collision_checks";
    pub const CONTACT_POINTS: &str = "physics.contact_points";
}

//  音频指标
pub mod audio {
    use super::*;

    pub const LATENCY: &str = "audio.latency";
    pub const BUFFER_USAGE: &str = "audio.buffer_usage";
    pub const PROCESSING_TIME: &str = "audio.processing_time";
    pub const DECODING_TIME: &str = "audio.decoding_time";
    pub const MIXING_TIME: &str = "audio.mixing_time";
    pub const SAMPLE_RATE: &str = "audio.sample_rate";
    pub const CHANNEL_COUNT: &str = "audio.channel_count";
    pub const DROPPED_FRAMES: &str = "audio.dropped_frames";
}

//  系统指标
pub mod system {
    use super::*;

    pub const CPU_USAGE: &str = "system.cpu_usage";
    pub const MEMORY_USAGE: &str = "system.memory_usage";
    pub const TASK_SCHEDULER_LATENCY: &str = "system.task_scheduler_latency";
    pub const ERROR_RATE: &str = "system.error_rate";
    pub const THREAD_COUNT: &str = "system.thread_count";
    pub const CONTEXT_SWITCHES: &str = "system.context_switches";
    pub const DISK_IO_RATE: &str = "system.disk_io_rate";
    pub const NETWORK_BANDWIDTH: &str = "system.network_bandwidth";
}

// ============================================================================
// 指标注册表
// ============================================================================

//  指标定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    /// 指标名称
    pub name: String,
    /// 指标类别
    pub category: MetricCategory,
    /// 指标单位
    pub unit: MetricUnit,
    /// 指标描述
    pub description: String,
    /// 是否为关键指标
    pub critical: bool,
    /// 默认阈值
    pub default_threshold: Option<f32>,
}

//  指标注册表
#[derive(Debug, Default)]
pub struct MetricRegistry {
    /// 指标定义映射
    definitions: HashMap<String, MetricDefinition>,
    /// 性能计数器映射
    counters: HashMap<String, PerformanceCounter>,
}

impl MetricRegistry {
    /// 创建新的指标注册表
    pub fn new() -> Self {
        let mut registry = Self {
            definitions: HashMap::new(),
            counters: HashMap::new(),
        };
        
        // 注册默认指标
        registry.register_default_metrics();
        registry
    }

    /// 注册指标定义
    pub fn register_metric(&mut self, definition: MetricDefinition) {
        self.definitions.insert(definition.name.clone(), definition);
    }

    /// 获取指标定义
    pub fn get_definition(&self, name: &str) -> Option<&MetricDefinition> {
        self.definitions.get(name)
    }

    /// 获取所有指标定义
    pub fn get_all_definitions(&self) -> &HashMap<String, MetricDefinition> {
        &self.definitions
    }

    /// 创建或获取性能计数器
    pub fn get_counter(&mut self, name: &str) -> Option<&PerformanceCounter> {
        if !self.counters.contains_key(name) {
            if let Some(def) = self.definitions.get(name) {
                let counter = PerformanceCounter::new(
                    name,
                    def.category,
                    def.unit,
                );
                self.counters.insert(name.to_string(), counter);
            }
        }
        self.counters.get(name)
    }

    /// 获取所有计数器
    pub fn get_all_counters(&self) -> &HashMap<String, PerformanceCounter> {
        &self.counters
    }

    /// 重置所有计数器
    pub fn reset_all(&self) {
        for counter in self.counters.values() {
            counter.reset();
        }
    }

    /// 注册默认指标
    fn register_default_metrics(&mut self) {
        // 渲染指标
        self.register_metric(MetricDefinition {
            name: render::FRAME_TIME.to_string(),
            category: MetricCategory::Render,
            unit: MetricUnit::Milliseconds,
            description: "帧渲染时间".to_string(),
            critical: true,
            default_threshold: Some(16.67), // 60 FPS
        });

        self.register_metric(MetricDefinition {
            name: render::FPS.to_string(),
            category: MetricCategory::Render,
            unit: MetricUnit::FPS,
            description: "帧率".to_string(),
            critical: true,
            default_threshold: Some(30.0),
        });

        self.register_metric(MetricDefinition {
            name: render::DRAW_CALLS.to_string(),
            category: MetricCategory::Render,
            unit: MetricUnit::Count,
            description: "绘制调用次数".to_string(),
            critical: false,
            default_threshold: Some(1000.0),
        });

        // 内存指标
        self.register_metric(MetricDefinition {
            name: memory::USAGE_MB.to_string(),
            category: MetricCategory::Memory,
            unit: MetricUnit::Megabytes,
            description: "内存使用量".to_string(),
            critical: true,
            default_threshold: Some(1024.0), // 1GB
        });

        self.register_metric(MetricDefinition {
            name: memory::ALLOCATION_COUNT.to_string(),
            category: MetricCategory::Memory,
            unit: MetricUnit::PerSecond,
            description: "内存分配频率".to_string(),
            critical: false,
            default_threshold: Some(1000.0),
        });

        // 物理指标
        self.register_metric(MetricDefinition {
            name: physics::STEP_TIME.to_string(),
            category: MetricCategory::Physics,
            unit: MetricUnit::Milliseconds,
            description: "物理步进时间".to_string(),
            critical: true,
            default_threshold: Some(5.0),
        });

        // 系统指标
        self.register_metric(MetricDefinition {
            name: system::CPU_USAGE.to_string(),
            category: MetricCategory::System,
            unit: MetricUnit::Percentage,
            description: "CPU使用率".to_string(),
            critical: true,
            default_threshold: Some(80.0),
        });
    }
}

// ============================================================================
// 宏定义
// ============================================================================

//  创建性能计数器的宏
#[macro_export]
macro_rules! define_counter {
    ($registry:expr, $name:expr, $category:expr, $unit:expr) => {
        $registry.get_counter($name).unwrap()
    };
}

//  记录性能指标的宏
#[macro_export]
macro_rules! record_metric {
    ($registry:expr, $name:expr, $value:expr) => {
        if let Some(counter) = $registry.get_counter($name) {
            counter.set($value);
        }
    };
}

//  增加性能指标的宏
#[macro_export]
macro_rules! increment_metric {
    ($registry:expr, $name:expr) => {
        if let Some(counter) = $registry.get_counter($name) {
            counter.increment(1);
        }
    };
    ($registry:expr, $name:expr, $delta:expr) => {
        if let Some(counter) = $registry.get_counter($name) {
            counter.increment($delta);
        }
    };
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_counter() {
        let counter = PerformanceCounter::new("test", MetricCategory::Render, MetricUnit::Count);
        
        assert_eq!(counter.value(), 0);
        assert_eq!(counter.peak(), 0);
        
        counter.increment(5);
        assert_eq!(counter.value(), 5);
        assert_eq!(counter.peak(), 5);
        
        counter.set(3);
        assert_eq!(counter.value(), 3);
        assert_eq!(counter.peak(), 5); // 峰值保持不变
        
        counter.reset();
        assert_eq!(counter.value(), 0);
        assert_eq!(counter.peak(), 0);
    }

    #[test]
    fn test_metric_registry() {
        let mut registry = MetricRegistry::new();
        
        // 测试获取预定义指标
        let frame_time_counter = registry.get_counter(render::FRAME_TIME);
        assert!(frame_time_counter.is_some());
        
        // 测试注册新指标
        registry.register_metric(MetricDefinition {
            name: "custom.test_metric".to_string(),
            category: MetricCategory::System,
            unit: MetricUnit::Milliseconds,
            description: "测试指标".to_string(),
            critical: false,
            default_threshold: Some(10.0),
        });
        
        let custom_counter = registry.get_counter("custom.test_metric");
        assert!(custom_counter.is_some());
    }

    #[test]
    fn test_metric_categories() {
        assert_eq!(MetricCategory::Render.name(), "渲染");
        assert_eq!(MetricCategory::Memory.name(), "内存");
        assert_eq!(MetricCategory::Physics.name(), "物理");
        
        assert!(!MetricCategory::Render.description().is_empty());
    }

    #[test]
    fn test_metric_units() {
        assert_eq!(MetricUnit::Milliseconds.symbol(), "ms");
        assert_eq!(MetricUnit::FPS.symbol(), "fps");
        assert_eq!(MetricUnit::Percentage.symbol(), "%");
        assert_eq!(MetricUnit::Megabytes.symbol(), "MB");
    }

    #[test]
    fn test_concurrent_counter() {
        use std::sync::Arc;
        use std::thread;
        
        let counter = Arc::new(PerformanceCounter::new("concurrent", MetricCategory::System, MetricUnit::Count));
        let mut handles = Vec::new();
        
        // 创建多个线程同时增加计数器
        for _ in 0..10 {
            let counter_clone = Arc::clone(&counter);
            let handle = thread::spawn(move || {
                for _ in 0..100 {
                    counter_clone.increment(1);
                }
            });
            handles.push(handle);
        }
        
        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(counter.value(), 1000);
    }
}