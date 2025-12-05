//! Rust脚本引擎
//!
//! 提供Rust代码的动态编译和执行功能。

use super::system::{ScriptContext, ScriptResult, ScriptValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};


/// Rust脚本引擎
pub struct RustScriptEngine {
    context: Arc<Mutex<RustScriptContext>>,
}

/// Rust脚本上下文
#[derive(Default)]
pub struct RustScriptContext {
    /// 已编译的脚本
    compiled_scripts: HashMap<String, Box<dyn Fn() + Send + Sync>>,
}

impl RustScriptEngine {
    /// 创建新的Rust脚本引擎
    pub fn new() -> Self {
        Self {
            context: Arc::new(Mutex::new(RustScriptContext::default())),
        }
    }

    /// 执行Rust脚本
    pub fn execute_script(&mut self, script_name: &str, script_source: &str) -> Result<(), String> {
        // 注意：真正的Rust脚本执行需要动态编译，这在稳定版Rust中是不可能的
        // 这里提供一个模拟实现，实际项目中可能需要使用第三方库或WASM

        tracing::debug!(target: "rust_scripting", "Executing Rust script: {}", script_name);
        tracing::debug!(target: "rust_scripting", "Script source: {}", script_source);

        // 模拟脚本执行
        match script_name {
            "hello_world" => {
                tracing::info!(target: "rust_scripting", "Hello from Rust script!");
                Ok(())
            }
            "entity_script" => {
                tracing::info!(target: "rust_scripting", "Entity script executed");
                Ok(())
            }
            _ => {
                // 尝试解析简单的脚本命令
                self.execute_simple_script(script_source)
            }
        }
    }

    /// 执行简单的Rust脚本命令
    fn execute_simple_script(&self, script: &str) -> Result<(), String> {
        let lines: Vec<&str> = script.lines().map(|l| l.trim()).collect();

        for line in lines {
            if line.is_empty() || line.starts_with("//") {
                continue;
            }

            // 解析简单的函数调用
            if line.starts_with("println!(") && line.ends_with(");") {
                let content = &line[9..line.len() - 2];
                if content.starts_with('"') && content.ends_with('"') {
                    let message = &content[1..content.len() - 1];
                    tracing::info!(target: "rust_scripting", "{}", message);
                }
            } else if line == "return;" {
                break;
            }
            // 可以扩展更多简单的Rust语法支持
        }

        Ok(())
    }

    /// 注册Rust函数
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        if let Ok(mut context) = self.context.lock() {
            context
                .compiled_scripts
                .insert(name.to_string(), Box::new(func));
        }
    }

    /// 调用已注册的函数
    pub fn call_function(&self, name: &str) -> Result<(), String> {
        if let Ok(context) = self.context.lock() {
            if let Some(func) = context.compiled_scripts.get(name) {
                func();
                Ok(())
            } else {
                Err(format!("Function '{}' not found", name))
            }
        } else {
            Err("Failed to access script context".to_string())
        }
    }

    /// 更新引擎状态
    pub fn update(&mut self) {
        // 执行周期性任务
    }

    /// 获取已注册的脚本数量
    pub fn script_count(&self) -> usize {
        if let Ok(context) = self.context.lock() {
            context.compiled_scripts.len()
        } else {
            0
        }
    }
}


impl RustScriptContext {
    /// 添加预定义的脚本函数
    pub fn register_builtin_functions(&mut self) {
        // 这里可以注册一些内置的Rust函数
        // 实际实现中，这些函数需要是Send + Sync的
    }
}

/// Rust脚本上下文适配器 - 实现ScriptContext trait
///
/// 这个适配器将RustScriptEngine包装为ScriptContext trait对象，
/// 使其可以通过ScriptSystem统一管理
pub struct RustScriptContextAdapter {
    engine: Arc<Mutex<RustScriptEngine>>,
    globals: Arc<Mutex<HashMap<String, ScriptValue>>>,
}

impl RustScriptContextAdapter {
    /// 创建新的适配器
    pub fn new(engine: RustScriptEngine) -> Self {
        Self {
            engine: Arc::new(Mutex::new(engine)),
            globals: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ScriptContext for RustScriptContextAdapter {
    fn execute(&mut self, code: &str) -> ScriptResult {
        // 使用一个临时脚本名执行代码
        let script_name = format!("inline_{}", crate::core::utils::current_timestamp_nanos());

        if let Ok(mut engine) = self.engine.lock() {
            match engine.execute_script(&script_name, code) {
                Ok(_) => ScriptResult::Void,
                Err(e) => ScriptResult::Error(e),
            }
        } else {
            ScriptResult::Error("Failed to lock Rust script engine".to_string())
        }
    }

    fn call_function(&mut self, name: &str, _args: &[ScriptValue]) -> ScriptResult {
        if let Ok(engine) = self.engine.lock() {
            match engine.call_function(name) {
                Ok(_) => ScriptResult::Void,
                Err(e) => ScriptResult::Error(e),
            }
        } else {
            ScriptResult::Error("Failed to lock Rust script engine".to_string())
        }
    }

    fn set_global(&mut self, name: &str, value: ScriptValue) -> ScriptResult {
        if let Ok(mut globals) = self.globals.lock() {
            globals.insert(name.to_string(), value);
            ScriptResult::Void
        } else {
            ScriptResult::Error("Failed to lock globals".to_string())
        }
    }

    fn get_global(&self, name: &str) -> Option<ScriptValue> {
        if let Ok(globals) = self.globals.lock() {
            globals.get(name).cloned()
        } else {
            None
        }
    }

    fn reset(&mut self) {
        if let Ok(mut globals) = self.globals.lock() {
            globals.clear();
        }
        // Rust脚本引擎不需要重置，因为它没有全局状态
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_script_engine() {
        let mut engine = RustScriptEngine::new();

        // 测试简单脚本执行
        let result = engine.execute_script("test", "println!(\"Hello Rust!\");");
        assert!(result.is_ok());

        // 测试函数注册
        let call_count = Arc::new(Mutex::new(0));
        let call_count_clone = call_count.clone();

        engine.register_function("test_func", move || {
            let mut count = call_count_clone.lock().unwrap();
            *count += 1;
        });

        // 测试函数调用
        let result = engine.call_function("test_func");
        assert!(result.is_ok());
        assert_eq!(*call_count.lock().unwrap(), 1);
    }

    #[test]
    fn test_simple_script_parsing() {
        let engine = RustScriptEngine::new();

        // 测试简单的println!解析
        let script = r#"
// This is a comment
println!("Hello World!");
println!("Second line");
// Another comment
return;
println!("This should not print");
"#;

        let result = engine.execute_simple_script(script);
        assert!(result.is_ok());
    }
}
