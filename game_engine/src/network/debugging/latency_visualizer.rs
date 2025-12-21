//  延迟可视化模块
// 
//  实现网络延迟数据的可视化功能，包括热图、趋势图和客户端-服务器延迟分析。
// 
//  ## 功能特性
// 
//  - 延迟热图和趋势图
//  - 客户端-服务器延迟可视化
//  - 预测准确性可视化
//  - 网络事件时间线
//  - 延迟统计分析
//  - 自适应可视化更新

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};
use rand;

/// 延迟可视化器
pub struct LatencyVisualizer {
    /// 是否启用
    enabled: bool,
    /// 延迟数据历史
    latency_history: VecDeque<LatencyDataPoint>,
    /// 最大历史长度
    max_history_size: usize,
    /// 可视化配置
    config: VisualizationConfig,
    /// 热图生成器
    heatmap_generator: HeatmapGenerator,
    /// 趋势图生成器
    trend_generator: TrendGenerator,
    /// 客户端-服务器分析器
    client_server_analyzer: ClientServerAnalyzer,
    /// 预测准确性分析器
    prediction_accuracy_analyzer: PredictionAccuracyAnalyzer,
    /// 事件时间线
    event_timeline: EventTimeline,
    /// 统计分析器
    statistics_analyzer: StatisticsAnalyzer,
    /// 最后更新时间
    last_update: Instant,
}

/// 延迟数据点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyDataPoint {
    /// 时间戳
    pub timestamp_ms: u64,
    /// 客户端ID
    pub client_id: Option<u64>,
    /// 服务器ID
    pub server_id: Option<u64>,
    /// 延迟值（毫秒）
    pub latency_ms: f32,
    /// 延迟类型
    pub latency_type: LatencyType,
    /// 预测延迟（如果有）
    pub predicted_latency_ms: Option<f32>,
    /// 预测误差
    pub prediction_error: Option<f32>,
    /// 网络状况
    pub network_condition: NetworkCondition,
    /// 事件标记
    pub event_markers: Vec<String>,
}

/// 延迟类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LatencyType {
    /// 单向延迟
    OneWay,
    /// 往返延迟（RTT）
    RoundTrip,
    /// 服务器处理延迟
    ServerProcessing,
    /// 客户端处理延迟
    ClientProcessing,
    /// 网络传输延迟
    NetworkTransmission,
    /// 队列延迟
    Queue,
    /// 未知类型
    Unknown,
}

/// 网络状况
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCondition {
    /// 带宽利用率（0-1）
    pub bandwidth_utilization: f32,
    /// 丢包率（0-1）
    pub packet_loss_rate: f32,
    /// 网络拥塞程度（0-1）
    pub congestion_level: f32,
    /// 网络质量评分（0-100）
    pub quality_score: f32,
}

/// 可视化配置
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    /// 更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 历史数据保留时间（秒）
    pub history_retention_s: u64,
    /// 热图分辨率
    pub heatmap_resolution: (u32, u32),
    /// 热图颜色方案
    pub heatmap_color_scheme: ColorScheme,
    /// 趋势图采样率
    pub trend_sampling_rate: f32,
    /// 是否启用实时更新
    pub enable_realtime_updates: bool,
    /// 是否启用预测分析
    pub enable_prediction_analysis: bool,
    /// 是否启用事件标记
    pub enable_event_markers: bool,
}

/// 颜色方案
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorScheme {
    /// 彩虹色
    Rainbow,
    /// 热力色
    Heat,
    /// 蓝绿色
    BlueGreen,
    /// 灰度
    Grayscale,
    /// 自定义
    Custom,
}

/// 热图生成器
#[derive(Debug)]
struct HeatmapGenerator {
    /// 热图数据
    heatmap_data: Vec<Vec<f32>>,
    /// 分辨率
    resolution: (u32, u32),
    /// 颜色方案
    color_scheme: ColorScheme,
    /// 最小延迟值
    min_latency: f32,
    /// 最大延迟值
    max_latency: f32,
    /// 自动缩放
    #[allow(dead_code)]
    auto_scale: bool,
}

/// 趋势图生成器
#[derive(Debug)]
struct TrendGenerator {
    /// 趋势数据点
    trend_data: VecDeque<TrendDataPoint>,
    /// 最大数据点数
    max_data_points: usize,
    /// 采样间隔（毫秒）
    sampling_interval_ms: u64,
    /// 平滑窗口大小
    #[allow(dead_code)]
    smoothing_window: usize,
}

/// 趋势数据点
#[derive(Debug, Clone)]
pub struct TrendDataPoint {
    /// 时间戳
    timestamp: Instant,
    /// 平均延迟
    average_latency: f32,
    /// 最小延迟
    min_latency: f32,
    /// 最大延迟
    max_latency: f32,
    /// 延迟标准差
    latency_std_dev: f32,
    /// 数据点数量
    sample_count: u32,
}

/// 客户端-服务器分析器
#[derive(Debug)]
struct ClientServerAnalyzer {
    /// 客户端延迟数据
    client_latency_data: HashMap<u64, VecDeque<LatencyDataPoint>>,
    /// 服务器延迟数据
    server_latency_data: HashMap<u64, VecDeque<LatencyDataPoint>>,
    /// 客户端-服务器延迟对
    client_server_pairs: HashMap<(u64, u64), VecDeque<LatencyPair>>,
    /// 最大历史长度
    max_history_length: usize,
}

/// 延迟对
#[derive(Debug, Clone)]
struct LatencyPair {
    /// 客户端延迟
    client_latency: LatencyDataPoint,
    /// 服务器延迟
    server_latency: LatencyDataPoint,
    /// 延迟差
    latency_difference: f32,
    /// 相关性
    #[allow(dead_code)]
    correlation: f32,
}

/// 预测准确性分析器
#[derive(Debug)]
struct PredictionAccuracyAnalyzer {
    /// 预测数据
    prediction_data: VecDeque<PredictionDataPoint>,
    /// 最大历史长度
    max_history_length: usize,
    /// 准确性统计
    accuracy_statistics: AccuracyStatistics,
}

/// 预测数据点
#[derive(Debug, Clone)]
pub struct PredictionDataPoint {
    /// 时间戳
    #[allow(dead_code)]
    timestamp: Instant,
    /// 实际延迟
    #[allow(dead_code)]
    actual_latency: f32,
    /// 预测延迟
    #[allow(dead_code)]
    predicted_latency: f32,
    /// 预测误差
    #[allow(dead_code)]
    prediction_error: f32,
    /// 预测算法
    #[allow(dead_code)]
    prediction_algorithm: String,
    /// 置信度
    #[allow(dead_code)]
    confidence: f32,
}

