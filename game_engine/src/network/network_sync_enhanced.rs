//  增强网络同步系统
//
//  集成优先级同步、包丢失恢复和客户端插值的高级网络同步系统：
//  - 智能优先级同步
//  - 包丢失恢复策略
//  - 客户端插值优化
//  - 网络质量自适应
//  - 性能监控和报告

use crate::network::delta_serialization::{DeltaPacket, EntityDelta};
use crate::network::priority_sync::{BandwidthStats, PrioritySyncManager, SyncPriority};
use glam::Vec3;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

/// 增强网络同步配置
#[derive(Debug, Clone)]
pub struct EnhancedNetworkSyncConfig {
    /// 是否启用包丢失恢复
    pub enable_packet_recovery: bool,
    /// 是否启用客户端插值
    pub enable_client_interpolation: bool,
    /// 插值延迟（毫秒）
    pub interpolation_delay_ms: u32,
    /// 最大重传次数
    pub max_retransmissions: u8,
    /// 重传超时（毫秒）
    pub retransmission_timeout_ms: u32,
    /// 网络质量检测间隔（帧数）
    pub quality_check_interval: u32,
    /// 自适应质量调整
    pub enable_adaptive_quality: bool,
}

impl Default for EnhancedNetworkSyncConfig {
    fn default() -> Self {
        Self {
            enable_packet_recovery: true,
            enable_client_interpolation: true,
            interpolation_delay_ms: 100, // 100ms插值延迟
            max_retransmissions: 3,
            retransmission_timeout_ms: 500,
            quality_check_interval: 60,
            enable_adaptive_quality: true,
        }
    }
}

/// 网络质量指标
#[derive(Debug, Clone)]
pub struct NetworkQuality {
    /// 延迟（毫秒）
    pub latency_ms: f32,
    /// 抖动（毫秒）
    pub jitter_ms: f32,
    /// 包丢失率（0.0-1.0）
    pub packet_loss_rate: f32,
    /// 带宽使用率（0.0-1.0）
    pub bandwidth_usage_ratio: f32,
    /// 质量评分（0.0-1.0，越高越好）
    pub quality_score: f32,
    /// 最后更新时间
    pub last_update: Instant,
}

impl Default for NetworkQuality {
    fn default() -> Self {
        Self {
            latency_ms: 0.0,
            jitter_ms: 0.0,
            packet_loss_rate: 0.0,
            bandwidth_usage_ratio: 0.0,
            quality_score: 1.0,
            last_update: Instant::now(),
        }
    }
}

impl NetworkQuality {
    /// 计算质量评分
    pub fn calculate_score(&mut self) {
        // 延迟评分（0-100ms = 1.0, 100-300ms = 0.5, >300ms = 0.1）
        let latency_score = if self.latency_ms < 100.0 {
            1.0
        } else if self.latency_ms < 300.0 {
            0.5
        } else {
            0.1
        };

        // 抖动评分
        let jitter_score = if self.jitter_ms < 20.0 {
            1.0
        } else if self.jitter_ms < 50.0 {
            0.6
        } else {
            0.2
        };

        // 包丢失评分
        let loss_score = 1.0 - self.packet_loss_rate;

        // 带宽使用评分（保持适中，不接近满载）
        let bandwidth_score = if self.bandwidth_usage_ratio < 0.5 {
            1.0
        } else if self.bandwidth_usage_ratio < 0.8 {
            0.8
        } else {
            0.4
        };

        // 综合评分（加权平均）
        self.quality_score = latency_score * 0.3
            + jitter_score * 0.2
            + loss_score * 0.3
            + bandwidth_score * 0.2;

        self.last_update = Instant::now();
    }

    /// 获取质量等级
    pub fn get_quality_level(&self) -> QualityLevel {
        if self.quality_score > 0.8 {
            QualityLevel::Excellent
        } else if self.quality_score > 0.6 {
            QualityLevel::Good
        } else if self.quality_score > 0.4 {
            QualityLevel::Fair
        } else if self.quality_score > 0.2 {
            QualityLevel::Poor
        } else {
            QualityLevel::Terrible
        }
    }
}

/// 网络质量等级
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    /// 优秀 (0.8-1.0)
    Excellent,
    /// 良好 (0.6-0.8)
    Good,
    /// 一般 (0.4-0.6)
    Fair,
    /// 较差 (0.2-0.4)
    Poor,
    /// 很差 (0.0-0.2)
    Terrible,
}

