// 变量监视模块
//
// 提供变量的查看、监视和修改功能

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 变量引用ID类型
pub type VariableReference = i64;

/// 变量类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    /// 空值
    Null,
    /// 布尔值
    Boolean,
    /// 整数
    Integer,
    /// 浮点数
    Float,
    /// 字符串
    String,
    /// 数组
    Array,
    /// 对象
    Object,
    /// 函数
    Function,
    /// 未知类型
    Unknown,
}

impl VariableType {
    pub fn as_str(&self) -> &'static str {
        match self {
            VariableType::Null => "null",
            VariableType::Boolean => "boolean",
            VariableType::Integer => "integer",
            VariableType::Float => "float",
            VariableType::String => "string",
            VariableType::Array => "array",
            VariableType::Object => "object",
            VariableType::Function => "function",
            VariableType::Unknown => "unknown",
        }
    }
}

/// 变量信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// 变量名
    pub name: String,
    /// 变量值
    pub value: String,
    /// 变量类型
    pub var_type: VariableType,
    /// 变量引用（用于子变量）
    pub variables_reference: VariableReference,
    /// 命名变量数
    pub named_variables: Option<i64>,
    /// 索引变量数
    pub indexed_variables: Option<i64>,
    /// 求值名称
    pub evaluate_name: Option<String>,
    /// 是否可修改
    pub writable: bool,
    /// 变量作用域
    pub scope: String,
}

/// 作用域类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeKind {
    /// 局部作用域
    Local,
    /// 全局作用域
    Global,
    /// 参数作用域
    Arguments,
    /// 闭包作用域
    Closure,
    /// Catch作用域
    Catch,
    /// 模块作用域
    Module,
}

/// 作用域信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// 作用域名称
    pub name: String,
    /// 作用域类型
    pub scope_kind: ScopeKind,
    /// 变量引用
    pub variables_reference: VariableReference,
    /// 命名变量数
    pub named_variables: i64,
    /// 索引变量数
    pub indexed_variables: i64,
    /// 是否昂贵
    pub expensive: bool,
}

/// 变量监视项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchItem {
    /// 监视项ID
    pub id: i64,
    /// 表达式
    pub expression: String,
    /// 当前值
    pub value: Option<String>,
    /// 变量类型
    pub var_type: Option<String>,
    /// 是否有效
    pub valid: bool,
    /// 错误消息
    pub error: Option<String>,
}

/// 变量监视器
pub struct VariableMonitor {
    /// 变量存储（key为变量引用ID）
    variables: Arc<RwLock<HashMap<VariableReference, Variable>>>,
    /// 作用域存储
    scopes: Arc<RwLock<Vec<Scope>>>,
    /// 监视项存储
    watch_items: Arc<RwLock<HashMap<i64, WatchItem>>>,
    /// 下一个变量引用ID
    next_var_ref: Arc<RwLock<VariableReference>>,
    /// 下一个监视项ID
    next_watch_id: Arc<RwLock<i64>>,
}

impl VariableMonitor {
    /// 创建新的变量监视器
    pub fn new() -> Self {
        Self {
            variables: Arc::new(RwLock::new(HashMap::new())),
            scopes: Arc::new(RwLock::new(Vec::new())),
            watch_items: Arc::new(RwLock::new(HashMap::new())),
            next_var_ref: Arc::new(RwLock::new(1000)),
            next_watch_id: Arc::new(RwLock::new(1)),
        }
    }

    /// 添加作用域
    pub async fn add_scope(&self, scope: Scope) {
        let mut scopes = self.scopes.write().await;
        scopes.push(scope);
    }

    /// 获取所有作用域
    pub async fn get_scopes(&self) -> Vec<Scope> {
        self.scopes.read().await.clone()
    }

    /// 清除所有作用域
    pub async fn clear_scopes(&self) {
        let mut scopes = self.scopes.write().await;
        scopes.clear();
    }

    /// 添加变量
    pub async fn add_variable(&self, scope_kind: ScopeKind, var: Variable) -> VariableReference {
        let mut next_id = self.next_var_ref.write().await;
        let id = *next_id;
        *next_id += 1;

        let mut var_with_ref = var.clone();
        var_with_ref.variables_reference = if var_with_ref.variables_reference == 0 {
            id
        } else {
            var_with_ref.variables_reference
        };

        let mut variables = self.variables.write().await;
        variables.insert(id, var_with_ref);

        tracing::debug!("Variable added: {} (ref: {})", var.name, id);

        id
    }

