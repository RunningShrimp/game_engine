//  性能告警系统
//
//  提供实时性能监控和告警功能，当性能指标超过阈值时触发告警。
//  支持多种告警类型、告警级别和告警通知方式。

use crate::performance::monitoring::system_monitor::PerformanceMetrics;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AlertLevel {
    /// 信息级别
    Info = 0,
    /// 警告级别
    Warning = 1,
    /// 错误级别
    Error = 2,
    /// 严重错误级别
    Critical = 3,
}

/// 告警类型
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AlertType {
    /// FPS过低
    LowFps,
    /// 帧时间过长
    HighFrameTime,
    /// 内存使用过高
    HighMemoryUsage,
    /// CPU使用率过高
    HighCpuUsage,
    /// GPU使用率过高
    HighGpuUsage,
    /// 渲染时间过长
    HighRenderTime,
    /// 物理时间过长
    HighPhysicsTime,
    /// 网络延迟过高
    HighNetworkLatency,
    /// 自定义告警
    Custom(String),
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AlertType::LowFps => write!(f, "LowFps"),
            AlertType::HighFrameTime => write!(f, "HighFrameTime"),
            AlertType::HighMemoryUsage => write!(f, "HighMemoryUsage"),
            AlertType::HighCpuUsage => write!(f, "HighCpuUsage"),
            AlertType::HighGpuUsage => write!(f, "HighGpuUsage"),
            AlertType::HighRenderTime => write!(f, "HighRenderTime"),
            AlertType::HighPhysicsTime => write!(f, "HighPhysicsTime"),
            AlertType::HighNetworkLatency => write!(f, "HighNetworkLatency"),
            AlertType::Custom(name) => write!(f, "{}", name),
        }
    }
}

/// 告警规则
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// 告警类型
    pub alert_type: AlertType,
    /// 告警级别
    pub level: AlertLevel,
    /// 阈值
    pub threshold: f64,
    /// 比较操作符
    pub operator: ComparisonOperator,
    /// 持续时间（超过阈值多久才触发）
    pub duration: Duration,
    /// 告警冷却时间（避免重复告警）
    pub cooldown: Duration,
    /// 是否启用
    pub enabled: bool,
    /// 告警消息模板
    pub message_template: String,
}

/// 比较操作符
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    /// 大于
    GreaterThan,
    /// 小于
    LessThan,
    /// 大于等于
    GreaterThanOrEqual,
    /// 小于等于
    LessThanOrEqual,
    /// 等于
    Equal,
    /// 不等于
    NotEqual,
}

/// 告警事件
#[derive(Debug, Clone)]
pub struct AlertEvent {
    /// 告警ID
    pub id: String,
    /// 告警类型
    pub alert_type: AlertType,
    /// 告警级别
    pub level: AlertLevel,
    /// 告警消息
    pub message: String,
    /// 当前值
    pub current_value: f64,
    /// 阈值
    pub threshold: f64,
    /// 触发时间
    pub timestamp: Instant,
    /// 持续时间
    pub duration: Duration,
    /// 元数据
    pub metadata: HashMap<String, String>,
}

/// 告警统计
#[derive(Debug, Clone, Default)]
pub struct AlertStatistics {
    /// 总告警数
    pub total_alerts: u64,
    /// 各级别告警数
    pub alerts_by_level: HashMap<AlertLevel, u64>,
    /// 各类型告警数
    pub alerts_by_type: HashMap<AlertType, u64>,
    /// 最近告警时间
    pub last_alert_time: Option<Instant>,
}

/// 告警处理器trait
pub trait AlertHandler: Send + Sync {
    /// 处理告警
    fn handle_alert(&self, alert: &AlertEvent);

    /// 获取处理器名称
    fn name(&self) -> &str;
}

/// 控制台告警处理器
pub struct ConsoleAlertHandler;

impl AlertHandler for ConsoleAlertHandler {
    fn handle_alert(&self, alert: &AlertEvent) {
        let level_str = match alert.level {
            AlertLevel::Info => "INFO",
            AlertLevel::Warning => "WARNING",
            AlertLevel::Error => "ERROR",
            AlertLevel::Critical => "CRITICAL",
        };

        println!(
            "[{}] {} - {} (value: {:.2}, threshold: {:.2})",
            level_str, alert.alert_type, alert.message, alert.current_value, alert.threshold
        );
    }

