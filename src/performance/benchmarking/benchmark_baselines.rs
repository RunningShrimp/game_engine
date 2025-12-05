//! 性能基准测试套件
//!
//! 建立性能基线并跟踪优化效果
//! - 关键操作基准测试
//! - 性能回归检测
//! - 基线管理
//! - 对比分析

use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// 基准测试结果数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkBaseline {
    /// 基准名称
    pub name: String,
    /// 操作平均时间 (微秒)
    pub avg_time_us: f64,
    /// 最小时间 (微秒)
    pub min_time_us: f64,
    /// 最大时间 (微秒)
    pub max_time_us: f64,
    /// 标准差
    pub stddev_us: f64,
    /// 操作次数
    pub operation_count: u64,
    /// 创建时间戳
    pub timestamp: String,
    /// 系统信息
    pub system_info: String,
}

impl BenchmarkBaseline {
    /// 创建新的基准
    pub fn new(name: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_default();

        Self {
            name,
            avg_time_us: 0.0,
            min_time_us: 0.0,
            max_time_us: 0.0,
            stddev_us: 0.0,
            operation_count: 0,
            timestamp,
            system_info: "Rust game engine".to_string(),
        }
    }
}

/// 性能回归检测器
#[derive(Debug)]
pub struct RegressionDetector {
    /// 基准数据
    baselines: HashMap<String, BenchmarkBaseline>,
    /// 回归阈值 (%)
    regression_threshold: f32,
    /// 改进阈值 (%)
    improvement_threshold: f32,
}

impl RegressionDetector {
    /// 创建新的回归检测器
    pub fn new(regression_threshold: f32, improvement_threshold: f32) -> Self {
        Self {
            baselines: HashMap::new(),
            regression_threshold,
            improvement_threshold,
        }
    }

    /// 设置基准
    pub fn set_baseline(&mut self, baseline: BenchmarkBaseline) {
        self.baselines.insert(baseline.name.clone(), baseline);
    }

    /// 检测回归
    pub fn detect_regression(&self, name: &str, current_time_us: f64) -> Option<RegressionReport> {
        let baseline = self.baselines.get(name)?;

        let percent_change =
            ((current_time_us - baseline.avg_time_us) / baseline.avg_time_us) * 100.0;

        if percent_change > self.regression_threshold as f64 {
            Some(RegressionReport {
                name: name.to_string(),
                baseline_time_us: baseline.avg_time_us,
                current_time_us,
                percent_change,
                is_regression: true,
                severity: if percent_change > 50.0 {
                    "Critical"
                } else {
                    "Warning"
                }
                .to_string(),
            })
        } else if percent_change < -(self.improvement_threshold as f64) {
            Some(RegressionReport {
                name: name.to_string(),
                baseline_time_us: baseline.avg_time_us,
                current_time_us,
                percent_change,
                is_regression: false,
                severity: "Improvement".to_string(),
            })
        } else {
            None
        }
    }

    /// 保存基准到文件
    pub fn save_baselines<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let baselines: Vec<_> = self.baselines.values().collect();
        let json = serde_json::to_string_pretty(&baselines)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// 从文件加载基准
    pub fn load_baselines<P: AsRef<Path>>(
        &mut self,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        let baselines: Vec<BenchmarkBaseline> = serde_json::from_str(&json)?;

        for baseline in baselines {
            self.baselines.insert(baseline.name.clone(), baseline);
        }
        Ok(())
    }
}

/// 回归报告
#[derive(Debug, Clone)]
pub struct RegressionReport {
    /// 基准名称
    pub name: String,
    /// 基准时间 (微秒)
    pub baseline_time_us: f64,
    /// 当前时间 (微秒)
    pub current_time_us: f64,
    /// 变化百分比
    pub percent_change: f64,
    /// 是否为回归
    pub is_regression: bool,
    /// 严重级别
    pub severity: String,
}

/// 关键路径基准测试套件
pub struct CriticalPathBenchmarks;

impl CriticalPathBenchmarks {
    /// 运行所有关键路径基准测试
    pub fn run_all() -> HashMap<String, BenchmarkBaseline> {
        let mut results = HashMap::new();

        // 向量操作
        results.insert("vec3_add".to_string(), Self::benchmark_vec3_add());
        results.insert(
            "vec3_normalize".to_string(),
            Self::benchmark_vec3_normalize(),
        );
        results.insert("vec3_dot".to_string(), Self::benchmark_vec3_dot());

        // 矩阵操作
        results.insert("mat4_multiply".to_string(), Self::benchmark_mat4_multiply());
        results.insert(
            "mat4_transform".to_string(),
            Self::benchmark_mat4_transform(),
        );

        // 四元数操作
        results.insert("quat_multiply".to_string(), Self::benchmark_quat_multiply());
        results.insert(
            "quat_normalize".to_string(),
            Self::benchmark_quat_normalize(),
        );

        // 内存操作
        results.insert(
            "vector_allocate".to_string(),
            Self::benchmark_vector_allocate(),
        );
        results.insert(
            "hashmap_insert".to_string(),
            Self::benchmark_hashmap_insert(),
        );

        results
    }

    /// Vec3 加法基准
    fn benchmark_vec3_add() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("vec3_add".to_string());
        let iterations = 100_000_000u64;

