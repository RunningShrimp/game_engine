//  统一网络优化管理器
//
//  整合包丢失恢复、带宽管理和插值优化功能，简化架构。
//
//  ## 架构改进
//
//  将原来的 PacketRecoveryManager 和 BandwidthManager 合并为统一的 NetworkOptimizationManager，
//  同时集成客户端插值功能，提供一站式的网络性能优化。
//
//  ## 设计优势
//
//  1. **统一策略**: 根据网络质量统一调整恢复和带宽策略
//  2. **减少开销**: 共享网络质量监测和统计
//  3. **简化调用**: 客户端通过单一接口管理所有网络优化
//  4. **更好的协调**: 恢复和带宽策略可以协同优化

use crate::network::NetworkError;
use glam::Vec3;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// 网络质量指标
#[derive(Debug, Clone)]
pub struct NetworkQualityMetrics {
    /// 当前延迟（毫秒）
    pub latency_ms: f32,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f32,
    /// 抖动（延迟变化）
    pub jitter_ms: f32,
    /// 包丢失率（0.0-1.0）
    pub packet_loss_rate: f32,
    /// 带宽（字节/秒）
    pub bandwidth_bps: f64,
    /// 可用带宽（字节/秒）
    pub available_bandwidth_bps: f64,
    /// 测量时间戳
    pub measured_at: Instant,
}

impl Default for NetworkQualityMetrics {
    fn default() -> Self {
        Self {
            latency_ms: 0.0,
            average_latency_ms: 0.0,
            jitter_ms: 0.0,
            packet_loss_rate: 0.0,
            bandwidth_bps: 0.0,
            available_bandwidth_bps: 0.0,
            measured_at: Instant::now(),
        }
    }
}

impl NetworkQualityMetrics {
    /// 计算网络质量评分（0-100）
    pub fn quality_score(&self) -> f32 {
        let latency_score = (100.0 - self.latency_ms.min(200.0) / 2.0).max(0.0);
        let loss_score = (100.0 * (1.0 - self.packet_loss_rate)).max(0.0);
        let bandwidth_score = ((self.available_bandwidth_bps / 10000.0) as f32).min(100.0);

        latency_score * 0.4 + loss_score * 0.4 + bandwidth_score * 0.2
    }

    /// 是否网络良好
    pub fn is_good(&self) -> bool {
        self.quality_score() >= 70.0
    }

    /// 是否网络较差
    pub fn is_poor(&self) -> bool {
        self.quality_score() < 40.0
    }
}

/// 包丢失恢复策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketLossRecoveryStrategy {
    /// 无恢复
    None,
    /// 选择性重传
    SelectiveRepeat,
    /// 前向纠错（FEC）
    ForwardErrorCorrection,
    /// 冗余发送
    RedundantTransmission,
    /// 混合策略
    Hybrid,
}

/// 待确认的包
#[derive(Debug, Clone)]
pub struct PendingPacket {
    /// 包数据
    data: Vec<u8>,
    /// 发送时间
    sent_at: Instant,
    /// 重传次数
    retransmission_count: u32,
    /// 是否使用FEC
    use_fec: bool,
}

/// 插值状态
#[derive(Debug, Clone)]
pub struct InterpolationState {
    /// 位置历史
    position_history: VecDeque<(u64, Vec3)>,
    /// 速度历史
    velocity_history: VecDeque<Vec3>,
    /// 最后更新时间
    last_update: Instant,
    /// 当前插值位置
    current_position: Vec3,
    /// 当前速度
    current_velocity: Vec3,
}

/// 网络优化配置
#[derive(Debug, Clone)]
pub struct NetworkOptimizationConfig {
    // 包恢复配置
    /// 恢复策略
    pub recovery_strategy: PacketLossRecoveryStrategy,
    /// 冗余度（0.0-1.0）
    pub redundancy_rate: f32,
    /// FEC块大小
    pub fec_block_size: usize,
    /// 最大重传次数
    pub max_retransmissions: u32,
    /// 重传超时（毫秒）
    pub retransmission_timeout_ms: u64,

    // 带宽配置
    /// 总带宽预算（字节/秒）
    pub total_bandwidth_bps: f64,
    /// 高优先级保留比例
    pub high_priority_reserve: f32,
    /// 中优先级保留比例
    pub medium_priority_reserve: f32,
    /// 低优先级保留比例
    pub low_priority_reserve: f32,

    // 插值配置
    /// 插值缓冲区大小（帧数）
    pub interpolation_buffer_size: usize,
    /// 插值延迟（毫秒）
    pub interpolation_delay_ms: u64,
    /// 最大外推时间（毫秒）
    pub max_extrapolation_ms: u64,
    /// 是否启用速度匹配
    pub enable_velocity_matching: bool,
    /// 是否启用外推
    pub enable_extrapolation: bool,
}

