//! 性能报告生成器
//!
//! 提供统一的性能报告生成功能，支持多种输出格式（JSON、HTML、Markdown、Text）。
//! 整合所有性能监控数据，生成详细的性能分析报告。

use std::collections::HashMap;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// 报告模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportTemplate {
    /// 纯文本格式
    Text,
    /// Markdown格式
    Markdown,
    /// HTML格式（带样式和图表）
    Html,
    /// JSON格式（机器可读）
    Json,
}

impl ReportTemplate {
    /// 从文件扩展名推断模板类型
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "md" | "markdown" => Self::Markdown,
            "html" | "htm" => Self::Html,
            "json" => Self::Json,
            _ => Self::Text,
        }
    }

    /// 获取默认文件扩展名
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Text => "txt",
            Self::Markdown => "md",
            Self::Html => "html",
            Self::Json => "json",
        }
    }
}

/// 报告摘要
#[derive(Debug, Clone)]
pub struct ReportSummary {
    /// 监控持续时间
    pub duration: Duration,
    /// 平均FPS
    pub avg_fps: f64,
    /// 最小FPS
    pub min_fps: f64,
    /// 最大FPS
    pub max_fps: f64,
    /// 平均帧时间（毫秒）
    pub avg_frame_time_ms: f64,
    /// 第99百分位帧时间（毫秒）
    pub p99_frame_time_ms: f64,
    /// 峰值内存使用（MB）
    pub peak_memory_mb: u64,
    /// 平均CPU使用率（%）
    pub avg_cpu_usage: f64,
    /// 平均GPU使用率（%）
    pub avg_gpu_usage: f64,
    /// 总绘制调用数
    pub total_draw_calls: u64,
    /// 总三角形数
    pub total_triangles: u64,
}

impl Default for ReportSummary {
    fn default() -> Self {
        Self {
            duration: Duration::ZERO,
            avg_fps: 0.0,
            min_fps: 0.0,
            max_fps: 0.0,
            avg_frame_time_ms: 0.0,
            p99_frame_time_ms: 0.0,
            peak_memory_mb: 0,
            avg_cpu_usage: 0.0,
            avg_gpu_usage: 0.0,
            total_draw_calls: 0,
            total_triangles: 0,
        }
    }
}

/// 指标详情
#[derive(Debug, Clone)]
pub struct MetricDetails {
    /// 指标名称
    pub name: String,
    /// 当前值
    pub current_value: f64,
    /// 平均值
    pub average_value: f64,
    /// 最小值
    pub min_value: f64,
    /// 最大值
    pub max_value: f64,
    /// 单位
    pub unit: String,
    /// 历史数据点
    pub data_points: Vec<DataPoint>,
}

/// 数据点
#[derive(Debug, Clone)]
pub struct DataPoint {
    /// 时间戳（相对于报告开始时间）
    pub timestamp: Duration,
    /// 值
    pub value: f64,
}

/// 图表类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChartType {
    /// 折线图
    Line,
    /// 柱状图
    Bar,
    /// 直方图
    Histogram,
    /// 散点图
    Scatter,
}

/// 可视化
#[derive(Debug, Clone)]
pub struct Visualization {
    /// 图表类型
    pub chart_type: ChartType,
    /// 数据点
    pub data: Vec<DataPoint>,
    /// 标题
    pub title: String,
    /// 说明
    pub caption: String,
    /// X轴标签
    pub x_label: String,
    /// Y轴标签
    pub y_label: String,
}

/// 优化建议
#[derive(Debug, Clone)]
pub struct Recommendation {
    /// 建议类型
    pub category: String,
    /// 优先级（1-5，5最高）
    pub priority: u8,
    /// 建议描述
    pub description: String,
    /// 预期改进
    pub expected_improvement: Option<String>,
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// 报告生成时间
    pub generated_at: SystemTime,
    /// 报告摘要
    pub summary: ReportSummary,
    /// 详细指标
    pub detailed_metrics: HashMap<String, MetricDetails>,
    /// 可视化
    pub visualizations: Vec<Visualization>,
    /// 优化建议
    pub recommendations: Vec<Recommendation>,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

impl PerformanceReport {
    /// 创建新的性能报告
    pub fn new(summary: ReportSummary) -> Self {
        Self {
            generated_at: SystemTime::now(),
            summary,
            detailed_metrics: HashMap::new(),
            visualizations: Vec::new(),
            recommendations: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// 添加指标详情
    pub fn add_metric(&mut self, metric: MetricDetails) {
        self.detailed_metrics.insert(metric.name.clone(), metric);
    }

    /// 添加可视化
    pub fn add_visualization(&mut self, viz: Visualization) {
        self.visualizations.push(viz);
    }

    /// 添加优化建议
    pub fn add_recommendation(&mut self, rec: Recommendation) {
        self.recommendations.push(rec);
    }

    /// 设置元数据
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }
}

/// 性能报告生成器
pub struct PerformanceReportGenerator {
    /// 报告模板
    template: ReportTemplate,
}

impl PerformanceReportGenerator {
    /// 创建新的报告生成器
    pub fn new(template: ReportTemplate) -> Self {
        Self { template }
    }

