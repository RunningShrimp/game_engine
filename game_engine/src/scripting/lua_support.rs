use super::system::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue};
use std::collections::HashMap;

/// Lua脚本上下文 (简化版)
#[derive(Default)]
pub struct LuaContext {
    /// 脚本存储
    scripts: HashMap<String, String>,
    /// 变量存储
    variables: HashMap<String, LuaValue>,
}

/// Lua值
#[derive(Debug, Clone, PartialEq)]
pub enum LuaValue {
    /// 空值
    Nil,
    /// 布尔值
    Boolean(bool),
    /// 数值
    Number(f64),
    /// 字符串值
    String(String),
    /// 表值
    Table(HashMap<String, LuaValue>),
}

impl LuaContext {
    /// 创建新的Lua上下文
    pub fn new() -> Self {
        Self {
            scripts: HashMap::new(),
            variables: HashMap::new(),
        }
    }

    /// 执行Lua脚本
    pub fn execute(&mut self, script_name: &str, code: &str) -> Result<LuaValue, String> {
        // 保存脚本
        self.scripts.insert(script_name.to_string(), code.to_string());

        // 实际实现需要集成mlua或rlua库
        // 这里返回一个模拟值
        Ok(LuaValue::Nil)
    }

    /// 调用Lua函数
    pub fn call_function(
        &mut self,
        function_name: &str,
        args: Vec<LuaValue>,
    ) -> Result<LuaValue, String> {
        // 实际实现需要集成mlua或rlua库
        // 这里返回一个模拟值
        let _ = (function_name, args);
        Ok(LuaValue::Nil)
    }

    /// 设置全局变量
    pub fn set_global(&mut self, name: &str, value: LuaValue) {
        self.variables.insert(name.to_string(), value);
    }

    /// 获取全局变量
    pub fn get_global(&self, name: &str) -> Option<&LuaValue> {
        self.variables.get(name)
    }

    /// 注册Rust函数到Lua
    pub fn register_function<F>(&mut self, name: &str, _func: F)
    where
        F: Fn(Vec<LuaValue>) -> Result<LuaValue, String> + 'static,
    {
        // 实际实现需要集成mlua或rlua库
        // 这里只是一个占位
        let _ = name;
    }
}

/// Lua脚本引擎
pub struct LuaEngine {
    /// Lua上下文
    pub context: LuaContext,
}

impl LuaEngine {
    /// 创建新的Lua引擎
    pub fn new() -> Self {
        Self {
            context: LuaContext::default(),
        }
    }

    /// 执行Lua脚本
    pub fn execute(&mut self, script_name: &str, code: &str) -> Result<LuaValue, String> {
        self.context.execute(script_name, code)
    }

    /// 调用Lua函数
    pub fn call_function(
        &mut self,
        function_name: &str,
        args: Vec<LuaValue>,
    ) -> Result<LuaValue, String> {
        self.context.call_function(function_name, args)
    }

    /// 注册引擎API到Lua
    pub fn register_engine_api(&mut self) {
        // 注册实体操作
        self.context.register_function("spawn_entity", |_args| {
            // 实际实现需要访问ECS World
            Ok(LuaValue::Number(0.0))
        });

        self.context.register_function("despawn_entity", |_args| {
            // 实际实现需要访问ECS World
            Ok(LuaValue::Nil)
        });

        // 注册组件操作
        self.context.register_function("add_component", |_args| {
            // 实际实现需要访问ECS World
            Ok(LuaValue::Nil)
        });

        self.context.register_function("get_component", |_args| {
            // 实际实现需要访问ECS World
            Ok(LuaValue::Nil)
        });

        // 注册输入操作
        self.context.register_function("is_key_pressed", |_args| {
            // 实际实现需要访问输入系统
            Ok(LuaValue::Boolean(false))
        });

        // 注册音频操作
        self.context.register_function("play_sound", |_args| {
            // 实际实现需要访问音频系统
            Ok(LuaValue::Nil)
        });
    }
}

impl Default for LuaEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ScriptContext trait 实现
// ============================================================================

impl ScriptContext for LuaContext {
    fn execute(&mut self, script: &str, _source_code: Option<&str>) -> ScriptResult {
        match self.execute("script", script) {
            Ok(value) => ScriptResult::Success(lua_value_to_script_value(&value)),
            Err(e) => ScriptResult::Error(e),
        }
    }

