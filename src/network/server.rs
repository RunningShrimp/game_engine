//! 网络服务器模块
//!
//! 实现游戏服务器的核心功能，包括：
//! - 客户端连接管理
//! - 消息路由和分发
//! - 服务器端状态管理
//! - 权威状态同步
//!
//! ## 架构设计
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │           Game Server                   │
//! ├─────────────────────────────────────────┤
//! │  ┌──────────┐  ┌──────────┐  ┌─────────┐│
//! │  │ Client 1 │  │ Client 2 │  │Client N││
//! │  └────┬─────┘  └────┬─────┘  └────┬────┘│
//! │       │             │             │     │
//! │       └─────────────┼─────────────┘     │
//! │                     │                   │
//! │              ┌──────▼──────┐            │
//! │              │   Router    │            │
//! │              └──────┬──────┘            │
//! │                     │                   │
//! │       ┌─────────────┼─────────────┐    │
//! │       │             │             │    │
//! │  ┌────▼────┐  ┌─────▼─────┐ ┌────▼───┐│
//! │  │ Game    │  │ Authority  │ │ State  ││
//! │  │ Logic   │  │ Manager    │ │ Sync   ││
//! │  └─────────┘  └────────────┘ └────────┘│
//! └─────────────────────────────────────────┘
//! ```

use crate::core::utils::current_timestamp_ms;
use crate::impl_default;
use crate::network::compression;
use crate::network::delay_compensation;
use crate::network::delta_serialization;
use crate::network::{ConnectionState, NetworkError, NetworkMessage};
use bincode;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 客户端连接信息
#[derive(Debug, Clone)]
pub struct ClientConnection {
    /// 客户端ID
    pub client_id: u64,
    /// 客户端地址
    pub address: SocketAddr,
    /// 连接状态
    pub state: ConnectionState,
    /// 最后心跳时间
    pub last_heartbeat: u64,
    /// 是否已认证
    pub authenticated: bool,
    /// 客户端名称
    pub name: Option<String>,
}

impl ClientConnection {
    /// 创建新的客户端连接
    pub fn new(client_id: u64, address: SocketAddr) -> Self {
        Self {
            client_id,
            address,
            state: ConnectionState::Connecting,
            last_heartbeat: current_timestamp_ms(),
            authenticated: false,
            name: None,
        }
    }

    /// 更新心跳时间
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = current_timestamp_ms();
    }

    /// 检查连接是否超时
    pub fn is_timeout(&self, timeout_ms: u64) -> bool {
        current_timestamp_ms() - self.last_heartbeat > timeout_ms
    }
}

/// 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// 监听地址
    pub bind_address: String,
    /// 监听端口
    pub port: u16,
    /// 最大连接数
    pub max_connections: usize,
    /// 心跳超时时间（毫秒）
    pub heartbeat_timeout_ms: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用延迟补偿
    pub enable_delay_compensation: bool,
}

impl_default!(ServerConfig {
    bind_address: "0.0.0.0".to_string(),
    port: 8080,
    max_connections: 100,
    heartbeat_timeout_ms: 30000,
    enable_compression: true,
    enable_delay_compensation: true,
});

/// 游戏服务器
pub struct GameServer {
    /// 配置
    config: ServerConfig,
    /// 客户端连接映射
    clients: Arc<Mutex<HashMap<u64, ClientConnection>>>,
    /// 延迟补偿管理器
    delay_compensation: Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
    /// 压缩器
    compressor: Option<Arc<compression::NetworkCompressor>>,
    /// 增量序列化器
    delta_serializer: Arc<Mutex<delta_serialization::DeltaSerializer>>,
    /// 当前服务器tick
    current_tick: Arc<Mutex<u64>>,
    /// 是否运行中
    running: Arc<Mutex<bool>>,
}

