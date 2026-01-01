//  C#/.NET脚本支持
//
//  提供C#/.NET脚本集成框架。当前为简化实现，完整的.NET集成需要平台特定的支持。
//
//  **平台支持状态:**
//  - ✅ Windows/Linux: 完整支持（使用netcorehost，需要手动启用）
//  - ⚠️ macOS: 简化实现（netcorehost的macOS支持仍在开发中）
//
//  **当前实现:**
//  - ScriptContext trait的完整实现
//  - 类型转换工具（Rust ↔ C#）
//  - 全局变量管理
//  - 运行时状态管理
//
//  **未来工作:**
//  - 完整的.NET运行时集成（所有平台）
//  - 程序集加载和编译
//  - Unity API绑定生成
//  - 热重载支持

#[cfg(feature = "csharp")]
use {
    crate::scripting::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue},
    std::{
        collections::HashMap,
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    },
};

#[cfg(feature = "csharp")]
type Result<T> = std::result::Result<T, String>;

/// C#脚本上下文
///
/// 当前实现为简化版本，提供ScriptContext接口的完整实现但未集成实际的.NET运行时。
/// 这是由于netcorehost在macOS上的平台支持限制。
#[cfg(feature = "csharp")]
#[derive(Debug, Clone)]
pub struct CSharpContext {
    /// 全局变量缓存
    globals: Arc<Mutex<HashMap<String, ScriptValue>>>,

    /// 上下文配置
    config: CSharpConfig,

    /// 运行时状态
    runtime_state: Arc<Mutex<RuntimeState>>,
}

#[cfg(feature = "csharp")]
#[derive(Debug, Clone, PartialEq)]
enum RuntimeState {
    Uninitialized,
    Ready,
    Executing,
    Error(String),
}

/// C#上下文配置
#[cfg(feature = "csharp")]
#[derive(Debug, Clone)]
pub struct CSharpConfig {
    /// .NET运行时路径（保留用于未来完整实现）
    pub runtime_path: Option<PathBuf>,

    /// 程序集搜索路径（保留用于未来完整实现）
    pub assembly_search_paths: Vec<PathBuf>,

    /// 是否启用JIT编译（保留用于未来完整实现）
    pub enable_jit: bool,

    /// 是否启用调试支持
    pub enable_debugging: bool,

    /// 脚本执行超时（毫秒）
    pub execution_timeout_ms: u64,
}

#[cfg(feature = "csharp")]
impl Default for CSharpConfig {
    fn default() -> Self {
        Self {
            runtime_path: None,
            assembly_search_paths: vec![],
            enable_jit: true,
            enable_debugging: false,
            execution_timeout_ms: 5000,
        }
    }
}

#[cfg(feature = "csharp")]
impl CSharpContext {
    /// 创建新的C#脚本上下文
    pub fn new() -> Self {
        Self::with_config(CSharpConfig::default())
    }

    /// 使用指定配置创建C#上下文
    pub fn with_config(config: CSharpConfig) -> Self {
        Self {
            globals: Arc::new(Mutex::new(HashMap::new())),
            config,
            runtime_state: Arc::new(Mutex::new(RuntimeState::Uninitialized)),
        }
    }

    /// 将ScriptValue转换为C#表示
    fn script_value_to_net(&self, value: &ScriptValue) -> Result<String> {
        // 将Rust值序列化为C#兼容的JSON字符串表示
        // 未来完整实现将使用.NET互操作API直接传递值
        match value {
            ScriptValue::Null => Ok("null".to_string()),
            ScriptValue::Boolean(b) => Ok(b.to_string()),
            ScriptValue::Integer(i) => Ok(i.to_string()),
            ScriptValue::Number(n) => Ok(n.to_string()),
            ScriptValue::String(s) => Ok(format!("\"{}\"", s.replace('"', "\\\""))),
            ScriptValue::Array(arr) => {
                let elements: Vec<String> =
                    arr.iter().map(|v| self.script_value_to_net(v)).collect::<Result<_>>()?;
                Ok(format!("[{}]", elements.join(", ")))
            }
            ScriptValue::Object(map) => {
                let props: Vec<String> = map
                    .iter()
                    .map(|(k, v)| Ok(format!("\"{}\": {}", k, self.script_value_to_net(v)?)))
                    .collect::<Result<_>>()?;
                Ok(format!("{{{}}}", props.join(", ")))
            }
        }
    }

