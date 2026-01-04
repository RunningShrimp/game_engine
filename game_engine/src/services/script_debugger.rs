//! # Script Debugger
//!
//! 脚本调试器 - 支持JavaScript和Python脚本调试。
//!
//! ## 核心功能
//!
//! 1. **BreakpointManager** - 断点管理
//! 2. **SteppingController** - 单步执行控制
//! 3. **VariableWatcher** - 变量监视
//! 4. **CallStackInspector** - 调用栈查看
//! 5. **ExpressionEvaluator** - 表达式求值

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// 调试器状态
#[derive(Clone, Debug, PartialEq)]
pub enum DebuggerState {
    /// 未启动
    Idle,
    /// 运行中
    Running,
    /// 暂停在断点
    PausedAtBreakpoint,
    /// 单步执行中
    Stepping,
    /// 已停止
    Stopped,
}

/// 断点信息
#[derive(Clone, Debug)]
pub struct Breakpoint {
    /// 断点ID
    pub id: u32,
    /// 文件路径
    pub file_path: PathBuf,
    /// 行号
    pub line: u32,
    /// 是否启用
    pub enabled: bool,
    /// 命中次数
    pub hit_count: u32,
    /// 条件表达式（可选）
    pub condition: Option<String>,
}

/// 调用栈帧
#[derive(Clone, Debug)]
pub struct StackFrame {
    /// 帧索引
    pub index: u32,
    /// 函数名
    pub function_name: String,
    /// 文件路径
    pub file_path: PathBuf,
    /// 行号
    pub line: u32,
    /// 局部变量
    pub locals: HashMap<String, String>,
}

/// 变量监视
#[derive(Clone, Debug)]
pub struct WatchedVariable {
    /// 变量名
    pub name: String,
    /// 当前值
    pub current_value: String,
    /// 类型
    pub type_name: String,
}

/// 调试器
pub struct ScriptDebugger {
    /// 当前状态
    state: Arc<Mutex<DebuggerState>>,
    /// 断点列表
    breakpoints: Arc<Mutex<HashMap<u32, Breakpoint>>>,
    /// 下一个断点ID
    next_breakpoint_id: Arc<Mutex<u32>>,
    /// 调用栈
    call_stack: Arc<Mutex<Vec<StackFrame>>>,
    /// 监视的变量
    watched_variables: Arc<Mutex<HashMap<String, WatchedVariable>>>,
    /// 当前文件和行号
    current_position: Arc<Mutex<Option<(PathBuf, u32)>>>,
}

