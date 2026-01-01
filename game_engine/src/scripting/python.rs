// Python脚本绑定
//
// 提供Python脚本集成，基于pyo3

use crate::error::ScriptError;
use crate::scripting::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue};
use std::collections::HashMap;
use std::sync::Arc;

/// Python结果的便捷别名
pub type Result<T> = std::result::Result<T, ScriptError>;

#[cfg(feature = "pyo3")]
use pyo3::{
    Python,
    prelude::*,
    types::{PyDict, PyList, PyString},
    wrap_pyfunction,
};

/// Python运行时
#[derive(Debug)]
pub struct PythonRuntime {
    /// 是否已初始化
    initialized: bool,
    /// 全局变量缓存
    globals: HashMap<String, ScriptValue>,
}

impl Default for PythonRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonRuntime {
    /// 创建新的Python运行时
    pub fn new() -> Self {
        Self {
            initialized: false,
            globals: HashMap::new(),
        }
    }

    /// 初始化运行时
    #[cfg(feature = "pyo3")]
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        // 初始化Python解释器
        pyo3::prepare_freethreaded_python();

        tracing::info!("Python runtime initialized successfully");
        self.initialized = true;
        Ok(())
    }

    /// 初始化运行时（无feature gate版本）
    #[cfg(not(feature = "pyo3"))]
    pub fn initialize(&mut self) -> Result<()> {
        tracing::warn!("Python runtime not available (pyo3 feature not enabled)");
        Err(ScriptError::ExecutionError(
            "Python feature not enabled".to_string(),
        ))
    }

    /// 执行Python脚本
    #[cfg(feature = "pyo3")]
    pub fn execute(&mut self, script: &str) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        Python::with_gil(|py| {
            // 执行脚本
            let result = py.run_bound(script, None, None);

            match result {
                Ok(_) => Ok(ScriptValue::Null),
                Err(e) => Err(ScriptError::ExecutionError(format!(
                    "Python execution error: {}",
                    e
                ))),
            }
        })
    }

    /// 执行脚本（无feature gate版本）
    #[cfg(not(feature = "pyo3"))]
    pub fn execute(&mut self, _script: &str) -> Result<ScriptValue> {
        Err(ScriptError::ExecutionError(
            "Python feature not enabled".to_string(),
        ))
    }

    /// 评估Python表达式
    #[cfg(feature = "pyo3")]
    pub fn eval(&mut self, expression: &str) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        Python::with_gil(|py| match py.eval_bound(expression, None, None) {
            Ok(value) => Ok(python_value_to_script_value(py, &value)),
            Err(e) => Err(ScriptError::ExecutionError(format!(
                "Python eval error: {}",
                e
            ))),
        })
    }

    /// 评估表达式（无feature gate版本）
    #[cfg(not(feature = "pyo3"))]
    pub fn eval(&mut self, _expression: &str) -> Result<ScriptValue> {
        Err(ScriptError::ExecutionError(
            "Python feature not enabled".to_string(),
        ))
    }

    /// 调用Python函数
    #[cfg(feature = "pyo3")]
    pub fn call_function(
        &mut self,
        function_name: &str,
        args: &[ScriptValue],
    ) -> Result<ScriptValue> {
        self.ensure_initialized()?;

        Python::with_gil(|py| {
            // 构建调用代码
            let args_str: Vec<String> =
                args.iter().map(|v| script_value_to_python_code(v)).collect();

            let call_code = if args.is_empty() {
                format!("{}()", function_name)
            } else {
                format!("{}({})", function_name, args_str.join(", "))
            };

            match py.eval_bound(&call_code, None, None) {
                Ok(value) => Ok(python_value_to_script_value(py, &value)),
                Err(e) => Err(ScriptError::ExecutionError(format!(
                    "Python function call error: {}",
                    e
                ))),
            }
        })
    }

    /// 调用函数（无feature gate版本）
    #[cfg(not(feature = "pyo3"))]
    pub fn call_function(
        &mut self,
        _function_name: &str,
        _args: &[ScriptValue],
    ) -> Result<ScriptValue> {
        Err(ScriptError::ExecutionError(
            "Python feature not enabled".to_string(),
        ))
    }

    /// 设置全局变量
    #[cfg(feature = "pyo3")]
    pub fn set_global(&mut self, name: &str, value: &ScriptValue) -> Result<()> {
        self.globals.insert(name.to_string(), value.clone());

        Python::with_gil(|py| {
            let globals = PyDict::new_bound(py);
            let py_value = script_value_to_python(py, value).map_err(|e| {
                ScriptError::ExecutionError(format!("Failed to convert value: {}", e))
            })?;
            globals
                .set_item(name, py_value)
                .map_err(|e| ScriptError::ExecutionError(format!("Failed to set global: {}", e)))?;
            Ok::<(), ScriptError>(())
        })?;
        Ok(())
    }

    /// 设置全局变量（无feature gate版本）
    #[cfg(not(feature = "pyo3"))]
    pub fn set_global(&mut self, _name: &str, _value: &ScriptValue) -> Result<()> {
        Err(ScriptError::ExecutionError(
            "Python feature not enabled".to_string(),
        ))
    }

    /// 获取全局变量
    #[cfg(feature = "pyo3")]
    pub fn get_global(&mut self, name: &str) -> Result<ScriptValue> {
        match self.globals.get(name) {
            Some(value) => Ok(value.clone()),
            None => Err(ScriptError::ExecutionError(format!(
                "Global '{}' not found",
                name
            ))),
        }
    }

    /// 获取全局变量（无feature gate版本）
    #[cfg(not(feature = "pyo3"))]
    pub fn get_global(&mut self, _name: &str) -> Result<ScriptValue> {
        Err(ScriptError::ExecutionError(
            "Python feature not enabled".to_string(),
        ))
    }

    /// 重置运行时
    pub fn reset(&mut self) {
        self.globals.clear();
        self.initialized = false;
        tracing::info!("Python runtime reset");
    }

    /// 确保运行时已初始化
    fn ensure_initialized(&mut self) -> Result<()> {
        if !self.initialized {
            self.initialize()?;
        }
        Ok(())
    }
}

