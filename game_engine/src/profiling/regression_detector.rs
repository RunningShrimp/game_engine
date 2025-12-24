//! 性能回归检测器
//!
//! 提供自动性能回归检测功能，支持基线管理、阈值配置和CI/CD集成。
//! 能够检测性能退化并生成详细的回归报告。

use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// 回归严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegressionSeverity {
    /// 轻微回归（可接受范围内）
    Minor,
    /// 中等回归（需要关注）
    Moderate,
    /// 严重回归（需要立即修复）
    Severe,
}

impl RegressionSeverity {
    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Minor => "Minor",
            Self::Moderate => "Moderate",
            Self::Severe => "Severe",
        }
    }

    /// 转换为数字（用于排序）
    pub fn as_number(&self) -> u8 {
        match self {
            Self::Minor => 1,
            Self::Moderate => 2,
            Self::Severe => 3,
        }
    }
}

/// 性能基线
#[derive(Debug, Clone)]
pub struct PerformanceBaseline {
    /// 平均FPS
    pub avg_fps: f64,
    /// 第95百分位帧时间
    pub p95_frame_time: Duration,
    /// 第99百分位帧时间
    pub p99_frame_time: Duration,
    /// 平均内存使用（MB）
    pub avg_memory_mb: f64,
    /// 时间戳
    pub timestamp: SystemTime,
    /// 样本数量
    pub sample_count: usize,
    /// 环境信息（用于基线比较）
    pub environment: HashMap<String, String>,
}

impl PerformanceBaseline {
    /// 创建新的性能基线
    pub fn new(
        avg_fps: f64,
        p95_frame_time: Duration,
        p99_frame_time: Duration,
        avg_memory_mb: f64,
        sample_count: usize,
    ) -> Self {
        Self {
            avg_fps,
            p95_frame_time,
            p99_frame_time,
            avg_memory_mb,
            timestamp: SystemTime::now(),
            sample_count,
            environment: HashMap::new(),
        }
    }

    /// 添加环境信息
    pub fn with_environment(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.environment.insert(key.into(), value.into());
        self
    }

    /// 设置环境信息
    pub fn set_environment(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.environment.insert(key.into(), value.into());
    }
}

/// 回归阈值配置
#[derive(Debug, Clone)]
pub struct RegressionThresholds {
    /// FPS下降百分比阈值
    pub fps_degradation_percent: f64,
    /// 帧时间增加百分比阈值
    pub frame_time_increase_percent: f64,
    /// 内存增加百分比阈值
    pub memory_increase_percent: f64,
    /// 最小样本数（用于统计显著性）
    pub min_samples: usize,
    /// 轻微回归阈值（百分比）
    pub minor_threshold: f64,
    /// 中等回归阈值（百分比）
    pub moderate_threshold: f64,
    /// 严重回归阈值（百分比）
    pub severe_threshold: f64,
}

impl Default for RegressionThresholds {
    fn default() -> Self {
        Self {
            fps_degradation_percent: 5.0,     // 5% FPS下降
            frame_time_increase_percent: 5.0, // 5% 帧时间增加
            memory_increase_percent: 10.0,    // 10% 内存增加
            min_samples: 30,                  // 至少30个样本
            minor_threshold: 5.0,             // 5% 轻微回归
            moderate_threshold: 10.0,         // 10% 中等回归
            severe_threshold: 20.0,           // 20% 严重回归
        }
    }
}

/// 性能回归
#[derive(Debug, Clone)]
pub struct PerformanceRegression {
    /// 指标名称
    pub metric_name: String,
    /// 基线值
    pub baseline_value: f64,
    /// 当前值
    pub current_value: f64,
    /// 回归百分比
    pub regression_percent: f64,
    /// 严重程度
    pub severity: RegressionSeverity,
    /// 时间戳
    pub timestamp: SystemTime,
    /// 建议的修复措施
    pub suggested_fix: Option<String>,
}

