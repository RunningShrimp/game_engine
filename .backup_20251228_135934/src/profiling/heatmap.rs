//! 性能热力图
//!
//! 提供代码级性能热点分析和可视化功能。
//! 支持识别性能瓶颈、代码位置追踪和多种格式导出。

use std::collections::HashMap;
use std::time::Duration;

/// 代码位置
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Location {
    /// 文件路径
    pub file: String,
    /// 行号
    pub line: u32,
    /// 函数名
    pub function: String,
    /// 模块路径（可选）
    pub module: Option<String>,
}

impl Location {
    /// 创建新的代码位置
    pub fn new(file: impl Into<String>, line: u32, function: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            line,
            function: function.into(),
            module: None,
        }
    }

    /// 设置模块路径
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }

    /// 获取完整路径字符串
    pub fn full_path(&self) -> String {
        if let Some(module) = &self.module {
            format!("{}::{}:{}:{}", module, self.file, self.line, self.function)
        } else {
            format!("{}:{}:{}", self.file, self.line, self.function)
        }
    }
}

/// 热点性能指标
#[derive(Debug, Clone)]
pub struct HotspotMetrics {
    /// 总执行时间
    pub total_time: Duration,
    /// 调用次数
    pub call_count: usize,
    /// 平均执行时间
    pub avg_time: Duration,
    /// 最大执行时间
    pub max_time: Duration,
    /// 最小执行时间
    pub min_time: Duration,
    /// 自身时间（不包括子调用）
    pub self_time: Duration,
    /// 子调用时间
    pub children_time: Duration,
}

impl HotspotMetrics {
    /// 创建新的热点指标
    pub fn new(total_time: Duration, call_count: usize) -> Self {
        Self {
            total_time,
            call_count,
            avg_time: if call_count > 0 {
                Duration::from_nanos(total_time.as_nanos() as u64 / call_count as u64)
            } else {
                Duration::ZERO
            },
            max_time: total_time,
            min_time: total_time,
            self_time: total_time,
            children_time: Duration::ZERO,
        }
    }

    /// 记录一次调用
    pub fn record_call(&mut self, duration: Duration, self_time: Duration) {
        self.call_count += 1;
        self.total_time += duration;
        self.self_time += self_time;
        self.children_time = self.total_time - self.self_time;

        if duration > self.max_time {
            self.max_time = duration;
        }
        if duration < self.min_time || self.min_time == Duration::ZERO {
            self.min_time = duration;
        }

        self.avg_time = if self.call_count > 0 {
            Duration::from_nanos(self.total_time.as_nanos() as u64 / self.call_count as u64)
        } else {
            Duration::ZERO
        };
    }

    /// 合并另一个热点指标
    pub fn merge(&mut self, other: &HotspotMetrics) {
        self.call_count += other.call_count;
        self.total_time += other.total_time;
        self.self_time += other.self_time;
        self.children_time += other.children_time;

        if other.max_time > self.max_time {
            self.max_time = other.max_time;
        }
        if other.min_time < self.min_time || self.min_time == Duration::ZERO {
            self.min_time = other.min_time;
        }

        self.avg_time = if self.call_count > 0 {
            Duration::from_nanos(self.total_time.as_nanos() as u64 / self.call_count as u64)
        } else {
            Duration::ZERO
        };
    }

    /// 获取热点强度（0.0 - 1.0）
    pub fn intensity(&self, threshold: Duration) -> f64 {
        if threshold.as_nanos() == 0 {
            return 0.0;
        }
        (self.total_time.as_nanos() as f64 / threshold.as_nanos() as f64).min(1.0)
    }
}

/// 性能热力图
pub struct PerformanceHeatmap {
    /// 代码位置 -> 性能指标映射
    hotspots: HashMap<Location, HotspotMetrics>,
    /// 热点阈值
    threshold: Duration,
    /// 总采样时间
    total_sample_time: Duration,
}

impl PerformanceHeatmap {
    /// 创建新的性能热力图
    ///
    /// # 参数
    /// - `threshold`: 热点阈值，超过此时间的代码位置将被标记为热点
    pub fn new(threshold: Duration) -> Self {
        Self {
            hotspots: HashMap::new(),
            threshold,
            total_sample_time: Duration::ZERO,
        }
    }

    /// 记录代码位置的性能数据
    pub fn record(&mut self, location: Location, duration: Duration, self_time: Duration) {
        self.total_sample_time += duration;
        let entry = self
            .hotspots
            .entry(location)
            .or_insert_with(|| HotspotMetrics::new(Duration::ZERO, 0));
        entry.record_call(duration, self_time);
    }

    /// 合并另一个热力图
    pub fn merge(&mut self, other: &PerformanceHeatmap) {
        for (location, metrics) in &other.hotspots {
            if let Some(existing) = self.hotspots.get_mut(location) {
                existing.merge(metrics);
            } else {
                self.hotspots.insert(location.clone(), metrics.clone());
            }
        }
        self.total_sample_time += other.total_sample_time;
    }

