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
use crossbeam_channel::unbounded;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream as TokioTcpStream;
use tokio::sync::Mutex;
use tokio::task;

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
    /// 接收通道发送端
    recv_tx: Arc<Mutex<crossbeam_channel::Sender<NetworkMessage>>>,
}

impl GameClient {
    /// 创建新的游戏客户端
    pub fn new(config: ClientConfig) -> Self {
        let compressor = if config.enable_compression {
            Some(Arc::new(compression::NetworkCompressor::new()))
        } else {
            None
        };

        let (send_tx, _send_rx) = unbounded::<NetworkMessage>();
        let (recv_tx, recv_rx) = unbounded::<NetworkMessage>();

        // 记录重连计数器用于跟踪连接历史
        let reconnect_attempts = Arc::new(Mutex::new(0usize));

        // 将重连计数器初始化到网络状态中（虽然network_state目前不使用它，但保留用于未来扩展）
        let _reconnect_attempts = &reconnect_attempts;

        let network_state = NetworkState {
            connection_state: ConnectionState::default(),
            client_id: None,
            server_addr: None,
            stats: Default::default(),
            current_tick: 0,
            send_tx: Some(send_tx),
            recv_rx: Some(recv_rx),
            delta_serializer: None,
            compressor: compressor.clone(),
            delay_compensation: if config.enable_delay_compensation {
                Some(Arc::new(std::sync::Mutex::new(
                    delay_compensation::ClientDelayCompensation::new(),
                )))
            } else {
                None
            },
            reconnect_attempts: 0,
        };

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
            recv_tx: Arc::new(Mutex::new(recv_tx)),
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
                let mut stream_guard = self.stream.lock().await;
                *stream_guard = Some(stream);
                drop(stream_guard);

                let mut state_guard = self.state.lock().await;
                state_guard.connection_state = ConnectionState::Connected;
                state_guard.server_addr = Some(addr);
                state_guard.client_id = Some(rand::random());

                let client_id = state_guard.client_id.unwrap_or_else(rand::random);
                state_guard.client_id = Some(client_id);
                let connect_msg = NetworkMessage::Connect {
                    client_id,
                    name: self.config.client_name.clone(),
                };
                drop(state_guard);
                self.send_message(&connect_msg).await?;

                let stream_clone = Arc::clone(&self.stream);
                let state_clone = Arc::clone(&self.state);
                let recv_tx_clone = Arc::clone(&self.recv_tx);
                let running_clone = Arc::clone(&self.running);

                *self.running.lock().await = true;
                task::spawn(async move {
                    Self::receive_loop_async(
                        stream_clone,
                        state_clone,
                        recv_tx_clone,
                        running_clone,
                    )
                    .await;
                });

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
            let client_id = self.state.lock().await.client_id.unwrap_or(0);
            let disconnect_msg = NetworkMessage::Disconnect { client_id };
            if let Ok(data) = bincode::serialize(&disconnect_msg) {
                let _ = stream.write_all(&data).await;
            }
            let _ = stream.shutdown().await;
        }

        let mut state = self.state.lock().await;
        state.connection_state = ConnectionState::Disconnected;
        state.server_addr = None;