/// 准确性统计
#[derive(Debug, Clone, Default)]
pub struct AccuracyStatistics {
    /// 总预测数
    total_predictions: u64,
    /// 平均绝对误差
    mean_absolute_error: f32,
    /// 均方根误差
    root_mean_square_error: f32,
    /// 平均绝对百分比误差
    mean_absolute_percentage_error: f32,
    /// 最大误差
    max_error: f32,
    /// 最小误差
    min_error: f32,
    /// 准确率（误差在阈值内的比例）
    accuracy_rate: f32,
    /// 置信度校准
    #[allow(dead_code)]
    confidence_calibration: f32,
}

/// 事件时间线
#[derive(Debug)]
struct EventTimeline {
    /// 事件列表
    events: VecDeque<NetworkEvent>,
    /// 最大事件数
    max_events: usize,
    /// 事件分类
    event_categories: HashMap<String, Vec<String>>,
}

/// 网络事件
#[derive(Debug, Clone)]
pub struct NetworkEvent {
    /// 事件ID
    #[allow(dead_code)]
    event_id: u64,
    /// 时间戳
    #[allow(dead_code)]
    timestamp: Instant,
    /// 事件类型
    event_type: EventType,
    /// 事件描述
    description: String,
    /// 严重程度
    severity: EventSeverity,
    /// 相关数据
    #[allow(dead_code)]
    related_data: HashMap<String, String>,
    /// 延迟影响
    #[allow(dead_code)]
    latency_impact: Option<LatencyImpact>,
}

/// 事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    /// 连接事件
    Connection,
    /// 断开连接事件
    Disconnection,
    /// 网络拥塞
    Congestion,
    /// 丢包事件
    PacketLoss,
    /// 延迟峰值
    LatencySpike,
    /// 网络恢复
    Recovery,
    /// 配置更改
    ConfigurationChange,
    /// 自定义事件
    Custom,
}

/// 事件严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重错误
    Critical,
}

/// 延迟影响
#[derive(Debug, Clone)]
struct LatencyImpact {
    /// 影响开始时间
    #[allow(dead_code)]
    start_time: Instant,
    /// 影响结束时间
    #[allow(dead_code)]
    end_time: Option<Instant>,
    /// 延迟增加量
    #[allow(dead_code)]
    latency_increase: f32,
    /// 影响的客户端
    #[allow(dead_code)]
    affected_clients: Vec<u64>,
    /// 影响持续时间
    #[allow(dead_code)]
    duration: Option<Duration>,
}

/// 统计分析器
#[derive(Debug)]
struct StatisticsAnalyzer {
    /// 延迟分布
    latency_distribution: LatencyDistribution,
    /// 百分位数
    percentiles: Percentiles,
    /// 趋势分析
    trend_analysis: TrendAnalysis,
    /// 异常检测
    anomaly_detector: AnomalyDetector,
}

/// 延迟分布
#[derive(Debug, Clone)]
pub struct LatencyDistribution {
    /// 分布区间
    #[allow(dead_code)]
    bins: Vec<LatencyBin>,
    /// 总样本数
    #[allow(dead_code)]
    total_samples: u64,
    /// 平均值
    #[allow(dead_code)]
    mean: f32,
    /// 方差
    #[allow(dead_code)]
    variance: f32,
    /// 偏度
    #[allow(dead_code)]
    skewness: f32,
    /// 峰度
    #[allow(dead_code)]
    kurtosis: f32,
}

/// 延迟区间
#[derive(Debug, Clone)]
pub struct LatencyBin {
    /// 最小值
    #[allow(dead_code)]
    min_value: f32,
    /// 最大值
    #[allow(dead_code)]
    max_value: f32,
    /// 计数
    #[allow(dead_code)]
    count: u64,
    /// 频率
    #[allow(dead_code)]
    frequency: f32,
}

/// 百分位数
#[derive(Debug, Clone)]
pub struct Percentiles {
    /// 50%分位数（中位数）
    #[allow(dead_code)]
    p50: f32,
    /// 75%分位数
    #[allow(dead_code)]
    p75: f32,
    /// 90%分位数
    #[allow(dead_code)]
    p90: f32,
    /// 95%分位数
    #[allow(dead_code)]
    p95: f32,
    /// 99%分位数
    #[allow(dead_code)]
    p99: f32,
}

/// 趋势分析
#[derive(Debug, Clone)]
pub struct TrendAnalysis {
    /// 趋势方向
    trend_direction: TrendDirection,
    /// 趋势强度
    trend_strength: f32,
    /// 变化率
    #[allow(dead_code)]
    change_rate: f32,
    /// 预测值
    #[allow(dead_code)]
    predicted_value: Option<f32>,
    /// 置信区间
    #[allow(dead_code)]
    confidence_interval: Option<(f32, f32)>,
}

/// 趋势方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendDirection {
    /// 上升
    Increasing,
    /// 下降
    Decreasing,
    /// 稳定
    Stable,
    /// 未知
    Unknown,
}

/// 异常检测器
#[derive(Debug)]
struct AnomalyDetector {
    /// 检测方法
    #[allow(dead_code)]
    detection_method: AnomalyDetectionMethod,
    /// 敏感度
    #[allow(dead_code)]
    sensitivity: f32,
    /// 历史窗口大小
    #[allow(dead_code)]
    history_window: usize,
    /// 检测到的异常
    detected_anomalies: VecDeque<LatencyAnomaly>,
}

/// 异常检测方法
#[derive(Debug, Clone, Copy)]
enum AnomalyDetectionMethod {
    /// 统计方法
    #[allow(dead_code)]
    Statistical,
    /// 机器学习方法
    #[allow(dead_code)]
    MachineLearning,
    /// 阈值方法
    #[allow(dead_code)]
    Threshold,
    /// 混合方法
    #[allow(dead_code)]
    Hybrid,
}

/// 延迟异常
#[derive(Debug, Clone)]
pub struct LatencyAnomaly {
    /// 异常ID
    #[allow(dead_code)]
    anomaly_id: u64,
    /// 时间戳
    #[allow(dead_code)]
    timestamp: Instant,
    /// 异常值
    #[allow(dead_code)]
    anomaly_value: f32,
    /// 预期值
    #[allow(dead_code)]
    expected_value: f32,
    /// 异常分数
    #[allow(dead_code)]
    anomaly_score: f32,
    /// 异常类型
    #[allow(dead_code)]
    anomaly_type: AnomalyType,
    /// 描述
    #[allow(dead_code)]
    description: String,
}

/// 异常类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyType {
    /// 峰值异常
    Spike,
    /// 谷值异常
    Dip,
    /// 模式变化
    PatternChange,
    /// 趋势变化
    TrendChange,
    /// 未知异常
    Unknown,
}

impl LatencyVisualizer {
    /// 创建新的延迟可视化器
    pub fn new() -> Self {
        Self::with_config(VisualizationConfig::default())
    }

