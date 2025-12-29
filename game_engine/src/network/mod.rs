//! # 网络同步模块（Network Synchronization）
//!
//! 本模块提供完整的多人游戏网络同步框架，支持客户端-服务器架构。
//!
//! ## 核心组件
//!
//! ### 通信协议（Communication Protocols）
//! - [`Client`][]: 网络客户端
//! - [`Server`][]: 网络服务器
//! - [`parallel`][]: 并行消息处理
//! - [`webrtc`][]: WebRTC P2P通信
//!
//! ### 同步机制（Synchronization）
//! - [`delta_serialization`][]: Delta序列化，减少带宽占用
//! - [`priority_sync`][]: 优先级同步，重要状态优先
//! - [`interpolation`][]: 插值算法，平滑移动
//! - [`prediction`][]: 客户端预测
//! - [`replay`][]: 回放系统
//!
//! ### 网络优化（Network Optimization）
//! - [`compression`][]: 网络压缩
//! - [`delay_compensation`][]: 延迟补偿
//! - [`authority`][]: 服务器权威
//!
//! ### 安全性（Security）
//! - [`key_exchange`][]: 密钥交换（ECDH）
//! - [`security`][]: 消息加密和认证
//!
//! ## 架构设计
//!
//! ### 客户端-服务器架构
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐
//!  │     Client      │────►│     Server      │
//!  │                 │◄────│                 │
//!  │  Local State    │     │  Authoritative  │
//!  │  Prediction     │     │  State          │
//!  └─────────────────┘     └─────────────────┘
//! ```
//!
//! ### 同步流程
//!
//! 1. **客户端预测**: 立即应用玩家输入，预测结果
//! 2. **发送输入**: 将输入发送给服务器
//! 3. **服务器验证**: 服务器验证并计算权威状态
//! 4. **状态同步**: 服务器发送权威状态给客户端
//! 5. **回滚/协调**: 客户端根据服务器状态校正预测
//!
//! ## 使用示例
//!
//! ### 服务器端
//!
//! ```rust,no_run
//! use game_engine::network::{Server, ServerConfig};
//!
//! # async fn start_server() {
//! let config = ServerConfig::new("127.0.0.1:8080");
//! let mut server = Server::new(config).await.expect("Test: operation should succeed");
//!
//! // 启动服务器
//! server.listen().await.expect("Test: operation should succeed");
//! # }
//! ```
//!
//! ### 客户端
//!
//! ```rust,no_run
//! use game_engine::network::{Client, ClientConfig};
//!
//! # async fn connect_to_server() {
//! let config = ClientConfig::new("127.0.0.1:8080");
//! let mut client = Client::new(config).await.expect("Test: operation should succeed");
//!
//! // 连接服务器
//! client.connect().await.expect("Test: operation should succeed");
//!
//! // 发送输入
//! client.send_input(input).await;
//! # }
//! ```
//!
//! ## Delta序列化
//!
//! 只同步变化的数据，显著减少带宽占用：
//!
//! ```rust,no_run
//! use game_engine::network::delta_serialization::*;
//!
//! # fn serialize_delta() {
//! // 全量状态: 1000 bytes
//! let full_state = vec![0u8; 1000];
//!
//! // Delta: 只有100 bytes变化
//! let delta = DeltaEncoder::encode(&full_state, &previous_state);
//!
//! // 带宽节省: 90%
//! # }
//! ```
//!
//! ## 客户端预测
//!
//! 客户端预测玩家输入，减少延迟感知：
//!
//! ```text
//! 时间线:
//! t0: 玩家输入 → 客户端立即预测 → 渲染
//! t1: 发送到服务器
//! t2: 服务器处理 → 发回权威状态
//! t3: 客户端收到 → 回滚到服务器状态 → 重新预测
//! ```
//!
//! ## 延迟补偿
//!
//! 服务器补偿客户端延迟，确保公平性：
//!
//! ```text
//! 客户端A (ping 50ms)  →  服务器  → 客户端B (ping 100ms)
//!     │                                │
//!   t0                              t0
//!     │                              │
//!   t0+50                           t0
//!     │                              │
//!   t0+100                          t0+100
//! ```
//!
//! ## 性能优化
//!
//! - **Delta序列化**: 减少70-90%带宽
//! - **优先级同步**: 重要状态优先发送
//! - **压缩**: 进一步减少数据量
//! - **批处理**: 减少packet数量
//! - **插值**: 平滑显示，减少抖动
//!
//! ## 安全特性
//!
//! - **密钥交换**: ECDH (elliptic-curve Diffie-Hellman)
//! - **消息加密**: AES加密网络消息
//! - **防作弊**: 服务器权威验证
//! - **重放攻击防护**: 时间戳和序列号
//!
//! ## WebRTC支持
//!
//! 支持点对点连接，适合浏览器游戏：
//!
//! ```rust,no_run
//! use game_engine::network::webrtc::WebRTCConnection;
//!
//! # async fn p2p_connect() {
//! // 创建WebRTC连接
//! let conn = WebRTCConnection::new().await.expect("Test: operation should succeed");
//!
//! // P2P通信
//! conn.send(data).await.expect("Test: operation should succeed");
//! # }
//! ```
//!
//! ## 相关模块
//!
//! - [`crate::domain`][]: 领域事件用于网络同步
//! - [`crate::physics`][]: 物理状态同步
//! - [`crate::ecs`][]: ECS网络组件
//!

