//  网络性能监控模块
// 
//  实现实时网络性能指标收集、分析和报告功能。
// 
//  ## 功能特性
// 
//  - 实时网络性能指标收集
//  - 带宽使用率监控
//  - 延迟和丢包率统计
//  - 网络质量评估
//  - 性能趋势分析
//  - 自适应监控频率

use crate::core::utils::current_timestamp_ms;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// 网络性能监控器
pub struct NetworkPerformanceMonitor {
    /// 是否启用
    enabled: bool,
    /// 性能指标历史
    performance_history: VecDeque<NetworkPerformanceMetrics>,
    /// 最大历史长度
    max_history_size: usize,
    /// 当前指标
    current_metrics: NetworkPerformanceMetrics,
    /// 监控配置
    config: MonitorConfig,
    /// 连接统计（连接ID -> 统计）
    connection_stats: HashMap<u64, ConnectionPerformanceStats>,
    /// 带宽监控器
    bandwidth_monitor: BandwidthMonitor,
    /// 延迟监控器
    latency_monitor: LatencyMonitor,
    /// 丢包监控器
    packet_loss_monitor: PacketLossMonitor,
    /// 网络质量评估器
    quality_assessor: NetworkQualityAssessor,
    /// 最后更新时间
    last_update: Instant,
    /// 监控开始时间
    start_time: Instant,
}

/// 网络性能指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPerformanceMetrics {
    /// 时间戳
    pub timestamp_ms: u64,
    /// 上行带宽（Kbps）
    pub upload_bandwidth_kbps: f32,
    /// 下行带宽（Kbps）
    pub download_bandwidth_kbps: f32,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f32,
    /// 最小延迟（毫秒）
    pub min_latency_ms: f32,
    /// 最大延迟（毫秒）
    pub max_latency_ms: f32,
    /// 延迟抖动（毫秒）
    pub latency_jitter_ms: f32,
    /// 丢包率（0-1）
    pub packet_loss_rate: f32,
    /// 发送包数
    pub packets_sent: u64,
    /// 接收包数
    pub packets_received: u64,
    /// 丢失包数
    pub packets_lost: u64,
    /// 重传包数
    pub packets_retransmitted: u64,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 接收字节数
    pub bytes_received: u64,
    /// 网络质量评分（0-100）
    pub quality_score: f32,
    /// 连接数
    pub active_connections: u32,
    /// 错误计数
    pub error_count: u32,
    /// 警告计数
    pub warning_count: u32,
}

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// 更新间隔（毫秒）
    pub update_interval_ms: u64,
    /// 历史数据保留时间（秒）
    pub history_retention_s: u64,
    /// 是否启用详细监控
    pub detailed_monitoring: bool,
    /// 带宽采样窗口大小
    pub bandwidth_sample_window: usize,
    /// 延迟采样窗口大小
    pub latency_sample_window: usize,
    /// 丢包采样窗口大小
    pub packet_loss_sample_window: usize,
    /// 质量评估间隔（秒）
    pub quality_assessment_interval_s: u64,
    /// 性能警告阈值
    pub performance_thresholds: PerformanceThresholds,
}

/// 性能警告阈值
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// 高延迟阈值（毫秒）
    pub high_latency_threshold: f32,
    /// 高丢包率阈值
    pub high_packet_loss_threshold: f32,
    /// 低带宽阈值（Kbps）
    pub low_bandwidth_threshold: f32,
    /// 高抖动阈值（毫秒）
    pub high_jitter_threshold: f32,
}

/// 连接性能统计
#[derive(Debug, Clone)]
pub struct ConnectionPerformanceStats {
    /// 连接ID
    pub connection_id: u64,
    /// 连接建立时间
    pub connection_time: Instant,
    /// 最后活动时间
    pub last_activity: Instant,
    /// 发送包数
    pub packets_sent: u64,
    /// 接收包数
    pub packets_received: u64,
    /// 丢失包数
    pub packets_lost: u64,
    /// 平均延迟
    pub average_latency: f32,
    /// 连接质量评分
    pub quality_score: f32,
}

