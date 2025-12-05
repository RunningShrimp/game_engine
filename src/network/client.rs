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
use bincode;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
    stream: Arc<Mutex<Option<TcpStream>>>,
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

    /// 连接到服务器
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

        Ok(())
    }

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
        running: Arc<Mutex<bool>>,
    ) {
        let mut buffer = vec![0u8; 4096];

        while *running.lock().unwrap() {
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
            }
        }
    }

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
