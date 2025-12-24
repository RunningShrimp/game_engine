//  性能告警模块
//
//  提供阈值告警、趋势异常检测、多级告警和通知集成功能。

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

use super::ProfilingResult;
use super::visualization::TrendDirection;

// ============================================================================
// 告警配置
// ============================================================================

/// 告警配置
#[derive(Debug, Clone)]
pub struct AlertingConfig {
    /// 告警检查间隔
    pub check_interval: Duration,
    /// 告警去重时间窗口
    pub deduplication_window: Duration,
    /// 最大活跃告警数
    pub max_active_alerts: usize,
    /// 告警历史保留数量
    pub alert_history_size: usize,
    /// 是否启用自动确认
    pub enable_auto_acknowledge: bool,
    /// 自动确认时间
    pub auto_acknowledge_after: Duration,
    /// 通知配置
    pub notification_config: NotificationConfig,
}

impl Default for AlertingConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(1),
            deduplication_window: Duration::from_secs(30),
            max_active_alerts: 1000,
            alert_history_size: 10000,
            enable_auto_acknowledge: false,
            auto_acknowledge_after: Duration::from_secs(300), // 5分钟
            notification_config: NotificationConfig::default(),
        }
    }
}

/// 通知配置
#[derive(Debug, Clone)]
pub struct NotificationConfig {
    /// 是否启用邮件通知
    pub enable_email: bool,
    /// 邮件服务器配置
    pub email_config: Option<EmailConfig>,
    /// 是否启用Webhook通知
    pub enable_webhook: bool,
    /// Webhook URL列表
    webhook_urls: Vec<String>,
    /// 是否启用日志通知
    pub enable_log: bool,
    /// 日志级别
    pub log_level: LogLevel,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            enable_email: false,
            email_config: None,
            enable_webhook: false,
            webhook_urls: Vec::new(),
            enable_log: true,
            log_level: LogLevel::Warning,
        }
    }
}

/// 邮件配置
#[derive(Debug, Clone)]
pub struct EmailConfig {
    /// SMTP服务器地址
    pub smtp_server: String,
    /// SMTP端口
    pub smtp_port: u16,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 发件人
    pub from_address: String,
    /// 收件人列表
    pub to_addresses: Vec<String>,
    /// 是否使用TLS
    pub use_tls: bool,
}

/// 日志级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Critical,
}

impl LogLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Critical => "CRITICAL",
        }
    }
}

// ============================================================================
// 告警策略
// ============================================================================

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum AlertLevel {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重
    Critical,
}

impl AlertLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertLevel::Info => "Info",
            AlertLevel::Warning => "Warning",
            AlertLevel::Error => "Error",
            AlertLevel::Critical => "Critical",
        }
    }
}

/// 告警操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertOperator {
    /// 大于
    GreaterThan,
    /// 小于
    LessThan,
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
    /// 大于等于
    GreaterThanOrEqual,
    /// 小于等于
    LessThanOrEqual,
}

impl AlertOperator {
    /// 评估操作符
    pub fn evaluate(&self, value: f64, threshold: f64) -> bool {
        match self {
            AlertOperator::GreaterThan => value > threshold,
            AlertOperator::LessThan => value < threshold,
            AlertOperator::Equal => (value - threshold).abs() < f64::EPSILON,
            AlertOperator::NotEqual => (value - threshold).abs() >= f64::EPSILON,
            AlertOperator::GreaterThanOrEqual => value >= threshold,
            AlertOperator::LessThanOrEqual => value <= threshold,
        }
    }
}

// TrendDirection 从 visualization 模块导入

/// 告警策略类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStrategy {
    /// 阈值告警
    Threshold(ThresholdAlertStrategy),
    /// 趋势告警
    Trend(TrendAlertStrategy),
    /// 异常检测告警
    Anomaly(AnomalyAlertStrategy),
    /// 复合告警
    Composite(CompositeAlertStrategy),
}

/// 阈值告警策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdAlertStrategy {
    /// 告警级别
    pub level: AlertLevel,
    /// 阈值
    pub threshold: f64,
    /// 比较操作
    pub operator: AlertOperator,
    /// 持续时间
    pub duration: Duration,
    /// 是否启用恢复通知
    pub enable_recovery_notification: bool,
}