impl PerformanceRegression {
    /// 创建新的性能回归
    pub fn new(
        metric_name: impl Into<String>,
        baseline_value: f64,
        current_value: f64,
        regression_percent: f64,
        severity: RegressionSeverity,
    ) -> Self {
        let metric_name = metric_name.into();
        let suggested_fix = Self::suggest_fix(&metric_name, severity);

        Self {
            metric_name,
            baseline_value,
            current_value,
            regression_percent,
            severity,
            timestamp: SystemTime::now(),
            suggested_fix,
        }
    }

    /// 根据指标和严重程度建议修复措施
    fn suggest_fix(metric_name: &str, severity: RegressionSeverity) -> Option<String> {
        let base_suggestion = match metric_name {
            "fps" | "frame_rate" => match severity {
                RegressionSeverity::Minor => "检查最近的代码更改，优化渲染循环",
                RegressionSeverity::Moderate => "分析性能热点，优化关键路径，考虑LOD",
                RegressionSeverity::Severe => "立即回滚最近的更改，进行深度性能分析",
            },
            "frame_time" | "p95_frame_time" | "p99_frame_time" => match severity {
                RegressionSeverity::Minor => "检查帧时间分布，优化慢帧",
                RegressionSeverity::Moderate => "识别并优化性能瓶颈，减少过度绘制",
                RegressionSeverity::Severe => "紧急优化，检查是否有内存泄漏或死锁",
            },
            "memory" | "memory_usage" => match severity {
                RegressionSeverity::Minor => "检查内存分配模式，优化资源管理",
                RegressionSeverity::Moderate => "检查内存泄漏，优化资源加载策略",
                RegressionSeverity::Severe => "立即检查内存泄漏，审查资源生命周期",
            },
            _ => "检查相关代码更改，进行性能分析",
        };

        Some(base_suggestion.to_string())
    }
}

/// 性能回归检测器
pub struct PerformanceRegressionDetector {
    /// 性能基线
    baseline: Option<PerformanceBaseline>,
    /// 回归阈值
    thresholds: RegressionThresholds,
    /// 检测到的回归列表
    regressions: Vec<PerformanceRegression>,
    /// 当前样本收集
    current_samples: Vec<PerformanceSample>,
}

/// 性能样本
#[derive(Debug, Clone)]
struct PerformanceSample {
    /// FPS
    fps: f64,
    /// 帧时间
    frame_time: Duration,
    /// 内存使用（MB）
    memory_mb: f64,
    /// 时间戳
    timestamp: SystemTime,
}

impl PerformanceRegressionDetector {
    /// 创建新的性能回归检测器
    pub fn new(thresholds: RegressionThresholds) -> Self {
        Self {
            baseline: None,
            thresholds,
            regressions: Vec::new(),
            current_samples: Vec::new(),
        }
    }

    /// 使用默认阈值创建
    pub fn with_default_thresholds() -> Self {
        Self::new(RegressionThresholds::default())
    }

    /// 设置性能基线
    pub fn set_baseline(&mut self, baseline: PerformanceBaseline) {
        self.baseline = Some(baseline);
        self.regressions.clear();
    }

    /// 添加性能样本
    pub fn add_sample(&mut self, fps: f64, frame_time: Duration, memory_mb: f64) {
        self.current_samples.push(PerformanceSample {
            fps,
            frame_time,
            memory_mb,
            timestamp: SystemTime::now(),
        });
    }

