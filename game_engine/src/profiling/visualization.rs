//  性能数据可视化模块
// 
//  提供性能趋势分析、图表生成和数据导出功能。

use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::profiling::storage::*;
use crate::profiling::ProfilingResult;

// ============================================================================
// 图表数据结构
// ============================================================================

/// 图表类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    /// 折线图
    Line,
    /// 柱状图
    Bar,
    /// 散点图
    Scatter,
    /// 面积图
    Area,
    /// 热力图
    Heatmap,
    /// 饼图
    Pie,
}

impl ChartType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChartType::Line => "line",
            ChartType::Bar => "bar",
            ChartType::Scatter => "scatter",
            ChartType::Area => "area",
            ChartType::Heatmap => "heatmap",
            ChartType::Pie => "pie",
        }
    }
}

/// 图表数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    /// X轴值
    pub x: f64,
    /// Y轴值
    pub y: f64,
    /// 标签
    pub label: Option<String>,
    /// 颜色
    pub color: Option<String>,
}

/// 图表数据系列
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    /// 系列名称
    pub name: String,
    /// 数据点
    pub data: Vec<ChartDataPoint>,
    /// 颜色
    pub color: Option<String>,
    /// 线型
    pub line_type: Option<String>,
    /// 是否显示
    pub visible: bool,
}

/// 图表配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// 图表标题
    pub title: String,
    /// X轴标签
    pub x_axis_label: String,
    /// Y轴标签
    pub y_axis_label: String,
    /// 图表类型
    pub chart_type: ChartType,
    /// 宽度
    pub width: u32,
    /// 高度
    pub height: u32,
    /// 背景颜色
    pub background_color: String,
    /// 网格线颜色
    pub grid_color: String,
    /// 是否显示图例
    pub show_legend: bool,
    /// 是否显示工具提示
    pub show_tooltip: bool,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            title: "性能图表".to_string(),
            x_axis_label: "时间".to_string(),
            y_axis_label: "值".to_string(),
            chart_type: ChartType::Line,
            width: 800,
            height: 600,
            background_color: "#ffffff".to_string(),
            grid_color: "#e0e0e0".to_string(),
            show_legend: true,
            show_tooltip: true,
        }
    }
}

/// 图表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// 图表配置
    pub config: ChartConfig,
    /// 数据系列
    pub series: Vec<ChartSeries>,
}

// ============================================================================
// 趋势分析
// ============================================================================

/// 趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrendDirection {
    /// 上升
    Increasing,
    /// 下降
    Decreasing,
    /// 稳定
    Stable,
    /// 波动
    Fluctuating,
}

impl TrendDirection {
    pub fn as_str(&self) -> &'static str {
        match self {
            TrendDirection::Increasing => "上升",
            TrendDirection::Decreasing => "下降",
            TrendDirection::Stable => "稳定",
            TrendDirection::Fluctuating => "波动",
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            TrendDirection::Increasing => "#28a745",  // 绿色
            TrendDirection::Decreasing => "#dc3545",  // 红色
            TrendDirection::Stable => "#6c757d",    // 灰色
            TrendDirection::Fluctuating => "#ffc107", // 黄色
        }
    }
}

/// 趋势分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// 指标名称
    pub metric_name: String,
    /// 趋势方向
    pub direction: TrendDirection,
    /// 变化率 (%)
    pub change_rate: f64,
    /// 置信度
    pub confidence: f64,
    /// 分析时间段
    pub period: Duration,
    /// 数据点数量
    pub data_points: usize,
    /// 预测值
    pub prediction: Option<f64>,
    /// 异常点
    pub anomalies: Vec<AnomalyPoint>,
}

/// 异常点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyPoint {
    /// 时间戳
    pub timestamp: u64,
    /// 值
    pub value: f64,
    /// 异常分数
    pub anomaly_score: f64,
    /// 异常类型
    pub anomaly_type: AnomalyType,
}

