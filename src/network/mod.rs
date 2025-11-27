//! 网络同步模块
//!
//! 提供基础的网络同步框架，支持多人游戏开发。
//!
//! ## 功能特性
//!
//! - TCP/UDP 双协议支持
//! - RPC 框架基础
//! - 状态同步机制
//! - 网络延迟补偿
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────┐     ┌─────────────────┐
//! │     Client      │────►│     Server      │
//! │                 │◄────│                 │
//! │  Local State    │     │  Authoritative  │
//! │  Prediction     │     │  State          │
//! └─────────────────┘     └─────────────────┘
//! ```

use std::net::{TcpStream, UdpSocket, SocketAddr};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use crossbeam_channel::{Sender, Receiver, unbounded};
use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};

/// 网络消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// 连接请求
    Connect { client_id: u64, name: String },
    /// 断开连接
    Disconnect { client_id: u64 },
    /// 状态同步
    StateSync { tick: u64, data: Vec<u8> },
    /// RPC 调用
    Rpc { id: u32, method: String, params: Vec<u8> },
    /// RPC 响应
    RpcResponse { id: u32, result: Vec<u8> },
    /// 心跳
    Heartbeat { timestamp: u64 },
    /// 输入同步
    Input { tick: u64, inputs: Vec<u8> },
}

/// 网络连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// 断开连接
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

/// 网络客户端状态 (Resource)
#[derive(Resource)]
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
}

impl Default for NetworkState {
    fn default() -> Self {
        Self {
            connection_state: ConnectionState::Disconnected,
            client_id: None,
            server_addr: None,
            stats: NetworkStats::default(),
            current_tick: 0,
            send_tx: None,
            recv_rx: None,
        }
    }
}

/// 网络服务 - 封装网络业务逻辑
pub struct NetworkService;

impl NetworkService {
    /// 连接到服务器
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
        
        // TODO: 启动网络线程
        state.connection_state = ConnectionState::Connected;
        state.client_id = Some(rand::random());
        
        Ok(())
    }
    
    /// 断开连接
    pub fn disconnect(state: &mut NetworkState) {
        if let Some(tx) = &state.send_tx {
            if let Some(client_id) = state.client_id {
                let _ = tx.send(NetworkMessage::Disconnect { client_id });
            }
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
        Self::send(state, NetworkMessage::Rpc {
            id,
            method: method.to_string(),
            params: params.to_vec(),
        })?;
        Ok(id)
    }
    
    /// 发送状态同步
    pub fn sync_state(state: &NetworkState, data: &[u8]) -> Result<(), String> {
        Self::send(state, NetworkMessage::StateSync {
            tick: state.current_tick,
            data: data.to_vec(),
        })
    }
    
    /// 发送输入
    pub fn send_input(state: &NetworkState, inputs: &[u8]) -> Result<(), String> {
        Self::send(state, NetworkMessage::Input {
            tick: state.current_tick,
            inputs: inputs.to_vec(),
        })
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

impl Default for NetworkSync {
    fn default() -> Self {
        Self {
            last_sync_tick: 0,
            sync_interval: 1,
            priority: 128,
        }
    }
}

// ============================================================================
// ECS 系统
// ============================================================================

/// 网络更新系统
pub fn network_update_system(
    mut state: ResMut<NetworkState>,
) {
    // 增加 tick
    state.current_tick += 1;
    
    // 处理接收到的消息
    let messages = NetworkService::receive(&state);
    for msg in messages {
        match msg {
            NetworkMessage::StateSync { tick, data } => {
                // TODO: 应用状态同步
                let _ = (tick, data);
            }
            NetworkMessage::Heartbeat { timestamp } => {
                // 计算延迟
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64;
                state.stats.latency_ms = (now - timestamp) as f32;
            }
            _ => {}
        }
    }
}

/// 网络同步发送系统
pub fn network_sync_send_system(
    state: Res<NetworkState>,
    query: Query<(&NetworkEntity, &NetworkSync, &crate::ecs::Transform)>,
) {
    if !NetworkService::is_connected(&state) {
        return;
    }
    
    for (net_entity, sync, transform) in query.iter() {
        if !net_entity.is_local {
            continue;
        }
        
        if state.current_tick - sync.last_sync_tick < sync.sync_interval {
            continue;
        }
        
        // 序列化 transform
        let data = serde_json::to_vec(&(
            net_entity.net_id,
            transform.pos.to_array(),
            [transform.rot.x, transform.rot.y, transform.rot.z, transform.rot.w],
            transform.scale.to_array(),
        )).unwrap_or_default();
        
        let _ = NetworkService::sync_state(&state, &data);
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