    /// 生成报告并保存到文件
    pub fn generate_to_file<P: AsRef<Path>>(
        &self,
        report: &PerformanceReport,
        path: P,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = match self.template {
            ReportTemplate::Text => self.generate_text(report),
            ReportTemplate::Markdown => self.generate_markdown(report),
            ReportTemplate::Html => self.generate_html(report),
            ReportTemplate::Json => self.generate_json(report)?,
        };

        std::fs::write(path, content)?;
        Ok(())
    }

    /// 生成文本格式报告
    pub fn generate_text(&self, report: &PerformanceReport) -> String {
        let mut output = String::new();

        output.push_str("=== 性能监控报告 ===\n\n");
        output.push_str(&format!(
            "生成时间: {}\n",
            format_timestamp(report.generated_at)
        ));
        output.push_str("\n");

        // 摘要
        output.push_str("--- 性能摘要 ---\n");
        output.push_str(&format!(
            "监控时长: {:.2}秒\n",
            report.summary.duration.as_secs_f64()
        ));
        output.push_str(&format!("平均FPS: {:.1}\n", report.summary.avg_fps));
        output.push_str(&format!(
            "FPS范围: {:.1} - {:.1}\n",
            report.summary.min_fps, report.summary.max_fps
        ));
        output.push_str(&format!(
            "平均帧时间: {:.2}ms\n",
            report.summary.avg_frame_time_ms
        ));
        output.push_str(&format!(
            "P99帧时间: {:.2}ms\n",
            report.summary.p99_frame_time_ms
        ));
        output.push_str(&format!("峰值内存: {}MB\n", report.summary.peak_memory_mb));
        output.push_str(&format!(
            "平均CPU使用率: {:.1}%\n",
            report.summary.avg_cpu_usage
        ));
        output.push_str(&format!(
            "平均GPU使用率: {:.1}%\n",
            report.summary.avg_gpu_usage
        ));
        output.push_str(&format!(
            "总绘制调用: {}\n",
            report.summary.total_draw_calls
        ));
        output.push_str(&format!("总三角形数: {}\n", report.summary.total_triangles));
        output.push_str("\n");

        // 详细指标
        if !report.detailed_metrics.is_empty() {
            output.push_str("--- 详细指标 ---\n");
            for (name, metric) in &report.detailed_metrics {
                output.push_str(&format!("\n{}:\n", name));
                output.push_str(&format!(
                    "  当前值: {:.2} {}\n",
                    metric.current_value, metric.unit
                ));
                output.push_str(&format!(
                    "  平均值: {:.2} {}\n",
                    metric.average_value, metric.unit
                ));
                output.push_str(&format!(
                    "  范围: {:.2} - {:.2} {}\n",
                    metric.min_value, metric.max_value, metric.unit
                ));
            }
            output.push_str("\n");
        }

        // 优化建议
        if !report.recommendations.is_empty() {
            output.push_str("--- 优化建议 ---\n");
            for (i, rec) in report.recommendations.iter().enumerate() {
                output.push_str(&format!(
                    "\n{}. [优先级: {}] {}\n",
                    i + 1,
                    rec.priority,
                    rec.category
                ));
                output.push_str(&format!("   {}\n", rec.description));
                if let Some(ref improvement) = rec.expected_improvement {
                    output.push_str(&format!("   预期改进: {}\n", improvement));
                }
            }
            output.push_str("\n");
        }

        output.push_str("====================\n");
        output
    }

