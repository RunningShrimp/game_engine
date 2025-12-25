//! 智能优先级网络同步模块
//!
//! 实现基于优先级的网络同步系统，优化带宽使用：
//! - 动态优先级计算（基于距离、重要性、变化率）
//! - 带宽预算管理
//! - 自适应更新频率
//! - 优先级队列调度

use crate::network::delta_serialization::{DeltaPacket, EntityDelta};
use glam::Vec3;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

/// 同步优先级（0-255，值越大优先级越高）
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SyncPriority(pub u8);

impl SyncPriority {
    /// 最高优先级（关键实体，如玩家）
    pub const CRITICAL: SyncPriority = SyncPriority(255);
    /// 高优先级（重要实体，如NPC、载具）
    pub const HIGH: SyncPriority = SyncPriority(200);
    /// 中等优先级（普通实体）
    pub const MEDIUM: SyncPriority = SyncPriority(128);
    /// 低优先级（背景实体）
    pub const LOW: SyncPriority = SyncPriority(64);
    /// 最低优先级（静态或很少变化的实体）
    pub const MINIMAL: SyncPriority = SyncPriority(32);
}

/// 实体同步信息
#[derive(Debug, Clone)]
pub struct EntitySyncInfo {
    /// 实体ID
    pub entity_id: u64,
    /// 当前优先级
    pub priority: SyncPriority,
    /// 计算出的优先级分数（用于排序）
    pub priority_score: f32,
    /// 距离（到观察者）
    pub distance: f32,
    /// 重要性权重
    pub importance_weight: f32,
    /// 变化率（位置变化速度）
    pub change_rate: f32,
    /// 上次更新tick
    pub last_update_tick: u64,
    /// 建议的更新间隔（tick数）
    pub suggested_interval: u64,
    /// 实体增量数据
    pub delta: EntityDelta,
    /// 估算的序列化大小（字节）
    pub estimated_size: usize,
}

impl EntitySyncInfo {
    /// 创建新的实体同步信息
    pub fn new(entity_id: u64, delta: EntityDelta) -> Self {
        Self {
            entity_id,
            priority: SyncPriority::MEDIUM,
            priority_score: 0.0,
            distance: 0.0,
            importance_weight: 1.0,
            change_rate: 0.0,
            last_update_tick: 0,
            suggested_interval: 1,
            estimated_size: delta.estimated_size(),
            delta,
        }
    }

    /// 计算优先级分数
    pub fn calculate_priority_score(&mut self, observer_position: Vec3) {
        // 更新距离
        if let Some(pos) = self.delta.position {
            let entity_pos = Vec3::new(pos[0], pos[1], pos[2]);
            self.distance = (entity_pos - observer_position).length();
        }

        // 优先级分数 = 重要性权重 * 距离因子 * 变化率因子
        let distance_factor = if self.distance < 10.0 {
            1.0 // 近距离，高优先级
        } else if self.distance < 50.0 {
            0.8
        } else if self.distance < 100.0 {
            0.5
        } else {
            0.2 // 远距离，低优先级
        };

        let change_rate_factor = (self.change_rate * 10.0).min(1.0); // 变化越快，优先级越高

        self.priority_score = self.importance_weight * distance_factor * (0.5 + change_rate_factor * 0.5);

        // 根据分数确定优先级等级
        self.priority = if self.priority_score > 0.8 {
            SyncPriority::CRITICAL
        } else if self.priority_score > 0.6 {
            SyncPriority::HIGH
        } else if self.priority_score > 0.4 {
            SyncPriority::MEDIUM
        } else if self.priority_score > 0.2 {
            SyncPriority::LOW
        } else {
            SyncPriority::MINIMAL
        };

        // 根据优先级和距离建议更新间隔
        self.suggested_interval = match self.priority {
            SyncPriority::CRITICAL => 1,      // 每帧更新
            SyncPriority::HIGH => 2,          // 每2帧
            SyncPriority::MEDIUM => 4,        // 每4帧
            SyncPriority::LOW => 8,           // 每8帧
            SyncPriority::MINIMAL => 16,      // 每16帧
        };
    }
}

impl PartialOrd for EntitySyncInfo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // 优先级分数高的排在前面
        self.priority_score.partial_cmp(&other.priority_score)
    }
}