pub mod client;
pub mod compression;
pub mod delay_compensation;
pub mod delta_serialization;
pub mod interpolation;
pub mod key_exchange;
/// 统一网络同步管理器（整合状态和事件同步）
pub mod network_sync;
pub mod network_sync_enhanced;
/// 并行网络消息处理
/// 并行功能默认启用，使用线程池进行并行消息处理
pub mod parallel;
pub mod prediction;
pub mod priority_sync;
pub mod replay;
pub mod security;
pub mod server;
pub mod synchronization;
/// WebRTC网络协议支持
pub mod webrtc;

use crate::impl_default;

// Re-export key exchange types
pub use key_exchange::{
    KeyExchange, KeyExchangeMessage, KeyExchangeProtocol, KeyPair, SharedSecret,
};

// Re-export priority sync types
pub use network_sync_enhanced::{
    ClientInterpolator, EnhancedNetworkSync, EnhancedNetworkSyncConfig, InterpolationStats,
    NetworkQuality, NetworkSyncPerformanceStats, PacketRecoveryStrategy, QualityLevel,
    RetransmissionStats,
};
pub use priority_sync::{
    BandwidthBudget, BandwidthStats, EntitySyncInfo, PrioritySyncManager, SyncPriority,
};

// Re-export delta serialization types (including enhanced features)
pub use delta_serialization::{
    BatchDeltaSerializer, DeltaPacket, DeltaSerializer, EntityDelta, QuantizationConfig,
    QuantizedEntityDelta, Quantizer,
};

// 向后兼容：Enhanced类型现在指向基础版本的增强功能
/// 增强的增量序列化器（向后兼容别名）
///
/// 注意：增强功能已整合到`DeltaSerializer`中，通过`enable_quantization`方法启用。
/// 保留此类型别名以保持向后兼容。
#[deprecated(
    since = "0.1.0",
    note = "Use DeltaSerializer with quantization enabled instead. This type is kept for backward compatibility only."
)]
pub type EnhancedDeltaSerializer = DeltaSerializer;

// Re-export replay types
pub use replay::{
    ReplayConfig, ReplayError, ReplayFrame, ReplayHeader, ReplayPlayer, ReplayRecorder,
    ReplaySnapshot, ReplayState, TimeTravelDebugger,
};

// Re-export WebRTC types
use bevy_ecs::prelude::*;
use crossbeam_channel::{Receiver, Sender, unbounded};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, TcpStream};
use thiserror::Error;
pub use webrtc::{
    DataChannelConfig, IceConnectionState, IceGatheringState, IceTransportPolicy, SignalingHandler,
    SignalingMessage, WebRtcConfig, WebRtcConnectionState, WebRtcError, WebRtcManager,
    WebRtcPeerConnection,
};

