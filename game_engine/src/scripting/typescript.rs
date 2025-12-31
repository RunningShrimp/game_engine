// TypeScript运行时集成
//
// 提供TypeScript和JavaScript脚本执行能力，基于deno_core和swc

use crate::error::{Error, Result};
use crate::scripting::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(feature = "typescript")]
use deno_core::{
    Extension, ExtensionBuilder, FsModuleLoader, JsRuntime, ModuleCodeString, OpState,
    RuntimeOptions, op2,
};
#[cfg(feature = "typescript")]
use std::rc::Rc;
#[cfg(feature = "typescript")]
use swc_common::{FileName, SourceMap};
#[cfg(feature = "typescript")]
use swc_ecma_codegen::{Config, Emitter, text_writer::JsWriter};
#[cfg(feature = "typescript")]
use swc_ecma_parser::{Parser, StringInput, Syntax};

/// TypeScript运行时
#[derive(Debug)]
pub struct TypeScriptRuntime {
    /// Deno运行时
    #[cfg(feature = "typescript")]
    runtime: Option<JsRuntime>,
    /// 脚本缓存
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

        // 创建扩展
        let extension = Extension::builder("game_engine")
            .ops(vec![
                // 注册引擎API ops
                op_log::decl(),
                op_spawn_entity::decl(),
                op_get_entity::decl(),
                op_set_position::decl(),
                op_get_position::decl(),
                op_add_component::decl(),
                op_get_component::decl(),
            ])
            .build();

        // 创建运行时
        let mut runtime = JsRuntime::new(RuntimeOptions {
            extensions: vec![extension],
            module_loader: Some(Box::new(FsModuleLoader)),
            ..Default::default()
        });

        // 执行初始化脚本
        let init_script = r#"
            // 全局引擎对象
            globalThis.Engine = {
                spawnEntity: () => Deno.core.ops.op_spawn_entity(),
                getEntity: (id) => Deno.core.ops.op_get_entity(id),

                log: (msg) => Deno.core.ops.op_log(msg),
                warn: (msg) => console.warn(msg),
                error: (msg) => console.error(msg),
            };

            // Entity类
            globalThis.Entity = class {
                constructor(id) {
                    this.id = id;
                }

                setPosition(x, y, z) {
                    Deno.core.ops.op_set_position(this.id, x, y, z);
                }

                getPosition() {
                    return Deno.core.ops.op_get_position(this.id);
                }

                addComponent(type, data) {
                    Deno.core.ops.op_add_component(this.id, type, data);
                }

                getComponent(type) {
                    return Deno.core.ops.op_get_component(this.id, type);
                }
            };
        "#;

        runtime
            .execute_script("[init]", ModuleCodeString::from(init_script))
            .map_err(|e| {
                Error::ScriptingError(format!("Failed to initialize TypeScript runtime: {}", e))
            })?;

        self.runtime = Some(runtime);
        self.initialized = true;