/// 带宽监控器
#[derive(Debug)]
struct BandwidthMonitor {
    /// 上行带宽采样
    upload_samples: VecDeque<BandwidthSample>,
    /// 下行带宽采样
    download_samples: VecDeque<BandwidthSample>,
    /// 采样窗口大小
    window_size: usize,
    /// 当前上行带宽
    current_upload_kbps: f32,
    /// 当前下行带宽
    current_download_kbps: f32,
    /// 峰值上行带宽
    peak_upload_kbps: f32,
    /// 峰值下行带宽
    peak_download_kbps: f32,
}

/// 带宽采样
#[derive(Debug, Clone)]
struct BandwidthSample {
    /// 时间戳
    #[allow(dead_code)]
    timestamp: Instant,
    /// 字节数
    #[allow(dead_code)]
    bytes: u64,
}

/// 延迟监控器
#[derive(Debug)]
struct LatencyMonitor {
    /// 延迟采样
    latency_samples: VecDeque<LatencySample>,
    /// 采样窗口大小
    window_size: usize,
    /// 当前平均延迟
    current_average_ms: f32,
    /// 当前最小延迟
    current_min_ms: f32,
    /// 当前最大延迟
    current_max_ms: f32,
    /// 当前抖动
    current_jitter_ms: f32,
}

/// 延迟采样
#[derive(Debug, Clone)]
struct LatencySample {
    /// 时间戳
    #[allow(dead_code)]
    timestamp: Instant,
    /// 延迟（毫秒）
    #[allow(dead_code)]
    latency_ms: f32,
}

/// 丢包监控器
#[derive(Debug)]
struct PacketLossMonitor {
    /// 发送包计数
    packets_sent: u64,
    /// 接收包计数
    packets_received: u64,
    /// 丢失包计数
    packets_lost: u64,
    /// 丢包率历史
    loss_rate_history: VecDeque<f32>,
    /// 历史窗口大小
    history_window: usize,
    /// 当前丢包率
    current_loss_rate: f32,
}

/// 网络质量评估器
#[derive(Debug)]
struct NetworkQualityAssessor {
    /// 最后评估时间
    last_assessment: Instant,
    /// 评估间隔
    assessment_interval: Duration,
    /// 当前质量评分
    current_score: f32,
    /// 质量历史
    quality_history: VecDeque<QualitySample>,
    /// 历史窗口大小
    history_window: usize,
}

/// 质量采样
#[derive(Debug, Clone)]
struct QualitySample {
    /// 时间戳
    #[allow(dead_code)]
    timestamp: Instant,
    /// 质量评分
    #[allow(dead_code)]
    score: f32,
    /// 评估因素
    #[allow(dead_code)]
    factors: QualityFactors,
}

/// 质量评估因素
#[derive(Debug, Clone)]
struct QualityFactors {
    /// 延迟因子
    #[allow(dead_code)]
    latency_factor: f32,
    /// 丢包因子
    #[allow(dead_code)]
    packet_loss_factor: f32,
    /// 带宽因子
    #[allow(dead_code)]
    bandwidth_factor: f32,
    /// 抖动因子
    #[allow(dead_code)]
    jitter_factor: f32,
}

impl NetworkPerformanceMonitor {
    /// 创建新的网络性能监控器
    pub fn new() -> Self {
        Self::with_config(MonitorConfig::default())
    }

    /// 创建带配置的网络性能监控器
    pub fn with_config(config: MonitorConfig) -> Self {
        let window_size = config.bandwidth_sample_window.max(config.latency_sample_window)
            .max(config.packet_loss_sample_window);

        Self {
            enabled: true,
            performance_history: VecDeque::with_capacity(
                (config.history_retention_s * 1000 / config.update_interval_ms) as usize
            ),
            max_history_size: (config.history_retention_s * 1000 / config.update_interval_ms) as usize,
            current_metrics: NetworkPerformanceMetrics::default(),
            config,
            connection_stats: HashMap::new(),
            bandwidth_monitor: BandwidthMonitor::new(window_size),
            latency_monitor: LatencyMonitor::new(window_size),
            packet_loss_monitor: PacketLossMonitor::new(window_size),
            quality_assessor: NetworkQualityAssessor::new(),
            last_update: Instant::now(),
            start_time: Instant::now(),
        }
    }

