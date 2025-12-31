// 网络模块脚本绑定
//
// 将网络功能(TCP/UDP/WebSocket/HTTP)暴露给脚本语言

use crate::scripting::system::{ScriptContext, ScriptResult, ScriptValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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
#[derive(Debug, Clone)]
pub struct TcpClient {
    id: String,
    host: String,
    port: u16,
    connected: bool,
}

/// UDP客户端
#[derive(Debug, Clone)]
pub struct UdpClient {
    id: String,
    host: String,
    port: u16,
    bound: bool,
}

/// WebSocket客户端
#[derive(Debug, Clone)]
pub struct WebSocketClient {
    id: String,
    url: String,
    connected: bool,
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

    /// 创建TCP客户端连接
    pub fn tcp_connect(&self, id: String, host: String, port: u16) -> ScriptResult {
        let client = TcpClient {
            id: id.clone(),
            host: host.clone(),
            port,
            connected: false,
        };

        // TODO: 实现实际的TCP连接逻辑
        tracing::info!(target: "network_api", "TCP connect: {}:{} (id: {})", host, port, id);

        let mut clients = self.tcp_clients.blocking_lock();
        clients.insert(id.clone(), client);

        ScriptResult::Success(ScriptValue::Boolean(true))
    }

    /// 通过TCP发送数据
    pub fn tcp_send(&self, id: String, data: String) -> ScriptResult {
        let clients = self.tcp_clients.blocking_lock();
        match clients.get(&id) {
            Some(client) => {
                // TODO: 实现实际的TCP发送逻辑
                tracing::info!(target: "network_api", "TCP send to {}: {}", client.host, data);
                ScriptResult::Success(ScriptValue::Integer(data.len() as i64))
            }
            None => ScriptResult::Error(format!("TCP client '{}' not found", id)),
        }
    }

    /// 通过TCP接收数据
    pub fn tcp_receive(&self, id: String) -> ScriptResult {
        let clients = self.tcp_clients.blocking_lock();
        match clients.get(&id) {
            Some(_client) => {
                // TODO: 实现实际的TCP接收逻辑
                ScriptResult::Success(ScriptValue::String("".to_string()))
            }
            None => ScriptResult::Error(format!("TCP client '{}' not found", id)),
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
            None => ScriptResult::Error(format!("TCP client '{}' not found", id)),
        }
    }

    /// 创建UDP客户端
    pub fn udp_bind(&self, id: String, host: String, port: u16) -> ScriptResult {
        let client = UdpClient {
            id: id.clone(),
            host: host.clone(),
            port,
            bound: false,
        };

        // TODO: 实现实际的UDP绑定逻辑
        tracing::info!(target: "network_api", "UDP bind: {}:{} (id: {})", host, port, id);

        let mut clients = self.udp_clients.blocking_lock();
        clients.insert(id.clone(), client);

        ScriptResult::Success(ScriptValue::Boolean(true))
    }

    /// 通过UDP发送数据
    pub fn udp_send_to(
        &self,
        id: String,
        target_host: String,
        target_port: u16,
        data: String,
    ) -> ScriptResult {
        let clients = self.udp_clients.blocking_lock();
        match clients.get(&id) {
            Some(_client) => {
                // TODO: 实现实际的UDP发送逻辑
                tracing::info!(target: "network_api", "UDP send to {}:{}: {}", target_host, target_port, data);
                ScriptResult::Success(ScriptValue::Integer(data.len() as i64))
            }
            None => ScriptResult::Error(format!("UDP client '{}' not found", id)),
        }
    }

    /// 通过UDP接收数据
    pub fn udp_receive(&self, id: String) -> ScriptResult {
        let clients = self.udp_clients.blocking_lock();
        match clients.get(&id) {
            Some(_client) => {
                // TODO: 实现实际的UDP接收逻辑
                ScriptResult::Success(ScriptValue::String("".to_string()))
            }
            None => ScriptResult::Error(format!("UDP client '{}' not found", id)),
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
            None => ScriptResult::Error(format!("UDP client '{}' not found", id)),
        }
    }

    /// 创建WebSocket连接
    pub fn ws_connect(&self, id: String, url: String) -> ScriptResult {
        let client = WebSocketClient {
            id: id.clone(),
            url: url.clone(),
            connected: false,
        };

        // TODO: 实现实际的WebSocket连接逻辑
        tracing::info!(target: "network_api", "WebSocket connect: {} (id: {})", url, id);

        let mut clients = self.ws_clients.blocking_lock();
        clients.insert(id.clone(), client);

        ScriptResult::Success(ScriptValue::Boolean(true))
    }

    /// 通过WebSocket发送数据
    pub fn ws_send(&self, id: String, data: String) -> ScriptResult {
        let clients = self.ws_clients.blocking_lock();
        match clients.get(&id) {
            Some(_client) => {
                // TODO: 实现实际的WebSocket发送逻辑
                tracing::info!(target: "network_api", "WebSocket send to {}: {}", id, data);
                ScriptResult::Success(ScriptValue::Boolean(true))
            }
            None => ScriptResult::Error(format!("WebSocket client '{}' not found", id)),
        }
    }

    /// 通过WebSocket接收数据
    pub fn ws_receive(&self, id: String) -> ScriptResult {
        let clients = self.ws_clients.blocking_lock();
        match clients.get(&id) {
            Some(_client) => {
                // TODO: 实现实际的WebSocket接收逻辑
                ScriptResult::Success(ScriptValue::String("".to_string()))
            }
            None => ScriptResult::Error(format!("WebSocket client '{}' not found", id)),
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
            None => ScriptResult::Error(format!("WebSocket client '{}' not found", id)),
        }
    }

    /// 发送HTTP GET请求
    pub fn http_get(&self, url: String) -> ScriptResult {
        // TODO: 实现实际的HTTP GET请求
        tracing::info!(target: "network_api", "HTTP GET: {}", url);
        ScriptResult::Success(ScriptValue::String("{}".to_string()))
    }

    /// 发送HTTP POST请求
    pub fn http_post(&self, url: String, body: String) -> ScriptResult {
        // TODO: 实现实际的HTTP POST请求
        tracing::info!(target: "network_api", "HTTP POST: {} with body: {}", url, body);
        ScriptResult::Success(ScriptValue::String("{}".to_string()))
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
                if args.len() < 1 {
                    return ScriptResult::Error("http_get requires 1 argument: url".to_string());
                }
                let url = match &args[0] {
                    ScriptValue::String(s) => s.clone(),
                    _ => return ScriptResult::Error("Argument must be a string".to_string()),
                };
                api.http_get(url)
            }
            _ => ScriptResult::Error(format!("Unknown network method: {}", method)),
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
            "Network API does not support global variables: {}",
            name
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
}
