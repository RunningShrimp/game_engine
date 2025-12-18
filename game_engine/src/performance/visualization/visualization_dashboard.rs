use std::collections::VecDeque;

/// 图表类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartType {
    #[default]
    LineChart,
    BarChart,
    PieChart,
    HistogramChart,
    HeatmapChart,
}

/// 单个数据点（用于图表）
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
    pub timestamp: u64,
    pub category: Option<String>,
}

impl DataPoint {
    pub fn new(label: impl Into<String>, value: f64, timestamp: u64) -> Self {
        Self {
            label: label.into(),
            value,
            timestamp,
            category: None,
        }
    }

    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// 简单图表组件
#[derive(Debug, Clone)]
pub struct Chart {
    pub name: String,
    pub chart_type: ChartType,
    pub data: VecDeque<DataPoint>,
    pub max_points: usize,
    pub y_min: Option<f64>,
    pub y_max: Option<f64>,
}

impl Default for Chart {
    fn default() -> Self {
        Self {
            name: String::new(),
            chart_type: ChartType::LineChart,
            data: VecDeque::new(),
            max_points: 300,
            y_min: None,
            y_max: None,
        }
    }
}

impl Chart {
    pub fn new(name: impl Into<String>, chart_type: ChartType) -> Self {
        Self {
            name: name.into(),
            chart_type,
            max_points: 300,
            ..Default::default()
        }
    }

    /// 添加数据点
    pub fn add_point(&mut self, point: DataPoint) {
        if self.data.len() >= self.max_points {
            self.data.pop_front();
        }
        self.data.push_back(point);
    }

    /// 添加简单数据点
    pub fn add_value(&mut self, label: impl Into<String>, value: f64, timestamp: u64) {
        self.add_point(DataPoint::new(label, value, timestamp));
    }

    /// 获取平均值
    pub fn get_average(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        let sum: f64 = self.data.iter().map(|p| p.value).sum();
        Some(sum / self.data.len() as f64)
    }

    /// 获取最大值
    pub fn get_max(&self) -> Option<f64> {
        self.data
            .iter()
            .map(|p| p.value)
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// 获取最小值
    pub fn get_min(&self) -> Option<f64> {
        self.data
            .iter()
            .map(|p| p.value)
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }

    /// 获取95分位数
    pub fn get_percentile_95(&self) -> Option<f64> {
        if self.data.is_empty() {
            return None;
        }

        let mut values: Vec<f64> = self.data.iter().map(|p| p.value).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = (values.len() as f64 * 0.95) as usize;
        values.get(index).copied()
    }

    /// 清空数据
    pub fn clear(&mut self) {
        self.data.clear();
    }

    /// 获取数据点总数
    pub fn point_count(&self) -> usize {
        self.data.len()
    }
}

/// 仪表板视图配置
#[derive(Debug, Clone)]
pub struct DashboardLayout {
    pub title: String,
    pub columns: usize,
    pub refresh_rate_ms: u64,
}

impl Default for DashboardLayout {
    fn default() -> Self {
        Self {
            title: String::new(),
            columns: 1,
            refresh_rate_ms: 33, // ~30 FPS
        }
    }
}

impl DashboardLayout {
    pub fn new(title: impl Into<String>, columns: usize) -> Self {
        Self {
            title: title.into(),
            columns,
            refresh_rate_ms: 33, // ~30 FPS
        }
    }

    pub fn with_refresh_rate(mut self, ms: u64) -> Self {
        self.refresh_rate_ms = ms;
        self
    }
}

/// 性能仪表板 - 可视化面板
pub struct VisualizationDashboard {
    layout: DashboardLayout,
    charts: Vec<Chart>,
    gauge_values: std::collections::HashMap<String, f64>,
    active: bool,
}

impl VisualizationDashboard {
    pub fn new(layout: DashboardLayout) -> Self {
        Self {
            layout,
            charts: Vec::new(),
            gauge_values: std::collections::HashMap::new(),
            active: true,
        }
    }

    /// 添加图表
    pub fn add_chart(&mut self, chart: Chart) {
        self.charts.push(chart);
    }

    /// 创建新图表并添加
    pub fn create_chart(&mut self, name: impl Into<String>, chart_type: ChartType) -> usize {
        let chart = Chart::new(name, chart_type);
        self.charts.push(chart);
        self.charts.len() - 1
    }