    fn name(&self) -> &str {
        "ConsoleAlertHandler"
    }
}

/// 日志告警处理器
pub struct LogAlertHandler;

impl AlertHandler for LogAlertHandler {
    fn handle_alert(&self, alert: &AlertEvent) {
        match alert.level {
            AlertLevel::Info => {
                tracing::info!(
                    target: "performance_alert",
                    alert_type = %alert.alert_type,
                    current_value = alert.current_value,
                    threshold = alert.threshold,
                    duration_ms = alert.duration.as_millis(),
                    "{}",
                    alert.message
                );
            }
            AlertLevel::Warning => {
                tracing::warn!(
                    target: "performance_alert",
                    alert_type = %alert.alert_type,
                    current_value = alert.current_value,
                    threshold = alert.threshold,
                    duration_ms = alert.duration.as_millis(),
                    "{}",
                    alert.message
                );
            }
            AlertLevel::Error | AlertLevel::Critical => {
                tracing::error!(
                    target: "performance_alert",
                    alert_type = %alert.alert_type,
                    current_value = alert.current_value,
                    threshold = alert.threshold,
                    duration_ms = alert.duration.as_millis(),
                    "{}",
                    alert.message
                );
            }
        }
    }

    fn name(&self) -> &str {
        "LogAlertHandler"
    }
}

/// 告警状态
#[derive(Debug, Clone)]
struct AlertState {
    /// 规则
    rule: AlertRule,
    /// 开始时间
    start_time: Option<Instant>,
    /// 是否已触发
    triggered: bool,
    /// 最后触发时间
    last_triggered: Option<Instant>,
}

/// 性能告警系统
pub struct PerformanceAlertSystem {
    /// 告警规则
    rules: HashMap<AlertType, AlertState>,
    /// 告警处理器
    handlers: Vec<Arc<dyn AlertHandler>>,
    /// 告警历史
    alert_history: VecDeque<AlertEvent>,
    /// 最大历史记录数
    max_history_size: usize,
    /// 告警统计
    statistics: AlertStatistics,
    /// 是否启用
    enabled: bool,
    /// 告警ID计数器
    alert_id_counter: u64,
}

impl std::fmt::Debug for PerformanceAlertSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PerformanceAlertSystem")
            .field("rules", &self.rules)
            .field("handlers", &self.handlers.len())
            .field("alert_history", &self.alert_history)
            .field("max_history_size", &self.max_history_size)
            .field("statistics", &self.statistics)
            .field("enabled", &self.enabled)
            .field("alert_id_counter", &self.alert_id_counter)
            .finish()
    }
}

impl Default for PerformanceAlertSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceAlertSystem {
    /// 创建新的性能告警系统
    pub fn new() -> Self {
        let mut system = Self {
            rules: HashMap::new(),
            handlers: Vec::new(),
            alert_history: VecDeque::with_capacity(1000),
            max_history_size: 1000,
            statistics: AlertStatistics::default(),
            enabled: true,
            alert_id_counter: 0,
        };

        system.add_default_rules();
        system.add_default_handlers();

        system
    }