    /// 生成Markdown格式报告
    pub fn generate_markdown(&self, report: &PerformanceReport) -> String {
        let mut output = String::new();

        output.push_str("# 性能监控报告\n\n");
        output.push_str(&format!(
            "**生成时间**: {}\n\n",
            format_timestamp(report.generated_at)
        ));

        // 摘要表格
        output.push_str("## 性能摘要\n\n");
        output.push_str("| 指标 | 值 |\n");
        output.push_str("|------|-----|\n");
        output.push_str(&format!(
            "| 监控时长 | {:.2}秒 |\n",
            report.summary.duration.as_secs_f64()
        ));
        output.push_str(&format!("| 平均FPS | {:.1} |\n", report.summary.avg_fps));
        output.push_str(&format!(
            "| FPS范围 | {:.1} - {:.1} |\n",
            report.summary.min_fps, report.summary.max_fps
        ));
        output.push_str(&format!(
            "| 平均帧时间 | {:.2}ms |\n",
            report.summary.avg_frame_time_ms
        ));
        output.push_str(&format!(
            "| P99帧时间 | {:.2}ms |\n",
            report.summary.p99_frame_time_ms
        ));
        output.push_str(&format!(
            "| 峰值内存 | {}MB |\n",
            report.summary.peak_memory_mb
        ));
        output.push_str(&format!(
            "| 平均CPU使用率 | {:.1}% |\n",
            report.summary.avg_cpu_usage
        ));
        output.push_str(&format!(
            "| 平均GPU使用率 | {:.1}% |\n",
            report.summary.avg_gpu_usage
        ));
        output.push_str(&format!(
            "| 总绘制调用 | {} |\n",
            report.summary.total_draw_calls
        ));
        output.push_str(&format!(
            "| 总三角形数 | {} |\n",
            report.summary.total_triangles
        ));
        output.push_str("\n");

        // 详细指标
        if !report.detailed_metrics.is_empty() {
            output.push_str("## 详细指标\n\n");
            for (name, metric) in &report.detailed_metrics {
                output.push_str(&format!("### {}\n\n", name));
                output.push_str("| 属性 | 值 |\n");
                output.push_str("|------|-----|\n");
                output.push_str(&format!(
                    "| 当前值 | {:.2} {} |\n",
                    metric.current_value, metric.unit
                ));
                output.push_str(&format!(
                    "| 平均值 | {:.2} {} |\n",
                    metric.average_value, metric.unit
                ));
                output.push_str(&format!(
                    "| 最小值 | {:.2} {} |\n",
                    metric.min_value, metric.unit
                ));
                output.push_str(&format!(
                    "| 最大值 | {:.2} {} |\n",
                    metric.max_value, metric.unit
                ));
                output.push_str("\n");
            }
        }

        // 优化建议
        if !report.recommendations.is_empty() {
            output.push_str("## 优化建议\n\n");
            for (i, rec) in report.recommendations.iter().enumerate() {
                output.push_str(&format!(
                    "### {}. {} (优先级: {})\n\n",
                    i + 1,
                    rec.category,
                    rec.priority
                ));
                output.push_str(&format!("{}\n\n", rec.description));
                if let Some(ref improvement) = rec.expected_improvement {
                    output.push_str(&format!("**预期改进**: {}\n\n", improvement));
                }
            }
        }

        output
    }

