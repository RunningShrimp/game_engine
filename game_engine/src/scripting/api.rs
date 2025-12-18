use super::system::{ScriptResult, ScriptValue};
use glam::{Quat, Vec2, Vec3};
use std::collections::HashMap;

/// 脚本API - 提供引擎功能的脚本接口
pub struct ScriptApi {
    registered_functions:
        HashMap<String, Box<dyn Fn(&[ScriptValue]) -> ScriptResult + Send + Sync>>,
}

impl ScriptApi {
    pub fn new() -> Self {
        let mut api = Self {
            registered_functions: HashMap::new(),
        };

        // 注册内置函数
        api.register_builtin_functions();

        api
    }

    /// 注册内置函数
    fn register_builtin_functions(&mut self) {
        // 日志函数
        self.register_function("log", |args| {
            if let Some(ScriptValue::String(msg)) = args.first() {
                tracing::info!(target: "scripting", "[Script] {}", msg);
                ScriptResult::Void
            } else {
                ScriptResult::Error("log() requires a string argument".to_string())
            }
        });

        // 数学函数
        self.register_function("sqrt", |args| {
            if let Some(ScriptValue::Float(x)) = args.first() {
                ScriptResult::Success(x.sqrt().to_string())
            } else {
                ScriptResult::Error("sqrt() requires a number argument".to_string())
            }
        });

        self.register_function("sin", |args| {
            if let Some(ScriptValue::Float(x)) = args.first() {
                ScriptResult::Success(x.sin().to_string())
            } else {
                ScriptResult::Error("sin() requires a number argument".to_string())
            }
        });

        self.register_function("cos", |args| {
            if let Some(ScriptValue::Float(x)) = args.first() {
                ScriptResult::Success(x.cos().to_string())
            } else {
                ScriptResult::Error("cos() requires a number argument".to_string())
            }
        });
    }

    /// 注册自定义函数
    pub fn register_function<F>(&mut self, name: &str, func: F)
    where
        F: Fn(&[ScriptValue]) -> ScriptResult + Send + Sync + 'static,
    {
        self.registered_functions
            .insert(name.to_string(), Box::new(func));
    }

    /// 调用已注册的函数
    pub fn call(&self, name: &str, args: &[ScriptValue]) -> ScriptResult {
        if let Some(func) = self.registered_functions.get(name) {
            func(args)
        } else {
            ScriptResult::Error(format!("Function '{}' not found", name))
        }
    }
}


/// 扩展的脚本值类型,支持更多的引擎类型
#[derive(Debug, Clone)]
pub enum ExtendedScriptValue {
    // 基础类型
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<ExtendedScriptValue>),
    Object(HashMap<String, ExtendedScriptValue>),

    // 引擎类型
    Vec2(Vec2),
    Vec3(Vec3),
    Quat(Quat),
    Entity(u64), // 实体ID
}

impl ExtendedScriptValue {
    /// 转换为基础ScriptValue
    pub fn to_script_value(&self) -> ScriptValue {
        match self {
            ExtendedScriptValue::Null => ScriptValue::Null,
            ExtendedScriptValue::Bool(b) => ScriptValue::Bool(*b),
            ExtendedScriptValue::Int(i) => ScriptValue::Int(*i),
            ExtendedScriptValue::Float(f) => ScriptValue::Float(*f),
            ExtendedScriptValue::String(s) => ScriptValue::String(s.clone()),
            ExtendedScriptValue::Array(arr) => {
                ScriptValue::Array(arr.iter().map(|v| v.to_script_value()).collect())
            }
            ExtendedScriptValue::Object(obj) => ScriptValue::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.to_script_value()))
                    .collect(),
            ),
            ExtendedScriptValue::Vec2(v) => {
                let mut obj = HashMap::new();
                obj.insert("x".to_string(), ScriptValue::Float(v.x as f64));
                obj.insert("y".to_string(), ScriptValue::Float(v.y as f64));
                ScriptValue::Object(obj)
            }
            ExtendedScriptValue::Vec3(v) => {
                let mut obj = HashMap::new();
                obj.insert("x".to_string(), ScriptValue::Float(v.x as f64));
                obj.insert("y".to_string(), ScriptValue::Float(v.y as f64));
                obj.insert("z".to_string(), ScriptValue::Float(v.z as f64));
                ScriptValue::Object(obj)
            }
            ExtendedScriptValue::Quat(q) => {
                let mut obj = HashMap::new();
                obj.insert("x".to_string(), ScriptValue::Float(q.x as f64));
                obj.insert("y".to_string(), ScriptValue::Float(q.y as f64));
                obj.insert("z".to_string(), ScriptValue::Float(q.z as f64));
                obj.insert("w".to_string(), ScriptValue::Float(q.w as f64));
                ScriptValue::Object(obj)
            }
            ExtendedScriptValue::Entity(id) => ScriptValue::Int(*id as i64),
        }
    }

    /// 从基础ScriptValue转换
    pub fn from_script_value(value: &ScriptValue) -> Self {
        match value {
            ScriptValue::Null => ExtendedScriptValue::Null,
            ScriptValue::Bool(b) => ExtendedScriptValue::Bool(*b),
            ScriptValue::Int(i) => ExtendedScriptValue::Int(*i),
            ScriptValue::Float(f) => ExtendedScriptValue::Float(*f),
            ScriptValue::String(s) => ExtendedScriptValue::String(s.clone()),
            ScriptValue::Array(arr) => {
                ExtendedScriptValue::Array(arr.iter().map(Self::from_script_value).collect())
            }
            ScriptValue::Object(obj) => ExtendedScriptValue::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), Self::from_script_value(v)))
                    .collect(),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_api() {
        let api = ScriptApi::new();

        // 测试日志函数
        let result = api.call(
            "log",
            &[ScriptValue::String("Hello from script!".to_string())],
        );
        assert!(matches!(result, ScriptResult::Void));

        // 测试数学函数
        let result = api.call("sqrt", &[ScriptValue::Float(16.0)]);
        assert!(matches!(result, ScriptResult::Success(_)));
    }

    #[test]
    fn test_extended_script_value() {
        let vec3 = ExtendedScriptValue::Vec3(Vec3::new(1.0, 2.0, 3.0));
        let script_value = vec3.to_script_value();

        if let ScriptValue::Object(obj) = script_value {
            assert_eq!(obj.get("x"), Some(&ScriptValue::Float(1.0)));
            assert_eq!(obj.get("y"), Some(&ScriptValue::Float(2.0)));
            assert_eq!(obj.get("z"), Some(&ScriptValue::Float(3.0)));
        } else {
            panic!("Expected Object");
        }
    }
}