/// 包丢失恢复策略
#[derive(Debug, Clone)]
pub struct PacketRecoveryStrategy {
    /// 待重传的数据包
    pending_retransmissions: HashMap<u64, RetransmissionInfo>,
    /// 已确认的数据包
    acknowledged_packets: VecDeque<u64>,
    /// 最大确认队列大小
    max_ack_queue_size: usize,
    /// 重传统计
    retransmission_stats: RetransmissionStats,
}

/// 重传信息
#[derive(Debug, Clone)]
struct RetransmissionInfo {
    /// 数据包
    packet: DeltaPacket,
    /// 首次发送时间
    first_sent_time: Instant,
    /// 重传次数
    retransmission_count: u8,
    /// 最后重传时间
    last_retransmission_time: Instant,
}

/// 重传统计
#[derive(Debug, Clone, Default)]
pub struct RetransmissionStats {
    /// 发送的数据包总数
    pub total_packets_sent: u64,
    /// 重传的数据包数
    pub retransmitted_packets: u64,
    /// 成功确认的数据包数
    pub acknowledged_packets: u64,
    /// 超时未确认的数据包数
    pub timeout_packets: u64,
    /// 平均重传次数
    pub average_retransmissions: f32,
    /// 重传率（0.0-1.0）
    pub retransmission_rate: f32,
}

impl PacketRecoveryStrategy {
    /// 创建新的包丢失恢复策略
    pub fn new(max_ack_queue_size: usize) -> Self {
        Self {
            pending_retransmissions: HashMap::new(),
            acknowledged_packets: VecDeque::with_capacity(max_ack_queue_size),
            max_ack_queue_size,
            retransmission_stats: RetransmissionStats::default(),
        }
    }

    /// 发送数据包（添加重传管理）
    pub fn send_packet(&mut self, packet: DeltaPacket) -> Vec<DeltaPacket> {
        self.retransmission_stats.total_packets_sent += 1;

        let packet_id = packet.sequence;
        let retransmission_info = RetransmissionInfo {
            packet: packet.clone(),
            first_sent_time: Instant::now(),
            retransmission_count: 0,
            last_retransmission_time: Instant::now(),
        };

        self.pending_retransmissions.insert(packet_id, retransmission_info);

        vec![packet] // 首次发送
    }

    /// 处理ACK确认
    pub fn handle_ack(&mut self, packet_id: u64) {
        if self.pending_retransmissions.remove(&packet_id).is_some() {
            self.retransmission_stats.acknowledged_packets += 1;
            self.acknowledged_packets.push_back(packet_id);

            if self.acknowledged_packets.len() > self.max_ack_queue_size {
                self.acknowledged_packets.pop_front();
            }
        }
    }

    /// 检查超时并生成重传数据包
    pub fn check_timeouts(&mut self, timeout_ms: u32, max_retransmissions: u8) -> Vec<DeltaPacket> {
        let timeout_duration = Duration::from_millis(timeout_ms as u64);
        let mut packets_to_resend = Vec::new();
        let mut packets_to_remove = Vec::new();

        for (packet_id, info) in self.pending_retransmissions.iter_mut() {
            let time_since_last_retransmission = info.last_retransmission_time.elapsed();

            if time_since_last_retransmission > timeout_duration {
                if info.retransmission_count < max_retransmissions {
                    // 重传
                    info.retransmission_count += 1;
                    info.last_retransmission_time = Instant::now();
                    packets_to_resend.push(info.packet.clone());
                    self.retransmission_stats.retransmitted_packets += 1;
                } else {
                    // 超过最大重传次数，放弃
                    packets_to_remove.push(*packet_id);
                    self.retransmission_stats.timeout_packets += 1;
                }
            }
        }

        // 移除超时的数据包
        for packet_id in packets_to_remove {
            self.pending_retransmissions.remove(&packet_id);
        }

        // 更新统计
        self.update_retransmission_stats();

        packets_to_resend
    }

    /// 更新重传统计
    fn update_retransmission_stats(&mut self) {
        let total = self.retransmission_stats.total_packets_sent;
        let retransmitted = self.retransmission_stats.retransmitted_packets;

        if total > 0 {
            self.retransmission_stats.retransmission_rate = retransmitted as f32 / total as f32;
        }

        // 计算平均重传次数
        let total_retransmissions: u32 = self.pending_retransmissions
            .values()
            .map(|info| info.retransmission_count as u32)
            .sum();

        let counted_packets = self.pending_retransmissions.len() + self.retransmission_stats.acknowledged_packets as usize
            + self.retransmission_stats.timeout_packets as usize;

        if counted_packets > 0 {
            self.retransmission_stats.average_retransmissions = total_retransmissions as f32 / counted_packets as f32;
        }
    }