// Re-export unified network sync manager (整合状态和事件同步)
pub use network_sync::{
    ConflictResolution, ConflictResolutionStrategy, ConflictType, EntityState, EntitySyncState,
    EventType, NetworkSyncConfig, NetworkSyncManager, NetworkSyncStats, ResolutionAction,
    SyncNetworkEvent, SyncStrategy,
};

/// 网络错误类型
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("Connection failed: {0}")]
    ConnectionError(String),
    #[error("Send failed: {0}")]
    SendError(String),
    #[error("Receive failed: {0}")]
    ReceiveError(String),
    #[error("Serialization failed: {0}")]
    SerializationError(String),
    #[error("Delta serialization failed: {0}")]
    DeltaSerializationError(String),
    #[error("Compression failed: {0}")]
    CompressionError(String),
    #[error("Invalid peer ID")]
    InvalidPeerId,
    #[error("Lock acquisition failed")]
    LockAcquisitionFailed,
    #[error("Sync operation called in runtime: {0}")]
    SyncOperationInRuntime(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// 连接请求
    Connect { client_id: u64, name: String },
    /// 断开连接
    Disconnect { client_id: u64 },
    /// 状态同步
    StateSync { tick: u64, data: Vec<u8> },
    /// RPC 调用
    Rpc {
        id: u32,
        method: String,
        params: Vec<u8>,
    },
    /// RPC 响应
    RpcResponse { id: u32, result: Vec<u8> },
    /// 心跳
    Heartbeat { timestamp: u64 },
    /// 输入同步
    Input { tick: u64, inputs: Vec<u8> },
    /// 时间同步请求
    TimeSyncRequest { client_send_time: u64 },
    /// 时间同步响应
    TimeSyncResponse {
        sync: delay_compensation::TimeSyncMessage,
    },
    /// 事件同步
    EventSync {
        events: Vec<synchronization::NetworkEvent>,
    },
}

/// 网络连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    /// 断开连接
    #[default]
    Disconnected,
    /// 连接中
    Connecting,
    /// 已连接
    Connected,
    /// 重连中
    Reconnecting,
}

/// 网络统计信息
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    /// 延迟 (毫秒)
    pub latency_ms: f32,
    /// 丢包率
    pub packet_loss: f32,
    /// 发送字节数
    pub bytes_sent: u64,
    /// 接收字节数
    pub bytes_received: u64,
    /// 发送消息数
    pub messages_sent: u64,
    /// 接收消息数
    pub messages_received: u64,
}

/// 网络事件
#[derive(Event)]
pub enum NetworkEvent {
    Connected { peer_id: u64 },
    Disconnected { peer_id: u64 },
    Message { peer_id: u64, data: Vec<u8> },
}

/// 网络配置
#[derive(Resource)]
pub struct NetworkConfig {
    pub server_address: String,
    pub port: u16,
    pub max_connections: usize,
}

impl_default!(NetworkConfig {
    server_address: "127.0.0.1".to_string(),
    port: 8080,
    max_connections: 100,
});

/// 网络管理器
pub struct NetworkManager {
    config: NetworkConfig,
    connections: HashMap<u64, Connection>,
}

/// 连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub peer_id: u64,
    pub address: SocketAddr,
    pub state: ConnectionState,
}

struct Connection {
    peer_id: u64,
    address: SocketAddr,
    state: ConnectionState,
}

impl NetworkManager {
    pub fn new(config: NetworkConfig) -> Result<Self, NetworkError> {
        // 创建 TCP 监听器
        let address = format!("{}:{}", config.server_address, config.port);
        match TcpStream::connect(&address) {
            Ok(_) => Ok(Self {
                config,
                connections: HashMap::new(),
            }),
            Err(e) => Err(NetworkError::ConnectionError(e.to_string())),
        }
    }