    /// 检测回归
    pub fn detect_regressions(&mut self) -> Vec<PerformanceRegression> {
        self.regressions.clear();

        let baseline = match &self.baseline {
            Some(b) => b,
            None => {
                // 如果没有基线，无法检测回归
                return Vec::new();
            }
        };

        // 检查是否有足够的样本
        if self.current_samples.len() < self.thresholds.min_samples {
            return Vec::new();
        }

        // 计算当前性能指标
        let avg_fps = self.current_samples.iter().map(|s| s.fps).sum::<f64>()
            / self.current_samples.len() as f64;

        let mut frame_times: Vec<Duration> =
            self.current_samples.iter().map(|s| s.frame_time).collect();
        frame_times.sort();

        let p95_index = (frame_times.len() as f64 * 0.95) as usize;
        let p99_index = (frame_times.len() as f64 * 0.99) as usize;
        let p95_frame_time = frame_times
            .get(p95_index.min(frame_times.len() - 1))
            .copied()
            .unwrap_or(Duration::ZERO);
        let p99_frame_time = frame_times
            .get(p99_index.min(frame_times.len() - 1))
            .copied()
            .unwrap_or(Duration::ZERO);

        let avg_memory_mb = self.current_samples.iter().map(|s| s.memory_mb).sum::<f64>()
            / self.current_samples.len() as f64;

        // 检测FPS回归
        if avg_fps < baseline.avg_fps {
            let degradation_percent = ((baseline.avg_fps - avg_fps) / baseline.avg_fps) * 100.0;
            if degradation_percent >= self.thresholds.fps_degradation_percent {
                let severity = self.determine_severity(degradation_percent);
                self.regressions.push(PerformanceRegression::new(
                    "fps",
                    baseline.avg_fps,
                    avg_fps,
                    degradation_percent,
                    severity,
                ));
            }
        }

        // 检测帧时间回归
        if p95_frame_time > baseline.p95_frame_time {
            let increase_percent = ((p95_frame_time.as_nanos() as f64
                - baseline.p95_frame_time.as_nanos() as f64)
                / baseline.p95_frame_time.as_nanos() as f64)
                * 100.0;
            if increase_percent >= self.thresholds.frame_time_increase_percent {
                let severity = self.determine_severity(increase_percent);
                self.regressions.push(PerformanceRegression::new(
                    "p95_frame_time",
                    baseline.p95_frame_time.as_secs_f64() * 1000.0,
                    p95_frame_time.as_secs_f64() * 1000.0,
                    increase_percent,
                    severity,
                ));
            }
        }

        if p99_frame_time > baseline.p99_frame_time {
            let increase_percent = ((p99_frame_time.as_nanos() as f64
                - baseline.p99_frame_time.as_nanos() as f64)
                / baseline.p99_frame_time.as_nanos() as f64)
                * 100.0;
            if increase_percent >= self.thresholds.frame_time_increase_percent {
                let severity = self.determine_severity(increase_percent);
                self.regressions.push(PerformanceRegression::new(
                    "p99_frame_time",
                    baseline.p99_frame_time.as_secs_f64() * 1000.0,
                    p99_frame_time.as_secs_f64() * 1000.0,
                    increase_percent,
                    severity,
                ));
            }
        }

        // 检测内存回归
        if avg_memory_mb > baseline.avg_memory_mb {
            let increase_percent =
                ((avg_memory_mb - baseline.avg_memory_mb) / baseline.avg_memory_mb) * 100.0;
            if increase_percent >= self.thresholds.memory_increase_percent {
                let severity = self.determine_severity(increase_percent);
                self.regressions.push(PerformanceRegression::new(
                    "memory_usage",
                    baseline.avg_memory_mb,
                    avg_memory_mb,
                    increase_percent,
                    severity,
                ));
            }
        }

        // 按严重程度排序
        self.regressions
            .sort_by(|a, b| b.severity.as_number().cmp(&a.severity.as_number()));

        self.regressions.clone()
    }

    /// 确定回归严重程度
    fn determine_severity(&self, regression_percent: f64) -> RegressionSeverity {
        if regression_percent >= self.thresholds.severe_threshold {
            RegressionSeverity::Severe
        } else if regression_percent >= self.thresholds.moderate_threshold {
            RegressionSeverity::Moderate
        } else {
            RegressionSeverity::Minor
        }
    }

    /// 获取检测到的回归
    pub fn get_regressions(&self) -> &[PerformanceRegression] {
        &self.regressions
    }