/// Python上下文（实现ScriptContext trait）
#[derive(Debug, Default)]
pub struct PythonContextImpl {
    runtime: PythonRuntime,
}

impl PythonContextImpl {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ScriptContext for PythonContextImpl {
    fn execute(&mut self, script: &str, _source_code: Option<&str>) -> ScriptResult {
        match self.runtime.execute(script) {
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
            Ok(_) => ScriptResult::Void,
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

    fn language(&self) -> ScriptLanguage {
        ScriptLanguage::Python
    }

    fn has_function(&mut self, name: &str) -> bool {
        match self.eval(&format!("callable({})", name)) {
            ScriptResult::Success(ScriptValue::Boolean(true)) => true,
            _ => false,
        }
    }
}

// ============================================================================
// 辅助函数：Python值和ScriptValue之间的转换
// ============================================================================

/// 将Python值转换为ScriptValue
#[cfg(feature = "pyo3")]
fn python_value_to_script_value(py: Python, value: &pyo3::Bound<pyo3::PyAny>) -> ScriptValue {
    if value.is_none() {
        return ScriptValue::Null;
    }

    if let Ok(b) = value.extract::<bool>() {
        return ScriptValue::Boolean(b);
    }

    if let Ok(i) = value.extract::<i64>() {
        return ScriptValue::Integer(i);
    }

    if let Ok(f) = value.extract::<f64>() {
        return ScriptValue::Number(f);
    }

    if let Ok(s) = value.extract::<String>() {
        return ScriptValue::String(s);
    }

    if let Ok(list) = value.downcast::<PyList>() {
        let items: Vec<ScriptValue> =
            list.iter().map(|item| python_value_to_script_value(py, &item)).collect();
        return ScriptValue::Array(items);
    }

    if let Ok(dict) = value.downcast::<PyDict>() {
        let mut map = HashMap::new();
        for (key, val) in dict.iter() {
            if let Ok(key_str) = key.extract::<String>() {
                map.insert(key_str, python_value_to_script_value(py, &val));
            }
        }
        return ScriptValue::Object(map);
    }

    ScriptValue::Null
}

/// 将ScriptValue转换为Python值
#[cfg(feature = "pyo3")]
fn script_value_to_python(py: Python, value: &ScriptValue) -> pyo3::PyResult<pyo3::PyObject> {
    match value {
        ScriptValue::Null => Ok(pyo3::PyObject::new_bound(py, py.None()).to_object(py)),
        ScriptValue::Boolean(b) => Ok(b.to_object(py)),
        ScriptValue::Integer(i) => Ok(i.to_object(py)),
        ScriptValue::Number(n) => Ok(n.to_object(py)),
        ScriptValue::String(s) => Ok(s.to_object(py)),
        ScriptValue::Array(arr) => {
            let py_list = PyList::new_bound(
                py,
                arr.iter().map(|item| {
                    script_value_to_python(py, item).unwrap_or_else(|_| py.None().to_object(py))
                }),
            );
            Ok(py_list.to_object(py))
        }
        ScriptValue::Object(map) => {
            let py_dict = PyDict::new_bound(py);
            for (key, val) in map.iter() {
                let py_val = script_value_to_python(py, val)?;
                py_dict.set_item(key, py_val)?;
            }
            Ok(py_dict.to_object(py))
        }
    }
}

/// 将ScriptValue转换为Python代码字符串
#[cfg(feature = "pyo3")]
fn script_value_to_python_code(value: &ScriptValue) -> String {
    match value {
        ScriptValue::Null => "None".to_string(),
        ScriptValue::Boolean(b) => b.to_string(),
        ScriptValue::Integer(i) => i.to_string(),
        ScriptValue::Number(n) => n.to_string(),
        ScriptValue::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
        ScriptValue::Array(_) => "[]".to_string(),
        ScriptValue::Object(_) => "{}".to_string(),
    }
}

// ============================================================================
// Python模块定义（引擎API绑定）
// ============================================================================

/// Python模块：game_engine
#[cfg(feature = "pyo3")]
#[pymodule]
fn game_engine(_py: Python, m: &Bound<PyModule>) -> PyResult<()> {
    // Engine类
    m.add_class::<Engine>()?;

    // 全局函数
    m.add_function(wrap_pyfunction!(spawn_entity, m)?)?;
    m.add_function(wrap_pyfunction!(py_log, m)?)?;
    m.add_function(wrap_pyfunction!(get_time, m)?)?;

    Ok(())
}

/// Engine类
#[cfg(feature = "pyo3")]
#[pyclass]
pub struct Engine {
    #[pyo3(get, set)]
    name: String,
}

#[cfg(feature = "pyo3")]
#[pymethods]
impl Engine {
    #[new]
    fn new(name: String) -> Self {
        Self { name }
    }

    fn spawn_entity(&self) -> u64 {
        // 生成实体ID
        // 注意：当前使用AtomicU64生成伪实体ID，实际实现需要访问ECS World
        //
        // 实际实现应该:
        // 1. 获取ECS World引用（通过全局状态或依赖注入）
        // 2. 调用 world.spawn_empty() 或 world.spawn((...components))
        // 3. 返回 entity.to_bits() 作为 u64
        //
        // 示例:
        // let world = get_world_from_context(); // 需要实现
        // let entity = world.spawn_empty();
        // entity.to_bits()
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    }

    fn log(&self, msg: String) {
        tracing::info!("[Python]: {}", msg);
    }
}

/// spawn_entity函数
#[cfg(feature = "pyo3")]
#[pyfunction]
fn spawn_entity() -> PyResult<u64> {
    Ok(Engine::new("default".to_string()).spawn_entity())
}

/// log函数
#[cfg(feature = "pyo3")]
#[pyfunction]
fn py_log(msg: String) -> PyResult<()> {
    tracing::info!("[Python]: {}", msg);
    Ok(())
}

/// get_time函数
#[cfg(feature = "pyo3")]
#[pyfunction]
fn get_time() -> PyResult<f64> {
    Ok(crate::core::utils::current_timestamp_f64())
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "pyo3")]
    fn test_python_initialization() {
        let mut runtime = PythonRuntime::new();
        assert!(runtime.initialize().is_ok());
        assert!(runtime.initialized);
    }