    pub fn connect_to_server(&mut self, address: &str) -> Result<u64, NetworkError> {
        let socket_addr: SocketAddr = address
            .parse()
            .map_err(|_| NetworkError::ConnectionError("Invalid address format".to_string()))?;

        // NOTE: 客户端连接逻辑待实现，当前使用简化实现
        let peer_id = rand::random();
        let connection = Connection {
            peer_id,
            address: socket_addr,
            state: ConnectionState::Connecting,
        };

        self.connections.insert(peer_id, connection);
        Ok(peer_id)
    }

    /// 获取网络配置，形成逻辑闭环
    pub fn config(&self) -> &NetworkConfig {
        &self.config
    }

    /// 获取连接信息
    pub fn get_connection_info(&self, peer_id: u64) -> Option<ConnectionInfo> {
        self.connections.get(&peer_id).map(|conn| ConnectionInfo {
            peer_id: conn.peer_id,
            address: conn.address,
            state: conn.state,
        })
    }

    /// 获取所有连接的摘要
    pub fn get_connections_summary(&self) -> Vec<(u64, SocketAddr, ConnectionState)> {
        self.connections
            .values()
            .map(|conn| (conn.peer_id, conn.address, conn.state))
            .collect()
    }
}

/// 网络客户端状态 (Resource)
#[derive(Resource, Default)]
pub struct NetworkState {
    /// 连接状态
    pub connection_state: ConnectionState,
    /// 客户端 ID
    pub client_id: Option<u64>,
    /// 服务器地址
    pub server_addr: Option<SocketAddr>,
    /// 网络统计
    pub stats: NetworkStats,
    /// 当前 tick
    pub current_tick: u64,
    /// 消息发送通道
    pub(crate) send_tx: Option<Sender<NetworkMessage>>,
    /// 消息接收通道
    pub(crate) recv_rx: Option<Receiver<NetworkMessage>>,
    /// 增量序列化器（用于增量序列化）
    pub(crate) delta_serializer:
        Option<std::sync::Arc<std::sync::Mutex<delta_serialization::DeltaSerializer>>>,
    /// 压缩器（用于网络数据压缩）
    pub(crate) compressor: Option<std::sync::Arc<compression::NetworkCompressor>>,
    /// 延迟补偿管理器（客户端）
    pub(crate) delay_compensation:
        Option<std::sync::Arc<std::sync::Mutex<delay_compensation::ClientDelayCompensation>>>,
    /// 重连尝试次数（用于跟踪客户端重连历史）
    pub(crate) reconnect_attempts: usize,
}

impl NetworkState {
    pub fn new() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            client_id: None,
            server_addr: None,
            stats: Default::default(),
            current_tick: 0,
            send_tx: None,
            recv_rx: None,
            delta_serializer: None,
            compressor: None,
            delay_compensation: None,
            reconnect_attempts: 0,
        }
    }

    /// 获取当前重连尝试次数
    pub fn get_reconnect_attempts(&self) -> usize {
        self.reconnect_attempts
    }

    /// 增加重连尝试计数
    pub fn increment_reconnect_attempts(&mut self) {
        self.reconnect_attempts += 1;
    }

    /// 重置重连尝试计数
    pub fn reset_reconnect_attempts(&mut self) {
        self.reconnect_attempts = 0;
    }
}

/// 网络服务 - 封装网络业务逻辑
pub struct NetworkService;

impl NetworkService {
    /// 异步连接到服务器
    pub async fn connect_async(state: &mut NetworkState, addr: SocketAddr) -> Result<(), String> {
        if state.connection_state != ConnectionState::Disconnected {
            return Err("Already connected or connecting".to_string());
        }

        state.connection_state = ConnectionState::Connecting;
        state.server_addr = Some(addr);

        // 创建通道
        let (send_tx, _send_rx) = unbounded::<NetworkMessage>();
        let (_recv_tx, recv_rx) = unbounded::<NetworkMessage>();

        state.send_tx = Some(send_tx);
        state.recv_rx = Some(recv_rx);

        // 初始化压缩器（如果尚未初始化）
        if state.compressor.is_none() {
            state.compressor = Some(std::sync::Arc::new(compression::NetworkCompressor::new()));
        }

        // 初始化延迟补偿管理器（如果尚未初始化）
        if state.delay_compensation.is_none() {
            state.delay_compensation = Some(std::sync::Arc::new(std::sync::Mutex::new(
                delay_compensation::ClientDelayCompensation::new(),
            )));
        }

        // NOTE: 网络线程启动逻辑待实现，当前仅设置连接状态
        state.connection_state = ConnectionState::Connected;
        state.client_id = Some(rand::random());

        Ok(())
    }