impl GameServer {
    /// 创建新的游戏服务器
    pub fn new(config: ServerConfig) -> Self {
        let compressor = if config.enable_compression {
            Some(Arc::new(compression::NetworkCompressor::new()))
        } else {
            None
        };

        Self {
            config,
            clients: Arc::new(Mutex::new(HashMap::new())),
            delay_compensation: Arc::new(Mutex::new(
                delay_compensation::ServerDelayCompensation::new(),
            )),
            compressor,
            delta_serializer: Arc::new(Mutex::new(delta_serialization::DeltaSerializer::new())),
            current_tick: Arc::new(Mutex::new(0)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// 启动服务器
    pub fn start(&mut self) -> Result<(), NetworkError> {
        let address = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&address)
            .map_err(|e| NetworkError::ConnectionError(format!("Failed to bind: {}", e)))?;

        listener.set_nonblocking(true).map_err(|e| {
            NetworkError::ConnectionError(format!("Failed to set nonblocking: {}", e))
        })?;

        *self.running.lock().unwrap() = true;

        let clients = Arc::clone(&self.clients);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();

        let delay_compensation = Arc::clone(&self.delay_compensation);

        // 启动监听线程
        thread::spawn(move || {
            Self::accept_connections(listener, clients, running, config, delay_compensation);
        });

        // 启动心跳检查线程
        let clients_clone = Arc::clone(&self.clients);
        let running_clone = Arc::clone(&self.running);
        let timeout = self.config.heartbeat_timeout_ms;

        thread::spawn(move || {
            Self::heartbeat_checker(clients_clone, running_clone, timeout);
        });

        Ok(())
    }

    /// 停止服务器
    pub fn stop(&mut self) {
        *self.running.lock().unwrap() = false;
    }

    /// 接受连接（在独立线程中运行）
    fn accept_connections(
        listener: TcpListener,
        clients: Arc<Mutex<HashMap<u64, ClientConnection>>>,
        running: Arc<Mutex<bool>>,
        config: ServerConfig,
        delay_compensation: Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
    ) {
        while *running.lock().unwrap() {
            match listener.accept() {
                Ok((stream, addr)) => {
                    let client_id = rand::random();
                    let mut clients_guard = clients.lock().unwrap();

                    // 检查连接数限制
                    if clients_guard.len() >= config.max_connections {
                        let _ = stream.shutdown(std::net::Shutdown::Both);
                        continue;
                    }

                    // 创建客户端连接
                    let connection = ClientConnection::new(client_id, addr);
                    clients_guard.insert(client_id, connection);

                    // 启动客户端处理线程
                    let clients_clone = Arc::clone(&clients);
                    let delay_compensation_clone = Arc::clone(&delay_compensation);
                    thread::spawn(move || {
                        Self::handle_client(
                            stream,
                            client_id,
                            clients_clone,
                            delay_compensation_clone,
                        );
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // 非阻塞模式下没有连接，继续等待
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }
    }

    /// 处理客户端连接（在独立线程中运行）
    fn handle_client(
        mut stream: TcpStream,
        client_id: u64,
        clients: Arc<Mutex<HashMap<u64, ClientConnection>>>,
        delay_compensation: Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
    ) {
        let mut buffer = vec![0u8; 4096];

        loop {
            match stream.read(&mut buffer) {
                Ok(0) => {
                    // 连接关闭
                    break;
                }
                Ok(n) => {
                    // 处理接收到的数据
                    let data = &buffer[..n];
                    if let Ok(message) = Self::deserialize_message(data) {
                        Self::process_message(
                            &message,
                            client_id,
                            &clients,
                            &delay_compensation,
                            &mut stream,
                        );
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("Read error for client {}: {}", client_id, e);
                    break;
                }
            }
        }

        // 清理客户端连接
        clients.lock().unwrap().remove(&client_id);
    }

    /// 处理消息
    fn process_message(
        message: &NetworkMessage,
        client_id: u64,
        clients: &Arc<Mutex<HashMap<u64, ClientConnection>>>,
        delay_compensation: &Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        stream: &mut TcpStream,
    ) {
        match message {
            NetworkMessage::Connect { client_id: _, name } => {
                // 处理连接请求
                if let Ok(mut clients_guard) = clients.lock() {
                    if let Some(conn) = clients_guard.get_mut(&client_id) {
                        conn.state = ConnectionState::Connected;
                        conn.authenticated = true;
                        conn.name = Some(name.clone());
                        conn.update_heartbeat();
                    }
                }
            }
            NetworkMessage::Disconnect { client_id: _ } => {
                // 处理断开连接
                clients.lock().unwrap().remove(&client_id);
            }
            NetworkMessage::Heartbeat { timestamp: _ } => {
                // 更新心跳
                if let Ok(mut clients_guard) = clients.lock() {
                    if let Some(conn) = clients_guard.get_mut(&client_id) {
                        conn.update_heartbeat();
                    }
                }
            }
            NetworkMessage::TimeSyncRequest { client_send_time } => {
                // 处理时间同步请求
                let mut sync = delay_compensation::TimeSyncMessage::new(*client_send_time);
                sync.server_receive_time = current_timestamp_ms();
                sync.server_send_time = current_timestamp_ms();

                if let Ok(mut delay_comp) = delay_compensation.lock() {
                    let response = delay_comp.process_sync_request(client_id, sync);
                    let response_msg = NetworkMessage::TimeSyncResponse { sync: response };
                    if let Ok(data) = Self::serialize_message(&response_msg) {
                        let _ = stream.write_all(&data);
                    }
                }
            }
            _ => {
                // 其他消息类型的处理
            }
        }
    }

    /// 广播消息给所有客户端
    pub fn broadcast(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        let _clients_guard = self
            .clients
            .lock()
            .map_err(|e| NetworkError::SendError(format!("Lock error: {}", e)))?;

        let _data = Self::serialize_message(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // NOTE: 实际实现中需要将消息发送到每个客户端的流
        // 这里简化处理，实际应该维护每个客户端的TcpStream

        Ok(())
    }

    /// 发送消息给特定客户端
    pub fn send_to_client(
        &self,
        client_id: u64,
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let clients_guard = self
            .clients
            .lock()
            .map_err(|e| NetworkError::SendError(format!("Lock error: {}", e)))?;

        if !clients_guard.contains_key(&client_id) {
            return Err(NetworkError::InvalidPeerId);
        }

        let _data = Self::serialize_message(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // NOTE: 实际实现中需要将消息发送到客户端的流

        Ok(())
    }

    /// 获取客户端连接数
    pub fn client_count(&self) -> usize {
        self.clients.lock().unwrap().len()
    }

    /// 获取所有客户端ID
    pub fn get_client_ids(&self) -> Vec<u64> {
        self.clients.lock().unwrap().keys().copied().collect()
    }

    /// 更新服务器tick
    pub fn update_tick(&self) {
        *self.current_tick.lock().unwrap() += 1;
    }

    /// 获取当前tick
    pub fn current_tick(&self) -> u64 {
        *self.current_tick.lock().unwrap()
    }

    /// 心跳检查器（在独立线程中运行）
    fn heartbeat_checker(
        clients: Arc<Mutex<HashMap<u64, ClientConnection>>>,
        running: Arc<Mutex<bool>>,
        timeout_ms: u64,
    ) {
        while *running.lock().unwrap() {
            thread::sleep(Duration::from_secs(1));

            let mut clients_guard = clients.lock().unwrap();
            let mut to_remove = Vec::new();

            for (client_id, conn) in clients_guard.iter() {
                if conn.is_timeout(timeout_ms) {
                    to_remove.push(*client_id);
                }
            }

            for client_id in to_remove {
                clients_guard.remove(&client_id);
            }
        }
    }

    /// 序列化消息
    fn serialize_message(message: &NetworkMessage) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(message)
    }

    /// 反序列化消息
    fn deserialize_message(data: &[u8]) -> Result<NetworkMessage, bincode::Error> {
        bincode::deserialize(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    fn test_client_connection() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let conn = ClientConnection::new(1, addr);
        assert_eq!(conn.client_id, 1);
        assert_eq!(conn.state, ConnectionState::Connecting);
    }

    #[test]
    fn test_server_creation() {
        let config = ServerConfig::default();
        let server = GameServer::new(config);
        assert_eq!(server.client_count(), 0);
    }
}