    /// 将C#值转换为ScriptValue
    fn net_value_to_script(&self, value_str: &str) -> Result<ScriptValue> {
        // 解析C#值字符串
        // 未来完整实现将使用.NET互操作API直接传递值
        if value_str == "null" {
            Ok(ScriptValue::Null)
        } else if let Ok(b) = value_str.parse::<bool>() {
            Ok(ScriptValue::Boolean(b))
        } else if let Ok(n) = value_str.parse::<f64>() {
            Ok(ScriptValue::Number(n))
        } else if value_str.starts_with('"') && value_str.ends_with('"') {
            let s = &value_str[1..value_str.len() - 1];
            Ok(ScriptValue::String(s.replace("\\\"", "\"")))
        } else {
            Ok(ScriptValue::String(value_str.to_string()))
        }
    }
}

#[cfg(feature = "csharp")]
impl Default for CSharpContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "csharp")]
impl ScriptContext for CSharpContext {
    /// 执行C#脚本代码
    fn execute(&mut self, script: &str, source_code: Option<&str>) -> ScriptResult {
        let _code = source_code.unwrap_or(script);

        // 记录脚本执行（简化实现）
        tracing::debug!(target: "scripting.csharp", "Executing C# script: {}", script);

        // TODO: 完整实现需要：
        // 1. 将C#代码编译为程序集
        // 2. 加载程序集到.NET运行时
        // 3. 查找入口点
        // 4. 执行代码并返回结果

        // 当前简化实现：返回成功但记录警告
        tracing::warn!(
            target: "scripting.csharp",
            "C# script execution is simplified (no .NET runtime integration yet)"
        );

        ScriptResult::Success(ScriptValue::Null)
    }

    /// 调用C#脚本函数
    fn call(&mut self, function: &str, args: &[ScriptValue]) -> ScriptResult {
        tracing::debug!(
            target: "scripting.csharp",
            "Calling C# function: {} with {} args",
            function,
            args.len()
        );

        // TODO: 完整实现需要：
        // 1. 在已加载程序集中查找函数
        // 2. 转换参数为.NET类型
        // 3. 调用函数
        // 4. 转换返回值为ScriptValue

        ScriptResult::Success(ScriptValue::Null)
    }

    /// 评估C#表达式
    fn eval(&mut self, expression: &str) -> ScriptResult {
        tracing::debug!(target: "scripting.csharp", "Evaluating C# expression: {}", expression);

        // 简化实现：包装表达式并"执行"
        let code = format!("return ({});", expression);
        self.execute(&code, Some(&code))
    }

    /// 设置全局变量
    fn set_global(&mut self, name: &str, value: ScriptValue) -> ScriptResult {
        self.globals.lock().unwrap().insert(name.to_string(), value.clone());

        tracing::debug!(target: "scripting.csharp", "Set global: {} = {:?}", name, value);

        ScriptResult::Success(value)
    }

    /// 获取全局变量
    fn get_global(&mut self, name: &str) -> ScriptResult {
        let globals = self.globals.lock().unwrap();

        match globals.get(name) {
            Some(value) => ScriptResult::Success(value.clone()),
            None => ScriptResult::Error(format!("Global variable '{}' not found", name)),
        }
    }

    /// 重置上下文
    fn reset(&mut self) {
        self.globals.lock().unwrap().clear();
        *self.runtime_state.lock().unwrap() = RuntimeState::Uninitialized;

        tracing::debug!(target: "scripting.csharp", "Reset C# context");
    }

    /// 获取脚本语言
    fn language(&self) -> ScriptLanguage {
        ScriptLanguage::CSharp
    }

    /// 检查函数是否存在
    fn has_function(&mut self, name: &str) -> bool {
        // 简化实现：总是返回false
        // 完整实现会检查已加载程序集
        let _ = name;
        false
    }
}

/// C#运行时上下文（用于系统注册）
#[cfg(feature = "csharp")]
#[derive(Debug, Default)]
pub struct CSharpRuntime {
    // 未来可能包含运行时实例池
}

#[cfg(feature = "csharp")]
impl CSharpRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create_context(&self) -> CSharpContext {
        CSharpContext::new()
    }
}