    /// 连接到服务器（同步版本，向后兼容）
    pub fn connect(state: &mut NetworkState, addr: SocketAddr) -> Result<(), String> {
        if state.connection_state != ConnectionState::Disconnected {
            return Err("Already connected or connecting".to_string());
        }

        state.connection_state = ConnectionState::Connecting;
        state.server_addr = Some(addr);

        // 创建通道
        let (send_tx, _send_rx) = unbounded::<NetworkMessage>();
        let (_recv_tx, recv_rx) = unbounded::<NetworkMessage>();

        state.send_tx = Some(send_tx);
        state.recv_rx = Some(recv_rx);

        // 初始化压缩器（如果尚未初始化）
        if state.compressor.is_none() {
            state.compressor = Some(std::sync::Arc::new(compression::NetworkCompressor::new()));
        }

        // 初始化延迟补偿管理器（如果尚未初始化）
        if state.delay_compensation.is_none() {
            state.delay_compensation = Some(std::sync::Arc::new(std::sync::Mutex::new(
                delay_compensation::ClientDelayCompensation::new(),
            )));
        }

        // NOTE: 网络线程启动逻辑待实现，当前仅设置连接状态
        state.connection_state = ConnectionState::Connected;
        state.client_id = Some(rand::random());

        Ok(())
    }

    /// 启用压缩（可配置压缩级别）
    pub fn enable_compression(state: &mut NetworkState, level: compression::CompressionLevel) {
        state.compressor = Some(std::sync::Arc::new(
            compression::NetworkCompressor::with_level(level),
        ));
    }

    /// 禁用压缩
    pub fn disable_compression(state: &mut NetworkState) {
        state.compressor = None;
    }

    /// 检查是否启用压缩
    pub fn is_compression_enabled(state: &NetworkState) -> bool {
        state.compressor.is_some()
    }

    /// 发送时间同步请求
    pub fn send_time_sync_request(state: &NetworkState) -> Result<(), String> {
        if let Some(ref compensation) = state.delay_compensation {
            let compensation_guard = compensation.lock().map_err(|e| e.to_string())?;
            let sync_request = compensation_guard.create_sync_request();
            Self::send(
                state,
                NetworkMessage::TimeSyncRequest {
                    client_send_time: sync_request.client_send_time,
                },
            )
        } else {
            Err("Delay compensation not initialized".to_string())
        }
    }

    /// 获取延迟补偿统计
    pub fn get_delay_compensation_stats(
        state: &NetworkState,
    ) -> Option<delay_compensation::LatencyStats> {
        state
            .delay_compensation
            .as_ref()
            .and_then(|c| c.lock().ok().map(|guard| guard.latency_stats()))
    }

    /// 检查是否需要时间同步
    pub fn should_sync_time(state: &NetworkState) -> bool {
        state
            .delay_compensation
            .as_ref()
            .is_some_and(|c| c.lock().ok().is_some_and(|guard| guard.should_sync()))
    }

    /// 断开连接
    pub fn disconnect(state: &mut NetworkState) {
        if let Some(tx) = &state.send_tx
            && let Some(client_id) = state.client_id
        {
            let _ = tx.send(NetworkMessage::Disconnect { client_id });
        }

        state.connection_state = ConnectionState::Disconnected;
        state.client_id = None;
        state.send_tx = None;
        state.recv_rx = None;
    }

    /// 发送消息
    pub fn send(state: &NetworkState, message: NetworkMessage) -> Result<(), String> {
        if let Some(tx) = &state.send_tx {
            tx.send(message).map_err(|e| e.to_string())
        } else {
            Err("Not connected".to_string())
        }
    }