    /// 获取图表的可变引用
    pub fn get_chart_mut(&mut self, index: usize) -> Option<&mut Chart> {
        self.charts.get_mut(index)
    }

    /// 获取图表的不可变引用
    pub fn get_chart(&self, index: usize) -> Option<&Chart> {
        self.charts.get(index)
    }

    /// 添加仪表值
    pub fn set_gauge(&mut self, name: impl Into<String>, value: f64) {
        self.gauge_values.insert(name.into(), value);
    }

    /// 获取仪表值
    pub fn get_gauge(&self, name: &str) -> Option<f64> {
        self.gauge_values.get(name).copied()
    }

    /// 添加数据点到特定图表
    pub fn add_data_to_chart(
        &mut self,
        chart_index: usize,
        label: impl Into<String>,
        value: f64,
        timestamp: u64,
    ) -> Result<(), &'static str> {
        if let Some(chart) = self.charts.get_mut(chart_index) {
            chart.add_value(label, value, timestamp);
            Ok(())
        } else {
            Err("Chart index out of bounds")
        }
    }

    /// 获取仪表板统计信息
    pub fn get_summary(&self) -> DashboardSummary {
        let mut summary = DashboardSummary {
            total_charts: self.charts.len(),
            total_data_points: 0,
            gauge_count: self.gauge_values.len(),
            ..Default::default()
        };

        for (idx, chart) in self.charts.iter().enumerate() {
            summary.total_data_points += chart.point_count();
            summary.chart_stats.push(ChartStatistics {
                index: idx,
                name: chart.name.clone(),
                chart_type: chart.chart_type,
                point_count: chart.point_count(),
                average: chart.get_average(),
                max: chart.get_max(),
                min: chart.get_min(),
                percentile_95: chart.get_percentile_95(),
            });
        }

        summary
    }

    /// 渲染为ASCII表格（用于调试）
    pub fn render_ascii(&self) -> String {
        let mut output = format!("╔════════════════════════════════════════════════╗\n");
        output.push_str(&format!("║ {} │\n", self.layout.title));
        output.push_str(&format!(
            "╠════════════════════════════════════════════════╣\n"
        ));

        for (name, value) in &self.gauge_values {
            output.push_str(&format!("║ {:<30} │ {:>12.2} ║\n", name, value));
        }

        output.push_str(&format!(
            "╚════════════════════════════════════════════════╝\n"
        ));

        for chart in &self.charts {
            output.push_str(&format!(
                "\n[{}] ({} points)\n",
                chart.name,
                chart.point_count()
            ));

            if let (Some(min), Some(max)) = (chart.get_min(), chart.get_max()) {
                output.push_str(&format!("  Range: {:.2} - {:.2}\n", min, max));

                if let Some(avg) = chart.get_average() {
                    output.push_str(&format!("  Average: {:.2}\n", avg));
                }

                if let Some(p95) = chart.get_percentile_95() {
                    output.push_str(&format!("  P95: {:.2}\n", p95));
                }
            }
        }

        output
    }

    /// 启用仪表板
    pub fn enable(&mut self) {
        self.active = true;
    }

    /// 禁用仪表板
    pub fn disable(&mut self) {
        self.active = false;
    }

    /// 是否启用
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// 清空所有数据
    pub fn clear_all(&mut self) {
        for chart in &mut self.charts {
            chart.clear();
        }
        self.gauge_values.clear();
    }

    /// 获取图表总数
    pub fn chart_count(&self) -> usize {
        self.charts.len()
    }
}

/// 仪表板统计摘要
#[derive(Debug, Clone, Default)]
pub struct DashboardSummary {
    pub total_charts: usize,
    pub total_data_points: usize,
    pub chart_stats: Vec<ChartStatistics>,
    pub gauge_count: usize,
}

/// 单个图表的统计信息
#[derive(Debug, Clone, Default)]
pub struct ChartStatistics {
    pub index: usize,
    pub name: String,
    pub chart_type: ChartType,
    pub point_count: usize,
    pub average: Option<f64>,
    pub max: Option<f64>,
    pub min: Option<f64>,
    pub percentile_95: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_point() {
        let point = DataPoint::new("fps", 60.0, 1000);
        assert_eq!(point.label, "fps");
        assert_eq!(point.value, 60.0);

        let point_with_cat = point.clone().with_category("graphics");
        assert_eq!(point_with_cat.category, Some("graphics".to_string()));
    }