/// 趋势告警策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAlertStrategy {
    /// 告警级别
    pub level: AlertLevel,
    /// 分析窗口大小
    pub window_size: usize,
    /// 趋势变化率阈值
    pub change_rate_threshold: f64,
    /// 置信度阈值
    pub confidence_threshold: f64,
    /// 趋势方向
    pub trend_direction: Option<TrendDirection>,
}

/// 异常检测告警策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyAlertStrategy {
    /// 告警级别
    pub level: AlertLevel,
    /// 异常检测算法
    pub algorithm: AnomalyAlgorithm,
    /// 异常分数阈值
    pub anomaly_score_threshold: f64,
    /// 敏感度
    pub sensitivity: AnomalySensitivity,
}

/// 异常检测算法
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnomalyAlgorithm {
    /// Z-Score检测
    ZScore,
    /// IQR检测
    IQR,
    /// 孤立森林
    IsolationForest,
    /// 移动平均偏差
    MovingAverageDeviation,
}

/// 异常敏感度
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AnomalySensitivity {
    Low,
    Medium,
    High,
    VeryHigh,
}

impl AnomalySensitivity {
    fn threshold_multiplier(&self) -> f64 {
        match self {
            AnomalySensitivity::Low => 3.0,
            AnomalySensitivity::Medium => 2.5,
            AnomalySensitivity::High => 2.0,
            AnomalySensitivity::VeryHigh => 1.5,
        }
    }
}

/// 复合告警策略
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeAlertStrategy {
    /// 子策略列表（使用Box避免递归类型大小问题）
    pub sub_strategies: Vec<Box<AlertStrategy>>,
    /// 逻辑操作符
    pub operator: CompositeOperator,
    /// 子策略满足数量
    pub required_matches: usize,
}

/// 复合操作符
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompositeOperator {
    /// 所有条件都满足
    All,
    /// 任一条件满足
    Any,
    /// 至少N个条件满足
    AtLeast,
}

// ============================================================================
// 告警实例
// ============================================================================

/// 告警实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertInstance {
    /// 告警ID
    pub id: String,
    /// 告警名称
    pub name: String,
    /// 告警级别
    pub level: AlertLevel,
    /// 指标名称
    pub metric_name: String,
    /// 当前值
    pub current_value: f64,
    /// 阈值/条件
    pub condition: String,
    /// 告警消息
    pub message: String,
    /// 创建时间
    pub created_at: u64,
    /// 最后更新时间
    pub updated_at: u64,
    /// 确认时间
    pub acknowledged_at: Option<u64>,
    /// 恢复时间
    pub resolved_at: Option<u64>,
    /// 告警状态
    pub status: AlertStatus,
    /// 告警策略
    pub strategy: AlertStrategy,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 告警状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertStatus {
    /// 活跃
    Active,
    /// 已确认
    Acknowledged,
    /// 已恢复
    Resolved,
    /// 已静默
    Suppressed,
}

impl AlertStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertStatus::Active => "活跃",
            AlertStatus::Acknowledged => "已确认",
            AlertStatus::Resolved => "已恢复",
            AlertStatus::Suppressed => "已静默",
        }
    }
}

/// 告警通知
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertNotification {
    /// 告警ID
    pub alert_id: String,
    /// 通知类型
    pub notification_type: NotificationType,
    /// 通知时间
    pub sent_at: u64,
    /// 通知状态
    pub status: NotificationStatus,
    /// 错误信息
    pub error_message: Option<String>,
}

/// 通知类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationType {
    Email,
    Webhook,
    Log,
    Slack,
    Teams,
}

/// 通知状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NotificationStatus {
    Pending,
    Sent,
    Failed,
    Retrying,
}

// ============================================================================
// 告警引擎
// ============================================================================