    /// 获取重传统计
    pub fn get_stats(&self) -> &RetransmissionStats {
        &self.retransmission_stats
    }
}

/// 客户端插值器
#[derive(Debug, Clone)]
pub struct ClientInterpolator {
    /// 插值缓冲区（每个实体）
    entity_buffers: HashMap<u64, InterpolationBuffer>,
    /// 插入延迟（秒）
    interpolation_delay: f32,
    /// 最大缓冲区大小
    max_buffer_size: usize,
    /// 插值统计
    interpolation_stats: InterpolationStats,
}

/// 插值缓冲区
#[derive(Debug, Clone)]
struct InterpolationBuffer {
    /// 状态快照队列
    snapshots: VecDeque<StateSnapshot>,
    /// 最后插值时间
    last_interpolation_time: f32,
}

/// 状态快照
#[derive(Debug, Clone)]
struct StateSnapshot {
    /// 时间戳
    timestamp: f32,
    /// 位置
    position: Vec3,
    /// 旋转（四元数）
    rotation: [f32; 4],
    /// 速度
    velocity: Vec3,
}

/// 插值统计
#[derive(Debug, Clone, Default)]
pub struct InterpolationStats {
    /// 插值的实体数
    pub interpolated_entities: usize,
    /// 丢弃的快照数（延迟过大）
    pub discarded_snapshots: usize,
    /// 平均缓冲区大小
    pub average_buffer_size: f32,
    /// 插值误差（估计）
    pub interpolation_error: f32,
}

impl ClientInterpolator {
    /// 创建新的客户端插值器
    pub fn new(interpolation_delay_ms: u32, max_buffer_size: usize) -> Self {
        Self {
            entity_buffers: HashMap::new(),
            interpolation_delay: interpolation_delay_ms as f32 / 1000.0,
            max_buffer_size,
            interpolation_stats: InterpolationStats::default(),
        }
    }

    /// 添加状态快照
    pub fn add_snapshot(&mut self, entity_id: u64, timestamp: f32, position: Vec3, rotation: [f32; 4], velocity: Vec3) {
        let buffer = self.entity_buffers
            .entry(entity_id)
            .or_insert_with(|| InterpolationBuffer {
                snapshots: VecDeque::new(),
                last_interpolation_time: 0.0,
            });

        let snapshot = StateSnapshot {
            timestamp,
            position,
            rotation,
            velocity,
        };

        buffer.snapshots.push_back(snapshot);

        // 限制缓冲区大小
        while buffer.snapshots.len() > self.max_buffer_size {
            buffer.snapshots.pop_front();
            self.interpolation_stats.discarded_snapshots += 1;
        }
    }

    /// 获取插值状态
    pub fn get_interpolated_state(&mut self, entity_id: u64, current_time: f32) -> Option<(Vec3, [f32; 4])> {
        let buffer = self.entity_buffers.get_mut(&entity_id)?;

        // 计算插值时间点
        let render_time = current_time - self.interpolation_delay;

        // 查找两个快照进行插值
        let mut snapshot_before = None;
        let mut snapshot_after = None;

        for (i, snapshot) in buffer.snapshots.iter().enumerate() {
            if snapshot.timestamp <= render_time {
                snapshot_before = Some((i, snapshot));
            } else {
                snapshot_after = Some((i, snapshot));
                break;
            }
        }

        match (snapshot_before, snapshot_after) {
            (Some((_, before)), Some((_, after))) => {
                // 线性插值
                let t = (render_time - before.timestamp) / (after.timestamp - before.timestamp);
                let t = t.clamp(0.0, 1.0);

                let position = before.position.lerp(after.position, t);

                // 四元数球面线性插值（简化版本）
                let rotation = [
                    before.rotation[0] + (after.rotation[0] - before.rotation[0]) * t,
                    before.rotation[1] + (after.rotation[1] - before.rotation[1]) * t,
                    before.rotation[2] + (after.rotation[2] - before.rotation[2]) * t,
                    before.rotation[3] + (after.rotation[3] - before.rotation[3]) * t,
                ];

                buffer.last_interpolation_time = render_time;
                self.interpolation_stats.interpolated_entities = self.entity_buffers.len();
                self.update_buffer_stats();

                Some((position, rotation))
            }
            (Some((_, snapshot)), None) => {
                // 只有较旧的快照，直接使用
                buffer.last_interpolation_time = render_time;
                Some((snapshot.position, snapshot.rotation))
            }
            _ => None,
        }
    }