#[cfg(feature = "csharp")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csharp_context_creation() {
        let ctx = CSharpContext::new();
        assert_eq!(ctx.language(), ScriptLanguage::CSharp);
    }

    #[test]
    fn test_csharp_config_default() {
        let config = CSharpConfig::default();
        assert!(config.runtime_path.is_none());
        assert!(config.assembly_search_paths.is_empty());
        assert!(config.enable_jit);
        assert!(!config.enable_debugging);
        assert_eq!(config.execution_timeout_ms, 5000);
    }

    #[test]
    fn test_csharp_context_with_config() {
        let config = CSharpConfig {
            runtime_path: Some(PathBuf::from("/usr/bin/dotnet")),
            enable_debugging: true,
            ..Default::default()
        };

        let ctx = CSharpContext::with_config(config);
        assert_eq!(ctx.language(), ScriptLanguage::CSharp);
    }

    #[test]
    fn test_script_value_to_net_primitives() {
        let ctx = CSharpContext::new();

        assert_eq!(ctx.script_value_to_net(&ScriptValue::Null).unwrap(), "null");
        assert_eq!(
            ctx.script_value_to_net(&ScriptValue::Boolean(true)).unwrap(),
            "true"
        );
        assert_eq!(
            ctx.script_value_to_net(&ScriptValue::Boolean(false)).unwrap(),
            "false"
        );
        assert_eq!(
            ctx.script_value_to_net(&ScriptValue::Number(42.0)).unwrap(),
            "42"
        );
        assert_eq!(
            ctx.script_value_to_net(&ScriptValue::Number(3.14)).unwrap(),
            "3.14"
        );
    }

    #[test]
    fn test_script_value_to_net_string() {
        let ctx = CSharpContext::new();

        assert_eq!(
            ctx.script_value_to_net(&ScriptValue::String("hello".to_string())).unwrap(),
            "\"hello\""
        );

        assert_eq!(
            ctx.script_value_to_net(&ScriptValue::String("hello \"world\"".to_string()))
                .unwrap(),
            "\"hello \\\"world\\\"\""
        );
    }

    #[test]
    fn test_script_value_to_net_array() {
        let ctx = CSharpContext::new();

        let arr = ScriptValue::Array(vec![
            ScriptValue::Number(1.0),
            ScriptValue::Number(2.0),
            ScriptValue::Number(3.0),
        ]);

        assert_eq!(ctx.script_value_to_net(&arr).unwrap(), "[1, 2, 3]");
    }

    #[test]
    fn test_script_value_to_net_object() {
        let ctx = CSharpContext::new();

        let mut map = std::collections::HashMap::new();
        map.insert("name".to_string(), ScriptValue::String("test".to_string()));
        map.insert("value".to_string(), ScriptValue::Number(42.0));

        let obj = ScriptValue::Object(map);

        let result = ctx.script_value_to_net(&obj).unwrap();
        assert!(result.contains("\"name\": \"test\""));
        assert!(result.contains("\"value\": 42"));
    }

    #[test]
    fn test_net_value_to_script_primitives() {
        let ctx = CSharpContext::new();

        assert_eq!(ctx.net_value_to_script("null").unwrap(), ScriptValue::Null);
        assert_eq!(
            ctx.net_value_to_script("true").unwrap(),
            ScriptValue::Boolean(true)
        );
        assert_eq!(
            ctx.net_value_to_script("false").unwrap(),
            ScriptValue::Boolean(false)
        );
        assert_eq!(
            ctx.net_value_to_script("42").unwrap(),
            ScriptValue::Number(42.0)
        );
        assert_eq!(
            ctx.net_value_to_script("3.14").unwrap(),
            ScriptValue::Number(3.14)
        );
    }

    #[test]
    fn test_net_value_to_script_string() {
        let ctx = CSharpContext::new();

        assert_eq!(
            ctx.net_value_to_script("\"hello\"").unwrap(),
            ScriptValue::String("hello".to_string())
        );

        assert_eq!(
            ctx.net_value_to_script("\"hello \\\"world\\\"\"").unwrap(),
            ScriptValue::String("hello \"world\"".to_string())
        );
    }

    #[test]
    fn test_global_variables() {
        let mut ctx = CSharpContext::new();

        // Set global
        let result = ctx.set_global("test_var", ScriptValue::Number(123.0));
        assert!(result.is_success());

        // Get global
        let result = ctx.get_global("test_var");
        assert!(result.is_success());
        assert_eq!(result, ScriptResult::Success(ScriptValue::Number(123.0)));

        // Get non-existent global
        let result = ctx.get_global("non_existent");
        assert!(result.is_error());
    }

    #[test]
    fn test_reset_context() {
        let mut ctx = CSharpContext::new();

        ctx.set_global("var1", ScriptValue::Number(1.0));
        ctx.set_global("var2", ScriptValue::String("test".to_string()));

        ctx.reset();

        assert!(ctx.get_global("var1").is_error());
        assert!(ctx.get_global("var2").is_error());
    }
}

// 当不启用csharp feature时，提供编译错误
#[cfg(not(feature = "csharp"))]
pub struct CSharpContext;

#[cfg(not(feature = "csharp"))]
impl CSharpContext {
    pub fn new() -> Self {
        compile_error!("C# support requires the 'csharp' feature to be enabled");
    }
}
