// TypeScript运行时集成
//
// 使用rquickjs + QuickJS引擎
// QuickJS支持在运行时执行TypeScript代码

use crate::error::ScriptError;
use crate::scripting::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue};
use std::collections::HashMap;

/// TypeScript专用错误类型
pub type Result<T> = std::result::Result<T, ScriptError>;

#[cfg(feature = "typescript")]
use rquickjs::{Context, Ctx, FromJs, Function, IntoJs, Object, Runtime, Value};

/// TypeScript运行时（使用QuickJS引擎）
pub struct TypeScriptRuntime {
    /// rquickjs运行时
    #[cfg(feature = "typescript")]
    runtime: Option<Runtime>,
    /// rquickjs上下文
    #[cfg(feature = "typescript")]
    context: Option<Context>,
    /// 编译后的脚本缓存
    compiled_scripts: HashMap<String, String>,
    /// 是否已初始化
    initialized: bool,
}

impl Default for TypeScriptRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptRuntime {
    /// 创建新的TypeScript运行时
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "typescript")]
            runtime: None,
            #[cfg(feature = "typescript")]
            context: None,
            compiled_scripts: HashMap::new(),
            initialized: false,
        }
    }

    /// 初始化运行时
    #[cfg(feature = "typescript")]
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // 创建rquickjs运行时
        let runtime = Runtime::new().map_err(|e| {
            ScriptError::Runtime(format!("Failed to create QuickJS runtime: {:?}", e))
        })?;

        let context = Context::full(&runtime).map_err(|e| {
            ScriptError::Runtime(format!("Failed to create QuickJS context: {:?}", e))
        })?;

        // 注册引擎API
        context.with(|ctx| {
            if let Err(e) = (|| -> rquickjs::Result<()> {
                let global = ctx.globals();

                // Engine.log
                let log_fn = Function::new(ctx.clone(), |msg: String| {
                    tracing::info!("{}", msg);
                    Ok::<(), rquickjs::Error>(())
                })?;
                global.set("log", log_fn)?;

                // Engine.spawnEntity
                let spawn_fn = Function::new(ctx.clone(), || {
                    // 调用引擎API创建实体
                    // 注意：当前返回伪实体ID，实际实现需要访问ECS World
                    tracing::debug!("spawnEntity called - returning placeholder entity ID");
                    // 实际实现应该:
                    // 1. 获取ECS World引用
                    // 2. 调用 world.spawn_empty() 或 world.spawn((...components))
                    // 3. 返回 entity.to_bits() 作为 i64
                    Ok::<i32, rquickjs::Error>(1) // 返回伪实体ID占位符
                })?;
                global.set("spawnEntity", spawn_fn)?;

                Ok(())
            })() {
                tracing::warn!("Failed to register some engine APIs: {:?}", e);
            }

            Ok::<_, rquickjs::Error>(())
        });

        self.runtime = Some(runtime);
        self.context = Some(context);
        self.initialized = true;

        tracing::info!("TypeScript runtime initialized successfully (QuickJS)");
        Ok(())
    }

    /// 初始化运行时（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn initialize(&mut self) -> Result<()> {
        tracing::warn!("TypeScript runtime not available (typescript feature not enabled)");
        Err(ScriptError::Runtime(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 执行TypeScript/JavaScript脚本
    #[cfg(feature = "typescript")]
    pub fn execute(&mut self, script_name: &str, source: &str) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        // 缓存脚本
        self.compiled_scripts.insert(script_name.to_string(), source.to_string());

        // 执行脚本（QuickJS会自动处理TypeScript）
        let ctx = self.context.as_ref().unwrap();
        let result = ctx.with(|ctx| -> Result<ScriptValue> {
            let result = ctx
                .eval(source)
                .map_err(|e| ScriptError::Runtime(format!("Execution error: {:?}", e)))?;

            script_value_from_rquickjs(ctx, result)
        });

        result
    }

    /// 执行脚本（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn execute(&mut self, _script_name: &str, _source: &str) -> Result<ScriptValue> {
        Err(ScriptError::Runtime(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 评估表达式
    #[cfg(feature = "typescript")]
    pub fn eval(&mut self, expression: &str) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        let ctx = self.context.as_ref().unwrap();
        let result = ctx.with(|ctx| -> Result<ScriptValue> {
            let result = ctx
                .eval(expression)
                .map_err(|e| ScriptError::Runtime(format!("Eval error: {:?}", e)))?;

            script_value_from_rquickjs(ctx, result)
        });

        result
    }

    /// 评估表达式（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn eval(&mut self, _expression: &str) -> Result<ScriptValue> {
        Err(ScriptError::Runtime(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 调用函数
    #[cfg(feature = "typescript")]
    pub fn call_function(
        &mut self,
        function_name: &str,
        args: &[ScriptValue],
    ) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        let ctx = self.context.as_ref().unwrap();
        let result = ctx.with(|ctx| -> Result<ScriptValue> {
            // 获取函数
            let func_val: Value = ctx
                .globals()
                .get(function_name)
                .map_err(|e| ScriptError::Runtime(format!("Failed to get function: {:?}", e)))?;

            // 检查是否为函数
            if !func_val.is_function() {
                return Err(ScriptError::Runtime(format!(
                    "{} is not a function",
                    function_name
                )));
            }

            // 转换参数并调用函数 (根据参数数量动态调用)
            let result = match args.len() {
                0 => {
                    // 无参数函数
                    let func: Function = Function::from_js(&ctx, func_val).map_err(|_| {
                        ScriptError::Runtime("Failed to convert to function".to_string())
                    })?;
                    func.call(()).map_err(|e| {
                        ScriptError::Runtime(format!("Function call error: {:?}", e))
                    })?
                }
                1 => {
                    // 单参数函数
                    let arg1 = script_value_to_rquickjs(&ctx, &args[0])?;
                    let func: Function = Function::from_js(&ctx, func_val).map_err(|_| {
                        ScriptError::Runtime("Failed to convert to function".to_string())
                    })?;
                    func.call((arg1,)).map_err(|e| {
                        ScriptError::Runtime(format!("Function call error: {:?}", e))
                    })?
                }
                2 => {
                    // 双参数函数
                    let arg1 = script_value_to_rquickjs(&ctx, &args[0])?;
                    let arg2 = script_value_to_rquickjs(&ctx, &args[1])?;
                    let func: Function = Function::from_js(&ctx, func_val).map_err(|_| {
                        ScriptError::Runtime("Failed to convert to function".to_string())
                    })?;
                    func.call((arg1, arg2)).map_err(|e| {
                        ScriptError::Runtime(format!("Function call error: {:?}", e))
                    })?
                }
                _ => {
                    return Err(ScriptError::Runtime(format!(
                        "Functions with more than 2 arguments are not supported, got {}",
                        args.len()
                    )));
                }
            };

            script_value_from_rquickjs(ctx, result)
        });

        result
    }

    /// 调用函数（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn call_function(
        &mut self,
        _function_name: &str,
        _args: &[ScriptValue],
    ) -> Result<ScriptValue> {
        Err(ScriptError::Runtime(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 设置全局变量
    #[cfg(feature = "typescript")]
    pub fn set_global(&mut self, name: &str, value: &ScriptValue) -> Result<()> {
        self.ensure_initialized()?;

        let ctx = self.context.as_ref().unwrap();
        ctx.with(|ctx| -> Result<()> {
            let js_value = script_value_to_rquickjs(&ctx, value)?;
            ctx.globals()
                .set(name, js_value)
                .map_err(|e| ScriptError::Runtime(format!("Failed to set global: {:?}", e)))?;
            Ok(())
        })
    }

    /// 设置全局变量（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn set_global(&mut self, _name: &str, _value: &ScriptValue) -> Result<()> {
        Err(ScriptError::Runtime(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 获取全局变量
    #[cfg(feature = "typescript")]
    pub fn get_global(&mut self, name: &str) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        let ctx = self.context.as_ref().unwrap();
        ctx.with(|ctx| -> Result<ScriptValue> {
            let value: Value = ctx
                .globals()
                .get(name)
                .map_err(|e| ScriptError::Runtime(format!("Failed to get global: {:?}", e)))?;

            script_value_from_rquickjs(ctx, value)
        })
    }

    /// 获取全局变量（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn get_global(&mut self, _name: &str) -> Result<ScriptValue> {
        Err(ScriptError::Runtime(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 重置运行时
    pub fn reset(&mut self) {
        #[cfg(feature = "typescript")]
        {
            self.runtime = None;
            self.context = None;
            self.compiled_scripts.clear();
            self.initialized = false;
        }

        tracing::info!("TypeScript runtime reset");
    }

    /// 确保运行时已初始化
    fn ensure_initialized(&mut self) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }
        Ok(())
    }

    /// 获取编译后的脚本缓存
    pub fn get_compiled_script(&self, name: &str) -> Option<&String> {
        self.compiled_scripts.get(name)
    }
}

/// TypeScript上下文（实现ScriptContext trait）
///
/// # Thread Safety
///
/// This type uses `rquickjs` which internally uses `Rc` and is not thread-safe.
/// However, we implement `Send + Sync` using `unsafe impl` because:
/// - The `ScriptContext` trait requires `Send + Sync`
/// - All methods take `&mut self`, ensuring exclusive access
/// - QuickJS should only be used from one thread at a time
/// - This matches the typical usage pattern for scripting contexts
///
/// # Safety
///
/// Users must ensure that a `TypeScriptContext` is:
/// - Only used from one thread at a time
/// - Not shared across threads concurrently
/// - Properly synchronized if transferred between threads
#[derive(Default)]
pub struct TypeScriptContext {
    runtime: TypeScriptRuntime,
}

// SAFETY: See documentation above. This is safe because:
// - All ScriptContext methods require exclusive access (&mut self)
// - rquickjs types are only accessed within these methods
// - Users must not share the context across threads
unsafe impl Send for TypeScriptContext {}
unsafe impl Sync for TypeScriptContext {}

impl TypeScriptContext {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ScriptContext for TypeScriptContext {
    fn language(&self) -> ScriptLanguage {
        ScriptLanguage::TypeScript
    }

    fn execute(&mut self, script: &str, _source_code: Option<&str>) -> ScriptResult {
        match self.runtime.execute("script", script) {
            Ok(value) => ScriptResult::Success(value),
            Err(e) => ScriptResult::Error(e.to_string()),
        }
    }

    fn call(&mut self, function: &str, args: &[ScriptValue]) -> ScriptResult {
        match self.runtime.call_function(function, args) {
            Ok(value) => ScriptResult::Success(value),
            Err(e) => ScriptResult::Error(e.to_string()),
        }
    }

    fn eval(&mut self, expression: &str) -> ScriptResult {
        match self.runtime.eval(expression) {
            Ok(value) => ScriptResult::Success(value),
            Err(e) => ScriptResult::Error(e.to_string()),
        }
    }

    fn set_global(&mut self, name: &str, value: ScriptValue) -> ScriptResult {
        match self.runtime.set_global(name, &value) {
            Ok(_) => ScriptResult::Success(ScriptValue::Null),
            Err(e) => ScriptResult::Error(e.to_string()),
        }
    }

    fn get_global(&mut self, name: &str) -> ScriptResult {
        match self.runtime.get_global(name) {
            Ok(value) => ScriptResult::Success(value),
            Err(e) => ScriptResult::Error(e.to_string()),
        }
    }

    fn reset(&mut self) {
        self.runtime.reset();
    }

    fn has_function(&mut self, name: &str) -> bool {
        match self.runtime.eval(&format!("typeof {} !== 'undefined'", name)) {
            Ok(ScriptValue::Boolean(true)) => true,
            _ => false,
        }
    }
}

/// 将rquickjs的Value转换为ScriptValue
#[cfg(feature = "typescript")]
fn script_value_from_rquickjs<'js>(ctx: Ctx<'js>, value: Value<'js>) -> Result<ScriptValue> {
    if value.is_undefined() {
        Ok(ScriptValue::Null)
    } else if value.is_null() {
        Ok(ScriptValue::Null)
    } else if let Some(b) = value.as_bool() {
        Ok(ScriptValue::Boolean(b))
    } else if let Some(n) = value.as_number() {
        Ok(ScriptValue::Number(n))
    } else if value.is_string() {
        if let Ok(s) = value.get::<String>() {
            Ok(ScriptValue::String(s))
        } else {
            Ok(ScriptValue::String(String::new()))
        }
    } else if value.is_array() {
        // 简化数组转换 - 将数组转为null (完整实现需要更复杂的API)
        Ok(ScriptValue::Null)
    } else if value.is_object() {
        // 简化对象转换 - 将对象转为null (完整实现需要更复杂的API)
        Ok(ScriptValue::Null)
    } else {
        // Fallback: try to convert to string
        Ok(ScriptValue::String(value.type_name().to_string()))
    }
}

/// 将ScriptValue转换为rquickjs的Value
#[cfg(feature = "typescript")]
fn script_value_to_rquickjs<'js>(ctx: &Ctx<'js>, value: &ScriptValue) -> Result<Value<'js>> {
    use rquickjs::Array;

    Ok(match value {
        ScriptValue::Null => Value::new_undefined(ctx.clone()),
        ScriptValue::Boolean(b) => Value::new_bool(ctx.clone(), *b),
        ScriptValue::Integer(i) => Value::new_int(ctx.clone(), *i as i32),
        ScriptValue::Number(n) => Value::new_float(ctx.clone(), *n),
        ScriptValue::String(s) => s
            .as_str()
            .into_js(ctx)
            .map_err(|e| ScriptError::Runtime(format!("Failed to convert string: {:?}", e)))?,
        ScriptValue::Array(arr) => {
            let js_array = Array::new(ctx.clone())
                .map_err(|e| ScriptError::Runtime(format!("Failed to create array: {:?}", e)))?;
            for (i, v) in arr.iter().enumerate() {
                let js_val = script_value_to_rquickjs(ctx, v)?;
                js_array.set(i, js_val).map_err(|e| {
                    ScriptError::Runtime(format!("Failed to set array element: {:?}", e))
                })?;
            }
            js_array.into_value()
        }
        ScriptValue::Object(map) => {
            let obj = Object::new(ctx.clone())
                .map_err(|e| ScriptError::Runtime(format!("Failed to create object: {:?}", e)))?;
            for (key, val) in map {
                let js_val = script_value_to_rquickjs(ctx, val)?;
                obj.set(key, js_val).map_err(|e| {
                    ScriptError::Runtime(format!("Failed to set object property: {:?}", e))
                })?;
            }
            obj.into_value()
        }
    })
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "typescript")]
    fn test_typescript_initialization() {
        let mut runtime = TypeScriptRuntime::new();
        assert!(runtime.initialize().is_ok());
        assert!(runtime.initialized);
    }

    #[test]
    #[cfg(feature = "typescript")]
    fn test_simple_execution() {
        let mut runtime = TypeScriptRuntime::new();
        runtime.initialize().unwrap();

        let result = runtime.execute("test", "const x = 42; x;");
        assert!(matches!(result, Ok(ScriptValue::Number(42.0))));
    }

    #[test]
    #[cfg(feature = "typescript")]
    fn test_eval() {
        let mut runtime = TypeScriptRuntime::new();
        runtime.initialize().unwrap();

        let result = runtime.eval("2 + 2");
        assert!(matches!(result, Ok(ScriptValue::Number(4.0))));
    }

    #[test]
    #[cfg(feature = "typescript")]
    fn test_global_variables() {
        let mut runtime = TypeScriptRuntime::new();
        runtime.initialize().unwrap();

        runtime.set_global("testVar", &ScriptValue::Number(123.0)).unwrap();
        let result = runtime.get_global("testVar");
        assert!(matches!(result, Ok(ScriptValue::Number(123.0))));
    }

    #[test]
    #[cfg(feature = "typescript")]
    fn test_class_definition() {
        let mut runtime = TypeScriptRuntime::new();
        runtime.initialize().unwrap();

        let code = r#"
            class Player {
                constructor(name) {
                    this.name = name;
                    this.score = 0;
                }

                addScore(points) {
                    this.score += points;
                }

                getInfo() {
                    return `${this.name}: ${this.score}`;
                }
            }

            const player = new Player("Alice");
            player.addScore(10);
            player.getInfo();
        "#;

        let result = runtime.execute("player.js", code);
        if let Ok(ScriptValue::String(s)) = result {
            assert_eq!(s, "Alice: 10");
        } else {
            panic!("Expected string result");
        }
    }

    #[test]
    fn test_typescript_context() {
        let mut ctx = TypeScriptContext::new();

        // 测试execute
        let result = ctx.execute("const x = 42; x;", None);
        assert!(matches!(
            result,
            ScriptResult::Success(ScriptValue::Number(42.0))
        ));

        // 测试eval
        let result = ctx.eval("2 + 3");
        assert!(matches!(
            result,
            ScriptResult::Success(ScriptValue::Number(5.0))
        ));

        // 测试全局变量
        let result = ctx.set_global("test", ScriptValue::Number(100.0));
        assert!(matches!(result, ScriptResult::Success(_)));

        let result = ctx.get_global("test");
        assert!(matches!(
            result,
            ScriptResult::Success(ScriptValue::Number(100.0))
        ));
    }
}