    /// 更新监控器
    pub fn update(&mut self, delta_time: Duration) {
        if !self.enabled {
            return;
        }

        let now = Instant::now();
        
        // 检查是否需要更新
        if now.duration_since(self.last_update).as_millis() < self.config.update_interval_ms as u128 {
            return;
        }

        // 更新各个监控器
        self.update_bandwidth_monitor(delta_time);
        self.update_latency_monitor();
        self.update_packet_loss_monitor();
        self.update_quality_assessment();

        // 生成当前指标
        self.generate_current_metrics();

        // 添加到历史
        self.performance_history.push_back(self.current_metrics.clone());

        // 限制历史长度
        while self.performance_history.len() > self.max_history_size {
            self.performance_history.pop_front();
        }

        // 清理过期的连接统计
        self.cleanup_expired_connections();

        self.last_update = now;
    }

    /// 记录发送的数据包
    pub fn record_packet_sent(&mut self, connection_id: u64, bytes: u64) {
        if !self.enabled {
            return;
        }

        // 更新包丢失监控器
        self.packet_loss_monitor.record_sent(bytes);

        // 更新连接统计
        if let Some(stats) = self.connection_stats.get_mut(&connection_id) {
            stats.packets_sent += 1;
            stats.last_activity = Instant::now();
        } else {
            // 创建新的连接统计
            self.connection_stats.insert(connection_id, ConnectionPerformanceStats {
                connection_id,
                connection_time: Instant::now(),
                last_activity: Instant::now(),
                packets_sent: 1,
                packets_received: 0,
                packets_lost: 0,
                average_latency: 0.0,
                quality_score: 100.0,
            });
        }

        // 更新当前指标
        self.current_metrics.packets_sent += 1;
        self.current_metrics.bytes_sent += bytes;
    }

    /// 记录接收的数据包
    pub fn record_packet_received(&mut self, connection_id: u64, bytes: u64, latency_ms: f32) {
        if !self.enabled {
            return;
        }

        // 更新包丢失监控器
        self.packet_loss_monitor.record_received(bytes);

        // 更新延迟监控器
        self.latency_monitor.add_sample(latency_ms);

        // 更新连接统计
        if let Some(stats) = self.connection_stats.get_mut(&connection_id) {
            stats.packets_received += 1;
            stats.last_activity = Instant::now();
            
            // 更新平均延迟
            let total_packets = stats.packets_sent + stats.packets_received;
            if total_packets > 0 {
                stats.average_latency = (stats.average_latency * (total_packets - 1) as f32 + latency_ms) / total_packets as f32;
            }
        }

        // 更新当前指标
        self.current_metrics.packets_received += 1;
        self.current_metrics.bytes_received += bytes;
    }

    /// 记录丢失的数据包
    pub fn record_packet_lost(&mut self, connection_id: u64) {
        if !self.enabled {
            return;
        }

        // 更新包丢失监控器
        self.packet_loss_monitor.record_lost();

        // 更新连接统计
        if let Some(stats) = self.connection_stats.get_mut(&connection_id) {
            stats.packets_lost += 1;
            stats.last_activity = Instant::now();
        }

        // 更新当前指标
        self.current_metrics.packets_lost += 1;
    }

    /// 记录重传的数据包
    pub fn record_packet_retransmitted(&mut self, _connection_id: u64) {
        if !self.enabled {
            return;
        }

        // 更新当前指标
        self.current_metrics.packets_retransmitted += 1;
    }

    /// 记录网络错误
    pub fn record_error(&mut self) {
        if !self.enabled {
            return;
        }

        self.current_metrics.error_count += 1;
    }

    /// 记录网络警告
    pub fn record_warning(&mut self) {
        if !self.enabled {
            return;
        }

        self.current_metrics.warning_count += 1;
    }

    /// 获取当前性能指标
    pub fn get_current_metrics(&self) -> &NetworkPerformanceMetrics {
        &self.current_metrics
    }

    /// 获取性能指标历史
    pub fn get_performance_history(&self) -> Vec<NetworkPerformanceMetrics> {
        self.performance_history.iter().cloned().collect()
    }