/// 异常类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnomalyType {
    /// 峰值异常
    Peak,
    /// 谷值异常
    Valley,
    /// 突变异常
    Spike,
    /// 趋势变化
    TrendChange,
}

impl AnomalyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AnomalyType::Peak => "峰值",
            AnomalyType::Valley => "谷值",
            AnomalyType::Spike => "突变",
            AnomalyType::TrendChange => "趋势变化",
        }
    }
}

/// 趋势分析器
pub struct TrendAnalyzer {
    /// 分析窗口大小
    window_size: usize,
    /// 异常检测阈值
    anomaly_threshold: f64,
}

impl TrendAnalyzer {
    /// 创建新的趋势分析器
    pub fn new(window_size: usize, anomaly_threshold: f64) -> Self {
        Self {
            window_size,
            anomaly_threshold,
        }
    }

    /// 分析指标趋势
    pub fn analyze_trend(&self, data_points: &[DataPoint]) -> ProfilingResult<TrendAnalysis> {
        if data_points.len() < 3 {
            return Err(crate::profiling::ProfilingError::ProcessingError(
                "数据点不足，无法进行趋势分析".to_string(),
            ));
        }

        let metric_name = data_points[0].metric_name.clone();
        let start_time = data_points.first().unwrap().timestamp;
        let end_time = data_points.last().unwrap().timestamp;
        let period = Duration::from_millis(end_time - start_time);

        // 提取数值
        let values: Vec<f64> = data_points.iter().map(|p| p.value).collect();
        
        // 分析趋势方向
        let direction = self.calculate_trend_direction(&values);
        
        // 计算变化率
        let change_rate = self.calculate_change_rate(&values);
        
        // 计算置信度
        let confidence = self.calculate_confidence(&values);
        
        // 预测下一个值
        let prediction = self.predict_next_value(&values);
        
        // 检测异常点
        let anomalies = self.detect_anomalies(data_points);

        Ok(TrendAnalysis {
            metric_name,
            direction,
            change_rate,
            confidence,
            period,
            data_points: data_points.len(),
            prediction,
            anomalies,
        })
    }

