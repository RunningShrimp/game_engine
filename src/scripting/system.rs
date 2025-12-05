use std::collections::HashMap;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

/// 脚本语言类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptLanguage {
    JavaScript,
    Python,
    Lua,
    CSharp,
    Rust,
}

/// 脚本执行结果
#[derive(Debug, Clone)]
pub enum ScriptResult {
    Success(String),
    Error(String),
    Void,
}

/// 脚本上下文 - 线程安全的脚本执行环境
pub trait ScriptContext: Send + Sync {
    /// 执行脚本代码
    fn execute(&mut self, code: &str) -> ScriptResult;

    /// 调用脚本函数
    fn call_function(&mut self, name: &str, args: &[ScriptValue]) -> ScriptResult;

    /// 设置全局变量
    fn set_global(&mut self, name: &str, value: ScriptValue) -> ScriptResult;

    /// 获取全局变量
    fn get_global(&self, name: &str) -> Option<ScriptValue>;

    /// 重置上下文
    fn reset(&mut self);
}

/// 脚本值 - 跨语言的通用数据类型
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<ScriptValue>),
    Object(HashMap<String, ScriptValue>),
}

/// 脚本系统 - 管理多个脚本上下文
pub struct ScriptSystem {
    contexts: Arc<Mutex<HashMap<ScriptLanguage, Box<dyn ScriptContext>>>>,
}

impl ScriptSystem {
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 注册脚本上下文
    pub fn register_context(&self, language: ScriptLanguage, context: Box<dyn ScriptContext>) {
        let mut contexts = self.contexts.lock().unwrap();
        contexts.insert(language, context);
    }

    /// 执行脚本
    pub fn execute(&self, language: ScriptLanguage, code: &str) -> ScriptResult {
        let mut contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get_mut(&language) {
            context.execute(code)
        } else {
            ScriptResult::Error(format!("No context registered for {:?}", language))
        }
    }

    /// 执行脚本 (便利方法)
    pub fn execute_script(
        &self,
        _name: &str,
        code: &str,
        language: ScriptLanguage,
    ) -> ScriptResult {
        self.execute(language, code)
    }

    /// 调用脚本函数
    pub fn call_function(
        &self,
        language: ScriptLanguage,
        name: &str,
        args: &[ScriptValue],
    ) -> ScriptResult {
        let mut contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get_mut(&language) {
            context.call_function(name, args)
        } else {
            ScriptResult::Error(format!("No context registered for {:?}", language))
        }
    }

    /// 设置全局变量
    pub fn set_global(
        &self,
        language: ScriptLanguage,
        name: &str,
        value: ScriptValue,
    ) -> ScriptResult {
        let mut contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get_mut(&language) {
            context.set_global(name, value)
        } else {
            ScriptResult::Error(format!("No context registered for {:?}", language))
        }
    }

    /// 获取全局变量
    pub fn get_global(&self, language: ScriptLanguage, name: &str) -> Option<ScriptValue> {
        let contexts = self.contexts.lock().unwrap();
        contexts.get(&language).and_then(|ctx| ctx.get_global(name))
    }
}