    /// 获取连接统计
    pub fn get_connection_stats(&self) -> &HashMap<u64, ConnectionPerformanceStats> {
        &self.connection_stats
    }

    /// 获取带宽使用率
    pub fn get_bandwidth_utilization(&self) -> (f32, f32) {
        (
            self.bandwidth_monitor.current_upload_kbps,
            self.bandwidth_monitor.current_download_kbps,
        )
    }

    /// 获取网络质量评分
    pub fn get_quality_score(&self) -> f32 {
        self.quality_assessor.current_score
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 设置启用状态
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// 重置监控器
    pub fn reset(&mut self) {
        self.performance_history.clear();
        self.current_metrics = NetworkPerformanceMetrics::default();
        self.connection_stats.clear();
        self.bandwidth_monitor.reset();
        self.latency_monitor.reset();
        self.packet_loss_monitor.reset();
        self.quality_assessor.reset();
        self.last_update = Instant::now();
    }

    /// 生成性能报告
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let uptime = self.start_time.elapsed();
        let avg_upload = self.performance_history.iter()
            .map(|m| m.upload_bandwidth_kbps)
            .sum::<f32>() / self.performance_history.len().max(1) as f32;
        let avg_download = self.performance_history.iter()
            .map(|m| m.download_bandwidth_kbps)
            .sum::<f32>() / self.performance_history.len().max(1) as f32;
        let avg_latency = self.performance_history.iter()
            .map(|m| m.average_latency_ms)
            .sum::<f32>() / self.performance_history.len().max(1) as f32;
        let avg_loss_rate = self.performance_history.iter()
            .map(|m| m.packet_loss_rate)
            .sum::<f32>() / self.performance_history.len().max(1) as f32;
        let avg_quality = self.performance_history.iter()
            .map(|m| m.quality_score)
            .sum::<f32>() / self.performance_history.len().max(1) as f32;

        PerformanceReport {
            uptime,
            total_packets_sent: self.current_metrics.packets_sent,
            total_packets_received: self.current_metrics.packets_received,
            total_packets_lost: self.current_metrics.packets_lost,
            total_bytes_sent: self.current_metrics.bytes_sent,
            total_bytes_received: self.current_metrics.bytes_received,
            average_upload_bandwidth_kbps: avg_upload,
            average_download_bandwidth_kbps: avg_download,
            average_latency_ms: avg_latency,
            average_packet_loss_rate: avg_loss_rate,
            average_quality_score: avg_quality,
            peak_upload_bandwidth_kbps: self.bandwidth_monitor.peak_upload_kbps,
            peak_download_bandwidth_kbps: self.bandwidth_monitor.peak_download_kbps,
            active_connections: self.connection_stats.len() as u32,
            total_errors: self.current_metrics.error_count,
            total_warnings: self.current_metrics.warning_count,
        }
    }

    /// 检查性能警告
    pub fn check_performance_warnings(&self) -> Vec<PerformanceWarning> {
        let mut warnings = Vec::new();

        // 检查延迟
        if self.current_metrics.average_latency_ms > self.config.performance_thresholds.high_latency_threshold {
            warnings.push(PerformanceWarning::HighLatency {
                current: self.current_metrics.average_latency_ms,
                threshold: self.config.performance_thresholds.high_latency_threshold,
            });
        }

        // 检查丢包率
        if self.current_metrics.packet_loss_rate > self.config.performance_thresholds.high_packet_loss_threshold {
            warnings.push(PerformanceWarning::HighPacketLoss {
                current: self.current_metrics.packet_loss_rate,
                threshold: self.config.performance_thresholds.high_packet_loss_threshold,
            });
        }

        // 检查带宽
        if self.current_metrics.upload_bandwidth_kbps < self.config.performance_thresholds.low_bandwidth_threshold &&
           self.current_metrics.download_bandwidth_kbps < self.config.performance_thresholds.low_bandwidth_threshold {
            warnings.push(PerformanceWarning::LowBandwidth {
                upload: self.current_metrics.upload_bandwidth_kbps,
                download: self.current_metrics.download_bandwidth_kbps,
                threshold: self.config.performance_thresholds.low_bandwidth_threshold,
            });
        }

        // 检查抖动
        if self.current_metrics.latency_jitter_ms > self.config.performance_thresholds.high_jitter_threshold {
            warnings.push(PerformanceWarning::HighJitter {
                current: self.current_metrics.latency_jitter_ms,
                threshold: self.config.performance_thresholds.high_jitter_threshold,
            });
        }

        warnings
    }

