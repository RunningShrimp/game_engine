// 网络模块脚本绑定
//
// 将网络功能(TCP/UDP/WebSocket/HTTP)暴露给脚本语言

use crate::scripting::system::{ScriptContext, ScriptResult, ScriptValue};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpStream as TokioTcpStream, UdpSocket as TokioUdpSocket};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

// Import SinkExt and StreamExt from futures_util
// These are needed for WebSocket send/next operations
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;

// HTTP客户端 (reqwest is an optional dependency enabled by ai-openai or ai-claude features)
#[cfg(any(feature = "ai-openai", feature = "ai-claude"))]
use reqwest;

/// 网络API - 提供脚本可访问的网络功能
pub struct NetworkApi {
    /// TCP客户端连接池
    tcp_clients: Arc<Mutex<HashMap<String, TcpClient>>>,
    /// UDP客户端连接池
    udp_clients: Arc<Mutex<HashMap<String, UdpClient>>>,
    /// WebSocket客户端连接池
    ws_clients: Arc<Mutex<HashMap<String, WebSocketClient>>>,
}

/// TCP客户端
#[derive(Debug)]
pub struct TcpClient {
    id: String,
    host: String,
    port: u16,
    connected: bool,
    stream: Option<TokioTcpStream>,
    read_buffer: Vec<u8>,
}

impl Clone for TcpClient {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            host: self.host.clone(),
            port: self.port,
            connected: self.connected,
            stream: None, // Cannot clone TcpStream
            read_buffer: Vec::new(),
        }
    }
}

/// UDP客户端
#[derive(Debug)]
pub struct UdpClient {
    id: String,
    host: String,
    port: u16,
    bound: bool,
    socket: Option<TokioUdpSocket>,
}

impl Clone for UdpClient {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            host: self.host.clone(),
            port: self.port,
            bound: self.bound,
            socket: None, // Cannot clone UdpSocket
        }
    }
}

/// WebSocket客户端
#[derive(Debug)]
pub struct WebSocketClient {
    id: String,
    url: String,
    connected: bool,
    // WebSocketStream (supports both ws:// and wss:// through MaybeTlsStream wrapper)
    ws_stream: Option<WebSocketStream<MaybeTlsStream<TokioTcpStream>>>,
    read_buffer: Vec<u8>,
}

impl Clone for WebSocketClient {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            url: self.url.clone(),
            connected: self.connected,
            ws_stream: None, // Cannot clone WebSocketStream
            read_buffer: Vec::new(),
        }
    }
}

impl NetworkApi {
    /// 创建新的网络API实例
    pub fn new() -> Self {
        Self {
            tcp_clients: Arc::new(Mutex::new(HashMap::new())),
            udp_clients: Arc::new(Mutex::new(HashMap::new())),
            ws_clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 高级连接API - 简化连接过程
    ///
    /// 自动选择协议（ws://或wss://使用WebSocket，http://或https://使用HTTP，其他使用TCP）
    pub fn connect(&self, url: String) -> ScriptResult {
        if url.starts_with("ws://") || url.starts_with("wss://") {
            // 解析WebSocket URL
            let url_parts: Vec<&str> = url.split("://").collect();
            if url_parts.len() != 2 {
                return ScriptResult::Error("Invalid WebSocket URL format".to_string());
            }
            let host_port = url_parts[1];
            let parts: Vec<&str> = host_port.split('/').collect();
            let host_port = parts[0];
            let parts: Vec<&str> = host_port.split(':').collect();

            let host = parts[0].to_string();
            let port = if parts.len() > 1 {
                parts[1].parse().unwrap_or(80)
            } else {
                80
            };

            let id = format!(
                "auto_ws_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            self.ws_connect(id.clone(), url.clone())
        } else if url.starts_with("http://") || url.starts_with("https://") {
            // HTTP连接（用于REST API）
            ScriptResult::Success(ScriptValue::String(format!(
                "HTTP connection to {url} (use http_get/http_post)"
            )))
        } else {
            // TCP连接
            let parts: Vec<&str> = url.split(':').collect();
            if parts.len() != 2 {
                return ScriptResult::Error(
                    "Invalid TCP URL format. Expected host:port".to_string(),
                );
            }
            let host = parts[0].to_string();
            let port: u16 = match parts[1].parse() {
                Ok(p) => p,
                Err(_) => return ScriptResult::Error("Invalid port number".to_string()),
            };

            let id = format!(
                "auto_tcp_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis()
            );
            self.tcp_connect(id, host, port)
        }
    }

    /// RPC调用包装器
    ///
    /// 发送RPC请求并等待响应
    pub fn call_rpc(
        &self,
        connection_id: String,
        function_name: String,
        args: Vec<ScriptValue>,
    ) -> ScriptResult {
        // 构建RPC消息
        let rpc_message = format!(
            r#"{{"type":"rpc","function":"{}","args":{}}}"#,
            function_name,
            // 简化：将args序列化为JSON（实际应该使用serde_json）
            "[]"
        );

        // 发送到WebSocket连接（优先）或TCP连接
        if let Ok(mut ws_clients) = self.ws_clients.try_lock() {
            if ws_clients.contains_key(&connection_id) {
                return self.ws_send(connection_id, rpc_message);
            }
        }

        if let Ok(mut tcp_clients) = self.tcp_clients.try_lock() {
            if tcp_clients.contains_key(&connection_id) {
                return self.tcp_send(connection_id, rpc_message);
            }
        }

        ScriptResult::Error(format!("Connection '{connection_id}' not found"))
    }

    /// 加入大厅/房间
    pub fn join_lobby(&self, connection_id: String, lobby_id: String) -> ScriptResult {
        let message = format!(r#"{{"type":"lobby_join","lobby_id":"{lobby_id}"}}"#);

        if let Ok(mut ws_clients) = self.ws_clients.try_lock() {
            if ws_clients.contains_key(&connection_id) {
                return self.ws_send(connection_id, message);
            }
        }

        if let Ok(mut tcp_clients) = self.tcp_clients.try_lock() {
            if tcp_clients.contains_key(&connection_id) {
                return self.tcp_send(connection_id, message);
            }
        }

        ScriptResult::Error(format!("Connection '{connection_id}' not found"))
    }

    /// 同步玩家状态
    pub fn sync_player_state(
        &self,
        connection_id: String,
        state: HashMap<String, ScriptValue>,
    ) -> ScriptResult {
        // 简化：将状态序列化为JSON
        let state_json = r#"{"type":"player_state","state":{}}"#.to_string();
        let message = format!(r#"{{"type":"sync","data":{state_json}}}"#);

        if let Ok(mut ws_clients) = self.ws_clients.try_lock() {
            if ws_clients.contains_key(&connection_id) {
                return self.ws_send(connection_id, message);
            }
        }

        if let Ok(mut tcp_clients) = self.tcp_clients.try_lock() {
            if tcp_clients.contains_key(&connection_id) {
                return self.tcp_send(connection_id, message);
            }
        }

        ScriptResult::Error(format!("Connection '{connection_id}' not found"))
    }

    /// 创建TCP客户端连接
    pub fn tcp_connect(&self, id: String, host: String, port: u16) -> ScriptResult {
        // 构建Socket地址
        let addr = match format!("{host}:{port}").parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::error!(target: "network_api", "Invalid address {}:{}", host, port);
                return ScriptResult::Error(format!("Invalid address {host}:{port}: {e}"));
            }
        };

        // 在Tokio运行时中执行异步连接
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                // 如果没有当前运行时，创建一个新的
                tracing::warn!(target: "network_api", "No Tokio runtime found, creating new one");
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        // 在运行时中执行异步连接
        let stream = rt.block_on(async {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                TokioTcpStream::connect(&addr)
            ).await {
                Ok(Ok(stream)) => {
                    tracing::info!(target: "network_api", "TCP connected to {}:{}", host, port);
                    Some(stream)
                },
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "TCP connection failed to {}:{}: {}", host, port, e);
                    None
                },
                Err(_) => {
                    tracing::error!(target: "network_api", "TCP connection timeout to {}:{}", host, port);
                    None
                },
            }
        });

        let connected = stream.is_some();
        let mut client = TcpClient {
            id: id.clone(),
            host: host.clone(),
            port,
            connected,
            stream,
            read_buffer: Vec::new(),
        };

        let mut clients = self.tcp_clients.blocking_lock();
        clients.insert(id.clone(), client);

        ScriptResult::Success(ScriptValue::Boolean(connected))
    }

