//! 网络客户端模块
//!
//! 实现游戏客户端的核心功能，包括：
//! - 服务器连接管理
//! - 消息发送和接收
//! - 客户端状态管理
//! - 重连机制
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           Game Client                   │
//! ├─────────────────────────────────────────┤
//! │  ┌──────────┐  ┌──────────┐  ┌─────────┐│
//! │  │ Input    │  │ Prediction│ │ Render ││
//! │  └────┬─────┘  └────┬─────┘  └────┬────┘│
//! │       │             │             │     │
//! │       └─────────────┼─────────────┘     │
//! │                     │                   │
//! │              ┌──────▼──────┐            │
//! │              │   Network   │            │
//! │              │   Client    │            │
//! │              └──────┬──────┘            │
//! │                     │                   │
//! │              ┌──────▼──────┐            │
//! │              │   Server    │            │
//! │              └─────────────┘            │
//! └─────────────────────────────────────────┘
//! ```

use crate::core::utils::current_timestamp_ms;
use crate::impl_default;
use crate::network::compression;
use crate::network::delay_compensation;
use crate::network::{ConnectionState, NetworkError, NetworkMessage, NetworkState};
use async_trait::async_trait;
use bincode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream as TokioTcpStream};
use tokio::sync::Mutex;
use tokio::task;
use std::net::{SocketAddr, TcpStream as StdTcpStream};

/// 客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// 服务器地址
    pub server_address: String,
    /// 服务器端口
    pub server_port: u16,
    /// 重连间隔（毫秒）
    pub reconnect_interval_ms: u64,
    /// 最大重连次数
    pub max_reconnect_attempts: usize,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用延迟补偿
    pub enable_delay_compensation: bool,
    /// 客户端名称
    pub client_name: String,
}

impl_default!(ClientConfig {
    server_address: "127.0.0.1".to_string(),
    server_port: 8080,
    reconnect_interval_ms: 5000,
    max_reconnect_attempts: 5,
    enable_compression: true,
    enable_delay_compensation: true,
    client_name: "Client".to_string(),
});

/// 游戏客户端
pub struct GameClient {
    /// 配置
    config: ClientConfig,
    /// 网络状态
    state: Arc<Mutex<NetworkState>>,
    /// TCP流
    stream: Arc<Mutex<Option<TokioTcpStream>>>,
    /// 延迟补偿管理器
    delay_compensation: Arc<Mutex<delay_compensation::ClientDelayCompensation>>,
    /// 压缩器
    compressor: Option<Arc<compression::NetworkCompressor>>,
    /// 是否运行中
    running: Arc<Mutex<bool>>,
    /// 重连尝试次数
    reconnect_attempts: Arc<Mutex<usize>>,
}

impl GameClient {
    /// 创建新的游戏客户端
    pub fn new(config: ClientConfig) -> Self {
        let compressor = if config.enable_compression {
            Some(Arc::new(compression::NetworkCompressor::new()))
        } else {
            None
        };

        let mut network_state = NetworkState::default();
        if config.enable_compression {
            network_state.compressor = compressor.as_ref().map(|c| Arc::clone(c));
        }
        if config.enable_delay_compensation {
            network_state.delay_compensation = Some(Arc::new(Mutex::new(
                delay_compensation::ClientDelayCompensation::new(),
            )));
        }

        Self {
            config,
            state: Arc::new(Mutex::new(network_state)),
            stream: Arc::new(Mutex::new(None)),
            delay_compensation: Arc::new(Mutex::new(
                delay_compensation::ClientDelayCompensation::new(),
            )),
            compressor,
            running: Arc::new(Mutex::new(false)),
            reconnect_attempts: Arc::new(Mutex::new(0)),
        }
    }

