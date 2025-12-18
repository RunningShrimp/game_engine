use std::collections::HashMap;
use std::time::Duration;

/// 瓶颈严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BottleneckSeverity {
    Low = 0,      // <20% variance
    Medium = 1,   // 20-50% variance
    High = 2,     // 50-100% variance
    Critical = 3, // >100% variance
}

impl BottleneckSeverity {
    fn from_variance(variance: f64) -> Self {
        match variance {
            v if v < 0.20 => BottleneckSeverity::Low,
            v if v < 0.50 => BottleneckSeverity::Medium,
            v if v < 1.00 => BottleneckSeverity::High,
            _ => BottleneckSeverity::Critical,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            BottleneckSeverity::Low => "Low",
            BottleneckSeverity::Medium => "Medium",
            BottleneckSeverity::High => "High",
            BottleneckSeverity::Critical => "Critical",
        }
    }
}

/// 瓶颈类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BottleneckType {
    CPU,
    GPU,
    Memory,
    Bandwidth,
    Synchronization,
    Unknown,
}

impl BottleneckType {
    pub fn as_str(&self) -> &str {
        match self {
            BottleneckType::CPU => "CPU",
            BottleneckType::GPU => "GPU",
            BottleneckType::Memory => "Memory",
            BottleneckType::Bandwidth => "Bandwidth",
            BottleneckType::Synchronization => "Synchronization",
            BottleneckType::Unknown => "Unknown",
        }
    }
}

/// 单个瓶颈诊断
#[derive(Debug, Clone)]
pub struct BottleneckDiagnosis {
    pub phase_name: String,
    pub bottleneck_type: BottleneckType,
    pub severity: BottleneckSeverity,
    pub variance: f64,
    pub average_duration: Duration,
    pub peak_duration: Duration,
    pub recommendation: String,
    pub frame_count: usize,
}

impl BottleneckDiagnosis {
    pub fn new(
        phase_name: String,
        bottleneck_type: BottleneckType,
        variance: f64,
        average_duration: Duration,
        peak_duration: Duration,
        frame_count: usize,
    ) -> Self {
        let severity = BottleneckSeverity::from_variance(variance);
        let recommendation = Self::generate_recommendation(bottleneck_type, severity);

        Self {
            phase_name,
            bottleneck_type,
            severity,
            variance,
            average_duration,
            peak_duration,
            recommendation,
            frame_count,
        }
    }

    fn generate_recommendation(
        bottleneck_type: BottleneckType,
        severity: BottleneckSeverity,
    ) -> String {
        match (bottleneck_type, severity) {
            (BottleneckType::CPU, BottleneckSeverity::Critical) => {
                "Critical CPU bottleneck. Consider GPU compute shaders or batch processing.".into()
            }
            (BottleneckType::CPU, BottleneckSeverity::High) => {
                "High CPU usage. Profile with flamegraph or reduce algorithm complexity.".into()
            }
            (BottleneckType::GPU, BottleneckSeverity::Critical) => {
                "Critical GPU bottleneck. Check shader complexity or reduce draw calls.".into()
            }
            (BottleneckType::GPU, BottleneckSeverity::High) => {
                "GPU underutilized. Try better work distribution or higher resolution.".into()
            }
            (BottleneckType::Memory, BottleneckSeverity::Critical) => {
                "Critical memory pressure. Enable memory pooling or reduce allocation frequency."
                    .into()
            }
            (BottleneckType::Memory, BottleneckSeverity::High) => {
                "High memory usage. Consider object pooling or streaming assets.".into()
            }
            (BottleneckType::Bandwidth, BottleneckSeverity::Critical) => {
                "Critical bandwidth bottleneck. Use compression or reduce data transfer.".into()
            }
            (BottleneckType::Synchronization, BottleneckSeverity::High) => {
                "High synchronization overhead. Try async operations or lock-free structures."
                    .into()
            }
            _ => "Monitor for further degradation.".into(),
        }
    }

    pub fn description(&self) -> String {
        format!(
            "{} bottleneck in [{}]: {}% variance, avg {:.2}ms, peak {:.2}ms",
            self.severity.as_str(),
            self.phase_name,
            (self.variance * 100.0) as i32,
            self.average_duration.as_secs_f64() * 1000.0,
            self.peak_duration.as_secs_f64() * 1000.0
        )
    }
}

/// 瓶颈检测引擎
#[derive(Default)]
pub struct BottleneckDetector {
    phase_history: HashMap<String, Vec<Duration>>,
    max_history_size: usize,
    variance_threshold: f64,
}

impl BottleneckDetector {
    pub fn new() -> Self {
        Self {
            max_history_size: 300,
            variance_threshold: 0.15,
            ..Default::default()
        }
    }

