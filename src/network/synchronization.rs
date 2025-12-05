<<<<<<< HEAD
//! 网络同步算法模块
//!
//! 实现状态同步和事件同步，包括冲突解决机制。
//!
//! ## 设计原理
//!
//! 网络同步通过以下机制确保客户端和服务器状态一致：
//!
//! ```text
//! ┌─────────────────┐         ┌─────────────────┐
//! │     Client      │         │     Server      │
//! │                 │         │                 │
//! │  Local State   │────────►│  Validate       │
//! │  (Predicted)   │         │  & Resolve      │
//! │                 │         │                 │
//! │  Apply Server  │◄────────│  Authoritative  │
//! │  State         │         │  State          │
//! └─────────────────┘         └─────────────────┘
//! ```
//!
//! ## 核心机制
//!
//! 1. **状态同步**: 定期同步游戏状态（位置、旋转、速度等）
//! 2. **事件同步**: 同步离散事件（开火、拾取物品等）
//! 3. **冲突解决**: 服务器权威，客户端校正
//! 4. **平滑插值**: 减少状态跳变，提升体验

use crate::core::utils::current_timestamp_ms;
use crate::network::delta_serialization::{DeltaPacket, DeltaSerializer, EntityDelta};
use crate::network::NetworkError;
use glam::{Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 同步策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStrategy {
    /// 服务器权威（服务器状态优先）
    ServerAuthoritative,
    /// 客户端预测（允许客户端预测，服务器验证）
    ClientPrediction,
    /// 混合模式（根据实体类型选择）
    Hybrid,
}

/// 冲突解决策略
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConflictResolutionStrategy {
    /// 服务器优先（直接使用服务器状态）
    ServerWins,
    /// 平滑校正（插值到服务器状态）
    SmoothCorrection,
    /// 延迟校正（延迟一段时间后校正）
    DelayedCorrection { delay_ms: u64 },
    /// 阈值校正（偏差超过阈值才校正）
    ThresholdCorrection { threshold: f32 },
}

/// 实体同步状态
#[derive(Debug, Clone)]
pub struct EntitySyncState {
    /// 实体网络ID
    pub entity_id: u64,
    /// 最后同步的tick
    pub last_sync_tick: u64,
    /// 同步策略
    pub sync_strategy: SyncStrategy,
    /// 冲突解决策略
    pub conflict_resolution: ConflictResolutionStrategy,
    /// 服务器状态（用于冲突检测）
    pub server_state: Option<EntityState>,
    /// 客户端状态（预测状态）
    pub client_state: Option<EntityState>,
    /// 是否正在校正
    pub correcting: bool,
    /// 校正开始时间
    pub correction_start_time: Option<u64>,
}

/// 实体状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    /// 位置
    pub position: Vec3,
    /// 旋转
    pub rotation: Quat,
    /// 缩放
    pub scale: Vec3,
    /// 速度
    pub velocity: Vec3,
    /// 时间戳
    pub timestamp: u64,
    /// 状态版本号（用于冲突检测）
    pub version: u64,
}

impl EntityState {
    /// 创建新的实体状态
    pub fn new(position: Vec3, rotation: Quat, scale: Vec3, velocity: Vec3) -> Self {
        Self {
            position,
            rotation,
            scale,
            velocity,
            timestamp: current_timestamp_ms(),
            version: 0,
        }
    }

    /// 计算与另一个状态的距离
    pub fn distance_to(&self, other: &EntityState) -> f32 {
        (self.position - other.position).length()
    }

    /// 检查状态是否相似（在阈值内）
    pub fn is_similar_to(&self, other: &EntityState, threshold: f32) -> bool {
        self.distance_to(other) < threshold
    }
}

/// 网络事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    /// 事件ID（全局唯一）
    pub event_id: u64,
    /// 事件类型
    pub event_type: EventType,
    /// 实体ID（如果适用）
    pub entity_id: Option<u64>,
    /// 事件数据
    pub data: Vec<u8>,
    /// 时间戳
    pub timestamp: u64,
    /// 发送者客户端ID
    pub sender_id: u64,
}