    /// 添加默认告警规则
    fn add_default_rules(&mut self) {
        self.add_rule(AlertRule {
            alert_type: AlertType::LowFps,
            level: AlertLevel::Warning,
            threshold: 30.0,
            operator: ComparisonOperator::LessThan,
            duration: Duration::from_secs(5),
            cooldown: Duration::from_secs(30),
            enabled: true,
            message_template: "FPS is too low: {:.2} (threshold: {:.2})".to_string(),
        });

        self.add_rule(AlertRule {
            alert_type: AlertType::HighFrameTime,
            level: AlertLevel::Warning,
            threshold: 33.33,
            operator: ComparisonOperator::GreaterThan,
            duration: Duration::from_secs(5),
            cooldown: Duration::from_secs(30),
            enabled: true,
            message_template: "Frame time is too high: {:.2}ms (threshold: {:.2}ms)".to_string(),
        });

        self.add_rule(AlertRule {
            alert_type: AlertType::HighMemoryUsage,
            level: AlertLevel::Warning,
            threshold: 2048.0,
            operator: ComparisonOperator::GreaterThan,
            duration: Duration::from_secs(10),
            cooldown: Duration::from_secs(60),
            enabled: true,
            message_template: "Memory usage is high: {:.2}MB (threshold: {:.2}MB)".to_string(),
        });

        self.add_rule(AlertRule {
            alert_type: AlertType::HighCpuUsage,
            level: AlertLevel::Warning,
            threshold: 90.0,
            operator: ComparisonOperator::GreaterThan,
            duration: Duration::from_secs(10),
            cooldown: Duration::from_secs(60),
            enabled: true,
            message_template: "CPU usage is high: {:.2}% (threshold: {:.2}%)".to_string(),
        });

        self.add_rule(AlertRule {
            alert_type: AlertType::HighGpuUsage,
            level: AlertLevel::Warning,
            threshold: 95.0,
            operator: ComparisonOperator::GreaterThan,
            duration: Duration::from_secs(10),
            cooldown: Duration::from_secs(60),
            enabled: true,
            message_template: "GPU usage is high: {:.2}% (threshold: {:.2}%)".to_string(),
        });

        self.add_rule(AlertRule {
            alert_type: AlertType::HighRenderTime,
            level: AlertLevel::Warning,
            threshold: 16.67,
            operator: ComparisonOperator::GreaterThan,
            duration: Duration::from_secs(5),
            cooldown: Duration::from_secs(30),
            enabled: true,
            message_template: "Render time is high: {:.2}ms (threshold: {:.2}ms)".to_string(),
        });

        self.add_rule(AlertRule {
            alert_type: AlertType::HighPhysicsTime,
            level: AlertLevel::Warning,
            threshold: 5.0,
            operator: ComparisonOperator::GreaterThan,
            duration: Duration::from_secs(5),
            cooldown: Duration::from_secs(30),
            enabled: true,
            message_template: "Physics time is high: {:.2}ms (threshold: {:.2}ms)".to_string(),
        });

        self.add_rule(AlertRule {
            alert_type: AlertType::HighNetworkLatency,
            level: AlertLevel::Warning,
            threshold: 100.0,
            operator: ComparisonOperator::GreaterThan,
            duration: Duration::from_secs(5),
            cooldown: Duration::from_secs(30),
            enabled: true,
            message_template: "Network latency is high: {:.2}ms (threshold: {:.2}ms)".to_string(),
        });
    }

    /// 添加默认告警处理器
    fn add_default_handlers(&mut self) {
        self.add_handler(Arc::new(ConsoleAlertHandler));
        self.add_handler(Arc::new(LogAlertHandler));
    }

    /// 添加告警规则
    pub fn add_rule(&mut self, rule: AlertRule) {
        let alert_type = rule.alert_type.clone();
        self.rules.insert(
            alert_type,
            AlertState {
                rule,
                start_time: None,
                triggered: false,
                last_triggered: None,
            },
        );
    }

    /// 移除告警规则
    pub fn remove_rule(&mut self, alert_type: &AlertType) {
        self.rules.remove(alert_type);
    }

    /// 添加告警处理器
    pub fn add_handler(&mut self, handler: Arc<dyn AlertHandler>) {
        self.handlers.push(handler);
    }

    /// 移除告警处理器
    pub fn remove_handler(&mut self, name: &str) {
        self.handlers.retain(|h| h.name() != name);
    }

    /// 更新性能指标并检查告警
    pub fn update(&mut self, metrics: &PerformanceMetrics) {
        if !self.enabled {
            return;
        }

        self.check_alert(AlertType::LowFps, metrics.fps as f64);
        self.check_alert(AlertType::HighFrameTime, metrics.frame_time_ms as f64);
        self.check_alert(AlertType::HighMemoryUsage, metrics.memory_usage_mb as f64);
        self.check_alert(AlertType::HighCpuUsage, metrics.cpu_usage_percent as f64);
        self.check_alert(AlertType::HighGpuUsage, metrics.gpu_usage_percent as f64);
        self.check_alert(AlertType::HighRenderTime, metrics.gpu_render_time_ms as f64);
        self.check_alert(
            AlertType::HighPhysicsTime,
            metrics.physics_update_time_ms as f64,
        );
        self.check_alert(
            AlertType::HighNetworkLatency,
            metrics.network_sync_latency_ms as f64,
        );
    }