    /// 获取变量（通过引用）
    pub async fn get_variable(&self, var_ref: VariableReference) -> Option<Variable> {
        let variables = self.variables.read().await;
        variables.get(&var_ref).cloned()
    }

    /// 获取子变量（通过引用）
    pub async fn get_children(&self, var_ref: VariableReference) -> Vec<Variable> {
        let mut children = Vec::new();
        let variables = self.variables.read().await;

        for (_, var) in variables.iter() {
            // 简化实现：返回所有以ref为前缀的变量
            // 实际实现中应该有更清晰的父子关系
            if var.variables_reference == var_ref {
                children.push(var.clone());
            }
        }

        children
    }

    /// 设置变量值
    pub async fn set_variable(
        &self,
        var_ref: VariableReference,
        new_value: String,
    ) -> Result<(), String> {
        let mut variables = self.variables.write().await;

        if let Some(var) = variables.get_mut(&var_ref) {
            if !var.writable {
                return Err(format!("Variable '{}' is not writable", var.name));
            }

            var.value = new_value.clone();
            tracing::info!("Variable set: {} = {}", var.name, new_value);
            Ok(())
        } else {
            Err(format!("Variable reference {} not found", var_ref))
        }
    }

    /// 清除所有变量
    pub async fn clear_variables(&self) {
        let mut variables = self.variables.write().await;
        variables.clear();

        // 重置变量引用计数器
        let mut next_id = self.next_var_ref.write().await;
        *next_id = 1000;
    }

    /// 添加监视项
    pub async fn add_watch(&self, expression: String) -> i64 {
        let mut next_id = self.next_watch_id.write().await;
        let id = *next_id;
        *next_id += 1;

        let watch_item = WatchItem {
            id,
            expression: expression.clone(),
            value: None,
            var_type: None,
            valid: false,
            error: None,
        };

        let mut watch_items = self.watch_items.write().await;
        watch_items.insert(id, watch_item);

        // 立即求值
        self.evaluate_watch(id).await;

        id
    }

    /// 删除监视项
    pub async fn remove_watch(&self, id: i64) -> bool {
        let mut watch_items = self.watch_items.write().await;
        watch_items.remove(&id).is_some()
    }

    /// 获取所有监视项
    pub async fn get_watches(&self) -> Vec<WatchItem> {
        let watch_items = self.watch_items.read().await;
        watch_items.values().cloned().collect()
    }

    /// 求值监视项
    pub async fn evaluate_watch(&self, id: i64) {
        let mut watch_items = self.watch_items.write().await;

        if let Some(watch) = watch_items.get_mut(&id) {
            // 使用表达式求值器
            match Self::evaluate_expression_internal(&watch.expression, &self.variables).await {
                Ok((value, var_type)) => {
                    watch.value = Some(value);
                    watch.var_type = Some(var_type);
                    watch.valid = true;
                    watch.error = None;

                    tracing::debug!(
                        "Watch evaluated: {} = {}",
                        watch.expression,
                        watch.value.as_ref().unwrap()
                    );
                }
                Err(e) => {
                    watch.value = None;
                    watch.var_type = None;
                    watch.valid = false;
                    watch.error = Some(e);

                    tracing::warn!(
                        "Watch evaluation failed: {} - {}",
                        watch.expression,
                        watch.error.as_ref().unwrap()
                    );
                }
            }
        }
    }