    #[test]
    fn test_chart_basic() {
        let mut chart = Chart::new("FPS", ChartType::LineChart);
        chart.add_value("frame_1", 60.0, 0);
        chart.add_value("frame_2", 61.0, 1);
        chart.add_value("frame_3", 59.0, 2);

        assert_eq!(chart.point_count(), 3);
        assert!(chart.get_average().unwrap() > 59.0 && chart.get_average().unwrap() < 61.0);
        assert_eq!(chart.get_max().unwrap(), 61.0);
        assert_eq!(chart.get_min().unwrap(), 59.0);
    }

    #[test]
    fn test_chart_percentile() {
        let mut chart = Chart::new("latency", ChartType::LineChart);
        for i in 0..100 {
            chart.add_value(format!("sample_{}", i), i as f64, i as u64);
        }

        let p95 = chart.get_percentile_95();
        assert!(p95.is_some());
        assert!(p95.unwrap() > 90.0 && p95.unwrap() <= 100.0);
    }

    #[test]
    fn test_chart_max_points() {
        let mut chart = Chart::new("test", ChartType::LineChart);
        chart.max_points = 10;

        for i in 0..20 {
            chart.add_value(format!("p_{}", i), i as f64, i as u64);
        }

        assert_eq!(chart.point_count(), 10);
    }

    #[test]
    fn test_dashboard_layout() {
        let layout = DashboardLayout::new("Performance Monitor", 2);
        assert_eq!(layout.title, "Performance Monitor");
        assert_eq!(layout.columns, 2);
        assert_eq!(layout.refresh_rate_ms, 33);

        let custom = layout.clone().with_refresh_rate(16);
        assert_eq!(custom.refresh_rate_ms, 16);
    }

    #[test]
    fn test_visualization_dashboard() {
        let layout = DashboardLayout::new("Dashboard", 3);
        let mut dashboard = VisualizationDashboard::new(layout);

        let idx1 = dashboard.create_chart("FPS", ChartType::LineChart);
        let idx2 = dashboard.create_chart("Memory", ChartType::LineChart);

        dashboard.set_gauge("current_fps", 60.5);
        dashboard.set_gauge("memory_usage_mb", 256.0);

        assert_eq!(dashboard.chart_count(), 2);
        assert_eq!(dashboard.get_gauge("current_fps"), Some(60.5));

        dashboard.add_data_to_chart(idx1, "f1", 60.0, 0).unwrap();
        dashboard.add_data_to_chart(idx2, "m1", 250.0, 0).unwrap();

        assert_eq!(dashboard.get_chart(idx1).unwrap().point_count(), 1);
    }

    #[test]
    fn test_dashboard_summary() {
        let layout = DashboardLayout::new("Dashboard", 2);
        let mut dashboard = VisualizationDashboard::new(layout);

        dashboard.create_chart("FPS", ChartType::LineChart);
        dashboard.create_chart("Memory", ChartType::LineChart);

        dashboard.set_gauge("uptime_s", 3600.0);

        let summary = dashboard.get_summary();
        assert_eq!(summary.total_charts, 2);
        assert_eq!(summary.gauge_count, 1);
    }

    #[test]
    fn test_dashboard_disable() {
        let layout = DashboardLayout::new("Dashboard", 1);
        let mut dashboard = VisualizationDashboard::new(layout);

        assert!(dashboard.is_active());
        dashboard.disable();
        assert!(!dashboard.is_active());
        dashboard.enable();
        assert!(dashboard.is_active());
    }

    #[test]
    fn test_dashboard_ascii_render() {
        let layout = DashboardLayout::new("Performance", 2);
        let mut dashboard = VisualizationDashboard::new(layout);

        dashboard.set_gauge("FPS", 60.0);
        dashboard.set_gauge("Latency (ms)", 16.67);

        let output = dashboard.render_ascii();
        assert!(output.contains("Performance"));
        assert!(output.contains("FPS"));
    }
}