impl Default for ScriptSystem {
    fn default() -> Self {
        Self {
            contexts: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

/// JavaScript执行请求
enum JsCommand {
    Execute(String, mpsc::Sender<ScriptResult>),
    CallFunction(String, Vec<ScriptValue>, mpsc::Sender<ScriptResult>),
    SetGlobal(String, ScriptValue, mpsc::Sender<ScriptResult>),
    GetGlobal(String, mpsc::Sender<Option<ScriptValue>>),
    Reset(mpsc::Sender<()>),
    Shutdown,
}

/// JavaScript上下文 - 基于rquickjs的线程安全实现
///
/// 使用QuickJS引擎在专用线程中执行JavaScript代码，
/// 通过channel通信保证线程安全
pub struct JavaScriptContext {
    sender: mpsc::Sender<JsCommand>,
    globals_cache: Arc<Mutex<HashMap<String, ScriptValue>>>,
}

impl JavaScriptContext {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<JsCommand>();
        let globals_cache = Arc::new(Mutex::new(HashMap::new()));
        let globals_clone = Arc::clone(&globals_cache);

        // 在专用线程中运行QuickJS
        thread::spawn(move || {
            use rquickjs::{Context, Function, Object, Runtime};

            let runtime = match Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    tracing::error!(target: "scripting", "Failed to create QuickJS runtime: {:?}", e);
                    return;
                }
            };
            let context = match Context::full(&runtime) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!(target: "scripting", "Failed to create QuickJS context: {:?}", e);
                    return;
                }
            };

            // 绑定基础API
            context.with(|ctx| {
                let global = ctx.globals();

                // 创建Engine命名空间
                if let Ok(engine_obj) = Object::new(ctx.clone()) {
                    let _ = engine_obj.set(
                        "log",
                        Function::new(ctx.clone(), |msg: String| {
                            tracing::info!(target: "scripting", "[JS]: {}", msg);
                        }),
                    );
                    let _ = engine_obj.set(
                        "time",
                        Function::new(ctx.clone(), || -> f64 {
                            crate::core::utils::current_timestamp_f64()
                        }),
                    );
                    let _ = global.set("Engine", engine_obj);
                }

                // 全局print函数
                let _ = global.set(
                    "print",
                    Function::new(ctx.clone(), |msg: String| {
                        tracing::info!(target: "scripting", "[JS]: {}", msg);
                    }),
                );

                // console对象
                if let Ok(console_obj) = Object::new(ctx.clone()) {
                    let _ = console_obj.set(
                        "log",
                        Function::new(ctx.clone(), |msg: String| {
                            tracing::info!(target: "scripting", "[JS console]: {}", msg);
                        }),
                    );
                    let _ = console_obj.set(
                        "warn",
                        Function::new(ctx.clone(), |msg: String| {
                            tracing::warn!(target: "scripting", "[JS warn]: {}", msg);
                        }),
                    );
                    let _ = console_obj.set(
                        "error",
                        Function::new(ctx.clone(), |msg: String| {
                            tracing::error!(target: "scripting", "[JS error]: {}", msg);
                        }),
                    );
                    let _ = global.set("console", console_obj);
                }
            });

            // 消息循环
            while let Ok(cmd) = rx.recv() {
                match cmd {
                    JsCommand::Execute(code, response) => {
                        let mut result = ScriptResult::Void;
                        context.with(|ctx| match ctx.eval::<rquickjs::Value, _>(code.as_str()) {
                            Ok(value) => {
                                if value.is_undefined() || value.is_null() {
                                    result = ScriptResult::Void;
                                } else if let Ok(s) = value.get::<String>() {
                                    result = ScriptResult::Success(s);
                                } else if let Ok(n) = value.get::<f64>() {
                                    result = ScriptResult::Success(n.to_string());
                                } else if let Ok(b) = value.get::<bool>() {
                                    result = ScriptResult::Success(b.to_string());
                                } else {
                                    result = ScriptResult::Success("[object]".to_string());
                                }
                            }
                            Err(e) => {
                                result = ScriptResult::Error(format!("{:?}", e));
                            }
                        });
                        let _ = response.send(result);
                    }
                    JsCommand::CallFunction(name, args, response) => {
                        let args_json = args
                            .iter()
                            .map(|v| match v {
                                ScriptValue::Null => "null".to_string(),
                                ScriptValue::Bool(b) => b.to_string(),
                                ScriptValue::Int(i) => i.to_string(),
                                ScriptValue::Float(f) => f.to_string(),
                                ScriptValue::String(s) => {
                                    format!("\"{}\"", s.replace('\"', "\\\""))
                                }
                                ScriptValue::Array(_) => "[]".to_string(),
                                ScriptValue::Object(_) => "{}".to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(", ");

                        let call_code = format!("{}({})", name, args_json);
                        let mut result = ScriptResult::Void;
                        context.with(|ctx| {
                            match ctx.eval::<rquickjs::Value, _>(call_code.as_str()) {
                                Ok(value) => {
                                    if value.is_undefined() || value.is_null() {
                                        result = ScriptResult::Void;
                                    } else if let Ok(s) = value.get::<String>() {
                                        result = ScriptResult::Success(s);
                                    } else if let Ok(n) = value.get::<f64>() {
                                        result = ScriptResult::Success(n.to_string());
                                    } else {
                                        result = ScriptResult::Success("[object]".to_string());
                                    }
                                }
                                Err(e) => {
                                    result = ScriptResult::Error(format!("{:?}", e));
                                }
                            }
                        });
                        let _ = response.send(result);
                    }
                    JsCommand::SetGlobal(name, value, response) => {
                        let js_value = match &value {
                            ScriptValue::Null => "null".to_string(),
                            ScriptValue::Bool(b) => b.to_string(),
                            ScriptValue::Int(i) => i.to_string(),
                            ScriptValue::Float(f) => f.to_string(),
                            ScriptValue::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
                            ScriptValue::Array(_) => "[]".to_string(),
                            ScriptValue::Object(_) => "{}".to_string(),
                        };
                        let set_code = format!("globalThis.{} = {}", name, js_value);

                        let mut result = ScriptResult::Void;
                        context.with(|ctx| {
                            if let Err(e) = ctx.eval::<(), _>(set_code.as_str()) {
                                result = ScriptResult::Error(format!("{:?}", e));
                            }
                        });

                        // 更新本地缓存
                        globals_clone.lock().unwrap().insert(name, value);
                        let _ = response.send(result);
                    }
                    JsCommand::GetGlobal(name, response) => {
                        let value = globals_clone.lock().unwrap().get(&name).cloned();
                        let _ = response.send(value);
                    }
                    JsCommand::Reset(response) => {
                        globals_clone.lock().unwrap().clear();
                        let _ = response.send(());
                    }
                    JsCommand::Shutdown => break,
                }
            }
        });

        Self {
            sender: tx,
            globals_cache,
        }
    }
}