    /// 发送 RPC 调用
    pub fn rpc_call(state: &NetworkState, method: &str, params: &[u8]) -> Result<u32, String> {
        let id = rand::random();
        Self::send(
            state,
            NetworkMessage::Rpc {
                id,
                method: method.to_string(),
                params: params.to_vec(),
            },
        )?;
        Ok(id)
    }

    /// 发送状态同步（可选压缩）
    pub fn sync_state(state: &NetworkState, data: &[u8]) -> Result<(), String> {
        // 如果启用了压缩，先压缩数据
        let final_data = if let Some(ref compressor) = state.compressor {
            compressor
                .compress_with_flag(data)
                .map_err(|e| format!("Compression failed: {}", e))?
        } else {
            data.to_vec()
        };

        Self::send(
            state,
            NetworkMessage::StateSync {
                tick: state.current_tick,
                data: final_data,
            },
        )
    }

    /// 发送状态同步（强制压缩）
    pub fn sync_state_compressed(state: &NetworkState, data: &[u8]) -> Result<(), String> {
        let compressor = compression::NetworkCompressor::new();
        let compressed = compressor
            .compress_with_flag(data)
            .map_err(|e| format!("Compression failed: {}", e))?;

        Self::send(
            state,
            NetworkMessage::StateSync {
                tick: state.current_tick,
                data: compressed,
            },
        )
    }

    /// 发送输入
    pub fn send_input(state: &NetworkState, inputs: &[u8]) -> Result<(), String> {
        Self::send(
            state,
            NetworkMessage::Input {
                tick: state.current_tick,
                inputs: inputs.to_vec(),
            },
        )
    }

    /// 接收消息
    pub fn receive(state: &NetworkState) -> Vec<NetworkMessage> {
        let mut messages = Vec::new();
        if let Some(rx) = &state.recv_rx {
            while let Ok(msg) = rx.try_recv() {
                messages.push(msg);
            }
        }
        messages
    }

    /// 获取连接状态
    pub fn is_connected(state: &NetworkState) -> bool {
        state.connection_state == ConnectionState::Connected
    }

    /// 获取延迟
    pub fn get_latency(state: &NetworkState) -> f32 {
        state.stats.latency_ms
    }
}

/// 网络组件 - 标记需要网络同步的实体
#[derive(Component, Clone)]
pub struct NetworkEntity {
    /// 网络 ID（全局唯一）
    pub net_id: u64,
    /// 所有者客户端 ID
    pub owner_id: u64,
    /// 是否本地控制
    pub is_local: bool,
}

/// 网络同步组件 - 存储同步数据
#[derive(Component, Clone)]
pub struct NetworkSync {
    /// 最后同步的 tick
    pub last_sync_tick: u64,
    /// 同步间隔
    pub sync_interval: u64,
    /// 同步优先级
    pub priority: u8,
}

impl_default!(NetworkSync {
    last_sync_tick: 0,
    sync_interval: 1,
    priority: 128,
});

// ============================================================================
// ECS 系统
// ============================================================================