/// 事件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventType {
    /// 开火
    Fire,
    /// 拾取物品
    Pickup,
    /// 使用物品
    Use,
    /// 交互
    Interact,
    /// 自定义事件
    Custom(u32),
}

/// 状态同步管理器
pub struct StateSyncManager {
    /// 实体同步状态映射
    entity_states: HashMap<u64, EntitySyncState>,
    /// 增量序列化器
    delta_serializer: DeltaSerializer,
    /// 同步间隔（tick数）
    sync_interval: u64,
    /// 冲突检测阈值
    conflict_threshold: f32,
    /// 事件队列
    event_queue: Vec<NetworkEvent>,
    /// 最大事件队列大小
    max_event_queue_size: usize,
}

impl StateSyncManager {
    /// 创建新的状态同步管理器
    pub fn new(sync_interval: u64, conflict_threshold: f32) -> Self {
        Self {
            entity_states: HashMap::new(),
            delta_serializer: DeltaSerializer::new(),
            sync_interval,
            conflict_threshold,
            event_queue: Vec::new(),
            max_event_queue_size: 1000,
        }
    }

    /// 注册需要同步的实体
    pub fn register_entity(
        &mut self,
        entity_id: u64,
        sync_strategy: SyncStrategy,
        conflict_resolution: ConflictResolutionStrategy,
    ) {
        self.entity_states.insert(
            entity_id,
            EntitySyncState {
                entity_id,
                last_sync_tick: 0,
                sync_strategy,
                conflict_resolution,
                server_state: None,
                client_state: None,
                correcting: false,
                correction_start_time: None,
            },
        );
    }

    /// 更新客户端状态
    pub fn update_client_state(
        &mut self,
        entity_id: u64,
        state: EntityState,
        current_tick: u64,
    ) -> Result<(), NetworkError> {
        if let Some(sync_state) = self.entity_states.get_mut(&entity_id) {
            sync_state.client_state = Some(state);
            sync_state.last_sync_tick = current_tick;
            let _ = current_tick; // 标记为已使用
            Ok(())
        } else {
            Err(NetworkError::InvalidPeerId)
        }
    }

    /// 更新服务器状态
    pub fn update_server_state(
        &mut self,
        entity_id: u64,
        state: EntityState,
        _current_tick: u64,
    ) -> Result<ConflictResolution, NetworkError> {
        if let Some(sync_state) = self.entity_states.get_mut(&entity_id) {
            let old_server_state = sync_state.server_state.clone();
            let client_state_clone = sync_state.client_state.clone();
            let conflict_resolution = sync_state.conflict_resolution;
            sync_state.server_state = Some(state.clone());

            // 检测冲突
            if let Some(ref client_state) = client_state_clone {
                // 如果有旧的服务器状态，使用它进行比较；否则直接比较客户端状态和新的服务器状态
                let should_check = if let Some(ref _old_server) = old_server_state {
                    // 如果服务器状态已经更新过，检查新状态是否与客户端状态一致
                    !state.is_similar_to(client_state, self.conflict_threshold)
                } else {
                    // 第一次更新服务器状态，直接检查是否与客户端状态一致
                    !state.is_similar_to(client_state, self.conflict_threshold)
                };

                if should_check {
                    // 检测到冲突
                    let resolution =
                        self.resolve_conflict_static(conflict_resolution, &state, client_state);
                    return Ok(ConflictResolution {
                        entity_id,
                        conflict_type: ConflictType::StateMismatch,
                        server_state: state.clone(),
                        client_state: Some(client_state.clone()),
                        resolution,
                    });
                }
            }

            Ok(ConflictResolution {
                entity_id,
                conflict_type: ConflictType::None,
                server_state: state,
                client_state: client_state_clone,
                resolution: ResolutionAction::Accept,
            })
        } else {
            Err(NetworkError::InvalidPeerId)
        }
    }

