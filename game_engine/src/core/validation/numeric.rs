//! 数值验证器

use super::error::{ValidationError, ValidationResult};
use std::fmt::Display;

/// 验证数值在指定范围内
///
/// # 参数
/// - `value`: 要验证的值
/// - `min`: 最小值（包含）
/// - `max`: 最大值（包含）
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_range;
///
/// assert!(validate_range(5u32, 0, 10).is_ok());
/// assert!(validate_range(15u32, 0, 10).is_err());
/// assert!(validate_range(-1i32, 0, 10).is_err());
/// ```
pub fn validate_range<T>(value: T, min: T, max: T) -> ValidationResult<T>
where
    T: PartialOrd + Copy + Display,
{
    if value < min || value > max {
        Err(ValidationError::out_of_range(value, min, max))
    } else {
        Ok(value)
    }
}

/// 验证f32是有限值（非NaN且非无限）
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_finite;
///
/// assert!(validate_finite(1.0).is_ok());
/// assert!(validate_finite(f32::INFINITY).is_err());
/// assert!(validate_finite(f32::NAN).is_err());
/// ```
pub fn validate_finite(value: f32) -> ValidationResult<f32> {
    if !value.is_finite() {
        if value.is_nan() {
            return Err(ValidationError::NaN(value));
        } else {
            return Err(ValidationError::NonFinite(value));
        }
    }
    Ok(value)
}

/// 验证f32非NaN
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::validators::validate_nan;
///
/// assert!(validate_nan(1.0).is_ok());
/// assert!(validate_nan(f32::NAN).is_err());
/// ```
pub fn validate_nan(value: f32) -> ValidationResult<f32> {
    if value.is_nan() {
        return Err(ValidationError::NaN(value));
    }
    Ok(value)
}

/// 验证数值非负（整数版本）
pub fn validate_non_negative_i32(value: i32) -> ValidationResult<i32> {
    if value < 0 {
        return Err(ValidationError::Negative(value.to_string()));
    }
    Ok(value)
}

/// 验证数值非负（浮点数版本）
pub fn validate_non_negative_f32(value: f32) -> ValidationResult<f32> {
    if value < 0.0 {
        return Err(ValidationError::Negative(value.to_string()));
    }
    Ok(value)
}

/// 验证数值非零（整数版本）
pub fn validate_non_zero_i32(value: i32) -> ValidationResult<i32> {
    if value == 0 {
        return Err(ValidationError::Zero);
    }
    Ok(value)
}

/// 验证数值非零（浮点数版本）
pub fn validate_non_zero_f32(value: f32) -> ValidationResult<f32> {
    if value == 0.0 {
        return Err(ValidationError::Zero);
    }
    Ok(value)
}

/// 验证数值为正（整数版本）
pub fn validate_positive_i32(value: i32) -> ValidationResult<i32> {
    if value <= 0 {
        return Err(ValidationError::Negative(value.to_string()));
    }
    Ok(value)
}

/// 验证数值为正（浮点数版本）
pub fn validate_positive_f32(value: f32) -> ValidationResult<f32> {
    if value <= 0.0 {
        return Err(ValidationError::Negative(value.to_string()));
    }
    Ok(value)
}

/// 验证偶数
pub fn validate_even(value: i64) -> ValidationResult<i64> {
    if value % 2 != 0 {
        return Err(ValidationError::custom(format!(
            "Value {} is not an even number",
            value
        )));
    }
    Ok(value)
}

/// 验证奇数
pub fn validate_odd(value: i64) -> ValidationResult<i64> {
    if value % 2 == 0 {
        return Err(ValidationError::custom(format!(
            "Value {} is not an odd number",
            value
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_range() {
        // 整数范围
        assert!(validate_range(5u32, 0, 10).is_ok());
        assert!(validate_range(0u32, 0, 10).is_ok());
        assert!(validate_range(10u32, 0, 10).is_ok());

        assert!(validate_range(11u32, 0, 10).is_err());
        assert!(validate_range(-1i32, 0, 10).is_err());

        // 浮点数范围
        assert!(validate_range(0.5f32, 0.0, 1.0).is_ok());
        assert!(validate_range(1.5f32, 0.0, 1.0).is_err());
    }

    #[test]
    fn test_validate_finite() {
        assert!(validate_finite(0.0).is_ok());
        assert!(validate_finite(1.0).is_ok());
        assert!(validate_finite(-1.0).is_ok());

        assert!(validate_finite(f32::INFINITY).is_err());
        assert!(validate_finite(f32::NEG_INFINITY).is_err());
        assert!(validate_finite(f32::NAN).is_err());
    }

    #[test]
    fn test_validate_non_negative() {
        assert!(validate_non_negative_i32(0).is_ok());
        assert!(validate_non_negative_i32(1).is_ok());
        assert!(validate_non_negative_f32(0.0).is_ok());
        assert!(validate_non_negative_f32(1.0).is_ok());

        assert!(validate_non_negative_i32(-1).is_err());
        assert!(validate_non_negative_f32(-1.0).is_err());
    }

    #[test]
    fn test_validate_positive() {
        assert!(validate_positive_i32(1).is_ok());
        assert!(validate_positive_f32(1.0).is_ok());

        assert!(validate_positive_i32(0).is_err());
        assert!(validate_positive_f32(0.0).is_err());
        assert!(validate_positive_i32(-1).is_err());
    }

    #[test]
    fn test_validate_even_odd() {
        assert!(validate_even(2).is_ok());
        assert!(validate_even(4).is_ok());
        assert!(validate_even(3).is_err());

        assert!(validate_odd(3).is_ok());
        assert!(validate_odd(5).is_ok());
        assert!(validate_odd(2).is_err());
    }
}