    #[test]
    #[cfg(feature = "pyo3")]
    fn test_simple_execution() {
        let mut runtime = PythonRuntime::new();
        runtime.initialize().unwrap();

        let result = runtime.execute("x = 42");
        assert!(matches!(result, Ok(ScriptValue::Null)));
    }

    #[test]
    #[cfg(feature = "pyo3")]
    fn test_eval() {
        let mut runtime = PythonRuntime::new();
        runtime.initialize().unwrap();

        let result = runtime.eval("2 + 2").unwrap();
        assert!(matches!(result, ScriptValue::Integer(4)));
    }

    #[test]
    #[cfg(feature = "pyo3")]
    fn test_string_eval() {
        let mut runtime = PythonRuntime::new();
        runtime.initialize().unwrap();

        let result = runtime.eval("'Hello' + ' ' + 'World'").unwrap();
        assert!(matches!(result, ScriptValue::String(s) if s == "Hello World"));
    }

    #[test]
    #[cfg(feature = "pyo3")]
    fn test_list_eval() {
        let mut runtime = PythonRuntime::new();
        runtime.initialize().unwrap();

        let result = runtime.eval("[1, 2, 3]").unwrap();
        assert!(matches!(result, ScriptValue::Array(_)));
    }

    #[test]
    #[cfg(feature = "pyo3")]
    fn test_dict_eval() {
        let mut runtime = PythonRuntime::new();
        runtime.initialize().unwrap();

        let result = runtime.eval("{'key': 'value'}").unwrap();
        assert!(matches!(result, ScriptValue::Object(_)));
    }

    #[test]
    #[cfg(feature = "pyo3")]
    fn test_global_variables() {
        let mut runtime = PythonRuntime::new();
        runtime.initialize().unwrap();

        runtime.set_global("test_var", &ScriptValue::Integer(123)).unwrap();
        let result = runtime.get_global("test_var").unwrap();
        assert!(matches!(result, ScriptValue::Integer(123)));
    }

    #[test]
    fn test_python_context() {
        let mut ctx = PythonContextImpl::new();

        // 测试execute
        let result = ctx.execute("x = 42", None);
        assert!(matches!(result, ScriptResult::Success(_)));

        // 测试eval
        let result = ctx.eval("2 + 3");
        assert!(matches!(
            result,
            ScriptResult::Success(ScriptValue::Integer(5))
        ));

        // 测试全局变量
        let result = ctx.set_global("test", ScriptValue::Integer(100));
        assert!(matches!(result, ScriptResult::Void));

        let result = ctx.get_global("test");
        assert!(matches!(
            result,
            ScriptResult::Success(ScriptValue::Integer(100))
        ));
    }
}