    fn call(&mut self, function: &str, args: &[ScriptValue]) -> ScriptResult {
        let lua_args: Vec<LuaValue> = args.iter().map(script_value_to_lua_value).collect();
        match LuaContext::call_function(self, function, lua_args) {
            Ok(value) => ScriptResult::Success(lua_value_to_script_value(&value)),
            Err(e) => ScriptResult::Error(e),
        }
    }

    fn eval(&mut self, expression: &str) -> ScriptResult {
        match self.execute("eval", expression) {
            Ok(value) => ScriptResult::Success(lua_value_to_script_value(&value)),
            Err(e) => ScriptResult::Error(e),
        }
    }

    fn set_global(&mut self, name: &str, value: ScriptValue) -> ScriptResult {
        LuaContext::set_global(self, name, script_value_to_lua_value(&value));
        ScriptResult::Void
    }

    fn get_global(&mut self, name: &str) -> ScriptResult {
        match LuaContext::get_global(self, name) {
            Some(value) => ScriptResult::Success(lua_value_to_script_value(value)),
            None => ScriptResult::Error(format!("Global '{name}' not found")),
        }
    }

    fn reset(&mut self) {
        self.scripts.clear();
        self.variables.clear();
    }

    fn language(&self) -> ScriptLanguage {
        ScriptLanguage::Lua
    }

    fn has_function(&mut self, name: &str) -> bool {
        matches!(
            self.eval(&format!("type({name}) == 'function'")),
            ScriptResult::Success(ScriptValue::Boolean(true))
        )
    }
}

/// 将 LuaValue 转换为 ScriptValue
pub fn lua_value_to_script_value(value: &LuaValue) -> ScriptValue {
    match value {
        LuaValue::Nil => ScriptValue::Null,
        LuaValue::Boolean(b) => ScriptValue::Boolean(*b),
        LuaValue::Number(n) => {
            // 判断是整数还是浮点数
            if n.fract() == 0.0 && *n >= (i64::MIN as f64) && *n <= (i64::MAX as f64) {
                ScriptValue::Integer(*n as i64)
            } else {
                ScriptValue::Number(*n)
            }
        }
        LuaValue::String(s) => ScriptValue::String(s.clone()),
        LuaValue::Table(t) => {
            let obj: HashMap<String, ScriptValue> =
                t.iter().map(|(k, v)| (k.clone(), lua_value_to_script_value(v))).collect();
            ScriptValue::Object(obj)
        }
    }
}

/// 将 ScriptValue 转换为 LuaValue
fn script_value_to_lua_value(value: &ScriptValue) -> LuaValue {
    match value {
        ScriptValue::Null => LuaValue::Nil,
        ScriptValue::Boolean(b) => LuaValue::Boolean(*b),
        ScriptValue::Integer(i) => LuaValue::Number(*i as f64),
        ScriptValue::Number(f) => LuaValue::Number(*f),
        ScriptValue::String(s) => LuaValue::String(s.clone()),
        ScriptValue::Array(arr) => {
            let table: HashMap<String, LuaValue> = arr
                .iter()
                .enumerate()
                .map(|(i, v)| (i.to_string(), script_value_to_lua_value(v)))
                .collect();
            LuaValue::Table(table)
        }
        ScriptValue::Object(obj) => {
            let table: HashMap<String, LuaValue> =
                obj.iter().map(|(k, v)| (k.clone(), script_value_to_lua_value(v))).collect();
            LuaValue::Table(table)
        }
    }
}

/// 将 ScriptValue 转换为 String 用于 ScriptResult::Success
fn script_value_to_string(value: &ScriptValue) -> String {
    match value {
        ScriptValue::Null => "null".to_string(),
        ScriptValue::Boolean(b) => b.to_string(),
        ScriptValue::Integer(i) => i.to_string(),
        ScriptValue::Number(f) => f.to_string(),
        ScriptValue::String(s) => s.clone(),
        ScriptValue::Array(_) | ScriptValue::Object(_) => format!("{value:?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_context() {
        let mut context = LuaContext::new();

        // 设置全局变量
        context.set_global("test_var", LuaValue::Number(42.0));

        // 获取全局变量
        let value = context.get_global("test_var");
        assert_eq!(value, Some(&LuaValue::Number(42.0)));
    }

    #[test]
    fn test_lua_engine() {
        let mut engine = LuaEngine::new();

        // 注册API
        engine.register_engine_api();

        // 执行脚本
        let result = engine.execute("test", "print('Hello from Lua!')");
        assert!(result.is_ok());
    }
}