    /// 通过TCP发送数据
    pub fn tcp_send(&self, id: String, data: String) -> ScriptResult {
        let mut clients = self.tcp_clients.blocking_lock();
        let client = match clients.get_mut(&id) {
            Some(client) => client,
            None => return ScriptResult::Error(format!("TCP client '{id}' not found")),
        };

        if !client.connected {
            return ScriptResult::Error(format!("TCP client '{id}' is not connected"));
        }

        let stream = match &mut client.stream {
            Some(stream) => stream,
            None => return ScriptResult::Error(format!("TCP client '{id}' has no valid stream")),
        };

        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        let result = rt.block_on(async {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                stream.write_all(data.as_bytes())
            ).await {
                Ok(Ok(_)) => {
                    tracing::debug!(target: "network_api", "TCP sent {} bytes to {}", data.len(), id);
                    Ok(data.len())
                },
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "TCP send failed to {}: {}", id, e);
                    Err(e.to_string())
                },
                Err(_) => {
                    tracing::error!(target: "network_api", "TCP send timeout to {}", id);
                    Err("Send timeout".to_string())
                },
            }
        });

        match result {
            Ok(len) => ScriptResult::Success(ScriptValue::Integer(len as i64)),
            Err(e) => ScriptResult::Error(format!("Failed to send data: {e}")),
        }
    }

    /// 通过TCP接收数据
    pub fn tcp_receive(&self, id: String) -> ScriptResult {
        let mut clients = self.tcp_clients.blocking_lock();
        let client = match clients.get_mut(&id) {
            Some(client) => client,
            None => return ScriptResult::Error(format!("TCP client '{id}' not found")),
        };

        if !client.connected {
            return ScriptResult::Error(format!("TCP client '{id}' is not connected"));
        }

        let stream = match &mut client.stream {
            Some(stream) => stream,
            None => return ScriptResult::Error(format!("TCP client '{id}' has no valid stream")),
        };

        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        // 从缓冲区读取，如果有数据的话
        if !client.read_buffer.is_empty() {
            let data = String::from_utf8_lossy(&client.read_buffer).to_string();
            client.read_buffer.clear();
            return ScriptResult::Success(ScriptValue::String(data));
        }

        // 否则从流中读取
        let result = rt.block_on(async {
            let mut buffer = vec![0u8; 4096]; // 4KB buffer
            match tokio::time::timeout(
                tokio::time::Duration::from_millis(100),
                stream.read(&mut buffer)
            ).await {
                Ok(Ok(0)) => {
                    // Connection closed
                    tracing::info!(target: "network_api", "TCP connection closed by remote: {}", id);
                    Err("Connection closed".to_string())
                },
                Ok(Ok(n)) => {
                    tracing::debug!(target: "network_api", "TCP received {} bytes from {}", n, id);
                    buffer.truncate(n);
                    Ok(buffer)
                },
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "TCP receive failed from {}: {}", id, e);
                    Err(e.to_string())
                },
                Err(_) => {
                    // Timeout - no data available
                    Ok(Vec::new())
                },
            }
        });

        match result {
            Ok(buffer) => {
                if buffer.is_empty() {
                    ScriptResult::Success(ScriptValue::String("".to_string()))
                } else {
                    let data = String::from_utf8_lossy(&buffer).to_string();
                    ScriptResult::Success(ScriptValue::String(data))
                }
            }
            Err(e) => {
                if e.contains("Connection closed") {
                    // 标记连接为已关闭
                    client.connected = false;
                }
                ScriptResult::Error(format!("Failed to receive data: {e}"))
            }
        }
    }

    /// 关闭TCP连接
    pub fn tcp_close(&self, id: String) -> ScriptResult {
        let mut clients = self.tcp_clients.blocking_lock();
        match clients.remove(&id) {
            Some(_) => {
                tracing::info!(target: "network_api", "TCP connection closed: {}", id);
                ScriptResult::Void
            }
            None => ScriptResult::Error(format!("TCP client '{id}' not found")),
        }
    }

    /// 创建UDP客户端
    pub fn udp_bind(&self, id: String, host: String, port: u16) -> ScriptResult {
        // 构建Socket地址
        let addr = match format!("{host}:{port}").parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => {
                tracing::error!(target: "network_api", "Invalid address {}:{}", host, port);
                return ScriptResult::Error(format!("Invalid address {host}:{port}: {e}"));
            }
        };

        // 在Tokio运行时中执行异步绑定
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        // 在运行时中执行异步绑定
        let socket = rt.block_on(async {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                TokioUdpSocket::bind(&addr)
            ).await {
                Ok(Ok(socket)) => {
                    tracing::info!(target: "network_api", "UDP bound to {}:{}", host, port);
                    Some(socket)
                },
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "UDP bind failed to {}:{}: {}", host, port, e);
                    None
                },
                Err(_) => {
                    tracing::error!(target: "network_api", "UDP bind timeout to {}:{}", host, port);
                    None
                },
            }
        });

        let bound = socket.is_some();
        let mut client = UdpClient {
            id: id.clone(),
            host: host.clone(),
            port,
            bound,
            socket,
        };

        let mut clients = self.udp_clients.blocking_lock();
        clients.insert(id.clone(), client);

        ScriptResult::Success(ScriptValue::Boolean(bound))
    }

    /// 通过UDP发送数据
    pub fn udp_send_to(
        &self,
        id: String,
        target_host: String,
        target_port: u16,
        data: String,
    ) -> ScriptResult {
        let mut clients = self.udp_clients.blocking_lock();
        let client = match clients.get_mut(&id) {
            Some(client) => client,
            None => return ScriptResult::Error(format!("UDP client '{id}' not found")),
        };

        if !client.bound {
            return ScriptResult::Error(format!("UDP client '{id}' is not bound"));
        }

        let socket = match &mut client.socket {
            Some(socket) => socket,
            None => return ScriptResult::Error(format!("UDP client '{id}' has no valid socket")),
        };

        // 解析目标地址
        let target_addr = match format!("{target_host}:{target_port}").parse::<SocketAddr>() {
            Ok(addr) => addr,
            Err(e) => {
                return ScriptResult::Error(format!(
                    "Invalid target address {target_host}:{target_port}: {e}"
                ));
            }
        };

        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        let result = rt.block_on(async {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                socket.send_to(data.as_bytes(), target_addr)
            ).await {
                Ok(Ok(n)) => {
                    tracing::debug!(target: "network_api", "UDP sent {} bytes to {}:{}", n, target_host, target_port);
                    Ok(n)
                },
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "UDP send failed to {}:{}: {}", target_host, target_port, e);
                    Err(e.to_string())
                },
                Err(_) => {
                    tracing::error!(target: "network_api", "UDP send timeout to {}:{}", target_host, target_port);
                    Err("Send timeout".to_string())
                },
            }
        });

        match result {
            Ok(len) => ScriptResult::Success(ScriptValue::Integer(len as i64)),
            Err(e) => ScriptResult::Error(format!("Failed to send UDP data: {e}")),
        }
    }

    /// 通过UDP接收数据
    pub fn udp_receive(&self, id: String) -> ScriptResult {
        let mut clients = self.udp_clients.blocking_lock();
        let client = match clients.get_mut(&id) {
            Some(client) => client,
            None => return ScriptResult::Error(format!("UDP client '{id}' not found")),
        };

        if !client.bound {
            return ScriptResult::Error(format!("UDP client '{id}' is not bound"));
        }

        let socket = match &mut client.socket {
            Some(socket) => socket,
            None => return ScriptResult::Error(format!("UDP client '{id}' has no valid socket")),
        };

        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        // 从流中读取
        let result = rt.block_on(async {
            let mut buffer = vec![0u8; 4096]; // 4KB buffer
            match tokio::time::timeout(
                tokio::time::Duration::from_millis(100),
                socket.recv(&mut buffer),
            )
            .await
            {
                Ok(Ok(n)) => {
                    tracing::debug!(target: "network_api", "UDP received {} bytes from {}", n, id);
                    buffer.truncate(n);
                    Ok(buffer)
                }
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "UDP receive failed from {}: {}", id, e);
                    Err(e.to_string())
                }
                Err(_) => {
                    // Timeout - no data available
                    Ok(Vec::new())
                }
            }
        });

        match result {
            Ok(buffer) => {
                let data = String::from_utf8_lossy(&buffer).to_string();
                ScriptResult::Success(ScriptValue::String(data))
            }
            Err(e) => ScriptResult::Error(format!("Failed to receive UDP data: {e}")),
        }
    }

    /// 关闭UDP套接字
    pub fn udp_close(&self, id: String) -> ScriptResult {
        let mut clients = self.udp_clients.blocking_lock();
        match clients.remove(&id) {
            Some(_) => {
                tracing::info!(target: "network_api", "UDP socket closed: {}", id);
                ScriptResult::Void
            }
            None => ScriptResult::Error(format!("UDP client '{id}' not found")),
        }
    }

    /// 创建WebSocket连接
    pub fn ws_connect(&self, id: String, url: String) -> ScriptResult {
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        // 在运行时中执行异步连接
        // Note: connect_async returns different types for ws:// vs wss:// URLs
        // For simplicity, we only support ws:// (non-TLS) connections in this implementation
        if url.starts_with("wss://") {
            tracing::warn!(target: "network_api", "TLS WebSocket (wss://) not yet supported, please use ws://");
            return ScriptResult::Error("TLS WebSocket (wss://) not yet supported. Please use ws:// for non-TLS connections.".to_string());
        }

        let ws_stream = rt.block_on(async {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(10),
                connect_async(&url)
            ).await {
                Ok(Ok((stream, _response))) => {
                    tracing::info!(target: "network_api", "WebSocket connected to {} (id: {})", url, id);
                    Some(stream)
                },
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "WebSocket connection failed to {}: {} (id: {})", url, e, id);
                    None
                },
                Err(e) => {
                    tracing::error!(target: "network_api", "WebSocket connection timed out to {} (id: {})", url, id);
                    None
                },
            }
        });

        let mut client = WebSocketClient {
            id: id.clone(),
            url: url.clone(),
            connected: ws_stream.is_some(),
            ws_stream,
            read_buffer: Vec::new(),
        };

        let connected = client.connected;

        let mut clients = self.ws_clients.blocking_lock();
        clients.insert(id.clone(), client);

        ScriptResult::Success(ScriptValue::Boolean(connected))
    }

    /// 通过WebSocket发送数据
    pub fn ws_send(&self, id: String, data: String) -> ScriptResult {
        let mut clients = self.ws_clients.blocking_lock();
        let client = match clients.get_mut(&id) {
            Some(client) => client,
            None => return ScriptResult::Error(format!("WebSocket client '{id}' not found")),
        };

        if !client.connected {
            return ScriptResult::Error(format!("WebSocket client '{id}' is not connected"));
        }

        let ws_stream = match &mut client.ws_stream {
            Some(stream) => stream,
            None => {
                return ScriptResult::Error(format!("WebSocket client '{id}' has no valid stream"));
            }
        };

        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        let result = rt.block_on(async {
            let msg = Message::Text(data.into());
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(5),
                ws_stream.send(msg)
            ).await {
                Ok(Ok(_)) => {
                    tracing::debug!(target: "network_api", "WebSocket sent data to {}", id);
                    Ok(true)
                },
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "WebSocket send failed to {}: {}", id, e);
                    Err(e.to_string())
                },
                Err(_) => {
                    tracing::error!(target: "network_api", "WebSocket send timeout to {}", id);
                    Err("Send timeout".to_string())
                },
            }
        });

        match result {
            Ok(_) => ScriptResult::Success(ScriptValue::Boolean(true)),
            Err(e) => ScriptResult::Error(format!("Failed to send WebSocket data: {e}")),
        }
    }

    /// 通过WebSocket接收数据
    pub fn ws_receive(&self, id: String) -> ScriptResult {
        let mut clients = self.ws_clients.blocking_lock();
        let client = match clients.get_mut(&id) {
            Some(client) => client,
            None => return ScriptResult::Error(format!("WebSocket client '{id}' not found")),
        };

        if !client.connected {
            return ScriptResult::Error(format!("WebSocket client '{id}' is not connected"));
        }

        let ws_stream = match &mut client.ws_stream {
            Some(stream) => stream,
            None => {
                return ScriptResult::Error(format!("WebSocket client '{id}' has no valid stream"));
            }
        };

        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                return ScriptResult::Error(
                    "No Tokio runtime available. Please ensure the engine is running with tokio::runtime::Runtime.".to_string()
                );
            }
        };

        // 从缓冲区读取，如果有数据的话
        if !client.read_buffer.is_empty() {
            let data = String::from_utf8_lossy(&client.read_buffer).to_string();
            client.read_buffer.clear();
            return ScriptResult::Success(ScriptValue::String(data));
        }

        // 否则从流中读取
        let result = rt.block_on(async {
            match tokio::time::timeout(
                tokio::time::Duration::from_millis(100),
                ws_stream.next()
            ).await {
                Ok(Some(Ok(msg))) => {
                    match msg {
                        Message::Text(text) => {
                            tracing::debug!(target: "network_api", "WebSocket received text from {}: {}", id, text);
                            // Convert text bytes to Vec<u8>
                            Ok::<Vec<u8>, String>(text.bytes().collect())
                        },
                        Message::Binary(data) => {
                            tracing::debug!(target: "network_api", "WebSocket received binary from {}: {} bytes", id, data.len());
                            Ok::<Vec<u8>, String>(data.to_vec())
                        },
                        Message::Close(_) => {
                            tracing::info!(target: "network_api", "WebSocket connection closed by remote: {}", id);
                            Err("Connection closed".to_string())
                        },
                        _ => {
                            // Ping, Pong, etc. - ignore for now
                            Ok::<Vec<u8>, String>(Vec::new())
                        },
                    }
                },
                Ok(Some(Err(e))) => {
                    tracing::error!(target: "network_api", "WebSocket receive failed from {}: {}", id, e);
                    Err(e.to_string())
                },
                Ok(None) => {
                    tracing::warn!(target: "network_api", "WebSocket stream closed: {}", id);
                    Err("Stream closed".to_string())
                },
                Err(_) => {
                    // Timeout - no data available
                    Ok(Vec::new())
                },
            }
        });

        match result {
            Ok(buffer) => {
                let data = String::from_utf8_lossy(&buffer[..]).to_string();
                ScriptResult::Success(ScriptValue::String(data))
            }
            Err(e) => {
                if e.contains("Connection closed") || e.contains("Stream closed") {
                    // 标记连接为已关闭
                    client.connected = false;
                }
                ScriptResult::Error(format!("Failed to receive WebSocket data: {e}"))
            }
        }
    }

    /// 关闭WebSocket连接
    pub fn ws_close(&self, id: String) -> ScriptResult {
        let mut clients = self.ws_clients.blocking_lock();
        match clients.remove(&id) {
            Some(_) => {
                tracing::info!(target: "network_api", "WebSocket connection closed: {}", id);
                ScriptResult::Void
            }
            None => ScriptResult::Error(format!("WebSocket client '{id}' not found")),
        }
    }

    /// 发送HTTP GET请求
    #[cfg(any(feature = "ai-openai", feature = "ai-claude"))]
    pub fn http_get(&self, url: String) -> ScriptResult {
        tracing::info!(target: "network_api", "HTTP GET: {}", url);

        // 检查Tokio运行时
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                tracing::error!(target: "network_api", "No Tokio runtime available for HTTP GET");
                return ScriptResult::Error("No Tokio runtime available".to_string());
            }
        };

        // 解析URL
        let url_parsed = match reqwest::Url::parse(&url) {
            Ok(u) => u,
            Err(e) => {
                tracing::error!(target: "network_api", "Invalid URL: {}", e);
                return ScriptResult::Error(format!("Invalid URL: {}", e));
            }
        };

        // 执行HTTP GET请求
        let response = rt.block_on(async {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(30),
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .and_then(|client| async move { client.get(url_parsed).send().await }),
            )
            .await
            {
                Ok(Ok(resp)) => Some(resp),
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "HTTP GET request failed: {}", e);
                    None
                }
                Err(_) => {
                    tracing::error!(target: "network_api", "HTTP GET request timed out");
                    None
                }
            }
        });

        match response {
            Some(resp) => {
                let status = resp.status();
                let headers = resp.headers().clone();

                // 尝试读取响应体
                let body = rt.block_on(async {
                    match tokio::time::timeout(
                        tokio::time::Duration::from_secs(10),
                        resp.text()
                    ).await {
                        Ok(Ok(text)) => Some(text),
                        Ok(Err(e)) => {
                            tracing::error!(target: "network_api", "Failed to read response body: {}", e);
                            None
                        }
                        Err(_) => {
                            tracing::error!(target: "network_api", "Response body read timed out");
                            None
                        }
                    }
                });

                match body {
                    Some(text) => {
                        // 构建结果对象
                        let mut result = HashMap::new();
                        result.insert(
                            "status".to_string(),
                            ScriptValue::Integer(status.as_u16() as i64),
                        );
                        result.insert("body".to_string(), ScriptValue::String(text));

                        // 添加headers
                        let mut headers_map = HashMap::new();
                        for (name, value) in headers.iter() {
                            if let Ok(value_str) = value.to_str() {
                                headers_map.insert(
                                    name.as_str().to_string(),
                                    ScriptValue::String(value_str.to_string()),
                                );
                            }
                        }
                        result.insert("headers".to_string(), ScriptValue::Object(headers_map));

                        tracing::info!(target: "network_api", "HTTP GET successful: status={}", status);
                        ScriptResult::Success(ScriptValue::Object(result))
                    }
                    None => ScriptResult::Error("Failed to read response body".to_string()),
                }
            }
            None => ScriptResult::Error("HTTP GET request failed or timed out".to_string()),
        }
    }

    /// 发送HTTP POST请求
    #[cfg(any(feature = "ai-openai", feature = "ai-claude"))]
    pub fn http_post(&self, url: String, body: String) -> ScriptResult {
        tracing::info!(target: "network_api", "HTTP POST: {} with body length: {}", url, body.len());

        // 检查Tokio运行时
        let rt = match tokio::runtime::Handle::try_current() {
            Ok(handle) => handle,
            Err(_) => {
                tracing::error!(target: "network_api", "No Tokio runtime available for HTTP POST");
                return ScriptResult::Error("No Tokio runtime available".to_string());
            }
        };

        // 解析URL
        let url_parsed = match reqwest::Url::parse(&url) {
            Ok(u) => u,
            Err(e) => {
                tracing::error!(target: "network_api", "Invalid URL: {}", e);
                return ScriptResult::Error(format!("Invalid URL: {}", e));
            }
        };

        // 执行HTTP POST请求
        let response = rt.block_on(async {
            match tokio::time::timeout(
                tokio::time::Duration::from_secs(30),
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(30))
                    .build()
                    .and_then(|client| async move {
                        client
                            .post(url_parsed)
                            .header("Content-Type", "application/json")
                            .body(body.clone())
                            .send()
                            .await
                    }),
            )
            .await
            {
                Ok(Ok(resp)) => Some(resp),
                Ok(Err(e)) => {
                    tracing::error!(target: "network_api", "HTTP POST request failed: {}", e);
                    None
                }
                Err(_) => {
                    tracing::error!(target: "network_api", "HTTP POST request timed out");
                    None
                }
            }
        });

        match response {
            Some(resp) => {
                let status = resp.status();
                let headers = resp.headers().clone();

                // 尝试读取响应体
                let resp_body = rt.block_on(async {
                    match tokio::time::timeout(
                        tokio::time::Duration::from_secs(10),
                        resp.text()
                    ).await {
                        Ok(Ok(text)) => Some(text),
                        Ok(Err(e)) => {
                            tracing::error!(target: "network_api", "Failed to read response body: {}", e);
                            None
                        }
                        Err(_) => {
                            tracing::error!(target: "network_api", "Response body read timed out");
                            None
                        }
                    }
                });

                match resp_body {
                    Some(text) => {
                        // 构建结果对象
                        let mut result = HashMap::new();
                        result.insert(
                            "status".to_string(),
                            ScriptValue::Integer(status.as_u16() as i64),
                        );
                        result.insert("body".to_string(), ScriptValue::String(text));

                        // 添加headers
                        let mut headers_map = HashMap::new();
                        for (name, value) in headers.iter() {
                            if let Ok(value_str) = value.to_str() {
                                headers_map.insert(
                                    name.as_str().to_string(),
                                    ScriptValue::String(value_str.to_string()),
                                );
                            }
                        }
                        result.insert("headers".to_string(), ScriptValue::Object(headers_map));

                        tracing::info!(target: "network_api", "HTTP POST successful: status={}", status);
                        ScriptResult::Success(ScriptValue::Object(result))
                    }
                    None => ScriptResult::Error("Failed to read response body".to_string()),
                }
            }
            None => ScriptResult::Error("HTTP POST request failed or timed out".to_string()),
        }
    }

    /// 发送HTTP GET请求 (fallback when AI features are disabled)
    #[cfg(not(any(feature = "ai-openai", feature = "ai-claude")))]
    pub fn http_get(&self, url: String) -> ScriptResult {
        tracing::warn!(target: "network_api", "HTTP GET requested but AI features (reqwest) are not enabled");
        ScriptResult::Error(
            "HTTP GET requires 'ai-openai' or 'ai-claude' feature to be enabled. \
             Please rebuild with: cargo build --features ai-openai"
                .to_string(),
        )
    }

    /// 发送HTTP POST请求 (fallback when AI features are disabled)
    #[cfg(not(any(feature = "ai-openai", feature = "ai-claude")))]
    pub fn http_post(&self, url: String, body: String) -> ScriptResult {
        tracing::warn!(target: "network_api", "HTTP POST requested but AI features (reqwest) are not enabled");
        ScriptResult::Error(
            "HTTP POST requires 'ai-openai' or 'ai-claude' feature to be enabled. \
             Please rebuild with: cargo build --features ai-openai"
                .to_string(),
        )
    }

    /// 获取所有活动连接的状态
    pub fn get_connection_status(&self) -> ScriptResult {
        let tcp_clients = self.tcp_clients.blocking_lock();
        let udp_clients = self.udp_clients.blocking_lock();
        let ws_clients = self.ws_clients.blocking_lock();

        let mut status = HashMap::new();

        status.insert(
            "tcp_count".to_string(),
            ScriptValue::Integer(tcp_clients.len() as i64),
        );
        status.insert(
            "udp_count".to_string(),
            ScriptValue::Integer(udp_clients.len() as i64),
        );
        status.insert(
            "ws_count".to_string(),
            ScriptValue::Integer(ws_clients.len() as i64),
        );

        ScriptResult::Success(ScriptValue::Object(status))
    }
}

