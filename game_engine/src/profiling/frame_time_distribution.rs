//! 帧时间分布分析
//!
//! 提供帧时间的分布分析、异常检测和可视化功能。
//! 支持识别卡顿、掉帧和帧率稳定性分析。

use std::collections::VecDeque;
use std::time::Duration;

/// 帧异常
#[derive(Debug, Clone)]
pub struct FrameOutlier {
    /// 帧索引
    pub index: usize,
    /// 帧持续时间
    pub duration: Duration,
    /// 偏差（标准差的倍数）
    pub deviation: f64,
    /// 可能的原因
    pub possible_cause: String,
}

impl FrameOutlier {
    /// 创建新的帧异常
    pub fn new(index: usize, duration: Duration, deviation: f64, cause: impl Into<String>) -> Self {
        Self {
            index,
            duration,
            deviation,
            possible_cause: cause.into(),
        }
    }
}

/// 帧时间分析器
pub struct FrameTimeAnalyzer {
    /// 帧时间样本
    samples: VecDeque<Duration>,
    /// 最大样本数
    max_samples: usize,
    /// 百分位数（50th, 95th, 99th, 99.9th）
    percentiles: [Duration; 4],
    /// 异常帧列表
    outliers: Vec<FrameOutlier>,
    /// 平均值
    mean: Duration,
    /// 标准差
    std_dev: Duration,
    /// 是否已计算统计信息
    stats_computed: bool,
}

impl FrameTimeAnalyzer {
    /// 创建新的帧时间分析器
    ///
    /// # 参数
    /// - `max_samples`: 最大样本数
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(max_samples),
            max_samples,
            percentiles: [Duration::ZERO; 4],
            outliers: Vec::new(),
            mean: Duration::ZERO,
            std_dev: Duration::ZERO,
            stats_computed: false,
        }
    }

    /// 添加帧时间样本
    pub fn add_sample(&mut self, duration: Duration) {
        if self.samples.len() >= self.max_samples {
            self.samples.pop_front();
        }
        self.samples.push_back(duration);
        self.stats_computed = false;
    }

    /// 计算统计信息
    pub fn compute_stats(&mut self) {
        if self.samples.is_empty() {
            return;
        }

        // 计算平均值
        let total: Duration = self.samples.iter().sum();
        self.mean = Duration::from_nanos(total.as_nanos() as u64 / self.samples.len() as u64);

        // 计算标准差
        let variance: f64 = self
            .samples
            .iter()
            .map(|d| {
                let diff = d.as_nanos() as f64 - self.mean.as_nanos() as f64;
                diff * diff
            })
            .sum::<f64>()
            / self.samples.len() as f64;
        self.std_dev = Duration::from_nanos(variance.sqrt() as u64);

        // 计算百分位数
        let mut sorted: Vec<Duration> = self.samples.iter().copied().collect();
        sorted.sort();

        self.percentiles[0] = self.percentile(&sorted, 0.50); // 50th
        self.percentiles[1] = self.percentile(&sorted, 0.95); // 95th
        self.percentiles[2] = self.percentile(&sorted, 0.99); // 99th
        self.percentiles[3] = self.percentile(&sorted, 0.999); // 99.9th

        // 检测异常
        self.detect_outliers();

        self.stats_computed = true;
    }

    /// 计算百分位数
    fn percentile(&self, sorted: &[Duration], p: f64) -> Duration {
        if sorted.is_empty() {
            return Duration::ZERO;
        }
        let index = ((sorted.len() - 1) as f64 * p) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    /// 检测异常帧
    fn detect_outliers(&mut self) {
        self.outliers.clear();

        if self.std_dev.as_nanos() == 0 {
            return;
        }

        let threshold = self.mean.as_nanos() as f64 + 3.0 * self.std_dev.as_nanos() as f64;
        let threshold_duration = Duration::from_nanos(threshold as u64);

        for (i, &duration) in self.samples.iter().enumerate() {
            if duration > threshold_duration {
                let deviation = (duration.as_nanos() as f64 - self.mean.as_nanos() as f64)
                    / self.std_dev.as_nanos() as f64;
                let cause = self.analyze_cause(duration);
                self.outliers.push(FrameOutlier::new(i, duration, deviation, cause));
            }
        }
    }

    /// 分析异常原因
    fn analyze_cause(&self, duration: Duration) -> String {
        let fps = 1.0 / duration.as_secs_f64();

        if fps < 20.0 {
            "严重掉帧，可能原因：大量资源加载、复杂渲染、GC暂停".to_string()
        } else if fps < 30.0 {
            "明显掉帧，可能原因：复杂计算、大量绘制调用、内存分配".to_string()
        } else if fps < 50.0 {
            "轻微掉帧，可能原因：单帧计算量过大、同步操作".to_string()
        } else {
            "帧时间波动，可能原因：系统负载、后台任务".to_string()
        }
    }

    /// 获取平均值
    pub fn mean(&mut self) -> Duration {
        if !self.stats_computed {
            self.compute_stats();
        }
        self.mean
    }

    /// 获取标准差
    pub fn std_dev(&mut self) -> Duration {
        if !self.stats_computed {
            self.compute_stats();
        }
        self.std_dev
    }

    /// 获取百分位数
    pub fn percentiles(&mut self) -> [Duration; 4] {
        if !self.stats_computed {
            self.compute_stats();
        }
        self.percentiles
    }

    /// 获取异常帧列表
    pub fn outliers(&mut self) -> &[FrameOutlier] {
        if !self.stats_computed {
            self.compute_stats();
        }
        &self.outliers
    }

    /// 获取样本数量
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// 清空所有样本
    pub fn clear(&mut self) {
        self.samples.clear();
        self.outliers.clear();
        self.stats_computed = false;
    }
}