    // 私有方法

    /// 更新带宽监控器
    fn update_bandwidth_monitor(&mut self, delta_time: Duration) {
        let bytes_sent = self.current_metrics.bytes_sent;
        let bytes_received = self.current_metrics.bytes_received;
        
        self.bandwidth_monitor.update(bytes_sent, bytes_received, delta_time);
        
        self.current_metrics.upload_bandwidth_kbps = self.bandwidth_monitor.current_upload_kbps;
        self.current_metrics.download_bandwidth_kbps = self.bandwidth_monitor.current_download_kbps;
    }

    /// 更新延迟监控器
    fn update_latency_monitor(&mut self) {
        self.current_metrics.average_latency_ms = self.latency_monitor.current_average_ms;
        self.current_metrics.min_latency_ms = self.latency_monitor.current_min_ms;
        self.current_metrics.max_latency_ms = self.latency_monitor.current_max_ms;
        self.current_metrics.latency_jitter_ms = self.latency_monitor.current_jitter_ms;
    }

    /// 更新包丢失监控器
    fn update_packet_loss_monitor(&mut self) {
        self.current_metrics.packet_loss_rate = self.packet_loss_monitor.current_loss_rate;
    }

    /// 更新质量评估
    fn update_quality_assessment(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.quality_assessor.last_assessment) >= self.quality_assessor.assessment_interval {
            self.quality_assessor.assess_quality(
                self.current_metrics.average_latency_ms,
                self.current_metrics.packet_loss_rate,
                self.current_metrics.upload_bandwidth_kbps + self.current_metrics.download_bandwidth_kbps,
                self.current_metrics.latency_jitter_ms,
            );
            
            self.current_metrics.quality_score = self.quality_assessor.current_score;
        }
    }

    /// 生成当前指标
    fn generate_current_metrics(&mut self) {
        self.current_metrics.timestamp_ms = current_timestamp_ms();
        self.current_metrics.active_connections = self.connection_stats.len() as u32;
    }

    /// 清理过期的连接统计
    fn cleanup_expired_connections(&mut self) {
        let now = Instant::now();
        let timeout = Duration::from_secs(300); // 5分钟超时
        
        self.connection_stats.retain(|_, stats| {
            now.duration_since(stats.last_activity) < timeout
        });
    }

    /// 检查是否活跃
    pub fn is_active(&self) -> bool {
        self.enabled && !self.connection_stats.is_empty()
    }
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// 运行时间
    pub uptime: Duration,
    /// 总发送包数
    pub total_packets_sent: u64,
    /// 总接收包数
    pub total_packets_received: u64,
    /// 总丢失包数
    pub total_packets_lost: u64,
    /// 总发送字节数
    pub total_bytes_sent: u64,
    /// 总接收字节数
    pub total_bytes_received: u64,
    /// 平均上行带宽（Kbps）
    pub average_upload_bandwidth_kbps: f32,
    /// 平均下行带宽（Kbps）
    pub average_download_bandwidth_kbps: f32,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f32,
    /// 平均丢包率
    pub average_packet_loss_rate: f32,
    /// 平均质量评分
    pub average_quality_score: f32,
    /// 峰值上行带宽（Kbps）
    pub peak_upload_bandwidth_kbps: f32,
    /// 峰值下行带宽（Kbps）
    pub peak_download_bandwidth_kbps: f32,
    /// 活跃连接数
    pub active_connections: u32,
    /// 总错误数
    pub total_errors: u32,
    /// 总警告数
    pub total_warnings: u32,
}

/// 性能警告
#[derive(Debug, Clone)]
pub enum PerformanceWarning {
    /// 高延迟
    HighLatency { current: f32, threshold: f32 },
    /// 高丢包率
    HighPacketLoss { current: f32, threshold: f32 },
    /// 低带宽
    LowBandwidth { upload: f32, download: f32, threshold: f32 },
    /// 高抖动
    HighJitter { current: f32, threshold: f32 },
}