    /// 创建带配置的延迟可视化器
    pub fn with_config(config: VisualizationConfig) -> Self {
        let max_history_size = (config.history_retention_s * 1000 / config.update_interval_ms) as usize;

        Self {
            enabled: true,
            latency_history: VecDeque::with_capacity(max_history_size),
            max_history_size,
            config: config.clone(),
            heatmap_generator: HeatmapGenerator::new(config.heatmap_resolution, config.heatmap_color_scheme),
            trend_generator: TrendGenerator::new(max_history_size, config.update_interval_ms),
            client_server_analyzer: ClientServerAnalyzer::new(max_history_size),
            prediction_accuracy_analyzer: PredictionAccuracyAnalyzer::new(max_history_size),
            event_timeline: EventTimeline::new(1000),
            statistics_analyzer: StatisticsAnalyzer::new(),
            last_update: Instant::now(),
        }
    }

    /// 添加延迟数据点
    pub fn add_latency_data(&mut self, data_point: LatencyDataPoint) {
        if !self.enabled {
            return;
        }

        // 添加到历史
        self.latency_history.push_back(data_point.clone());

        // 限制历史长度
        while self.latency_history.len() > self.max_history_size {
            self.latency_history.pop_front();
        }

        // 更新各个分析器
        self.update_analyzers(&data_point);

        // 检查是否需要更新可视化
        if self.config.enable_realtime_updates {
            let now = Instant::now();
            if now.duration_since(self.last_update).as_millis() >= self.config.update_interval_ms as u128 {
                self.update_visualizations();
                self.last_update = now;
            }
        }
    }

    /// 生成热图
    pub fn generate_heatmap(&self) -> HeatmapData {
        self.heatmap_generator.generate(&self.latency_history)
    }

    /// 生成趋势图
    pub fn generate_trend_chart(&self) -> TrendChartData {
        self.trend_generator.generate()
    }

    /// 生成客户端-服务器延迟分析
    pub fn generate_client_server_analysis(&self) -> ClientServerAnalysisData {
        self.client_server_analyzer.generate_analysis()
    }

    /// 生成预测准确性分析
    pub fn generate_prediction_accuracy_analysis(&self) -> PredictionAccuracyData {
        self.prediction_accuracy_analyzer.generate_analysis()
    }

    /// 生成事件时间线
    pub fn generate_event_timeline(&self) -> EventTimelineData {
        self.event_timeline.generate_timeline()
    }

    /// 生成统计分析
    pub fn generate_statistical_analysis(&self) -> StatisticalAnalysisData {
        self.statistics_analyzer.generate_analysis(&self.latency_history)
    }

    /// 添加网络事件
    pub fn add_network_event(&mut self, event: NetworkEvent) {
        if !self.enabled || !self.config.enable_event_markers {
            return;
        }

        self.event_timeline.add_event(event);
    }

    /// 获取延迟历史
    pub fn get_latency_history(&self) -> Vec<LatencyDataPoint> {
        self.latency_history.iter().cloned().collect()
    }

    /// 获取实时延迟统计
    pub fn get_realtime_statistics(&self) -> RealtimeStatistics {
        if self.latency_history.is_empty() {
            return RealtimeStatistics::default();
        }

        let recent_data: Vec<_> = self.latency_history.iter().rev().take(100).collect();
        let latencies: Vec<f32> = recent_data.iter().map(|d| d.latency_ms).collect();

        let average = latencies.iter().sum::<f32>() / latencies.len() as f32;
        let min = latencies.iter().fold(f32::MAX, |a, &b| a.min(b));
        let max = latencies.iter().fold(0.0_f32, |a, &b| a.max(b));

        // 计算标准差
        let variance = latencies.iter()
            .map(|&latency| {
                let diff = latency - average;
                diff * diff
            })
            .sum::<f32>() / latencies.len() as f32;
        let std_dev = variance.sqrt();

        RealtimeStatistics {
            current_latency: recent_data.first().map(|d| d.latency_ms).unwrap_or(0.0),
            average_latency: average,
            min_latency: min,
            max_latency: max,
            standard_deviation: std_dev,
            sample_count: latencies.len() as u32,
            last_update: self.last_update,
        }
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 设置启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 更新可视化器
    pub fn update(&mut self, _delta_time: Duration) {
        if !self.enabled {
            return;
        }

        // 定期更新可视化
        if self.config.enable_realtime_updates {
            let now = Instant::now();
            if now.duration_since(self.last_update).as_millis() >= self.config.update_interval_ms as u128 {
                self.update_visualizations();
                self.last_update = now;
            }
        }
    }

    /// 重置可视化器
    pub fn reset(&mut self) {
        self.latency_history.clear();
        self.heatmap_generator.reset();
        self.trend_generator.reset();
        self.client_server_analyzer.reset();
        self.prediction_accuracy_analyzer.reset();
        self.event_timeline.reset();
        self.statistics_analyzer.reset();
        self.last_update = Instant::now();
    }

    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        self.enabled && !self.latency_history.is_empty()
    }

    // 私有方法

    /// 更新分析器
    fn update_analyzers(&mut self, data_point: &LatencyDataPoint) {
        // 更新热图生成器
        self.heatmap_generator.add_data_point(data_point);

        // 更新趋势生成器
        self.trend_generator.add_data_point(data_point);

        // 更新客户端-服务器分析器
        self.client_server_analyzer.add_data_point(data_point);

        // 更新预测准确性分析器
        if let (Some(predicted_latency), Some(prediction_error)) = 
            (data_point.predicted_latency_ms, data_point.prediction_error) {
            self.prediction_accuracy_analyzer.add_prediction_data(
                data_point.latency_ms,
                predicted_latency,
                prediction_error,
            );
        }

        // 更新统计分析器
        self.statistics_analyzer.add_data_point(data_point);
    }

    /// 更新可视化
    fn update_visualizations(&mut self) {
        // 这里可以触发可视化数据的重新计算
        // 实际实现中可能需要与渲染系统集成
    }
}

/// 热图数据
#[derive(Debug, Clone)]
pub struct HeatmapData {
    /// 热图数据矩阵
    pub data: Vec<Vec<f32>>,
    /// 分辨率
    pub resolution: (u32, u32),
    /// 颜色映射
    pub color_mapping: Vec<(f32, [u8; 3])>,
    /// 最小值
    pub min_value: f32,
    /// 最大值
    pub max_value: f32,
}

/// 趋势图数据
#[derive(Debug, Clone)]
pub struct TrendChartData {
    /// 数据点
    pub data_points: Vec<TrendDataPoint>,
    /// 平均延迟线
    pub average_line: Vec<(f64, f32)>,
    /// 置信区间
    pub confidence_interval: Option<Vec<(f64, f32, f32)>>,
    /// 趋势线
    pub trend_line: Option<Vec<(f64, f32)>>,
}