    /// 清理旧快照
    pub fn cleanup_old_snapshots(&mut self, current_time: f32) {
        let cleanup_time = current_time - self.interpolation_delay * 2.0;

        for buffer in self.entity_buffers.values_mut() {
            while let Some(front) = buffer.snapshots.front() {
                if front.timestamp < cleanup_time {
                    buffer.snapshots.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    /// 更新缓冲区统计
    fn update_buffer_stats(&mut self) {
        let total_size: usize = self.entity_buffers
            .values()
            .map(|b| b.snapshots.len())
            .sum();

        let count = self.entity_buffers.len();
        if count > 0 {
            self.interpolation_stats.average_buffer_size = total_size as f32 / count as f32;
        }
    }

    /// 获取插值统计
    pub fn get_stats(&self) -> &InterpolationStats {
        &self.interpolation_stats
    }

    /// 移除实体
    pub fn remove_entity(&mut self, entity_id: u64) {
        self.entity_buffers.remove(&entity_id);
    }
}

/// 增强网络同步管理器
pub struct EnhancedNetworkSync {
    /// 优先级同步管理器
    priority_sync: PrioritySyncManager,
    /// 包丢失恢复策略
    packet_recovery: PacketRecoveryStrategy,
    /// 客户端插值器
    client_interpolator: ClientInterpolator,
    /// 配置
    config: EnhancedNetworkSyncConfig,
    /// 网络质量
    network_quality: NetworkQuality,
    /// 当前tick
    current_tick: u64,
    /// 帧计数
    frame_count: u32,
    /// 性能统计
    performance_stats: NetworkSyncPerformanceStats,
}

/// 网络同步性能统计
#[derive(Debug, Clone, Default)]
pub struct NetworkSyncPerformanceStats {
    /// 总帧数
    pub total_frames: u64,
    /// 发送的总数据包数
    pub total_packets_sent: u64,
    /// 接收的总数据包数
    pub total_packets_received: u64,
    /// 平均带宽使用（字节/帧）
    pub average_bandwidth_usage: f32,
    /// 平均延迟（毫秒）
    pub average_latency_ms: f32,
    /// 包丢失率
    pub packet_loss_rate: f32,
    /// 插值的实体数
    pub interpolated_entities: usize,
    /// 重传率
    pub retransmission_rate: f32,
}

impl EnhancedNetworkSync {
    /// 创建新的增强网络同步管理器
    pub fn new(config: EnhancedNetworkSyncConfig, max_bytes_per_frame: usize) -> Self {
        Self {
            priority_sync: PrioritySyncManager::new(max_bytes_per_frame),
            packet_recovery: PacketRecoveryStrategy::new(1000),
            client_interpolator: ClientInterpolator::new(config.interpolation_delay_ms, 10),
            config,
            network_quality: NetworkQuality::default(),
            current_tick: 0,
            frame_count: 0,
            performance_stats: NetworkSyncPerformanceStats::default(),
        }
    }

    /// 使用默认配置创建
    pub fn default_config(max_bytes_per_frame: usize) -> Self {
        Self::new(EnhancedNetworkSyncConfig::default(), max_bytes_per_frame)
    }

    /// 服务器端：生成并发送同步数据包
    pub fn server_update_and_send(&mut self, current_tick: u64) -> Vec<DeltaPacket> {
        self.current_tick = current_tick;

        // 生成优先级同步数据包
        let sync_packet = self.priority_sync.generate_priority_sync_packet(current_tick);

        // 通过包恢复策略发送
        let packets = self.packet_recovery.send_packet(sync_packet);

        // 更新统计
        self.performance_stats.total_packets_sent += packets.len() as u64;
        self.performance_stats.total_frames += 1;

        // 检查超时和重传
        let retransmit_packets = self.packet_recovery.check_timeouts(
            self.config.retransmission_timeout_ms,
            self.config.max_retransmissions,
        );

        // 合并数据包
        let mut all_packets = packets;
        all_packets.extend(retransmit_packets);

        all_packets
    }

    /// 客户端：接收并处理同步数据包
    pub fn client_receive_and_interpolate(
        &mut self,
        packet: &DeltaPacket,
        current_time: f32,
    ) -> HashMap<u64, (Vec3, [f32; 4])> {
        self.performance_stats.total_packets_received += 1;

        // 处理增量数据
        let mut interpolated_states = HashMap::new();

        for delta in &packet.deltas {
            if let (Some(position), Some(rotation)) = (delta.position, delta.rotation) {
                let pos = Vec3::new(position[0], position[1], position[2]);

                // 添加到插值缓冲区
                self.client_interpolator.add_snapshot(
                    delta.id,
                    current_time,
                    pos,
                    rotation,
                    Vec3::from_array(delta.velocity.unwrap_or([0.0, 0.0, 0.0])),
                );
            }
        }

        // 清理旧快照
        self.client_interpolator.cleanup_old_snapshots(current_time);

        // 获取所有实体的插值状态
        let entity_ids: Vec<_> = self.packet_recovery.pending_retransmissions.keys().copied().collect();
        for entity_id in entity_ids {
            if let Some(state) = self.client_interpolator.get_interpolated_state(entity_id, current_time) {
                interpolated_states.insert(entity_id, state);
            }
        }

        // 发送ACK
        self.packet_recovery.handle_ack(packet.sequence);

        interpolated_states
    }

    /// 更新网络质量
    pub fn update_network_quality(&mut self, latency_ms: f32, bandwidth_stats: BandwidthStats) {
        self.network_quality.latency_ms = latency_ms;
        self.network_quality.bandwidth_usage_ratio = bandwidth_stats.usage_ratio;
        self.network_quality.calculate_score();

        // 更新性能统计
        self.performance_stats.average_bandwidth_usage = bandwidth_stats.average_usage;
        self.performance_stats.average_latency_ms = latency_ms;

        // 自适应调整
        if self.config.enable_adaptive_quality {
            self.adaptive_quality_adjustment();
        }
    }

    /// 自适应质量调整
    fn adaptive_quality_adjustment(&mut self) {
        let quality_level = self.network_quality.get_quality_level();

        match quality_level {
            QualityLevel::Excellent => {
                // 优秀：提高同步频率
                // 可以添加更多高优先级实体
            }
            QualityLevel::Good => {
                // 良好：保持当前配置
            }
            QualityLevel::Fair => {
                // 一般：略微降低同步频率
            }
            QualityLevel::Poor => {
                // 较差：降低同步频率，减少带宽使用
            }
            QualityLevel::Terrible => {
                // 很差：大幅降低同步频率，只同步关键实体
            }
        }
    }

    /// 获取性能统计
    pub fn get_performance_stats(&self) -> &NetworkSyncPerformanceStats {
        &self.performance_stats
    }

    /// 获取网络质量
    pub fn get_network_quality(&self) -> &NetworkQuality {
        &self.network_quality
    }

    /// 生成性能报告
    pub fn generate_performance_report(&self) -> String {
        let stats = &self.performance_stats;
        let quality = &self.network_quality;
        let retransmission_stats = self.packet_recovery.get_stats();
        let interpolation_stats = self.client_interpolator.get_stats();

        format!(
            "=== Enhanced Network Sync Performance Report ===\n\
             Total Frames: {}\n\
             Packets Sent: {} | Received: {}\n\
             Average Bandwidth: {:.1} bytes/frame\n\
             Average Latency: {:.1}ms\n\
             Packet Loss Rate: {:.1}%\n\
             Network Quality Score: {:.2} ({:?})\n\
             Retransmission Rate: {:.1}%\n\
             Interpolated Entities: {}\n\
             ============================================",
            stats.total_frames,
            stats.total_packets_sent,
            stats.total_packets_received,
            stats.average_bandwidth_usage,
            stats.average_latency_ms,
            stats.packet_loss_rate * 100.0,
            quality.quality_score,
            quality.get_quality_level(),
            retransmission_stats.retransmission_rate * 100.0,
            interpolation_stats.interpolated_entities
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_quality_calculation() {
        let mut quality = NetworkQuality::default();
        quality.latency_ms = 50.0;
        quality.jitter_ms = 10.0;
        quality.packet_loss_rate = 0.01;
        quality.bandwidth_usage_ratio = 0.4;

        quality.calculate_score();

        assert!(quality.quality_score > 0.7);
        assert_eq!(quality.get_quality_level(), QualityLevel::Excellent);
    }

    #[test]
    fn test_client_interpolation() {
        let mut interpolator = ClientInterpolator::new(100, 10);

        // 添加两个快照
        interpolator.add_snapshot(1, 0.0, Vec3::ZERO, [0.0, 0.0, 0.0, 1.0], Vec3::ZERO);
        interpolator.add_snapshot(1, 1.0, Vec3::new(1.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0], Vec3::ZERO);

        // 插值（时间点0.5）
        let state = interpolator.get_interpolated_state(1, 0.6); // 0.6 - 0.1 (delay) = 0.5

        assert!(state.is_some());
        let (pos, _) = state.unwrap();
        assert!((pos.x - 0.5).abs() < 0.01); // 应该接近中间位置
    }
}