impl Default for NetworkPerformanceMetrics {
    fn default() -> Self {
        Self {
            timestamp_ms: current_timestamp_ms(),
            upload_bandwidth_kbps: 0.0,
            download_bandwidth_kbps: 0.0,
            average_latency_ms: 0.0,
            min_latency_ms: f32::MAX,
            max_latency_ms: 0.0,
            latency_jitter_ms: 0.0,
            packet_loss_rate: 0.0,
            packets_sent: 0,
            packets_received: 0,
            packets_lost: 0,
            packets_retransmitted: 0,
            bytes_sent: 0,
            bytes_received: 0,
            quality_score: 100.0,
            active_connections: 0,
            error_count: 0,
            warning_count: 0,
        }
    }
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            update_interval_ms: 100, // 100ms
            history_retention_s: 300, // 5分钟
            detailed_monitoring: true,
            bandwidth_sample_window: 10,
            latency_sample_window: 50,
            packet_loss_sample_window: 100,
            quality_assessment_interval_s: 1,
            performance_thresholds: PerformanceThresholds::default(),
        }
    }
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            high_latency_threshold: 150.0, // 150ms
            high_packet_loss_threshold: 0.05, // 5%
            low_bandwidth_threshold: 100.0, // 100Kbps
            high_jitter_threshold: 30.0, // 30ms
        }
    }
}

// 带宽监控器实现
impl BandwidthMonitor {
    fn new(window_size: usize) -> Self {
        Self {
            upload_samples: VecDeque::with_capacity(window_size),
            download_samples: VecDeque::with_capacity(window_size),
            window_size,
            current_upload_kbps: 0.0,
            current_download_kbps: 0.0,
            peak_upload_kbps: 0.0,
            peak_download_kbps: 0.0,
        }
    }

    fn update(&mut self, bytes_sent: u64, bytes_received: u64, delta_time: Duration) {
        let now = Instant::now();
        let delta_seconds = delta_time.as_secs_f32();
        
        if delta_seconds > 0.0 {
            let upload_kbps = (bytes_sent as f32 * 8.0) / (delta_seconds * 1000.0);
            let download_kbps = (bytes_received as f32 * 8.0) / (delta_seconds * 1000.0);
            
            self.upload_samples.push_back(BandwidthSample { timestamp: now, bytes: bytes_sent });
            self.download_samples.push_back(BandwidthSample { timestamp: now, bytes: bytes_received });
            
            // 限制窗口大小
            while self.upload_samples.len() > self.window_size {
                self.upload_samples.pop_front();
            }
            while self.download_samples.len() > self.window_size {
                self.download_samples.pop_front();
            }
            
            self.current_upload_kbps = upload_kbps;
            self.current_download_kbps = download_kbps;
            
            self.peak_upload_kbps = self.peak_upload_kbps.max(upload_kbps);
            self.peak_download_kbps = self.peak_download_kbps.max(download_kbps);
        }
    }

    fn reset(&mut self) {
        self.upload_samples.clear();
        self.download_samples.clear();
        self.current_upload_kbps = 0.0;
        self.current_download_kbps = 0.0;
        self.peak_upload_kbps = 0.0;
        self.peak_download_kbps = 0.0;
    }
}

// 延迟监控器实现
impl LatencyMonitor {
    fn new(window_size: usize) -> Self {
        Self {
            latency_samples: VecDeque::with_capacity(window_size),
            window_size,
            current_average_ms: 0.0,
            current_min_ms: f32::MAX,
            current_max_ms: 0.0,
            current_jitter_ms: 0.0,
        }
    }

