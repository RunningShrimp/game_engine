//  便捷错误处理工具
//
//  提供安全的错误处理方法，替代不安全的 unwrap() 和 expect()。
//
//  ## 目标
//
//  1. 减少代码库中 1,407 处 unwrap/expect 调用
//  2. 提供安全的替代方案
//  3. 保持代码简洁性
//  4. 提供更好的错误消息

use std::fmt;
use crate::error::{ErrorSeverity, EngineError};

/// 安全的 unwrap 替代品 - Option 版本
///
/// 当 Option 为 None 时，返回有意义的错误而非 panic。
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::safe_unwrap_option;
///
/// let maybe_value: Option<i32> = None;
/// let value = safe_unwrap_option(maybe_value, "player_id", "Player ID not found")?;
/// // 返回 Err 而非 panic
/// ```
pub fn safe_unwrap_option<T>(
    option: Option<T>,
    context: &str,
    error_msg: &str,
) -> Result<T, String> {
    option.ok_or_else(|| {
        format!("{}: {}", context, error_msg)
    })
}

/// 安全的 unwrap 替代品 - Result 版本
///
/// 提供更好的错误消息，当 Result 为 Err 时。
pub fn safe_unwrap_result<T, E: fmt::Display>(
    result: Result<T, E>,
    context: &str,
) -> Result<T, String> {
    result.map_err(|e| format!("{}: {}", context, e))
}

/// Option 的默认值或错误
///
/// 如果 Option 为 None，返回提供的默认值，否则返回 Some 中的值。
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::unwrap_or_default;
///
/// let maybe_value: Option<i32> = None;
/// let value = unwrap_or_default(maybe_value, 42);  // 使用默认值 42
/// ```
pub fn unwrap_or_default<T>(option: Option<T>, default: T) -> T {
    option.unwrap_or(default)
}

/// Option 的默认值或闭包
///
/// 如果 Option 为 None，通过闭包计算默认值。
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::unwrap_or_else_default;
///
/// let maybe_value: Option<i32> = None;
/// let value = unwrap_or_else_default(maybe_value, || expensive_calculation());
/// ```
pub fn unwrap_or_else_default<T, F: FnOnce() -> T>(option: Option<T>, default: F) -> T {
    option.unwrap_or_else(default)
}

/// Result unwrap 或带有上下文的错误
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::unwrap_or_context;
///
/// let result: Result<i32, &str> = Err("connection failed");
/// let value = unwrap_or_context(result, "Failed to connect to database");
/// // 返回 Err("Failed to connect to database: connection failed")
/// ```
pub fn unwrap_or_context<T, E: fmt::Display>(
    result: Result<T, E>,
    context: &str,
) -> Result<T, String> {
    result.map_err(|e| format!("{}: {}", context, e))
}

/// Vec get 或错误（替代 vec.get(i).unwrap()）
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::vec_get_or_err;
///
/// let vec = vec![1, 2, 3];
/// let value = vec_get_or_err(&vec, 5, "index out of bounds")?;
/// // 返回 Err("index out of bounds: index 5, length 3")
/// ```
pub fn vec_get_or_err<'a, T>(vec: &'a [T], index: usize, context: &str) -> Result<&'a T, String> {
    vec.get(index).ok_or_else(|| {
        format!("{}: index {} out of bounds for length {}", context, index, vec.len())
    })
}

/// Vec get mut 或错误
pub fn vec_get_mut_or_err<'a, T>(
    vec: &'a mut [T],
    index: usize,
    context: &str,
) -> Result<&'a mut T, String> {
    if index < vec.len() {
        Ok(&mut vec[index])
    } else {
        Err(format!(
            "{}: index {} out of bounds for length {}",
            context,
            index,
            vec.len()
        ))
    }
}

/// HashMap get 或错误（替代 map.get(k).unwrap()）
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::map_get_or_err;
/// use std::collections::HashMap;
///
/// let mut map = HashMap::new();
/// map.insert("player_1", 100);
///
/// let score = map_get_or_err(&map, &"player_2", "Player not found")?;
/// // 返回 Err("Player not found: key 'player_2' not found")
/// ```
pub fn map_get_or_err<'a, K: std::hash::Hash + Eq + fmt::Display, V>(
    map: &'a std::collections::HashMap<K, V>,
    key: &K,
    context: &str,
) -> Result<&'a V, String> {
    map.get(key).ok_or_else(|| {
        format!("{}: key '{}' not found", context, key)
    })
}

