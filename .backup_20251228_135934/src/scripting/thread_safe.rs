use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread;

/// 脚本命令
#[derive(Debug, Clone)]
pub enum ScriptCommand {
    /// 执行脚本命令
    Execute {
        /// 脚本ID
        script_id: u64,
        /// 脚本代码
        code: String,
    },
    /// 调用函数命令
    CallFunction {
        /// 脚本ID
        script_id: u64,
        /// 函数名称
        function_name: String,
        /// 参数列表
        args: Vec<ScriptValue>,
    },
    /// 停止脚本命令
    Stop {
        /// 脚本ID
        script_id: u64,
    },
    /// 关闭脚本系统命令
    Shutdown,
}

/// 脚本值
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValue {
    /// 空值
    Null,
    /// 布尔值
    Bool(bool),
    /// 数值
    Number(f64),
    /// 字符串值
    String(String),
    /// 数组值
    Array(Vec<ScriptValue>),
    /// 对象值
    Object(HashMap<String, ScriptValue>),
}

/// 脚本结果
#[derive(Debug, Clone)]
pub enum ScriptResult {
    /// 成功结果
    Success {
        /// 脚本ID
        script_id: u64,
        /// 返回值
        value: ScriptValue,
    },
    /// 错误结果
    Error {
        /// 脚本ID
        script_id: u64,
        /// 错误消息
        message: String,
    },
}

/// 线程安全的脚本系统
pub struct ThreadSafeScriptSystem {
    /// 命令发送器
    command_sender: Sender<ScriptCommand>,
    /// 结果接收器
    result_receiver: Receiver<ScriptResult>,
    /// 脚本线程句柄
    script_thread: Option<thread::JoinHandle<()>>,
}

impl ThreadSafeScriptSystem {
    /// 创建新的线程安全脚本系统
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = channel::<ScriptCommand>();
        let (result_tx, result_rx) = channel::<ScriptResult>();

        // 启动脚本执行线程
        let script_thread = thread::spawn(move || {
            Self::script_thread_loop(cmd_rx, result_tx);
        });

        Self {
            command_sender: cmd_tx,
            result_receiver: result_rx,
            script_thread: Some(script_thread),
        }
    }

    /// 脚本线程主循环
    fn script_thread_loop(cmd_rx: Receiver<ScriptCommand>, result_tx: Sender<ScriptResult>) {
        let mut scripts: HashMap<u64, String> = HashMap::new();

        while let Ok(cmd) = cmd_rx.recv() {
            match cmd {
                ScriptCommand::Execute { script_id, code } => {
                    // 保存脚本
                    scripts.insert(script_id, code.clone());

                    // 执行脚本 (简化版)
                    let result = Self::execute_script(&code);
                    let _ = result_tx.send(ScriptResult::Success {
                        script_id,
                        value: result,
                    });
                }
                ScriptCommand::CallFunction {
                    script_id,
                    function_name,
                    args,
                } => {
                    // 调用函数 (简化版)
                    if let Some(_script) = scripts.get(&script_id) {
                        let result = Self::call_function_internal(&function_name, args);
                        let _ = result_tx.send(ScriptResult::Success {
                            script_id,
                            value: result,
                        });
                    } else {
                        let _ = result_tx.send(ScriptResult::Error {
                            script_id,
                            message: format!("Script {} not found", script_id),
                        });
                    }
                }
                ScriptCommand::Stop { script_id } => {
                    scripts.remove(&script_id);
                }
                ScriptCommand::Shutdown => {
                    break;
                }
            }
        }
    }

    /// 执行脚本 (简化版)
    fn execute_script(_code: &str) -> ScriptValue {
        // 实际实现需要集成脚本引擎
        // 这里返回一个模拟值
        ScriptValue::Null
    }

    /// 调用函数 (简化版)
    fn call_function_internal(_function_name: &str, _args: Vec<ScriptValue>) -> ScriptValue {
        // 实际实现需要集成脚本引擎
        // 这里返回一个模拟值
        ScriptValue::Null
    }

    /// 执行脚本
    pub fn execute(&self, script_id: u64, code: String) -> Result<(), String> {
        self.command_sender
            .send(ScriptCommand::Execute { script_id, code })
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// 调用函数
    pub fn call_function(
        &self,
        script_id: u64,
        function_name: String,
        args: Vec<ScriptValue>,
    ) -> Result<(), String> {
        self.command_sender
            .send(ScriptCommand::CallFunction {
                script_id,
                function_name,
                args,
            })
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// 停止脚本
    pub fn stop(&self, script_id: u64) -> Result<(), String> {
        self.command_sender
            .send(ScriptCommand::Stop { script_id })
            .map_err(|e| format!("Failed to send command: {}", e))
    }

    /// 获取所有结果
    pub fn poll_results(&self) -> Vec<ScriptResult> {
        let mut results = Vec::new();
        while let Ok(result) = self.result_receiver.try_recv() {
            results.push(result);
        }
        results
    }

    /// 关闭脚本系统
    pub fn shutdown(mut self) {
        let _ = self.command_sender.send(ScriptCommand::Shutdown);
        if let Some(handle) = self.script_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Default for ThreadSafeScriptSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_thread_safe_script_system() {
        let script_system = ThreadSafeScriptSystem::new();

        // 执行脚本
        script_system.execute(1, "print('Hello, World!')".to_string()).unwrap();

        // 等待结果
        thread::sleep(Duration::from_millis(100));

        // 获取结果
        let results = script_system.poll_results();
        assert!(results.len() > 0);

        // 关闭系统
        script_system.shutdown();
    }

    #[test]
    fn test_call_function() {
        let script_system = ThreadSafeScriptSystem::new();

        // 先执行脚本
        script_system
            .execute(1, "function add(a, b) { return a + b; }".to_string())
            .unwrap();

        // 调用函数
        script_system
            .call_function(
                1,
                "add".to_string(),
                vec![ScriptValue::Number(1.0), ScriptValue::Number(2.0)],
            )
            .unwrap();

        // 等待结果
        thread::sleep(Duration::from_millis(100));

        // 获取结果
        let results = script_system.poll_results();
        assert!(results.len() > 0);

        // 关闭系统
        script_system.shutdown();
    }
}