    /// 获取所有热点（按总时间排序）
    pub fn get_hotspots(&self) -> Vec<(&Location, &HotspotMetrics)> {
        let mut hotspots: Vec<_> = self.hotspots.iter().collect();
        hotspots.sort_by(|(_, a), (_, b)| b.total_time.cmp(&a.total_time));
        hotspots
    }

    /// 获取超过阈值的热点
    pub fn get_critical_hotspots(&self) -> Vec<(&Location, &HotspotMetrics)> {
        self.get_hotspots()
            .into_iter()
            .filter(|(_, metrics)| metrics.total_time >= self.threshold)
            .collect()
    }

    /// 获取热点数量
    pub fn hotspot_count(&self) -> usize {
        self.hotspots.len()
    }

    /// 获取总采样时间
    pub fn total_sample_time(&self) -> Duration {
        self.total_sample_time
    }

    /// 设置阈值
    pub fn set_threshold(&mut self, threshold: Duration) {
        self.threshold = threshold;
    }

    /// 获取阈值
    pub fn threshold(&self) -> Duration {
        self.threshold
    }

    /// 清空所有数据
    pub fn clear(&mut self) {
        self.hotspots.clear();
        self.total_sample_time = Duration::ZERO;
    }

    /// 获取指定位置的热点指标
    pub fn get_metrics(&self, location: &Location) -> Option<&HotspotMetrics> {
        self.hotspots.get(location)
    }
}

/// 热力图可视化器trait
pub trait HeatmapVisualizer {
    /// 渲染热力图为字符串
    fn render(&self, heatmap: &PerformanceHeatmap) -> String;
}

/// 文本热力图可视化器
pub struct TextHeatmapVisualizer {
    /// 最大显示热点数
    max_hotspots: usize,
    /// 是否显示详细信息
    detailed: bool,
}

impl TextHeatmapVisualizer {
    /// 创建新的文本可视化器
    pub fn new(max_hotspots: usize) -> Self {
        Self {
            max_hotspots,
            detailed: false,
        }
    }

    /// 启用详细信息显示
    pub fn with_details(mut self) -> Self {
        self.detailed = true;
        self
    }
}

impl HeatmapVisualizer for TextHeatmapVisualizer {
    fn render(&self, heatmap: &PerformanceHeatmap) -> String {
        let mut output = String::new();
        output.push_str("=== 性能热力图 ===\n\n");
        output.push_str(&format!(
            "总采样时间: {:.2}ms\n",
            heatmap.total_sample_time().as_secs_f64() * 1000.0
        ));
        output.push_str(&format!("热点数量: {}\n", heatmap.hotspot_count()));
        output.push_str(&format!(
            "阈值: {:.2}ms\n\n",
            heatmap.threshold().as_secs_f64() * 1000.0
        ));

        let hotspots = heatmap.get_hotspots();
        let display_count = hotspots.len().min(self.max_hotspots);

        output.push_str("热点列表（按总时间排序）:\n");
        output.push_str("─".repeat(80).as_str());
        output.push('\n');

        for (i, (location, metrics)) in hotspots.iter().take(display_count).enumerate() {
            let intensity = metrics.intensity(heatmap.threshold());
            let intensity_bar = "█".repeat((intensity * 20.0) as usize);

            output.push_str(&format!(
                "{}. {} ({:.2}ms total, {} calls, {:.2}ms avg)\n",
                i + 1,
                location.full_path(),
                metrics.total_time.as_secs_f64() * 1000.0,
                metrics.call_count,
                metrics.avg_time.as_secs_f64() * 1000.0,
            ));
            output.push_str(&format!(
                "   [{}] {:.1}%\n",
                intensity_bar,
                intensity * 100.0
            ));

            if self.detailed {
                output.push_str(&format!(
                    "   - 自身时间: {:.2}ms\n",
                    metrics.self_time.as_secs_f64() * 1000.0
                ));
                output.push_str(&format!(
                    "   - 子调用时间: {:.2}ms\n",
                    metrics.children_time.as_secs_f64() * 1000.0
                ));
                output.push_str(&format!(
                    "   - 最小/最大: {:.2}ms / {:.2}ms\n",
                    metrics.min_time.as_secs_f64() * 1000.0,
                    metrics.max_time.as_secs_f64() * 1000.0
                ));
            }
            output.push('\n');
        }

        output
    }
}

/// HTML热力图可视化器
pub struct HtmlHeatmapVisualizer {
    /// 最大显示热点数
    max_hotspots: usize,
    /// 是否包含样式
    include_styles: bool,
}

impl HtmlHeatmapVisualizer {
    /// 创建新的HTML可视化器
    pub fn new(max_hotspots: usize) -> Self {
        Self {
            max_hotspots,
            include_styles: true,
        }
    }