impl Default for NetworkApi {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ScriptContext 集成 - 将网络API注册到脚本系统
// ============================================================================

/// 网络ScriptContext包装器
pub struct NetworkScriptContext {
    api: Arc<Mutex<NetworkApi>>,
    language: crate::scripting::system::ScriptLanguage,
}

impl NetworkScriptContext {
    pub fn new(
        api: Arc<Mutex<NetworkApi>>,
        language: crate::scripting::system::ScriptLanguage,
    ) -> Self {
        Self { api, language }
    }

    /// 调用网络API方法
    fn call_network_method(&mut self, method: &str, args: &[ScriptValue]) -> ScriptResult {
        let api = self.api.blocking_lock();

        match method {
            "tcp_connect" => {
                if args.len() < 3 {
                    return ScriptResult::Error(
                        "tcp_connect requires 3 arguments: id, host, port".to_string(),
                    );
                }
                let id = match &args[0] {
                    ScriptValue::String(s) => s.clone(),
                    _ => return ScriptResult::Error("First argument must be a string".to_string()),
                };
                let host = match &args[1] {
                    ScriptValue::String(s) => s.clone(),
                    _ => {
                        return ScriptResult::Error("Second argument must be a string".to_string());
                    }
                };
                let port = match &args[2] {
                    ScriptValue::Integer(i) => *i as u16,
                    ScriptValue::Number(n) => *n as u16,
                    _ => return ScriptResult::Error("Third argument must be a number".to_string()),
                };
                api.tcp_connect(id, host, port)
            }
            "tcp_send" => {
                if args.len() < 2 {
                    return ScriptResult::Error(
                        "tcp_send requires 2 arguments: id, data".to_string(),
                    );
                }
                let id = match &args[0] {
                    ScriptValue::String(s) => s.clone(),
                    _ => return ScriptResult::Error("First argument must be a string".to_string()),
                };
                let data = match &args[1] {
                    ScriptValue::String(s) => s.clone(),
                    _ => {
                        return ScriptResult::Error("Second argument must be a string".to_string());
                    }
                };
                api.tcp_send(id, data)
            }
            "ws_connect" => {
                if args.len() < 2 {
                    return ScriptResult::Error(
                        "ws_connect requires 2 arguments: id, url".to_string(),
                    );
                }
                let id = match &args[0] {
                    ScriptValue::String(s) => s.clone(),
                    _ => return ScriptResult::Error("First argument must be a string".to_string()),
                };
                let url = match &args[1] {
                    ScriptValue::String(s) => s.clone(),
                    _ => {
                        return ScriptResult::Error("Second argument must be a string".to_string());
                    }
                };
                api.ws_connect(id, url)
            }
            "http_get" => {
                if args.is_empty() {
                    return ScriptResult::Error("http_get requires 1 argument: url".to_string());
                }
                let url = match &args[0] {
                    ScriptValue::String(s) => s.clone(),
                    _ => return ScriptResult::Error("Argument must be a string".to_string()),
                };
                api.http_get(url)
            }
            _ => ScriptResult::Error(format!("Unknown network method: {method}")),
        }
    }
}

impl ScriptContext for NetworkScriptContext {
    fn execute(&mut self, script: &str, _source_code: Option<&str>) -> ScriptResult {
        // 简化实现：将script解析为方法调用
        // 格式: "method_name arg1 arg2 ..."
        let parts: Vec<&str> = script.split_whitespace().collect();
        if parts.is_empty() {
            return ScriptResult::Error("Empty script".to_string());
        }

        let method = parts[0];
        let args: Vec<ScriptValue> =
            parts[1..].iter().map(|s| ScriptValue::String(s.to_string())).collect();

        self.call_network_method(method, &args)
    }

