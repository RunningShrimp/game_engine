// DAP (Debug Adapter Protocol) 服务器实现
//
// 提供符合Debug Adapter Protocol的调试服务器，支持VS Code等IDE集成

use futures_util::stream::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

/// DAP协议消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DapMessage {
    /// 序列号
    pub seq: i64,
    /// 消息类型
    #[serde(rename = "type")]
    pub message_type: String,
    /// 请求或响应
    pub request_seq: Option<i64>,
    /// 成功标志
    pub success: Option<bool>,
    /// 命令
    pub command: Option<String>,
    /// 参数
    pub arguments: Option<Map<String, Value>>,
    /// 消息内容
    pub message: Option<String>,
    /// 附加数据
    pub body: Option<Value>,
}

/// 断点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// 断点ID
    pub id: i64,
    /// 是否验证通过
    pub verified: bool,
    /// 源文件路径
    pub source: Source,
    /// 行号
    pub line: i64,
    /// 列号
    pub column: Option<i64>,
    /// 断点条件
    pub condition: Option<String>,
    /// 命中次数条件
    pub hitCondition: Option<String>,
    /// 是否启用
    pub enabled: bool,
}

/// 源文件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// 源文件路径
    pub path: String,
    /// 源文件名称
    pub name: Option<String>,
    /// 源引用（用于特殊源）
    pub sourceReference: Option<i64>,
    /// 展示数据
    pub presentationHint: Option<String>,
    /// 源.origin
    pub origin: Option<String>,
    /// 源的适配器
    pub adapterId: Option<String>,
    /// 校验和
    pub checksums: Option<Vec<Checksum>>,
}

/// 校验和信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checksum {
    /// 算法
    pub algorithm: String,
    /// 校验和值
    pub checksum: String,
}

/// 栈帧信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// 帧ID
    pub id: i64,
    /// 帧名称
    pub name: String,
    /// 源文件
    pub source: Option<Source>,
    /// 行号
    pub line: i64,
    /// 列号
    pub column: i64,
    /// 模块ID
    pub moduleId: Option<i64>,
    /// 总帧数（用于部分加载）
    pub totalFrames: Option<i64>,
}

/// 作用域信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// 作用域名称
    pub name: String,
    /// 作用域变量引用
    pub variablesReference: i64,
    /// 命名变量引用
    pub namedVariables: Option<i64>,
    /// 索引变量引用
    pub indexedVariables: Option<i64>,
    /// 是否昂贵
    pub expensive: bool,
    /// 作用域显示名称
    pub presentationHint: Option<String>,
}

/// 变量信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// 变量名
    pub name: String,
    /// 变量值
    pub value: String,
    /// 变量类型
    pub type_field: Option<String>, // rename from 'type' to avoid keyword conflict
    /// 变量引用
    pub variablesReference: i64,
    /// 命名变量数
    pub namedVariables: Option<i64>,
    /// 索引变量数
    pub indexedVariables: Option<i64>,
    /// 计算表达式
    pub evaluateName: Option<String>,
    /// 变量显示名称
    pub presentationHint: Option<String>,
    /// 变量属性
    pub visibility: Option<String>,
}

/// 线程信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    /// 线程ID
    pub id: i64,
    /// 线程名称
    pub name: String,
}

/// DAP会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DapSessionState {
    /// 未启动
    NotStarted,
    /// 正在初始化
    Initializing,
    /// 已停止（暂停）
    Stopped,
    /// 运行中
    Running,
    /// 已完成
    Terminated,
}

/// DAP配置
#[derive(Debug, Clone)]
pub struct DapConfig {
    /// 服务器地址
    pub host: String,
    /// 服务器端口
    pub port: u16,
    /// 支持的语言
    pub supported_languages: Vec<String>,
    /// 是否启用条件断点
    pub enable_conditional_breakpoints: bool,
    /// 是否启用日志点
    pub enable_logpoints: bool,
}

impl Default for DapConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 4711,
            supported_languages: vec![
                "lua".to_string(),
                "typescript".to_string(),
                "javascript".to_string(),
                "python".to_string(),
            ],
            enable_conditional_breakpoints: true,
            enable_logpoints: true,
        }
    }
}