    pub fn with_variance_threshold(mut self, threshold: f64) -> Self {
        self.variance_threshold = threshold;
        self
    }

    /// 记录阶段性能
    pub fn record_phase(&mut self, phase_name: impl Into<String>, duration: Duration) {
        let name = phase_name.into();
        let entry = self.phase_history.entry(name).or_insert_with(Vec::new);

        if entry.len() >= self.max_history_size {
            entry.remove(0);
        }
        entry.push(duration);
    }

    /// 检测特定阶段的瓶颈
    pub fn detect_phase_bottleneck(&self, phase_name: &str) -> Option<BottleneckDiagnosis> {
        let durations = self.phase_history.get(phase_name)?;

        if durations.len() < 2 {
            return None;
        }

        let average = self.calculate_average(durations);
        let peak = *durations.iter().max()?;
        let variance = self.calculate_variance(durations, average);

        if variance < self.variance_threshold {
            return None;
        }

        // 根据名称推断瓶颈类型
        let bottleneck_type = Self::infer_bottleneck_type(phase_name, variance);

        Some(BottleneckDiagnosis::new(
            phase_name.to_string(),
            bottleneck_type,
            variance,
            average,
            peak,
            durations.len(),
        ))
    }

    /// 检测所有瓶颈
    pub fn detect_all_bottlenecks(&self) -> Vec<BottleneckDiagnosis> {
        self.phase_history
            .keys()
            .filter_map(|name| self.detect_phase_bottleneck(name))
            .collect()
    }