    /// 禁用样式
    pub fn without_styles(mut self) -> Self {
        self.include_styles = false;
        self
    }
}

impl HeatmapVisualizer for HtmlHeatmapVisualizer {
    fn render(&self, heatmap: &PerformanceHeatmap) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<title>性能热力图</title>\n");

        if self.include_styles {
            html.push_str("<style>\n");
            html.push_str("body { font-family: monospace; margin: 20px; }\n");
            html.push_str("h1 { color: #333; }\n");
            html.push_str("table { border-collapse: collapse; width: 100%; margin-top: 20px; }\n");
            html.push_str("th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }\n");
            html.push_str("th { background-color: #f2f2f2; }\n");
            html.push_str("tr:nth-child(even) { background-color: #f9f9f9; }\n");
            html.push_str(".hotspot-bar { height: 20px; background: linear-gradient(to right, #4CAF50, #FFC107, #FF5722); }\n");
            html.push_str("</style>\n");
        }

        html.push_str("</head>\n<body>\n");
        html.push_str("<h1>性能热力图</h1>\n");
        html.push_str(&format!(
            "<p>总采样时间: {:.2}ms</p>\n",
            heatmap.total_sample_time().as_secs_f64() * 1000.0
        ));
        html.push_str(&format!("<p>热点数量: {}</p>\n", heatmap.hotspot_count()));
        html.push_str(&format!(
            "<p>阈值: {:.2}ms</p>\n",
            heatmap.threshold().as_secs_f64() * 1000.0
        ));

        html.push_str("<table>\n");
        html.push_str("<tr><th>排名</th><th>位置</th><th>总时间 (ms)</th><th>调用次数</th><th>平均时间 (ms)</th><th>自身时间 (ms)</th><th>强度</th></tr>\n");

        let hotspots = heatmap.get_hotspots();
        for (i, (location, metrics)) in hotspots.iter().take(self.max_hotspots).enumerate() {
            let intensity = metrics.intensity(heatmap.threshold());
            let intensity_percent = (intensity * 100.0) as u32;

            html.push_str("<tr>\n");
            html.push_str(&format!("<td>{}</td>\n", i + 1));
            html.push_str(&format!("<td>{}</td>\n", location.full_path()));
            html.push_str(&format!(
                "<td>{:.2}</td>\n",
                metrics.total_time.as_secs_f64() * 1000.0
            ));
            html.push_str(&format!("<td>{}</td>\n", metrics.call_count));
            html.push_str(&format!(
                "<td>{:.2}</td>\n",
                metrics.avg_time.as_secs_f64() * 1000.0
            ));
            html.push_str(&format!(
                "<td>{:.2}</td>\n",
                metrics.self_time.as_secs_f64() * 1000.0
            ));
            html.push_str(&format!(
                "<td><div class=\"hotspot-bar\" style=\"width: {}%\"></div> {}%</td>\n",
                intensity_percent, intensity_percent
            ));
            html.push_str("</tr>\n");
        }

        html.push_str("</table>\n");
        html.push_str("</body>\n</html>\n");

        html
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location() {
        let loc = Location::new("test.rs", 42, "test_function");
        assert_eq!(loc.file, "test.rs");
        assert_eq!(loc.line, 42);
        assert_eq!(loc.function, "test_function");
    }

    #[test]
    fn test_hotspot_metrics() {
        let mut metrics = HotspotMetrics::new(Duration::from_millis(100), 10);
        assert_eq!(metrics.call_count, 10);
        assert_eq!(metrics.avg_time, Duration::from_millis(10));

        metrics.record_call(Duration::from_millis(20), Duration::from_millis(15));
        assert_eq!(metrics.call_count, 11);
    }

    #[test]
    fn test_heatmap() {
        let mut heatmap = PerformanceHeatmap::new(Duration::from_millis(10));
        let location = Location::new("test.rs", 42, "test_function");

        heatmap.record(
            location.clone(),
            Duration::from_millis(50),
            Duration::from_millis(30),
        );
        assert_eq!(heatmap.hotspot_count(), 1);

        let metrics = heatmap.get_metrics(&location).unwrap();
        assert_eq!(metrics.call_count, 1);
        assert_eq!(metrics.total_time, Duration::from_millis(50));
    }

    #[test]
    fn test_text_visualizer() {
        let mut heatmap = PerformanceHeatmap::new(Duration::from_millis(10));
        let location = Location::new("test.rs", 42, "test_function");
        heatmap.record(
            location,
            Duration::from_millis(50),
            Duration::from_millis(30),
        );

        let visualizer = TextHeatmapVisualizer::new(10);
        let output = visualizer.render(&heatmap);
        assert!(output.contains("性能热力图"));
        assert!(output.contains("test.rs"));
    }
}