impl Default for NetworkOptimizationConfig {
    fn default() -> Self {
        Self {
            recovery_strategy: PacketLossRecoveryStrategy::Hybrid,
            redundancy_rate: 0.2,
            fec_block_size: 10,
            max_retransmissions: 3,
            retransmission_timeout_ms: 100,
            total_bandwidth_bps: 100000.0,
            high_priority_reserve: 0.5,
            medium_priority_reserve: 0.3,
            low_priority_reserve: 0.2,
            interpolation_buffer_size: 64,
            interpolation_delay_ms: 100,
            max_extrapolation_ms: 500,
            enable_velocity_matching: true,
            enable_extrapolation: true,
        }
    }
}

/// 恢复统计
#[derive(Debug, Clone, Default)]
pub struct RecoveryStats {
    /// 发送的包总数
    pub total_packets_sent: u64,
    /// 重传的包数
    pub retransmitted_packets: u64,
    /// 恢复的包数
    pub recovered_packets: u64,
    /// 丢失的包数
    pub lost_packets: u64,
    /// 冗余包数
    pub redundant_packets: u64,
    /// FEC恢复的包数
    pub fec_recovered_packets: u64,
}

/// 带宽使用统计
#[derive(Debug, Clone, Default)]
pub struct OptimizationBandwidthStats {
    /// 当前使用量（字节/秒）
    pub current_usage: f64,
    /// 使用率（0.0-1.0）
    pub utilization_rate: f32,
    /// 高优先级使用量
    pub high_priority_usage: f64,
    /// 中优先级使用量
    pub medium_priority_usage: f64,
    /// 低优先级使用量
    pub low_priority_usage: f64,
}

/// 统一网络优化管理器
///
/// 整合包丢失恢复、带宽管理和客户端插值功能。
pub struct NetworkOptimizationManager {
    /// 配置
    config: NetworkOptimizationConfig,
    /// 网络质量
    network_quality: NetworkQualityMetrics,

    // 包恢复
    /// 待确认的包
    pending_packets: HashMap<u32, PendingPacket>,
    /// 包序号
    sequence_number: u32,
    /// 恢复统计
    recovery_stats: RecoveryStats,

    // 带宽管理
    /// 当前带宽使用量
    bandwidth_usage: f64,
    /// 优先级使用量
    priority_usage: HashMap<String, f64>,

    // 插值
    /// 实体插值状态
    entity_states: HashMap<u64, InterpolationState>,
}

impl NetworkOptimizationManager {
    /// 创建新的网络优化管理器
    pub fn new(config: NetworkOptimizationConfig) -> Self {
        Self {
            config,
            network_quality: NetworkQualityMetrics::default(),
            pending_packets: HashMap::new(),
            sequence_number: 0,
            recovery_stats: RecoveryStats::default(),
            bandwidth_usage: 0.0,
            priority_usage: HashMap::new(),
            entity_states: HashMap::new(),
        }
    }

    /// 使用默认配置创建
    pub fn default_config() -> Self {
        Self::new(NetworkOptimizationConfig::default())
    }

    // ========================================================================
    // 网络质量 API
    // ========================================================================

    /// 更新网络质量
    pub fn update_network_quality(&mut self, quality: NetworkQualityMetrics) {
        self.network_quality = quality.clone();

        // 根据网络质量动态调整策略
        if quality.is_poor() {
            // 网络差，增加冗余
            self.config.redundancy_rate = (self.config.redundancy_rate * 1.5).min(0.5);
            self.config.retransmission_timeout_ms = ((self.config.retransmission_timeout_ms as f64) * 2.0).min(1000.0) as u64;
            self.config.interpolation_delay_ms = ((self.config.interpolation_delay_ms as f64) * 3.0).min(500.0) as u64;
        } else if quality.is_good() {
            // 网络好，减少冗余
            self.config.redundancy_rate = (self.config.redundancy_rate * 0.8).max(0.1);
            self.config.retransmission_timeout_ms = ((self.config.retransmission_timeout_ms as f64) * 0.8).max(50.0) as u64;
            self.config.interpolation_delay_ms = ((self.config.interpolation_delay_ms as f64) * 0.8).max(50.0) as u64;
        }
    }

    /// 获取网络质量
    pub fn get_network_quality(&self) -> &NetworkQualityMetrics {
        &self.network_quality
    }

    // ========================================================================
    // 包恢复 API
    // ========================================================================