    /// 异步连接到服务器
    pub async fn connect(&self) -> Result<(), NetworkError> {
        let address = format!("{}:{}", self.config.server_address, self.config.server_port);
        let addr: SocketAddr = address
            .parse()
            .map_err(|e| NetworkError::ConnectionError(format!("Invalid address: {}", e)))?;

        match TokioTcpStream::connect(&addr).await {
            Ok(stream) => {
                *self.stream.lock().await = Some(stream);

                let mut state_guard = self.state.lock().await;
                state_guard.connection_state = ConnectionState::Connected;
                state_guard.server_addr = Some(addr);
                state_guard.client_id = Some(rand::random());

                // 发送连接请求
                let connect_msg = NetworkMessage::Connect {
                    client_id: state_guard.client_id.unwrap(),
                    name: self.config.client_name.clone(),
                };
                self.send_message(&connect_msg).await?;

                // 启动接收任务
                let stream_clone = Arc::clone(&self.stream);
                let state_clone = Arc::clone(&self.state);
                let running_clone = Arc::clone(&self.running);

                *self.running.lock().await = true;
                task::spawn(async move {
                    Self::receive_loop_async(stream_clone, state_clone, running_clone).await;
                });

                // 启动心跳任务
                let stream_clone = Arc::clone(&self.stream);
                let running_clone = Arc::clone(&self.running);
                task::spawn(async move {
                    Self::heartbeat_loop_async(stream_clone, running_clone).await;
                });

                Ok(())
            }
            Err(e) => Err(NetworkError::ConnectionError(format!(
                "Connection failed: {}",
                e
            ))),
        }
    }

    /// 异步断开连接
    pub async fn disconnect(&self) -> Result<(), NetworkError> {
        *self.running.lock().await = false;

        if let Some(mut stream) = self.stream.lock().await.take() {
            let disconnect_msg = NetworkMessage::Disconnect {
                client_id: self.state.lock().await.client_id.unwrap_or(0),
            };
            let _ = Self::send_to_stream_async(&mut stream, &disconnect_msg).await;
            let _ = stream.shutdown().await;
        }

        let mut state_guard = self.state.lock().await;
        state_guard.connection_state = ConnectionState::Disconnected;

        Ok(())
    }

    /// 异步发送消息
    pub async fn send_message(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        if let Some(ref mut stream) = *self.stream.lock().await {
            Self::send_to_stream_async(stream, message).await
        } else {
            Err(NetworkError::SendError("Not connected".to_string()))
        }
    }