    fn add_sample(&mut self, latency_ms: f32) {
        let now = Instant::now();
        self.latency_samples.push_back(LatencySample { timestamp: now, latency_ms });
        
        // 限制窗口大小
        while self.latency_samples.len() > self.window_size {
            self.latency_samples.pop_front();
        }
        
        // 计算统计指标
        if !self.latency_samples.is_empty() {
            let latencies: Vec<f32> = self.latency_samples.iter().map(|s| s.latency_ms).collect();
            self.current_average_ms = latencies.iter().sum::<f32>() / latencies.len() as f32;
            self.current_min_ms = latencies.iter().fold(f32::MAX, |a, &b| a.min(b));
            self.current_max_ms = latencies.iter().fold(0.0, |a, &b| a.max(b));
            
            // 计算抖动（标准差）
            if latencies.len() > 1 {
                let variance = latencies.iter()
                    .map(|&latency| {
                        let diff = latency - self.current_average_ms;
                        diff * diff
                    })
                    .sum::<f32>() / (latencies.len() - 1) as f32;
                self.current_jitter_ms = variance.sqrt();
            } else {
                self.current_jitter_ms = 0.0;
            }
        }
    }

    fn reset(&mut self) {
        self.latency_samples.clear();
        self.current_average_ms = 0.0;
        self.current_min_ms = f32::MAX;
        self.current_max_ms = 0.0;
        self.current_jitter_ms = 0.0;
    }
}

// 包丢失监控器实现
impl PacketLossMonitor {
    fn new(history_window: usize) -> Self {
        Self {
            packets_sent: 0,
            packets_received: 0,
            packets_lost: 0,
            loss_rate_history: VecDeque::with_capacity(history_window),
            history_window,
            current_loss_rate: 0.0,
        }
    }

    fn record_sent(&mut self, count: u64) {
        self.packets_sent += count;
        self.update_loss_rate();
    }

    fn record_received(&mut self, count: u64) {
        self.packets_received += count;
        self.update_loss_rate();
    }

    fn record_lost(&mut self) {
        self.packets_lost += 1;
        self.update_loss_rate();
    }

    fn update_loss_rate(&mut self) {
        if self.packets_sent > 0 {
            let loss_rate = self.packets_lost as f32 / self.packets_sent as f32;
            self.loss_rate_history.push_back(loss_rate);
            
            // 限制历史窗口
            while self.loss_rate_history.len() > self.history_window {
                self.loss_rate_history.pop_front();
            }
            
            // 计算当前平均丢包率
            if !self.loss_rate_history.is_empty() {
                self.current_loss_rate = self.loss_rate_history.iter().sum::<f32>() / self.loss_rate_history.len() as f32;
            }
        }
    }

    fn reset(&mut self) {
        self.packets_sent = 0;
        self.packets_received = 0;
        self.packets_lost = 0;
        self.loss_rate_history.clear();
        self.current_loss_rate = 0.0;
    }
}

// 网络质量评估器实现
impl NetworkQualityAssessor {
    fn new() -> Self {
        Self {
            last_assessment: Instant::now(),
            assessment_interval: Duration::from_secs(1),
            current_score: 100.0,
            quality_history: VecDeque::with_capacity(60), // 保留1分钟的历史
            history_window: 60,
        }
    }

    fn assess_quality(&mut self, latency_ms: f32, packet_loss_rate: f32, bandwidth_kbps: f32, jitter_ms: f32) {
        let now = Instant::now();
        self.last_assessment = now;
        
        // 计算各因素得分（0-100）
        let latency_factor = self.calculate_latency_factor(latency_ms);
        let packet_loss_factor = self.calculate_packet_loss_factor(packet_loss_rate);
        let bandwidth_factor = self.calculate_bandwidth_factor(bandwidth_kbps);
        let jitter_factor = self.calculate_jitter_factor(jitter_ms);
        
        // 加权计算总分
        let factors = QualityFactors {
            latency_factor,
            packet_loss_factor,
            bandwidth_factor,
            jitter_factor,
        };
        
        self.current_score = latency_factor * 0.3 + 
                           packet_loss_factor * 0.3 + 
                           bandwidth_factor * 0.2 + 
                           jitter_factor * 0.2;
        
        // 添加到历史
        self.quality_history.push_back(QualitySample {
            timestamp: now,
            score: self.current_score,
            factors,
        });
        
        // 限制历史窗口
        while self.quality_history.len() > self.history_window {
            self.quality_history.pop_front();
        }
    }

    fn calculate_latency_factor(&self, latency_ms: f32) -> f32 {
        // 延迟越低，得分越高
        if latency_ms <= 50.0 {
            100.0
        } else if latency_ms <= 100.0 {
            100.0 - (latency_ms - 50.0) * 0.5
        } else if latency_ms <= 200.0 {
            75.0 - (latency_ms - 100.0) * 0.25
        } else {
            50.0 - (latency_ms - 200.0) * 0.1
        }.max(0.0)
    }