        let start = std::time::Instant::now();
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        for _ in 0..iterations {
            let _ = v1 + v2;
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }

    /// Vec3 归一化基准
    fn benchmark_vec3_normalize() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("vec3_normalize".to_string());
        let iterations = 10_000_000u64;

        let start = std::time::Instant::now();
        let v = Vec3::new(1.0, 2.0, 3.0);

        for _ in 0..iterations {
            let _ = v.normalize();
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }

    /// Vec3 点积基准
    fn benchmark_vec3_dot() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("vec3_dot".to_string());
        let iterations = 100_000_000u64;

        let start = std::time::Instant::now();
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);

        for _ in 0..iterations {
            let _ = v1.dot(v2);
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }

    /// Mat4 乘法基准
    fn benchmark_mat4_multiply() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("mat4_multiply".to_string());
        let iterations = 1_000_000u64;

        let start = std::time::Instant::now();
        let m1 = Mat4::IDENTITY;
        let m2 = Mat4::IDENTITY;

        for _ in 0..iterations {
            let _ = m1 * m2;
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }

    /// Mat4 变换基准
    fn benchmark_mat4_transform() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("mat4_transform".to_string());
        let iterations = 10_000_000u64;

        let start = std::time::Instant::now();
        let m = Mat4::IDENTITY;
        let v = Vec3::new(1.0, 2.0, 3.0);

        for _ in 0..iterations {
            let _ = m.transform_point3(v);
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }

    /// 四元数乘法基准
    fn benchmark_quat_multiply() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("quat_multiply".to_string());
        let iterations = 10_000_000u64;

        let start = std::time::Instant::now();
        let q1 = Quat::IDENTITY;
        let q2 = Quat::IDENTITY;

        for _ in 0..iterations {
            let _ = q1 * q2;
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }

    /// 四元数归一化基准
    fn benchmark_quat_normalize() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("quat_normalize".to_string());
        let iterations = 10_000_000u64;

        let start = std::time::Instant::now();
        let q = Quat::IDENTITY;

        for _ in 0..iterations {
            let _ = q.normalize();
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }

    /// 向量分配基准
    fn benchmark_vector_allocate() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("vector_allocate".to_string());
        let iterations = 100_000u64;

        let start = std::time::Instant::now();

        for _ in 0..iterations {
            let mut v: Vec<f32> = Vec::with_capacity(1000);
            for i in 0..1000 {
                v.push(i as f32);
            }
            let _ = v.len();
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }

    /// HashMap 插入基准
    fn benchmark_hashmap_insert() -> BenchmarkBaseline {
        let mut baseline = BenchmarkBaseline::new("hashmap_insert".to_string());
        let iterations = 10_000u64;

        let start = std::time::Instant::now();

        for _ in 0..iterations {
            let mut map: HashMap<i32, String> = HashMap::new();
            for i in 0..100 {
                map.insert(i, format!("key_{}", i));
            }
            let _ = map.len();
        }

        let elapsed = start.elapsed().as_secs_f64();
        baseline.avg_time_us = (elapsed * 1_000_000.0) / iterations as f64;
        baseline.operation_count = iterations;
        baseline
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_baseline_creation() {
        let baseline = BenchmarkBaseline::new("test".to_string());
        assert_eq!(baseline.name, "test");
        assert_eq!(baseline.avg_time_us, 0.0);
    }

    #[test]
    fn test_regression_detector() {
        let mut detector = RegressionDetector::new(10.0, 10.0);

        let mut baseline = BenchmarkBaseline::new("test_op".to_string());
        baseline.avg_time_us = 100.0;
        detector.set_baseline(baseline);

        // 不超过阈值
        let report = detector.detect_regression("test_op", 105.0);
        assert!(report.is_none());

        // 超过阈值 (>10%)
        let report = detector.detect_regression("test_op", 120.0);
        assert!(report.is_some());

        let report = report.unwrap();
        assert!(report.is_regression);
        assert!(report.percent_change > 10.0);
    }

    #[test]
    fn test_critical_path_benchmarks() {
        let results = CriticalPathBenchmarks::run_all();
        assert!(results.len() > 0);

        // 验证每个基准都有结果
        for (name, baseline) in results {
            assert!(!name.is_empty());
            assert!(baseline.avg_time_us > 0.0);
            assert!(baseline.operation_count > 0);
        }
    }

    #[test]
    fn test_vec3_benchmark() {
        let baseline = CriticalPathBenchmarks::benchmark_vec3_add();
        assert_eq!(baseline.name, "vec3_add");
        assert!(baseline.avg_time_us > 0.0);
    }

    #[test]
    fn test_improvement_detection() {
        let detector = RegressionDetector::new(10.0, 10.0);

        let mut baseline = BenchmarkBaseline::new("test_op".to_string());
        baseline.avg_time_us = 100.0;
        let detector = {
            let mut d = detector;
            d.set_baseline(baseline);
            d
        };

        // 改进情况
        let report = detector.detect_regression("test_op", 85.0);
        assert!(report.is_some());

        let report = report.unwrap();
        assert!(!report.is_regression);
        assert_eq!(report.severity, "Improvement");
    }
}