    /// 解决冲突（静态方法，避免借用问题）
    fn resolve_conflict_static(
        &self,
        strategy: ConflictResolutionStrategy,
        server_state: &EntityState,
        client_state: &EntityState,
    ) -> ResolutionAction {
        match strategy {
            ConflictResolutionStrategy::ServerWins => ResolutionAction::ReplaceWithServer,
            ConflictResolutionStrategy::SmoothCorrection => ResolutionAction::SmoothInterpolate {
                target: server_state.clone(),
                duration_ms: 100,
            },
            ConflictResolutionStrategy::DelayedCorrection { delay_ms } => {
                ResolutionAction::DelayedReplace {
                    target: server_state.clone(),
                    delay_ms,
                }
            }
            ConflictResolutionStrategy::ThresholdCorrection { threshold } => {
                let distance = server_state.distance_to(client_state);
                if distance > threshold {
                    ResolutionAction::ReplaceWithServer
                } else {
                    ResolutionAction::Accept
                }
            }
        }
    }

    /// 生成状态同步数据（增量）
    pub fn generate_sync_data(&mut self, current_tick: u64) -> Result<DeltaPacket, NetworkError> {
        let mut deltas = Vec::new();

        for (entity_id, sync_state) in &self.entity_states {
            // 检查是否需要同步
            if current_tick - sync_state.last_sync_tick < self.sync_interval {
                continue;
            }

            if let Some(ref client_state) = sync_state.client_state {
                let mut delta = EntityDelta::new(*entity_id);
                delta.position = Some([
                    client_state.position.x,
                    client_state.position.y,
                    client_state.position.z,
                ]);
                delta.rotation = Some([
                    client_state.rotation.x,
                    client_state.rotation.y,
                    client_state.rotation.z,
                    client_state.rotation.w,
                ]);
                delta.scale = Some([
                    client_state.scale.x,
                    client_state.scale.y,
                    client_state.scale.z,
                ]);
                delta.velocity = Some([
                    client_state.velocity.x,
                    client_state.velocity.y,
                    client_state.velocity.z,
                ]);

                deltas.push(delta);
            }
        }

        // 计算增量
        let packet = self.delta_serializer.compute_delta(&deltas);
        Ok(packet)
    }

    /// 应用服务器状态更新
    pub fn apply_server_update(
        &mut self,
        packet: &DeltaPacket,
        current_tick: u64,
    ) -> Vec<ConflictResolution> {
        let mut conflicts = Vec::new();

        for delta in &packet.deltas {
            if let Some(sync_state) = self.entity_states.get_mut(&delta.id) {
                // 构建服务器状态
                let server_state = EntityState {
                    position: delta
                        .position
                        .map(|p| Vec3::new(p[0], p[1], p[2]))
                        .unwrap_or(Vec3::ZERO),
                    rotation: delta
                        .rotation
                        .map(|r| Quat::from_xyzw(r[0], r[1], r[2], r[3]))
                        .unwrap_or(Quat::IDENTITY),
                    scale: delta
                        .scale
                        .map(|s| Vec3::new(s[0], s[1], s[2]))
                        .unwrap_or(Vec3::ONE),
                    velocity: delta
                        .velocity
                        .map(|v| Vec3::new(v[0], v[1], v[2]))
                        .unwrap_or(Vec3::ZERO),
                    timestamp: current_timestamp_ms(),
                    version: sync_state
                        .server_state
                        .as_ref()
                        .map(|s| s.version + 1)
                        .unwrap_or(0),
                };

                // 更新服务器状态并检测冲突
                if let Ok(conflict) = self.update_server_state(delta.id, server_state, current_tick)
                {
                    if conflict.conflict_type != ConflictType::None {
                        conflicts.push(conflict);
                    }
                }
            }
        }

        conflicts
    }