/// 客户端-服务器分析数据
#[derive(Debug, Clone)]
pub struct ClientServerAnalysisData {
    /// 客户端延迟分布
    pub client_latency_distribution: HashMap<u64, Vec<LatencyDataPoint>>,
    /// 服务器延迟分布
    pub server_latency_distribution: HashMap<u64, Vec<LatencyDataPoint>>,
    /// 延迟相关性
    pub latency_correlations: HashMap<(u64, u64), f32>,
    /// 延迟差分布
    pub latency_differences: Vec<f32>,
}

/// 预测准确性数据
#[derive(Debug, Clone)]
pub struct PredictionAccuracyData {
    /// 准确性统计
    pub accuracy_statistics: AccuracyStatistics,
    /// 预测误差历史
    pub prediction_errors: Vec<PredictionDataPoint>,
    /// 准确性趋势
    pub accuracy_trend: Vec<(f64, f32)>,
}

/// 事件时间线数据
#[derive(Debug, Clone)]
pub struct EventTimelineData {
    /// 事件列表
    pub events: Vec<NetworkEvent>,
    /// 事件类型分布
    pub event_type_distribution: HashMap<EventType, u64>,
    /// 严重程度分布
    pub severity_distribution: HashMap<EventSeverity, u64>,
}

/// 统计分析数据
#[derive(Debug, Clone)]
pub struct StatisticalAnalysisData {
    /// 延迟分布
    pub latency_distribution: LatencyDistribution,
    /// 百分位数
    pub percentiles: Percentiles,
    /// 趋势分析
    pub trend_analysis: TrendAnalysis,
    /// 检测到的异常
    pub detected_anomalies: Vec<LatencyAnomaly>,
}

/// 实时统计
#[derive(Debug, Clone)]
pub struct RealtimeStatistics {
    /// 当前延迟
    pub current_latency: f32,
    /// 平均延迟
    pub average_latency: f32,
    /// 最小延迟
    pub min_latency: f32,
    /// 最大延迟
    pub max_latency: f32,
    /// 标准差
    pub standard_deviation: f32,
    /// 样本数量
    pub sample_count: u32,
    /// 最后更新时间
    pub last_update: Instant,
}

impl Default for RealtimeStatistics {
    fn default() -> Self {
        Self {
            current_latency: 0.0,
            average_latency: 0.0,
            min_latency: 0.0,
            max_latency: 0.0,
            standard_deviation: 0.0,
            sample_count: 0,
            last_update: std::time::Instant::now(),
        }
    }
}

// 热图生成器实现
impl HeatmapGenerator {
    fn new(resolution: (u32, u32), color_scheme: ColorScheme) -> Self {
        Self {
            heatmap_data: vec![vec![0.0; resolution.1 as usize]; resolution.0 as usize],
            resolution,
            color_scheme,
            min_latency: f32::MAX,
            max_latency: f32::MIN,
            auto_scale: true,
        }
    }

    fn add_data_point(&mut self, data_point: &LatencyDataPoint) {
        // 更新最小/最大值
        self.min_latency = self.min_latency.min(data_point.latency_ms);
        self.max_latency = self.max_latency.max(data_point.latency_ms);

        // 将数据点映射到热图网格
        let (x, y) = self.map_to_grid(data_point);
        if x < self.resolution.0 as usize && y < self.resolution.1 as usize {
            self.heatmap_data[x][y] = data_point.latency_ms;
        }
    }

    fn generate(&self, latency_history: &VecDeque<LatencyDataPoint>) -> HeatmapData {
        let mut data = self.heatmap_data.clone();

        // 如果历史数据中有更多点，可以用来填充热图
        for data_point in latency_history {
            let (x, y) = self.map_to_grid(data_point);
            if x < self.resolution.0 as usize && y < self.resolution.1 as usize {
                // 使用平均值或最大值
                data[x][y] = data[x][y].max(data_point.latency_ms);
            }
        }

        // 生成颜色映射
        let color_mapping = self.generate_color_mapping();

        HeatmapData {
            data,
            resolution: self.resolution,
            color_mapping,
            min_value: self.min_latency,
            max_value: self.max_latency,
        }
    }

    fn map_to_grid(&self, data_point: &LatencyDataPoint) -> (usize, usize) {
        // 简单的映射：基于时间和客户端ID
        let time_factor = (data_point.timestamp_ms % 86400000) as f32 / 86400000.0; // 一天内的比例
        let client_factor = data_point.client_id.unwrap_or(0) as f32 / 1000.0; // 简化的客户端ID映射

        let x = (time_factor * self.resolution.0 as f32) as usize;
        let y = (client_factor * self.resolution.1 as f32) as usize % self.resolution.1 as usize;

        (x, y)
    }

    fn generate_color_mapping(&self) -> Vec<(f32, [u8; 3])> {
        let mut mapping = Vec::new();
        let range = self.max_latency - self.min_latency;

        if range == 0.0 {
            return mapping;
        }

        match self.color_scheme {
            ColorScheme::Heat => {
                // 热力色：蓝 -> 绿 -> 黄 -> 红
                for i in 0..=100 {
                    let t = i as f32 / 100.0;
                    let value = self.min_latency + t * range;
                    let color = if t < 0.33 {
                        // 蓝到绿
                        let local_t = t / 0.33;
                        [0, (local_t * 255.0) as u8, (255.0 - local_t * 255.0) as u8]
                    } else if t < 0.67 {
                        // 绿到黄
                        let local_t = (t - 0.33) / 0.34;
                        [(local_t * 255.0) as u8, 255, 0]
                    } else {
                        // 黄到红
                        let local_t = (t - 0.67) / 0.33;
                        [255, (255.0 - local_t * 255.0) as u8, 0]
                    };
                    mapping.push((value, color));
                }
            }
            ColorScheme::Rainbow => {
                // 彩虹色
                for i in 0..=100 {
                    let t = i as f32 / 100.0;
                    let value = self.min_latency + t * range;
                    let hue = t * 270.0; // 从蓝到红
                    let color = self.hsv_to_rgb(hue, 1.0, 1.0);
                    mapping.push((value, color));
                }
            }
            ColorScheme::BlueGreen => {
                // 蓝绿色
                for i in 0..=100 {
                    let t = i as f32 / 100.0;
                    let value = self.min_latency + t * range;
                    let color = [0, (t * 255.0) as u8, ((1.0 - t) * 255.0) as u8];
                    mapping.push((value, color));
                }
            }
            ColorScheme::Grayscale => {
                // 灰度
                for i in 0..=100 {
                    let t = i as f32 / 100.0;
                    let value = self.min_latency + t * range;
                    let gray = (t * 255.0) as u8;
                    mapping.push((value, [gray, gray, gray]));
                }
            }
            ColorScheme::Custom => {
                // 自定义颜色方案
                // 这里可以实现自定义颜色映射逻辑
            }
        }

        mapping
    }