        tracing::info!("TypeScript runtime initialized successfully");
        Ok(())
    }

    /// 初始化运行时（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn initialize(&mut self) -> Result<()> {
        tracing::warn!("TypeScript runtime not available (typescript feature not enabled)");
        Err(Error::ScriptingError(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 编译TypeScript代码为JavaScript
    #[cfg(feature = "typescript")]
    pub fn compile_typescript(&self, source: &str, filename: &str) -> Result<String> {
        let compiler = swc_common::SourceMap::default();
        let cm = Rc::new(compiler);

        // 创建解析器
        let mut parser = Parser::new(
            Syntax::Typescript(Default::default()),
            StringInput::new(source, FileName::Custom(filename.into()), false),
            None,
        );

        // 解析TypeScript
        let module = parser
            .parse_module()
            .map_err(|e| Error::ScriptingError(format!("TypeScript parse error: {:?}", e)))?;

        // 生成JavaScript代码
        let mut buf = Vec::new();
        {
            let writer = JsWriter::new(cm, "\n", &mut buf, None);
            let mut emitter = Emitter {
                cfg: Config::default().with_minify(false),
                cm: cm.clone(),
                comments: None,
                wr: writer,
            };
            emitter
                .emit_module(&module)
                .map_err(|e| Error::ScriptingError(format!("Codegen error: {:?}", e)))?;
        }

        let js_code = String::from_utf8(buf)
            .map_err(|e| Error::ScriptingError(format!("Invalid UTF-8: {}", e)))?;

        Ok(js_code)
    }

    /// 编译TypeScript代码（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn compile_typescript(&self, _source: &str, _filename: &str) -> Result<String> {
        Err(Error::ScriptingError(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 执行TypeScript/JavaScript脚本
    #[cfg(feature = "typescript")]
    pub fn execute(&mut self, script_name: &str, source: &str) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        // 编译TypeScript（如果需要）
        let js_code = if script_name.ends_with(".ts") || script_name.contains("typescript") {
            self.compile_typescript(source, script_name)?
        } else {
            source.to_string()
        };

        // 缓存编译后的代码
        self.compiled_scripts.insert(script_name.to_string(), js_code.clone());

        // 执行脚本
        let runtime = self.runtime.as_mut().unwrap();
        let result = runtime
            .execute_script(script_name, ModuleCodeString::from(js_code))
            .map_err(|e| Error::ScriptingError(format!("Execution error: {}", e)))?;

        // 转换返回值
        let serde_value = runtime
            .to_value(&result)
            .map_err(|e| Error::ScriptingError(format!("Value conversion error: {}", e)))?;

        Ok(script_value_from_denon(&serde_value))
    }

    /// 执行脚本（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn execute(&mut self, _script_name: &str, _source: &str) -> Result<ScriptValue> {
        Err(Error::ScriptingError(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 评估表达式
    #[cfg(feature = "typescript")]
    pub fn eval(&mut self, expression: &str) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        let runtime = self.runtime.as_mut().unwrap();
        let code = format!("({})", expression);

        let result = runtime
            .execute_script("[eval]", ModuleCodeString::from(code))
            .map_err(|e| Error::ScriptingError(format!("Eval error: {}", e)))?;

        let serde_value = runtime
            .to_value(&result)
            .map_err(|e| Error::ScriptingError(format!("Value conversion error: {}", e)))?;

        Ok(script_value_from_denon(&serde_value))
    }

    /// 评估表达式（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn eval(&mut self, _expression: &str) -> Result<ScriptValue> {
        Err(Error::ScriptingError(
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

        // 构建调用代码
        let args_json: Vec<String> =
            args.iter().map(|v| serde_json::to_string(v).unwrap_or_default()).collect();

        let call_code = if args.is_empty() {
            format!("{}()", function_name)
        } else {
            format!("{}(...{})", function_name, args_json.join(","))
        };

        self.eval(&call_code)
    }

    /// 调用函数（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn call_function(
        &mut self,
        _function_name: &str,
        _args: &[ScriptValue],
    ) -> Result<ScriptValue> {
        Err(Error::ScriptingError(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 设置全局变量
    #[cfg(feature = "typescript")]
    pub fn set_global(&mut self, name: &str, value: &ScriptValue) -> Result<()> {
        self.ensure_initialized()?;

        let runtime = self.runtime.as_mut().unwrap();
        let code = format!("globalThis.{} = {};", name, serde_json::to_string(value)?);

        runtime
            .execute_script("[set_global]", ModuleCodeString::from(code))
            .map_err(|e| Error::ScriptingError(format!("Failed to set global: {}", e)))?;

        Ok(())
    }

    /// 设置全局变量（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn set_global(&mut self, _name: &str, _value: &ScriptValue) -> Result<()> {
        Err(Error::ScriptingError(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 获取全局变量
    #[cfg(feature = "typescript")]
    pub fn get_global(&mut self, name: &str) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        let runtime = self.runtime.as_mut().unwrap();
        let code = format!("globalThis.{}", name);

        let result = runtime
            .execute_script("[get_global]", ModuleCodeString::from(code))
            .map_err(|e| Error::ScriptingError(format!("Failed to get global: {}", e)))?;

        let serde_value = runtime
            .to_value(&result)
            .map_err(|e| Error::ScriptingError(format!("Value conversion error: {}", e)))?;

        Ok(script_value_from_denon(&serde_value))
    }

    /// 获取全局变量（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn get_global(&mut self, _name: &str) -> Result<ScriptValue> {
        Err(Error::ScriptingError(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 添加模块
    #[cfg(feature = "typescript")]
    pub fn add_module(&mut self, name: &str, source: &str) -> Result<()> {
        self.ensure_initialized()?;

        let js_code = if name.ends_with(".ts") {
            self.compile_typescript(source, name)?
        } else {
            source.to_string()
        };

        let runtime = self.runtime.as_mut().unwrap();
        runtime
            .execute_script(name, ModuleCodeString::from(js_code))
            .map_err(|e| Error::ScriptingError(format!("Failed to add module: {}", e)))?;

        Ok(())
    }

    /// 添加模块（无feature gate版本）
    #[cfg(not(feature = "typescript"))]
    pub fn add_module(&mut self, _name: &str, _source: &str) -> Result<()> {
        Err(Error::ScriptingError(
            "TypeScript feature not enabled".to_string(),
        ))
    }

    /// 重置运行时
    pub fn reset(&mut self) {
        #[cfg(feature = "typescript")]
        {
            self.runtime = None;
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
#[derive(Debug, Default)]
pub struct TypeScriptContext {
    runtime: TypeScriptRuntime,
}

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

/// 将deno_core的serde值转换为ScriptValue
#[cfg(feature = "typescript")]
fn script_value_from_denon(value: &deno_core::serde_v8::Value) -> ScriptValue {
    use deno_core::serde_v8::Value;

    match value {
        Value::Null => ScriptValue::Null,
        Value::Boolean(b) => ScriptValue::Boolean(*b),
        Value::Number(n) => ScriptValue::Number(*n),
        Value::String(s) => ScriptValue::String(s.clone()),
        Value::Array(arr) => ScriptValue::Array(arr.iter().map(script_value_from_denon).collect()),
        Value::Object(obj) => {
            let mut map = HashMap::new();
            for (k, v) in obj.iter() {
                map.insert(k.clone(), script_value_from_denon(v));
            }
            ScriptValue::Object(map)
        }
        Value::Undefined => ScriptValue::Null,
    }
}

// ============================================================================
// Deno ops（引擎API绑定）
// ============================================================================

#[cfg(feature = "typescript")]
#[op2]
fn op_log(#[string] msg: String) {
    tracing::info!("{}", msg);
}

#[cfg(feature = "typescript")]
#[op2]
fn op_spawn_entity() -> u64 {
    // TODO: 调用引擎API创建实体
    // 暂时返回一个伪ID
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT_ID: AtomicU64 = AtomicU64::new(1);
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

#[cfg(feature = "typescript")]
#[op2]
fn op_get_entity(#[string] id: String) -> bool {
    // TODO: 检查实体是否存在
    id.parse::<u64>().is_ok()
}

#[cfg(feature = "typescript")]
#[op2]
fn op_set_position(#[string] id: String, x: f64, y: f64, z: f64) {
    // TODO: 设置实体位置
    tracing::debug!("Set position of entity {} to ({}, {}, {})", id, x, y, z);
}

#[cfg(feature = "typescript")]
#[op2]
fn op_get_position(#[string] id: String) -> Result<Vec<f64>, String> {
    // TODO: 获取实体位置
    Ok(vec![0.0, 0.0, 0.0])
}

#[cfg(feature = "typescript")]
#[op2]
fn op_add_component(
    #[string] id: String,
    #[string] component_type: String,
    #[string] data: String,
) {
    // TODO: 添加组件
    tracing::debug!(
        "Add component {} to entity {} with data: {}",
        component_type,
        id,
        data
    );
}

#[cfg(feature = "typescript")]
#[op2]
fn op_get_component(#[string] id: String, #[string] component_type: String) -> Option<String> {
    // TODO: 获取组件
    Some(r#"{"enabled": true}"#.to_string())
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
    fn test_typescript_compilation() {
        let mut runtime = TypeScriptRuntime::new();
        runtime.initialize().unwrap();

        let ts_code = r#"
            interface Vector3 {
                x: number;
                y: number;
                z: number;
            }

            const pos: Vector3 = { x: 1.0, y: 2.0, z: 3.0 };
            pos.x + pos.y + pos.z;
        "#;

        let result = runtime.execute("test.ts", ts_code);
        assert!(matches!(result, Ok(ScriptValue::Number(6.0))));
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
                public name: string;
                public score: number;

                constructor(name: string) {
                    this.name = name;
                    this.score = 0;
                }

                addScore(points: number): void {
                    this.score += points;
                }

                getInfo(): string {
                    return `${this.name}: ${this.score}`;
                }
            }

            const player = new Player("Alice");
            player.addScore(10);
            player.getInfo();
        "#;

        let result = runtime.execute("player.ts", code);
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