    /// 发送数据包（带恢复）
    pub fn send_packet(&mut self, data: Vec<u8>) -> Result<u32, NetworkError> {
        let seq = self.sequence_number;
        self.sequence_number = self.sequence_number.wrapping_add(1);

        let packet = PendingPacket {
            sent_at: Instant::now(),
            retransmission_count: 0,
            data: data.clone(),
            use_fec: matches!(self.config.recovery_strategy,
                PacketLossRecoveryStrategy::ForwardErrorCorrection |
                PacketLossRecoveryStrategy::Hybrid),
        };

        self.pending_packets.insert(seq, packet);
        self.recovery_stats.total_packets_sent += 1;

        // 根据策略决定是否发送冗余
        if matches!(self.config.recovery_strategy,
            PacketLossRecoveryStrategy::RedundantTransmission |
            PacketLossRecoveryStrategy::Hybrid)
            && rand::random::<f32>() < self.config.redundancy_rate {
                self.send_redundant_packet(seq, &data)?;
            }

        Ok(seq)
    }

    /// 发送冗余包
    fn send_redundant_packet(&mut self, _seq: u32, _data: &[u8]) -> Result<(), NetworkError> {
        self.recovery_stats.redundant_packets += 1;
        // 实际实现会通过网络发送
        Ok(())
    }

    /// 确认包接收
    pub fn acknowledge_packet(&mut self, seq: u32) {
        if let Some(_packet) = self.pending_packets.remove(&seq) {
            // 包成功确认
        }
    }

    /// 检查超时并重传
    pub fn check_timeouts(&mut self) -> Vec<Vec<u8>> {
        let now = Instant::now();
        let mut to_retransmit = Vec::new();
        let timeout = Duration::from_millis(self.config.retransmission_timeout_ms);

        self.pending_packets.retain(|_seq, packet| {
            if now.duration_since(packet.sent_at) > timeout {
                if packet.retransmission_count < self.config.max_retransmissions {
                    to_retransmit.push(packet.data.clone());
                    self.recovery_stats.retransmitted_packets += 1;
                    false // 保留在pending中
                } else {
                    // 超过最大重传次数，放弃
                    self.recovery_stats.lost_packets += 1;
                    false
                }
            } else {
                true
            }
        });

        to_retransmit
    }

    /// 获取恢复统计
    pub fn get_recovery_stats(&self) -> &RecoveryStats {
        &self.recovery_stats
    }

    // ========================================================================
    // 带宽管理 API
    // ========================================================================

    /// 请求带宽
    pub fn request_bandwidth(&mut self, priority: &str, size: usize) -> bool {
        let available = self.get_available_bandwidth(priority);
        let size_bps = size as f64;

        if available >= size_bps {
            *self.priority_usage.entry(priority.to_string()).or_insert(0.0) += size_bps;
            self.bandwidth_usage += size_bps;
            true
        } else {
            false
        }
    }

    /// 获取可用带宽
    pub fn get_available_bandwidth(&self, priority: &str) -> f64 {
        let reserve = match priority {
            "high" => self.config.high_priority_reserve,
            "medium" => self.config.medium_priority_reserve,
            "low" => self.config.low_priority_reserve,
            _ => 0.0,
        };

        let reserved = self.config.total_bandwidth_bps * reserve as f64;
        let used = self.priority_usage.get(priority).copied().unwrap_or(0.0);
        (reserved - used).max(0.0)
    }

    /// 重置带宽使用量（每秒调用）
    pub fn reset_bandwidth_usage(&mut self) {
        self.bandwidth_usage = 0.0;
        self.priority_usage.clear();
    }

    /// 获取带宽统计
    pub fn get_bandwidth_stats(&self) -> OptimizationBandwidthStats {
        OptimizationBandwidthStats {
            current_usage: self.bandwidth_usage,
            utilization_rate: (self.bandwidth_usage / self.config.total_bandwidth_bps) as f32,
            high_priority_usage: self.priority_usage.get("high").copied().unwrap_or(0.0),
            medium_priority_usage: self.priority_usage.get("medium").copied().unwrap_or(0.0),
            low_priority_usage: self.priority_usage.get("low").copied().unwrap_or(0.0),
        }
    }

    // ========================================================================
    // 客户端插值 API
    // ========================================================================

    /// 添加网络状态更新
    pub fn add_network_update(&mut self, entity_id: u64, tick: u64, position: Vec3, velocity: Vec3) {
        let state = self.entity_states.entry(entity_id).or_insert_with(|| {
            InterpolationState {
                position_history: VecDeque::with_capacity(self.config.interpolation_buffer_size),
                velocity_history: VecDeque::with_capacity(self.config.interpolation_buffer_size),
                last_update: Instant::now(),
                current_position: position,
                current_velocity: velocity,
            }
        });

        // 添加到历史
        state.position_history.push_back((tick, position));
        state.velocity_history.push_back(velocity);
        state.last_update = Instant::now();

        // 限制缓冲区大小
        if state.position_history.len() > self.config.interpolation_buffer_size {
            state.position_history.pop_front();
            state.velocity_history.pop_front();
        }
    }