impl ScriptContext for JavaScriptContext {
    fn execute(&mut self, code: &str) -> ScriptResult {
        let (tx, rx) = mpsc::channel();
        if self
            .sender
            .send(JsCommand::Execute(code.to_string(), tx))
            .is_ok()
        {
            rx.recv()
                .unwrap_or(ScriptResult::Error("Channel closed".to_string()))
        } else {
            ScriptResult::Error("Failed to send command".to_string())
        }
    }

    fn call_function(&mut self, name: &str, args: &[ScriptValue]) -> ScriptResult {
        let (tx, rx) = mpsc::channel();
        if self
            .sender
            .send(JsCommand::CallFunction(name.to_string(), args.to_vec(), tx))
            .is_ok()
        {
            rx.recv()
                .unwrap_or(ScriptResult::Error("Channel closed".to_string()))
        } else {
            ScriptResult::Error("Failed to send command".to_string())
        }
    }

    fn set_global(&mut self, name: &str, value: ScriptValue) -> ScriptResult {
        let (tx, rx) = mpsc::channel();
        if self
            .sender
            .send(JsCommand::SetGlobal(name.to_string(), value, tx))
            .is_ok()
        {
            rx.recv()
                .unwrap_or(ScriptResult::Error("Channel closed".to_string()))
        } else {
            ScriptResult::Error("Failed to send command".to_string())
        }
    }

    fn get_global(&self, name: &str) -> Option<ScriptValue> {
        self.globals_cache.lock().unwrap().get(name).cloned()
    }

    fn reset(&mut self) {
        let (tx, rx) = mpsc::channel();
        if self.sender.send(JsCommand::Reset(tx)).is_ok() {
            let _ = rx.recv();
        }
    }
}

impl Default for JavaScriptContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Python上下文的简单实现 (占位)
#[derive(Default)]
pub struct PythonContext {
    globals: HashMap<String, ScriptValue>,
}

impl PythonContext {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ScriptContext for PythonContext {
    fn execute(&mut self, code: &str) -> ScriptResult {
        // 占位实现
        ScriptResult::Success(format!("Executed Python: {}", code))
    }

    fn call_function(&mut self, name: &str, _args: &[ScriptValue]) -> ScriptResult {
        ScriptResult::Success(format!("Called Python function: {}", name))
    }

    fn set_global(&mut self, name: &str, value: ScriptValue) -> ScriptResult {
        self.globals.insert(name.to_string(), value);
        ScriptResult::Void
    }

    fn get_global(&self, name: &str) -> Option<ScriptValue> {
        self.globals.get(name).cloned()
    }

    fn reset(&mut self) {
        self.globals.clear();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_system() {
        let system = ScriptSystem::new();

        // 注册JavaScript上下文
        system.register_context(
            ScriptLanguage::JavaScript,
            Box::new(JavaScriptContext::new()),
        );

        // 执行脚本 - console.log返回undefined, 所以结果是Void
        let result = system.execute(ScriptLanguage::JavaScript, "console.log('Hello')");
        assert!(
            matches!(result, ScriptResult::Void),
            "Expected Void, got {:?}",
            result
        );

        // 执行有返回值的脚本
        let result = system.execute(ScriptLanguage::JavaScript, "1 + 2");
        assert!(
            matches!(result, ScriptResult::Success(ref s) if s == "3"),
            "Expected '3', got {:?}",
            result
        );

        // 执行字符串运算
        let result = system.execute(ScriptLanguage::JavaScript, "'Hello' + ' World'");
        assert!(
            matches!(result, ScriptResult::Success(ref s) if s == "Hello World"),
            "Expected 'Hello World', got {:?}",
            result
        );

        // 设置全局变量
        system.set_global(ScriptLanguage::JavaScript, "test", ScriptValue::Int(42));

        // 获取全局变量
        let value = system.get_global(ScriptLanguage::JavaScript, "test");
        assert!(matches!(value, Some(ScriptValue::Int(42))));

        // 使用引擎API
        let result = system.execute(ScriptLanguage::JavaScript, "Engine.time()");
        assert!(
            matches!(result, ScriptResult::Success(_)),
            "Engine.time() should return timestamp"
        );
    }

    #[test]
    fn test_script_system_rust_language() {
        use super::super::rust_scripting::RustScriptContextAdapter;

        let system = ScriptSystem::new();

        // 注册Rust脚本上下文适配器
        let rust_engine = super::super::rust_scripting::RustScriptEngine::new();
        system.register_context(
            ScriptLanguage::Rust,
            Box::new(RustScriptContextAdapter::new(rust_engine)),
        );

        // 测试Rust脚本执行
        let result = system.execute(ScriptLanguage::Rust, "println!(\"Hello Rust!\");");
        // Rust脚本执行应该成功（返回Void或Success）
        assert!(
            matches!(result, ScriptResult::Void | ScriptResult::Success(_)),
            "Expected Void or Success, got {:?}",
            result
        );

        // 测试未注册的语言
        let result = system.execute(ScriptLanguage::CSharp, "test");
        assert!(
            matches!(result, ScriptResult::Error(_)),
            "Expected Error for unregistered language, got {:?}",
            result
        );
    }
}
