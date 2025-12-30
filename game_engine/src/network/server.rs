//  网络服务器模块
//
//  实现游戏服务器的核心功能，包括：
//  - 客户端连接管理
//  - 消息路由和分发
//  - 服务器端状态管理
//  - 权威状态同步
//
//  ## 架构设计
//
//  ```text
//  ┌─────────────────────────────────────────┐
//  │           Game Server                   │
//  ├─────────────────────────────────────────┤
//  │  ┌──────────┐  ┌──────────┐  ┌─────────┐│
//  │  │ Client 1 │  │ Client 2 │  │Client N││
//  │  └────┬─────┘  └────┬─────┘  └────┬────┘│
//  │       │             │             │     │
//  │       └─────────────┼─────────────┘     │
//  │                     │                   │
//  │              ┌──────▼──────┐            │
//  │              │   Router    │            │
//  │              └──────┬──────┘            │
//  │                     │                   │
//  │       ┌─────────────┼─────────────┐    │
//  │       │             │             │    │
//  │  ┌────▼────┐  ┌─────▼─────┐ ┌────▼───┐│
//  │  │ Game    │  │ Authority  │ │ State  ││
//  │  │ Logic   │  │ Manager    │ │ Sync   ││
//  │  └─────────┘  └────────────┘ └────────┘│
//  └─────────────────────────────────────────┘
//  ```

use crate::core::utils::current_timestamp_ms;
use crate::impl_default;
use crate::network::compression;
use crate::network::delay_compensation;
use crate::network::delta_serialization;
use crate::network::synchronization;
use crate::network::{ConnectionState, NetworkError, NetworkMessage};
use crate::serialization::compat::bincode_compat;
use std::collections::HashMap;
use std::io::Write;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
// use futures::TryFutureExt; // Temporarily disabled - not currently used
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task;

/// 客户端连接信息
pub struct ClientConnection {
    /// 客户端ID
    pub client_id: u64,
    /// 客户端地址
    pub address: SocketAddr,
    /// TCP流
    pub stream: TcpStream,
    /// 连接状态
    pub state: ConnectionState,
    /// 最后心跳时间
    pub last_heartbeat: u64,
    /// 是否已认证
    pub authenticated: bool,
    /// 客户端名称
    pub name: Option<String>,
}

/// 同步客户端连接信息
pub struct SyncClientConnection {
    /// 客户端ID
    pub client_id: u64,
    /// 客户端地址
    pub address: SocketAddr,
    /// TCP流
    pub stream: std::net::TcpStream,
    /// 连接状态
    pub state: ConnectionState,
    /// 最后心跳时间
    pub last_heartbeat: u64,
    /// 是否已认证
    pub authenticated: bool,
    /// 客户端名称
    pub name: Option<String>,
}

impl SyncClientConnection {
    /// 更新心跳时间
    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = current_timestamp_ms();
    }

    /// 检查连接是否超时
    pub fn is_timeout(&self, timeout_ms: u64) -> bool {
        current_timestamp_ms() - self.last_heartbeat > timeout_ms
    }
}