    fn hsv_to_rgb(&self, h: f32, s: f32, v: f32) -> [u8; 3] {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;

        let (r_prime, g_prime, b_prime) = if h < 60.0 {
            (c, x, 0.0)
        } else if h < 120.0 {
            (x, c, 0.0)
        } else if h < 180.0 {
            (0.0, c, x)
        } else if h < 240.0 {
            (0.0, x, c)
        } else if h < 300.0 {
            (x, 0.0, c)
        } else {
            (c, 0.0, x)
        };

        [
            ((r_prime + m) * 255.0) as u8,
            ((g_prime + m) * 255.0) as u8,
            ((b_prime + m) * 255.0) as u8,
        ]
    }

    fn reset(&mut self) {
        self.heatmap_data = vec![vec![0.0; self.resolution.1 as usize]; self.resolution.0 as usize];
        self.min_latency = f32::MAX;
        self.max_latency = f32::MIN;
    }
}

// 趋势图生成器实现
impl TrendGenerator {
    fn new(max_data_points: usize, sampling_interval_ms: u64) -> Self {
        Self {
            trend_data: VecDeque::with_capacity(max_data_points),
            max_data_points,
            sampling_interval_ms,
            smoothing_window: 10,
        }
    }

    fn add_data_point(&mut self, data_point: &LatencyDataPoint) {
        let timestamp = Instant::now();
        
        // 检查是否需要创建新的趋势数据点
        if let Some(last_point) = self.trend_data.back() {
            let elapsed = timestamp.duration_since(last_point.timestamp).as_millis() as u64;
            if elapsed < self.sampling_interval_ms {
                // 更新最后一个数据点
                self.update_last_data_point(data_point);
                return;
            }
        }

        // 创建新的趋势数据点
        let trend_point = TrendDataPoint {
            timestamp,
            average_latency: data_point.latency_ms,
            min_latency: data_point.latency_ms,
            max_latency: data_point.latency_ms,
            latency_std_dev: 0.0,
            sample_count: 1,
        };

        self.trend_data.push_back(trend_point);

        // 限制数据点数量
        while self.trend_data.len() > self.max_data_points {
            self.trend_data.pop_front();
        }
    }

    fn update_last_data_point(&mut self, data_point: &LatencyDataPoint) {
        if let Some(last_point) = self.trend_data.back_mut() {
            last_point.sample_count += 1;
            
            // 更新平均值
            let n = last_point.sample_count as f32;
            last_point.average_latency = (last_point.average_latency * (n - 1.0) + data_point.latency_ms) / n;
            
            // 更新最小/最大值
            last_point.min_latency = last_point.min_latency.min(data_point.latency_ms);
            last_point.max_latency = last_point.max_latency.max(data_point.latency_ms);
        }
    }

    fn generate(&self) -> TrendChartData {
        let data_points: Vec<_> = self.trend_data.iter().cloned().collect();
        
        // 生成平均延迟线
        let average_line: Vec<_> = data_points.iter()
            .map(|point| {
                let timestamp = point.timestamp.duration_since(Instant::now()).as_secs_f64();
                (timestamp, point.average_latency)
            })
            .collect();

        // 计算置信区间
        let confidence_interval = if data_points.len() > 2 {
            Some(self.calculate_confidence_interval(&data_points))
        } else {
            None
        };

        // 计算趋势线
        let trend_line = if data_points.len() > 2 {
            Some(self.calculate_trend_line(&data_points))
        } else {
            None
        };

        TrendChartData {
            data_points,
            average_line,
            confidence_interval,
            trend_line,
        }
    }

    fn calculate_confidence_interval(&self, data_points: &[TrendDataPoint]) -> Vec<(f64, f32, f32)> {
        data_points.iter()
            .map(|point| {
                let timestamp = point.timestamp.duration_since(Instant::now()).as_secs_f64();
                let margin = point.latency_std_dev * 1.96; // 95% 置信区间
                (timestamp, point.average_latency - margin, point.average_latency + margin)
            })
            .collect()
    }

    fn calculate_trend_line(&self, data_points: &[TrendDataPoint]) -> Vec<(f64, f32)> {
        if data_points.len() < 2 {
            return Vec::new();
        }

        // 简单线性回归
        let n = data_points.len() as f32;
        let sum_x: f32 = (0..data_points.len()).map(|i| i as f32).sum();
        let sum_y: f32 = data_points.iter().map(|p| p.average_latency).sum();
        let sum_xy: f32 = data_points.iter().enumerate()
            .map(|(i, p)| i as f32 * p.average_latency)
            .sum();
        let sum_x2: f32 = (0..data_points.len()).map(|i| (i as f32).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;

        data_points.iter().enumerate()
            .map(|(i, point)| {
                let timestamp = point.timestamp.duration_since(Instant::now()).as_secs_f64();
                let trend_value = slope * i as f32 + intercept;
                (timestamp, trend_value)
            })
            .collect()
    }

    fn reset(&mut self) {
        self.trend_data.clear();
    }
}

// 客户端-服务器分析器实现
impl ClientServerAnalyzer {
    fn new(max_history_length: usize) -> Self {
        Self {
            client_latency_data: HashMap::new(),
            server_latency_data: HashMap::new(),
            client_server_pairs: HashMap::new(),
            max_history_length,
        }
    }

    fn add_data_point(&mut self, data_point: &LatencyDataPoint) {
        // 添加到客户端或服务器数据
        if let Some(client_id) = data_point.client_id {
            let client_data = self.client_latency_data.entry(client_id).or_insert_with(VecDeque::new);
            client_data.push_back(data_point.clone());
            
            // 限制历史长度
            while client_data.len() > self.max_history_length {
                client_data.pop_front();
            }
        }

        if let Some(server_id) = data_point.server_id {
            let server_data = self.server_latency_data.entry(server_id).or_insert_with(VecDeque::new);
            server_data.push_back(data_point.clone());
            
            // 限制历史长度
            while server_data.len() > self.max_history_length {
                server_data.pop_front();
            }
        }

        // 如果同时有客户端和服务器ID，创建延迟对
        if let (Some(client_id), Some(server_id)) = (data_point.client_id, data_point.server_id) {
            let pair_key = (client_id, server_id);
            
            // 先查找对应的延迟数据，避免可变 borrow
            let corresponding_latency = self.find_corresponding_latency(data_point, client_id, server_id);
            
            if let Some(corresponding) = corresponding_latency {
                let latency_pair = LatencyPair {
                    client_latency: data_point.clone(),
                    server_latency: corresponding.clone(),
                    latency_difference: (data_point.latency_ms - corresponding.latency_ms).abs(),
                    correlation: 0.0, // 需要更多数据来计算相关性
                };
                
                // 现在再获取可变 borrow 来更新
                let pair_data = self.client_server_pairs.entry(pair_key).or_insert_with(VecDeque::new);
                pair_data.push_back(latency_pair);
                
                // 限制历史长度
                while pair_data.len() > self.max_history_length {
                    pair_data.pop_front();
                }
            }
        }
    }