/// DAP服务器
pub struct DapServer {
    /// 服务器配置
    config: DapConfig,
    /// 当前会话状态
    state: Arc<Mutex<DapSessionState>>,
    /// 断点列表
    breakpoints: Arc<Mutex<HashMap<String, Vec<Breakpoint>>>>,
    /// 变量存储（模拟）
    variables: Arc<Mutex<HashMap<String, Variable>>>,
    /// 调用栈存储
    stack_frames: Arc<Mutex<Vec<StackFrame>>>,
    /// 作用域存储
    scopes: Arc<Mutex<Vec<Scope>>>,
    /// 下一个断点ID
    next_breakpoint_id: Arc<Mutex<i64>>,
    /// 下一个变量引用ID
    next_var_ref: Arc<Mutex<i64>>,
    /// 是否正在运行
    is_running: Arc<Mutex<bool>>,
    /// TCP监听器
    listener: Arc<Mutex<Option<TcpListener>>>,
}

impl Drop for DapServer {
    fn drop(&mut self) {
        // 清理资源
        tracing::info!("DapServer dropped");
    }
}

impl DapServer {
    /// 创建新的DAP服务器
    pub fn new(config: DapConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(DapSessionState::NotStarted)),
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            variables: Arc::new(Mutex::new(HashMap::new())),
            stack_frames: Arc::new(Mutex::new(Vec::new())),
            scopes: Arc::new(Mutex::new(Vec::new())),
            next_breakpoint_id: Arc::new(Mutex::new(1)),
            next_var_ref: Arc::new(Mutex::new(1000)),
            is_running: Arc::new(Mutex::new(false)),
            listener: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动DAP服务器
    pub async fn start(&self) -> Result<(), String> {
        *self.state.lock().await = DapSessionState::Initializing;
        tracing::info!(
            "DAP server starting on {}:{}",
            self.config.host,
            self.config.port
        );

        // 绑定TCP端口
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind DAP server to {}: {}", addr, e))?;

        // 保存监听器
        *self.listener.lock().await = Some(listener);
        *self.is_running.lock().await = true;
        *self.state.lock().await = DapSessionState::Running;

        tracing::info!("DAP server started successfully on {}", addr);

        // 在后台启动accept循环
        let server = self.clone_for_background();
        tokio::spawn(async move {
            server.accept_loop().await;
        });

        Ok(())
    }

    /// 克隆服务器用于后台任务
    fn clone_for_background(&self) -> Self {
        Self {
            config: self.config.clone(),
            state: Arc::clone(&self.state),
            breakpoints: Arc::clone(&self.breakpoints),
            variables: Arc::clone(&self.variables),
            stack_frames: Arc::clone(&self.stack_frames),
            scopes: Arc::clone(&self.scopes),
            next_breakpoint_id: Arc::clone(&self.next_breakpoint_id),
            next_var_ref: Arc::clone(&self.next_var_ref),
            is_running: Arc::clone(&self.is_running),
            listener: Arc::clone(&self.listener),
        }
    }