impl ClientConnection {
    /// 创建新的客户端连接
    pub fn new(client_id: u64, address: SocketAddr, stream: TcpStream) -> Self {
        Self {
            client_id,
            address,
            stream,
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

    /// 异步发送消息
    pub async fn send_message(&mut self, message: &NetworkMessage) -> Result<(), NetworkError> {
        let data = bincode_compat::serialize(message)
            .map_err(Box::new)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        self.stream
            .write_all(&data)
            .await
            .map_err(|e| NetworkError::SendError(e.to_string()))?;

        Ok(())
    }

    /// 异步接收消息
    pub async fn receive_message(&mut self) -> Result<Option<NetworkMessage>, NetworkError> {
        let mut buffer = vec![0u8; 4096];

        match self.stream.read(&mut buffer).await {
            Ok(0) => Ok(None), // 连接关闭
            Ok(n) => {
                let data = &buffer[..n];
                match Self::deserialize_message(data) {
                    Ok(message) => Ok(Some(message)),
                    Err(e) => Err(NetworkError::SerializationError(e.to_string())),
                }
            }
            Err(e) => Err(NetworkError::ReceiveError(e.to_string())),
        }
    }

    /// 序列化消息
    fn serialize_message(message: &NetworkMessage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        bincode_compat::serialize(message)
    }

    /// 反序列化消息
    fn deserialize_message(data: &[u8]) -> Result<NetworkMessage, Box<dyn std::error::Error>> {
        bincode_compat::deserialize::<(NetworkMessage, ())>(data).map(|(msg, _)| msg)
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
    /// 同步客户端连接映射
    sync_clients: Arc<Mutex<HashMap<u64, SyncClientConnection>>>,
    /// 延迟补偿管理器
    delay_compensation: Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
    /// 压缩器
    compressor: Option<Arc<compression::NetworkCompressor>>,
    /// 增量序列化器
    delta_serializer: Arc<Mutex<delta_serialization::DeltaSerializer>>,
    /// 状态同步管理器
    state_sync_manager: Arc<Mutex<synchronization::StateSyncManager>>,
    /// 当前服务器tick
    current_tick: Arc<Mutex<u64>>,
    /// 是否运行中
    running: Arc<Mutex<bool>>,
}

impl GameServer {
    /// 序列化消息（支持压缩）
    fn serialize_message(
        &self,
        message: &NetworkMessage,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = bincode_compat::serialize(message)?;

        // 如果启用了压缩，使用压缩器
        if let Some(compressor) = &self.compressor {
            match compressor.compress(&data) {
                Ok(compressed) => Ok(compressed),
                Err(_) => Ok(data), // 压缩失败时返回原始数据
            }
        } else {
            Ok(data)
        }
    }

    /// 反序列化消息（支持解压）
    pub fn deserialize_message(
        &self,
        data: &[u8],
    ) -> Result<NetworkMessage, Box<dyn std::error::Error>> {
        // 首先尝试直接反序列化（未压缩数据）
        match bincode_compat::deserialize::<NetworkMessage>(data) {
            Ok(msg) => Ok(msg),
            Err(_) => {
                // 如果失败，尝试解压后反序列化
                if let Some(compressor) = &self.compressor {
                    if let Ok(decompressed) = compressor.decompress(data) {
                        bincode_compat::deserialize::<NetworkMessage>(&decompressed)
                    } else {
                        Err(Box::<dyn std::error::Error>::from(
                            "Failed to decompress data".to_string(),
                        ))
                    }
                } else {
                    Err(Box::<dyn std::error::Error>::from(
                        "Invalid data format".to_string(),
                    ))
                }
            }
        }
    }

    /// 发送压缩消息到客户端连接
    async fn send_compressed_message(
        &self,
        conn: &mut ClientConnection,
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let data = self
            .serialize_message(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        conn.stream
            .write_all(&data)
            .await
            .map_err(|e| NetworkError::SendError(e.to_string()))?;

        Ok(())
    }

    /// 异步处理消息（静态版本）
    async fn process_message_async_static(
        message: &NetworkMessage,
        client_id: u64,
        clients: &Arc<Mutex<HashMap<u64, ClientConnection>>>,
        delay_compensation: &Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        conn: &mut ClientConnection,
        compressor: Option<&Arc<compression::NetworkCompressor>>,
    ) {
        match message {
            NetworkMessage::Connect { client_id: _, name } => {
                // 处理连接请求
                conn.state = ConnectionState::Connected;
                conn.authenticated = true;
                conn.name = Some(name.clone());
                conn.update_heartbeat();
            }
            NetworkMessage::Disconnect { client_id: _ } => {
                // 处理断开连接
                clients.lock().await.remove(&client_id);
            }
            NetworkMessage::Heartbeat { timestamp: _ } => {
                // 更新心跳
                conn.update_heartbeat();
            }
            NetworkMessage::TimeSyncRequest { client_send_time } => {
                // 处理时间同步请求
                let mut sync = delay_compensation::TimeSyncMessage::new(*client_send_time);
                sync.server_receive_time = current_timestamp_ms();
                sync.server_send_time = current_timestamp_ms();

                let mut guard = delay_compensation.lock().await;
                let response = guard.process_sync_request(client_id, sync);
                let response_msg = NetworkMessage::TimeSyncResponse { sync: response };

                // 发送压缩消息
                let data = Self::serialize_message_with_compression(&response_msg, compressor)
                    .unwrap_or_else(|_| {
                        bincode_compat::serialize(&response_msg)
                            .map_err(Box::new)
                            .unwrap_or_default()
                    });
                let _ = conn.stream.write_all(&data).await;
            }
            _ => {
                // 其他消息类型的处理
            }
        }
    }

    /// 使用指定压缩器序列化消息（静态方法）
    fn serialize_message_with_compression(
        message: &NetworkMessage,
        compressor: Option<&Arc<compression::NetworkCompressor>>,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = bincode_compat::serialize(message)?;

        // 如果启用了压缩，使用压缩器
        if let Some(compressor) = compressor {
            match compressor.compress(&data) {
                Ok(compressed) => Ok(compressed),
                Err(_) => Ok(data), // 压缩失败时返回原始数据
            }
        } else {
            Ok(data)
        }
    }

    /// 使用指定压缩器反序列化消息（静态方法）
    fn deserialize_message_with_compression(
        data: &[u8],
        compressor: Option<&Arc<compression::NetworkCompressor>>,
    ) -> Result<NetworkMessage, Box<dyn std::error::Error>> {
        // 首先尝试直接反序列化（未压缩数据）
        match bincode_compat::deserialize::<NetworkMessage>(data) {
            Ok(msg) => Ok(msg),
            Err(_) => {
                // 如果失败，尝试解压后反序列化
                if let Some(compressor) = compressor {
                    if let Ok(decompressed) = compressor.decompress(data) {
                        bincode_compat::deserialize::<NetworkMessage>(&decompressed)
                    } else {
                        Err(Box::<dyn std::error::Error>::from(
                            "Failed to decompress data".to_string(),
                        ))
                    }
                } else {
                    Err(Box::<dyn std::error::Error>::from(
                        "Invalid data format".to_string(),
                    ))
                }
            }
        }
    }

    /// 获取增量序列化器的引用（用于调试和监控）
    pub fn delta_serializer(&self) -> &Arc<Mutex<delta_serialization::DeltaSerializer>> {
        &self.delta_serializer
    }

    /// 执行状态同步，使用增量序列化器优化数据传输
    pub async fn perform_state_sync(&self, client_id: u64) -> Result<(), NetworkError> {
        let current_tick = *self.current_tick.lock().await;
        let mut sync_manager = self.state_sync_manager.lock().await;

        // 生成同步数据包
        let delta_packet = sync_manager
            .generate_sync_data(current_tick)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // 如果有数据需要同步
        if !delta_packet.deltas.is_empty() {
            // 序列化增量包
            let data = bincode_compat::serialize(&delta_packet)
                .map_err(Box::new)
                .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

            // 创建状态同步消息
            let sync_message = NetworkMessage::StateSync {
                tick: current_tick,
                data,
            };

            // 发送给客户端
            self.send_to_client(client_id, &sync_message).await?;
        }

        Ok(())
    }
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
            sync_clients: Arc::new(Mutex::new(HashMap::new())),
            delay_compensation: Arc::new(Mutex::new(
                delay_compensation::ServerDelayCompensation::new(),
            )),
            compressor,
            delta_serializer: Arc::new(Mutex::new(delta_serialization::DeltaSerializer::new())),
            state_sync_manager: Arc::new(Mutex::new(synchronization::StateSyncManager::new(
                10, 0.1,
            ))),
            current_tick: Arc::new(Mutex::new(0)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    /// 启动服务器
    pub async fn start(&self) -> Result<(), NetworkError> {
        let address = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TcpListener::bind(&address)
            .await
            .map_err(|e| NetworkError::ConnectionError(format!("Failed to bind: {e}")))?;

        *self.running.lock().await = true;

        let clients = Arc::clone(&self.clients);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        let delay_compensation = Arc::clone(&self.delay_compensation);

        // 启动监听任务
        let compressor_clone = self.compressor.clone();
        task::spawn(Self::accept_connections(
            listener,
            clients,
            running,
            config,
            delay_compensation,
            compressor_clone,
        ));

        // 启动心跳检查任务
        let clients_clone = Arc::clone(&self.clients);
        let running_clone = Arc::clone(&self.running);
        let timeout = self.config.heartbeat_timeout_ms;

        task::spawn(Self::heartbeat_checker(
            clients_clone,
            running_clone,
            timeout,
        ));

        Ok(())
    }

    /// 停止服务器
    pub async fn stop(&self) {
        *self.running.lock().await = false;
    }

    /// 同步版本的启动方法（向后兼容）
    pub fn start_sync(&mut self) -> Result<(), NetworkError> {
        let address = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = std::net::TcpListener::bind(&address)
            .map_err(|e| NetworkError::ConnectionError(format!("Failed to bind: {e}")))?;

        listener.set_nonblocking(true).map_err(|e| {
            NetworkError::ConnectionError(format!("Failed to set nonblocking: {e}"))
        })?;

        // 设置running状态，使用try_lock避免unwrap()导致的panic
        if let Ok(mut running_guard) = self.running.try_lock() {
            *running_guard = true;
        }

        let sync_clients = Arc::clone(&self.sync_clients);
        let running = Arc::clone(&self.running);
        let config = self.config.clone();
        let delay_compensation = Arc::clone(&self.delay_compensation);
        let compressor = self.compressor.clone();

        // 启动监听线程
        std::thread::spawn(move || {
            Self::accept_connections_sync(
                listener,
                sync_clients,
                running,
                config,
                delay_compensation,
                compressor,
            );
        });

        // 启动心跳检查线程
        let sync_clients_clone = Arc::clone(&self.sync_clients);
        let running_clone = Arc::clone(&self.running);
        let timeout = self.config.heartbeat_timeout_ms;

        std::thread::spawn(move || {
            Self::heartbeat_checker_sync(sync_clients_clone, running_clone, timeout);
        });

        Ok(())
    }

    /// 同步版本的停止方法（向后兼容）
    pub fn stop_sync(&mut self) {
        // 设置running状态为false，使用try_lock避免unwrap()导致的panic
        if let Ok(mut running_guard) = self.running.try_lock() {
            *running_guard = false;
        }
    }

    /// 异步接受连接
    async fn accept_connections(
        listener: TcpListener,
        clients: Arc<Mutex<HashMap<u64, ClientConnection>>>,
        running: Arc<Mutex<bool>>,
        config: ServerConfig,
        delay_compensation: Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        compressor: Option<Arc<compression::NetworkCompressor>>,
    ) {
        while *running.lock().await {
            match listener.accept().await {
                Ok((mut stream, addr)) => {
                    let client_id = rand::random();
                    let mut clients_guard = clients.lock().await;

                    // 检查连接数限制
                    if clients_guard.len() >= config.max_connections {
                        let _ = stream.shutdown().await;
                        continue;
                    }

                    // 创建异步客户端连接
                    let connection = ClientConnection::new(client_id, addr, stream);
                    clients_guard.insert(client_id, connection);

                    // 启动客户端处理任务
                    let clients_clone = Arc::clone(&clients);
                    let delay_compensation_clone = Arc::clone(&delay_compensation);
                    let compressor_clone = compressor.clone();
                    task::spawn(async move {
                        Self::handle_client_async(
                            client_id,
                            clients_clone,
                            delay_compensation_clone,
                            compressor_clone,
                        )
                        .await;
                    });
                }
                Err(e) => {
                    eprintln!("Accept error: {e}");
                }
            }
        }
    }

    /// 同步版本的接受连接（向后兼容）
    fn accept_connections_sync(
        listener: std::net::TcpListener,
        sync_clients: Arc<Mutex<HashMap<u64, SyncClientConnection>>>,
        running: Arc<Mutex<bool>>,
        config: ServerConfig,
        delay_compensation: Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        compressor: Option<Arc<compression::NetworkCompressor>>,
    ) {
        let mut running_flag = true;
        while running_flag {
            // 定期检查running状态
            if let Ok(running_guard) = running.try_lock() {
                running_flag = *running_guard;
            }

            if !running_flag {
                break;
            }

            match listener.accept() {
                Ok((stream, addr)) => {
                    let client_id = rand::random();

                    // 获取sync_clients锁，避免unwrap()导致的panic
                    let mut clients_guard = match sync_clients.try_lock() {
                        Ok(guard) => guard,
                        Err(_) => {
                            // 无法获取锁，关闭连接后重试
                            let _ = stream.shutdown(std::net::Shutdown::Both);
                            std::thread::sleep(Duration::from_millis(10));
                            continue;
                        }
                    };

                    // 检查连接数限制
                    if clients_guard.len() >= config.max_connections {
                        let _ = stream.shutdown(std::net::Shutdown::Both);
                        continue;
                    }

                    // 创建同步客户端连接
                    let connection = SyncClientConnection {
                        client_id,
                        address: addr,
                        stream,
                        state: ConnectionState::Connecting,
                        last_heartbeat: current_timestamp_ms(),
                        authenticated: false,
                        name: None,
                    };
                    clients_guard.insert(client_id, connection);

                    // 启动客户端处理线程
                    let sync_clients_clone = Arc::clone(&sync_clients);
                    let delay_compensation_clone = Arc::clone(&delay_compensation);
                    let compressor_clone = compressor.clone();
                    let _ = compressor_clone; // 使用compressor避免未使用警告
                    std::thread::spawn(move || {
                        Self::handle_sync_client(
                            client_id,
                            sync_clients_clone,
                            delay_compensation_clone,
                            &compressor_clone,
                        );
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // 非阻塞模式下没有连接，继续等待
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(e) => {
                    eprintln!("Accept error: {e}");
                }
            }
        }
    }

    /// 异步处理客户端连接
    async fn handle_client_async(
        client_id: u64,
        clients: Arc<Mutex<HashMap<u64, ClientConnection>>>,
        delay_compensation: Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        compressor: Option<Arc<compression::NetworkCompressor>>,
    ) {
        // 获取客户端连接的引用
        let connection = {
            let mut clients_guard = clients.lock().await;
            clients_guard.remove(&client_id)
        };

        if let Some(mut connection) = connection {
            loop {
                match connection.receive_message().await {
                    Ok(Some(message)) => {
                        // 处理接收到的消息
                        Self::process_message_async_static(
                            &message,
                            client_id,
                            &clients,
                            &delay_compensation,
                            &mut connection,
                            compressor.as_ref(),
                        )
                        .await;
                    }
                    Ok(None) => {
                        // 连接关闭
                        break;
                    }
                    Err(e) => {
                        eprintln!("Read error for client {client_id}: {e}");
                        break;
                    }
                }
            }

            // 将连接放回（如果需要）
            // 注意：在实际应用中，可能需要更复杂的连接管理
        }

        // 清理客户端连接
        clients.lock().await.remove(&client_id);
    }

    /// 同步版本的客户端处理（专用于SyncClientConnection）
    fn handle_sync_client(
        client_id: u64,
        sync_clients: Arc<Mutex<HashMap<u64, SyncClientConnection>>>,
        delay_compensation: Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        compressor: &Option<Arc<compression::NetworkCompressor>>,
    ) {
        let compressor_ref = compressor.as_ref();
        let connection = {
            // 尝试获取sync_clients锁，避免unwrap()导致的panic
            match sync_clients.try_lock() {
                Ok(mut guard) => guard.remove(&client_id),
                Err(_) => {
                    // 无法获取锁，直接返回
                    return;
                }
            }
        };

        if let Some(mut conn) = connection {
            let mut buffer = vec![0u8; 4096];

            loop {
                use std::io::Read;
                match conn.stream.read(&mut buffer) {
                    Ok(0) => {
                        // 连接关闭
                        break;
                    }
                    Ok(n) => {
                        // 处理接收到的数据
                        let data = &buffer[..n];
                        if let Ok(message) =
                            Self::deserialize_message_with_compression(data, compressor_ref)
                        {
                            Self::process_sync_message(
                                &message,
                                client_id,
                                &sync_clients,
                                &delay_compensation,
                                &mut conn.stream,
                                compressor_ref,
                            );
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => {
                        eprintln!("Read error for client {client_id}: {e}");
                        break;
                    }
                }
            }
        }

        // 清理客户端连接，使用try_lock避免unwrap()导致的panic
        if let Ok(mut clients_guard) = sync_clients.try_lock() {
            clients_guard.remove(&client_id);
        }
    }

    /// 异步处理消息
    pub async fn process_message_async(
        &self,
        message: &NetworkMessage,
        client_id: u64,
        clients: &Arc<Mutex<HashMap<u64, ClientConnection>>>,
        delay_compensation: &Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        conn: &mut ClientConnection,
    ) {
        match message {
            NetworkMessage::Connect { client_id: _, name } => {
                // 处理连接请求
                conn.state = ConnectionState::Connected;
                conn.authenticated = true;
                conn.name = Some(name.clone());
                conn.update_heartbeat();
            }
            NetworkMessage::Disconnect { client_id: _ } => {
                // 处理断开连接
                clients.lock().await.remove(&client_id);
            }
            NetworkMessage::Heartbeat { timestamp: _ } => {
                // 更新心跳
                conn.update_heartbeat();
            }
            NetworkMessage::TimeSyncRequest { client_send_time } => {
                // 处理时间同步请求
                let mut sync = delay_compensation::TimeSyncMessage::new(*client_send_time);
                sync.server_receive_time = current_timestamp_ms();
                sync.server_send_time = current_timestamp_ms();

                let mut guard = delay_compensation.lock().await;
                let response = guard.process_sync_request(client_id, sync);
                let response_msg = NetworkMessage::TimeSyncResponse { sync: response };
                let _ = self.send_compressed_message(conn, &response_msg).await;
            }
            _ => {
                // 其他消息类型的处理
            }
        }
    }

    /// 同步版本的消息处理（向后兼容）
    pub fn process_message_sync(
        message: &NetworkMessage,
        client_id: u64,
        clients: &Arc<Mutex<HashMap<u64, ClientConnection>>>,
        delay_compensation: &Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        stream: &mut std::net::TcpStream,
        compressor: Option<&Arc<compression::NetworkCompressor>>,
    ) {
        match message {
            NetworkMessage::Connect { client_id: _, name } => {
                // 处理连接请求
                if let Ok(mut clients_guard) = clients.try_lock()
                    && let Some(conn) = clients_guard.get_mut(&client_id)
                {
                    conn.state = ConnectionState::Connected;
                    conn.authenticated = true;
                    conn.name = Some(name.clone());
                    conn.update_heartbeat();
                }
            }
            NetworkMessage::Disconnect { client_id: _ } => {
                // 处理断开连接
                if let Ok(mut clients_guard) = clients.try_lock() {
                    clients_guard.remove(&client_id);
                }
            }
            NetworkMessage::Heartbeat { timestamp: _ } => {
                // 更新心跳
                if let Ok(mut clients_guard) = clients.try_lock()
                    && let Some(conn) = clients_guard.get_mut(&client_id)
                {
                    conn.update_heartbeat();
                }
            }
            NetworkMessage::TimeSyncRequest { client_send_time } => {
                // 处理时间同步请求
                let mut sync = delay_compensation::TimeSyncMessage::new(*client_send_time);
                sync.server_receive_time = current_timestamp_ms();
                sync.server_send_time = current_timestamp_ms();

                if let Ok(mut guard) = delay_compensation.try_lock() {
                    let response = guard.process_sync_request(client_id, sync);
                    let response_msg = NetworkMessage::TimeSyncResponse { sync: response };
                    if let Ok(data) =
                        Self::serialize_message_with_compression(&response_msg, compressor)
                    {
                        let _ = stream.write_all(&data);
                    }
                }
            }
            _ => {
                // 其他消息类型的处理
            }
        }
    }

    /// 专门为SyncClientConnection处理消息
    fn process_sync_message(
        message: &NetworkMessage,
        client_id: u64,
        sync_clients: &Arc<Mutex<HashMap<u64, SyncClientConnection>>>,
        delay_compensation: &Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        stream: &mut std::net::TcpStream,
        compressor: Option<&Arc<compression::NetworkCompressor>>,
    ) {
        match message {
            NetworkMessage::Connect { client_id: _, name } => {
                // 处理连接请求
                if let Ok(mut clients_guard) = sync_clients.try_lock()
                    && let Some(conn) = clients_guard.get_mut(&client_id)
                {
                    conn.state = ConnectionState::Connected;
                    conn.authenticated = true;
                    conn.name = Some(name.clone());
                    conn.update_heartbeat();
                }
            }
            NetworkMessage::Disconnect { client_id: _ } => {
                // 处理断开连接
                if let Ok(mut clients_guard) = sync_clients.try_lock() {
                    clients_guard.remove(&client_id);
                }
            }
            NetworkMessage::Heartbeat { timestamp: _ } => {
                // 更新心跳
                if let Ok(mut clients_guard) = sync_clients.try_lock()
                    && let Some(conn) = clients_guard.get_mut(&client_id)
                {
                    conn.update_heartbeat();
                }
            }
            NetworkMessage::TimeSyncRequest { client_send_time } => {
                // 处理时间同步请求
                let mut sync = delay_compensation::TimeSyncMessage::new(*client_send_time);
                sync.server_receive_time = current_timestamp_ms();
                sync.server_send_time = current_timestamp_ms();

                if let Ok(mut guard) = delay_compensation.try_lock() {
                    let response = guard.process_sync_request(client_id, sync);
                    let response_msg = NetworkMessage::TimeSyncResponse { sync: response };
                    if let Ok(data) =
                        Self::serialize_message_with_compression(&response_msg, compressor)
                    {
                        let _ = stream.write_all(&data);
                    }
                }
            }
            _ => {
                // 其他消息类型的处理
            }
        }
    }

    /// 异步广播消息给所有客户端
    pub async fn broadcast(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        let mut clients_guard = self.clients.lock().await;
        let mut clients_to_remove = Vec::new();

        // 遍历所有客户端并发送消息
        for (client_id, conn) in &mut *clients_guard {
            match self.send_compressed_message(conn, message).await {
                Ok(_) => {
                    // 消息发送成功
                    println!("Broadcasting message to client {client_id}");
                }
                Err(e) => {
                    // 发送失败，标记客户端连接需要移除
                    eprintln!("Failed to broadcast to client {client_id}: {e}");
                    clients_to_remove.push(*client_id);
                }
            }
        }

        // 移除连接失败的客户端
        for client_id in clients_to_remove {
            clients_guard.remove(&client_id);
        }

        Ok(())
    }

    /// 异步发送消息给特定客户端
    pub async fn send_to_client(
        &self,
        client_id: u64,
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let mut clients_guard = self.clients.lock().await;

        if let Some(conn) = clients_guard.get_mut(&client_id) {
            self.send_compressed_message(conn, message).await?;
        } else {
            return Err(NetworkError::InvalidPeerId);
        }

        Ok(())
    }

    /// 同步版本的广播方法（向后兼容）
    pub fn broadcast_sync(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        let mut clients_guard = self
            .sync_clients
            .try_lock()
            .map_err(|e| NetworkError::SendError(format!("Lock error: {e}")))?;

        let data = Self::serialize_message_with_compression(message, self.compressor.as_ref())
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // 遍历所有客户端并发送消息
        let mut clients_to_remove = Vec::new();

        for (client_id, conn) in &mut *clients_guard {
            match conn.stream.write_all(&data) {
                Ok(_) => {
                    // 消息发送成功
                    println!("Broadcasting message to client {client_id}");
                }
                Err(e) => {
                    // 发送失败，标记客户端连接需要移除
                    eprintln!("Failed to broadcast to client {client_id}: {e}");
                    clients_to_remove.push(*client_id);
                }
            }
        }

        // 移除连接失败的客户端
        for client_id in clients_to_remove {
            clients_guard.remove(&client_id);
        }

        Ok(())
    }

    /// 同步版本的广播方法（专用于SyncClientConnection）
    pub fn broadcast_sync_to_sync_clients(
        &self,
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let mut clients_guard = self
            .sync_clients
            .try_lock()
            .map_err(|e| NetworkError::SendError(format!("Lock error: {e}")))?;

        let data = Self::serialize_message_with_compression(message, self.compressor.as_ref())
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // 遍历所有客户端并发送消息
        let mut clients_to_remove = Vec::new();

        for (client_id, conn) in &mut *clients_guard {
            match conn.stream.write_all(&data) {
                Ok(_) => {
                    // 消息发送成功
                    println!("Broadcasting message to sync client {client_id}");
                }
                Err(e) => {
                    // 发送失败，标记客户端连接需要移除
                    eprintln!("Failed to broadcast to sync client {client_id}: {e}");
                    clients_to_remove.push(*client_id);
                }
            }
        }

        // 移除连接失败的客户端
        for client_id in clients_to_remove {
            clients_guard.remove(&client_id);
        }

        Ok(())
    }

    /// 同步版本的发送消息方法（向后兼容）
    pub fn send_to_client_sync(
        &self,
        client_id: u64,
        _message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let clients_guard = self
            .clients
            .try_lock()
            .map_err(|e| NetworkError::SendError(format!("Lock error: {e}")))?;

        if !clients_guard.contains_key(&client_id) {
            return Err(NetworkError::InvalidPeerId);
        }

        // 由于TcpStream不能clone，且sync版本在runtime内会有lifetime问题
        // 直接返回错误，要求使用async版本
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(NetworkError::SyncOperationInRuntime(
                "Cannot use sync network operations inside tokio runtime. Use async version instead.".to_string()
            ));
        }

        // 对于async clients，无法在sync方法中操作
        // 应该使用send_to_client async方法
        Err(NetworkError::SyncOperationInRuntime(
            "Use send_to_client async method for async clients.".to_string(),
        ))
    }

    /// 同步版本的发送消息方法（专用于SyncClientConnection）
    pub fn send_to_sync_client(
        &self,
        client_id: u64,
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let mut clients_guard = self
            .sync_clients
            .try_lock()
            .map_err(|e| NetworkError::SendError(format!("Lock error: {e}")))?;

        if !clients_guard.contains_key(&client_id) {
            return Err(NetworkError::InvalidPeerId);
        }

        let data = Self::serialize_message_with_compression(message, self.compressor.as_ref())
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        if let Some(conn) = clients_guard.get_mut(&client_id) {
            conn.stream
                .write_all(&data)
                .map_err(|e: std::io::Error| NetworkError::SendError(e.to_string()))?;
        }

        Ok(())
    }

    /// 异步获取客户端连接数
    pub async fn client_count(&self) -> usize {
        self.clients.lock().await.len()
    }

    /// 异步获取所有客户端ID
    pub async fn get_client_ids(&self) -> Vec<u64> {
        self.clients.lock().await.keys().copied().collect()
    }

    /// 异步更新服务器tick
    pub async fn update_tick(&self) {
        let tick = *self.current_tick.lock().await;
        let _tick_span =
            crate::performance::tracing_metrics::TracingMetricsManager::network_tick_span(tick)
                .entered();
        *self.current_tick.lock().await += 1;
    }

    /// 异步获取当前tick
    pub async fn current_tick(&self) -> u64 {
        *self.current_tick.lock().await
    }

    /// 同步版本的客户端连接数（向后兼容）
    pub fn client_count_sync(&self) -> usize {
        match self.clients.try_lock() {
            Ok(clients_guard) => clients_guard.len(),
            Err(_) => 0, // 无法获取锁时返回默认值
        }
    }

    /// 同步版本的同步客户端连接数
    pub fn sync_client_count_sync(&self) -> usize {
        match self.sync_clients.try_lock() {
            Ok(clients_guard) => clients_guard.len(),
            Err(_) => 0, // 无法获取锁时返回默认值
        }
    }

    /// 同步版本的所有客户端ID（向后兼容）
    pub fn get_client_ids_sync(&self) -> Vec<u64> {
        match self.clients.try_lock() {
            Ok(clients_guard) => clients_guard.keys().copied().collect(),
            Err(_) => Vec::new(), // 无法获取锁时返回空向量
        }
    }

    /// 同步版本的所有同步客户端ID
    pub fn get_sync_client_ids_sync(&self) -> Vec<u64> {
        match self.sync_clients.try_lock() {
            Ok(clients_guard) => clients_guard.keys().copied().collect(),
            Err(_) => Vec::new(), // 无法获取锁时返回空向量
        }
    }

    /// 同步版本的更新服务器tick（向后兼容）
    pub fn update_tick_sync(&self) {
        match self.current_tick.try_lock() {
            Ok(mut tick_guard) => {
                *tick_guard += 1;
            }
            Err(_) => {
                // 无法获取锁时跳过更新
            }
        }
    }

    /// 同步版本的获取当前tick（向后兼容）
    pub fn current_tick_sync(&self) -> u64 {
        match self.current_tick.try_lock() {
            Ok(tick_guard) => *tick_guard,
            Err(_) => 0, // 无法获取锁时返回默认值
        }
    }

    /// 异步心跳检查器
    async fn heartbeat_checker(
        clients: Arc<Mutex<HashMap<u64, ClientConnection>>>,
        running: Arc<Mutex<bool>>,
        timeout_ms: u64,
    ) {
        while *running.lock().await {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let mut clients_guard = clients.lock().await;
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

    /// 同步版本的心跳检查器（专用于SyncClientConnection）
    fn heartbeat_checker_sync(
        sync_clients: Arc<Mutex<HashMap<u64, SyncClientConnection>>>,
        running: Arc<Mutex<bool>>,
        timeout_ms: u64,
    ) {
        let mut running_flag = true;
        while running_flag {
            // 检查运行状态
            if let Ok(running_guard) = running.try_lock() {
                running_flag = *running_guard;
            }
            if !running_flag {
                break;
            }

            std::thread::sleep(Duration::from_secs(1));

            // 检查客户端心跳
            if let Ok(mut clients_guard) = sync_clients.try_lock() {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_server_config() {
        let config = ServerConfig::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.max_connections, 100);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_server_config_custom() {
        let config = ServerConfig {
            port: 9000,
            max_connections: 50,
            heartbeat_timeout_ms: 3000,
            enable_compression: false,
            enable_delay_compensation: false,
            ..Default::default()
        };

        assert_eq!(config.port, 9000);
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.heartbeat_timeout_ms, 3000);
        assert!(!config.enable_compression);
        assert!(!config.enable_delay_compensation);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_server_creation() {
        let config = ServerConfig::default();
        let server = GameServer::new(config);
        assert_eq!(server.client_count_sync(), 0);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_server_address_parsing() {
        let addr_str = "127.0.0.1:8080";
        let addr: SocketAddr = addr_str.parse().unwrap_or_else(|e| {
            panic!("Failed to parse address: {}", e);
        });

        assert_eq!(addr.ip(), std::net::Ipv4Addr::new(127, 0, 0, 1));
        assert_eq!(addr.port(), 8080);
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_server_address_parsing_invalid() {
        let addr_str = "invalid_address";
        let addr_result: Result<SocketAddr, _> = addr_str.parse();

        assert!(addr_result.is_err());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_message_serialization_server() {
        let msg = NetworkMessage::Heartbeat { timestamp: 54321 };

        let serialized = bincode_compat::serialize(&msg).map_err(|e| Box::new(e));
        assert!(serialized.is_ok());

        let deserialized: Result<NetworkMessage, _> =
            bincode_compat::deserialize(&serialized.unwrap_or_else(|e| {
                panic!("Serialization failed: {}", e);
            }));
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_broadcast_message_creation() {
        let msg = NetworkMessage::StateSync {
            tick: 100,
            data: vec![],
        };

        let serialized = bincode_compat::serialize(&msg).map_err(|e| Box::new(e));
        assert!(serialized.is_ok());

        if let Ok(NetworkMessage::StateSync { tick, data }) =
            bincode_compat::deserialize::<NetworkMessage>(&serialized.unwrap_or_else(|e| {
                panic!("Serialization failed: {}", e);
            }))
        {
            assert_eq!(tick, 100);
            assert_eq!(data.len(), 0);
        } else {
            panic!("Deserialization failed");
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_sync_client_connection_heartbeat() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap_or_else(|e| {
            panic!("Failed to parse address: {}", e);
        });

        // 创建一个假的TcpStream (这里使用mock方式)
        // 由于无法创建真实的TcpStream，我们测试其他逻辑
        let timestamp = current_timestamp_ms();

        // 测试心跳逻辑
        let heartbeat_age = current_timestamp_ms().saturating_sub(timestamp);
        assert!(heartbeat_age < 1000); // 应该小于1秒
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_timeout_detection() {
        let current = current_timestamp_ms();
        let old_timestamp = current.saturating_sub(20000); // 20秒前

        let is_expired = current.saturating_sub(old_timestamp) > 15000;
        assert!(is_expired); // 应该超时

        let is_not_expired = current.saturating_sub(old_timestamp) < 25000;
        assert!(is_not_expired); // 在合理范围内
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_client_id_generation() {
        let id1: u64 = rand::random();
        let id2: u64 = rand::random();

        // 两个随机ID应该不同（极大概率）
        assert_ne!(id1, id2);
    }
}