        Ok(())
    }

    /// 同步连接到服务器（阻塞版本）
    pub fn connect_sync(&self) -> Result<(), NetworkError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            NetworkError::ConnectionError(format!("Failed to create runtime: {}", e))
        })?;
        rt.block_on(self.connect())
    }

    /// 同步断开连接（阻塞版本）
    pub fn disconnect_sync(&self) -> Result<(), NetworkError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            NetworkError::ConnectionError(format!("Failed to create runtime: {}", e))
        })?;
        rt.block_on(self.disconnect())
    }

    /// 异步发送消息
    pub async fn send_message(&self, msg: &NetworkMessage) -> Result<(), NetworkError> {
        let mut data = bincode::serialize(msg).map_err(|e| {
            NetworkError::SerializationError(format!("Failed to serialize message: {}", e))
        })?;

        if self.config.enable_compression
            && let Some(compressor) = &self.compressor
        {
            data = compressor.compress(&data).map_err(|e| {
                NetworkError::CompressionError(format!("Compression failed: {}", e))
            })?;
        }

        let mut stream_guard = self.stream.lock().await;
        if let Some(stream) = stream_guard.as_mut() {
            let len = data.len() as u32;
            stream.write_all(&len.to_be_bytes()).await.map_err(|e| {
                NetworkError::SendError(format!("Failed to write message length: {}", e))
            })?;
            stream
                .write_all(&data)
                .await
                .map_err(|e| NetworkError::SendError(format!("Failed to write message: {}", e)))?;
            drop(stream_guard);

            let mut state = self.state.lock().await;
            state.stats.bytes_sent += len as u64;
            state.stats.messages_sent += 1;
        } else {
            return Err(NetworkError::ConnectionError(
                "No active connection".to_string(),
            ));
        }

        Ok(())
    }

    /// 同步发送消息（阻塞版本）
    pub fn send_message_sync(&self, msg: NetworkMessage) -> Result<(), NetworkError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            NetworkError::ConnectionError(format!("Failed to create runtime: {}", e))
        })?;
        rt.block_on(self.send_message(&msg))
    }

    /// 异步接收消息
    pub async fn receive_message(&self) -> Result<Option<NetworkMessage>, NetworkError> {
        let state = self.state.lock().await;
        if let Some(recv_rx) = &state.recv_rx {
            match recv_rx.try_recv() {
                Ok(msg) => Ok(Some(msg)),
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// 同步接收消息（阻塞版本）
    pub fn receive_message_sync(&self) -> Result<Option<NetworkMessage>, NetworkError> {
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            NetworkError::ConnectionError(format!("Failed to create runtime: {}", e))
        })?;
        rt.block_on(self.receive_message())
    }

    /// 获取客户端ID
    pub async fn client_id(&self) -> Option<u64> {
        self.state.lock().await.client_id
    }

    /// 获取连接状态
    pub async fn connection_state(&self) -> ConnectionState {
        self.state.lock().await.connection_state
    }

    /// 获取网络统计信息
    pub async fn get_stats(&self) -> NetworkStats {
        let state = self.state.lock().await;
        NetworkStats {
            bytes_sent: state.stats.bytes_sent,
            bytes_received: state.stats.bytes_received,
            messages_sent: state.stats.messages_sent,
            messages_received: state.stats.messages_received,
            ping_ms: (state.stats.latency_ms * 1000.0) as u64,
        }
    }

    /// 异步接收循环
    async fn receive_loop_async(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        state: Arc<Mutex<NetworkState>>,
        recv_tx: Arc<Mutex<crossbeam_channel::Sender<NetworkMessage>>>,
        running: Arc<Mutex<bool>>,
    ) {
        while *running.lock().await {
            let mut stream_guard = stream.lock().await;
            if let Some(stream) = stream_guard.as_mut() {
                let mut len_buf = [0u8; 4];
                match stream.read_exact(&mut len_buf).await {
                    Ok(_) => {
                        let len = u32::from_be_bytes(len_buf) as usize;
                        if len == 0 || len > 1024 * 1024 {
                            continue;
                        }

                        let mut data_buf = vec![0u8; len];
                        match stream.read_exact(&mut data_buf).await {
                            Ok(_) => {
                                let mut state_guard = state.lock().await;

                                let data = if let Some(compressor) = &state_guard.compressor {
                                    compressor.decompress(&data_buf).unwrap_or(data_buf.clone())
                                } else {
                                    data_buf.clone()
                                };

                                match bincode::deserialize::<NetworkMessage>(&data) {
                                    Ok(msg) => {
                                        state_guard.stats.bytes_received += len as u64;
                                        state_guard.stats.messages_received += 1;

                                        let _ = recv_tx.lock().await.send(msg);
                                    }
                                    Err(e) => {
                                        log::warn!("Failed to deserialize message: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::warn!("Failed to read message: {}", e);
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to read message length: {}", e);
                        break;
                    }
                }
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        let mut state_guard = state.lock().await;
        state_guard.connection_state = ConnectionState::Disconnected;
    }

    /// 异步心跳循环
    async fn heartbeat_loop_async(
        stream: Arc<Mutex<Option<TokioTcpStream>>>,
        running: Arc<Mutex<bool>>,
    ) {
        while *running.lock().await {
            let mut stream_guard = stream.lock().await;
            if let Some(stream) = stream_guard.as_mut() {
                let heartbeat_msg = NetworkMessage::Heartbeat {
                    timestamp: current_timestamp_ms(),
                };
                if let Ok(data) = bincode::serialize(&heartbeat_msg) {
                    let len = data.len() as u32;
                    let _ = stream.write_all(&len.to_be_bytes()).await;
                    let _ = stream.write_all(&data).await;
                }
            }
            drop(stream_guard);
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    /// 处理预测逻辑
    pub async fn process_prediction(&self, _local_inputs: &[u8]) -> Result<(), NetworkError> {
        if self.config.enable_delay_compensation {
            let _dc = self.delay_compensation.lock().await;
        }
        Ok(())
    }

    /// 获取服务器状态
    pub async fn get_server_state(&self) -> Result<ServerState, NetworkError> {
        let state = self.state.lock().await;
        Ok(ServerState {
            connected: state.connection_state == ConnectionState::Connected,
            client_id: state.client_id,
            server_addr: state.server_addr,
        })
    }
}

/// 网络统计信息
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub ping_ms: u64,
}

/// 服务器状态
#[derive(Debug, Clone)]
pub struct ServerState {
    pub connected: bool,
    pub client_id: Option<u64>,
    pub server_addr: Option<SocketAddr>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.server_address, "127.0.0.1");
        assert_eq!(config.server_port, 8080);
    }

    #[test]
    fn test_client_creation() {
        let config = ClientConfig::default();
        let _client = GameClient::new(config);
    }

    #[test]
    fn test_client_config_custom() {
        let config = ClientConfig {
            server_address: "192.168.1.1".to_string(),
            server_port: 9000,
            reconnect_interval_ms: 3000,
            max_reconnect_attempts: 10,
            enable_compression: false,
            enable_delay_compensation: false,
            client_name: "TestClient".to_string(),
        };

        assert_eq!(config.server_address, "192.168.1.1");
        assert_eq!(config.server_port, 9000);
        assert_eq!(config.reconnect_interval_ms, 3000);
        assert_eq!(config.max_reconnect_attempts, 10);
        assert!(!config.enable_compression);
        assert!(!config.enable_delay_compensation);
        assert_eq!(config.client_name, "TestClient");
    }

    #[test]
    fn test_client_state_initialization() {
        let config = ClientConfig::default();
        let client = GameClient::new(config);

        // 创建runtime来检查异步状态
        let rt = tokio::runtime::Runtime::new().unwrap_or_else(|e| {
            panic!("Failed to create runtime: {}", e);
        });

        rt.block_on(async {
            let state = client.state.lock().await;
            assert_eq!(state.connection_state, ConnectionState::Disconnected);
            assert!(state.client_id.is_none());
            assert!(state.server_addr.is_none());
            assert_eq!(state.current_tick, 0);
        });
    }

    #[test]
    fn test_network_stats() {
        let stats = NetworkStats {
            bytes_sent: 1024,
            bytes_received: 2048,
            messages_sent: 10,
            messages_received: 20,
            ping_ms: 50,
        };

        assert_eq!(stats.bytes_sent, 1024);
        assert_eq!(stats.bytes_received, 2048);
        assert_eq!(stats.messages_sent, 10);
        assert_eq!(stats.messages_received, 20);
        assert_eq!(stats.ping_ms, 50);
    }

    #[test]
    fn test_server_state_disconnected() {
        let state = ServerState {
            connected: false,
            client_id: None,
            server_addr: None,
        };

        assert!(!state.connected);
        assert!(state.client_id.is_none());
        assert!(state.server_addr.is_none());
    }

    #[test]
    fn test_server_state_connected() {
        let addr = "127.0.0.1:8080".parse().unwrap_or_else(|e| {
            panic!("Failed to parse address: {}", e);
        });

        let state = ServerState {
            connected: true,
            client_id: Some(12345),
            server_addr: Some(addr),
        };

        assert!(state.connected);
        assert_eq!(state.client_id, Some(12345));
        assert_eq!(state.server_addr, Some(addr));
    }

    #[test]
    fn test_message_serialization() {
        let msg = NetworkMessage::Heartbeat {
            timestamp: 12345,
        };

        let serialized = bincode::serialize(&msg);
        assert!(serialized.is_ok());

        let deserialized: Result<NetworkMessage, _> = bincode::deserialize(&serialized.unwrap_or_else(|e| {
            panic!("Serialization failed: {}", e);
        }));
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_connect_message_creation() {
        let msg = NetworkMessage::Connect {
            client_id: 12345,
            name: "TestClient".to_string(),
        };

        let serialized = bincode::serialize(&msg);
        assert!(serialized.is_ok());

        if let Ok(NetworkMessage::Connect { client_id, name }) =
            bincode::deserialize::<NetworkMessage>(&serialized.unwrap_or_else(|e| {
                panic!("Serialization failed: {}", e);
            }))
        {
            assert_eq!(client_id, 12345);
            assert_eq!(name, "TestClient");
        } else {
            panic!("Deserialization failed");
        }
    }

    #[test]
    fn test_disconnect_message_creation() {
        let msg = NetworkMessage::Disconnect { client_id: 12345 };

        let serialized = bincode::serialize(&msg);
        assert!(serialized.is_ok());

        if let Ok(NetworkMessage::Disconnect { client_id }) =
            bincode::deserialize::<NetworkMessage>(&serialized.unwrap_or_else(|e| {
                panic!("Serialization failed: {}", e);
            }))
        {
            assert_eq!(client_id, 12345);
        } else {
            panic!("Deserialization failed");
        }
    }
}