/// 网络更新系统
pub fn network_update_system(mut state: ResMut<NetworkState>) {
    state.current_tick += 1;

    let messages = NetworkService::receive(&state);
    for msg in messages {
        match msg {
            NetworkMessage::Connect { client_id: _, name } => {
                log::info!("Client connected: {}", name);
            }
            NetworkMessage::Disconnect { client_id } => {
                log::info!("Client disconnected: {}", client_id);
            }
            NetworkMessage::StateSync { tick, data } => {
                let decompressed_data = if let Some(ref compressor) = state.compressor {
                    compressor.decompress_with_flag(&data).unwrap_or_else(|_| data.clone())
                } else if !data.is_empty() && data[0] == 1 {
                    let temp_compressor = compression::NetworkCompressor::new();
                    temp_compressor.decompress_with_flag(&data).unwrap_or_else(|_| data.clone())
                } else {
                    data.clone()
                };

                if let Ok(delta_packet) =
                    bincode::deserialize::<delta_serialization::DeltaPacket>(&decompressed_data)
                {
                    for delta in delta_packet.deltas {
                        log::debug!(
                            "Received state update for entity {} at tick {}",
                            delta.id,
                            tick
                        );
                    }
                }
            }
            NetworkMessage::Rpc { id, method, params } => {
                log::debug!("RPC call: {} (id: {}), params: {:?}", method, id, params);
            }
            NetworkMessage::RpcResponse { id, result } => {
                log::debug!("RPC response for id {}: {:?}", id, result);
            }
            NetworkMessage::Heartbeat { timestamp } => {
                let now = crate::core::utils::current_timestamp_ms();
                state.stats.latency_ms = (now - timestamp) as f32;
            }
            NetworkMessage::Input { tick, inputs } => {
                log::debug!("Received input for tick {}: {} bytes", tick, inputs.len());
            }
            NetworkMessage::TimeSyncRequest { client_send_time } => {
                log::debug!("Time sync request from client: {}", client_send_time);
            }
            NetworkMessage::TimeSyncResponse { mut sync } => {
                if let Some(ref compensation) = state.delay_compensation
                    && let Ok(mut guard) = compensation.lock()
                {
                    guard.process_time_sync(&mut sync);
                }
            }
            NetworkMessage::EventSync { events } => {
                for event in events {
                    log::debug!(
                        "Received event: {:?} for entity: {:?}",
                        event.event_type,
                        event.entity_id
                    );
                }
            }
        }
    }
}

/// 网络同步发送系统（使用增量序列化）
pub fn network_sync_send_system(
    mut state: ResMut<NetworkState>,
    mut query: Query<(&NetworkEntity, &mut NetworkSync, &crate::ecs::Transform)>,
) {
    if !NetworkService::is_connected(&state) {
        return;
    }

    // 获取或创建增量序列化器
    if state.delta_serializer.is_none() {
        state.delta_serializer = Some(std::sync::Arc::new(std::sync::Mutex::new(
            delta_serialization::DeltaSerializer::new(),
        )));
    }

    let serializer = match state.delta_serializer.as_ref() {
        Some(s) => s,
        None => {
            eprintln!("Delta serializer not initialized");
            return;
        }
    };

    let mut serializer_guard = match serializer.lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Failed to lock delta serializer: {}", e);
            return;
        }
    };

    // 收集需要同步的实体
    let mut entities_to_sync = Vec::new();
    let mut entities_to_update = Vec::new();

    for (net_entity, sync, transform) in query.iter() {
        if !net_entity.is_local {
            continue;
        }

        if state.current_tick - sync.last_sync_tick < sync.sync_interval {
            continue;
        }

        // 创建实体增量数据
        let mut delta = delta_serialization::EntityDelta::new(net_entity.net_id);
        delta.position = Some(transform.pos.to_array());
        delta.rotation = Some([
            transform.rot.x,
            transform.rot.y,
            transform.rot.z,
            transform.rot.w,
        ]);
        delta.scale = Some(transform.scale.to_array());

        entities_to_sync.push(delta);
        entities_to_update.push(net_entity.net_id);
    }

    // 计算增量并序列化
    if !entities_to_sync.is_empty() {
        let delta_packet = serializer_guard.compute_delta(&entities_to_sync);

        // 序列化增量数据
        if let Ok(data) = serializer_guard.serialize_delta(&delta_packet) {
            let _ = NetworkService::sync_state(&state, &data);

            // 更新同步tick
            for (net_entity, mut sync, _) in query.iter_mut() {
                if entities_to_update.contains(&net_entity.net_id) {
                    sync.last_sync_tick = state.current_tick;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_state_default() {
        let state = NetworkState::default();
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
        assert!(state.client_id.is_none());
    }

    #[test]
    fn test_network_stats_default() {
        let stats = NetworkStats::default();
        assert_eq!(stats.latency_ms, 0.0);
        assert_eq!(stats.bytes_sent, 0);
    }

    #[test]
    fn test_network_sync_default() {
        let sync = NetworkSync::default();
        assert_eq!(sync.last_sync_tick, 0);
        assert_eq!(sync.sync_interval, 1);
    }
}
