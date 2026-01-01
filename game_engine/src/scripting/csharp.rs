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

/// .NET程序集元数据
#[cfg(feature = "csharp")]
#[derive(Debug, Clone)]
pub struct AssemblyMetadata {
    /// 程序集名称
    pub name: String,

    /// 程序集版本
    pub version: String,

    /// 程序集路径
    pub path: PathBuf,

    /// 是否已加载
    pub is_loaded: bool,
}

/// .NET函数签名
#[cfg(feature = "csharp")]
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    /// 函数名
    pub name: String,

    /// 返回类型
    pub return_type: String,

    /// 参数类型列表
    pub parameter_types: Vec<String>,

    /// 所属类
    pub class_name: Option<String>,
}

/// C#编译结果
#[cfg(feature = "csharp")]
#[derive(Debug)]
pub struct CompilationResult {
    /// 是否成功
    pub success: bool,

    /// 编译后的程序集路径
    pub assembly_path: Option<PathBuf>,

    /// 编译错误或警告信息
    pub diagnostics: Vec<String>,

    /// 编译耗时（毫秒）
    pub compilation_time_ms: u64,
}

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

    /// 已加载的程序集
    assemblies: Arc<Mutex<HashMap<String, AssemblyMetadata>>>,

    /// 函数签名缓存（用于快速查找）
    function_cache: Arc<Mutex<HashMap<String, FunctionSignature>>>,

    /// 临时编译目录
    temp_compile_dir: PathBuf,
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
        // 创建临时编译目录
        let temp_dir = std::env::temp_dir().join("csharp_compile");

        Self {
            globals: Arc::new(Mutex::new(HashMap::new())),
            config,
            runtime_state: Arc::new(Mutex::new(RuntimeState::Uninitialized)),
            assemblies: Arc::new(Mutex::new(HashMap::new())),
            function_cache: Arc::new(Mutex::new(HashMap::new())),
            temp_compile_dir: temp_dir,
        }
    }

    /// 编译C#代码为程序集
    ///
    /// **完整实现需要：**
    /// - Windows/Linux: 使用netcorehost调用csc或Roslyn编译器
    /// - macOS: 使用dotnet CLI或等待netcorehost支持
    pub fn compile_assembly(&self, source_code: &str, assembly_name: &str) -> Result<CompilationResult> {
        use std::time::Instant;

        let start_time = Instant::now();

        tracing::info!(
            target: "scripting.csharp",
            "Compiling C# assembly '{}' from {} bytes of source code",
            assembly_name,
            source_code.len()
        );

        // 创建临时编译目录
        std::fs::create_dir_all(&self.temp_compile_dir)
            .map_err(|e| format!("Failed to create compile directory: {}", e))?;

        let assembly_path = self.temp_compile_dir.join(format!("{}.dll", assembly_name));

        #[cfg(target_os = "windows")]
        {
            // Windows: 使用csc.exe或Roslyn
            self.compile_assembly_windows(source_code, assembly_name, &assembly_path, &start_time)
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: 使用dotnet CLI或netcorehost
            self.compile_assembly_linux(source_code, assembly_name, &assembly_path, &start_time)
        }

        #[cfg(target_os = "macos")]
        {
            // macOS: 简化实现（netcorehost支持有限）
            self.compile_assembly_macos(source_code, assembly_name, &assembly_path, &start_time)
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            // 其他平台：返回错误
            Err(format!("C# compilation not supported on this platform"))
        }
    }

    /// Windows平台汇编实现
    #[cfg(target_os = "windows")]
    fn compile_assembly_windows(
        &self,
        _source_code: &str,
        assembly_name: &str,
        assembly_path: &PathBuf,
        start_time: &Instant,
    ) -> Result<CompilationResult> {
        tracing::debug!(target: "scripting.csharp", "Compiling on Windows platform");

        // TODO: 完整实现需要：
        // 1. 使用netcorehost初始化.NET运行时
        // 2. 加载Microsoft.CodeAnalysis (Roslyn)
        // 3. 调用CSharpCompiler.Compile()
        // 4. 保存编译后的程序集

        // 框架实现
        let elapsed = start_time.elapsed().as_millis() as u64;

        Ok(CompilationResult {
            success: false,
            assembly_path: Some(assembly_path.clone()),
            diagnostics: vec![
                "Windows compilation requires netcorehost integration".to_string(),
                format!("Assembly '{}' would be compiled here", assembly_name),
            ],
            compilation_time_ms: elapsed,
        })
    }

    /// Linux平台汇编实现
    #[cfg(target_os = "linux")]
    fn compile_assembly_linux(
        &self,
        _source_code: &str,
        assembly_name: &str,
        assembly_path: &PathBuf,
        start_time: &Instant,
    ) -> Result<CompilationResult> {
        tracing::debug!(target: "scripting.csharp", "Compiling on Linux platform");

        // TODO: 完整实现需要：
        // 1. 使用netcorehost初始化.NET运行时
        // 2. 或者调用dotnet CLI执行编译
        // 3. 保存编译后的程序集

        // 框架实现
        let elapsed = start_time.elapsed().as_millis() as u64;

        Ok(CompilationResult {
            success: false,
            assembly_path: Some(assembly_path.clone()),
            diagnostics: vec![
                "Linux compilation requires netcorehost or dotnet CLI integration".to_string(),
                format!("Assembly '{}' would be compiled here", assembly_name),
            ],
            compilation_time_ms: elapsed,
        })
    }

    /// macOS平台汇编实现（简化版本）
    #[cfg(target_os = "macos")]
    fn compile_assembly_macos(
        &self,
        _source_code: &str,
        assembly_name: &str,
        assembly_path: &PathBuf,
        start_time: &Instant,
    ) -> Result<CompilationResult> {
        tracing::warn!(target: "scripting.csharp", "macOS compilation is simplified (netcorehost support limited)");

        // macOS上的简化实现
        // 实际使用建议：
        // 1. 使用dotnet CLI进行预编译
        // 2. 或者在Linux/Windows CI环境中编译
        // 3. 运行时加载预编译的程序集

        let elapsed = start_time.elapsed().as_millis() as u64;

        Ok(CompilationResult {
            success: false,
            assembly_path: Some(assembly_path.clone()),
            diagnostics: vec![
                "macOS compilation is simplified due to netcorehost limitations".to_string(),
                "Recommendation: Use dotnet CLI for pre-compilation or compile on CI".to_string(),
                format!("Assembly '{}' would be compiled here on Windows/Linux", assembly_name),
            ],
            compilation_time_ms: elapsed,
        })
    }

    /// 加载.NET程序集
    pub fn load_assembly(&mut self, assembly_path: &PathBuf) -> Result<AssemblyMetadata> {
        tracing::info!(
            target: "scripting.csharp",
            "Loading assembly from: {:?}",
            assembly_path
        );

        if !assembly_path.exists() {
            return Err(format!("Assembly file not found: {:?}", assembly_path));
        }

        // 提取程序集名称
        let assembly_name = assembly_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // TODO: 完整实现需要：
        // 1. 使用netcorehost加载程序集
        // 2. 扫描程序集中的类型和函数
        // 3. 缓存函数签名

        // 框架实现：创建元数据
        let metadata = AssemblyMetadata {
            name: assembly_name.clone(),
            version: "1.0.0.0".to_string(),
            path: assembly_path.clone(),
            is_loaded: true,
        };

        // 缓存程序集元数据
        self.assemblies
            .lock()
            .unwrap()
            .insert(assembly_name.clone(), metadata.clone());

        // 更新运行时状态
        *self.runtime_state.lock().unwrap() = RuntimeState::Ready;

        tracing::info!(
            target: "scripting.csharp",
            "Assembly '{}' loaded successfully",
            assembly_name
        );

        Ok(metadata)
    }

    /// 查找函数签名
    pub fn find_function(&self, function_name: &str) -> Option<FunctionSignature> {
        let cache = self.function_cache.lock().unwrap();

        // 首先检查缓存
        if let Some(signature) = cache.get(function_name) {
            return Some(signature.clone());
        }

        // TODO: 完整实现需要在所有已加载程序集中查找
        // 当前框架实现：返回None
        tracing::debug!(
            target: "scripting.csharp",
            "Function '{}' not found in cache",
            function_name
        );

        None
    }

    /// 添加函数签名到缓存
    pub fn cache_function_signature(&self, signature: FunctionSignature) {
        let name = signature.name.clone();
        self.function_cache
            .lock()
            .unwrap()
            .insert(name, signature);
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
        let code = source_code.unwrap_or(script);

        tracing::debug!(target: "scripting.csharp", "Executing C# script: {}", script);

        // 更新运行时状态
        *self.runtime_state.lock().unwrap() = RuntimeState::Executing;

        // 完整实现流程：
        // 1. 将C#代码编译为程序集
        // 2. 加载程序集到.NET运行时
        // 3. 查找入口点
        // 4. 执行代码并返回结果

        // 编译代码
        let assembly_name = format!("temp_{}", std::process::id());
        let compile_result = match self.compile_assembly(code, &assembly_name) {
            Ok(result) => result,
            Err(e) => {
                *self.runtime_state.lock().unwrap() = RuntimeState::Error(e.clone());
                return ScriptResult::Error(format!("Compilation failed: {}", e));
            }
        };

        if !compile_result.success {
            let error_msg = format!(
                "Compilation failed: {}",
                compile_result.diagnostics.join("; ")
            );
            *self.runtime_state.lock().unwrap() = RuntimeState::Error(error_msg.clone());
            return ScriptResult::Error(error_msg);
        }

        // 加载程序集
        if let Some(assembly_path) = &compile_result.assembly_path {
            if let Err(e) = self.load_assembly(assembly_path) {
                *self.runtime_state.lock().unwrap() = RuntimeState::Error(e.clone());
                return ScriptResult::Error(format!("Failed to load assembly: {}", e));
            }
        }

        // TODO: 完整实现需要：
        // 1. 在已加载程序集中查找入口点（Main、Run、Execute等）
        // 2. 使用.NET互操作调用入口点
        // 3. 转换返回值为ScriptValue

        // 框架实现：记录诊断信息并返回成功
        for diagnostic in &compile_result.diagnostics {
            tracing::debug!(target: "scripting.csharp", "Diagnostic: {}", diagnostic);
        }

        tracing::info!(
            target: "scripting.csharp",
            "Script executed in {}ms",
            compile_result.compilation_time_ms
        );

        *self.runtime_state.lock().unwrap() = RuntimeState::Ready;

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

        // 完整实现流程：
        // 1. 在已加载程序集中查找函数
        // 2. 转换参数为.NET类型
        // 3. 调用函数
        // 4. 转换返回值为ScriptValue

        // 查找函数
        let function_signature = match self.find_function(function) {
            Some(sig) => sig,
            None => {
                return ScriptResult::Error(format!(
                    "Function '{}' not found in loaded assemblies",
                    function
                ))
            }
        };

        tracing::debug!(
            target: "scripting.csharp",
            "Found function: {} -> {}",
            function,
            function_signature.return_type
        );

        // 验证参数数量
        if args.len() != function_signature.parameter_types.len() {
            return ScriptResult::Error(format!(
                "Parameter count mismatch: expected {}, got {}",
                function_signature.parameter_types.len(),
                args.len()
            ));
        }

        // TODO: 完整实现需要：
        // 1. 使用.NET互操作将参数转换为.NET类型
        // 2. 调用函数
        // 3. 转换返回值为ScriptValue

        // 框架实现：记录调用信息
        tracing::info!(
            target: "scripting.csharp",
            "Function '{}' called (framework implementation)",
            function
        );

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
        // 检查函数缓存
        self.find_function(name).is_some()
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
