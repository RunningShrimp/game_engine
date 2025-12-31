//! Lua调试器集成
//!
//! 提供Lua脚本调试功能，集成到DAP服务器中。

use crate::debug::dap::server::{
    Breakpoint, DapMessage, DapServer, Scope, Source, StackFrame, Thread, Variable,
};
use crate::debug::variables::{Scope as VariableScope, ScopeKind, Variable as DebugVariable};
use crate::scripting::lua_support::LuaContext;
use async_trait::async_trait;
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Lua调试器
///
/// 提供Lua脚本的断点调试、变量监视等功能。
pub struct LuaDebugger {
    /// Lua上下文
    lua_context: Arc<Mutex<LuaContext>>,
    /// 断点映射 (源文件 -> 行号 -> 断点ID)
    breakpoints: Arc<Mutex<HashMap<String, HashMap<i64, i64>>>>,
    /// 调试会话状态
    session_state: Arc<Mutex<LuaDebugSession>>,
    /// 是否已启用
    enabled: Arc<Mutex<bool>>,
}

/// Lua调试会话状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaDebugSession {
    /// 未启动
    NotStarted,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 步进模式
    Stepping,
    /// 已结束
    Ended,
}

impl LuaDebugger {
    /// 创建新的Lua调试器
    pub fn new(lua_context: Arc<Mutex<LuaContext>>) -> Self {
        Self {
            lua_context,
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            session_state: Arc::new(Mutex::new(LuaDebugSession::NotStarted)),
            enabled: Arc::new(Mutex::new(false)),
        }
    }

    /// 启用调试器
    pub fn enable(&self) {
        let mut enabled = self.enabled.lock().unwrap();
        *enabled = true;

        // 安装调试钩子
        self.install_debug_hook();
    }

    /// 禁用调试器
    pub fn disable(&self) {
        let mut enabled = self.enabled.lock().unwrap();
        *enabled = false;
    }

    /// 检查是否已启用
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    /// 安装调试钩子
    fn install_debug_hook(&self) {
        // 在Lua中设置调试钩子
        // debug.sethook(function(event, line) ... end, "l")
        log::info!("Installing Lua debug hook");
    }

    /// 添加断点
    pub fn add_breakpoint(&self, source: &str, line: i64) -> i64 {
        let mut breakpoints = self.breakpoints.lock().unwrap();
        let source_breakpoints = breakpoints.entry(source.to_string()).or_insert_with(HashMap::new);

        let bp_id = source_breakpoints.len() as i64 + 1;
        source_breakpoints.insert(line, bp_id);

        log::info!("Added Lua breakpoint: {}:{} (ID: {})", source, line, bp_id);

        bp_id
    }

    /// 移除断点
    pub fn remove_breakpoint(&self, source: &str, line: i64) -> bool {
        let mut breakpoints = self.breakpoints.lock().unwrap();

        if let Some(source_breakpoints) = breakpoints.get_mut(source) {
            let removed = source_breakpoints.remove(&line).is_some();

            if source_breakpoints.is_empty() {
                breakpoints.remove(source);
            }

            removed
        } else {
            false
        }
    }

    /// 检查是否应该在此行暂停
    pub fn should_break(&self, source: &str, line: i64) -> bool {
        let breakpoints = self.breakpoints.lock().unwrap();

        if let Some(source_breakpoints) = breakpoints.get(source) {
            source_breakpoints.contains_key(&line)
        } else {
            false
        }
    }

    /// 获取调用栈
    pub fn get_stack_trace(&self) -> Vec<StackFrame> {
        let mut lua = self.lua_context.lock().unwrap();
        // TODO: 使用Lua debug API获取栈跟踪
        // debug.traceback()

        vec![StackFrame {
            id: 1,
            name: "main".to_string(),
            source: Source {
                name: "script.lua".to_string(),
                path: Some("script.lua".to_string()),
                source_reference: None,
                presentation_hint: None,
                origin: None,
                sources: None,
                adapter_data: None,
                checksums: None,
            },
            line: 1,
            column: 1,
            end_line: None,
            end_column: None,
            can_restart: false,
            instruction_pointer_reference: None,
        }]
    }

    /// 获取局部变量
    pub fn get_local_variables(&self, frame_id: i64) -> Vec<DebugVariable> {
        let mut lua = self.lua_context.lock().unwrap();
        // TODO: 使用Lua debug API获取局部变量
        // debug.getlocal(level, i)

        vec![DebugVariable {
            name: "test_var".to_string(),
            value: "42".to_string(),
            type_name: Some("number".to_string()),
            variables_reference: None,
            named_variables: None,
            indexed_variables: None,
            evaluate_name: Some("test_var".to_string()),
            memory_reference: None,
        }]
    }

    /// 获取全局变量
    pub fn get_global_variables(&self) -> Vec<DebugVariable> {
        let mut lua = self.lua_context.lock().unwrap();
        // TODO: 遍历_G获取全局变量

        vec![DebugVariable {
            name: "print".to_string(),
            value: "function: 0x...".to_string(),
            type_name: Some("function".to_string()),
            variables_reference: None,
            named_variables: None,
            indexed_variables: None,
            evaluate_name: Some("print".to_string()),
            memory_reference: None,
        }]
    }