    /// 添加网络事件
    pub fn add_event(&mut self, event: NetworkEvent) {
        self.event_queue.push(event);

        // 限制队列大小
        while self.event_queue.len() > self.max_event_queue_size {
            self.event_queue.remove(0);
        }
    }

    /// 获取待发送的事件
    pub fn get_pending_events(&self) -> &[NetworkEvent] {
        &self.event_queue
    }

    /// 清除已发送的事件
    pub fn clear_events(&mut self, event_ids: &[u64]) {
        self.event_queue
            .retain(|e| !event_ids.contains(&e.event_id));
    }

    /// 获取实体同步状态
    pub fn get_entity_sync_state(&self, entity_id: u64) -> Option<&EntitySyncState> {
        self.entity_states.get(&entity_id)
    }
}

/// 冲突类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConflictType {
    /// 无冲突
    None,
    /// 状态不匹配
    StateMismatch,
    /// 版本冲突
    VersionConflict,
    /// 时间戳冲突
    TimestampConflict,
}

/// 冲突解决结果
#[derive(Debug, Clone)]
pub struct ConflictResolution {
    /// 实体ID
    pub entity_id: u64,
    /// 冲突类型
    pub conflict_type: ConflictType,
    /// 服务器状态
    pub server_state: EntityState,
    /// 客户端状态
    pub client_state: Option<EntityState>,
    /// 解决动作
    pub resolution: ResolutionAction,
}

/// 解决动作
#[derive(Debug, Clone)]
pub enum ResolutionAction {
    /// 接受（无冲突）
    Accept,
    /// 使用服务器状态替换
    ReplaceWithServer,
    /// 平滑插值到服务器状态
    SmoothInterpolate {
        target: EntityState,
        duration_ms: u64,
    },
    /// 延迟替换
    DelayedReplace { target: EntityState, delay_ms: u64 },
}

/// 事件同步管理器
pub struct EventSyncManager {
    /// 事件队列
    events: Vec<NetworkEvent>,
    /// 已确认的事件ID集合
    confirmed_events: std::collections::HashSet<u64>,
    /// 最大队列大小
    max_queue_size: usize,
    /// 事件确认超时（毫秒）
    confirmation_timeout_ms: u64,
}

impl EventSyncManager {
    /// 创建新的事件同步管理器
    pub fn new(max_queue_size: usize, confirmation_timeout_ms: u64) -> Self {
        Self {
            events: Vec::new(),
            confirmed_events: std::collections::HashSet::new(),
            max_queue_size,
            confirmation_timeout_ms,
        }
    }

    /// 添加事件
    pub fn add_event(&mut self, event: NetworkEvent) {
        self.events.push(event);

        // 限制队列大小
        while self.events.len() > self.max_queue_size {
            self.events.remove(0);
        }
    }

    /// 获取未确认的事件
    pub fn get_unconfirmed_events(&self) -> Vec<&NetworkEvent> {
        let now = current_timestamp_ms();
        self.events
            .iter()
            .filter(|e| {
                !self.confirmed_events.contains(&e.event_id)
                    && now - e.timestamp < self.confirmation_timeout_ms
            })
            .collect()
    }

    /// 确认事件
    pub fn confirm_event(&mut self, event_id: u64) {
        self.confirmed_events.insert(event_id);
    }

