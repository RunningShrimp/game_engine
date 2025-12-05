//! 网络客户端模块
//!
<<<<<<< HEAD
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
=======
//! 实现游戏客户端的网络功能，包括：
//! - 服务器连接管理
//! - 消息发送和接收
//! - 客户端状态管理
//! - 时间同步
//! - 客户端预测支持
//!
//! 基于 Tokio 的异步 I/O 实现，支持高并发网络操作。
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

use crate::core::utils::current_timestamp_ms;
use crate::impl_default;
use crate::network::compression;
use crate::network::delay_compensation;
<<<<<<< HEAD
use crate::network::{ConnectionState, NetworkError, NetworkMessage, NetworkState};
use bincode;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
=======
use crate::network::delta_serialization;
use crate::network::{ConnectionState, NetworkError, NetworkMessage};
use bincode;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::net::{TcpStream as TokioTcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{Duration, interval, timeout};
use tokio::sync::Notify;
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

/// 客户端配置
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// 服务器地址
    pub server_address: String,
    /// 服务器端口
    pub server_port: u16,
<<<<<<< HEAD
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
=======
    /// 重试次数
    pub max_retries: u32,
    /// 重试间隔（毫秒）
    pub retry_interval_ms: u64,
    /// 心跳间隔（毫秒）
    pub heartbeat_interval_ms: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 消息超时时间（毫秒）
    pub message_timeout_ms: u64,
}

impl_default!(ClientConfig {
    server_address: "localhost".to_string(),
    server_port: 8080,
    max_retries: 3,
    retry_interval_ms: 1000,
    heartbeat_interval_ms: 5000,
    enable_compression: true,
    message_timeout_ms: 3000,
});

/// 客户端状态
pub struct ClientState {
    /// 连接状态
    pub connection_state: ConnectionState,
    /// 客户端ID
    pub client_id: Option<u64>,
    /// 客户端名称
    pub name: String,
    /// 延迟补偿器
    pub delay_compensation: delay_compensation::ClientDelayCompensation,
    /// 当前客户端tick
    pub current_tick: u64,
    /// 本地延迟估计值（毫秒）
    pub round_trip_time: u64,
    /// 本地延迟抖动（毫秒）
    pub jitter: u64,
}

impl_default!(ClientState {
    connection_state: ConnectionState::Disconnected,
    client_id: None,
    name: "Player".to_string(),
    delay_compensation: delay_compensation::ClientDelayCompensation::new(),
    current_tick: 0,
    round_trip_time: 0,
    jitter: 0,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
});

/// 游戏客户端
pub struct GameClient {
    /// 配置
    config: ClientConfig,
<<<<<<< HEAD
    /// 网络状态
    state: Arc<Mutex<NetworkState>>,
    /// TCP流
    stream: Arc<Mutex<Option<TcpStream>>>,
    /// 延迟补偿管理器
    delay_compensation: Arc<Mutex<delay_compensation::ClientDelayCompensation>>,
    /// 压缩器
    compressor: Option<Arc<compression::NetworkCompressor>>,
    /// 是否运行中
    running: Arc<Mutex<bool>>,
    /// 重连尝试次数
    reconnect_attempts: Arc<Mutex<usize>>,
=======
    /// 客户端状态
    state: Arc<Mutex<ClientState>>,
    /// TCP流
    stream: Arc<Mutex<Option<TokioTcpStream>>>,
    /// 压缩器
    compressor: Option<Arc<compression::NetworkCompressor>>,
    /// 增量序列化器
    delta_serializer: Arc<Mutex<delta_serialization::DeltaSerializer>>,
    /// 是否运行中
    running: Arc<Mutex<bool>>,
    /// 通知信号，用于唤醒事件循环
    notify: Arc<Notify>,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
}

