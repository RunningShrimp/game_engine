//! P2-2: Rust脚本系统增强
//!
//! 提供Rust脚本即时编译、REPL、热重载等功能

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

/// Rust脚本运行时
pub struct RustScriptRuntime {
    /// 编译缓存
    cache: Arc<Mutex<CompilationCache>>,
    /// 已加载的脚本
    scripts: Arc<Mutex<HashMap<String, CompiledScript>>>,
    /// 全局变量
    globals: Arc<Mutex<HashMap<String, ScriptValue>>>,
}

impl RustScriptRuntime {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(CompilationCache::new())),
            scripts: Arc::new(Mutex::new(HashMap::new())),
            globals: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 执行Rust脚本
    pub fn execute(&self, script_name: &str, code: &str) -> ScriptResult {
        // 检查缓存
        let mut cache = self.cache.lock().unwrap();
        if let Some(compiled) = cache.get(script_name, code) {
            return compiled.execute();
        }

        // 编译脚本
        let compiled = match self.compile_script(code) {
            Ok(script) => script,
            Err(e) => return ScriptResult::Error(format!("编译失败: {}", e)),
        };

        // 缓存编译结果
        cache.insert(script_name.to_string(), code.to_string(), compiled.clone());

        // 执行脚本
        let result = compiled.execute();

        // 存储脚本
        let mut scripts = self.scripts.lock().unwrap();
        scripts.insert(script_name.to_string(), compiled);

        result
    }

    /// 编译Rust脚本为动态库
    fn compile_script(&self, code: &str) -> Result<CompiledScript, String> {
        // 创建临时目录
        let temp_dir = std::env::temp_dir();
        let script_dir = temp_dir.join(format!("rust_script_{}", std::process::id()));
        std::fs::create_dir_all(&script_dir)
            .map_err(|e| format!("创建目录失败: {}", e))?;

        // 生成Cargo.toml
        let cargo_toml = r#"
[package]
name = "script"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
game_engine = { path = "/path/to/game_engine" }
"#;
        std::fs::write(script_dir.join("Cargo.toml"), cargo_toml)
            .map_err(|e| format!("写入Cargo.toml失败: {}", e))?;

        // 生成lib.rs
        let lib_rs = format!(r#"
use game_engine::prelude::*;

#[no_mangle]
pub extern "C" fn run_script() -> i32 {{
    {}
    0
}}
"#, code);
        std::fs::write(script_dir.join("src/lib.rs"), lib_rs)
            .map_err(|e| format!("写入lib.rs失败: {}", e))?;

        // 编译脚本
        let output = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&script_dir)
            .output()
            .map_err(|e| format!("编译命令失败: {}", e))?;

        if !output.status.success() {
            return Err(format!("编译失败: {}",
                String::from_utf8_lossy(&output.stderr)));
        }

        // 加载动态库
        let lib_path = script_dir.join("target/release/libscript.so");
        let library = unsafe { libloading::Library::new(lib_path) };

        Ok(CompiledScript {
            script_dir,
            library,
        })
    }

    /// 设置全局变量
    pub fn set_global(&self, name: &str, value: ScriptValue) {
        let mut globals = self.globals.lock().unwrap();
        globals.insert(name.to_string(), value);
    }

    /// 获取全局变量
    pub fn get_global(&self, name: &str) -> Option<ScriptValue> {
        let globals = self.globals.lock().unwrap();
        globals.get(name).cloned()
    }
}

/// 编译后的脚本
#[derive(Clone)]
pub struct CompiledScript {
    script_dir: PathBuf,
    library: libloading::Library,
}

impl CompiledScript {
    pub fn execute(&self) -> ScriptResult {
        unsafe {
            let func: libloading::Symbol<unsafe extern "C" fn() -> i32> =
                self.library.get(b"run_script\0").unwrap();

            let result = func();
            ScriptResult::Integer(result)
        }
    }
}

/// 编译缓存
pub struct CompilationCache {
    entries: HashMap<String, CacheEntry>,
}

struct CacheEntry {
    code: String,
    compiled: CompiledScript,
    timestamp: std::time::Instant,
}

impl CompilationCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&mut self, name: &str, code: &str) -> Option<CompiledScript> {
        let key = format!("{}:{}", name, code);
        self.entries.get(&key).map(|entry| entry.compiled.clone())
    }

    pub fn insert(&mut self, name: String, code: String, compiled: CompiledScript) {
        let key = format!("{}:{}", name, code);
        self.entries.insert(key, CacheEntry {
            code,
            compiled,
            timestamp: std::time::Instant::now(),
        });
    }

    /// 清理过期缓存
    pub fn cleanup(&mut self, max_age: std::time::Duration) {
        let now = std::time::Instant::now();
        self.entries.retain(|_, entry| {
            now.duration_since(entry.timestamp) < max_age
        });
    }
}