    /// 评估表达式
    pub fn evaluate(&self, expression: &str) -> Result<String, String> {
        let mut lua = self.lua_context.lock().unwrap();

        // 执行表达式并返回结果
        match lua.eval(expression, None) {
            Ok(result) => Ok(result.to_string()),
            Err(e) => Err(format!("Evaluation error: {}", e)),
        }
    }

    /// 继续执行
    pub fn continue_execution(&self) {
        let mut state = self.session_state.lock().unwrap();
        *state = LuaDebugSession::Running;
        log::info!("Lua debugger: Continuing execution");
    }

    /// 暂停执行
    pub fn pause(&self) {
        let mut state = self.session_state.lock().unwrap();
        *state = LuaDebugSession::Paused;
        log::info!("Lua debugger: Paused");
    }

    /// 单步执行
    pub fn step(&self) {
        let mut state = self.session_state.lock().unwrap();
        *state = LuaDebugSession::Stepping;
        log::info!("Lua debugger: Stepping");
    }

    /// 获取当前会话状态
    pub fn get_session_state(&self) -> LuaDebugSession {
        *self.session_state.lock().unwrap()
    }
}

/// Lua调试器DAP适配器
///
/// 将LuaDebugger集成到DAP服务器中。
pub struct LuaDapAdapter {
    debugger: Arc<LuaDebugger>,
}

impl LuaDapAdapter {
    /// 创建新的适配器
    pub fn new(debugger: Arc<LuaDebugger>) -> Self {
        Self { debugger }
    }