    /// 清理已确认的事件
    pub fn cleanup_confirmed(&mut self) {
        let confirmed_ids: Vec<u64> = self.confirmed_events.iter().copied().collect();
        self.events.retain(|e| !confirmed_ids.contains(&e.event_id));

        // 清理过期的确认记录
        let now = current_timestamp_ms();
        self.confirmed_events.retain(|&id| {
            self.events
                .iter()
                .any(|e| e.event_id == id && now - e.timestamp < self.confirmation_timeout_ms)
        });
=======
//! 网络同步模块
//!
//! 实现高并发游戏环境下的网络同步机制，包括：
//! - 状态同步
//! - 增量同步
//! - 兴趣管理
//! - 权威状态验证
//!
//! 基于客户端预测和服务器补偿的同步模型。

use crate::core::utils::current_timestamp_ms;
use crate::impl_default;
use crate::network::{NetworkError, NetworkMessage};
use glam::{ Vec3, Vec3A };
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock, Mutex};

/// 兴趣区域类型
#[derive(Debug, Clone, Copy)]
pub enum InterestArea {
    /// 球形兴趣区域
    Sphere { center: Vec3, radius: f32 },
    /// 立方体兴趣区域
    Cube { center: Vec3, half_extent: Vec3 },
}

/// 客户端兴趣
#[derive(Debug, Clone)]
pub struct ClientInterest {
    /// 客户端ID
    pub client_id: u64,
    /// 兴趣区域
    pub area: InterestArea,
    /// 兴趣对象类型
    pub object_types: HashSet<String>,
    /// 更新时间
    pub last_updated: u64,
}

/// 兴趣管理器
pub struct InterestManager {
    /// 客户端兴趣映射
    interests: RwLock<HashMap<u64, ClientInterest>>,
}

impl InterestManager {
    pub fn new() -> Self {
        Self {
            interests: RwLock::new(HashMap::new()),
        }
    }

    /// 更新客户端兴趣
    pub fn update_client_interest(&self, client_id: u64, interest: ClientInterest) {
        let mut interests = self.interests.write().unwrap();
        interests.insert(client_id, interest);
    }

    /// 获取客户端兴趣
    pub fn get_client_interest(&self, client_id: u64) -> Option<ClientInterest> {
        self.interests.read().unwrap().get(&client_id).cloned()
    }

    /// 删除客户端兴趣
    pub fn remove_client_interest(&self, client_id: u64) {
        let mut interests = self.interests.write().unwrap();
        interests.remove(&client_id);
    }

    /// 检查对象是否在客户端兴趣范围内
    pub fn is_object_in_interest(
        &self,
        client_id: u64,
        object_pos: Vec3,
        object_type: &str,
    ) -> bool {
        if let Some(interest) = self.interests.read().unwrap().get(&client_id) {
            // 检查对象类型是否在兴趣列表中
            if !interest.object_types.contains(object_type) {
                return false;
            }

            // 检查位置是否在兴趣区域内
            match interest.area {
                InterestArea::Sphere { center, radius } => {
                    let distance = (object_pos - center).length();
                    distance <= radius
                }
                InterestArea::Cube { center, half_extent } => {
                    let delta = object_pos - center;
                    delta.x.abs() <= half_extent.x
                        && delta.y.abs() <= half_extent.y
                        && delta.z.abs() <= half_extent.z
                }
            }
        } else {
            false
        }
    }

    /// 获取对对象感兴趣的客户端列表
    pub fn get_interested_clients(
        &self,
        object_pos: Vec3,
        object_type: &str,
    ) -> Vec<u64> {
        self.interests
            .read()
            .unwrap()
            .iter()
            .filter(|(_, interest)| {
                // 检查对象类型是否在兴趣列表中
                if !interest.object_types.contains(object_type) {
                    return false;
                }

                // 检查位置是否在兴趣区域内
                match interest.area {
                    InterestArea::Sphere { center, radius } => {
                        (object_pos - center).length() <= radius
                    }
                    InterestArea::Cube { center, half_extent } => {
                        let delta = object_pos - center;
                        delta.x.abs() <= half_extent.x
                            && delta.y.abs() <= half_extent.y
                            && delta.z.abs() <= half_extent.z
                    }
                }
            })
            .map(|(client_id, _)| *client_id)
            .collect()
    }
}

/// 同步对象类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncObjectType {
    Player,
    NPC,
    Item,
    World,
}