/// 告警引擎
pub struct AlertingEngine {
    config: AlertingConfig,
    alert_strategies: HashMap<String, Vec<AlertStrategy>>,
    active_alerts: HashMap<String, AlertInstance>,
    alert_history: VecDeque<AlertInstance>,
    notification_sender: Arc<Mutex<NotificationSender>>,
    last_check_time: Instant,
    metric_values: HashMap<String, VecDeque<f64>>,
}

impl AlertingEngine {
    /// 创建新的告警引擎
    pub fn new(config: AlertingConfig) -> Self {
        Self {
            config,
            alert_strategies: HashMap::new(),
            active_alerts: HashMap::new(),
            alert_history: VecDeque::new(),
            notification_sender: Arc::new(Mutex::new(NotificationSender::new())),
            last_check_time: Instant::now(),
            metric_values: HashMap::new(),
        }
    }

    /// 添加告警策略
    pub fn add_strategy(&mut self, metric_name: &str, strategy: AlertStrategy) {
        let strategies =
            self.alert_strategies.entry(metric_name.to_string()).or_insert_with(Vec::new);
        strategies.push(strategy);
    }

    /// 更新指标值
    pub fn update_metric(&mut self, metric_name: &str, value: f64) {
        let values =
            self.metric_values.entry(metric_name.to_string()).or_insert_with(VecDeque::new);

        // 添加新值
        values.push_back(value);

        // 限制历史数据大小
        let max_history = 1000; // 保留1000个历史值
        while values.len() > max_history {
            values.pop_front();
        }
    }

    /// 检查告警
    pub fn check_alerts(&mut self) -> ProfilingResult<Vec<AlertInstance>> {
        let now = Instant::now();
        if now.duration_since(self.last_check_time) < self.config.check_interval {
            return Ok(Vec::new());
        }

        let mut new_alerts = Vec::new();

        // 检查每个指标的告警策略
        for (metric_name, strategies) in &self.alert_strategies {
            if let Some(values) = self.metric_values.get(metric_name) {
                if let Some(&current_value) = values.back() {
                    for strategy in strategies {
                        if let Some(alert) =
                            self.evaluate_strategy(metric_name, current_value, values, strategy)?
                        {
                            // 检查去重
                            if !self.is_duplicate_alert(metric_name, &alert) {
                                new_alerts.push(alert);
                            }
                        }
                    }
                }
            }
        }

        // 处理新告警
        for alert in &new_alerts {
            self.add_alert(alert.clone());
        }

        // 检查恢复告警
        self.check_resolved_alerts();

        // 清理过期告警
        self.cleanup_expired_alerts();

        // 发送通知
        self.send_notifications(&new_alerts)?;

        self.last_check_time = now;
        Ok(new_alerts)
    }

    /// 评估告警策略
    fn evaluate_strategy(
        &self,
        metric_name: &str,
        current_value: f64,
        historical_values: &VecDeque<f64>,
        strategy: &AlertStrategy,
    ) -> ProfilingResult<Option<AlertInstance>> {
        match strategy {
            AlertStrategy::Threshold(threshold_strategy) => {
                self.evaluate_threshold_strategy(metric_name, current_value, threshold_strategy)
            }
            AlertStrategy::Trend(trend_strategy) => {
                self.evaluate_trend_strategy(metric_name, historical_values, trend_strategy)
            }
            AlertStrategy::Anomaly(anomaly_strategy) => {
                self.evaluate_anomaly_strategy(metric_name, historical_values, anomaly_strategy)
            }
            AlertStrategy::Composite(composite_strategy) => self.evaluate_composite_strategy(
                metric_name,
                current_value,
                historical_values,
                composite_strategy,
            ),
        }
    }