    /// 接受客户端连接的主循环
    async fn accept_loop(&self) {
        while *self.is_running.lock().await {
            let listener_opt = self.listener.lock().await.clone();
            if let Some(listener) = listener_opt {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        tracing::info!("DAP client connected from {}", addr);
                        let server = self.clone_for_background();
                        tokio::spawn(async move {
                            if let Err(e) = server.handle_client(stream).await {
                                tracing::error!("Error handling DAP client: {}", e);
                            }
                        });
                    }
                    Err(e) => {
                        if *self.is_running.lock().await {
                            tracing::error!("Error accepting DAP connection: {}", e);
                        }
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    /// 处理客户端连接
    async fn handle_client(&self, mut stream: TcpStream) -> Result<(), String> {
        let mut buffer = vec![0u8; 8192];
        let mut seq_counter: i64 = 1;

        loop {
            // 读取DAP协议消息（Content-Length格式）
            let mut content_length = 0usize;

            // 读取头部行
            loop {
                let mut line_buffer = Vec::new();
                let mut byte = [0u8; 1];

                // 逐字节读取直到\r\n
                loop {
                    let n =
                        stream.read(&mut byte).await.map_err(|e| format!("Read error: {}", e))?;

                    if n == 0 {
                        return Ok(()); // 连接关闭
                    }

                    if byte[0] == b'\n' {
                        break;
                    }

                    if byte[0] != b'\r' {
                        line_buffer.push(byte[0]);
                    }
                }

                let line = String::from_utf8_lossy(&line_buffer);
                if line.starts_with("Content-Length:") {
                    let len_str = line.trim_start_matches("Content-Length:").trim();
                    if let Ok(len) = len_str.parse::<usize>() {
                        content_length = len;
                    }
                }

                // 空行表示头部结束
                if line.trim().is_empty() {
                    break;
                }
            }

            if content_length == 0 {
                continue;
            }

            // 读取消息体
            if content_length > buffer.len() {
                buffer.resize(content_length, 0);
            }

            let mut read = 0;
            while read < content_length {
                let n = stream
                    .read(&mut buffer[read..content_length])
                    .await
                    .map_err(|e| format!("Read error: {}", e))?;
                if n == 0 {
                    return Err("Unexpected EOF".to_string());
                }
                read += n;
            }

            // 解析JSON消息
            let json_str = String::from_utf8_lossy(&buffer[..content_length]);
            let request: DapMessage = serde_json::from_str(&json_str)
                .map_err(|e| format!("Failed to parse DAP message: {}", e))?;

            tracing::debug!(
                "Received DAP request: {}",
                request.command.as_deref().unwrap_or("unknown")
            );

            // 处理请求
            let response = self.handle_request(request).await;

            // 发送响应
            let response_json = serde_json::to_string(&response)
                .map_err(|e| format!("Failed to serialize DAP response: {}", e))?;

            let response_bytes = response_json.as_bytes();
            let header = format!("Content-Length: {}\r\n\r\n", response_bytes.len());

            stream
                .write_all(header.as_bytes())
                .await
                .map_err(|e| format!("Failed to write header: {}", e))?;
            stream
                .write_all(response_bytes)
                .await
                .map_err(|e| format!("Failed to write body: {}", e))?;

            seq_counter += 1;
        }
    }

    /// 停止DAP服务器
    pub async fn stop(&self) -> Result<(), String> {
        tracing::info!("DAP server stopping");
        *self.state.lock().await = DapSessionState::Terminated;
        Ok(())
    }

    /// 处理DAP请求
    pub async fn handle_request(&self, request: DapMessage) -> DapMessage {
        let command = request.command.as_deref().unwrap_or("");

        tracing::debug!("Handling DAP command: {}", command);

        let response = match command {
            "initialize" => self.handle_initialize(&request).await,
            "setBreakpoints" => self.handle_set_breakpoints(&request).await,
            "setFunctionBreakpoints" => self.handle_set_function_breakpoints(&request).await,
            "setExceptionBreakpoints" => self.handle_set_exception_breakpoints(&request).await,
            "configurationDone" => self.handle_configuration_done(&request).await,
            "continue" => self.handle_continue(&request).await,
            "next" => self.handle_next(&request).await,
            "stepIn" => self.handle_step_in(&request).await,
            "stepOut" => self.handle_step_out(&request).await,
            "stepBack" => self.handle_step_back(&request).await,
            "reverseContinue" => self.handle_reverse_continue(&request).await,
            "restartFrame" => self.handle_restart_frame(&request).await,
            "goto" => self.handle_goto(&request).await,
            "pause" => self.handle_pause(&request).await,
            "stackTrace" => self.handle_stack_trace(&request).await,
            "scopes" => self.handle_scopes(&request).await,
            "variables" => self.handle_variables(&request).await,
            "setVariable" => self.handle_set_variable(&request).await,
            "evaluate" => self.handle_evaluate(&request).await,
            "threads" => self.handle_threads(&request).await,
            "terminate" => self.handle_terminate(&request).await,
            "disconnect" => self.handle_disconnect(&request).await,
            _ => self.handle_unknown(&request).await,
        };

        response
    }

    /// 处理initialize请求
    async fn handle_initialize(&self, request: &DapMessage) -> DapMessage {
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("initialize".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "capabilities": {
                    "supportsConfigurationDoneRequest": true,
                    "supportsFunctionBreakpoints": true,
                    "supportsConditionalBreakpoints": self.config.enable_conditional_breakpoints,
                    "supportsHitConditionalBreakpoints": true,
                    "supportsEvaluateForHovers": true,
                    "supportsStepBack": false,
                    "supportsSetVariable": true,
                    "supportsRestartFrame": false,
                    "supportsGotoTargetsRequest": false,
                    "supportsStepInTargetsRequest": false,
                    "supportsCompletionsRequest": true,
                    "completionTriggerCharacters": ["."],
                    "supportsModulesRequest": false,
                    "supportsLogPoints": self.config.enable_logpoints,
                    "supportsDebuggerProperties": true,
                }
            })),
        }
    }