    fn find_corresponding_latency(&self, data_point: &LatencyDataPoint, client_id: u64, server_id: u64) -> Option<LatencyDataPoint> {
        // 简化实现：查找时间最接近的对应延迟
        let time_threshold_ms = 1000; // 1秒内的数据认为是相关的
        
        if let Some(client_data) = self.client_latency_data.get(&client_id) {
            for point in client_data.iter().rev() {
                if (point.timestamp_ms as i64 - data_point.timestamp_ms as i64).abs() < time_threshold_ms {
                    return Some(point.clone());
                }
            }
        }

        if let Some(server_data) = self.server_latency_data.get(&server_id) {
            for point in server_data.iter().rev() {
                if (point.timestamp_ms as i64 - data_point.timestamp_ms as i64).abs() < time_threshold_ms {
                    return Some(point.clone());
                }
            }
        }

        None
    }

    fn generate_analysis(&self) -> ClientServerAnalysisData {
        let client_latency_distribution = self.client_latency_data.iter()
            .map(|(&id, data)| (id, data.iter().cloned().collect()))
            .collect();
        
        let server_latency_distribution = self.server_latency_data.iter()
            .map(|(&id, data)| (id, data.iter().cloned().collect()))
            .collect();

        let mut latency_correlations = HashMap::new();
        let mut latency_differences = Vec::new();

        for (&pair_key, pair_data) in &self.client_server_pairs {
            if !pair_data.is_empty() {
                // 计算相关性（简化实现）
                let correlation = self.calculate_correlation(pair_data);
                latency_correlations.insert(pair_key, correlation);

                // 收集延迟差
                for pair in pair_data {
                    latency_differences.push(pair.latency_difference);
                }
            }
        }

        ClientServerAnalysisData {
            client_latency_distribution,
            server_latency_distribution,
            latency_correlations,
            latency_differences,
        }
    }

    fn calculate_correlation(&self, pair_data: &VecDeque<LatencyPair>) -> f32 {
        if pair_data.len() < 2 {
            return 0.0;
        }

        // 简化的相关性计算
        let n = pair_data.len() as f32;
        let sum_x: f32 = pair_data.iter().map(|p| p.client_latency.latency_ms).sum();
        let sum_y: f32 = pair_data.iter().map(|p| p.server_latency.latency_ms).sum();
        let sum_xy: f32 = pair_data.iter()
            .map(|p| p.client_latency.latency_ms * p.server_latency.latency_ms)
            .sum();
        let sum_x2: f32 = pair_data.iter().map(|p| p.client_latency.latency_ms.powi(2)).sum();
        let sum_y2: f32 = pair_data.iter().map(|p| p.server_latency.latency_ms.powi(2)).sum();

        let numerator = n * sum_xy - sum_x * sum_y;
        let denominator = ((n * sum_x2 - sum_x.powi(2)) * (n * sum_y2 - sum_y.powi(2))).sqrt();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    fn reset(&mut self) {
        self.client_latency_data.clear();
        self.server_latency_data.clear();
        self.client_server_pairs.clear();
    }
}

// 预测准确性分析器实现
impl PredictionAccuracyAnalyzer {
    fn new(max_history_length: usize) -> Self {
        Self {
            prediction_data: VecDeque::with_capacity(max_history_length),
            max_history_length,
            accuracy_statistics: AccuracyStatistics::default(),
        }
    }

    fn add_prediction_data(&mut self, actual_latency: f32, predicted_latency: f32, prediction_error: f32) {
        let prediction_point = PredictionDataPoint {
            timestamp: Instant::now(),
            actual_latency,
            predicted_latency,
            prediction_error,
            prediction_algorithm: "Unknown".to_string(), // 可以从外部传入
            confidence: 0.5, // 可以从外部传入
        };

        self.prediction_data.push_back(prediction_point);

        // 限制历史长度
        while self.prediction_data.len() > self.max_history_length {
            self.prediction_data.pop_front();
        }

        // 更新统计信息
        self.update_accuracy_statistics();
    }

    fn update_accuracy_statistics(&mut self) {
        if self.prediction_data.is_empty() {
            return;
        }

        let n = self.prediction_data.len() as f32;
        let errors: Vec<f32> = self.prediction_data.iter().map(|p| p.prediction_error).collect();
        
        // 平均绝对误差
        self.accuracy_statistics.mean_absolute_error = errors.iter().sum::<f32>() / n;
        
        // 均方根误差
        self.accuracy_statistics.root_mean_square_error = (errors.iter().map(|e| e * e).sum::<f32>() / n).sqrt();
        
        // 平均绝对百分比误差
        self.accuracy_statistics.mean_absolute_percentage_error = 
            self.prediction_data.iter()
                .map(|p| (p.prediction_error / p.actual_latency).abs() * 100.0)
                .sum::<f32>() / n;
        
        // 最大/最小误差
        self.accuracy_statistics.max_error = errors.iter().fold(0.0, |a, &b| a.max(b));
        self.accuracy_statistics.min_error = errors.iter().fold(f32::MAX, |a, &b| a.min(b));
        
        // 准确率（误差在10%内的比例）
        let accurate_count = self.prediction_data.iter()
            .filter(|p| (p.prediction_error / p.actual_latency).abs() < 0.1)
            .count() as f32;
        self.accuracy_statistics.accuracy_rate = accurate_count / n;
        
        self.accuracy_statistics.total_predictions = self.prediction_data.len() as u64;
    }

    fn generate_analysis(&self) -> PredictionAccuracyData {
        let prediction_errors: Vec<_> = self.prediction_data.iter().cloned().collect();
        
        // 生成准确性趋势
        let accuracy_trend: Vec<_> = self.prediction_data.iter()
            .enumerate()
            .map(|(_i, p)| {
                let timestamp = p.timestamp.duration_since(Instant::now()).as_secs_f64();
                let accuracy = if p.actual_latency > 0.0 {
                    1.0 - (p.prediction_error / p.actual_latency).abs()
                } else {
                    0.0
                };
                (timestamp, accuracy)
            })
            .collect();

        PredictionAccuracyData {
            accuracy_statistics: self.accuracy_statistics.clone(),
            prediction_errors,
            accuracy_trend,
        }
    }