    /// 计算趋势方向
    fn calculate_trend_direction(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        // 计算线性回归
        let n = values.len() as f64;
        let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = values.iter().enumerate()
            .map(|(i, &y)| i as f64 * y).sum();
        let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        
        // 计算方差
        let mean_y = sum_y / n;
        let variance: f64 = values.iter()
            .map(|y| (y - mean_y).powi(2))
            .sum::<f64>() / n;
        let std_dev = variance.sqrt();

        // 根据斜率和标准差判断趋势
        if std_dev < mean_y.abs() * 0.05 {
            TrendDirection::Stable
        } else if slope > mean_y.abs() * 0.1 {
            TrendDirection::Increasing
        } else if slope < -mean_y.abs() * 0.1 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Fluctuating
        }
    }

    /// 计算变化率
    fn calculate_change_rate(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let first = values.first().unwrap();
        let last = values.last().unwrap();
        
        if first.abs() < f64::EPSILON {
            return 0.0;
        }

        ((last - first) / first.abs()) * 100.0
    }

    /// 计算置信度
    fn calculate_confidence(&self, values: &[f64]) -> f64 {
        if values.len() < 3 {
            return 0.0;
        }

        // 计算相关系数
        let n = values.len() as f64;
        let indices: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();
        
        let mean_x = indices.iter().sum::<f64>() / n;
        let mean_y = values.iter().sum::<f64>() / n;
        
        let numerator: f64 = indices.iter().zip(values.iter())
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum();
        
        let sum_x2: f64 = indices.iter()
            .map(|x| (x - mean_x).powi(2))
            .sum();
        let sum_y2: f64 = values.iter()
            .map(|y| (y - mean_y).powi(2))
            .sum();
        
        let denominator = (sum_x2 * sum_y2).sqrt();
        
        if denominator < f64::EPSILON {
            return 0.0;
        }

        let correlation = numerator / denominator;
        correlation.abs().min(1.0)
    }

    /// 预测下一个值
    fn predict_next_value(&self, values: &[f64]) -> Option<f64> {
        if values.len() < 3 {
            return None;
        }

        // 使用简单移动平均预测
        let window_size = (values.len() / 3).min(5);
        if values.len() >= window_size {
            let recent_avg: f64 = values.iter()
                .rev()
                .take(window_size)
                .sum::<f64>() / window_size as f64;
            
            Some(recent_avg)
        } else {
            None
        }
    }

    /// 检测异常点
    fn detect_anomalies(&self, data_points: &[DataPoint]) -> Vec<AnomalyPoint> {
        let mut anomalies = Vec::new();
        
        if data_points.len() < 5 {
            return anomalies;
        }

        let values: Vec<f64> = data_points.iter().map(|p| p.value).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance: f64 = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        // 检测3σ外的点
        for (i, &value) in values.iter().enumerate() {
            let z_score = (value - mean) / std_dev;
            if z_score.abs() > self.anomaly_threshold {
                let anomaly_type = if z_score > 0.0 {
                    AnomalyType::Peak
                } else {
                    AnomalyType::Valley
                };

                anomalies.push(AnomalyPoint {
                    timestamp: data_points[i].timestamp,
                    value,
                    anomaly_score: z_score.abs(),
                    anomaly_type,
                });
            }
        }

        // 检测突变
        for i in 1..values.len() {
            let prev_value = values[i - 1];
            let curr_value = values[i];
            let change_rate = (curr_value - prev_value).abs() / prev_value.abs();
            
            if change_rate > 0.5 { // 50%变化
                anomalies.push(AnomalyPoint {
                    timestamp: data_points[i].timestamp,
                    value: curr_value,
                    anomaly_score: change_rate,
                    anomaly_type: AnomalyType::Spike,
                });
            }
        }

        anomalies
    }
}

// ============================================================================
// 数据导出
// ============================================================================

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// CSV格式
    Csv,
    /// JSON格式
    Json,
    /// XML格式
    Xml,
    /// Excel格式
    Excel,
}

impl ExportFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ExportFormat::Csv => ".csv",
            ExportFormat::Json => ".json",
            ExportFormat::Xml => ".xml",
            ExportFormat::Excel => ".xlsx",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            ExportFormat::Csv => "text/csv",
            ExportFormat::Json => "application/json",
            ExportFormat::Xml => "application/xml",
            ExportFormat::Excel => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        }
    }
}

/// 导出配置
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// 导出格式
    pub format: ExportFormat,
    /// 是否包含元数据
    pub include_metadata: bool,
    /// 是否压缩
    pub compress: bool,
    /// 时间范围开始
    pub start_time: Option<u64>,
    /// 时间范围结束
    pub end_time: Option<u64>,
    /// 指标过滤
    pub metric_filter: Option<Vec<String>>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            format: ExportFormat::Csv,
            include_metadata: true,
            compress: false,
            start_time: None,
            end_time: None,
            metric_filter: None,
        }
    }
}

/// 数据导出器
pub struct DataExporter {
    queryer: DataQueryer,
}

impl DataExporter {
    /// 创建新的数据导出器
    pub fn new(storage_dir: &Path, file_prefix: &str) -> Self {
        Self {
            queryer: DataQueryer::new(storage_dir, file_prefix),
        }
    }

    /// 导出数据
    pub fn export(&self, config: &ExportConfig, output_path: &Path) -> ProfilingResult<()> {
        // 构建查询条件
        let mut condition = QueryCondition {
            metric_names: config.metric_filter.clone(),
            categories: None,
            start_time: config.start_time,
            end_time: config.end_time,
            tags: None,
            limit: None,
            order_by: Some(QueryOrder::TimestampAsc),
        };

        // 执行查询
        let result = self.queryer.query(&condition)?;
        
        // 根据格式导出
        match config.format {
            ExportFormat::Csv => self.export_csv(&result.data_points, config, output_path),
            ExportFormat::Json => self.export_json(&result.data_points, config, output_path),
            ExportFormat::Xml => self.export_xml(&result.data_points, config, output_path),
            ExportFormat::Excel => self.export_excel(&result.data_points, config, output_path),
        }
    }