    /// 异步发送消息到流
    async fn send_to_stream_async(
        stream: &mut TokioTcpStream,
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let data = bincode::serialize(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // 发送消息
        stream.write_all(&data).await
            .map_err(|e| NetworkError::SendError(e.to_string()))
    }

    /// 同步版本的连接方法（向后兼容）
    pub fn connect_sync(&mut self) -> Result<(), NetworkError> {
        let address = format!("{}:{}", self.config.server_address, self.config.server_port);
        let addr: SocketAddr = address
            .parse()
            .map_err(|e| NetworkError::ConnectionError(format!("Invalid address: {}", e)))?;

        match StdTcpStream::connect(&addr) {
            Ok(stream) => {
                stream.set_nonblocking(true).map_err(|e| {
                    NetworkError::ConnectionError(format!("Failed to set nonblocking: {}", e))
                })?;

                *crate::error::safe_lock(&self.stream, "client_stream").unwrap() = Some(stream.into());

                let mut state_guard = crate::error::safe_lock(&self.state, "client_state").unwrap();
                state_guard.connection_state = ConnectionState::Connected;
                state_guard.server_addr = Some(addr);
                state_guard.client_id = Some(rand::random());

                // 发送连接请求
                let connect_msg = NetworkMessage::Connect {
                    client_id: state_guard.client_id.unwrap(),
                    name: self.config.client_name.clone(),
                };
                self.send_message_sync(&connect_msg)?;

                // 启动接收线程
                let stream_clone = Arc::clone(&self.stream);
                let state_clone = Arc::clone(&self.state);
                let running_clone = Arc::clone(&self.running);

                *crate::error::safe_lock(&self.running, "client_running").unwrap() = true;
                std::thread::spawn(move || {
                    Self::receive_loop_sync(stream_clone, state_clone, running_clone);
                });

                // 启动心跳线程
                let stream_clone = Arc::clone(&self.stream);
                let running_clone = Arc::clone(&self.running);
                std::thread::spawn(move || {
                    Self::heartbeat_loop_sync(stream_clone, running_clone);
                });

                Ok(())
            }
            Err(e) => Err(NetworkError::ConnectionError(format!(
                "Connection failed: {}",
                e
            ))),
        }
    }

    /// 同步版本的断开连接方法（向后兼容）
    pub fn disconnect_sync(&mut self) -> Result<(), NetworkError> {
        *crate::error::safe_lock(&self.running, "client_running").unwrap() = false;

        if let Some(stream) = crate::error::safe_lock(&self.stream, "client_stream").unwrap().take() {
            let disconnect_msg = NetworkMessage::Disconnect {
                client_id: crate::error::safe_lock(&self.state, "client_state").unwrap().client_id.unwrap_or(0),
            };
            let _ = Self::send_to_stream_sync(&stream, &disconnect_msg);
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }

        let mut state_guard = crate::error::safe_lock(&self.state, "client_state").unwrap();
        state_guard.connection_state = ConnectionState::Disconnected;

        Ok(())
    }

    /// 同步版本的发送消息方法（向后兼容）
    pub fn send_message_sync(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        if let Some(ref stream) = *crate::error::safe_lock(&self.stream, "client_stream").unwrap() {
            Self::send_to_stream_sync(stream, message)
        } else {
            Err(NetworkError::SendError("Not connected".to_string()))
        }
    }

    /// 同步发送消息到流（向后兼容）
    fn send_to_stream_sync(
        stream: &StdTcpStream,
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let data = bincode::serialize(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // 发送消息
        stream.write_all(&data)
            .map_err(|e| NetworkError::SendError(e.to_string()))
    }

    /// 异步接收循环
    async fn receive_loop_async(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        state: Arc<Mutex<NetworkState>>,
        running: Arc<Mutex<bool>>,
    ) {
        let mut buffer = vec![0u8; 4096];

        while *running.lock().await {
            if let Some(ref mut stream) = *stream.lock().await {
                match stream.read(&mut buffer).await {
                    Ok(0) => {
                        // 连接关闭
                        break;
                    }
                    Ok(n) => {
                        // 处理接收到的数据
                        let data = &buffer[..n];
                        if let Ok(message) = bincode::deserialize::<NetworkMessage>(data) {
                            Self::process_message_async(&message, &state).await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        // 更新连接状态
        let mut state_guard = state.lock().await;
        state_guard.connection_state = ConnectionState::Disconnected;
        drop(state_guard);
        
        // 启动重连机制
    }

    /// 同步版本的接收循环（向后兼容）
    fn receive_loop_sync(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        state: Arc<Mutex<NetworkState>>,
        running: Arc<Mutex<bool>>,
    ) {
        let mut buffer = vec![0u8; 4096];

        while *crate::error::safe_lock(running, "client_running").unwrap() {
            if let Some(ref stream) = *crate::error::safe_lock(stream, "client_stream").unwrap() {
                match stream.read(&mut buffer) {
                    Ok(0) => {
                        // 连接关闭
                        break;
                    }
                    Ok(n) => {
                        // 处理接收到的数据
                        let data = &buffer[..n];
                        if let Ok(message) = bincode::deserialize::<NetworkMessage>(data) {
                            Self::process_message_sync(&message, &state);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            } else {
                std::thread::sleep(Duration::from_millis(100));
            }
        }

        // 更新连接状态
        let mut state_guard = crate::error::safe_lock(state, "client_state").unwrap();
        state_guard.connection_state = ConnectionState::Disconnected;
        drop(state_guard);
        
        // 启动重连机制
    }

    /// 异步处理接收到的消息
    async fn process_message_async(message: &NetworkMessage, state: &Arc<Mutex<NetworkState>>) {
        match message {
            NetworkMessage::TimeSyncResponse { sync } => {
                // 处理时间同步响应
                if let Some(ref compensation) = state.lock().await.delay_compensation {
                    if let Ok(mut comp_guard) = compensation.lock() {
                        let mut sync_clone = sync.clone();
                        comp_guard.process_time_sync(&mut sync_clone);
                    }
                }
            }
            NetworkMessage::StateSync { tick, data: _ } => {
                // 处理状态同步
                let mut state_guard = state.lock().await;
                state_guard.current_tick = *tick;
            }
            _ => {
                // 其他消息类型的处理
            }
        }
    }

    /// 同步版本的处理接收到的消息（向后兼容）
    fn process_message_sync(message: &NetworkMessage, state: &Arc<Mutex<NetworkState>>) {
        match message {
            NetworkMessage::TimeSyncResponse { sync } => {
                // 处理时间同步响应
                if let Ok(state_guard) = state.lock() {
                    if let Some(ref compensation) = state_guard.delay_compensation {
                        if let Ok(mut comp_guard) = compensation.lock() {
                            let mut sync_clone = sync.clone();
                            comp_guard.process_time_sync(&mut sync_clone);
                        }
                    }
                }
            }
            NetworkMessage::StateSync { tick, data: _ } => {
                // 处理状态同步
                if let Ok(mut state_guard) = state.lock() {
                    state_guard.current_tick = *tick;
                }
            }
            _ => {
                // 其他消息类型的处理
            }
        }
    }

    /// 异步心跳循环
    async fn heartbeat_loop_async(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        running: Arc<Mutex<bool>>,
    ) {
        while *running.lock().await {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let heartbeat_msg = NetworkMessage::Heartbeat {
                timestamp: current_timestamp_ms(),
            };

            if let Some(ref mut stream) = *stream.lock().await {
                let _ = Self::send_to_stream_async(stream, &heartbeat_msg).await;
            }
        }
    }

    /// 同步版本的心跳循环（向后兼容）
    fn heartbeat_loop_sync(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        running: Arc<Mutex<bool>>,
    ) {
        while *crate::error::safe_lock(running, "client_running").unwrap() {
            std::thread::sleep(Duration::from_secs(1));

            let heartbeat_msg = NetworkMessage::Heartbeat {
                timestamp: current_timestamp_ms(),
            };

            if let Some(ref stream) = *crate::error::safe_lock(&stream, "client_stream").unwrap() {
                let _ = Self::send_to_stream_sync(stream, &heartbeat_msg);
            }
        }
    }

    /// 异步获取网络状态引用
    pub async fn state(&self) -> tokio::sync::MutexGuard<'_, NetworkState> {
        self.state.lock().await
    }

    /// 异步获取连接状态
    pub async fn connection_state(&self) -> ConnectionState {
        self.state.lock().await.connection_state
    }

    /// 异步获取客户端ID
    pub async fn client_id(&self) -> Option<u64> {
        self.state.lock().await.client_id
    }

    /// 异步检查是否已连接
    pub async fn is_connected(&self) -> bool {
        self.state.lock().await.connection_state == ConnectionState::Connected
    }

    /// 同步版本的获取网络状态引用（向后兼容）
    pub fn state_sync(&self) -> std::sync::MutexGuard<'_, NetworkState> {
        crate::error::safe_lock(&self.state, "client_state").unwrap()
    }

    /// 同步版本的获取连接状态（向后兼容）
    pub fn connection_state_sync(&self) -> ConnectionState {
        crate::error::safe_lock(&self.state, "client_state").unwrap().connection_state
    }

    /// 同步版本的获取客户端ID（向后兼容）
    pub fn client_id_sync(&self) -> Option<u64> {
        crate::error::safe_lock(&self.state, "client_state").unwrap().client_id
    }

    /// 同步版本的检查是否已连接（向后兼容）
    pub fn is_connected_sync(&self) -> bool {
        crate::error::safe_lock(&self.state, "client_state").unwrap().connection_state == ConnectionState::Connected
    }

    /// 异步重连循环
    async fn start_reconnect_loop_async(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        state: Arc<Mutex<NetworkState>>,
        running: Arc<Mutex<bool>>,
        config: ClientConfig,
    ) {
        // 检查是否需要重连
        if !*running.lock().await {
            return;
        }
        
        // 启动重连尝试计数
        let reconnect_attempts = Arc::new(Mutex::new(0));
        
        // 创建重连任务
        task::spawn(async move {
            while *running.lock().await {
                let mut attempts_guard = reconnect_attempts.lock().await;
                
                // 检查是否达到最大重连次数
                if *attempts_guard >= config.max_reconnect_attempts {
                    // 达到最大重连次数，停止尝试
                    let mut state_guard = state.lock().await;
                    state_guard.connection_state = ConnectionState::Disconnected;
                    break;
                }
                
                // 更新连接状态为正在重连
                let mut state_guard = state.lock().await;
                state_guard.connection_state = ConnectionState::Reconnecting;
                drop(state_guard);
                
                // 增加重连尝试次数
                *attempts_guard += 1;
                
                // 尝试重连
                match TokioTcpStream::connect(format!("{}:{}", config.server_address, config.server_port)).await {
                    Ok(new_stream) => {
                        // 重连成功，更新流和状态
                        let mut stream_guard = stream.lock().await;
                        *stream_guard = Some(new_stream);
                        
                        let mut state_guard = state.lock().await;
                        state_guard.connection_state = ConnectionState::Connected;
                        state_guard.client_id = Some(rand::random());
                        
                        // 发送连接请求
                        let client_id = state_guard.client_id.unwrap();
                        let connect_msg = NetworkMessage::Connect {
                            client_id,
                            name: config.client_name.clone(),
                        };
                        
                        // 启动接收任务和心跳任务
                        let stream_clone = Arc::clone(&stream);
                        let state_clone = Arc::clone(&state);
                        let running_clone = Arc::clone(&running);
                        
                        task::spawn(async move {
                            Self::receive_loop_async(stream_clone, state_clone, running_clone).await;
                        });
                        
                        let stream_clone = Arc::clone(&stream);
                        let running_clone = Arc::clone(&running);
                        
                        task::spawn(async move {
                            Self::heartbeat_loop_async(stream_clone, running_clone).await;
                        });
                        
                        // 重置重连尝试次数
                        *attempts_guard = 0;
                        
                        // 重连成功，退出循环
                        break;
                    },
                    Err(e) => {
                        // 重连失败，等待一段时间后重试
                        eprintln!("Reconnect failed: {}, attempt {}/{}",
                            e, *attempts_guard, config.max_reconnect_attempts);
                        
                        tokio::time::sleep(Duration::from_millis(config.reconnect_interval_ms)).await;
                    }
                }
            }
        });
    }

    /// 同步版本的重连循环（向后兼容）
    fn start_reconnect_loop_sync(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        state: Arc<Mutex<NetworkState>>,
        running: Arc<Mutex<bool>>,
        config: ClientConfig,
    ) {
        // 检查是否需要重连
        if !*crate::error::safe_lock(&running, "client_running").unwrap() {
            return;
        }
        
        // 启动重连尝试计数
        let reconnect_attempts = Arc::new(Mutex::new(0));
        
        // 创建重连线程
        std::thread::spawn(move || {
            while *crate::error::safe_lock(&running, "client_running").unwrap() {
                let mut attempts_guard = crate::error::safe_lock(&reconnect_attempts, "reconnect_attempts").unwrap();
                
                // 检查是否达到最大重连次数
                if *attempts_guard >= config.max_reconnect_attempts {
                    // 达到最大重连次数，停止尝试
                    let mut state_guard = crate::error::safe_lock(&state, "client_state").unwrap();
                    state_guard.connection_state = ConnectionState::Disconnected;
                    break;
                }
                
                // 更新连接状态为正在重连
                let mut state_guard = crate::error::safe_lock(&state, "client_state").unwrap();
                state_guard.connection_state = ConnectionState::Reconnecting;
                drop(state_guard);
                
                // 增加重连尝试次数
                *attempts_guard += 1;
                
                // 尝试重连
                match StdTcpStream::connect(format!("{}:{}", config.server_address, config.server_port)) {
                    Ok(new_stream) => {
                        // 重连成功，更新流和状态
                        let mut stream_guard = crate::error::safe_lock(&stream, "client_stream").unwrap();
                        *stream_guard = Some(new_stream.into());
                        
                        let mut state_guard = crate::error::safe_lock(&state, "client_state").unwrap();
                        state_guard.connection_state = ConnectionState::Connected;
                        state_guard.client_id = Some(rand::random());
                        
                        // 发送连接请求
                        let client_id = state_guard.client_id.unwrap();
                        let connect_msg = NetworkMessage::Connect {
                            client_id,
                            name: config.client_name.clone(),
                        };
                        
                        // 启动接收线程和心跳线程
                        let stream_clone = Arc::clone(&stream);
                        let state_clone = Arc::clone(&state);
                        let running_clone = Arc::clone(&running);
                        
                        std::thread::spawn(move || {
                            Self::receive_loop_sync(stream_clone, state_clone, running_clone);
                        });
                        
                        let stream_clone = Arc::clone(&stream);
                        let running_clone = Arc::clone(&running);
                        
                        std::thread::spawn(move || {
                            Self::heartbeat_loop_sync(stream_clone, running_clone);
                        });
                        
                        // 重置重连尝试次数
                        *attempts_guard = 0;
                        
                        // 重连成功，退出循环
                        break;
                    },
                    Err(e) => {
                        // 重连失败，等待一段时间后重试
                        eprintln!("Reconnect failed: {}, attempt {}/{}",
                            e, *attempts_guard, config.max_reconnect_attempts);
                        
                        std::thread::sleep(Duration::from_millis(config.reconnect_interval_ms));
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = ClientConfig::default();
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.max_reconnect_attempts, 5);
    }

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::default();
        let client = GameClient::new(config);
        assert!(!client.is_connected());
    }
}