/// 同步对象状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncObjectState {
    /// 对象ID
    pub object_id: u64,
    /// 对象类型
    pub object_type: SyncObjectType,
    /// 位置
    pub position: Vec3,
    /// 旋转
    pub rotation: Vec3,
    /// 速度
    pub velocity: Vec3,
    /// 加速度
    pub acceleration: Vec3,
    /// 缩放
    pub scale: Vec3,
    /// 状态数据
    pub state_data: HashMap<String, serde_json::Value>,
    /// 更新时间戳
    pub timestamp: u64,
    /// 服务器tick
    pub tick: u64,
}

impl_default!(SyncObjectState {
    object_id: 0,
    object_type: SyncObjectType::World,
    position: Vec3::ZERO,
    rotation: Vec3::ZERO,
    velocity: Vec3::ZERO,
    acceleration: Vec3::ZERO,
    scale: Vec3::ONE,
    state_data: HashMap::new(),
    timestamp: 0,
    tick: 0,
});

/// 同步配置
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// 同步帧率
    pub sync_fps: u32,
    /// 增量同步阈值
    pub delta_sync_threshold: f32,
    /// 兴趣管理更新间隔
    pub interest_update_interval_ms: u64,
    /// 最大同步延迟（毫秒）
    pub max_sync_delay_ms: u64,
    /// 是否启用兴趣管理
    pub enable_interest_management: bool,
}

impl_default!(SyncConfig {
    sync_fps: 30,
    delta_sync_threshold: 0.1,
    interest_update_interval_ms: 1000,
    max_sync_delay_ms: 200,
    enable_interest_management: true,
});

/// 网络同步管理器
pub struct NetworkSynchronizationManager {
    /// 配置
    config: SyncConfig,
    /// 当前服务器tick
    current_tick: Arc<Mutex<u64>>,
    /// 兴趣管理器
    interest_manager: Arc<InterestManager>,
    /// 对象状态历史
    object_state_history: Arc<Mutex<HashMap<u64, Vec<SyncObjectState>>>>,
    /// 同步数据缓冲区
    sync_buffer: Arc<Mutex<HashMap<u64, Vec<SyncObjectState>>>>,
    /// 客户端状态映射
    client_states: Arc<Mutex<HashMap<u64, Vec<SyncObjectState>>>>,
}