/// 帧时间分布图
pub struct FrameTimeDistribution {
    /// 直方图数据（每个bin的计数）
    bins: Vec<usize>,
    /// bin大小（持续时间）
    bin_size: Duration,
    /// 最小值
    min: Duration,
    /// 最大值
    max: Duration,
    /// bin数量
    bin_count: usize,
}

impl FrameTimeDistribution {
    /// 从分析器创建分布图
    ///
    /// # 参数
    /// - `analyzer`: 帧时间分析器
    /// - `bin_count`: bin数量
    pub fn from_analyzer(analyzer: &FrameTimeAnalyzer, bin_count: usize) -> Self {
        if analyzer.samples.is_empty() {
            return Self {
                bins: vec![0; bin_count],
                bin_size: Duration::ZERO,
                min: Duration::ZERO,
                max: Duration::ZERO,
                bin_count,
            };
        }

        let min = analyzer.samples.iter().copied().min().unwrap_or(Duration::ZERO);
        let max = analyzer.samples.iter().copied().max().unwrap_or(Duration::ZERO);
        let range = max.as_nanos() - min.as_nanos();
        let bin_size = if range > 0 && bin_count > 0 {
            Duration::from_nanos((range / bin_count as u128) as u64)
        } else {
            Duration::from_nanos(1)
        };

        let mut bins = vec![0; bin_count];

        for &duration in &analyzer.samples {
            let bin_index = if bin_size.as_nanos() > 0 {
                ((duration.as_nanos() - min.as_nanos()) / bin_size.as_nanos()) as usize
            } else {
                0
            };
            let bin_index = bin_index.min(bin_count - 1);
            bins[bin_index] += 1;
        }

        Self {
            bins,
            bin_size,
            min,
            max,
            bin_count,
        }
    }

    /// 生成直方图数据
    pub fn generate_histogram(&self) -> HistogramData {
        let mut data_points = Vec::new();

        for (i, &count) in self.bins.iter().enumerate() {
            let bin_offset_nanos = (self.bin_size.as_nanos() as u64) * (i as u64);
            let bin_start = self.min + Duration::from_nanos(bin_offset_nanos);
            let bin_end = bin_start + self.bin_size;

            data_points.push(HistogramBin {
                start: bin_start,
                end: bin_end,
                count,
            });
        }

        HistogramData {
            bins: data_points,
            total_samples: self.bins.iter().sum(),
        }
    }

    /// 生成ASCII分布图
    pub fn render_distribution(&self, _width: usize, height: usize) -> String {
        let histogram = self.generate_histogram();
        let max_count = histogram.bins.iter().map(|b| b.count).max().unwrap_or(1);

        let mut output = String::new();
        output.push_str("帧时间分布图:\n");
        output.push_str(&format!(
            "范围: {:.2}ms - {:.2}ms\n",
            self.min.as_secs_f64() * 1000.0,
            self.max.as_secs_f64() * 1000.0
        ));
        output.push_str(&format!(
            "Bin大小: {:.2}ms\n\n",
            self.bin_size.as_secs_f64() * 1000.0
        ));

        // 生成垂直直方图
        for row in (0..height).rev() {
            let threshold = (max_count as f64 * row as f64 / height as f64) as usize;

            for bin in &histogram.bins {
                if bin.count >= threshold {
                    output.push('█');
                } else {
                    output.push(' ');
                }
            }
            output.push('\n');
        }

        // 添加底部标签
        output.push_str(&"─".repeat(histogram.bins.len()));
        output.push('\n');

        output
    }

    /// 获取bin数据
    pub fn bins(&self) -> &[usize] {
        &self.bins
    }

    /// 获取bin大小
    pub fn bin_size(&self) -> Duration {
        self.bin_size
    }
}

/// 直方图数据
#[derive(Debug, Clone)]
pub struct HistogramData {
    /// bin列表
    pub bins: Vec<HistogramBin>,
    /// 总样本数
    pub total_samples: usize,
}

/// 直方图bin
#[derive(Debug, Clone)]
pub struct HistogramBin {
    /// bin起始时间
    pub start: Duration,
    /// bin结束时间
    pub end: Duration,
    /// 样本计数
    pub count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_time_analyzer() {
        let mut analyzer = FrameTimeAnalyzer::new(100);

        // 添加一些样本
        for i in 0..10 {
            analyzer.add_sample(Duration::from_millis(16 + i));
        }

        analyzer.compute_stats();
        assert!(analyzer.mean() > Duration::ZERO);
        assert!(analyzer.sample_count() == 10);
    }

    #[test]
    fn test_outlier_detection() {
        let mut analyzer = FrameTimeAnalyzer::new(100);

        // 添加正常样本
        for _ in 0..10 {
            analyzer.add_sample(Duration::from_millis(16));
        }

        // 添加异常样本
        analyzer.add_sample(Duration::from_millis(100));

        analyzer.compute_stats();
        let outliers = analyzer.outliers();
        assert!(!outliers.is_empty());
    }

    #[test]
    fn test_distribution() {
        let mut analyzer = FrameTimeAnalyzer::new(100);

        for i in 0..20 {
            analyzer.add_sample(Duration::from_millis(16 + (i % 5)));
        }

        let distribution = FrameTimeDistribution::from_analyzer(&analyzer, 10);
        let histogram = distribution.generate_histogram();

        assert_eq!(histogram.total_samples, 20);
        assert!(!histogram.bins.is_empty());
    }
}