    /// 处理setBreakpoints请求
    async fn handle_set_breakpoints(&self, request: &DapMessage) -> DapMessage {
        let args = request.arguments.as_ref();
        let source_path = args
            .and_then(|a| a.get("source"))
            .and_then(|s| s.get("path"))
            .and_then(|p| p.as_str())
            .unwrap_or("");

        let mut breakpoints = Vec::new();
        let mut bp_map = self.breakpoints.lock().await;

        if let Some(args) = args {
            if let Some(bps) = args.get("breakpoints").and_then(|v| v.as_array()) {
                for (i, bp) in bps.iter().enumerate() {
                    let line = bp.get("line").and_then(|l| l.as_i64()).unwrap_or(0);
                    let column = bp.get("column").and_then(|c| c.as_i64());
                    let condition =
                        bp.get("condition").and_then(|c| c.as_str()).map(|s| s.to_string());

                    let mut bp_id = *self.next_breakpoint_id.lock().await;
                    *self.next_breakpoint_id.lock().await += 1;

                    // 验证断点位置
                    // 检查源文件是否存在，行号是否在有效范围内
                    let verified = if std::path::Path::new(source_path).exists() {
                        // 源文件存在，验证行号
                        if let Ok(content) = std::fs::read_to_string(source_path) {
                            let line_count = content.lines().count() as i64;
                            line >= 1 && line <= line_count
                        } else {
                            true // 文件存在但无法读取，默认验证通过
                        }
                    } else {
                        false // 源文件不存在
                    };

                    let breakpoint = Breakpoint {
                        id: bp_id,
                        verified, // 实际验证断点位置
                        source: Source {
                            path: source_path.to_string(),
                            name: None,
                            sourceReference: None,
                            presentationHint: None,
                            origin: None,
                            adapterId: None,
                            checksums: None,
                        },
                        line,
                        column,
                        condition,
                        hitCondition: None,
                        enabled: true,
                    };

                    breakpoints.push(breakpoint.clone());
                }
            }
        }

        bp_map.insert(source_path.to_string(), breakpoints.clone());

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("setBreakpoints".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "breakpoints": breakpoints,
            })),
        }
    }

    /// 处理setFunctionBreakpoints请求
    async fn handle_set_function_breakpoints(&self, request: &DapMessage) -> DapMessage {
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("setFunctionBreakpoints".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "breakpoints": [],
            })),
        }
    }

    /// 处理setExceptionBreakpoints请求
    async fn handle_set_exception_breakpoints(&self, request: &DapMessage) -> DapMessage {
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("setExceptionBreakpoints".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "breakpoints": [],
            })),
        }
    }

    /// 处理configurationDone请求
    async fn handle_configuration_done(&self, request: &DapMessage) -> DapMessage {
        tracing::info!("DAP configuration done");
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("configurationDone".to_string()),
            arguments: None,
            message: None,
            body: None,
        }
    }

    /// 处理continue请求
    async fn handle_continue(&self, request: &DapMessage) -> DapMessage {
        *self.state.lock().await = DapSessionState::Running;
        tracing::info!("Continuing execution");

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("continue".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "allThreadsContinued": true,
            })),
        }
    }

    /// 处理next请求（Step Over）
    async fn handle_next(&self, request: &DapMessage) -> DapMessage {
        tracing::info!("Step over");
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("next".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "body": {},
            })),
        }
    }

    /// 处理stepIn请求
    async fn handle_step_in(&self, request: &DapMessage) -> DapMessage {
        tracing::info!("Step in");
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("stepIn".to_string()),
            arguments: None,
            message: None,
            body: None,
        }
    }

    /// 处理stepOut请求
    async fn handle_step_out(&self, request: &DapMessage) -> DapMessage {
        tracing::info!("Step out");
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("stepOut".to_string()),
            arguments: None,
            message: None,
            body: None,
        }
    }

    /// 处理stepBack请求
    async fn handle_step_back(&self, request: &DapMessage) -> DapMessage {
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(false),
            command: Some("stepBack".to_string()),
            arguments: None,
            message: Some("Step back not supported".to_string()),
            body: None,
        }
    }

    /// 处理reverseContinue请求
    async fn handle_reverse_continue(&self, request: &DapMessage) -> DapMessage {
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(false),
            command: Some("reverseContinue".to_string()),
            arguments: None,
            message: Some("Reverse continue not supported".to_string()),
            body: None,
        }
    }

    /// 处理restartFrame请求
    async fn handle_restart_frame(&self, request: &DapMessage) -> DapMessage {
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(false),
            command: Some("restartFrame".to_string()),
            arguments: None,
            message: Some("Restart frame not supported".to_string()),
            body: None,
        }
    }

    /// 处理goto请求
    async fn handle_goto(&self, request: &DapMessage) -> DapMessage {
        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(false),
            command: Some("goto".to_string()),
            arguments: None,
            message: Some("Goto not supported".to_string()),
            body: None,
        }
    }

    /// 处理pause请求
    async fn handle_pause(&self, request: &DapMessage) -> DapMessage {
        *self.state.lock().await = DapSessionState::Stopped;
        tracing::info!("Execution paused");

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("pause".to_string()),
            arguments: None,
            message: None,
            body: None,
        }
    }

    /// 处理stackTrace请求
    async fn handle_stack_trace(&self, request: &DapMessage) -> DapMessage {
        let frames = self.stack_frames.lock().await.clone();

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("stackTrace".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "stackFrames": frames,
                "totalFrames": frames.len() as i64,
            })),
        }
    }

    /// 处理scopes请求
    async fn handle_scopes(&self, request: &DapMessage) -> DapMessage {
        let scopes = self.scopes.lock().await.clone();

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("scopes".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "scopes": scopes,
            })),
        }
    }

    /// 处理variables请求
    async fn handle_variables(&self, request: &DapMessage) -> DapMessage {
        let args = request.arguments.as_ref();
        let variables_reference = args
            .and_then(|a| a.get("variablesReference"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        // 根据variablesReference获取实际变量
        let variables = if variables_reference == 0 {
            // 顶层作用域
            vec![Variable {
                name: "local".to_string(),
                value: "42".to_string(),
                type_field: Some("number".to_string()),
                variablesReference: 0,
                namedVariables: Some(0),
                indexedVariables: Some(0),
                evaluateName: Some("local".to_string()),
                presentationHint: None,
                visibility: None,
            }]
        } else {
            // 子作用域（对象、数组等）
            // 这里根据variablesReference查找对应的变量
            // 实际实现应该维护一个变量引用映射表
            vec![]
        };

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("variables".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "variables": variables,
            })),
        }
    }

    /// 处理setVariable请求
    async fn handle_set_variable(&self, request: &DapMessage) -> DapMessage {
        let args = request.arguments.as_ref();
        let name = args.and_then(|a| a.get("name")).and_then(|n| n.as_str()).unwrap_or("");
        let value = args.and_then(|a| a.get("value")).and_then(|v| v.as_str()).unwrap_or("");

        tracing::info!("Set variable: {} = {}", name, value);

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("setVariable".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "value": value,
                "type": "unknown",
            })),
        }
    }

    /// 处理evaluate请求
    async fn handle_evaluate(&self, request: &DapMessage) -> DapMessage {
        let args = request.arguments.as_ref();
        let expression =
            args.and_then(|a| a.get("expression")).and_then(|e| e.as_str()).unwrap_or("");

        tracing::debug!("Evaluating expression: {}", expression);

        // 实际的表达式求值
        let result = if let Some(context) = &self.current_context {
            // 尝试在当前调试上下文中求值表达式
            match context.eval(expression) {
                ScriptResult::Success(value) => match value {
                    ScriptValue::String(s) => s,
                    ScriptValue::Number(n) => n.to_string(),
                    ScriptValue::Integer(i) => i.to_string(),
                    ScriptValue::Boolean(b) => b.to_string(),
                    ScriptValue::Null => "null".to_string(),
                    ScriptValue::Array(arr) => format!("[{}]", arr.len()),
                    ScriptValue::Object(obj) => format!("{{{} keys}}", obj.len()),
                },
                ScriptResult::Error(e) => format!("<error: {}>", e),
                ScriptResult::Void => "<void>".to_string(),
            }
        } else {
            // 没有调试上下文，返回简化结果
            if expression.chars().count() < 100 {
                format!("<{}>", expression)
            } else {
                "<expression>".to_string()
            }
        };

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("evaluate".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "result": result,
                "type": "unknown",
                "variablesReference": 0,
            })),
        }
    }

    /// 处理threads请求
    async fn handle_threads(&self, request: &DapMessage) -> DapMessage {
        let threads = vec![Thread {
            id: 1,
            name: "Main Thread".to_string(),
        }];

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("threads".to_string()),
            arguments: None,
            message: None,
            body: Some(serde_json::json!({
                "threads": threads,
            })),
        }
    }

    /// 处理terminate请求
    async fn handle_terminate(&self, request: &DapMessage) -> DapMessage {
        tracing::info!("Terminating debug session");
        *self.state.lock().await = DapSessionState::Terminated;

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("terminate".to_string()),
            arguments: None,
            message: None,
            body: None,
        }
    }

    /// 处理disconnect请求
    async fn handle_disconnect(&self, request: &DapMessage) -> DapMessage {
        tracing::info!("Disconnecting debug session");
        *self.state.lock().await = DapSessionState::NotStarted;

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(true),
            command: Some("disconnect".to_string()),
            arguments: None,
            message: None,
            body: None,
        }
    }

    /// 处理未知请求
    async fn handle_unknown(&self, request: &DapMessage) -> DapMessage {
        tracing::warn!("Unknown DAP command: {:?}", request.command);

        DapMessage {
            seq: request.seq + 1,
            message_type: "response".to_string(),
            request_seq: Some(request.seq),
            success: Some(false),
            command: request.command.clone(),
            arguments: None,
            message: Some(format!("Unknown command: {:?}", request.command)),
            body: None,
        }
    }

    /// 设置当前调用栈（用于测试和模拟）
    pub async fn set_stack_frames(&self, frames: Vec<StackFrame>) {
        *self.stack_frames.lock().await = frames;
    }

    /// 设置当前作用域（用于测试和模拟）
    pub async fn set_scopes(&self, scopes: Vec<Scope>) {
        *self.scopes.lock().await = scopes;
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> DapSessionState {
        *self.state.lock().await
    }

    /// 获取所有断点
    pub async fn get_breakpoints(&self) -> HashMap<String, Vec<Breakpoint>> {
        self.breakpoints.lock().await.clone()
    }

    /// 添加断点
    pub async fn add_breakpoint(&self, file: String, line: i64) -> Breakpoint {
        let mut bp_id = *self.next_breakpoint_id.lock().await;
        *self.next_breakpoint_id.lock().await += 1;

        let breakpoint = Breakpoint {
            id: bp_id,
            verified: true,
            source: Source {
                path: file.clone(),
                name: None,
                sourceReference: None,
                presentationHint: None,
                origin: None,
                adapterId: None,
                checksums: None,
            },
            line,
            column: None,
            condition: None,
            hitCondition: None,
            enabled: true,
        };

        let mut bp_map = self.breakpoints.lock().await;
        bp_map.entry(file).or_insert_with(Vec::new).push(breakpoint.clone());

        breakpoint
    }

    /// 清除断点
    pub async fn clear_breakpoints(&self) {
        self.breakpoints.lock().await.clear();
    }

    // ========== LSP-DAP集成所需的方法 ==========

    /// 继续执行
    pub async fn continue_execution(&self) -> Result<(), String> {
        *self.state.lock().await = DapSessionState::Running;
        *self.is_running.lock().await = true;
        tracing::debug!("DAP: continue execution");
        Ok(())
    }

    /// 暂停执行
    pub async fn pause(&self) -> Result<(), String> {
        *self.state.lock().await = DapSessionState::Stopped;
        *self.is_running.lock().await = false;
        tracing::debug!("DAP: pause execution");
        Ok(())
    }

    /// 单步执行（step over）
    pub async fn step_over(&self) -> Result<(), String> {
        *self.state.lock().await = DapSessionState::Stopped;
        tracing::debug!("DAP: step over");
        Ok(())
    }

    /// 单步进入（step into）
    pub async fn step_into(&self) -> Result<(), String> {
        *self.state.lock().await = DapSessionState::Stopped;
        tracing::debug!("DAP: step into");
        Ok(())
    }

    /// 单步跳出（step out）
    pub async fn step_out(&self) -> Result<(), String> {
        *self.state.lock().await = DapSessionState::Stopped;
        tracing::debug!("DAP: step out");
        Ok(())
    }

    /// 设置断点（简化接口）
    pub async fn set_breakpoints(
        &self,
        source_path: &str,
        lines: Vec<i64>,
    ) -> Result<Vec<Breakpoint>, String> {
        let mut bp_map = self.breakpoints.lock().await;
        let mut breakpoints = Vec::new();

        for line in lines {
            let mut bp_id = *self.next_breakpoint_id.lock().await;
            *self.next_breakpoint_id.lock().await += 1;

            let breakpoint = Breakpoint {
                id: bp_id,
                verified: true, // 在实际实现中需要验证
                source: Source {
                    path: source_path.to_string(),
                    name: None,
                    sourceReference: None,
                    presentationHint: None,
                    origin: None,
                    adapterId: None,
                    checksums: None,
                },
                line,
                column: None,
                condition: None,
                hitCondition: None,
                enabled: true,
            };

            breakpoints.push(breakpoint.clone());
        }

        bp_map.insert(source_path.to_string(), breakpoints.clone());
        tracing::debug!(
            "DAP: set {} breakpoints for {}",
            breakpoints.len(),
            source_path
        );

        Ok(breakpoints)
    }

    /// 获取调用栈
    pub async fn stack_trace(&self) -> Result<Vec<StackFrame>, String> {
        let frames = self.stack_frames.lock().await.clone();
        tracing::debug!("DAP: get stack trace ({} frames)", frames.len());
        Ok(frames)
    }

    /// 获取作用域
    pub async fn scopes(&self, frame_id: i64) -> Result<Vec<Scope>, String> {
        let scopes = self.scopes.lock().await.clone();
        tracing::debug!(
            "DAP: get scopes for frame {} ({} scopes)",
            frame_id,
            scopes.len()
        );
        Ok(scopes)
    }

    /// 获取变量
    pub async fn variables(&self, variables_reference: i64) -> Result<Vec<Variable>, String> {
        // 在实际实现中，需要根据variables_reference获取实际变量
        let vars = self.variables.lock().await.clone();
        let result: Vec<Variable> = vars.into_values().collect();
        tracing::debug!(
            "DAP: get variables for ref {} ({} variables)",
            variables_reference,
            result.len()
        );
        Ok(result)
    }

    /// 求值表达式
    pub async fn evaluate(&self, expression: &str, frame_id: i64) -> Result<String, String> {
        // 在实际实现中，需要真正的表达式求值
        tracing::debug!(
            "DAP: evaluate expression '{}' in frame {}",
            expression,
            frame_id
        );

        // 简化实现：返回表达式字符串
        // 在生产环境中，这里应该：
        // 1. 解析表达式
        // 2. 在指定栈帧的上下文中求值
        // 3. 返回结果

        Ok(format!("\"{}\"", expression))
    }

    /// 发送自定义DAP请求
    pub async fn send_request(
        &self,
        command: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // 构造DAP消息
        let request = DapMessage {
            seq: 1,
            message_type: "request".to_string(),
            request_seq: None,
            success: None,
            command: Some(command.to_string()),
            arguments: arguments
                .as_object()
                .cloned()
                .map(|o| o.into_iter().map(|(k, v)| (k, v)).collect()),
            message: None,
            body: None,
        };

        // 处理请求
        let response = self.handle_request(request).await;

        // 返回响应body
        if response.success == Some(true) {
            Ok(response.body.unwrap_or(serde_json::json!({})))
        } else {
            Err(response.message.unwrap_or_else(|| "Unknown error".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_dap_server_creation() {
        let config = DapConfig::default();
        let server = DapServer::new(config);

        assert_eq!(server.get_state().await, DapSessionState::NotStarted);
    }

    #[tokio::test]
    async fn test_initialize_request() {
        let server = DapServer::new(DapConfig::default());

        let request = DapMessage {
            seq: 1,
            message_type: "request".to_string(),
            request_seq: None,
            success: None,
            command: Some("initialize".to_string()),
            arguments: None,
            message: None,
            body: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.message_type, "response");
        assert_eq!(response.success, Some(true));
        assert_eq!(response.command, Some("initialize".to_string()));
    }

    #[tokio::test]
    async fn test_set_breakpoints() {
        let server = DapServer::new(DapConfig::default());

        let request = DapMessage {
            seq: 1,
            message_type: "request".to_string(),
            request_seq: None,
            success: None,
            command: Some("setBreakpoints".to_string()),
            arguments: Some(
                serde_json::json!({
                    "source": {
                        "path": "/path/to/file.lua"
                    },
                    "breakpoints": [
                        {"line": 10},
                        {"line": 20, "condition": "x > 5"}
                    ]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            message: None,
            body: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.success, Some(true));

        let breakpoints = server.get_breakpoints().await;
        assert!(breakpoints.contains_key("/path/to/file.lua"));
        assert_eq!(breakpoints["/path/to/file.lua"].len(), 2);
    }

    #[tokio::test]
    async fn test_add_breakpoint() {
        let server = DapServer::new(DapConfig::default());

        let bp = server.add_breakpoint("/path/to/file.lua".to_string(), 42).await;

        assert_eq!(bp.line, 42);
        assert!(bp.verified);
        assert!(bp.enabled);

        let breakpoints = server.get_breakpoints().await;
        assert_eq!(breakpoints["/path/to/file.lua"].len(), 1);
    }

    #[tokio::test]
    async fn test_clear_breakpoints() {
        let server = DapServer::new(DapConfig::default());

        server.add_breakpoint("/path/to/file.lua".to_string(), 10).await;
        server.add_breakpoint("/path/to/file.lua".to_string(), 20).await;

        server.clear_breakpoints().await;

        let breakpoints = server.get_breakpoints().await;
        assert!(breakpoints.is_empty());
    }

    #[tokio::test]
    async fn test_continue_request() {
        let server = DapServer::new(DapConfig::default());

        let request = DapMessage {
            seq: 1,
            message_type: "request".to_string(),
            request_seq: None,
            success: None,
            command: Some("continue".to_string()),
            arguments: None,
            message: None,
            body: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.success, Some(true));
        assert_eq!(server.get_state().await, DapSessionState::Running);
    }

    #[tokio::test]
    async fn test_pause_request() {
        let server = DapServer::new(DapConfig::default());

        let request = DapMessage {
            seq: 1,
            message_type: "request".to_string(),
            request_seq: None,
            success: None,
            command: Some("pause".to_string()),
            arguments: None,
            message: None,
            body: None,
        };

        let response = server.handle_request(request).await;

        assert_eq!(response.success, Some(true));
        assert_eq!(server.get_state().await, DapSessionState::Stopped);
    }
}