    /// 评估阈值策略
    fn evaluate_threshold_strategy(
        &self,
        metric_name: &str,
        current_value: f64,
        strategy: &ThresholdAlertStrategy,
    ) -> ProfilingResult<Option<AlertInstance>> {
        let should_alert = strategy.operator.evaluate(current_value, strategy.threshold);

        if should_alert {
            let alert = AlertInstance {
                id: format!(
                    "alert_{}_{}",
                    metric_name,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                ),
                name: format!("{}阈值告警", metric_name),
                level: strategy.level,
                metric_name: metric_name.to_string(),
                current_value,
                condition: format!(
                    "{} {}",
                    Self::operator_to_string(strategy.operator),
                    strategy.threshold
                ),
                message: format!(
                    "{} {} {} (当前: {}, 阈值: {})",
                    metric_name,
                    Self::operator_to_string(strategy.operator),
                    strategy.threshold,
                    current_value,
                    strategy.threshold
                ),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                acknowledged_at: None,
                resolved_at: None,
                status: AlertStatus::Active,
                strategy: AlertStrategy::Threshold(strategy.clone()),
                metadata: HashMap::new(),
            };

            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    /// 评估趋势策略
    fn evaluate_trend_strategy(
        &self,
        metric_name: &str,
        historical_values: &VecDeque<f64>,
        strategy: &TrendAlertStrategy,
    ) -> ProfilingResult<Option<AlertInstance>> {
        if historical_values.len() < strategy.window_size {
            return Ok(None);
        }

        // 提取窗口数据
        let window_data: Vec<f64> =
            historical_values.iter().rev().take(strategy.window_size).cloned().collect();

        // 计算趋势
        let trend_direction = self.calculate_trend(&window_data);

        // 检查趋势方向匹配
        if let Some(required_direction) = strategy.trend_direction {
            if trend_direction != required_direction {
                return Ok(None);
            }
        }

        // 计算变化率和置信度
        let change_rate = self.calculate_change_rate(&window_data);
        let confidence = self.calculate_confidence(&window_data);

        // 检查阈值
        if change_rate.abs() >= strategy.change_rate_threshold
            && confidence >= strategy.confidence_threshold
        {
            let alert = AlertInstance {
                id: format!(
                    "trend_alert_{}_{}",
                    metric_name,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                ),
                name: format!("{}趋势告警", metric_name),
                level: strategy.level,
                metric_name: metric_name.to_string(),
                current_value: *window_data.last().unwrap(),
                condition: format!(
                    "趋势变化率 {:.2}% (置信度: {:.2}%)",
                    change_rate,
                    confidence * 100.0
                ),
                message: format!(
                    "{}检测到{}趋势，变化率: {:.2}%，置信度: {:.2}%",
                    metric_name,
                    trend_direction.as_str(),
                    change_rate,
                    confidence * 100.0
                ),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                acknowledged_at: None,
                resolved_at: None,
                status: AlertStatus::Active,
                strategy: AlertStrategy::Trend(strategy.clone()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert(
                        "trend_direction".to_string(),
                        trend_direction.as_str().to_string(),
                    );
                    meta.insert("change_rate".to_string(), change_rate.to_string());
                    meta.insert("confidence".to_string(), confidence.to_string());
                    meta
                },
            };

            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    /// 评估异常检测策略
    fn evaluate_anomaly_strategy(
        &self,
        metric_name: &str,
        historical_values: &VecDeque<f64>,
        strategy: &AnomalyAlertStrategy,
    ) -> ProfilingResult<Option<AlertInstance>> {
        if historical_values.len() < 10 {
            return Ok(None);
        }

        let anomaly_score = match strategy.algorithm {
            AnomalyAlgorithm::ZScore => self.calculate_zscore_anomaly(historical_values),
            AnomalyAlgorithm::IQR => self.calculate_iqr_anomaly(historical_values),
            AnomalyAlgorithm::IsolationForest => {
                self.calculate_isolation_forest_anomaly(historical_values)
            }
            AnomalyAlgorithm::MovingAverageDeviation => {
                self.calculate_moving_average_anomaly(historical_values)
            }
        };

        let threshold =
            strategy.anomaly_score_threshold * strategy.sensitivity.threshold_multiplier();

        if anomaly_score >= threshold {
            let alert = AlertInstance {
                id: format!(
                    "anomaly_alert_{}_{}",
                    metric_name,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                ),
                name: format!("{}异常检测告警", metric_name),
                level: strategy.level,
                metric_name: metric_name.to_string(),
                current_value: *historical_values.back().unwrap(),
                condition: format!("异常分数 {:.2} (阈值: {:.2})", anomaly_score, threshold),
                message: format!(
                    "{}检测到异常，异常分数: {:.2}，算法: {:?}",
                    metric_name, anomaly_score, strategy.algorithm
                ),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                acknowledged_at: None,
                resolved_at: None,
                status: AlertStatus::Active,
                strategy: AlertStrategy::Anomaly(strategy.clone()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("anomaly_score".to_string(), anomaly_score.to_string());
                    meta.insert("algorithm".to_string(), format!("{:?}", strategy.algorithm));
                    meta.insert(
                        "sensitivity".to_string(),
                        format!("{:?}", strategy.sensitivity),
                    );
                    meta
                },
            };

            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    /// 评估复合策略
    fn evaluate_composite_strategy(
        &self,
        metric_name: &str,
        current_value: f64,
        historical_values: &VecDeque<f64>,
        strategy: &CompositeAlertStrategy,
    ) -> ProfilingResult<Option<AlertInstance>> {
        let mut matched_alerts = Vec::new();

        for sub_strategy in &strategy.sub_strategies {
            if let Some(alert) =
                self.evaluate_strategy(metric_name, current_value, historical_values, sub_strategy)?
            {
                matched_alerts.push(alert);
            }
        }

        let should_alert = match strategy.operator {
            CompositeOperator::All => matched_alerts.len() == strategy.sub_strategies.len(),
            CompositeOperator::Any => !matched_alerts.is_empty(),
            CompositeOperator::AtLeast => matched_alerts.len() >= strategy.required_matches,
        };

        if should_alert && !matched_alerts.is_empty() {
            // 合并告警信息
            let highest_level = matched_alerts
                .iter()
                .map(|a| a.level)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap();

            let alert = AlertInstance {
                id: format!(
                    "composite_alert_{}_{}",
                    metric_name,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                ),
                name: format!("{}复合告警", metric_name),
                level: highest_level,
                metric_name: metric_name.to_string(),
                current_value,
                condition: format!("复合条件满足 ({}个条件)", matched_alerts.len()),
                message: format!(
                    "{}复合告警触发，满足{}个子条件",
                    metric_name,
                    matched_alerts.len()
                ),
                created_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                acknowledged_at: None,
                resolved_at: None,
                status: AlertStatus::Active,
                strategy: AlertStrategy::Composite(strategy.clone()),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert(
                        "sub_alerts".to_string(),
                        format!("{}", matched_alerts.len()),
                    );
                    meta.insert("operator".to_string(), format!("{:?}", strategy.operator));
                    meta
                },
            };

            Ok(Some(alert))
        } else {
            Ok(None)
        }
    }

    /// 添加告警
    fn add_alert(&mut self, alert: AlertInstance) {
        let alert_id = alert.id.clone();

        // 检查是否已存在
        if let Some(existing_alert) = self.active_alerts.get_mut(&alert_id) {
            // 更新现有告警
            existing_alert.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            existing_alert.current_value = alert.current_value;
            return;
        }

        // 添加到历史记录（在移动alert之前）
        self.alert_history.push_back(alert.clone());

        // 添加新告警
        self.active_alerts.insert(alert_id, alert);

        // 限制历史记录大小
        while self.active_alerts.len() > self.config.alert_history_size {
            if let Some(key) = self.active_alerts.keys().next().cloned() {
                self.active_alerts.remove(&key);
            } else {
                break;
            }
        }
    }

    /// 检查恢复告警
    fn check_resolved_alerts(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut alerts_to_resolve = Vec::new();

        for (alert_id, alert) in &self.active_alerts {
            if alert.status == AlertStatus::Active {
                // 重新评估告警条件
                if let Some(historical_values) = self.metric_values.get(&alert.metric_name) {
                    if let Some(&current_value) = historical_values.back() {
                        if let Ok(should_resolve) =
                            self.should_resolve_alert(alert, current_value, historical_values)
                        {
                            if should_resolve {
                                alerts_to_resolve.push((alert_id.clone(), now));
                            }
                        }
                    }
                }
            }
        }

        for (alert_id, resolved_time) in alerts_to_resolve {
            if let Some(alert) = self.active_alerts.get_mut(&alert_id) {
                alert.status = AlertStatus::Resolved;
                alert.resolved_at = Some(resolved_time);
                alert.updated_at = resolved_time;
            }
        }
    }

    /// 判断告警是否应该恢复
    fn should_resolve_alert(
        &self,
        alert: &AlertInstance,
        current_value: f64,
        historical_values: &VecDeque<f64>,
    ) -> ProfilingResult<bool> {
        match &alert.strategy {
            AlertStrategy::Threshold(threshold_strategy) => Ok(!threshold_strategy
                .operator
                .evaluate(current_value, threshold_strategy.threshold)),
            AlertStrategy::Trend(_) => {
                // 趋势告警通常需要手动确认
                Ok(false)
            }
            AlertStrategy::Anomaly(anomaly_strategy) => {
                let anomaly_score = match anomaly_strategy.algorithm {
                    AnomalyAlgorithm::ZScore => self.calculate_zscore_anomaly(historical_values),
                    AnomalyAlgorithm::IQR => self.calculate_iqr_anomaly(historical_values),
                    AnomalyAlgorithm::IsolationForest => {
                        self.calculate_isolation_forest_anomaly(historical_values)
                    }
                    AnomalyAlgorithm::MovingAverageDeviation => {
                        self.calculate_moving_average_anomaly(historical_values)
                    }
                };

                let threshold = anomaly_strategy.anomaly_score_threshold
                    * anomaly_strategy.sensitivity.threshold_multiplier();
                Ok(anomaly_score < threshold)
            }
            AlertStrategy::Composite(_) => {
                // 复合告警需要所有子条件都不满足
                Ok(false)
            }
        }
    }

    /// 清理过期告警
    fn cleanup_expired_alerts(&mut self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.active_alerts.retain(|_, alert| {
            // 移除超过最大活跃时间的告警
            let alert_age = now - alert.created_at;
            let max_age = 24 * 60 * 60; // 24小时

            alert_age < max_age
        });
    }

    /// 检查是否为重复告警
    fn is_duplicate_alert(&self, metric_name: &str, new_alert: &AlertInstance) -> bool {
        for alert in self.active_alerts.values() {
            if alert.metric_name == metric_name
                && alert.status == AlertStatus::Active
                && alert.level == new_alert.level
            {
                let time_diff = new_alert.created_at.saturating_sub(alert.created_at);
                if time_diff < self.config.deduplication_window.as_secs() {
                    return true;
                }
            }
        }
        false
    }

    /// 发送通知
    fn send_notifications(&self, alerts: &[AlertInstance]) -> ProfilingResult<()> {
        if alerts.is_empty() {
            return Ok(());
        }

        if let Ok(mut sender) = self.notification_sender.lock() {
            for alert in alerts {
                let _: ProfilingResult<()> = sender.send_alert(alert);
            }
        }

        Ok(())
    }

    // 辅助方法
    fn calculate_trend(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::Stable;
        }

        let first = values[0];
        let last = values[values.len() - 1];

        if last > first * 1.1 {
            TrendDirection::Increasing
        } else if last < first * 0.9 {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    fn calculate_change_rate(&self, values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let first = values[0];
        let last = values[values.len() - 1];

        if first.abs() < f64::EPSILON {
            return 0.0;
        }

        ((last - first) / first.abs()) * 100.0
    }

    fn calculate_confidence(&self, values: &[f64]) -> f64 {
        if values.len() < 3 {
            return 0.0;
        }

        let n = values.len() as f64;
        let indices: Vec<f64> = (0..values.len()).map(|i| i as f64).collect();

        let mean_x = indices.iter().sum::<f64>() / n;
        let mean_y = values.iter().sum::<f64>() / n;

        let numerator: f64 = indices
            .iter()
            .zip(values.iter())
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum();

        let sum_x2: f64 = indices.iter().map(|x| (x - mean_x).powi(2)).sum();
        let sum_y2: f64 = values.iter().map(|y| (y - mean_y).powi(2)).sum();

        let denominator = (sum_x2 * sum_y2).sqrt();

        if denominator < f64::EPSILON {
            return 0.0;
        }

        let correlation = numerator / denominator;
        correlation.abs().min(1.0)
    }

    fn calculate_zscore_anomaly(&self, values: &VecDeque<f64>) -> f64 {
        if values.len() < 3 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
        let std_dev = variance.sqrt();

        let current = *values.back().unwrap();
        if std_dev < f64::EPSILON {
            return 0.0;
        }

        (current - mean).abs() / std_dev
    }

    fn calculate_iqr_anomaly(&self, values: &VecDeque<f64>) -> f64 {
        if values.len() < 4 {
            return 0.0;
        }

        let mut sorted_values: Vec<f64> = values.iter().cloned().collect();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let n = sorted_values.len();
        let q1 = sorted_values[n / 4];
        let q3 = sorted_values[3 * n / 4];
        let iqr = q3 - q1;

        let current = *values.back().unwrap();

        if current < q1 - 1.5 * iqr || current > q3 + 1.5 * iqr {
            ((current - q1).abs().max((current - q3).abs())) / iqr
        } else {
            0.0
        }
    }

    fn calculate_isolation_forest_anomaly(&self, _values: &VecDeque<f64>) -> f64 {
        // 简化实现，实际应使用专门的机器学习库
        0.0
    }

    fn calculate_moving_average_anomaly(&self, values: &VecDeque<f64>) -> f64 {
        if values.len() < 10 {
            return 0.0;
        }

        let window_size = (values.len() / 3).min(20);
        let current = *values.back().unwrap();

        let recent_avg: f64 =
            values.iter().rev().take(window_size).sum::<f64>() / window_size as f64;

        let overall_avg = values.iter().sum::<f64>() / values.len() as f64;

        if overall_avg < f64::EPSILON {
            return 0.0;
        }

        ((current - recent_avg).abs() / overall_avg.abs()) * 100.0
    }

    fn operator_to_string(op: AlertOperator) -> &'static str {
        match op {
            AlertOperator::GreaterThan => ">",
            AlertOperator::LessThan => "<",
            AlertOperator::Equal => "==",
            AlertOperator::NotEqual => "!=",
            AlertOperator::GreaterThanOrEqual => ">=",
            AlertOperator::LessThanOrEqual => "<=",
        }
    }

    /// 获取活跃告警
    pub fn get_active_alerts(&self) -> Vec<&AlertInstance> {
        self.active_alerts.values().collect()
    }

    /// 获取告警历史
    pub fn get_alert_history(&self, limit: Option<usize>) -> Vec<&AlertInstance> {
        let history = &self.alert_history;
        if let Some(limit) = limit {
            history.iter().rev().take(limit).collect()
        } else {
            history.iter().rev().collect()
        }
    }

    /// 确认告警
    pub fn acknowledge_alert(&mut self, alert_id: &str) -> bool {
        if let Some(alert) = self.active_alerts.get_mut(alert_id) {
            if alert.status == AlertStatus::Active {
                alert.status = AlertStatus::Acknowledged;
                alert.acknowledged_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                );
                return true;
            }
        }
        false
    }
}

// ============================================================================
// 通知发送器
// ============================================================================

/// 通知发送器
pub struct NotificationSender {
    config: NotificationConfig,
}

impl NotificationSender {
    /// 创建新的通知发送器
    pub fn new() -> Self {
        Self {
            config: NotificationConfig::default(),
        }
    }

    /// 发送告警通知
    pub fn send_alert(&mut self, alert: &AlertInstance) -> ProfilingResult<()> {
        // 发送邮件通知
        if self.config.enable_email {
            if let Some(ref email_config) = self.config.email_config {
                self.send_email_notification(alert, email_config)?;
            }
        }

        // 发送Webhook通知
        if self.config.enable_webhook {
            for webhook_url in &self.config.webhook_urls {
                self.send_webhook_notification(alert, webhook_url)?;
            }
        }

        // 发送日志通知
        if self.config.enable_log {
            self.send_log_notification(alert)?;
        }

        Ok(())
    }

    /// 发送邮件通知
    fn send_email_notification(
        &self,
        alert: &AlertInstance,
        _config: &EmailConfig,
    ) -> ProfilingResult<()> {
        // 简化实现，实际应使用SMTP库
        tracing::info!(
            target: "profiling",
            "发送邮件通知: {} - {}",
            alert.name,
            alert.message
        );
        Ok(())
    }

    /// 发送Webhook通知
    fn send_webhook_notification(
        &self,
        alert: &AlertInstance,
        webhook_url: &str,
    ) -> ProfilingResult<()> {
        // 简化实现，实际应使用HTTP客户端
        tracing::info!(
            target: "profiling",
            "发送Webhook通知: {} - {}",
            alert.name,
            webhook_url
        );
        Ok(())
    }

    /// 发送日志通知
    fn send_log_notification(&self, alert: &AlertInstance) -> ProfilingResult<()> {
        let log_level = match alert.level {
            AlertLevel::Info => LogLevel::Info,
            AlertLevel::Warning => LogLevel::Warning,
            AlertLevel::Error => LogLevel::Error,
            AlertLevel::Critical => LogLevel::Critical,
        };

        if log_level as u8 >= self.config.log_level as u8 {
            tracing::info!(
                target: "profiling",
                "告警通知 [{}]: {} - {}",
                log_level.as_str(),
                alert.name,
                alert.message
            );
        }

        Ok(())
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
    fn test_alert_status() {
        assert_eq!(AlertStatus::Active.as_str(), "活跃");
        assert_eq!(AlertStatus::Acknowledged.as_str(), "已确认");
        assert_eq!(AlertStatus::Resolved.as_str(), "已恢复");
        assert_eq!(AlertStatus::Suppressed.as_str(), "已静默");
    }

    #[test]
    fn test_log_level() {
        assert_eq!(LogLevel::Info.as_str(), "INFO");
        assert_eq!(LogLevel::Warning.as_str(), "WARNING");
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Critical.as_str(), "CRITICAL");
    }

    #[test]
    fn test_anomaly_sensitivity() {
        assert_eq!(AnomalySensitivity::Low.threshold_multiplier(), 3.0);
        assert_eq!(AnomalySensitivity::Medium.threshold_multiplier(), 2.5);
        assert_eq!(AnomalySensitivity::High.threshold_multiplier(), 2.0);
        assert_eq!(AnomalySensitivity::VeryHigh.threshold_multiplier(), 1.5);
    }

    #[test]
    fn test_alerting_engine() {
        let config = AlertingConfig::default();
        let mut engine = AlertingEngine::new(config);

        // 添加阈值告警策略
        engine.add_strategy(
            "test_metric",
            AlertStrategy::Threshold(ThresholdAlertStrategy {
                level: AlertLevel::Warning,
                threshold: 50.0,
                operator: AlertOperator::GreaterThan,
                duration: Duration::from_secs(5),
                enable_recovery_notification: true,
            }),
        );

        // 更新指标值
        engine.update_metric("test_metric", 60.0);

        // 检查告警
        let new_alerts = engine.check_alerts().unwrap();
        assert_eq!(new_alerts.len(), 1);
        assert_eq!(new_alerts[0].metric_name, "test_metric");
        assert_eq!(new_alerts[0].level, AlertLevel::Warning);
    }

    #[test]
    fn test_notification_sender() {
        let mut sender = NotificationSender::new();

        let alert = AlertInstance {
            id: "test_alert".to_string(),
            name: "测试告警".to_string(),
            level: AlertLevel::Warning,
            metric_name: "test_metric".to_string(),
            current_value: 60.0,
            condition: "> 50.0".to_string(),
            message: "测试指标超过阈值".to_string(),
            created_at: 1234567890,
            updated_at: 1234567890,
            acknowledged_at: None,
            resolved_at: None,
            status: AlertStatus::Active,
            strategy: AlertStrategy::Threshold(ThresholdAlertStrategy {
                level: AlertLevel::Warning,
                threshold: 50.0,
                operator: AlertOperator::GreaterThan,
                duration: Duration::from_secs(5),
                enable_recovery_notification: true,
            }),
            metadata: HashMap::new(),
        };

        // 测试通知发送（不会实际发送）
        let result = sender.send_alert(&alert);
        assert!(result.is_ok());
    }
}