impl NetworkSynchronizationManager {
    pub fn new(config: SyncConfig) -> Self {
        Self {
            config,
            current_tick: Arc::new(Mutex::new(0)),
            interest_manager: Arc::new(InterestManager::new()),
            object_state_history: Arc::new(Mutex::new(HashMap::new())),
            sync_buffer: Arc::new(Mutex::new(HashMap::new())),
            client_states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 更新服务器tick
    pub fn update_tick(&self) {
        let mut current_tick = self.current_tick.lock().unwrap();
        *current_tick += 1;
    }

    /// 获取当前tick
    pub fn current_tick(&self) -> u64 {
        *self.current_tick.lock().unwrap()
    }

    /// 更新对象状态并准备同步
    pub fn update_object_state(&self, state: SyncObjectState) {
        let tick = self.current_tick();
        let mut state = state;
        state.tick = tick;

        // 更新对象状态历史
        let mut object_history = self.object_state_history.lock().unwrap();
        let history = object_history.entry(state.object_id).or_insert(Vec::new());
        history.push(state.clone());
        // 只保留最近的100个状态
        if history.len() > 100 {
            history.drain(0..history.len() - 100);
        }
        drop(object_history);

        // 准备同步数据
        if self.config.enable_interest_management {
            // 获取感兴趣的客户端
            let interested_clients = self.interest_manager.get_interested_clients(
                state.position,
                match state.object_type {
                    SyncObjectType::Player => "player",
                    SyncObjectType::NPC => "npc",
                    SyncObjectType::Item => "item",
                    SyncObjectType::World => "world",
                },
            );

            // 将数据添加到对应客户端的同步缓冲区
            let mut sync_buffer = self.sync_buffer.lock().unwrap();
            for client_id in interested_clients {
                let buffer = sync_buffer.entry(client_id).or_insert(Vec::new());
                buffer.push(state.clone());
            }
        } else {
            // 广播给所有客户端
            let mut sync_buffer = self.sync_buffer.lock().unwrap();
            let all_clients_buffer = sync_buffer.entry(0).or_insert(Vec::new());
            all_clients_buffer.push(state);
        }
    }

    /// 获取要同步给客户端的数据
    pub fn get_sync_data(&self, client_id: u64) -> Vec<SyncObjectState> {
        let mut sync_buffer = self.sync_buffer.lock().unwrap();
        if let Some(buffer) = sync_buffer.remove(&client_id) {
            buffer
        } else if let Some(buffer) = sync_buffer.remove(&0) {
            // 广播数据
            buffer
        } else {
            Vec::new()
        }
    }

    /// 更新客户端兴趣
    pub fn update_client_interest(&self, client_id: u64, interest: ClientInterest) {
        self.interest_manager.update_client_interest(client_id, interest);
    }

    /// 获取兴趣管理器
    pub fn interest_manager(&self) -> Arc<InterestManager> {
        Arc::clone(&self.interest_manager)
    }

    /// 获取对象状态历史
    pub fn get_object_state_history(&self, object_id: u64) -> Vec<SyncObjectState> {
        let object_history = self.object_state_history.lock().unwrap();
        if let Some(history) = object_history.get(&object_id) {
            history.clone()
        } else {
            Vec::new()
        }
    }

    /// 清除旧的状态历史
    pub fn clean_state_history(&self) {
        let mut object_history = self.object_state_history.lock().unwrap();
        for (_, history) in object_history.iter_mut() {
            // 只保留最近1秒的数据（假设帧率为100Hz）
            let one_second_ago = current_timestamp_ms() - 1000;
            history.retain(|state| state.timestamp >= one_second_ago);
        }
    }

    /// 预测对象的未来状态
    pub fn predict_future_state(
        &self,
        object_id: u64,
        future_tick: u64,
    ) -> Option<SyncObjectState> {
        let object_history = self.object_state_history.lock().unwrap();
        if let Some(history) = object_history.get(&object_id) {
            // 检查是否有足够的数据
            if history.len() < 2 {
                return None;
            }

            // 使用最新的两个状态来预测未来状态
            let latest = history.last()?;
            let previous = history.get(history.len() - 2)?;

            // 计算时间差（以tick为单位）
            let time_delta = latest.tick - previous.tick;
            if time_delta == 0 {
                return None;
            }

            // 计算状态变化率
            let pos_delta = (latest.position - previous.position) / time_delta as f32;
            let rot_delta = (latest.rotation - previous.rotation) / time_delta as f32;
            let vel_delta = (latest.velocity - previous.velocity) / time_delta as f32;
            let acc_delta = (latest.acceleration - previous.acceleration) / time_delta as f32;

            // 计算到未来tick的时间差
            let future_delta = future_tick - latest.tick;

            // 预测未来状态
            Some(SyncObjectState {
                object_id: latest.object_id,
                object_type: latest.object_type,
                position: latest.position + pos_delta * future_delta as f32,
                rotation: latest.rotation + rot_delta * future_delta as f32,
                velocity: latest.velocity + vel_delta * future_delta as f32,
                acceleration: latest.acceleration + acc_delta * future_delta as f32,
                scale: latest.scale,
                state_data: latest.state_data.clone(),
                timestamp: latest.timestamp + future_delta * (1000 / self.config.sync_fps as u64),
                tick: future_tick,
            })
        } else {
            None
        }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
<<<<<<< HEAD
    fn test_state_sync_manager() {
        let mut manager = StateSyncManager::new(10, 0.1);

        manager.register_entity(
            1,
            SyncStrategy::ClientPrediction,
            ConflictResolutionStrategy::ServerWins,
        );

        let state = EntityState::new(
            Vec3::new(0.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec3::ZERO,
        );

        assert!(manager.update_client_state(1, state, 5).is_ok());
    }

    #[test]
    fn test_conflict_detection() {
        let mut manager = StateSyncManager::new(10, 0.1);

        manager.register_entity(
            1,
            SyncStrategy::ClientPrediction,
            ConflictResolutionStrategy::ServerWins,
        );

        let client_state = EntityState::new(
            Vec3::new(0.0, 0.0, 0.0),
            Quat::IDENTITY,
            Vec3::ONE,
            Vec3::ZERO,
        );
        manager.update_client_state(1, client_state, 5).unwrap();

        let server_state = EntityState::new(
            Vec3::new(1.0, 0.0, 0.0), // 超出阈值
            Quat::IDENTITY,
            Vec3::ONE,
            Vec3::ZERO,
        );

        let resolution = manager.update_server_state(1, server_state, 5).unwrap();
        assert_eq!(resolution.conflict_type, ConflictType::StateMismatch);
    }

    #[test]
    fn test_event_sync_manager() {
        let mut manager = EventSyncManager::new(100, 5000);

        let event = NetworkEvent {
            event_id: 1,
            event_type: EventType::Fire,
            entity_id: Some(1),
            data: vec![1, 2, 3],
            timestamp: current_timestamp_ms(),
            sender_id: 1,
        };

        manager.add_event(event);
        assert_eq!(manager.get_unconfirmed_events().len(), 1);

        manager.confirm_event(1);
        assert_eq!(manager.get_unconfirmed_events().len(), 0);
=======
    fn test_sync_config() {
        let config = SyncConfig::default();
        assert_eq!(config.sync_fps, 30);
        assert!(config.enable_interest_management);
    }

    #[test]
    fn test_sync_object_state() {
        let state = SyncObjectState::default();
        assert_eq!(state.position, Vec3::ZERO);
        assert_eq!(state.scale, Vec3::ONE);
    }

    #[test]
    fn test_interest_manager() {
        let interest_manager = InterestManager::new();
        let client_id = 1;
        let interest = ClientInterest {
            client_id,
            area: InterestArea::Sphere { center: Vec3::ZERO, radius: 100.0 },
            object_types: vec!["player".to_string(), "item".to_string()].into_iter().collect(),
            last_updated: current_timestamp_ms(),
        };
        interest_manager.update_client_interest(client_id, interest);
        
        // 检查玩家位置在范围内
        assert!(interest_manager.is_object_in_interest(1, Vec3::ZERO, "player"));
        
        // 检查NPC位置不在范围内（NPC不在兴趣类型中）
        assert!(!interest_manager.is_object_in_interest(1, Vec3::ZERO, "npc"));
        
        // 检查位置超出范围
        assert!(!interest_manager.is_object_in_interest(1, Vec3::new(200.0, 0.0, 0.0), "player"));
    }

    #[test]
    fn test_network_synchronization_manager() {
        let config = SyncConfig::default();
        let sync_manager = NetworkSynchronizationManager::new(config);
        
        // 更新对象状态
        let state = SyncObjectState {
            object_id: 1,
            object_type: SyncObjectType::Player,
            position: Vec3::ZERO,
            rotation: Vec3::ZERO,
            velocity: Vec3::ZERO,
            acceleration: Vec3::ZERO,
            scale: Vec3::ONE,
            state_data: HashMap::new(),
            timestamp: current_timestamp_ms(),
            tick: 0,
        };
        sync_manager.update_object_state(state.clone());
        
        // 检查对象状态历史
        let history = sync_manager.get_object_state_history(1);
        assert_eq!(history.len(), 1);
        
        // 测试预测功能
        sync_manager.update_tick();
        let future_state = sync_manager.predict_future_state(1, 1);
        assert!(future_state.is_some());
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }
}