    fn reset(&mut self) {
        self.prediction_data.clear();
        self.accuracy_statistics = AccuracyStatistics::default();
    }
}

// 事件时间线实现
impl EventTimeline {
    fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events),
            max_events,
            event_categories: HashMap::new(),
        }
    }

    fn add_event(&mut self, event: NetworkEvent) {
        self.events.push_back(event.clone());

        // 限制事件数量
        while self.events.len() > self.max_events {
            self.events.pop_front();
        }

        // 更新事件分类
        let category = self.event_category_for_type(&event.event_type);
        self.event_categories.entry(category).or_insert_with(Vec::new).push(event.description.clone());
    }

    fn generate_timeline(&self) -> EventTimelineData {
        let events: Vec<_> = self.events.iter().cloned().collect();
        
        let mut event_type_distribution = HashMap::new();
        let mut severity_distribution = HashMap::new();
        
        for event in &events {
            *event_type_distribution.entry(event.event_type).or_insert(0) += 1;
            *severity_distribution.entry(event.severity).or_insert(0) += 1;
        }

        EventTimelineData {
            events,
            event_type_distribution,
            severity_distribution,
        }
    }

    fn event_category_for_type(&self, event_type: &EventType) -> String {
        match event_type {
            EventType::Connection => "连接事件".to_string(),
            EventType::Disconnection => "断开连接".to_string(),
            EventType::Congestion => "网络拥塞".to_string(),
            EventType::PacketLoss => "丢包事件".to_string(),
            EventType::LatencySpike => "延迟峰值".to_string(),
            EventType::Recovery => "网络恢复".to_string(),
            EventType::ConfigurationChange => "配置更改".to_string(),
            EventType::Custom => "自定义事件".to_string(),
        }
    }

    fn reset(&mut self) {
        self.events.clear();
        self.event_categories.clear();
    }
}

// 统计分析器实现
impl StatisticsAnalyzer {
    fn new() -> Self {
        Self {
            latency_distribution: LatencyDistribution::default(),
            percentiles: Percentiles::default(),
            trend_analysis: TrendAnalysis::default(),
            anomaly_detector: AnomalyDetector::new(),
        }
    }

    fn add_data_point(&mut self, data_point: &LatencyDataPoint) {
        // 更新延迟分布
        self.update_latency_distribution(data_point.latency_ms);
        
        // 更新百分位数
        self.update_percentiles();
        
        // 更新趋势分析
        self.update_trend_analysis();
        
        // 异常检测
        self.anomaly_detector.detect_anomaly(data_point.latency_ms);
    }

    fn update_latency_distribution(&mut self, latency: f32) {
        // 简化实现：更新基本统计信息
        self.latency_distribution.total_samples += 1;
        
        // 更新平均值
        let n = self.latency_distribution.total_samples as f32;
        self.latency_distribution.mean = 
            (self.latency_distribution.mean * (n - 1.0) + latency) / n;
        
        // 更新方差
        // 这里需要维护更多的历史数据来准确计算方差
    }

    fn update_percentiles(&mut self) {
        // 简化实现：需要收集所有延迟数据来计算百分位数
        // 实际实现中应该维护一个排序的数据结构
    }

    fn update_trend_analysis(&mut self) {
        // 简化实现：需要更多历史数据来分析趋势
        self.trend_analysis.trend_direction = TrendDirection::Unknown;
        self.trend_analysis.trend_strength = 0.0;
    }

    fn generate_analysis(&self, latency_history: &VecDeque<LatencyDataPoint>) -> StatisticalAnalysisData {
        let latencies: Vec<f32> = latency_history.iter().map(|d| d.latency_ms).collect();
        
        // 计算延迟分布
        let latency_distribution = self.calculate_latency_distribution(&latencies);
        
        // 计算百分位数
        let percentiles = self.calculate_percentiles(&latencies);
        
        // 趋势分析
        let trend_analysis = self.calculate_trend_analysis(&latencies);
        
        // 获取检测到的异常
        let detected_anomalies: Vec<_> = self.anomaly_detector.detected_anomalies.iter().cloned().collect();

        StatisticalAnalysisData {
            latency_distribution,
            percentiles,
            trend_analysis,
            detected_anomalies,
        }
    }

    fn calculate_latency_distribution(&self, latencies: &[f32]) -> LatencyDistribution {
        if latencies.is_empty() {
            return LatencyDistribution::default();
        }

        let n = latencies.len() as f32;
        let mean = latencies.iter().sum::<f32>() / n;
        
        let variance = latencies.iter()
            .map(|&latency| {
                let diff = latency - mean;
                diff * diff
            })
            .sum::<f32>() / n;
        
        // 创建分布区间
        let min = latencies.iter().fold(f32::MAX, |a, &b| a.min(b));
        let max = latencies.iter().fold(0.0_f32, |a, &b| a.max(b));
        let bin_count = 20;
        let bin_width = (max - min) / bin_count as f32;
        
        let mut bins = Vec::new();
        for i in 0..bin_count {
            let bin_min = min + i as f32 * bin_width;
            let bin_max = bin_min + bin_width;
            let count = latencies.iter()
                .filter(|&&latency| latency >= bin_min && latency < bin_max)
                .count() as u64;
            
            bins.push(LatencyBin {
                min_value: bin_min,
                max_value: bin_max,
                count,
                frequency: count as f32 / n,
            });
        }

        LatencyDistribution {
            bins,
            total_samples: latencies.len() as u64,
            mean,
            variance,
            skewness: 0.0, // 需要更复杂的计算
            kurtosis: 0.0, // 需要更复杂的计算
        }
    }

    fn calculate_percentiles(&self, latencies: &[f32]) -> Percentiles {
        if latencies.is_empty() {
            return Percentiles::default();
        }

        let mut sorted_latencies = latencies.to_vec();
        sorted_latencies.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let len = sorted_latencies.len();
        let get_percentile = |p: f32| {
            let index = ((len as f32 - 1.0) * p / 100.0) as usize;
            sorted_latencies[index]
        };

        Percentiles {
            p50: get_percentile(50.0),
            p75: get_percentile(75.0),
            p90: get_percentile(90.0),
            p95: get_percentile(95.0),
            p99: get_percentile(99.0),
        }
    }

    fn calculate_trend_analysis(&self, latencies: &[f32]) -> TrendAnalysis {
        if latencies.len() < 2 {
            return TrendAnalysis::default();
        }

        // 简单线性回归
        let n = latencies.len() as f32;
        let sum_x: f32 = (0..latencies.len()).map(|i| i as f32).sum();
        let sum_y: f32 = latencies.iter().sum();
        let sum_xy: f32 = latencies.iter().enumerate()
            .map(|(i, &latency)| i as f32 * latency)
            .sum();
        let sum_x2: f32 = (0..latencies.len()).map(|i| (i as f32).powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x.powi(2));

        let trend_direction = if slope.abs() < 0.1 {
            TrendDirection::Stable
        } else if slope > 0.0 {
            TrendDirection::Increasing
        } else {
            TrendDirection::Decreasing
        };

        let trend_strength = slope.abs() / latencies.iter().sum::<f32>() * latencies.len() as f32;

        TrendAnalysis {
            trend_direction,
            trend_strength,
            change_rate: slope,
            predicted_value: None,
            confidence_interval: None,
        }
    }