    /// 处理DAP initialize请求
    pub fn handle_initialize(&self, request: &DapMessage) -> DapMessage {
        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "initialize".to_string(),
            message: None,
            body: Some(json!({
                "configurationDone": true,
                "supportsConditionalBreakpoints": true,
                "supportsHitConditionalBreakpoints": false,
                "supportsEvaluateForHovers": true,
                "supportsStepBack": false,
                "supportsSetVariable": true,
                "supportsRestartFrame": false,
                "supportsGotoTargetsRequest": false,
                "supportsStepInTargetsRequest": false,
                "supportsCompletionsRequest": true,
                "supportsModulesRequest": false,
                "supportsModulesRequest": false,
                "supportsValueFormattingOptions": true,
                "supportsExceptionInfoRequest": true,
                "supportTerminateDebuggee": true,
                "supportSuspendDebuggee": true,
                "supportsDelayedStackTraceLoading": false,
                "supportsLoadedSourcesRequest": false,
                "supportsLogPoints": false,
                "supportsBreakpointsLocationsRequest": false,
                "supportsClipboardContext": true,
                "exceptionBreakpointFilters": [],
                "completionTriggerCharacters": [".", ":", "["],
            })),
        }
    }

    /// 处理setBreakpoints请求
    pub fn handle_set_breakpoints(&self, request: &DapMessage) -> DapMessage {
        let body = request.body.as_ref().unwrap();

        let source = body["source"]["path"].as_str().unwrap_or("unknown.lua");

        let breakpoints = body["breakpoints"].as_array().unwrap();

        let mut results = Vec::new();

        for (i, bp) in breakpoints.iter().enumerate() {
            let line = bp["line"].as_i64().unwrap();
            let bp_id = self.debugger.add_breakpoint(source, line);

            results.push(json!({
                "verified": true,
                "line": line,
                "id": bp_id,
            }));
        }

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "setBreakpoints".to_string(),
            message: None,
            body: Some(json!({
                "breakpoints": results,
            })),
        }
    }

    /// 处理stackTrace请求
    pub fn handle_stack_trace(&self, request: &DapMessage) -> DapMessage {
        let frames = self.debugger.get_stack_trace();

        let frames_json: Vec<serde_json::Value> = frames
            .iter()
            .map(|f| {
                json!({
                    "id": f.id,
                    "name": f.name,
                    "source": {
                        "name": f.source.name,
                        "path": f.source.path,
                    },
                    "line": f.line,
                    "column": f.column,
                })
            })
            .collect();

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "stackTrace".to_string(),
            message: None,
            body: Some(json!({
                "stackFrames": frames_json,
                "totalFrames": frames_json.len(),
            })),
        }
    }

    /// 处理scopes请求
    pub fn handle_scopes(&self, request: &DapMessage) -> DapMessage {
        let body = request.body.as_ref().unwrap();
        let frame_id = body["frameId"].as_i64().unwrap();

        let scopes = vec![
            Scope {
                name: "Locals".to_string(),
                variables_reference: (frame_id * 10 + 1),
                named_variables: Some(10),
                indexed_variables: Some(0),
                expensive: false,
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            },
            Scope {
                name: "Globals".to_string(),
                variables_reference: (frame_id * 10 + 2),
                named_variables: Some(100),
                indexed_variables: Some(0),
                expensive: false,
                source: None,
                line: None,
                column: None,
                end_line: None,
                end_column: None,
            },
        ];

        let scopes_json: Vec<serde_json::Value> = scopes
            .iter()
            .map(|s| {
                json!({
                    "name": s.name,
                    "variablesReference": s.variables_reference,
                    "namedVariables": s.named_variables,
                    "indexedVariables": s.indexed_variables,
                    "expensive": s.expensive,
                })
            })
            .collect();

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "scopes".to_string(),
            message: None,
            body: Some(json!({
                "scopes": scopes_json,
            })),
        }
    }

    /// 处理variables请求
    pub fn handle_variables(&self, request: &DapMessage) -> DapMessage {
        let body = request.body.as_ref().unwrap();
        let variables_reference = body["variablesReference"].as_i64().unwrap();

        let vars = if variables_reference % 10 == 1 {
            // Locals
            self.debugger.get_local_variables(variables_reference / 10)
        } else {
            // Globals
            self.debugger.get_global_variables()
        };

        let vars_json: Vec<serde_json::Value> = vars
            .iter()
            .map(|v| {
                json!({
                    "name": v.name,
                    "value": v.value,
                    "type": v.type_name,
                    "variablesReference": v.variables_reference.unwrap_or(0),
                    "namedVariables": v.named_variables,
                    "indexedVariables": v.indexed_variables,
                    "evaluateName": v.evaluate_name,
                })
            })
            .collect();

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "variables".to_string(),
            message: None,
            body: Some(json!({
                "variables": vars_json,
            })),
        }
    }

    /// 处理evaluate请求
    pub fn handle_evaluate(&self, request: &DapMessage) -> DapMessage {
        let body = request.body.as_ref().unwrap();
        let expression = body["expression"].as_str().unwrap_or("");

        match self.debugger.evaluate(expression) {
            Ok(result) => DapMessage {
                seq: request.seq + 1,
                type_: "response".to_string(),
                request_seq: request.seq,
                success: true,
                command: "evaluate".to_string(),
                message: None,
                body: Some(json!({
                    "result": result,
                    "variablesReference": 0,
                })),
            },
            Err(e) => DapMessage {
                seq: request.seq + 1,
                type_: "response".to_string(),
                request_seq: request.seq,
                success: false,
                command: "evaluate".to_string(),
                message: Some(e),
                body: None,
            },
        }
    }

    /// 处理continue请求
    pub fn handle_continue(&self, request: &DapMessage) -> DapMessage {
        self.debugger.continue_execution();

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "continue".to_string(),
            message: None,
            body: Some(json!({
                "allThreadsContinued": true,
            })),
        }
    }

    /// 处理next请求（Step Over）
    pub fn handle_next(&self, request: &DapMessage) -> DapMessage {
        self.debugger.step();

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "next".to_string(),
            message: None,
            body: Some(json!({
                "allThreadsContinued": false,
            })),
        }
    }

    /// 处理stepIn请求
    pub fn handle_step_in(&self, request: &DapMessage) -> DapMessage {
        self.debugger.step();

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "stepIn".to_string(),
            message: None,
            body: Some(json!({
                "allThreadsContinued": false,
            })),
        }
    }

    /// 处理stepOut请求
    pub fn handle_step_out(&self, request: &DapMessage) -> DapMessage {
        self.debugger.step();

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "stepOut".to_string(),
            message: None,
            body: Some(json!({
                "allThreadsContinued": false,
            })),
        }
    }

    /// 处理pause请求
    pub fn handle_pause(&self, request: &DapMessage) -> DapMessage {
        self.debugger.pause();

        DapMessage {
            seq: request.seq + 1,
            type_: "response".to_string(),
            request_seq: request.seq,
            success: true,
            command: "pause".to_string(),
            message: None,
            body: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_debugger_creation() {
        let lua_context = Arc::new(Mutex::new(LuaContext::new()));
        let debugger = LuaDebugger::new(lua_context);

        assert!(!debugger.is_enabled());

        debugger.enable();
        assert!(debugger.is_enabled());
    }

    #[test]
    fn test_breakpoint_management() {
        let lua_context = Arc::new(Mutex::new(LuaContext::new()));
        let debugger = LuaDebugger::new(lua_context);

        let bp_id = debugger.add_breakpoint("test.lua", 10);
        assert!(bp_id > 0);

        assert!(debugger.should_break("test.lua", 10));
        assert!(!debugger.should_break("test.lua", 11));

        assert!(debugger.remove_breakpoint("test.lua", 10));
        assert!(!debugger.should_break("test.lua", 10));
    }

    #[test]
    fn test_session_state() {
        let lua_context = Arc::new(Mutex::new(LuaContext::new()));
        let debugger = LuaDebugger::new(lua_context);

        assert_eq!(debugger.get_session_state(), LuaDebugSession::NotStarted);

        debugger.continue_execution();
        assert_eq!(debugger.get_session_state(), LuaDebugSession::Running);

        debugger.pause();
        assert_eq!(debugger.get_session_state(), LuaDebugSession::Paused);
    }
}
