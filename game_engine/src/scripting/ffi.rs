//! 跨语言FFI层
//!
//! 提供统一的类型转换和参数传递接口，确保所有脚本语言使用一致的API

use crate::scripting::system::{ScriptResult, ScriptValue};
use std::collections::HashMap;

/// FFI类型转换器
///
/// 在不同脚本语言之间转换类型
pub struct FFIConverter;

impl FFIConverter {
    /// 将ScriptValue转换为字符串表示（用于跨语言传递）
    pub fn to_string(value: &ScriptValue) -> String {
        match value {
            ScriptValue::Null => "null".to_string(),
            ScriptValue::Boolean(b) => b.to_string(),
            ScriptValue::Integer(i) => i.to_string(),
            ScriptValue::Number(n) => n.to_string(),
            ScriptValue::String(s) => s.clone(),
            ScriptValue::Array(arr) => {
                let items: Vec<String> = arr.iter().map(Self::to_string).collect();
                format!("[{}]", items.join(","))
            }
            ScriptValue::Object(obj) => {
                let pairs: Vec<String> =
                    obj.iter().map(|(k, v)| format!("{}:{}", k, Self::to_string(v))).collect();
                format!("{{{}}}", pairs.join(","))
            }
        }
    }

    /// 从字符串解析ScriptValue（简化实现）
    pub fn from_string(s: &str) -> ScriptValue {
        // 尝试解析为数字
        if let Ok(i) = s.parse::<i64>() {
            return ScriptValue::Integer(i);
        }
        if let Ok(f) = s.parse::<f64>() {
            return ScriptValue::Number(f);
        }
        // 尝试解析为布尔值
        if s == "true" {
            return ScriptValue::Boolean(true);
        }
        if s == "false" {
            return ScriptValue::Boolean(false);
        }
        // 默认为字符串
        ScriptValue::String(s.to_string())
    }

    /// 将参数数组转换为统一格式
    pub fn normalize_args(args: &[ScriptValue]) -> Vec<ScriptValue> {
        args.to_vec()
    }

    /// 验证参数类型
    pub fn validate_args(args: &[ScriptValue], expected_types: &[&str]) -> Result<(), String> {
        if args.len() != expected_types.len() {
            return Err(format!(
                "Expected {} arguments, got {}",
                expected_types.len(),
                args.len()
            ));
        }

        for (i, (arg, expected_type)) in args.iter().zip(expected_types.iter()).enumerate() {
            let actual_type = match arg {
                ScriptValue::Null => "null",
                ScriptValue::Boolean(_) => "boolean",
                ScriptValue::Integer(_) => "integer",
                ScriptValue::Number(_) => "number",
                ScriptValue::String(_) => "string",
                ScriptValue::Array(_) => "array",
                ScriptValue::Object(_) => "object",
            };

            if actual_type != *expected_type && *expected_type != "any" {
                return Err(format!(
                    "Argument {i}: expected {expected_type}, got {actual_type}"
                ));
            }
        }

        Ok(())
    }
}

/// FFI错误处理
pub struct FFIErrorHandler;

impl FFIErrorHandler {
    /// 将Rust错误转换为ScriptResult
    pub fn to_script_result<T>(result: Result<T, impl std::fmt::Display>) -> ScriptResult {
        match result {
            Ok(_) => ScriptResult::Void,
            Err(e) => ScriptResult::Error(e.to_string()),
        }
    }

    /// 将ScriptResult转换为Result
    pub fn from_script_result<T>(result: ScriptResult) -> Result<T, String> {
        match result {
            ScriptResult::Success(value) => {
                // 尝试从value提取T（简化实现）
                Err("Cannot extract typed value from ScriptResult".to_string())
            }
            ScriptResult::Error(e) => Err(e),
            ScriptResult::Void => Err("Void result cannot be converted".to_string()),
        }
    }
}

/// 统一的API调用接口
pub struct UnifiedAPI {
    /// 注册的函数
    functions: HashMap<String, Box<dyn Fn(&[ScriptValue]) -> ScriptResult + Send + Sync>>,
}

impl UnifiedAPI {
    /// 创建新的统一API
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    /// 注册函数
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[ScriptValue]) -> ScriptResult + Send + Sync + 'static,
    {
        self.functions.insert(name.to_string(), Box::new(func));
    }

    /// 调用函数
    pub fn call(&self, name: &str, args: &[ScriptValue]) -> ScriptResult {
        match self.functions.get(name) {
            Some(func) => func(args),
            None => ScriptResult::Error(format!("Function '{name}' not found")),
        }
    }
}

impl Default for UnifiedAPI {
    fn default() -> Self {
        Self::new()
    }
}

/// 类型映射表
///
/// 定义不同脚本语言之间的类型映射关系
pub struct TypeMapping;

impl TypeMapping {
    /// 获取类型映射
    pub fn get_mapping(from_lang: &str, to_lang: &str) -> HashMap<String, String> {
        let mut mapping = HashMap::new();

        // 基本类型映射（大多数语言相同）
        mapping.insert("integer".to_string(), "integer".to_string());
        mapping.insert("number".to_string(), "number".to_string());
        mapping.insert("string".to_string(), "string".to_string());
        mapping.insert("boolean".to_string(), "boolean".to_string());

        // 语言特定映射
        if from_lang == "lua" && to_lang == "javascript" {
            mapping.insert("table".to_string(), "object".to_string());
            mapping.insert("nil".to_string(), "null".to_string());
        } else if from_lang == "javascript" && to_lang == "lua" {
            mapping.insert("object".to_string(), "table".to_string());
            mapping.insert("null".to_string(), "nil".to_string());
        }

        mapping
    }
}