    /// 获取最严重的N个瓶颈
    pub fn get_critical_bottlenecks(&self, count: usize) -> Vec<BottleneckDiagnosis> {
        let mut bottlenecks = self.detect_all_bottlenecks();
        bottlenecks.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| b.variance.partial_cmp(&a.variance).unwrap())
        });
        bottlenecks.truncate(count);
        bottlenecks
    }

    /// 获取GPU相关的瓶颈
    pub fn get_gpu_bottlenecks(&self) -> Vec<BottleneckDiagnosis> {
        self.detect_all_bottlenecks()
            .into_iter()
            .filter(|b| matches!(b.bottleneck_type, BottleneckType::GPU))
            .collect()
    }

    /// 获取CPU相关的瓶颈
    pub fn get_cpu_bottlenecks(&self) -> Vec<BottleneckDiagnosis> {
        self.detect_all_bottlenecks()
            .into_iter()
            .filter(|b| matches!(b.bottleneck_type, BottleneckType::CPU))
            .collect()
    }

    /// 获取内存相关的瓶颈
    pub fn get_memory_bottlenecks(&self) -> Vec<BottleneckDiagnosis> {
        self.detect_all_bottlenecks()
            .into_iter()
            .filter(|b| matches!(b.bottleneck_type, BottleneckType::Memory))
            .collect()
    }

    // 私有辅助函数
    fn calculate_average(&self, durations: &[Duration]) -> Duration {
        if durations.is_empty() {
            return Duration::ZERO;
        }
        let total: u128 = durations.iter().map(|d| d.as_micros()).sum();
        Duration::from_micros((total / durations.len() as u128) as u64)
    }

    fn calculate_variance(&self, durations: &[Duration], average: Duration) -> f64 {
        if durations.len() < 2 {
            return 0.0;
        }

        let avg_micros = average.as_micros() as f64;
        let variance_sum: f64 = durations
            .iter()
            .map(|d| {
                let diff = d.as_micros() as f64 - avg_micros;
                diff * diff
            })
            .sum();

        let variance = variance_sum / durations.len() as f64;
        let std_dev = variance.sqrt();

        if avg_micros == 0.0 {
            0.0
        } else {
            std_dev / avg_micros
        }
    }

    fn infer_bottleneck_type(phase_name: &str, variance: f64) -> BottleneckType {
        let lower = phase_name.to_lowercase();

        if lower.contains("render") || lower.contains("draw") || lower.contains("shader") {
            BottleneckType::GPU
        } else if lower.contains("physics")
            || lower.contains("collision")
            || lower.contains("compute")
        {
            if variance > 0.8 {
                BottleneckType::CPU
            } else {
                BottleneckType::GPU
            }
        } else if lower.contains("memory")
            || lower.contains("allocation")
            || lower.contains("alloc")
        {
            BottleneckType::Memory
        } else if lower.contains("io") || lower.contains("transfer") || lower.contains("network") {
            BottleneckType::Bandwidth
        } else if lower.contains("sync") || lower.contains("wait") || lower.contains("lock") {
            BottleneckType::Synchronization
        } else {
            BottleneckType::Unknown
        }
    }

    /// 清空历史记录
    pub fn clear(&mut self) {
        self.phase_history.clear();
    }

    /// 获取记录的阶段数
    pub fn phase_count(&self) -> usize {
        self.phase_history.len()
    }

    /// 获取特定阶段的样本数
    pub fn sample_count(&self, phase_name: &str) -> usize {
        self.phase_history
            .get(phase_name)
            .map(|v| v.len())
            .unwrap_or(0)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bottleneck_severity() {
        assert_eq!(
            BottleneckSeverity::from_variance(0.10),
            BottleneckSeverity::Low
        );
        assert_eq!(
            BottleneckSeverity::from_variance(0.35),
            BottleneckSeverity::Medium
        );
        assert_eq!(
            BottleneckSeverity::from_variance(0.75),
            BottleneckSeverity::High
        );
        assert_eq!(
            BottleneckSeverity::from_variance(1.50),
            BottleneckSeverity::Critical
        );
    }

    #[test]
    fn test_bottleneck_diagnosis() {
        let diagnosis = BottleneckDiagnosis::new(
            "render".to_string(),
            BottleneckType::GPU,
            0.75,
            Duration::from_micros(10000),
            Duration::from_micros(20000),
            100,
        );

        assert_eq!(diagnosis.severity, BottleneckSeverity::High);
        assert_eq!(diagnosis.bottleneck_type, BottleneckType::GPU);
        assert!(!diagnosis.recommendation.is_empty());
    }

    #[test]
    fn test_bottleneck_detection() {
        let mut detector = BottleneckDetector::new().with_variance_threshold(0.15);

        // 添加稳定的物理计算
        for i in 0..50 {
            detector.record_phase("physics", Duration::from_micros(5000 + i % 100));
        }

        // 添加高度不稳定的渲染（更大的变异）
        for i in 0..50 {
            detector.record_phase("render", Duration::from_micros(10000 + i * 1500));
        }

        let physics_bottleneck = detector.detect_phase_bottleneck("physics");
        let render_bottleneck = detector.detect_phase_bottleneck("render");

        assert!(physics_bottleneck.is_none());
        assert!(render_bottleneck.is_some());

        let render_diag = render_bottleneck.unwrap();
        // Higher variance should trigger at least Medium bottleneck
        assert!(render_diag.severity >= BottleneckSeverity::Medium);
    }

    #[test]
    fn test_bottleneck_type_inference() {
        let render_type = BottleneckDetector::infer_bottleneck_type("render", 0.5);
        assert_eq!(render_type, BottleneckType::GPU);

        let physics_type = BottleneckDetector::infer_bottleneck_type("physics_compute", 0.5);
        assert_eq!(physics_type, BottleneckType::GPU);

        let memory_type = BottleneckDetector::infer_bottleneck_type("memory_alloc", 0.5);
        assert_eq!(memory_type, BottleneckType::Memory);

        let sync_type = BottleneckDetector::infer_bottleneck_type("sync_wait", 0.5);
        assert_eq!(sync_type, BottleneckType::Synchronization);
    }

    #[test]
    fn test_get_critical_bottlenecks() {
        let mut detector = BottleneckDetector::new();

        for i in 0..50 {
            detector.record_phase("phase1", Duration::from_micros(5000 + i * 100));
            detector.record_phase("phase2", Duration::from_micros(10000 + i * 500));
            detector.record_phase("phase3", Duration::from_micros(15000 + i * 50));
        }

        let critical = detector.get_critical_bottlenecks(2);
        // Both phase1 and phase2 have variance, with phase2 being more severe
        assert!(critical.len() >= 1);
        assert_eq!(critical[0].phase_name, "phase2");
    }

    #[test]
    fn test_get_gpu_bottlenecks() {
        let mut detector = BottleneckDetector::new();

        for i in 0..50 {
            detector.record_phase("render", Duration::from_micros(10000 + i * 500));
            detector.record_phase("physics", Duration::from_micros(5000 + i * 200));
        }

        let gpu_bottlenecks = detector.get_gpu_bottlenecks();
        assert!(gpu_bottlenecks.len() > 0);
    }

    #[test]
    fn test_detector_statistics() {
        let mut detector = BottleneckDetector::new();

        for _ in 0..20 {
            detector.record_phase("render", Duration::from_micros(1000));
            detector.record_phase("physics", Duration::from_micros(500));
        }

        assert_eq!(detector.phase_count(), 2);
        assert_eq!(detector.sample_count("render"), 20);
        assert_eq!(detector.sample_count("physics"), 20);
        assert_eq!(detector.sample_count("nonexistent"), 0);
    }
}