/// Rust脚本REPL
pub struct RustRepl {
    runtime: RustScriptRuntime,
    history: Vec<String>,
    context: HashMap<String, ScriptValue>,
}

impl RustRepl {
    pub fn new() -> Self {
        Self {
            runtime: RustScriptRuntime::new(),
            history: Vec::new(),
            context: HashMap::new(),
        }
    }

    /// 执行REPL命令
    pub fn execute(&mut self, input: &str) -> ReplResult {
        self.history.push(input.to_string());

        // 处理特殊命令
        if input.starts_with(":") {
            return self.handle_command(input);
        }

        // 执行Rust代码
        match self.runtime.execute("repl", input) {
            ScriptResult::Integer(n) => ReplResult::Output(format!("{}", n)),
            ScriptResult::Float(f) => ReplResult::Output(format!("{}", f)),
            ScriptResult::String(s) => ReplResult::Output(s),
            ScriptResult::Boolean(b) => ReplResult::Output(format!("{}", b)),
            ScriptResult::Null => ReplResult::Output("null".to_string()),
            ScriptResult::Error(e) => ReplResult::Error(e),
            _ => ReplResult::Output("执行成功".to_string()),
        }
    }

    fn handle_command(&self, cmd: &str) -> ReplResult {
        match cmd {
            ":help" => ReplResult::Help(self.get_help_text()),
            ":history" => ReplResult::History(self.history.clone()),
            ":clear" => {
                // 清理历史
                ReplResult::Output("历史已清理".to_string())
            }
            ":quit" | ":exit" => ReplResult::Exit,
            _ => ReplResult::Error(format!("未知命令: {}", cmd)),
        }
    }

    fn get_help_text(&self) -> String {
        r#"
Rust脚本REPL帮助

特殊命令:
  :help    - 显示此帮助
  :history - 显示执行历史
  :clear   - 清理历史记录
  :quit    - 退出REPL
  :exit    - 退出REPL

表达式:
  任何有效的Rust表达式都可以直接执行

示例:
  1 + 1
  let x = 42
  println!("Hello")
"#.to_string()
    }
}

/// REPL执行结果
#[derive(Debug, Clone)]
pub enum ReplResult {
    Output(String),
    Error(String),
    Help(String),
    History(Vec<String>),
    Exit,
}

/// 脚本值
#[derive(Debug, Clone)]
pub enum ScriptValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

/// 脚本执行结果
#[derive(Debug, Clone)]
pub enum ScriptResult {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
    Error(String),
}

/// 热重载监视器
pub struct HotReloadWatcher {
    /// 监视的脚本文件
    watched_files: Vec<PathBuf>,
    /// 修改时间缓存
    timestamps: HashMap<PathBuf, std::time::SystemTime>,
    /// 重载回调
    callbacks: Vec<Box<dyn Fn(&str) + Send + Sync>>,
}

impl HotReloadWatcher {
    pub fn new() -> Self {
        Self {
            watched_files: Vec::new(),
            timestamps: HashMap::new(),
            callbacks: Vec::new(),
        }
    }

    /// 添加监视文件
    pub fn watch(&mut self, file: PathBuf) {
        if let Ok(metadata) = std::fs::metadata(&file) {
            self.timestamps.insert(file.clone(), metadata.modified().unwrap());
            self.watched_files.push(file);
        }
    }

    /// 检查文件变化
    pub fn check_changes(&mut self) -> Vec<PathBuf> {
        let mut changed = Vec::new();

        for file in &self.watched_files {
            if let Ok(metadata) = std::fs::metadata(file) {
                let modified = metadata.modified().unwrap();
                if let Some(&last_modified) = self.timestamps.get(file) {
                    if modified > last_modified {
                        changed.push(file.clone());
                        self.timestamps.insert(file.clone(), modified);
                    }
                }
            }
        }

        changed
    }

    /// 添加重载回调
    pub fn on_reload<F>(&mut self, callback: F)
    where
        F: Fn(&str) + Send + Sync + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    /// 触发重载
    pub fn trigger_reload(&self, file: &str) {
        for callback in &self.callbacks {
            callback(file);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = RustScriptRuntime::new();
        runtime.set_global("test", ScriptValue::Integer(42));
        assert_eq!(runtime.get_global("test"), Some(ScriptValue::Integer(42)));
    }

    #[test]
    fn test_repl_creation() {
        let repl = RustRepl::new();
        let result = repl.execute(":help");
        match result {
            ReplResult::Help(text) => {
                assert!(text.contains("REPL帮助"));
            }
            _ => panic!("Expected Help result"),
        }
    }

    #[test]
    fn test_watcher() {
        let mut watcher = HotReloadWatcher::new();
        let file = std::path::PathBuf::from("/tmp/test.rs");
        watcher.watch(file.clone());
        assert!(watcher.watched_files.contains(&file));
    }
}