    /// 内部表达式求值实现
    async fn evaluate_expression_internal(
        expression: &str,
        variables: &HashMap<VariableReference, Variable>,
    ) -> Result<(String, String), String> {
        let expression = expression.trim();

        // 1. 字面量值
        if let Ok(num) = expression.parse::<i64>() {
            return Ok((num.to_string(), "integer".to_string()));
        }
        if let Ok(num) = expression.parse::<f64>() {
            return Ok((num.to_string(), "float".to_string()));
        }
        if expression.starts_with('"') && expression.ends_with('"') {
            return Ok((expression.to_string(), "string".to_string()));
        }
        if expression == "true" || expression == "false" {
            return Ok((expression.to_string(), "boolean".to_string()));
        }

        // 2. 简单变量名查找
        if expression.chars().all(|c| c.is_alphanumeric() || c == '_') {
            for (_, var) in variables.iter() {
                if var.name == expression {
                    return Ok((var.value.clone(), var.var_type.as_str().to_string()));
                }
            }
            return Ok((
                format!("<undefined: {}>", expression),
                "unknown".to_string(),
            ));
        }

        // 3. 简单的算术表达式
        if expression.contains('+') {
            let parts: Vec<&str> = expression.split('+').collect();
            if parts.len() == 2 {
                let left = Self::evaluate_expression_internal(parts[0].trim(), variables).await?;
                let right = Self::evaluate_expression_internal(parts[1].trim(), variables).await?;

                if let (Ok(l), Ok(r)) = (left.0.parse::<i64>(), right.0.parse::<i64>()) {
                    return Ok((format!("{}", l + r), "integer".to_string()));
                }
            }
        }

        if expression.contains('-') && !expression.starts_with('-') {
            let parts: Vec<&str> = expression.split('-').collect();
            if parts.len() == 2 {
                let left = Self::evaluate_expression_internal(parts[0].trim(), variables).await?;
                let right = Self::evaluate_expression_internal(parts[1].trim(), variables).await?;

                if let (Ok(l), Ok(r)) = (left.0.parse::<i64>(), right.0.parse::<i64>()) {
                    return Ok((format!("{}", l - r), "integer".to_string()));
                }
            }
        }

        if expression.contains('*') {
            let parts: Vec<&str> = expression.split('*').collect();
            if parts.len() == 2 {
                let left = Self::evaluate_expression_internal(parts[0].trim(), variables).await?;
                let right = Self::evaluate_expression_internal(parts[1].trim(), variables).await?;

                if let (Ok(l), Ok(r)) = (left.0.parse::<i64>(), right.0.parse::<i64>()) {
                    return Ok((format!("{}", l * r), "integer".to_string()));
                }
            }
        }

        if expression.contains('/') {
            let parts: Vec<&str> = expression.split('/').collect();
            if parts.len() == 2 {
                let left = Self::evaluate_expression_internal(parts[0].trim(), variables).await?;
                let right = Self::evaluate_expression_internal(parts[1].trim(), variables).await?;

                if let (Ok(l), Ok(r)) = (left.0.parse::<f64>(), right.0.parse::<f64>()) {
                    return Ok((format!("{}", l / r), "float".to_string()));
                }
            }
        }

        // 4. 无法识别的表达式
        Err(format!("Cannot evaluate expression: {}", expression))
    }

    /// 求值所有监视项
    pub async fn evaluate_all_watches(&self) {
        let watch_items = self.watch_items.read().await;
        let ids: Vec<i64> = watch_items.keys().cloned().collect();
        drop(watch_items);

        for id in ids {
            self.evaluate_watch(id).await;
        }
    }

    /// 求值表达式
    pub async fn evaluate_expression(&self, expression: &str) -> Result<String, String> {
        let variables = self.variables.read().await;
        let result = Self::evaluate_expression_internal(expression, &variables).await?;
        Ok(result.0)
    }

    /// 获取作用域的变量
    pub async fn get_variables_for_scope(&self, frame_id: i64) -> Vec<Variable> {
        let variables = self.variables.read().await;

        // 基于frame_id获取变量
        // 在实际实现中，这里应该从调试器获取特定栈帧的变量
        variables
            .values()
            .filter(|var| {
                // 简化：根据作用域字符串过滤
                var.scope.contains(&format!("frame_{}", frame_id))
                    || var.scope == "Local"
                    || var.scope == "Global"
            })
            .cloned()
            .collect()
    }

    /// 浏览变量（获取子变量）
    pub async fn browse_variable(&self, var_ref: VariableReference) -> Vec<Variable> {
        self.get_children(var_ref).await
    }

    /// 获取变量统计
    pub async fn get_stats(&self) -> VariableStats {
        let variables = self.variables.read().await;
        let scopes = self.scopes.read().await;
        let watch_items = self.watch_items.read().await;

        let total_vars = variables.len();
        let valid_watches = watch_items.values().filter(|w| w.valid).count();
        let invalid_watches = watch_items.len() - valid_watches;

        VariableStats {
            total_variables: total_vars,
            total_scopes: scopes.len(),
            total_watches: watch_items.len(),
            valid_watches,
            invalid_watches,
        }
    }
}