    /// 检查告警
    fn check_alert(&mut self, alert_type: AlertType, value: f64) {
        let (should_trigger, rule_info) = {
            let state = match self.rules.get_mut(&alert_type) {
                Some(state) => state,
                None => return,
            };

            if !state.rule.enabled {
                return;
            }

            let triggered = match state.rule.operator {
                ComparisonOperator::GreaterThan => value > state.rule.threshold,
                ComparisonOperator::LessThan => value < state.rule.threshold,
                ComparisonOperator::GreaterThanOrEqual => value >= state.rule.threshold,
                ComparisonOperator::LessThanOrEqual => value <= state.rule.threshold,
                ComparisonOperator::Equal => (value - state.rule.threshold).abs() < f64::EPSILON,
                ComparisonOperator::NotEqual => {
                    (value - state.rule.threshold).abs() >= f64::EPSILON
                }
            };

            let now = Instant::now();

            if triggered {
                if state.start_time.is_none() {
                    state.start_time = Some(now);
                }

                let duration = now.duration_since(state.start_time.expect("Test: operation should succeed"));

                if duration >= state.rule.duration {
                    if !state.triggered {
                        if let Some(last_triggered) = state.last_triggered {
                            if now.duration_since(last_triggered) < state.rule.cooldown {
                                return;
                            }
                        }

                        state.triggered = true;
                        state.last_triggered = Some(now);

                        let rule_info = (
                            state.rule.alert_type.clone(),
                            state.rule.level,
                            state.rule.message_template.clone(),
                            state.rule.threshold,
                            state.rule.operator,
                        );

                        (true, Some((duration, rule_info)))
                    } else {
                        (false, None)
                    }
                } else {
                    (false, None)
                }
            } else {
                state.start_time = None;
                state.triggered = false;
                (false, None)
            }
        };

        if should_trigger {
            if let Some((duration, rule_info)) = rule_info {
                self.trigger_alert_direct(rule_info, value, duration);
            }
        }
    }

    /// 直接触发告警
    fn trigger_alert_direct(
        &mut self,
        rule_info: (AlertType, AlertLevel, String, f64, ComparisonOperator),
        value: f64,
        duration: Duration,
    ) {
        let (alert_type, level, message_template, threshold, operator) = rule_info;

        let alert = AlertEvent {
            id: format!("alert_{}", self.alert_id_counter),
            alert_type: alert_type.clone(),
            level,
            message: message_template
                .replace("{:.2}", &format!("{:.2}", value))
                .replace("{:.2}", &format!("{:.2}", threshold)),
            current_value: value,
            threshold,
            timestamp: Instant::now(),
            duration,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("rule".to_string(), format!("{:?}", alert_type));
                meta.insert("operator".to_string(), format!("{:?}", operator));
                meta
            },
        };

        self.alert_id_counter += 1;

        self.statistics.total_alerts += 1;
        *self.statistics.alerts_by_level.entry(alert.level).or_insert(0) += 1;
        *self.statistics.alerts_by_type.entry(alert.alert_type.clone()).or_insert(0) += 1;
        self.statistics.last_alert_time = Some(alert.timestamp);

        self.alert_history.push_back(alert.clone());
        if self.alert_history.len() > self.max_history_size {
            self.alert_history.pop_front();
        }

        for handler in &self.handlers {
            handler.handle_alert(&alert);
        }
    }

    /// 获取告警历史
    pub fn get_alert_history(&self) -> Vec<AlertEvent> {
        self.alert_history.iter().cloned().collect()
    }

    /// 获取最近的告警
    pub fn get_recent_alerts(&self, count: usize) -> Vec<AlertEvent> {
        self.alert_history.iter().rev().take(count).cloned().collect()
    }

    /// 获取告警统计
    pub fn get_statistics(&self) -> AlertStatistics {
        self.statistics.clone()
    }

    /// 清除告警历史
    pub fn clear_history(&mut self) {
        self.alert_history.clear();
        self.statistics = AlertStatistics::default();
        self.alert_id_counter = 0;
    }

    /// 启用/禁用告警系统
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}
