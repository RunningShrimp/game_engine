//! 网络服务器模块
<<<<<<< HEAD
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
=======
        match self.running.lock() {
            Ok(mut guard) => *guard = false,
            Err(poison) => {
                let mut guard = poison.into_inner();
                *guard = false;
                eprintln!("Warning: running boolean mutex poisoned on stop — recovered and set to false");
            }
        }
    }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
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
<<<<<<< HEAD
=======
    /// 连接池大小
    pub connection_pool_size: usize,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
}

impl_default!(ServerConfig {
    bind_address: "0.0.0.0".to_string(),
    port: 8080,
<<<<<<< HEAD
    max_connections: 100,
    heartbeat_timeout_ms: 30000,
    enable_compression: true,
    enable_delay_compensation: true,
});

=======
    max_connections: 200,
    heartbeat_timeout_ms: 30000,
    enable_compression: true,
    enable_delay_compensation: true,
    connection_pool_size: 10,
});

/// 会话管理模块
pub struct SessionManager {
    /// 会话映射（会话ID -> 客户端连接）
    sessions: RwLock<HashMap<Uuid, ClientConnection>>,
    /// 客户端ID -> 会话ID映射
    client_session_map: RwLock<HashMap<u64, Uuid>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            client_session_map: RwLock::new(HashMap::new()),
        }
    }

    /// 创建新会话
    pub fn create_session(&self, client_id: u64, address: SocketAddr) -> ClientConnection {
        let connection = ClientConnection::new(client_id, address);
        let session_id = connection.session_id;
        
        // 保存会话
        let mut sessions = match self.sessions.write() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: sessions RwLock poisoned — recovering");
                poison.into_inner()
            }
        };
        sessions.insert(session_id, connection.clone());

        let mut client_map = match self.client_session_map.write() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: client_session_map RwLock poisoned — recovering");
                poison.into_inner()
            }
        };
        client_map.insert(client_id, session_id);
        
        connection
    }

    /// 获取会话
    pub fn get_session(&self, session_id: &Uuid) -> Option<ClientConnection> {
        let sessions_guard = match self.sessions.read() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: sessions RwLock poisoned on read — recovering");
                poison.into_inner()
            }
        };
        sessions_guard.get(session_id).cloned()
    }

    /// 根据客户端ID获取会话
    pub fn get_session_by_client_id(&self, client_id: u64) -> Option<ClientConnection> {
        let client_map_guard = match self.client_session_map.read() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: client_session_map RwLock poisoned on read — recovering");
                poison.into_inner()
            }
        };

        if let Some(session_id) = client_map_guard.get(&client_id) {
            let sessions_guard = match self.sessions.read() {
                Ok(g) => g,
                Err(poison) => {
                    eprintln!("Warning: sessions RwLock poisoned on read — recovering");
                    poison.into_inner()
                }
            };

            sessions_guard.get(session_id).cloned()
        } else {
            None
        }
    }

    /// 更新会话
    pub fn update_session(&self, session: ClientConnection) {
        let mut sessions = match self.sessions.write() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: sessions RwLock poisoned during update — recovering");
                poison.into_inner()
            }
        };
        sessions.insert(session.session_id, session);
    }

    /// 删除会话
    pub fn remove_session(&self, session_id: &Uuid) {
        let mut sessions = match self.sessions.write() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: sessions RwLock poisoned during remove — recovering");
                poison.into_inner()
            }
        };

        if let Some(session) = sessions.remove(session_id) {
            let mut client_map = match self.client_session_map.write() {
                Ok(g) => g,
                Err(poison) => {
                    eprintln!("Warning: client_session_map RwLock poisoned during remove — recovering");
                    poison.into_inner()
                }
            };

            client_map.remove(&session.client_id);
        }
    }

    /// 获取当前会话数量
    pub fn session_count(&self) -> usize {
        let sessions_guard = match self.sessions.read() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: sessions RwLock poisoned on read (session_count) — recovering");
                poison.into_inner()
            }
        };

        sessions_guard.len()
    }
}