/// HashMap get mut 或错误
pub fn map_get_mut_or_err<'a, K: std::hash::Hash + Eq + fmt::Display, V>(
    map: &'a mut std::collections::HashMap<K, V>,
    key: &K,
    context: &str,
) -> Result<&'a mut V, String> {
    map.get_mut(key).ok_or_else(|| {
        format!("{}: key '{}' not found", context, key)
    })
}

/// 检查布尔值或错误
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::ok_or_else_err;
///
/// let is_valid = validate();
/// ok_or_else_err(is_valid, "Validation failed")?;
/// ```
pub fn ok_or_else_err(condition: bool, error_msg: &str) -> Result<(), String> {
    if condition {
        Ok(())
    } else {
        Err(error_msg.to_string())
    }
}

/// 检查数值范围或错误
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::check_range_or_err;
///
/// let value = 150;
/// check_range_or_err(value, 0..100, "Value out of range")?;
/// // 返回 Err("Value out of range: 150 not in range 0..100")
/// ```
pub fn check_range_or_err<T: fmt::Display + PartialOrd>(
    value: T,
    range: std::ops::Range<T>,
    context: &str,
) -> Result<(), String> {
    if value >= range.start && value < range.end {
        Ok(())
    } else {
        Err(format!(
            "{}: {} not in range {}..{}",
            context, value, range.start, range.end
        ))
    }
}

/// 检查非空或错误
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::check_non_empty_or_err;
///
/// let name = "";
/// check_non_empty_or_err(name, "Name cannot be empty")?;
/// // 返回 Err("Name cannot be empty")
/// ```
pub fn check_non_empty_or_err(s: &str, context: &str) -> Result<(), String> {
    if !s.is_empty() {
        Ok(())
    } else {
        Err(context.to_string())
    }
}

/// String 转 数字或错误
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::parse_to_number_or_err;
///
/// let s = "42";
/// let value: i32 = parse_to_number_or_err(s, "Failed to parse")?;
/// // 返回 Ok(42)
///
/// let s = "invalid";
/// let value: i32 = parse_to_number_or_err(s, "Failed to parse")?;
/// // 返回 Err("Failed to parse: invalid digit found in string")
/// ```
pub fn parse_to_number_or_err<T: std::str::FromStr>(
    s: &str,
    context: &str,
) -> Result<T, String> {
    s.parse::<T>().map_err(|_| format!("{}: '{}'", context, s))
}

/// Option 到 Result 的转换（与 error/traits.rs 中的 OptionExt 类似，但更简单）
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::option_to_result;
///
/// let maybe_value: Option<i32> = None;
/// let value = option_to_result(maybe_value, "Value not found")?;
/// // 返回 Err("Value not found")
/// ```
pub fn option_to_result<T>(option: Option<T>, error_msg: &str) -> Result<T, String> {
    option.ok_or_else(|| error_msg.to_string())
}

/// 带日志的安全 unwrap（开发时使用）
///
/// 在开发模式下，这会记录警告但仍允许 panic。
/// 在发布模式下，这会返回 Err。
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::safe_unwrap_with_log;
///
/// #[cfg(debug_assertions)]
/// let value = safe_unwrap_with_log(Some(42), "Expected value in debug mode")?;
/// ```
#[cfg(debug_assertions)]
pub fn safe_unwrap_with_log<T>(option: Option<T>, msg: &str) -> Result<T, String> {
    if let Some(value) = option {
        Ok(value)
    } else {
        eprintln!("[WARNING] unwrap_or_log: {}", msg);
        Err(format!("unwrap_or_log failed: {}", msg))
    }
}

/// 验证器链 - 用于组合多个验证
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::Validator;
///
/// let result = Validator::new()
///     .validate(|| value > 0, "Value must be positive")
///     .validate(|| value < 100, "Value must be less than 100")
///     .check()?;
/// ```
pub struct Validator {
    errors: Vec<String>,
}

impl Validator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
        }
    }

    /// 添加验证条件
    pub fn validate<F>(mut self, condition: F, error_msg: &str) -> Self
    where
        F: FnOnce() -> bool,
    {
        if !condition() {
            self.errors.push(error_msg.to_string());
        }
        self
    }

    /// 检查验证结果
    pub fn check(self) -> Result<(), String> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.join("; "))
        }
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.errors.len()
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