    /// 获取严重回归
    pub fn get_severe_regressions(&self) -> Vec<&PerformanceRegression> {
        self.regressions
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Severe)
            .collect()
    }

    /// 检查是否有严重回归（用于CI/CD）
    pub fn has_severe_regression(&self) -> bool {
        !self.get_severe_regressions().is_empty()
    }

    /// 获取基线
    pub fn baseline(&self) -> Option<&PerformanceBaseline> {
        self.baseline.as_ref()
    }

    /// 清空当前样本
    pub fn clear_samples(&mut self) {
        self.current_samples.clear();
    }

    /// 获取当前样本数
    pub fn sample_count(&self) -> usize {
        self.current_samples.len()
    }

    /// 生成CI/CD友好的报告（JSON格式）
    pub fn generate_cicd_report(&self) -> serde_json::Value {
        let severe_count = self.get_severe_regressions().len();
        let moderate_count = self
            .regressions
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Moderate)
            .count();
        let minor_count = self
            .regressions
            .iter()
            .filter(|r| r.severity == RegressionSeverity::Minor)
            .count();

        serde_json::json!({
            "has_regression": !self.regressions.is_empty(),
            "has_severe_regression": severe_count > 0,
            "regression_count": {
                "severe": severe_count,
                "moderate": moderate_count,
                "minor": minor_count,
                "total": self.regressions.len()
            },
            "regressions": self.regressions.iter().map(|r| {
                serde_json::json!({
                    "metric": r.metric_name,
                    "baseline": r.baseline_value,
                    "current": r.current_value,
                    "regression_percent": r.regression_percent,
                    "severity": r.severity.as_str(),
                    "suggested_fix": r.suggested_fix
                })
            }).collect::<Vec<_>>(),
            "sample_count": self.sample_count(),
            "baseline_set": self.baseline.is_some()
        })
    }

    /// 保存基线到文件
    pub fn save_baseline(
        &self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let baseline = self.baseline.as_ref().ok_or("No baseline set")?;
        let json = serde_json::to_string_pretty(baseline)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    /// 从文件加载基线
    pub fn load_baseline(
        &mut self,
        path: impl AsRef<std::path::Path>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let baseline: PerformanceBaseline = serde_json::from_str(&json)?;
        self.set_baseline(baseline);
        Ok(())
    }
}

// 为PerformanceBaseline添加序列化支持
impl serde::Serialize for PerformanceBaseline {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("PerformanceBaseline", 7)?;
        state.serialize_field("avg_fps", &self.avg_fps)?;
        let p95_ms = self.p95_frame_time.as_secs_f64() * 1000.0;
        let p99_ms = self.p99_frame_time.as_secs_f64() * 1000.0;
        state.serialize_field("p95_frame_time_ms", &p95_ms)?;
        state.serialize_field("p99_frame_time_ms", &p99_ms)?;
        state.serialize_field("avg_memory_mb", &self.avg_memory_mb)?;
        state.serialize_field(
            "timestamp",
            &self.timestamp.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
        )?;
        state.serialize_field("sample_count", &self.sample_count)?;
        state.serialize_field("environment", &self.environment)?;
        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for PerformanceBaseline {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct PerformanceBaselineVisitor;

        impl<'de> Visitor<'de> for PerformanceBaselineVisitor {
            type Value = PerformanceBaseline;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct PerformanceBaseline")
            }

            fn visit_map<V>(self, mut map: V) -> Result<PerformanceBaseline, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut avg_fps = None;
                let mut p95_frame_time_ms = None;
                let mut p99_frame_time_ms = None;
                let mut avg_memory_mb = None;
                let mut timestamp_secs = None;
                let mut sample_count = None;
                let mut environment = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "avg_fps" => {
                            if avg_fps.is_some() {
                                return Err(de::Error::duplicate_field("avg_fps"));
                            }
                            avg_fps = Some(map.next_value()?);
                        }
                        "p95_frame_time_ms" => {
                            if p95_frame_time_ms.is_some() {
                                return Err(de::Error::duplicate_field("p95_frame_time_ms"));
                            }
                            p95_frame_time_ms = Some(map.next_value()?);
                        }
                        "p99_frame_time_ms" => {
                            if p99_frame_time_ms.is_some() {
                                return Err(de::Error::duplicate_field("p99_frame_time_ms"));
                            }
                            p99_frame_time_ms = Some(map.next_value()?);
                        }
                        "avg_memory_mb" => {
                            if avg_memory_mb.is_some() {
                                return Err(de::Error::duplicate_field("avg_memory_mb"));
                            }
                            avg_memory_mb = Some(map.next_value()?);
                        }
                        "timestamp" => {
                            if timestamp_secs.is_some() {
                                return Err(de::Error::duplicate_field("timestamp"));
                            }
                            timestamp_secs = Some(map.next_value()?);
                        }
                        "sample_count" => {
                            if sample_count.is_some() {
                                return Err(de::Error::duplicate_field("sample_count"));
                            }
                            sample_count = Some(map.next_value()?);
                        }
                        "environment" => {
                            if environment.is_some() {
                                return Err(de::Error::duplicate_field("environment"));
                            }
                            environment = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let avg_fps = avg_fps.ok_or_else(|| de::Error::missing_field("avg_fps"))?;
                let p95_frame_time_ms: f64 = p95_frame_time_ms
                    .ok_or_else(|| de::Error::missing_field("p95_frame_time_ms"))?;
                let p99_frame_time_ms: f64 = p99_frame_time_ms
                    .ok_or_else(|| de::Error::missing_field("p99_frame_time_ms"))?;
                let avg_memory_mb =
                    avg_memory_mb.ok_or_else(|| de::Error::missing_field("avg_memory_mb"))?;
                let sample_count = sample_count.unwrap_or(0);
                let environment = environment.unwrap_or_default();

                let timestamp =
                    std::time::UNIX_EPOCH + Duration::from_secs(timestamp_secs.unwrap_or(0));

                Ok(PerformanceBaseline {
                    avg_fps,
                    p95_frame_time: Duration::from_secs_f64(p95_frame_time_ms / 1000.0),
                    p99_frame_time: Duration::from_secs_f64(p99_frame_time_ms / 1000.0),
                    avg_memory_mb,
                    timestamp,
                    sample_count,
                    environment,
                })
            }
        }