impl Ord for EntitySyncInfo {
    fn cmp(&self, other: &Self) -> Ordering {
        // BinaryHeap是最大堆，所以需要反转比较
        other.priority_score.partial_cmp(&self.priority_score)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialEq for EntitySyncInfo {
    fn eq(&self, other: &Self) -> bool {
        self.entity_id == other.entity_id
    }
}

impl Eq for EntitySyncInfo {}

/// 带宽预算管理器
#[derive(Debug, Clone)]
pub struct BandwidthBudget {
    /// 每帧最大字节数
    pub max_bytes_per_frame: usize,
    /// 当前帧已使用字节数
    pub current_bytes_used: usize,
    /// 平均带宽使用（移动平均）
    pub average_bandwidth: f32,
    /// 带宽使用历史（用于计算平均值）
    bandwidth_history: Vec<usize>,
    /// 历史窗口大小
    history_window_size: usize,
}

impl BandwidthBudget {
    /// 创建带宽预算管理器
    pub fn new(max_bytes_per_frame: usize) -> Self {
        Self {
            max_bytes_per_frame,
            current_bytes_used: 0,
            average_bandwidth: 0.0,
            bandwidth_history: Vec::new(),
            history_window_size: 60, // 60帧历史
        }
    }

    /// 检查是否有足够的带宽
    pub fn has_capacity(&self, size: usize) -> bool {
        self.current_bytes_used + size <= self.max_bytes_per_frame
    }

    /// 分配带宽
    pub fn allocate(&mut self, size: usize) -> bool {
        if self.has_capacity(size) {
            self.current_bytes_used += size;
            true
        } else {
            false
        }
    }

    /// 重置当前帧的带宽使用
    pub fn reset_frame(&mut self) {
        // 更新历史
        self.bandwidth_history.push(self.current_bytes_used);
        if self.bandwidth_history.len() > self.history_window_size {
            self.bandwidth_history.remove(0);
        }

        // 计算平均带宽
        if !self.bandwidth_history.is_empty() {
            let sum: usize = self.bandwidth_history.iter().sum();
            self.average_bandwidth = sum as f32 / self.bandwidth_history.len() as f32;
        }

        // 重置当前帧
        self.current_bytes_used = 0;
    }

    /// 获取带宽使用率（0.0-1.0）
    pub fn usage_ratio(&self) -> f32 {
        if self.max_bytes_per_frame == 0 {
            return 0.0;
        }
        self.current_bytes_used as f32 / self.max_bytes_per_frame as f32
    }

    /// 获取平均带宽使用率
    pub fn average_usage_ratio(&self) -> f32 {
        if self.max_bytes_per_frame == 0 {
            return 0.0;
        }
        self.average_bandwidth / self.max_bytes_per_frame as f32
    }
}

/// 智能优先级同步管理器
pub struct PrioritySyncManager {
    /// 实体同步信息映射
    entity_infos: HashMap<u64, EntitySyncInfo>,
    /// 优先级队列（最大堆）
    priority_queue: BinaryHeap<EntitySyncInfo>,
    /// 带宽预算管理器
    bandwidth_budget: BandwidthBudget,
    /// 观察者位置（用于距离计算）
    observer_position: Vec3,
    /// 当前tick
    current_tick: u64,
    /// 实体重要性权重映射
    importance_weights: HashMap<u64, f32>,
}

impl PrioritySyncManager {
    /// 创建智能优先级同步管理器
    pub fn new(max_bytes_per_frame: usize) -> Self {
        Self {
            entity_infos: HashMap::new(),
            priority_queue: BinaryHeap::new(),
            bandwidth_budget: BandwidthBudget::new(max_bytes_per_frame),
            observer_position: Vec3::ZERO,
            current_tick: 0,
            importance_weights: HashMap::new(),
        }
    }

    /// 设置观察者位置（用于距离计算）
    pub fn set_observer_position(&mut self, position: Vec3) {
        self.observer_position = position;
    }

    /// 设置实体重要性权重
    pub fn set_importance_weight(&mut self, entity_id: u64, weight: f32) {
        self.importance_weights.insert(entity_id, weight);
    }

    /// 注册或更新实体同步信息
    pub fn update_entity(&mut self, entity_id: u64, delta: EntityDelta, current_tick: u64) {
        let importance_weight = self.importance_weights.get(&entity_id).copied().unwrap_or(1.0);

        // 计算变化率
        let change_rate = if let Some(old_info) = self.entity_infos.get(&entity_id) {
            if let (Some(old_pos), Some(new_pos)) = (old_info.delta.position, delta.position) {
                let old_vec = Vec3::new(old_pos[0], old_pos[1], old_pos[2]);
                let new_vec = Vec3::new(new_pos[0], new_pos[1], new_pos[2]);
                let distance = (new_vec - old_vec).length();
                let ticks_since_update = current_tick - old_info.last_update_tick;
                if ticks_since_update > 0 {
                    distance / ticks_since_update as f32
                } else {
                    0.0
                }
            } else {
                0.0
            }
        } else {
            0.0
        };

        let mut info = EntitySyncInfo::new(entity_id, delta);
        info.importance_weight = importance_weight;
        info.change_rate = change_rate;
        info.last_update_tick = current_tick;
        info.calculate_priority_score(self.observer_position);

        self.entity_infos.insert(entity_id, info);
    }

    /// 生成优先级同步数据包（在带宽预算内）
    pub fn generate_priority_sync_packet(&mut self, current_tick: u64) -> DeltaPacket {
        self.current_tick = current_tick;
        self.bandwidth_budget.reset_frame();

        // 重新计算所有实体的优先级
        self.priority_queue.clear();
        for (_, info) in self.entity_infos.iter_mut() {
            info.calculate_priority_score(self.observer_position);
            self.priority_queue.push(info.clone());
        }

        // 创建增量数据包
        let mut packet = DeltaPacket::new(current_tick, 0);

        // 按优先级顺序添加实体，直到带宽预算用完
        while let Some(mut info) = self.priority_queue.pop() {
            // 检查是否需要更新（基于建议的间隔）
            let ticks_since_update = current_tick - info.last_update_tick;
            if ticks_since_update < info.suggested_interval {
                continue;
            }

            // 检查带宽预算
            if !self.bandwidth_budget.has_capacity(info.estimated_size) {
                // 带宽不足，跳过低优先级实体
                break;
            }

            // 分配带宽并添加增量
            if self.bandwidth_budget.allocate(info.estimated_size) {
                packet.add_delta(info.delta.clone());
                // 更新最后更新tick
                if let Some(entity_info) = self.entity_infos.get_mut(&info.entity_id) {
                    entity_info.last_update_tick = current_tick;
                }
            }
        }

        packet
    }

    /// 获取带宽使用统计
    pub fn get_bandwidth_stats(&self) -> BandwidthStats {
        BandwidthStats {
            current_usage: self.bandwidth_budget.current_bytes_used,
            max_capacity: self.bandwidth_budget.max_bytes_per_frame,
            usage_ratio: self.bandwidth_budget.usage_ratio(),
            average_usage: self.bandwidth_budget.average_bandwidth,
            average_usage_ratio: self.bandwidth_budget.average_usage_ratio(),
        }
    }

    /// 获取实体优先级信息
    pub fn get_entity_priority(&self, entity_id: u64) -> Option<SyncPriority> {
        self.entity_infos.get(&entity_id).map(|info| info.priority)
    }

    /// 清除实体信息
    pub fn remove_entity(&mut self, entity_id: u64) {
        self.entity_infos.remove(&entity_id);
    }
}

/// 带宽使用统计
#[derive(Debug, Clone)]
pub struct BandwidthStats {
    /// 当前使用字节数
    pub current_usage: usize,
    /// 最大容量字节数
    pub max_capacity: usize,
    /// 使用率（0.0-1.0）
    pub usage_ratio: f32,
    /// 平均使用字节数
    pub average_usage: f32,
    /// 平均使用率
    pub average_usage_ratio: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_calculation() {
        let mut manager = PrioritySyncManager::new(10000);
        manager.set_observer_position(Vec3::new(0.0, 0.0, 0.0));

        // 创建近距离实体（高优先级）
        let mut close_delta = EntityDelta::new(1);
        close_delta.position = Some([1.0, 1.0, 1.0]);
        manager.update_entity(1, close_delta, 0);

        // 创建远距离实体（低优先级）
        let mut far_delta = EntityDelta::new(2);
        far_delta.position = Some([100.0, 100.0, 100.0]);
        manager.update_entity(2, far_delta, 0);

        // 生成同步数据包
        let packet = manager.generate_priority_sync_packet(1);

        // 近距离实体应该优先被包含
        assert!(packet.deltas.iter().any(|d| d.id == 1));
    }

    #[test]
    fn test_bandwidth_budget() {
        let mut budget = BandwidthBudget::new(1000);

        assert!(budget.has_capacity(500));
        assert!(budget.allocate(500));
        assert!(!budget.has_capacity(600));
        assert_eq!(budget.usage_ratio(), 0.5);

        budget.reset_frame();
        assert_eq!(budget.current_bytes_used, 0);
    }
}

