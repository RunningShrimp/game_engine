use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// 脚本语言类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptLanguage {
    JavaScript,
    Python,
    Lua,
    CSharp,
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
#[derive(Debug, Clone)]
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
    
    /// 调用脚本函数
    pub fn call_function(&self, language: ScriptLanguage, name: &str, args: &[ScriptValue]) -> ScriptResult {
        let mut contexts = self.contexts.lock().unwrap();
        if let Some(context) = contexts.get_mut(&language) {
            context.call_function(name, args)
        } else {
            ScriptResult::Error(format!("No context registered for {:?}", language))
        }
    }
    
    /// 设置全局变量
    pub fn set_global(&self, language: ScriptLanguage, name: &str, value: ScriptValue) -> ScriptResult {
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
        Self::new()
    }
}

/// JavaScript上下文的简单实现 (占位)
pub struct JavaScriptContext {
    globals: HashMap<String, ScriptValue>,
}

impl JavaScriptContext {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
    }
}

impl ScriptContext for JavaScriptContext {
    fn execute(&mut self, code: &str) -> ScriptResult {
        // 占位实现
        ScriptResult::Success(format!("Executed: {}", code))
    }
    
    fn call_function(&mut self, name: &str, _args: &[ScriptValue]) -> ScriptResult {
        ScriptResult::Success(format!("Called function: {}", name))
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

impl Default for JavaScriptContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Python上下文的简单实现 (占位)
pub struct PythonContext {
    globals: HashMap<String, ScriptValue>,
}

impl PythonContext {
    pub fn new() -> Self {
        Self {
            globals: HashMap::new(),
        }
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

impl Default for PythonContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_script_system() {
        let system = ScriptSystem::new();
        
        // 注册JavaScript上下文
        system.register_context(ScriptLanguage::JavaScript, Box::new(JavaScriptContext::new()));
        
        // 执行脚本
        let result = system.execute(ScriptLanguage::JavaScript, "console.log('Hello')");
        assert!(matches!(result, ScriptResult::Success(_)));
        
        // 设置全局变量
        system.set_global(ScriptLanguage::JavaScript, "test", ScriptValue::Int(42));
        
        // 获取全局变量
        let value = system.get_global(ScriptLanguage::JavaScript, "test");
        assert!(matches!(value, Some(ScriptValue::Int(42))));
    }
}