        const FIELDS: &[&str] = &[
            "avg_fps",
            "p95_frame_time_ms",
            "p99_frame_time_ms",
            "avg_memory_mb",
            "timestamp",
            "sample_count",
            "environment",
        ];
        deserializer.deserialize_struct("PerformanceBaseline", FIELDS, PerformanceBaselineVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regression_detector() {
        let mut detector = PerformanceRegressionDetector::with_default_thresholds();

        // 设置基线
        let baseline = PerformanceBaseline::new(
            60.0,
            Duration::from_millis(16),
            Duration::from_millis(20),
            256.0,
            100,
        );
        detector.set_baseline(baseline);

        // 添加性能下降的样本
        for _ in 0..50 {
            detector.add_sample(55.0, Duration::from_millis(18), 280.0);
        }

        let regressions = detector.detect_regressions();
        assert!(!regressions.is_empty());
    }

    #[test]
    fn test_severity_determination() {
        let thresholds = RegressionThresholds {
            minor_threshold: 5.0,
            moderate_threshold: 10.0,
            severe_threshold: 20.0,
            ..Default::default()
        };
        let detector = PerformanceRegressionDetector::new(thresholds);

        assert_eq!(detector.determine_severity(3.0), RegressionSeverity::Minor);
        assert_eq!(
            detector.determine_severity(15.0),
            RegressionSeverity::Moderate
        );
        assert_eq!(
            detector.determine_severity(25.0),
            RegressionSeverity::Severe
        );
    }

    #[test]
    fn test_cicd_report() {
        let mut detector = PerformanceRegressionDetector::with_default_thresholds();
        let baseline = PerformanceBaseline::new(
            60.0,
            Duration::from_millis(16),
            Duration::from_millis(20),
            256.0,
            100,
        );
        detector.set_baseline(baseline);

        for _ in 0..50 {
            detector.add_sample(50.0, Duration::from_millis(20), 300.0);
        }

        detector.detect_regressions();
        let report = detector.generate_cicd_report();

        assert!(report["has_regression"].as_bool().unwrap());
        assert!(report["regression_count"]["total"].as_u64().unwrap() > 0);
    }
}