    fn reset(&mut self) {
        self.latency_distribution = LatencyDistribution::default();
        self.percentiles = Percentiles::default();
        self.trend_analysis = TrendAnalysis::default();
        self.anomaly_detector.reset();
    }
}

// 异常检测器实现
impl AnomalyDetector {
    fn new() -> Self {
        Self {
            detection_method: AnomalyDetectionMethod::Statistical,
            sensitivity: 2.0, // 2个标准差
            history_window: 100,
            detected_anomalies: VecDeque::with_capacity(100),
        }
    }

    fn detect_anomaly(&mut self, value: f32) {
        // 简化的异常检测：基于阈值
        // 实际实现中应该维护历史数据并计算统计信息
        
        // 这里只是一个示例，实际需要更复杂的逻辑
        if value > 200.0 { // 假设200ms为异常阈值
            let anomaly = LatencyAnomaly {
                anomaly_id: rand::random(),
                timestamp: Instant::now(),
                anomaly_value: value,
                expected_value: 50.0, // 假设期望值为50ms
                anomaly_score: (value - 50.0) / 50.0,
                anomaly_type: AnomalyType::Spike,
                description: format!("Latency spike detected: {:.2}ms", value),
            };
            
            self.detected_anomalies.push_back(anomaly);
            
            // 限制异常数量
            while self.detected_anomalies.len() > 100 {
                self.detected_anomalies.pop_front();
            }
        }
    }

    fn reset(&mut self) {
        self.detected_anomalies.clear();
    }
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 1000, // 1秒
            history_retention_s: 300, // 5分钟
            heatmap_resolution: (100, 100),
            heatmap_color_scheme: ColorScheme::Heat,
            trend_sampling_rate: 1.0,
            enable_realtime_updates: true,
            enable_prediction_analysis: true,
            enable_event_markers: true,
        }
    }
}

impl Default for NetworkCondition {
    fn default() -> Self {
        Self {
            bandwidth_utilization: 0.0,
            packet_loss_rate: 0.0,
            congestion_level: 0.0,
            quality_score: 100.0,
        }
    }
}

impl Default for Percentiles {
    fn default() -> Self {
        Self {
            p50: 0.0,
            p75: 0.0,
            p90: 0.0,
            p95: 0.0,
            p99: 0.0,
        }
    }
}

impl Default for TrendAnalysis {
    fn default() -> Self {
        Self {
            trend_direction: TrendDirection::Unknown,
            trend_strength: 0.0,
            change_rate: 0.0,
            predicted_value: None,
            confidence_interval: None,
        }
    }
}

impl Default for LatencyDistribution {
    fn default() -> Self {
        Self {
            bins: Vec::new(),
            total_samples: 0,
            mean: 0.0,
            variance: 0.0,
            skewness: 0.0,
            kurtosis: 0.0,
        }
    }
}

impl Default for LatencyVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_visualizer_creation() {
        let visualizer = LatencyVisualizer::new();
        assert!(visualizer.is_enabled());
        assert_eq!(visualizer.get_latency_history().len(), 0);
    }

    #[test]
    fn test_add_latency_data() {
        let mut visualizer = LatencyVisualizer::new();
        
        let data_point = LatencyDataPoint {
            timestamp_ms: current_timestamp_ms(),
            client_id: Some(1),
            server_id: Some(100),
            latency_ms: 50.0,
            latency_type: LatencyType::RoundTrip,
            predicted_latency_ms: Some(45.0),
            prediction_error: Some(5.0),
            network_condition: NetworkCondition::default(),
            event_markers: vec!["test".to_string()],
        };
        
        visualizer.add_latency_data(data_point);
        assert_eq!(visualizer.get_latency_history().len(), 1);
    }

    #[test]
    fn test_heatmap_generation() {
        let mut visualizer = LatencyVisualizer::new();
        
        // 添加一些测试数据
        for i in 0..10 {
            let data_point = LatencyDataPoint {
                timestamp_ms: current_timestamp_ms() + i as u64 * 1000,
                client_id: Some(i),
                server_id: Some(100),
                latency_ms: 50.0 + i as f32 * 5.0,
                latency_type: LatencyType::RoundTrip,
                predicted_latency_ms: None,
                prediction_error: None,
                network_condition: NetworkCondition::default(),
                event_markers: Vec::new(),
            };
            visualizer.add_latency_data(data_point);
        }
        
        let heatmap = visualizer.generate_heatmap();
        assert_eq!(heatmap.resolution, (100, 100)); // 默认分辨率
        assert!(!heatmap.data.is_empty());
    }

    #[test]
    fn test_trend_chart_generation() {
        let mut visualizer = LatencyVisualizer::new();
        
        // 添加一些测试数据
        for i in 0..10 {
            let data_point = LatencyDataPoint {
                timestamp_ms: current_timestamp_ms() + i as u64 * 1000,
                client_id: Some(1),
                server_id: Some(100),
                latency_ms: 50.0 + i as f32 * 2.0,
                latency_type: LatencyType::RoundTrip,
                predicted_latency_ms: None,
                prediction_error: None,
                network_condition: NetworkCondition::default(),
                event_markers: Vec::new(),
            };
            visualizer.add_latency_data(data_point);
        }
        
        let trend_chart = visualizer.generate_trend_chart();
        assert!(!trend_chart.data_points.is_empty());
        assert!(!trend_chart.average_line.is_empty());
    }

    #[test]
    fn test_realtime_statistics() {
        let mut visualizer = LatencyVisualizer::new();
        
        // 添加一些测试数据
        for i in 0..5 {
            let data_point = LatencyDataPoint {
                timestamp_ms: current_timestamp_ms() + i as u64 * 1000,
                client_id: Some(1),
                server_id: Some(100),
                latency_ms: 50.0 + i as f32 * 10.0,
                latency_type: LatencyType::RoundTrip,
                predicted_latency_ms: None,
                prediction_error: None,
                network_condition: NetworkCondition::default(),
                event_markers: Vec::new(),
            };
            visualizer.add_latency_data(data_point);
        }
        
        let stats = visualizer.get_realtime_statistics();
        assert_eq!(stats.sample_count, 5);
        assert!(stats.current_latency > 0.0);
        assert!(stats.average_latency > 0.0);
    }
}