    /// 生成HTML格式报告
    pub fn generate_html(&self, report: &PerformanceReport) -> String {
        let mut output = String::new();

        output.push_str(r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>性能监控报告</title>
    <style>
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background-color: #f5f5f5;
        }
        .container {
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        h1 {
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 10px;
        }
        h2 {
            color: #34495e;
            margin-top: 30px;
            border-bottom: 2px solid #ecf0f1;
            padding-bottom: 5px;
        }
        table {
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }
        th, td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        th {
            background-color: #3498db;
            color: white;
            font-weight: 600;
        }
        tr:hover {
            background-color: #f5f5f5;
        }
        .metric-card {
            background: #f8f9fa;
            border-left: 4px solid #3498db;
            padding: 15px;
            margin: 15px 0;
            border-radius: 4px;
        }
        .recommendation {
            background: #fff3cd;
            border-left: 4px solid #ffc107;
            padding: 15px;
            margin: 15px 0;
            border-radius: 4px;
        }
        .priority-high {
            border-left-color: #dc3545;
        }
        .priority-medium {
            border-left-color: #ffc107;
        }
        .priority-low {
            border-left-color: #28a745;
        }
        .summary-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin: 20px 0;
        }
        .summary-item {
            background: #e8f4f8;
            padding: 15px;
            border-radius: 4px;
            text-align: center;
        }
        .summary-item .label {
            font-size: 0.9em;
            color: #666;
            margin-bottom: 5px;
        }
        .summary-item .value {
            font-size: 1.5em;
            font-weight: bold;
            color: #2c3e50;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>🎮 性能监控报告</h1>
        <p><strong>生成时间</strong>: "#);
        output.push_str(&format_timestamp(report.generated_at));
        output.push_str(
            r#"</p>

        <h2>性能摘要</h2>
        <div class="summary-grid">
"#,
        );

        // 摘要卡片
        let summary_items = vec![
            (
                "监控时长",
                format!("{:.2}秒", report.summary.duration.as_secs_f64()),
            ),
            ("平均FPS", format!("{:.1}", report.summary.avg_fps)),
            (
                "FPS范围",
                format!(
                    "{:.1} - {:.1}",
                    report.summary.min_fps, report.summary.max_fps
                ),
            ),
            (
                "平均帧时间",
                format!("{:.2}ms", report.summary.avg_frame_time_ms),
            ),
            (
                "P99帧时间",
                format!("{:.2}ms", report.summary.p99_frame_time_ms),
            ),
            ("峰值内存", format!("{}MB", report.summary.peak_memory_mb)),
            (
                "平均CPU使用率",
                format!("{:.1}%", report.summary.avg_cpu_usage),
            ),
            (
                "平均GPU使用率",
                format!("{:.1}%", report.summary.avg_gpu_usage),
            ),
        ];

        for (label, value) in summary_items {
            output.push_str(&format!(
                r#"            <div class="summary-item">
                <div class="label">{}</div>
                <div class="value">{}</div>
            </div>
"#,
                label, value
            ));
        }

        output.push_str(
            r#"        </div>

        <table>
            <tr>
                <th>指标</th>
                <th>值</th>
            </tr>
            <tr>
                <td>总绘制调用</td>
                <td>"#,
        );
        output.push_str(&format!("{}", report.summary.total_draw_calls));
        output.push_str(
            r#"</td>
            </tr>
            <tr>
                <td>总三角形数</td>
                <td>"#,
        );
        output.push_str(&format!("{}", report.summary.total_triangles));
        output.push_str(
            r#"</td>
            </tr>
        </table>
"#,
        );

        // 详细指标
        if !report.detailed_metrics.is_empty() {
            output.push_str(
                r#"        <h2>详细指标</h2>
"#,
            );
            for (name, metric) in &report.detailed_metrics {
                output.push_str(&format!(
                    r#"        <div class="metric-card">
            <h3>{}</h3>
            <table>
                <tr>
                    <th>属性</th>
                    <th>值</th>
                </tr>
                <tr>
                    <td>当前值</td>
                    <td>{:.2} {}</td>
                </tr>
                <tr>
                    <td>平均值</td>
                    <td>{:.2} {}</td>
                </tr>
                <tr>
                    <td>最小值</td>
                    <td>{:.2} {}</td>
                </tr>
                <tr>
                    <td>最大值</td>
                    <td>{:.2} {}</td>
                </tr>
            </table>
        </div>
"#,
                    name,
                    metric.current_value,
                    metric.unit,
                    metric.average_value,
                    metric.unit,
                    metric.min_value,
                    metric.unit,
                    metric.max_value,
                    metric.unit
                ));
            }
        }

        // 优化建议
        if !report.recommendations.is_empty() {
            output.push_str(
                r#"        <h2>优化建议</h2>
"#,
            );
            for (i, rec) in report.recommendations.iter().enumerate() {
                let priority_class = if rec.priority >= 4 {
                    "priority-high"
                } else if rec.priority >= 2 {
                    "priority-medium"
                } else {
                    "priority-low"
                };

                output.push_str(&format!(
                    r#"        <div class="recommendation {}">
            <h3>{}. {} (优先级: {})</h3>
            <p>{}</p>
"#,
                    priority_class,
                    i + 1,
                    rec.category,
                    rec.priority,
                    rec.description
                ));

                if let Some(ref improvement) = rec.expected_improvement {
                    output.push_str(&format!(
                        r#"            <p><strong>预期改进</strong>: {}</p>
"#,
                        improvement
                    ));
                }

                output.push_str("        </div>\n");
            }
        }

        output.push_str(
            r#"    </div>
</body>
</html>"#,
        );

        output
    }

    /// 生成JSON格式报告
    pub fn generate_json(&self, report: &PerformanceReport) -> Result<String, serde_json::Error> {
        #[derive(serde::Serialize)]
        struct JsonReport {
            generated_at: u64,
            summary: ReportSummary,
            detailed_metrics: HashMap<String, MetricDetails>,
            recommendations: Vec<Recommendation>,
            metadata: HashMap<String, String>,
        }

        let json_report = JsonReport {
            generated_at: report
                .generated_at
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            summary: report.summary.clone(),
            detailed_metrics: report.detailed_metrics.clone(),
            recommendations: report.recommendations.clone(),
            metadata: report.metadata.clone(),
        };

        serde_json::to_string_pretty(&json_report)
    }
}

// 为相关类型添加序列化支持
impl serde::Serialize for ReportSummary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ReportSummary", 11)?;
        state.serialize_field("duration_secs", &self.duration.as_secs_f64())?;
        state.serialize_field("avg_fps", &self.avg_fps)?;
        state.serialize_field("min_fps", &self.min_fps)?;
        state.serialize_field("max_fps", &self.max_fps)?;
        state.serialize_field("avg_frame_time_ms", &self.avg_frame_time_ms)?;
        state.serialize_field("p99_frame_time_ms", &self.p99_frame_time_ms)?;
        state.serialize_field("peak_memory_mb", &self.peak_memory_mb)?;
        state.serialize_field("avg_cpu_usage", &self.avg_cpu_usage)?;
        state.serialize_field("avg_gpu_usage", &self.avg_gpu_usage)?;
        state.serialize_field("total_draw_calls", &self.total_draw_calls)?;
        state.serialize_field("total_triangles", &self.total_triangles)?;
        state.end()
    }
}

impl serde::Serialize for MetricDetails {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("MetricDetails", 7)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("current_value", &self.current_value)?;
        state.serialize_field("average_value", &self.average_value)?;
        state.serialize_field("min_value", &self.min_value)?;
        state.serialize_field("max_value", &self.max_value)?;
        state.serialize_field("unit", &self.unit)?;
        state.serialize_field("data_points", &self.data_points)?;
        state.end()
    }
}

