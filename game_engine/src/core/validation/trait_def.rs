//! 验证trait定义

use super::error::{ValidationError, ValidationResult};

/// 可验证对象的trait
///
/// 实现此trait的类型可以验证其内部状态的有效性。
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::{Validate, ValidationError, validators};
///
/// struct EntityConfig {
///     name: String,
///     id: u64,
///     health: f32,
/// }
///
/// impl Validate for EntityConfig {
///     type Error = ValidationError;
///
///     fn validate(&self) -> Result<(), Self::Error> {
///         // 验证名称非空
///         validators::validate_non_empty(&self.name)?;
///
///         // 验证ID范围
///         validators::validate_range(self.id, 0, 10000)?;
///
///         // 验证健康值
///         validators::validate_range(self.health, 0.0, 100.0)?;
///         validators::validate_finite(self.health)?;
///
///         Ok(())
///     }
/// }
///
/// let config = EntityConfig {
///     name: "Hero".to_string(),
///     id: 100,
///     health: 100.0,
/// };
///
/// assert!(config.validate().is_ok());
/// ```
pub trait Validate {
    /// 验证错误类型
    type Error: std::error::Error + Send + Sync + 'static;

    /// 验证对象状态
    ///
    /// 如果验证成功，返回`Ok(())`。
    /// 如果验证失败，返回描述问题的错误。
    fn validate(&self) -> Result<(), Self::Error>;
}

/// 为Option实现Validate
impl<T> Validate for Option<T>
where
    T: Validate,
{
    type Error = T::Error;

    fn validate(&self) -> Result<(), Self::Error> {
        if let Some(value) = self {
            value.validate()?;
        }
        Ok(())
    }
}

/// 为Vec实现Validate（验证所有元素）
impl<T> Validate for Vec<T>
where
    T: Validate,
{
    type Error = T::Error;

    fn validate(&self) -> Result<(), Self::Error> {
        for (i, item) in self.iter().enumerate() {
            item.validate().map_err(|e| {
                // TODO: 将索引添加到错误上下文
                e
            })?;
        }
        Ok(())
    }
}

/// 为Result实现Validate（如果包含Ok值，则验证）
impl<T, E> Validate for Result<T, E>
where
    T: Validate,
    E: std::error::Error + Send + Sync + 'static,
{
    type Error = ValidationError;

    fn validate(&self) -> Result<(), ValidationError> {
        if let Ok(value) = self {
            value.validate().map_err(|e| ValidationError::custom(e.to_string()))?;
        }
        Ok(())
    }
}

/// 验证辅助宏（简化常见验证模式）
///
/// # 示例
///
/// ```
/// use game_engine::core::validation::{validate, validators};
///
/// fn create_entity(name: &str, id: u64) -> Result<(), ValidationError> {
///     validate! {
///         "name" => validators::validate_non_empty(name),
///         "id" => validators::validate_range(id, 0, 10000),
///     }
/// }
/// ```
#[macro_export]
macro_rules! validate {
    // 单个验证
    ($field:expr => $validator:expr) => {
        $validator.map_err(|e| {
            $crate::core::validation::ValidationError::custom(format!(
                "{}: {}",
                stringify!($field),
                e
            ))
        })?;
    };

    // 多个验证
    ($($field:expr => $validator:expr),* $(,)?) => {
        $(
            $crate::validate!($field => $validator);
        )*
    };
}

/// 验证集合辅助宏
#[macro_export]
macro_rules! validate_all {
    ($($item:expr),* $(,)?) => {
        $(
            $item.validate()?;
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::validation::validators;

    struct TestStruct {
        name: String,
        value: i32,
    }

    impl Validate for TestStruct {
        type Error = ValidationError;

        fn validate(&self) -> Result<(), Self::Error> {
            validators::validate_non_empty(&self.name)?;
            validators::validate_range(self.value, 0, 100)?;
            Ok(())
        }
    }

    #[test]
    fn test_validate_impl() {
        let valid = TestStruct {
            name: "test".to_string(),
            value: 50,
        };
        assert!(valid.validate().is_ok());

        let invalid_name = TestStruct {
            name: "".to_string(),
            value: 50,
        };
        assert!(invalid_name.validate().is_err());

        let invalid_value = TestStruct {
            name: "test".to_string(),
            value: 150,
        };
        assert!(invalid_value.validate().is_err());
    }

    #[test]
    fn test_validate_option() {
        let some_value: Option<TestStruct> = Some(TestStruct {
            name: "test".to_string(),
            value: 50,
        });
        assert!(some_value.validate().is_ok());

        let none_value: Option<TestStruct> = None;
        assert!(none_value.validate().is_ok());

        let some_invalid: Option<TestStruct> = Some(TestStruct {
            name: "".to_string(),
            value: 50,
        });
        assert!(some_invalid.validate().is_err());
    }

    #[test]
    fn test_validate_vec() {
        let valid_vec = vec![
            TestStruct {
                name: "test1".to_string(),
                value: 50,
            },
            TestStruct {
                name: "test2".to_string(),
                value: 60,
            },
        ];
        assert!(valid_vec.validate().is_ok());

        let invalid_vec = vec![
            TestStruct {
                name: "test1".to_string(),
                value: 50,
            },
            TestStruct {
                name: "".to_string(),
                value: 60,
            },
        ];
        assert!(invalid_vec.validate().is_err());
    }
}