impl Default for ScriptDebugger {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptDebugger {
    /// 创建新的调试器
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(DebuggerState::Idle)),
            breakpoints: Arc::new(Mutex::new(HashMap::new())),
            next_breakpoint_id: Arc::new(Mutex::new(1)),
            call_stack: Arc::new(Mutex::new(Vec::new())),
            watched_variables: Arc::new(Mutex::new(HashMap::new())),
            current_position: Arc::new(Mutex::new(None)),
        }
    }

    /// 启动调试
    pub fn start(&self) {
        *self.state.lock().unwrap() = DebuggerState::Running;
        tracing::info!(target: "script_debugger", "Debugger started");
    }

    /// 停止调试
    pub fn stop(&self) {
        *self.state.lock().unwrap() = DebuggerState::Stopped;
        tracing::info!(target: "script_debugger", "Debugger stopped");
    }

    /// 暂停执行
    pub fn pause(&self) {
        *self.state.lock().unwrap() = DebuggerState::PausedAtBreakpoint;
        tracing::info!(target: "script_debugger", "Debugger paused");
    }

    /// 继续执行
    pub fn continue_execution(&self) {
        *self.state.lock().unwrap() = DebuggerState::Running;
        tracing::info!(target: "script_debugger", "Debugger continuing");
    }

    /// 单步执行（进入函数）
    pub fn step_into(&self) {
        *self.state.lock().unwrap() = DebuggerState::Stepping;
        tracing::info!(target: "script_debugger", "Step into");
    }

    /// 单步执行（跳过函数）
    pub fn step_over(&self) {
        *self.state.lock().unwrap() = DebuggerState::Stepping;
        tracing::info!(target: "script_debugger", "Step over");
    }

    /// 跳出当前函数
    pub fn step_out(&self) {
        *self.state.lock().unwrap() = DebuggerState::Stepping;
        tracing::info!(target: "script_debugger", "Step out");
    }

    /// 添加断点
    pub fn add_breakpoint(&self, file_path: PathBuf, line: u32, condition: Option<String>) -> u32 {
        let mut next_id = self.next_breakpoint_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let breakpoint = Breakpoint {
            id,
            file_path: file_path.clone(),
            line,
            enabled: true,
            hit_count: 0,
            condition,
        };

        self.breakpoints.lock().unwrap().insert(id, breakpoint);
        tracing::info!(target: "script_debugger", "Added breakpoint #{} at {}:{}", id, file_path.display(), line);
        id
    }

    /// 移除断点
    pub fn remove_breakpoint(&self, breakpoint_id: u32) -> bool {
        let removed = self.breakpoints.lock().unwrap().remove(&breakpoint_id).is_some();
        if removed {
            tracing::info!(target: "script_debugger", "Removed breakpoint #{}", breakpoint_id);
        }
        removed
    }

    /// 启用/禁用断点
    pub fn toggle_breakpoint(&self, breakpoint_id: u32) -> bool {
        if let Some(mut bp) = self.breakpoints.lock().unwrap().get_mut(&breakpoint_id) {
            bp.enabled = !bp.enabled;
            tracing::info!(target: "script_debugger", "Toggled breakpoint #{}: {}",
                breakpoint_id, if bp.enabled { "enabled" } else { "disabled" });
            true
        } else {
            false
        }
    }

    /// 获取所有断点
    pub fn get_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoints.lock().unwrap().values().cloned().collect()
    }

    /// 检查是否应该在此处暂停
    pub fn should_pause_at(&self, file_path: &PathBuf, line: u32) -> bool {
        for breakpoint in self.breakpoints.lock().unwrap().values() {
            if breakpoint.enabled && breakpoint.file_path == *file_path && breakpoint.line == line {
                // 检查条件
                if let Some(condition) = &breakpoint.condition {
                    // 在实际实现中，这里会评估条件表达式
                    tracing::debug!(target: "script_debugger", "Breakpoint condition: {}", condition);
                    return true;
                } else {
                    return true;
                }
            }
        }
        false
    }

    /// 添加调用栈帧
    pub fn push_stack_frame(&self, frame: StackFrame) {
        self.call_stack.lock().unwrap().push(frame);
    }

    /// 弹出调用栈帧
    pub fn pop_stack_frame(&self) -> Option<StackFrame> {
        self.call_stack.lock().unwrap().pop()
    }

    /// 获取调用栈
    pub fn get_call_stack(&self) -> Vec<StackFrame> {
        self.call_stack.lock().unwrap().clone()
    }

    /// 清空调用栈
    pub fn clear_call_stack(&self) {
        self.call_stack.lock().unwrap().clear();
    }

    /// 添加监视变量
    pub fn add_watch(&self, variable_name: String) {
        let watched = WatchedVariable {
            name: variable_name.clone(),
            current_value: "<unknown>".to_string(),
            type_name: "unknown".to_string(),
        };
        self.watched_variables.lock().unwrap().insert(variable_name.clone(), watched);
        tracing::info!(target: "script_debugger", "Added watch for variable '{}'", variable_name);
    }

    /// 移除监视变量
    pub fn remove_watch(&self, variable_name: &str) -> bool {
        let removed = self.watched_variables.lock().unwrap().remove(variable_name).is_some();
        if removed {
            tracing::info!(target: "script_debugger", "Removed watch for variable '{}'", variable_name);
        }
        removed
    }

    /// 更新监视变量的值
    pub fn update_watch_value(&self, variable_name: &str, value: String, type_name: String) {
        if let Some(mut watched) = self.watched_variables.lock().unwrap().get_mut(variable_name) {
            watched.current_value = value;
            watched.type_name = type_name;
        }
    }

    /// 获取所有监视变量
    pub fn get_watched_variables(&self) -> Vec<WatchedVariable> {
        self.watched_variables.lock().unwrap().values().cloned().collect()
    }

    /// 设置当前位置
    pub fn set_current_position(&self, file_path: PathBuf, line: u32) {
        *self.current_position.lock().unwrap() = Some((file_path, line));
    }

    /// 获取当前位置
    pub fn get_current_position(&self) -> Option<(PathBuf, u32)> {
        self.current_position.lock().unwrap().clone()
    }

    /// 获取调试器状态
    pub fn get_state(&self) -> DebuggerState {
        self.state.lock().unwrap().clone()
    }

    /// 评估表达式
    pub fn evaluate_expression(&self, expression: &str) -> Result<String, String> {
        // 在实际实现中，这里会使用脚本引擎来评估表达式
        tracing::info!(target: "script_debugger", "Evaluating expression: {}", expression);
        Ok(format!("<evaluated: {expression}>"))
    }

    /// 获取局部变量
    pub fn get_locals(&self, frame_index: usize) -> HashMap<String, String> {
        if let Some(frames) = self.call_stack.lock().unwrap().get(frame_index) {
            frames.locals.clone()
        } else {
            HashMap::new()
        }
    }

    /// 设置局部变量值
    pub fn set_local(&self, frame_index: usize, name: &str, value: &str) -> Result<(), String> {
        let mut frames = self.call_stack.lock().unwrap();
        if let Some(frame) = frames.get_mut(frame_index) {
            frame.locals.insert(name.to_string(), value.to_string());
            Ok(())
        } else {
            Err(format!("Frame {frame_index} not found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debugger_creation() {
        let debugger = ScriptDebugger::new();
        assert_eq!(debugger.get_state(), DebuggerState::Idle);
    }

    #[test]
    fn test_debugger_start_stop() {
        let debugger = ScriptDebugger::new();
        debugger.start();
        assert_eq!(debugger.get_state(), DebuggerState::Running);
        debugger.stop();
        assert_eq!(debugger.get_state(), DebuggerState::Stopped);
    }

    #[test]
    fn test_breakpoint_management() {
        let debugger = ScriptDebugger::new();
        let path = PathBuf::from("test.js");
        let bp_id = debugger.add_breakpoint(path.clone(), 10, None);

        let breakpoints = debugger.get_breakpoints();
        assert_eq!(breakpoints.len(), 1);
        assert_eq!(breakpoints[0].line, 10);

        assert!(debugger.remove_breakpoint(bp_id));
        assert_eq!(debugger.get_breakpoints().len(), 0);
    }

    #[test]
    fn test_watch_variables() {
        let debugger = ScriptDebugger::new();
        debugger.add_watch("testVar".to_string());

        let watches = debugger.get_watched_variables();
        assert_eq!(watches.len(), 1);
        assert_eq!(watches[0].name, "testVar");

        debugger.update_watch_value("testVar", "42".to_string(), "number".to_string());
        let watches = debugger.get_watched_variables();
        assert_eq!(watches[0].current_value, "42");
    }

    #[test]
    fn test_call_stack() {
        let debugger = ScriptDebugger::new();

        let frame1 = StackFrame {
            index: 0,
            function_name: "main".to_string(),
            file_path: PathBuf::from("main.js"),
            line: 1,
            locals: HashMap::new(),
        };

        debugger.push_stack_frame(frame1);
        assert_eq!(debugger.get_call_stack().len(), 1);

        debugger.pop_stack_frame();
        assert_eq!(debugger.get_call_stack().len(), 0);
    }

    #[test]
    fn test_should_pause_at() {
        let debugger = ScriptDebugger::new();
        let path = PathBuf::from("test.js");
        debugger.add_breakpoint(path.clone(), 10, None);

        assert!(debugger.should_pause_at(&path, 10));
        assert!(!debugger.should_pause_at(&path, 11));
    }
}