    fn calculate_packet_loss_factor(&self, packet_loss_rate: f32) -> f32 {
        // 丢包率越低，得分越高
        if packet_loss_rate <= 0.01 {
            100.0
        } else if packet_loss_rate <= 0.05 {
            100.0 - (packet_loss_rate - 0.01) * 1000.0
        } else if packet_loss_rate <= 0.1 {
            60.0 - (packet_loss_rate - 0.05) * 400.0
        } else {
            40.0 - (packet_loss_rate - 0.1) * 200.0
        }.max(0.0)
    }

    fn calculate_bandwidth_factor(&self, bandwidth_kbps: f32) -> f32 {
        // 带宽越高，得分越高
        if bandwidth_kbps >= 1000.0 {
            100.0
        } else if bandwidth_kbps >= 500.0 {
            50.0 + (bandwidth_kbps - 500.0) * 0.1
        } else if bandwidth_kbps >= 100.0 {
            20.0 + (bandwidth_kbps - 100.0) * 0.06
        } else {
            bandwidth_kbps * 0.2
        }.max(0.0)
    }

    fn calculate_jitter_factor(&self, jitter_ms: f32) -> f32 {
        // 抖动越低，得分越高
        if jitter_ms <= 5.0 {
            100.0
        } else if jitter_ms <= 20.0 {
            100.0 - (jitter_ms - 5.0) * 2.0
        } else if jitter_ms <= 50.0 {
            70.0 - (jitter_ms - 20.0) * 1.0
        } else {
            40.0 - (jitter_ms - 50.0) * 0.2
        }.max(0.0)
    }

    fn reset(&mut self) {
        self.current_score = 100.0;
        self.quality_history.clear();
        self.last_assessment = Instant::now();
    }
}

impl Default for NetworkPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = NetworkPerformanceMonitor::new();
        assert!(monitor.is_enabled());
        assert_eq!(monitor.get_current_metrics().packets_sent, 0);
        assert_eq!(monitor.get_current_metrics().packets_received, 0);
    }

    #[test]
    fn test_packet_recording() {
        let mut monitor = NetworkPerformanceMonitor::new();
        
        // 记录发送包
        monitor.record_packet_sent(1, 1024);
        assert_eq!(monitor.get_current_metrics().packets_sent, 1);
        assert_eq!(monitor.get_current_metrics().bytes_sent, 1024);
        
        // 记录接收包
        monitor.record_packet_received(1, 512, 50.0);
        assert_eq!(monitor.get_current_metrics().packets_received, 1);
        assert_eq!(monitor.get_current_metrics().bytes_received, 512);
        
        // 记录丢失包
        monitor.record_packet_lost(1);
        assert_eq!(monitor.get_current_metrics().packets_lost, 1);
    }

    #[test]
    fn test_performance_warnings() {
        let mut monitor = NetworkPerformanceMonitor::new();
        
        // 设置高延迟
        monitor.record_packet_received(1, 1024, 200.0); // 超过默认阈值150ms
        
        let warnings = monitor.check_performance_warnings();
        assert!(!warnings.is_empty());
        
        match &warnings[0] {
            PerformanceWarning::HighLatency { current, threshold } => {
                assert_eq!(*current, 200.0);
                assert_eq!(*threshold, 150.0);
            }
            _ => panic!("Expected HighLatency warning"),
        }
    }

    #[test]
    fn test_performance_report() {
        let mut monitor = NetworkPerformanceMonitor::new();
        
        monitor.record_packet_sent(1, 1024);
        monitor.record_packet_received(1, 512, 50.0);
        monitor.record_packet_lost(1);
        
        let report = monitor.generate_performance_report();
        assert_eq!(report.total_packets_sent, 1);
        assert_eq!(report.total_packets_received, 1);
        assert_eq!(report.total_packets_lost, 1);
        assert_eq!(report.total_bytes_sent, 1024);
        assert_eq!(report.total_bytes_received, 512);
    }
}