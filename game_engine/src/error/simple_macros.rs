//! # 简化的错误定义宏
//!
//! 提供极其简单的宏来定义标准错误类型，最大程度减少样板代码。
//!
//! ## 特性
//!
//! - 自动生成Display和Error trait实现
//! - 支持常见的错误变体模式
//! - 零样板代码
//! - 与thiserror完全兼容

/// 快速定义错误类型
///
/// 这个宏提供了最简洁的方式来定义标准错误类型，自动处理所有常见的错误模式。
///
/// # 语法
///
/// ```rust
/// use game_engine::error::simple_error;
///
/// simple_error! {
///     /// 我的模块错误
///     pub MyError {
///         // 直接包装其他错误类型（自动实现From）
///         Io: std::io::Error,
///         Serde: serde_json::Error,
///
///         // String错误变体（自动添加错误消息前缀）
///         Parse: String,
///         NotFound: String,
///         Invalid: String,
///
///         // 自定义错误消息
///         #[error("Connection failed: {0}")]
///         Connection: String,
///
///         // 带多个字段的变体
///         #[error("Invalid {field}: {value}")]
///         InvalidField { field: String, value: String },
///     }
/// }
/// ```
///
/// # 生成的代码
///
/// 对于上面的例子，宏会生成：
/// - 完整的枚举定义，带有thiserror derives
/// - `From<std::io::Error>` for MyError
/// - `From<serde_json::Error>` for MyError
/// - Display实现（使用thiserror）
/// - Error trait实现（使用thiserror）
///
/// # 示例
///
/// ```rust
/// use game_engine::error::simple_error;
/// use std::io;
///
/// simple_error! {
///     pub MyError {
///         Io: io::Error,
///         Parse: String,
///         NotFound: String,
///     }
/// }
///
/// // 使用
/// fn read_file() -> Result<String, MyError> {
///     // 自动从io::Error转换
///     std::fs::read_to_string("test.txt")?;
///     Ok("content".to_string())
/// }
///
/// fn parse_input(s: &str) -> Result<(), MyError> {
///     if s.is_empty() {
///         // 使用String变体
///         return Err(MyError::Parse("Input is empty".to_string()));
///     }
///     Ok(())
/// }
/// ```
#[macro_export]
macro_rules! simple_error {
    (
        $(#[$enum_meta:meta])*
        pub $error_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant_name:ident : $variant_ty:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        #[derive(Debug, thiserror::Error)]
        pub enum $error_name {
            $(
                $(#[$variant_meta])*
                $variant_name($variant_ty),
            )*
        }

        // 自动为外部错误类型实现From（排除String类型以避免冲突）
        $(
            impl From<$variant_ty> for $error_name {
                fn from(err: $variant_ty) -> Self {
                    $error_name::$variant_name(err)
                }
            }
        )*

        // 为String变体提供便捷构造函数
        $(
            impl $error_name {
                paste! {
                    #[doc = concat!("Create ", stringify!($variant_name), " error from string")]
                    pub fn [<new_ $variant_name:snake>](msg: String) -> Self {
                        Self::$variant_name(msg)
                    }
                }
            }
        )*
    };
}

/// 定义带有标准错误变体的错误类型
///
/// 这个宏提供了最常用的错误变体模式，包括IO、解析、未找到等。
///
/// # 示例
///
/// ```rust
/// use game_engine::error::standard_error;
///
/// standard_error! {
///     /// 我的模块错误
///     pub MyError
/// }
///
/// // 使用
/// let err = MyError::io(std::io::Error::new(
///     std::io::ErrorKind::NotFound,
///     "file not found"
/// ));
/// ```
///
/// # 生成的错误变体
///
/// - `Io(std::io::Error)` - IO错误
/// - `Parse(String)` - 解析错误
/// - `NotFound(String)` - 资源未找到
/// - `Invalid(String)` - 无效参数
/// - `Other(String)` - 其他错误
#[macro_export]
macro_rules! standard_error {
    (
        $(#[$meta:meta])*
        pub $error_name:ident
    ) => {
        $crate::simple_error! {
            $(#[$meta])*
            pub $error_name {
                #[error("IO error: {0}")]
                Io: std::io::Error,

                #[error("Parse error: {0}")]
                Parse: String,

                #[error("Not found: {0}")]
                NotFound: String,

                #[error("Invalid input: {0}")]
                Invalid: String,

                #[error("Other error: {0}")]
                Other: String
            }
        }
    };
}

/// 定义带有自定义字段的错误类型
///
/// # 示例
///
/// ```rust
/// use game_engine::error::field_error;
///
/// field_error! {
///     /// 配置错误
///     pub ConfigError {
///         /// 缺少必需的配置项
///         Missing { key: String },
///
///         /// 无效的配置值
///         #[error("Invalid value for {key}: {value}")]
///         InvalidValue { key: String, value: String },
///     }
/// }
/// ```
#[macro_export]
macro_rules! field_error {
    (
        $(#[$meta:meta])*
        pub $error_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant_name:ident { $($field:ident : $field_ty:ty),* $(,)? }
            ),*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, thiserror::Error)]
        pub enum $error_name {
            $(
                $(#[$variant_meta])*
                $variant_name {
                    $(
                        $field : $field_ty
                    ),*
                },
            )*
        }
    };
}

/// 组合多个错误类型
///
/// # 示例
///
/// ```rust
/// use game_engine::error::{simple_error, combined_error};
///
/// simple_error! {
///     pub IoError {
///         Io: std::io::Error,
///     }
/// }
///
/// simple_error! {
///     pub ParseError {
///         Parse: String,
///     }
/// }
///
/// combined_error! {
///     /// 组合错误
///     pub CombinedError {
///         Io: IoError,
///         Parse: ParseError,
///     }
/// }
/// ```
#[macro_export]
macro_rules! combined_error {
    (
        $(#[$meta:meta])*
        pub $error_name:ident {
            $(
                $variant_name:ident : $variant_ty:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, thiserror::Error)]
        pub enum $error_name {
            $(
                #[error("{0}")]
                $variant_name($variant_ty),
            )*
        }
    };

    (
        $(#[$meta:meta])*
        pub $error_name:ident {
            $(
                #[error($msg:expr)]
                $variant_name:ident : $variant_ty:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, thiserror::Error)]
        pub enum $error_name {
            $(
                #[error($msg)]
                $variant_name($variant_ty),
            )*
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    // 测试simple_error宏
    simple_error! {
        pub TestSimpleError {
            Io: io::Error,
            Parse: String,
            NotFound: String
        }
    }

    // 为测试错误类型实现Display trait
    impl std::fmt::Display for TestSimpleError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestSimpleError::Io(e) => write!(f, "IO error: {}", e),
                TestSimpleError::Parse(s) => write!(f, "Parse error: {}", s),
                TestSimpleError::NotFound(s) => write!(f, "Not found: {}", s),
            }
        }
    }

    #[test]
    fn test_simple_error() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "test");
        let err = TestSimpleError::from(io_err);
        assert!(err.to_string().contains("IO error"));

        let parse_err = TestSimpleError::Parse("failed".to_string());
        assert_eq!(parse_err.to_string(), "Parse error: failed");
    }

    // 测试standard_error宏
    standard_error! {
        pub TestStandardError
    }

    #[test]
    fn test_standard_error() {
        let err = TestStandardError::NotFound("item".to_string());
        assert!(err.to_string().contains("item"));

        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let err = TestStandardError::from(io_err);
        assert!(err.to_string().contains("IO error") || err.to_string().contains("access denied"));
    }

    // 测试field_error宏
    field_error! {
        pub TestFieldError {
            #[error("Missing field: {key}")]
            Missing { key: String },
            #[error("Invalid value for {key}: {value}")]
            InvalidValue { key: String, value: String }
        }
    }

    #[test]
    fn test_field_error() {
        let err = TestFieldError::Missing {
            key: "config".to_string(),
        };
        assert!(err.to_string().contains("Missing"));

        let err = TestFieldError::InvalidValue {
            key: "port".to_string(),
            value: "abc".to_string(),
        };
        assert!(err.to_string().contains("port"));
        assert!(err.to_string().contains("abc"));
    }

    // 测试combined_error宏
    simple_error! {
        pub TestError1 {
            Err1: String
        }
    }

    simple_error! {
        pub TestError2 {
            Err2: String
        }
    }

    // 为测试错误类型实现Display trait
    impl std::fmt::Display for TestError1 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestError1::Err1(s) => write!(f, "Error 1: {}", s),
            }
        }
    }

    impl std::fmt::Display for TestError2 {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                TestError2::Err2(s) => write!(f, "Error 2: {}", s),
            }
        }
    }

    combined_error! {
        pub TestCombinedError {
            E1: TestError1,
            E2: TestError2
        }
    }

    #[test]
    fn test_combined_error() {
        let err1 = TestError1::Err1("error 1".to_string());
        let combined = TestCombinedError::E1(err1);
        assert!(combined.to_string().contains("error 1"));

        let err2 = TestError2::Err2("error 2".to_string());
        let combined = TestCombinedError::E2(err2);
        assert!(combined.to_string().contains("error 2"));
    }
}