    /// 获取插值位置
    pub fn get_interpolated_position(&mut self, entity_id: u64, target_tick: u64) -> Option<Vec3> {
        let state = self.entity_states.get_mut(&entity_id)?;

        // 如果历史不足，直接返回最新位置
        if state.position_history.len() < 2 {
            return Some(state.current_position);
        }

        // 计算插值时间
        let now = Instant::now();
        let delay = Duration::from_millis(self.config.interpolation_delay_ms);
        let elapsed = now.duration_since(state.last_update);

        // 如果网络延迟高，使用外推
        if elapsed > delay
            && self.config.enable_extrapolation {
                let extrapolation_time = elapsed - delay;
                let max_time = Duration::from_millis(self.config.max_extrapolation_ms);

                if extrapolation_time < max_time {
                    // 外推：位置 += 速度 * 时间
                    let dt = extrapolation_time.as_secs_f32();
                    state.current_position += state.current_velocity * dt;
                    return Some(state.current_position);
                } else {
                    // 外推超时，返回最后已知位置
                    return Some(state.current_position);
                }
            }

        // 找到插值的两个关键帧
        let front_entry = state.position_history.front()?;
        let back_entry = state.position_history.back()?;
        let (tick1, pos1) = *front_entry;
        let (tick2, pos2) = *back_entry;

        if target_tick <= tick1 {
            return Some(pos1);
        }
        if target_tick >= tick2 {
            return Some(pos2);
        }

        // 线性插值
        let t = (target_tick - tick1) as f32 / (tick2 - tick1) as f32;
        let interpolated = pos1.lerp(pos2, t);

        state.current_position = interpolated;
        Some(interpolated)
    }

    /// 清理不活跃的实体
    pub fn cleanup_inactive_entities(&mut self, timeout: Duration) {
        let now = Instant::now();
        self.entity_states.retain(|_, state| {
            now.duration_since(state.last_update) < timeout
        });
    }

    // ========================================================================
    // 统计和调试
    // ========================================================================

    /// 获取综合统计信息
    pub fn get_stats(&self) -> NetworkOptimizationStats {
        NetworkOptimizationStats {
            recovery: self.recovery_stats.clone(),
            bandwidth: self.get_bandwidth_stats(),
            network_quality: self.network_quality.clone(),
            interpolated_entities: self.entity_states.len(),
            pending_packets: self.pending_packets.len(),
        }
    }
}

/// 综合网络优化统计
#[derive(Debug, Clone)]
pub struct NetworkOptimizationStats {
    /// 恢复统计
    pub recovery: RecoveryStats,
    /// 带宽统计
    pub bandwidth: OptimizationBandwidthStats,
    /// 网络质量
    pub network_quality: NetworkQualityMetrics,
    /// 插值实体数量
    pub interpolated_entities: usize,
    /// 待确认包数量
    pub pending_packets: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_quality_score() {
        let quality = NetworkQualityMetrics {
            latency_ms: 50.0,
            packet_loss_rate: 0.01,
            available_bandwidth_bps: 50000.0,
            ..Default::default()
        };

        let score = quality.quality_score();
        assert!(score > 70.0);
        assert!(quality.is_good());
        assert!(!quality.is_poor());
    }

    #[test]
    fn test_packet_recovery() {
        let mut manager = NetworkOptimizationManager::default_config();
        let data = vec![1, 2, 3, 4];

        let seq = manager.send_packet(data.clone()).unwrap();
        assert!(manager.pending_packets.contains_key(&seq));

        manager.acknowledge_packet(seq);
        assert!(!manager.pending_packets.contains_key(&seq));

        let stats = manager.get_recovery_stats();
        assert_eq!(stats.total_packets_sent, 1);
    }

    #[test]
    fn test_bandwidth_management() {
        let mut manager = NetworkOptimizationManager::default_config();

        // 请求带宽
        assert!(manager.request_bandwidth("high", 1000));
        assert!(manager.request_bandwidth("high", 40000));

        let stats = manager.get_bandwidth_stats();
        assert!(stats.utilization_rate > 0.0);
    }

    #[test]
    fn test_interpolation() {
        let mut manager = NetworkOptimizationManager::default_config();

        // 添加网络更新
        manager.add_network_update(1, 0, Vec3::new(0.0, 0.0, 0.0), Vec3::ZERO);
        manager.add_network_update(1, 1, Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
        manager.add_network_update(1, 2, Vec3::new(2.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));

        // 获取插值位置
        let pos = manager.get_interpolated_position(1, 1);
        assert!(pos.is_some());
    }

    #[test]
    fn test_comprehensive_stats() {
        let manager = NetworkOptimizationManager::default_config();
        let stats = manager.get_stats();

        assert_eq!(stats.recovery.total_packets_sent, 0);
        assert_eq!(stats.interpolated_entities, 0);
        assert_eq!(stats.pending_packets, 0);
    }
}
