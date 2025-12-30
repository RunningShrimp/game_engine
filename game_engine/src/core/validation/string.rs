//! 字符串验证器

use super::error::{ValidationError, ValidationResult};

/// 验证字符串非空
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_non_empty;
///
/// assert!(validate_non_empty("test").is_ok());
/// assert!(validate_non_empty("").is_err());
/// ```
pub fn validate_non_empty(s: &str) -> ValidationResult<&str> {
    if s.is_empty() {
        Err(ValidationError::EmptyString)
    } else {
        Ok(s)
    }
}

/// 验证字符串长度
///
/// # 参数
/// - `s`: 要验证的字符串
/// - `min`: 最小长度（字符数）
/// - `max`: 最大长度（字符数）
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_length;
///
/// assert!(validate_length("test", 1, 10).is_ok());
/// assert!(validate_length("", 1, 10).is_err());  // 太短
/// assert!(validate_length("abcdefghijk", 1, 10).is_err());  // 太长
/// ```
pub fn validate_length(s: &str, min: usize, max: usize) -> ValidationResult<&str> {
    let len = s.chars().count();

    if len < min {
        Err(ValidationError::TooShort { min, actual: len })
    } else if len > max {
        Err(ValidationError::TooLong { max, actual: len })
    } else {
        Ok(s)
    }
}

/// 验证字符串只包含允许的字符集
///
/// # 参数
/// - `s`: 要验证的字符串
/// - `allowed`: 允许的字符集合
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_charset;
///
/// assert!(validate_charset("abc123", "abcdefghijklmnopqrstuvwxyz0123456789").is_ok());
/// assert!(validate_charset("abc!", "abcdefghijklmnopqrstuvwxyz").is_err());
/// ```
pub fn validate_charset<'a>(s: &'a str, allowed: &str) -> ValidationResult<&'a str> {
    if !s.chars().all(|c| allowed.contains(c)) {
        return Err(ValidationError::InvalidCharacters);
    }
    Ok(s)
}

/// 验证字符串是有效的UTF-8
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_utf8;
///
/// assert!(validate_utf8(b"hello").is_ok());
/// assert!(validate_utf8(b"\xff\xfe").is_err());  // 无效UTF-8
/// ```
pub fn validate_utf8(bytes: &[u8]) -> ValidationResult<&str> {
    std::str::from_utf8(bytes).map_err(|_| ValidationError::custom("Invalid UTF-8 byte sequence"))
}

/// 验证字符串是有效的标识符（字母开头，只包含字母数字下划线）
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_identifier;
///
/// assert!(validate_identifier("test_123").is_ok());
/// assert!(validate_identifier("123test").is_err());  // 数字开头
/// assert!(validate_identifier("test-123").is_err());  // 包含连字符
/// ```
pub fn validate_identifier(s: &str) -> ValidationResult<&str> {
    if s.is_empty() {
        return Err(ValidationError::EmptyString);
    }

    if !s.chars().next().unwrap().is_alphabetic() {
        return Err(ValidationError::custom(
            "Identifier must start with a letter",
        ));
    }

    if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(ValidationError::InvalidCharacters);
    }

    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_non_empty() {
        assert!(validate_non_empty("test").is_ok());
        assert!(validate_non_empty("a").is_ok());
        assert!(validate_non_empty("").is_err());
    }

    #[test]
    fn test_validate_length() {
        assert!(validate_length("test", 1, 10).is_ok());
        assert!(validate_length("a", 1, 10).is_ok());
        assert!(validate_length("abcdefghij", 1, 10).is_ok());

        assert!(validate_length("", 1, 10).is_err());
        assert!(validate_length("abcdefghijk", 1, 10).is_err());

        // Unicode字符
        assert!(validate_length("你好", 1, 10).is_ok());
        assert!(validate_length("你好世界", 1, 3).is_err());
    }

    #[test]
    fn test_validate_charset() {
        assert!(validate_charset("abc123", "abcdefghijklmnopqrstuvwxyz0123456789").is_ok());
        assert!(validate_charset("ABC", "ABCDEFGHIJKLMNOPQRSTUVWXYZ").is_ok());

        assert!(validate_charset("abc!", "abcdefghijklmnopqrstuvwxyz").is_err());
        assert!(validate_charset("123", "abcdefghijklmnopqrstuvwxyz").is_err());
    }

    #[test]
    fn test_validate_utf8() {
        assert!(validate_utf8(b"hello").is_ok());
        assert!(validate_utf8("hello".as_bytes()).is_ok());
        assert!(validate_utf8("你好".as_bytes()).is_ok());

        assert!(validate_utf8(b"\xff\xfe").is_err());
        assert!(validate_utf8(b"\xc3\x28").is_err()); // 无效UTF-8
    }

    #[test]
    fn test_validate_identifier() {
        assert!(validate_identifier("test").is_ok());
        assert!(validate_identifier("test_123").is_ok());
        assert!(validate_identifier("_test").is_err()); // 下划线开头
        assert!(validate_identifier("123test").is_err()); // 数字开头
        assert!(validate_identifier("test-123").is_err()); // 连字符
    }
}