    /// 导出为CSV
    fn export_csv(&self, data_points: &[DataPoint], config: &ExportConfig, output_path: &Path) -> ProfilingResult<()> {
        let mut file = File::create(output_path)?;
        
        // 写入CSV头
        writeln!(file, "timestamp,metric_name,value,category")?;
        
        // 写入数据
        for point in data_points {
            writeln!(
                file,
                "{},{},{},{}",
                point.timestamp,
                point.metric_name,
                point.value,
                point.category as u8
            )?;
        }

        // 写入元数据
        if config.include_metadata {
            writeln!(file, "\n# Metadata")?;
            writeln!(file, "# Total Records: {}", data_points.len())?;
            writeln!(file, "# Export Time: {}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())?;
        }

        Ok(())
    }

    /// 导出为JSON
    fn export_json(&self, data_points: &[DataPoint], config: &ExportConfig, output_path: &Path) -> ProfilingResult<()> {
        use serde_json;
        
        let mut export_data = serde_json::json!({
            "data": data_points,
        });

        // 添加元数据
        if config.include_metadata {
            export_data["metadata"] = serde_json::json!({
                "total_records": data_points.len(),
                "export_time": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
                "format_version": "1.0"
            });
        }

        let json_string = serde_json::to_string_pretty(&export_data)?;
        let mut file = File::create(output_path)?;
        file.write_all(json_string.as_bytes())?;

        Ok(())
    }

    /// 导出为XML
    fn export_xml(&self, data_points: &[DataPoint], config: &ExportConfig, output_path: &Path) -> ProfilingResult<()> {
        let mut file = File::create(output_path)?;
        
        writeln!(file, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
        writeln!(file, "<performance_data>")?;
        
        // 写入数据
        writeln!(file, "  <data_points>")?;
        for point in data_points {
            writeln!(file, "    <data_point>")?;
            writeln!(file, "      <timestamp>{}</timestamp>", point.timestamp)?;
            writeln!(file, "      <metric_name>{}</metric_name>", point.metric_name)?;
            writeln!(file, "      <value>{}</value>", point.value)?;
            writeln!(file, "      <category>{}</category>", point.category as u8)?;
            writeln!(file, "    </data_point>")?;
        }
        writeln!(file, "  </data_points>")?;
        
        // 写入元数据
        if config.include_metadata {
            writeln!(file, "  <metadata>")?;
            writeln!(file, "    <total_records>{}</total_records>", data_points.len())?;
            writeln!(file, "    <export_time>{}</export_time>", 
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())?;
            writeln!(file, "  </metadata>")?;
        }
        
        writeln!(file, "</performance_data>")?;
        
        Ok(())
    }

    /// 导出为Excel（简化实现）
    fn export_excel(&self, data_points: &[DataPoint], config: &ExportConfig, output_path: &Path) -> ProfilingResult<()> {
        // 简化实现：导出为CSV格式，但使用.xlsx扩展名
        // 实际项目中应使用excelwriter或calamine库
        self.export_csv(data_points, config, output_path)
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_chart_type() {
        assert_eq!(ChartType::Line.as_str(), "line");
        assert_eq!(ChartType::Bar.as_str(), "bar");
        assert_eq!(ChartType::Scatter.as_str(), "scatter");
        assert_eq!(ChartType::Area.as_str(), "area");
        assert_eq!(ChartType::Heatmap.as_str(), "heatmap");
        assert_eq!(ChartType::Pie.as_str(), "pie");
    }

    #[test]
    fn test_trend_direction() {
        assert_eq!(TrendDirection::Increasing.as_str(), "上升");
        assert_eq!(TrendDirection::Decreasing.as_str(), "下降");
        assert_eq!(TrendDirection::Stable.as_str(), "稳定");
        assert_eq!(TrendDirection::Fluctuating.as_str(), "波动");
        
        assert_eq!(TrendDirection::Increasing.color(), "#28a745");
        assert_eq!(TrendDirection::Decreasing.color(), "#dc3545");
        assert_eq!(TrendDirection::Stable.color(), "#6c757d");
        assert_eq!(TrendDirection::Fluctuating.color(), "#ffc107");
    }

    #[test]
    fn test_anomaly_type() {
        assert_eq!(AnomalyType::Peak.as_str(), "峰值");
        assert_eq!(AnomalyType::Valley.as_str(), "谷值");
        assert_eq!(AnomalyType::Spike.as_str(), "突变");
        assert_eq!(AnomalyType::TrendChange.as_str(), "趋势变化");
    }

    #[test]
    fn test_export_format() {
        assert_eq!(ExportFormat::Csv.extension(), ".csv");
        assert_eq!(ExportFormat::Json.extension(), ".json");
        assert_eq!(ExportFormat::Xml.extension(), ".xml");
        assert_eq!(ExportFormat::Excel.extension(), ".xlsx");
        
        assert_eq!(ExportFormat::Csv.mime_type(), "text/csv");
        assert_eq!(ExportFormat::Json.mime_type(), "application/json");
        assert_eq!(ExportFormat::Xml.mime_type(), "application/xml");
        assert_eq!(ExportFormat::Excel.mime_type(), "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");
    }

    #[test]
    fn test_trend_analyzer() {
        let analyzer = TrendAnalyzer::new(10, 2.0);
        
        // 创建测试数据点
        let data_points = vec![
            DataPoint::new("test_metric", 10.0, crate::profiling::metrics::MetricCategory::Render),
            DataPoint::new("test_metric", 12.0, crate::profiling::metrics::MetricCategory::Render),
            DataPoint::new("test_metric", 15.0, crate::profiling::metrics::MetricCategory::Render),
            DataPoint::new("test_metric", 18.0, crate::profiling::metrics::MetricCategory::Render),
            DataPoint::new("test_metric", 20.0, crate::profiling::metrics::MetricCategory::Render),
        ];
        
        let analysis = analyzer.analyze_trend(&data_points).unwrap();
        
        assert_eq!(analysis.metric_name, "test_metric");
        assert_eq!(analysis.data_points, 5);
        assert!(analysis.change_rate > 0.0); // 上升趋势
        assert!(analysis.confidence > 0.0);
    }

    #[test]
    fn test_chart_data() {
        let series = ChartSeries {
            name: "测试系列".to_string(),
            data: vec![
                ChartDataPoint { x: 1.0, y: 10.0, label: None, color: None },
                ChartDataPoint { x: 2.0, y: 15.0, label: None, color: None },
                ChartDataPoint { x: 3.0, y: 12.0, label: None, color: None },
            ],
            color: Some("#ff0000".to_string()),
            line_type: Some("solid".to_string()),
            visible: true,
        };
        
        assert_eq!(series.name, "测试系列");
        assert_eq!(series.data.len(), 3);
        assert_eq!(series.data[0].x, 1.0);
        assert_eq!(series.data[0].y, 10.0);
        assert_eq!(series.color, Some("#ff0000".to_string()));
    }

    #[test]
    fn test_chart_config() {
        let config = ChartConfig::default();
        
        assert_eq!(config.title, "性能图表");
        assert_eq!(config.x_axis_label, "时间");
        assert_eq!(config.y_axis_label, "值");
        assert_eq!(config.chart_type, ChartType::Line);
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.background_color, "#ffffff");
        assert_eq!(config.grid_color, "#e0e0e0");
        assert!(config.show_legend);
        assert!(config.show_tooltip);
    }
}