>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
/// 游戏服务器
pub struct GameServer {
    /// 配置
    config: ServerConfig,
    /// 客户端连接映射
<<<<<<< HEAD
    clients: Arc<Mutex<HashMap<u64, ClientConnection>>>,
=======
    clients: Arc<Mutex<HashMap<u64, Arc<tokio::sync::Mutex<TokioTcpStream>>>>>,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
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
<<<<<<< HEAD
=======
    /// 会话管理器
    session_manager: Arc<SessionManager>,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
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
<<<<<<< HEAD
=======
            session_manager: Arc::new(SessionManager::new()),
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        }
    }

    /// 启动服务器
<<<<<<< HEAD
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
=======
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        let address = format!("{}:{}", self.config.bind_address, self.config.port);
        let listener = TokioTcpListener::bind(&address)
            .await
            .map_err(|e| NetworkError::ConnectionError(format!("Failed to bind: {}", e)))?;

        match self.running.lock() {
            Ok(mut guard) => *guard = true,
            Err(poison) => {
                // Recover from poisoned mutex by taking the inner value.
                let mut guard = poison.into_inner();
                *guard = true;
                eprintln!("Warning: running boolean mutex poisoned on start — recovered and set to true");
            }
        }

        // 启动监听任务
        let clients_clone = Arc::clone(&self.clients);
        let config_clone = self.config.clone();
        let delay_compensation_clone = Arc::clone(&self.delay_compensation);
        let session_manager_clone = Arc::clone(&self.session_manager);
        let running_clone = Arc::clone(&self.running);

        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((mut stream, addr)) => {
                        let client_id = rand::random();
                        let mut clients_guard = match clients_clone.lock() {
                            Ok(g) => g,
                            Err(poison) => {
                                eprintln!("Warning: clients mutex poisoned in accept loop — recovering");
                                poison.into_inner()
                            }
                        };

                        // 检查连接数限制
                            // 检查连接数限制 — evaluate count without holding the guard while awaiting
                            let current_clients = clients_guard.len();
                            if current_clients >= config_clone.max_connections {
                                // release the guard before awaiting
                                drop(clients_guard);
                                let _ = stream.shutdown().await;
                                continue;
                            }

                        // 创建客户端连接会话
                        let connection = session_manager_clone.create_session(client_id, addr);

                        // 添加到客户端映射
                        let stream_arc = Arc::new(tokio::sync::Mutex::new(stream));
                        clients_guard.insert(client_id, stream_arc.clone());

                        // 启动客户端处理任务
                        let clients_clone = Arc::clone(&clients_clone);
                        let delay_compensation_clone = Arc::clone(&delay_compensation_clone);
                        let session_manager_clone = Arc::clone(&session_manager_clone);
                        
                        tokio::spawn(async move {
                            Self::handle_client(
                                stream_arc,
                                client_id,
                                clients_clone,
                                delay_compensation_clone,
                                session_manager_clone,
                            )
                            .await;
                        });
                    }
                    Err(e) => {
                        eprintln!("Accept error: {}", e);
                    }
                }

                // Evaluate running flag safely (guard against poisoning)
                let still_running = match running_clone.lock() {
                    Ok(g) => *g,
                    Err(poison) => {
                        eprintln!("Warning: running mutex poisoned in accept loop — assuming running=false and breaking");
                        // If poisoned, be conservative and stop accepting new connections.
                        *poison.into_inner()
                    }
                };

                if !still_running {
                    break;
                }
            }
        });

        // 启动心跳检查任务
        let clients_clone = Arc::clone(&self.clients);
        let running_clone = Arc::clone(&self.running);
        let timeout = self.config.heartbeat_timeout_ms;
        let session_manager_clone = Arc::clone(&self.session_manager);
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            while {
                // Check running state safely
                let running_val = match running_clone.lock() {
                    Ok(g) => *g,
                    Err(poison) => {
                        eprintln!("Warning: running mutex poisoned in heartbeat loop — treating as false and exiting");
                        *poison.into_inner()
                    }
                };

                running_val
            } {
                interval.tick().await;

                let clients_guard = match clients_clone.lock() {
                    Ok(g) => g,
                    Err(poison) => {
                        eprintln!("Warning: clients mutex poisoned in heartbeat loop — recovering");
                        poison.into_inner()
                    }
                };
                let mut to_remove = Vec::new();

                let session_read_guard = match session_manager_clone.sessions.read() {
                    Ok(g) => g,
                    Err(poison) => {
                        eprintln!("Warning: sessions RwLock poisoned on read during heartbeat — recovering");
                        poison.into_inner()
                    }
                };

                for (client_id, session) in session_read_guard.iter() {
                    if session.is_timeout(timeout) {
                        to_remove.push(*client_id);
                    }
                }
                let mut session_guard = match session_manager_clone.sessions.write() {
                    Ok(g) => g,
                    Err(poison) => {
                        eprintln!("Warning: sessions RwLock poisoned on write during heartbeat — recovering");
                        poison.into_inner()
                    }
                };

                let mut client_map_guard = match session_manager_clone.client_session_map.write() {
                    Ok(g) => g,
                    Err(poison) => {
                        eprintln!("Warning: client_session_map RwLock poisoned on write during heartbeat — recovering");
                        poison.into_inner()
                    }
                };
                
                for session_id in to_remove {
                    if let Some(session) = session_guard.remove(&session_id) {
                        client_map_guard.remove(&session.client_id);
                        // 关闭连接
                        if let Some(stream) = clients_guard.get(&session.client_id) {
                            let mut stream_guard = stream.lock().await;
                            let _ = stream_guard.shutdown().await;
                        }
                    }
                }
            }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        });

        Ok(())
    }

    /// 停止服务器
    pub fn stop(&mut self) {
<<<<<<< HEAD
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
=======
        match self.running.lock() {
            Ok(mut guard) => *guard = false,
            Err(poison) => {
                let mut guard = poison.into_inner();
                *guard = false;
                        // Collect expired sessions and the streams we need to shutdown while keeping locks only
                        // for the minimal time so we don't hold std::sync locks across await points.
                        let mut to_remove = Vec::new();
                        let mut streams_to_shutdown: Vec<Arc<tokio::sync::Mutex<TokioTcpStream>>> = Vec::new();

                        // Short-lived scope for read locks
                        {
                            let clients_guard = match clients_clone.lock() {
                                Ok(g) => g,
                                Err(poison) => {
                                    eprintln!("Warning: clients mutex poisoned in heartbeat loop (collect) — recovering");
                                    poison.into_inner()
                                }
                            };

                            let session_read_guard = match session_manager_clone.sessions.read() {
                                Ok(g) => g,
                                Err(poison) => {
                                    eprintln!("Warning: sessions RwLock poisoned on read during heartbeat (collect) — recovering");
                                    poison.into_inner()
                                }
                            };

                            for (session_id, session) in session_read_guard.iter() {
                                if session.is_timeout(timeout) {
                                    to_remove.push(*session_id);
                                    if let Some(stream_arc) = clients_guard.get(&session.client_id).cloned() {
                                        streams_to_shutdown.push(stream_arc);
                                    }
                                }
                            }
                        }

                        // Now take write locks and remove the sessions
                        let mut session_guard = match session_manager_clone.sessions.write() {
                            Ok(g) => g,
                            Err(poison) => {
                                eprintln!("Warning: sessions RwLock poisoned on write during heartbeat (remove) — recovering");
                                poison.into_inner()
                            }
                        };

                        let mut client_map_guard = match session_manager_clone.client_session_map.write() {
                            Ok(g) => g,
                            Err(poison) => {
                                eprintln!("Warning: client_session_map RwLock poisoned on write during heartbeat (remove) — recovering");
                                poison.into_inner()
                            }
                        };

                        for session_id in to_remove {
                            if let Some(session) = session_guard.remove(&session_id) {
                                client_map_guard.remove(&session.client_id);
                            }
                        }

                        // finally shutdown streams (we collected them earlier)
                        for stream_arc in streams_to_shutdown {
                            let mut stream_guard = stream_arc.lock().await;
                            let _ = stream_guard.shutdown().await;
                        }

        // 清理客户端连接
        let mut clients_guard = match clients.lock() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: clients mutex poisoned in client cleanup — recovering");
                poison.into_inner()
            }
        };
        clients_guard.remove(&client_id);
        
        // 清理会话
        if let Some(session) = session_manager.get_session_by_client_id(client_id) {
            session_manager.remove_session(&session.session_id);
        }
    }

    /// 处理消息
    async fn process_message(
        message: &NetworkMessage,
        client_id: u64,
        clients: &Arc<Mutex<HashMap<u64, Arc<tokio::sync::Mutex<TokioTcpStream>>>>>,
        delay_compensation: &Arc<Mutex<delay_compensation::ServerDelayCompensation>>,
        session_manager: &Arc<SessionManager>,
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    ) {
        match message {
            NetworkMessage::Connect { client_id: _, name } => {
                // 处理连接请求
<<<<<<< HEAD
                if let Ok(mut clients_guard) = clients.lock() {
                    if let Some(conn) = clients_guard.get_mut(&client_id) {
                        conn.state = ConnectionState::Connected;
                        conn.authenticated = true;
                        conn.name = Some(name.clone());
                        conn.update_heartbeat();
                    }
=======
                if let Some(mut session) = session_manager.get_session_by_client_id(client_id) {
                    session.state = ConnectionState::Connected;
                    session.authenticated = true;
                    session.name = Some(name.clone());
                    session.update_heartbeat();
                    
                    // 更新会话
                    session_manager.update_session(session);
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
                }
            }
            NetworkMessage::Disconnect { client_id: _ } => {
                // 处理断开连接
<<<<<<< HEAD
                clients.lock().unwrap().remove(&client_id);
            }
            NetworkMessage::Heartbeat { timestamp: _ } => {
                // 更新心跳
                if let Ok(mut clients_guard) = clients.lock() {
                    if let Some(conn) = clients_guard.get_mut(&client_id) {
                        conn.update_heartbeat();
                    }
=======
                if let Some(session) = session_manager.get_session_by_client_id(client_id) {
                    session_manager.remove_session(&session.session_id);
                    // 关闭连接
                    let clients_guard = match clients.lock() {
                        Ok(g) => g,
                        Err(poison) => {
                            eprintln!("Warning: clients mutex poisoned while disconnect handling — recovering");
                            poison.into_inner()
                        }
                    };
                    if let Some(stream) = clients_guard.get(&client_id) {
                        let mut stream_guard = stream.lock().await;
                        let _ = stream_guard.shutdown().await;
                    }
                }
            }
            NetworkMessage::Heartbeat { timestamp: _ } => {
                // 更新心跳
                if let Some(mut session) = session_manager.get_session_by_client_id(client_id) {
                    session.update_heartbeat();
                    session_manager.update_session(session);
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
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
<<<<<<< HEAD
                        let _ = stream.write_all(&data);
=======
                        // 发送响应
                        let clients_guard = match clients.lock() {
                            Ok(g) => g,
                            Err(poison) => {
                                eprintln!("Warning: clients mutex poisoned in TimeSyncRequest handling — recovering");
                                poison.into_inner()
                            }
                        };
                        if let Some(stream) = clients_guard.get(&client_id) {
                            let mut stream_guard = stream.lock().await;
                            let _ = stream_guard.write_all(&data).await;
                        }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
                    }
                }
            }
            _ => {
                // 其他消息类型的处理
            }
        }
    }

    /// 广播消息给所有客户端
<<<<<<< HEAD
    pub fn broadcast(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        let _clients_guard = self
=======
    pub async fn broadcast(&self, message: &NetworkMessage) -> Result<(), NetworkError> {
        let clients_guard = self
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
            .clients
            .lock()
            .map_err(|e| NetworkError::SendError(format!("Lock error: {}", e)))?;

<<<<<<< HEAD
        let _data = Self::serialize_message(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // NOTE: 实际实现中需要将消息发送到每个客户端的流
        // 这里简化处理，实际应该维护每个客户端的TcpStream
=======
        let data = Self::serialize_message(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // 发送给所有客户端
        for (_, stream) in clients_guard.iter() {
            let mut stream_guard = stream.lock().await;
            let _ = stream_guard.write_all(&data).await;
        }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)

        Ok(())
    }

    /// 发送消息给特定客户端
<<<<<<< HEAD
    pub fn send_to_client(
=======
    pub async fn send_to_client(
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
        &self,
        client_id: u64,
        message: &NetworkMessage,
    ) -> Result<(), NetworkError> {
        let clients_guard = self
            .clients
            .lock()
            .map_err(|e| NetworkError::SendError(format!("Lock error: {}", e)))?;

<<<<<<< HEAD
        if !clients_guard.contains_key(&client_id) {
            return Err(NetworkError::InvalidPeerId);
        }

        let _data = Self::serialize_message(message)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;

        // NOTE: 实际实现中需要将消息发送到客户端的流

        Ok(())
=======
        if let Some(stream) = clients_guard.get(&client_id) {
            let data = Self::serialize_message(message)
                .map_err(|e| NetworkError::SerializationError(e.to_string()))?;
            
            let mut stream_guard = stream.lock().await;
            stream_guard.write_all(&data).await
                .map_err(|e| NetworkError::SendError(format!("Send failed: {}", e)))?;
            
            Ok(())
        } else {
            Err(NetworkError::InvalidPeerId)
        }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    /// 获取客户端连接数
    pub fn client_count(&self) -> usize {
<<<<<<< HEAD
        self.clients.lock().unwrap().len()
=======
        self.session_manager.session_count()
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    /// 获取所有客户端ID
    pub fn get_client_ids(&self) -> Vec<u64> {
<<<<<<< HEAD
        self.clients.lock().unwrap().keys().copied().collect()
=======
        let map_guard = match self.session_manager.client_session_map.read() {
            Ok(g) => g,
            Err(poison) => {
                eprintln!("Warning: client_session_map RwLock poisoned while retrieving client IDs — recovering");
                poison.into_inner()
            }
        };

        map_guard.keys().copied().collect()
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    /// 更新服务器tick
    pub fn update_tick(&self) {
<<<<<<< HEAD
        *self.current_tick.lock().unwrap() += 1;
=======
        match self.current_tick.lock() {
            Ok(mut guard) => *guard += 1,
            Err(poison) => {
                let mut guard = poison.into_inner();
                *guard += 1;
                eprintln!("Warning: current_tick mutex poisoned while updating tick — recovered and incremented");
            }
        }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    /// 获取当前tick
    pub fn current_tick(&self) -> u64 {
<<<<<<< HEAD
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
=======
        match self.current_tick.lock() {
            Ok(guard) => *guard,
            Err(poison) => {
                eprintln!("Warning: current_tick mutex poisoned while reading tick — recovering");
                *poison.into_inner()
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
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
<<<<<<< HEAD
        assert_eq!(config.max_connections, 100);
=======
        assert_eq!(config.max_connections, 200);
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    #[test]
    fn test_client_connection() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let conn = ClientConnection::new(1, addr);
        assert_eq!(conn.client_id, 1);
        assert_eq!(conn.state, ConnectionState::Connecting);
<<<<<<< HEAD
=======
        assert!(conn.session_id.is_nil() == false);
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
    }

    #[test]
    fn test_server_creation() {
        let config = ServerConfig::default();
        let server = GameServer::new(config);
        assert_eq!(server.client_count(), 0);
    }
<<<<<<< HEAD
=======

    #[test]
    fn test_session_manager() {
        let session_manager = SessionManager::new();
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        let session = session_manager.create_session(1, addr);
        assert_eq!(session.client_id, 1);
        assert_eq!(session_manager.session_count(), 1);
        
        let retrieved = session_manager.get_session(&session.session_id).unwrap();
        assert_eq!(retrieved.client_id, 1);
        
        let retrieved_by_client = session_manager.get_session_by_client_id(1).unwrap();
        assert_eq!(retrieved_by_client.client_id, 1);
        
        session_manager.remove_session(&session.session_id);
        assert_eq!(session_manager.session_count(), 0);
    }

    #[test]
    fn test_tick_increment() {
        let config = ServerConfig::default();
        let server = GameServer::new(config);

        assert_eq!(server.current_tick(), 0);
        server.update_tick();
        assert_eq!(server.current_tick(), 1);
    }
>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
}