impl GameClient {
    /// 创建新的游戏客户端
<<<<<<< HEAD
    pub fn new(config: ClientConfig) -> Self {
=======
    pub fn new(name: &str, config: ClientConfig) -> Self {
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        let compressor = if config.enable_compression {
            Some(Arc::new(compression::NetworkCompressor::new()))
        } else {
            None
        };

<<<<<<< HEAD
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
=======
        Self {
            config,
            state: Arc::new(Mutex::new(ClientState {
                name: name.to_string(),
                ..Default::default()
            })),
            stream: Arc::new(Mutex::new(None)),
            compressor,
            delta_serializer: Arc::new(Mutex::new(delta_serialization::DeltaSerializer::new())),
            running: Arc::new(Mutex::new(false)),
            notify: Arc::new(Notify::new()),
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        }
    }

    /// 连接到服务器
<<<<<<< HEAD
    pub fn connect(&mut self) -> Result<(), NetworkError> {
        let address = format!("{}:{}", self.config.server_address, self.config.server_port);
        let addr: SocketAddr = address
            .parse()
            .map_err(|e| NetworkError::ConnectionError(format!("Invalid address: {}", e)))?;

        match TcpStream::connect(&addr) {
            Ok(stream) => {
                stream.set_nonblocking(true).map_err(|e| {
                    NetworkError::ConnectionError(format!("Failed to set nonblocking: {}", e))
                })?;

                *self.stream.lock().unwrap() = Some(stream);

                let mut state_guard = self.state.lock().unwrap();
                state_guard.connection_state = ConnectionState::Connected;
                state_guard.server_addr = Some(addr);
                state_guard.client_id = Some(rand::random());

                // 发送连接请求
                let connect_msg = NetworkMessage::Connect {
                    client_id: state_guard.client_id.unwrap(),
                    name: self.config.client_name.clone(),
                };
                self.send_message(&connect_msg)?;

                // 启动接收线程
                let stream_clone = Arc::clone(&self.stream);
                let state_clone = Arc::clone(&self.state);
                let running_clone = Arc::clone(&self.running);

                *self.running.lock().unwrap() = true;
                thread::spawn(move || {
                    Self::receive_loop(stream_clone, state_clone, running_clone);
                });

                // 启动心跳线程
                let stream_clone = Arc::clone(&self.stream);
                let running_clone = Arc::clone(&self.running);
                thread::spawn(move || {
                    Self::heartbeat_loop(stream_clone, running_clone);
                });

                Ok(())
            }
            Err(e) => Err(NetworkError::ConnectionError(format!(
                "Connection failed: {}",
                e
            ))),
        }
    }

    /// 断开连接
    pub fn disconnect(&mut self) -> Result<(), NetworkError> {
        *self.running.lock().unwrap() = false;

        if let Some(stream) = self.stream.lock().unwrap().take() {
            let disconnect_msg = NetworkMessage::Disconnect {
                client_id: self.state.lock().unwrap().client_id.unwrap_or(0),
            };
            let _ = Self::send_to_stream(&stream, &disconnect_msg);
            let _ = stream.shutdown(std::net::Shutdown::Both);
        }

        let mut state_guard = self.state.lock().unwrap();
        state_guard.connection_state = ConnectionState::Disconnected;
=======
    pub async fn connect(&self) -> Result<(), NetworkError> {
        let mut state = self.state.lock().unwrap();
        state.connection_state = ConnectionState::Connecting;

        // 尝试连接服务器
        let address = format!("{}:{}", self.config.server_address, self.config.server_port);
        let stream = TokioTcpStream::connect(&address)
            .await
            .map_err(|e| NetworkError::ConnectionError(format!("Connection failed: {}", e)))?;

        // 更新状态
        state.connection_state = ConnectionState::Connected;
        drop(state);

        // 保存流
        let mut stream_guard = self.stream.lock().unwrap();
        *stream_guard = Some(stream);
        drop(stream_guard);

        // 启动接收循环和心跳循环
        let stream_clone = Arc::clone(&self.stream);
        let state_clone = Arc::clone(&self.state);
        let running_clone = Arc::clone(&self.running);
        
        // 启动接收循环
        tokio::spawn(async move {
            Self::receive_loop(stream_clone, state_clone, running_clone).await;
        });

        // 启动心跳循环
        let stream_clone = Arc::clone(&self.stream);
        let running_clone = Arc::clone(&self.running);
        
        tokio::spawn(async move {
            Self::heartbeat_loop(stream_clone, running_clone).await;
        });

        Ok(())
    }

    /// 断开连接
    pub async fn disconnect(&self) -> Result<(), NetworkError> {
        *self.running.lock().unwrap() = false;

        // 更新状态
        let mut state = self.state.lock().unwrap();
        state.connection_state = ConnectionState::Disconnected;
        drop(state);

        // 关闭连接
        let mut stream = self.stream.lock().unwrap();
        if let Some(ref mut s) = *stream {
            s.shutdown().await
                .map_err(|e| NetworkError::ConnectionError(format!("Failed to shutdown: {}", e)))?;
            *stream = None;
        }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        Ok(())
    }

<<<<<<< HEAD
    /// 发送消息
    pub fn send_message(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        if let Some(ref stream) = *self.stream.lock().unwrap() {
            Self::send_to_stream(stream, message)
        } else {
            Err(NetworkError::SendError("Not connected".to_string()))
        }
    }

    /// 发送消息到流
    fn send_to_stream(_stream: &TcpStream, message: &NetworkMessage) -> Result<(), NetworkError> {
        let _data = bincode::serialize(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // NOTE: 实际实现中需要处理流写入
        // 这里简化处理

        Ok(())
    }

    /// 接收循环（在独立线程中运行）
    fn receive_loop(
        stream: Arc<Mutex<Option<TcpStream>>>,
        state: Arc<Mutex<NetworkState>>,
=======
    /// 发送消息到服务器
    pub async fn send_message(&self, message: NetworkMessage) -> Result<(), NetworkError> {
        // 检查连接状态
        let state = self.state.lock().unwrap();
        if state.connection_state != ConnectionState::Connected {
            return Err(NetworkError::ConnectionError("Not connected".to_string()));
        }

        // 序列化消息
        let data = Self::serialize_message(&message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // 获取流并发送
        let mut stream_guard = self.stream.lock().unwrap();
        if let Some(ref mut stream) = *stream_guard {
            let send_result = stream.write_all(&data).await;
            drop(stream_guard);
            drop(state);
            
            match send_result {
                Ok(_) => Ok(()),
                Err(e) => Err(NetworkError::SendError(format!("Send failed: {}", e))),
            }
        } else {
            drop(stream_guard);
            drop(state);
            Err(NetworkError::ConnectionError("Not connected".to_string()))
        }
    }

    /// 接收循环
    async fn receive_loop(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        state: Arc<Mutex<ClientState>>,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        running: Arc<Mutex<bool>>,
    ) {
        let mut buffer = vec![0u8; 4096];

        while *running.lock().unwrap() {
<<<<<<< HEAD
            if let Some(ref mut stream) = *stream.lock().unwrap() {
                match stream.read(&mut buffer) {
                    Ok(0) => {
                        // 连接关闭
                        break;
                    }
                    Ok(n) => {
                        // 处理接收到的数据
                        let data = &buffer[..n];
                        if let Ok(message) = bincode::deserialize::<NetworkMessage>(data) {
                            Self::process_message(&message, &state);
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("Read error: {}", e);
                        break;
                    }
                }
            } else {
                thread::sleep(Duration::from_millis(100));
            }
        }

        // 更新连接状态
        state.lock().unwrap().connection_state = ConnectionState::Disconnected;
    }

    /// 处理接收到的消息
    fn process_message(message: &NetworkMessage, state: &Arc<Mutex<NetworkState>>) {
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

    /// 心跳循环（在独立线程中运行）
    fn heartbeat_loop(stream: Arc<Mutex<Option<TcpStream>>>, running: Arc<Mutex<bool>>) {
        while *running.lock().unwrap() {
            thread::sleep(Duration::from_secs(1));

            let heartbeat_msg = NetworkMessage::Heartbeat {
                timestamp: current_timestamp_ms(),
            };

            if let Some(ref stream) = *stream.lock().unwrap() {
                let _ = Self::send_to_stream(stream, &heartbeat_msg);
=======
            // 获取流并读取数据
            let result = {
                let mut stream_guard = stream.lock().unwrap();
                if let Some(ref mut s) = *stream_guard {
                    match s.read(&mut buffer).await {
                        Ok(n) => Ok(Some(buffer[..n].to_vec())),
                        Err(e) => Err(e),
                    }
                } else {
                    Ok(None)
                }
            };

            match result {
                Ok(Some(data)) => {
                    // 处理接收到的数据
                    if let Ok(message) = Self::deserialize_message(&data) {
                        Self::process_message(message, &state).await;
                    }
                }
                Ok(None) => {
                    // 连接已关闭
                    break;
                }
                Err(e) => {
                    // 读取错误
                    break;
                }
            }
        }

        // 更新状态为断开连接
        let mut state = state.lock().unwrap();
        state.connection_state = ConnectionState::Disconnected;
    }

    /// 心跳循环
    async fn heartbeat_loop(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        running: Arc<Mutex<bool>>,
    ) {
        let mut interval = interval(Duration::from_millis(5000));

        while *running.lock().unwrap() {
            interval.tick().await;

            // 创建心跳消息
            let heartbeat_msg = NetworkMessage::Heartbeat {
                timestamp: current_timestamp_ms(),
            };
            
            // 序列化并发送消息
            if let Ok(data) = Self::serialize_message(&heartbeat_msg) {
                let mut stream_guard = stream.lock().unwrap();
                if let Some(ref mut s) = *stream_guard {
                    let _ = s.write_all(&data).await;
                }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
            }
        }
    }

<<<<<<< HEAD
    /// 获取网络状态引用（用于读取）
    pub fn state(&self) -> std::sync::MutexGuard<'_, NetworkState> {
        self.state.lock().unwrap()
    }

    /// 获取连接状态
    pub fn connection_state(&self) -> ConnectionState {
        self.state.lock().unwrap().connection_state
    }

    /// 获取客户端ID
    pub fn client_id(&self) -> Option<u64> {
        self.state.lock().unwrap().client_id
    }

    /// 检查是否已连接
    pub fn is_connected(&self) -> bool {
        self.state.lock().unwrap().connection_state == ConnectionState::Connected
=======
    /// 处理接收的消息
    async fn process_message(message: NetworkMessage, state: &Arc<Mutex<ClientState>>) {
        match message {
            NetworkMessage::TimeSyncResponse { sync } => {
                // 更新时间同步信息
                let mut state = state.lock().unwrap();
                state.delay_compensation.process_time_sync(&mut sync.clone());
            }
            _ => {}
        }
    }

    /// 获取客户端状态
    pub fn get_state(&self) -> ClientState {
        let state = self.state.lock().unwrap();
        ClientState {
            connection_state: state.connection_state,
            client_id: state.client_id,
            name: state.name.clone(),
            delay_compensation: state.delay_compensation.clone(), // ClientDelayCompensation已实现Clone
            current_tick: state.current_tick,
            round_trip_time: state.round_trip_time,
            jitter: state.jitter,
        }
    }

    /// 更新客户端tick
    pub fn update_tick(&self) {
        let mut state = self.state.lock().unwrap();
        state.current_tick += 1;
    }

    /// 获取当前tick
    pub fn current_tick(&self) -> u64 {
        self.state.lock().unwrap().current_tick
    }

    /// 序列化消息
    fn serialize_message(message: &NetworkMessage) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(message)
    }

    /// 反序列化消息
    fn deserialize_message(data: &[u8]) -> Result<NetworkMessage, bincode::Error> {
        bincode::deserialize(data)
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config() {
        let config = ClientConfig::default();
        assert_eq!(config.server_port, 8080);
<<<<<<< HEAD
        assert_eq!(config.max_reconnect_attempts, 5);
=======
    }

    #[test]
    fn test_client_state() {
        let state = ClientState::default();
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
        assert_eq!(state.name, "Player");
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::default();
<<<<<<< HEAD
        let client = GameClient::new(config);
        assert!(!client.is_connected());
=======
        let client = GameClient::new("TestPlayer", config);
        let state = client.get_state();
        assert_eq!(state.name, "TestPlayer");
        assert_eq!(state.connection_state, ConnectionState::Disconnected);
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }
}