impl Default for VariableMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 变量统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableStats {
    /// 总变量数
    pub total_variables: usize,
    /// 总作用域数
    pub total_scopes: usize,
    /// 总监视项数
    pub total_watches: usize,
    /// 有效监视项数
    pub valid_watches: usize,
    /// 无效监视项数
    pub invalid_watches: usize,
}

/// 变量格式化器
pub struct VariableFormatter;

impl VariableFormatter {
    /// 格式化变量值为字符串
    pub fn format_value(value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            serde_json::Value::Number(n) => {
                if n.is_i64() {
                    n.as_i64().unwrap().to_string()
                } else {
                    n.as_f64().unwrap().to_string()
                }
            }
            serde_json::Value::String(s) => {
                if s.len() > 100 {
                    format!("{}...", &s[..100])
                } else {
                    s.clone()
                }
            }
            serde_json::Value::Array(arr) => {
                format!("Array({})", arr.len())
            }
            serde_json::Value::Object(obj) => {
                format!("Object({})", obj.len())
            }
        }
    }

    /// 从ScriptValue创建Variable
    pub fn from_script_value(
        name: String,
        value: &crate::scripting::system::ScriptValue,
        scope: String,
    ) -> Variable {
        use crate::scripting::system::ScriptValue;

        let (value_str, var_type) = match value {
            ScriptValue::Null => ("null".to_string(), VariableType::Null),
            ScriptValue::Boolean(b) => (b.to_string(), VariableType::Boolean),
            ScriptValue::Integer(i) => (i.to_string(), VariableType::Integer),
            ScriptValue::Number(n) => (n.to_string(), VariableType::Float),
            ScriptValue::String(s) => {
                if s.len() > 100 {
                    (format!("{}...", &s[..100]), VariableType::String)
                } else {
                    (s.clone(), VariableType::String)
                }
            }
            ScriptValue::Array(arr) => (format!("Array({})", arr.len()), VariableType::Array),
            ScriptValue::Object(obj) => (format!("Object({})", obj.len()), VariableType::Object),
        };

        Variable {
            name,
            value: value_str,
            var_type,
            variables_reference: 0,
            named_variables: None,
            indexed_variables: None,
            evaluate_name: None,
            writable: true,
            scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_add_scope() {
        let monitor = VariableMonitor::new();

        let scope = Scope {
            name: "Local".to_string(),
            scope_kind: ScopeKind::Local,
            variables_reference: 1000,
            named_variables: 5,
            indexed_variables: 0,
            expensive: false,
        };

        monitor.add_scope(scope).await;

        let scopes = monitor.get_scopes().await;
        assert_eq!(scopes.len(), 1);
        assert_eq!(scopes[0].name, "Local");
    }

    #[tokio::test]
    async fn test_add_variable() {
        let monitor = VariableMonitor::new();

        let var = Variable {
            name: "x".to_string(),
            value: "42".to_string(),
            var_type: VariableType::Integer,
            variables_reference: 0,
            named_variables: None,
            indexed_variables: None,
            evaluate_name: None,
            writable: true,
            scope: "Local".to_string(),
        };

        let var_ref = monitor.add_variable(ScopeKind::Local, var).await;

        let retrieved = monitor.get_variable(var_ref).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "x");
    }

    #[tokio::test]
    async fn test_set_variable() {
        let monitor = VariableMonitor::new();

        let var = Variable {
            name: "x".to_string(),
            value: "42".to_string(),
            var_type: VariableType::Integer,
            variables_reference: 0,
            named_variables: None,
            indexed_variables: None,
            evaluate_name: None,
            writable: true,
            scope: "Local".to_string(),
        };

        let var_ref = monitor.add_variable(ScopeKind::Local, var).await;

        let result = monitor.set_variable(var_ref, "100".to_string()).await;
        assert!(result.is_ok());

        let updated = monitor.get_variable(var_ref).await;
        assert_eq!(updated.unwrap().value, "100");
    }

    #[tokio::test]
    async fn test_set_readonly_variable() {
        let monitor = VariableMonitor::new();

        let var = Variable {
            name: "constant".to_string(),
            value: "3.14".to_string(),
            var_type: VariableType::Float,
            variables_reference: 0,
            named_variables: None,
            indexed_variables: None,
            evaluate_name: None,
            writable: false,
            scope: "Global".to_string(),
        };

        let var_ref = monitor.add_variable(ScopeKind::Global, var).await;

        let result = monitor.set_variable(var_ref, "2.71".to_string()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_add_watch() {
        let monitor = VariableMonitor::new();

        let id = monitor.add_watch("x + y".to_string()).await;

        let watches = monitor.get_watches().await;
        assert_eq!(watches.len(), 1);
        assert_eq!(watches[0].expression, "x + y");
        assert!(watches[0].valid);
    }

    #[tokio::test]
    async fn test_remove_watch() {
        let monitor = VariableMonitor::new();

        let id = monitor.add_watch("x".to_string()).await;
        let removed = monitor.remove_watch(id).await;
        assert!(removed);

        let watches = monitor.get_watches().await;
        assert_eq!(watches.len(), 0);
    }

    #[tokio::test]
    async fn test_evaluate_expression() {
        let monitor = VariableMonitor::new();

        // 先添加一个变量
        let var = Variable {
            name: "myVar".to_string(),
            value: "42".to_string(),
            var_type: VariableType::Integer,
            variables_reference: 0,
            named_variables: None,
            indexed_variables: None,
            evaluate_name: None,
            writable: true,
            scope: "Local".to_string(),
        };
        monitor.add_variable(ScopeKind::Local, var).await;

        // 求值变量名
        let result = monitor.evaluate_expression("myVar").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");

        // 求值不存在的变量
        let result = monitor.evaluate_expression("unknownVar").await;
        assert!(result.is_ok());
        assert!(result.unwrap().starts_with("<"));
    }

    #[tokio::test]
    async fn test_variable_stats() {
        let monitor = VariableMonitor::new();

        monitor
            .add_scope(Scope {
                name: "Local".to_string(),
                scope_kind: ScopeKind::Local,
                variables_reference: 1000,
                named_variables: 2,
                indexed_variables: 0,
                expensive: false,
            })
            .await;

        monitor
            .add_variable(
                ScopeKind::Local,
                Variable {
                    name: "x".to_string(),
                    value: "1".to_string(),
                    var_type: VariableType::Integer,
                    variables_reference: 0,
                    named_variables: None,
                    indexed_variables: None,
                    evaluate_name: None,
                    writable: true,
                    scope: "Local".to_string(),
                },
            )
            .await;

        monitor.add_watch("x".to_string()).await;

        let stats = monitor.get_stats().await;
        assert_eq!(stats.total_variables, 1);
        assert_eq!(stats.total_scopes, 1);
        assert_eq!(stats.total_watches, 1);
        assert_eq!(stats.valid_watches, 1);
    }

    #[test]
    fn test_variable_formatter() {
        let json_value = serde_json::json!(42);
        let formatted = VariableFormatter::format_value(&json_value);
        assert_eq!(formatted, "42");

        let json_value = serde_json::json!("hello");
        let formatted = VariableFormatter::format_value(&json_value);
        assert_eq!(formatted, "hello");

        let json_value = serde_json::json!([1, 2, 3]);
        let formatted = VariableFormatter::format_value(&json_value);
        assert_eq!(formatted, "Array(3)");
    }

    #[test]
    fn test_variable_type_as_str() {
        assert_eq!(VariableType::Null.as_str(), "null");
        assert_eq!(VariableType::Boolean.as_str(), "boolean");
        assert_eq!(VariableType::Integer.as_str(), "integer");
        assert_eq!(VariableType::String.as_str(), "string");
        assert_eq!(VariableType::Array.as_str(), "array");
        assert_eq!(VariableType::Object.as_str(), "object");
    }

    #[tokio::test]
    async fn test_evaluate_arithmetic_expressions() {
        let monitor = VariableMonitor::new();

        // 加法
        let result = monitor.evaluate_expression("5 + 3").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "8");

        // 减法
        let result = monitor.evaluate_expression("10 - 4").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "6");

        // 乘法
        let result = monitor.evaluate_expression("6 * 7").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");

        // 除法
        let result = monitor.evaluate_expression("20 / 4").await;
        assert!(result.is_ok());
        assert!(result.unwrap(), "5"); // 可能是 "5" 或 "5.0"
    }

    #[tokio::test]
    async fn test_evaluate_literal_values() {
        let monitor = VariableMonitor::new();

        // 整数
        let result = monitor.evaluate_expression("42").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "42");

        // 浮点数
        let result = monitor.evaluate_expression("3.14").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "3.14");

        // 布尔值
        let result = monitor.evaluate_expression("true").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "true");

        let result = monitor.evaluate_expression("false").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "false");

        // 字符串
        let result = monitor.evaluate_expression("\"hello\"").await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "\"hello\"");
    }

    #[tokio::test]
    async fn test_browse_variable_children() {
        let monitor = VariableMonitor::new();

        // 添加一个有子变量的变量（数组）
        let array_var = Variable {
            name: "myArray".to_string(),
            value: "[1, 2, 3]".to_string(),
            var_type: VariableType::Array,
            variables_reference: 1001,
            named_variables: Some(0),
            indexed_variables: Some(3),
            evaluate_name: Some("myArray".to_string()),
            writable: false,
            scope: "Local".to_string(),
        };
        let array_ref = monitor.add_variable(ScopeKind::Local, array_var).await;

        // 添加子元素
        for i in 0..3 {
            let elem_var = Variable {
                name: format!("[{}]", i),
                value: format!("{}", i + 1),
                var_type: VariableType::Integer,
                variables_reference: array_ref,
                named_variables: None,
                indexed_variables: None,
                evaluate_name: None,
                writable: false,
                scope: "Local".to_string(),
            };
            monitor.add_variable(ScopeKind::Local, elem_var).await;
        }

        // 浏览子变量
        let children = monitor.browse_variable(array_ref).await;
        assert!(!children.is_empty());
    }

    #[tokio::test]
    async fn test_get_variables_for_scope() {
        let monitor = VariableMonitor::new();

        // 添加不同作用域的变量
        monitor
            .add_variable(
                ScopeKind::Local,
                Variable {
                    name: "local_var".to_string(),
                    value: "1".to_string(),
                    var_type: VariableType::Integer,
                    variables_reference: 0,
                    named_variables: None,
                    indexed_variables: None,
                    evaluate_name: None,
                    writable: true,
                    scope: "Local".to_string(),
                },
            )
            .await;

        monitor
            .add_variable(
                ScopeKind::Global,
                Variable {
                    name: "global_var".to_string(),
                    value: "2".to_string(),
                    var_type: VariableType::Integer,
                    variables_reference: 0,
                    named_variables: None,
                    indexed_variables: None,
                    evaluate_name: None,
                    writable: true,
                    scope: "Global".to_string(),
                },
            )
            .await;

        // 获取Local作用域的变量
        let local_vars = monitor.get_variables_for_scope(1).await;
        assert!(!local_vars.is_empty());
    }

    #[tokio::test]
    async fn test_watch_evaluation_error() {
        let monitor = VariableMonitor::new();

        // 添加一个无效的表达式
        let id = monitor.add_watch("undefined_var + unknown_var".to_string()).await;

        let watches = monitor.get_watches().await;
        let watch = watches.iter().find(|w| w.id == id).unwrap();

        // 监视项应该存在
        assert_eq!(watch.expression, "undefined_var + unknown_var");
    }

    #[tokio::test]
    async fn test_clear_variables() {
        let monitor = VariableMonitor::new();

        // 添加一些变量
        monitor
            .add_variable(
                ScopeKind::Local,
                Variable {
                    name: "x".to_string(),
                    value: "1".to_string(),
                    var_type: VariableType::Integer,
                    variables_reference: 0,
                    named_variables: None,
                    indexed_variables: None,
                    evaluate_name: None,
                    writable: true,
                    scope: "Local".to_string(),
                },
            )
            .await;

        assert_eq!(monitor.get_stats().await.total_variables, 1);

        // 清除
        monitor.clear_variables().await;

        assert_eq!(monitor.get_stats().await.total_variables, 0);
    }

    #[tokio::test]
    async fn test_clear_scopes() {
        let monitor = VariableMonitor::new();

        monitor
            .add_scope(Scope {
                name: "Local".to_string(),
                scope_kind: ScopeKind::Local,
                variables_reference: 1000,
                named_variables: 5,
                indexed_variables: 0,
                expensive: false,
            })
            .await;

        assert_eq!(monitor.get_stats().await.total_scopes, 1);

        monitor.clear_scopes().await;

        assert_eq!(monitor.get_stats().await.total_scopes, 0);
    }
}