impl serde::Serialize for DataPoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("DataPoint", 2)?;
        state.serialize_field("timestamp_secs", &self.timestamp.as_secs_f64())?;
        state.serialize_field("value", &self.value)?;
        state.end()
    }
}

impl serde::Serialize for Recommendation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Recommendation", 4)?;
        state.serialize_field("category", &self.category)?;
        state.serialize_field("priority", &self.priority)?;
        state.serialize_field("description", &self.description)?;
        state.serialize_field("expected_improvement", &self.expected_improvement)?;
        state.end()
    }
}

/// 格式化时间戳
fn format_timestamp(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    // 简单的格式化（不使用外部依赖）
    // 这里使用RFC3339格式的简化版本
    let datetime = secs_to_datetime(secs);
    format!("{} UTC", datetime)
}

/// 将秒数转换为日期时间字符串（简化版本）
fn secs_to_datetime(secs: u64) -> String {
    // 这是一个简化的实现，实际项目中可以使用chrono或其他库
    // 这里返回一个基本的时间戳字符串
    let days = secs / 86400;
    let rem = secs % 86400;
    let hours = rem / 3600;
    let rem = rem % 3600;
    let minutes = rem / 60;
    let seconds = rem % 60;

    // 简化的日期计算（从1970-01-01开始）
    let year = 1970 + (days / 365);
    let day_of_year = (days % 365) + 1;
    let month = ((day_of_year - 1) / 30) + 1;
    let day = ((day_of_year - 1) % 30) + 1;

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        year, month, day, hours, minutes, seconds
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_report_generator() {
        let mut report = PerformanceReport::new(ReportSummary {
            duration: Duration::from_secs(60),
            avg_fps: 60.0,
            min_fps: 55.0,
            max_fps: 65.0,
            avg_frame_time_ms: 16.67,
            p99_frame_time_ms: 20.0,
            peak_memory_mb: 512,
            avg_cpu_usage: 50.0,
            avg_gpu_usage: 70.0,
            total_draw_calls: 1000,
            total_triangles: 100000,
        });

        report.add_recommendation(Recommendation {
            category: "渲染优化".to_string(),
            priority: 3,
            description: "减少绘制调用".to_string(),
            expected_improvement: Some("预期提升5-10% FPS".to_string()),
        });

        let generator = PerformanceReportGenerator::new(ReportTemplate::Text);
        let text = generator.generate_text(&report);
        assert!(text.contains("性能监控报告"));
        assert!(text.contains("60.0"));
    }

    #[test]
    fn test_template_from_extension() {
        assert_eq!(
            ReportTemplate::from_extension("md"),
            ReportTemplate::Markdown
        );
        assert_eq!(ReportTemplate::from_extension("html"), ReportTemplate::Html);
        assert_eq!(ReportTemplate::from_extension("json"), ReportTemplate::Json);
        assert_eq!(ReportTemplate::from_extension("txt"), ReportTemplate::Text);
    }
}