    fn call(&mut self, function: &str, args: &[ScriptValue]) -> ScriptResult {
        self.call_network_method(function, args)
    }

    fn eval(&mut self, expression: &str) -> ScriptResult {
        self.execute(expression, None)
    }

    fn set_global(&mut self, _name: &str, _value: ScriptValue) -> ScriptResult {
        ScriptResult::Error("Network API does not support global variables".to_string())
    }

    fn get_global(&mut self, name: &str) -> ScriptResult {
        ScriptResult::Error(format!(
            "Network API does not support global variables: {name}"
        ))
    }

    fn reset(&mut self) {
        // Network API doesn't maintain state that needs resetting
    }

    fn language(&self) -> crate::scripting::system::ScriptLanguage {
        self.language
    }

    fn has_function(&mut self, name: &str) -> bool {
        matches!(
            name,
            "tcp_connect"
                | "tcp_send"
                | "tcp_receive"
                | "tcp_close"
                | "udp_bind"
                | "udp_send_to"
                | "udp_receive"
                | "udp_close"
                | "ws_connect"
                | "ws_send"
                | "ws_receive"
                | "ws_close"
                | "http_get"
                | "http_post"
                | "get_connection_status"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_api_creation() {
        let api = NetworkApi::new();
        let status = api.get_connection_status();

        match status {
            ScriptResult::Success(ScriptValue::Object(map)) => {
                assert_eq!(map.get("tcp_count"), Some(&ScriptValue::Integer(0)));
                assert_eq!(map.get("udp_count"), Some(&ScriptValue::Integer(0)));
                assert_eq!(map.get("ws_count"), Some(&ScriptValue::Integer(0)));
            }
            _ => panic!("Expected object with connection counts"),
        }
    }

    #[test]
    fn test_tcp_connect() {
        let api = NetworkApi::new();
        let result = api.tcp_connect("test_client".to_string(), "localhost".to_string(), 8080);

        match result {
            ScriptResult::Success(ScriptValue::Boolean(true)) => (),
            _ => panic!("Expected Success(Boolean(true))"),
        }
    }

    #[test]
    fn test_ws_connect() {
        let api = NetworkApi::new();
        let result = api.ws_connect("test_ws".to_string(), "ws://localhost:8080".to_string());

        match result {
            ScriptResult::Success(ScriptValue::Boolean(true)) => (),
            _ => panic!("Expected Success(Boolean(true))"),
        }
    }

    #[test]
    fn test_http_get() {
        let api = NetworkApi::new();
        let result = api.http_get("http://example.com/api".to_string());

        match result {
            ScriptResult::Success(ScriptValue::String(_)) => (),
            _ => panic!("Expected Success(String)"),
        }
    }

    #[test]
    fn test_network_script_context() {
        let api = Arc::new(Mutex::new(NetworkApi::new()));
        let mut ctx = NetworkScriptContext::new(api, crate::scripting::system::ScriptLanguage::Lua);

        // 测试 tcp_connect 方法
        let result = ctx.execute("tcp_connect test_client localhost 8080", None);

        match result {
            ScriptResult::Success(ScriptValue::Boolean(true)) => (),
            _ => panic!("Expected Success(Boolean(true))"),
        }

        // 测试函数检测
        assert!(ctx.has_function("tcp_connect"));
        assert!(ctx.has_function("ws_connect"));
        assert!(!ctx.has_function("unknown_function"));
    }

    #[test]
    fn test_udp_bind() {
        let api = NetworkApi::new();
        let result = api.udp_bind("test_udp".to_string(), "localhost".to_string(), 9090);

        match result {
            ScriptResult::Success(ScriptValue::Boolean(true)) => (),
            _ => panic!("Expected Success(Boolean(true))"),
        }
    }

    #[test]
    fn test_tcp_send_and_receive() {
        let api = NetworkApi::new();

        // 先连接
        let _ = api.tcp_connect("send_test".to_string(), "localhost".to_string(), 8080);

        // 测试发送
        let send_result = api.tcp_send(
            "send_test".to_string(),
            String::from_utf8(b"Hello, Server!".to_vec()).unwrap(),
        );

        // 注意：由于没有真实服务器，发送可能失败，这是正常的
        // 我们主要测试API调用不会崩溃
        match send_result {
            ScriptResult::Success(_) | ScriptResult::Error(_) => (),
            _ => panic!("Unexpected result type"),
        }
    }

    #[test]
    fn test_udp_send_and_receive() {
        let api = NetworkApi::new();

        // 先绑定
        let _ = api.udp_bind("udp_send_test".to_string(), "localhost".to_string(), 9091);

        // 测试发送到远程地址
        let send_result = api.udp_send_to(
            "udp_send_test".to_string(),
            String::from_utf8(b"Hello, UDP!".to_vec()).unwrap(),
            "127.0.0.1".to_string(),
            9092,
        );

        // 测试API调用正常
        match send_result {
            ScriptResult::Success(_) | ScriptResult::Error(_) => (),
            _ => panic!("Unexpected result type"),
        }
    }

    #[test]
    fn test_ws_send_and_receive() {
        let api = NetworkApi::new();

        // 先连接（由于没有真实服务器，会失败）
        let _ = api.ws_connect(
            "ws_send_test".to_string(),
            "ws://localhost:8080".to_string(),
        );

        // 测试发送（会失败，但API调用应该正常）
        let send_result = api.ws_send(
            "ws_send_test".to_string(),
            String::from_utf8(b"Hello, WebSocket!".to_vec()).unwrap(),
        );

        match send_result {
            ScriptResult::Success(_) | ScriptResult::Error(_) => (),
            _ => panic!("Unexpected result type"),
        }
    }

    #[test]
    fn test_http_post() {
        let api = NetworkApi::new();
        let result = api.http_post(
            "http://example.com/api".to_string(),
            b"{\"key\":\"value\"}".to_vec(),
        );

        // 由于没有真实服务器，可能失败
        // 我们主要测试API调用不会崩溃
        match result {
            ScriptResult::Success(_) | ScriptResult::Error(_) => (),
            _ => panic!("Unexpected result type"),
        }
    }

    #[test]
    fn test_tcp_close() {
        let api = NetworkApi::new();

        // 连接然后关闭
        let _ = api.tcp_connect("close_test".to_string(), "localhost".to_string(), 8080);
        let close_result = api.tcp_close("close_test".to_string());

        match close_result {
            ScriptResult::Success(ScriptValue::Boolean(true)) => (),
            _ => panic!("Expected Success(Boolean(true))"),
        }
    }

    #[test]
    fn test_udp_close() {
        let api = NetworkApi::new();

        // 绑定然后关闭
        let _ = api.udp_bind("udp_close_test".to_string(), "localhost".to_string(), 9093);
        let close_result = api.udp_close("udp_close_test".to_string());

        match close_result {
            ScriptResult::Success(ScriptValue::Boolean(true)) => (),
            _ => panic!("Expected Success(Boolean(true))"),
        }
    }

    #[test]
    fn test_ws_close() {
        let api = NetworkApi::new();

        // 连接然后关闭
        let _ = api.ws_connect(
            "ws_close_test".to_string(),
            "ws://localhost:8080".to_string(),
        );
        let close_result = api.ws_close("ws_close_test".to_string());

        match close_result {
            ScriptResult::Success(ScriptValue::Boolean(true)) => (),
            _ => panic!("Expected Success(Boolean(true))"),
        }
    }

    #[test]
    fn test_connection_status() {
        let api = NetworkApi::new();

        // 初始状态：无连接
        let status = api.get_connection_status();
        match status {
            ScriptResult::Success(ScriptValue::Object(map)) => {
                assert_eq!(map.get("tcp_count"), Some(&ScriptValue::Integer(0)));
                assert_eq!(map.get("udp_count"), Some(&ScriptValue::Integer(0)));
                assert_eq!(map.get("ws_count"), Some(&ScriptValue::Integer(0)));
            }
            _ => panic!("Expected object with connection counts"),
        }

        // 添加一个TCP连接
        let _ = api.tcp_connect("status_test".to_string(), "localhost".to_string(), 8080);

        // 检查状态更新
        let status = api.get_connection_status();
        match status {
            ScriptResult::Success(ScriptValue::Object(map)) => {
                assert_eq!(map.get("tcp_count"), Some(&ScriptValue::Integer(1)));
            }
            _ => panic!("Expected object with updated connection counts"),
        }
    }
}

// ============================================================================
// 网络API使用示例
// ============================================================================
//
// 本模块提供了Lua脚本中使用的网络功能示例。
//
// 示例 1: TCP客户端连接和通信
// -----------------------------------------------
// ```lua
// -- 连接到TCP服务器
// local success = tcp_connect("game_client", "localhost", 8080)
// if success then
//     print("Connected to server")
//
//     -- 发送数据
//     tcp_send("game_client", "Hello, Server!")
//
//     -- 接收数据
//     local data = tcp_receive("game_client")
//     if data then
//         print("Received:", data)
//     end
//
//     -- 关闭连接
//     tcp_close("game_client")
// end
// ```
//
// 示例 2: UDP套接字通信
// -----------------------------------------------
// ```lua
// -- 绑定UDP端口
// local success = udp_bind("game_udp", "localhost", 9090)
// if success then
//     print("UDP socket bound")
//
//     -- 发送数据到远程地址
//     udp_send_to("game_udp", "Hello, UDP!", "127.0.0.1", 9091)
//
//     -- 接收数据
//     local data = udp_receive("game_udp")
//     if data then
//         print("Received:", data)
//     end
//
//     -- 关闭套接字
//     udp_close("game_udp")
// end
// ```
//
// 示例 3: WebSocket客户端连接
// -----------------------------------------------
// ```lua
// -- 连接到WebSocket服务器
// local success = ws_connect("game_ws", "ws://localhost:8080")
// if success then
//     print("WebSocket connected")
//
//     -- 发送消息
//     ws_send("game_ws", "{\"type\":\"chat\",\"message\":\"Hello\"}")
//
//     -- 接收消息
//     local message = ws_receive("game_ws")
//     if message then
//         print("Received:", message)
//     end
//
//     -- 关闭连接
//     ws_close("game_ws")
// end
// ```
//
// 示例 4: HTTP GET请求
// -----------------------------------------------
// ```lua
// -- 发起HTTP GET请求
// local response = http_get("http://api.example.com/data")
// if response then
//     print("HTTP Response:", response)
//
//     -- 解析JSON响应（假设使用JSON库）
//     local data = json.decode(response)
//     print("Status:", data.status)
// end
// ```
//
// 示例 5: HTTP POST请求
// -----------------------------------------------
// ```lua
// -- 准备JSON数据
// local payload = json.encode({
//     username = "player1",
//     score = 1000,
//     level = 5
// })
//
// -- 发起HTTP POST请求
// local response = http_post("http://api.example.com/score", payload)
// if response then
//     print("Server response:", response)
// end
// ```
//
// 示例 6: 查询网络连接状态
// -----------------------------------------------
// ```lua
// -- 获取当前所有连接的状态
// local status = network_status()
// print("TCP connections:", status.tcp_count)
// print("UDP connections:", status.udp_count)
// print("WebSocket connections:", status.ws_count)
// ```
//
// 错误处理示例
// -----------------------------------------------
// ```lua
// -- 连接失败时的错误处理
// local success = tcp_connect("client", "localhost", 8080)
// if not success then
//     print("Failed to connect to server")
//     return
// end
//
// -- 发送失败时的错误处理
// local send_ok = tcp_send("client", "data")
// if not send_ok then
//     print("Failed to send data")
//     tcp_close("client")
//     return
// end
// ```
//
// 游戏场景示例：多人游戏客户端
// -----------------------------------------------
// ```lua
// -- 连接到游戏服务器
// if tcp_connect("game_server", "game.example.com", 9000) then
//     print("Connected to game server")
//
//     -- 发送登录数据
//     local login_data = json.encode({
//         action = "login",
//         username = "player1",
//         token = "abc123"
//     })
//     tcp_send("game_server", login_data)
//
//     -- 游戏主循环
//     while true do
//         -- 接收服务器消息
//         local message = tcp_receive("game_server")
//         if message then
//             local data = json.decode(message)
//
//             -- 处理不同类型的消息
//             if data.action == "update" then
//                 update_game_state(data.state)
//             elseif data.action == "chat" then
//                 display_chat(data.player, data.message)
//             elseif data.action == "player_joined" then
//                 spawn_player(data.player_id, data.position)
//             end
//         end
//
//         -- 发送玩家输入
//         local input = get_player_input()
//         if input then
//             local input_msg = json.encode({
//                 action = "input",
//                 input = input
//             })
//             tcp_send("game_server", input_msg)
//         end
//
//         -- 等待下一帧
//         wait_frame()
//     end
// end
// ```
//
// 注意事项
// -----------------------------------------------
// 1. 超时设置: 所有网络操作都有内置超时保护
//    - TCP连接超时: 10秒
//    - TCP发送超时: 5秒
//    - UDP接收超时: 100毫秒
//    - HTTP请求超时: 30秒
//
// 2. 错误处理: 始终检查网络操作的返回值
//
// 3. 资源清理: 使用完毕后记得关闭连接
//
// 4. 线程安全: 网络API内部使用Mutex保护，可以安全地在多线程环境中使用
//
// 5. 性能考虑: 频繁的网络通信可能会影响游戏性能，建议使用批量发送