/// 可选值的日志记录（用于调试 None 值）
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::log_option;
///
/// let maybe_value: Option<i32> = None;
/// log_option(maybe_value, "player_position");
/// // 输出: [DEBUG] Option 'player_position' is None
/// ```
pub fn log_option<T>(option: Option<T>, label: &str) -> Option<T> {
    if option.is_none() {
        eprintln!("[DEBUG] Option '{}' is None", label);
    }
    option
}

/// Result 的日志记录（用于调试 Err 值）
///
/// # 示例
///
/// ```rust
/// use game_engine::error::convenience::log_result;
///
/// let result: Result<i32, &str> = Err("connection failed");
/// let logged_result = log_result(result, "database_connection");
/// // 输出: [DEBUG] Result 'database_connection' is Err: connection failed
/// ```
pub fn log_result<T, E: fmt::Display>(result: Result<T, E>, label: &str) -> Result<T, E> {
    if let Err(ref e) = result {
        eprintln!("[DEBUG] Result '{}' is Err: {}", label, e);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_unwrap_option_ok() {
        let value = Some(42);
        assert_eq!(safe_unwrap_option(value, "test", "error").unwrap(), 42);
    }

    #[test]
    fn test_safe_unwrap_option_err() {
        let value: Option<i32> = None;
        let result = safe_unwrap_option(value, "test context", "error message");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("test context"));
        assert!(result.unwrap_err().contains("error message"));
    }

    #[test]
    fn test_vec_get_or_err_ok() {
        let vec = vec![1, 2, 3];
        assert_eq!(*vec_get_or_err(&vec, 1, "test").unwrap(), 2);
    }

    #[test]
    fn test_vec_get_or_err_err() {
        let vec = vec![1, 2, 3];
        let result = vec_get_or_err(&vec, 5, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("index 5"));
        assert!(result.unwrap_err().contains("length 3"));
    }

    #[test]
    fn test_map_get_or_err_ok() {
        let mut map = std::collections::HashMap::new();
        map.insert("key1", 100);
        assert_eq!(*map_get_or_err(&map, &"key1", "test").unwrap(), 100);
    }

    #[test]
    fn test_map_get_or_err_err() {
        let mut map = std::collections::HashMap::new();
        map.insert("key1", 100);
        let result = map_get_or_err(&map, &"key2", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("key 'key2'"));
    }

    #[test]
    fn test_check_range_or_err_ok() {
        assert!(check_range_or_err(50, 0..100, "test").is_ok());
    }

    #[test]
    fn test_check_range_or_err_err() {
        let result = check_range_or_err(150, 0..100, "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("150"));
    }

    #[test]
    fn test_check_non_empty_or_err_ok() {
        assert!(check_non_empty_or_err("hello", "test").is_ok());
    }

    #[test]
    fn test_check_non_empty_or_err_err() {
        let result = check_non_empty_or_err("", "test");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "test");
    }

    #[test]
    fn test_parse_to_number_or_err_ok() {
        let result: Result<i32, String> = parse_to_number_or_err("42", "test");
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_parse_to_number_or_err_err() {
        let result: Result<i32, String> = parse_to_number_or_err("invalid", "test");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid"));
    }

    #[test]
    fn test_validator_ok() {
        let result = Validator::new()
            .validate(|| 5 > 0, "Value must be positive")
            .validate(|| 5 < 10, "Value must be less than 10")
            .check();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validator_err() {
        let result = Validator::new()
            .validate(|| -1 > 0, "Value must be positive")
            .validate(|| -1 < 10, "Value must be less than 10")
            .check();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Value must be positive"));
    }

    #[test]
    fn test_unwrap_or_default() {
        assert_eq!(unwrap_or_default(Some(42), 0), 42);
        assert_eq!(unwrap_or_default(None::<i32>, 0), 0);
    }

    #[test]
    fn test_unwrap_or_else_default() {
        assert_eq!(unwrap_or_else_default(Some(42), || 0), 42);
        assert_eq!(unwrap_or_else_default(None::<i32>, || 100), 100);
    }

    #[test]
    fn test_option_to_result() {
        assert_eq!(option_to_result(Some(42), "error").unwrap(), 42);
        assert!(option_to_result(None::<i32>, "error").is_err());
    }

    #[test]
    fn test_ok_or_else_err() {
        assert!(ok_or_else_err(true, "error").is_ok());
        assert!(ok_or_else_err(false, "error").is_err());
    }
}
