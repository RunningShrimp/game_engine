//  错误类型定义宏
//
//  提供宏来减少错误类型定义中的重复代码。

/// 简化的错误变体定义宏
///
/// 使用此宏可以快速定义带有严重级别字段的错误变体，
/// 以及对应的构造函数。
///
/// # 示例
///
/// ```rust
/// use crate::error::{ErrorSeverity, error_macros::define_error_variants};
///
/// define_error_variants!(
///     pub enum MyError {
///         #[error("Item not found: {id}")]
///         NotFound { id: String },
///
///         #[error("Operation failed: {message}")]
///         OperationFailed { message: String },
///     }
/// );
///
/// // 自动生成构造函数
/// let err = MyError::not_found("item_123");
/// let err2 = MyError::operation_failed("Connection timeout");
/// ```
#[macro_export]
macro_rules! define_error_variants {
    // 匹配错误定义并生成枚举和构造函数
    (
        $(#[$enum_meta:meta])*
        pub enum $error_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant_name:ident {
                    $($field:ident : $field_ty:ty),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        $(#[$enum_meta])*
        pub enum $error_name {
            $(
                $(#[$variant_meta])*
                $variant_name {
                    $(
                        $field : $field_ty
                    ),*
                    severity: $crate::error::ErrorSeverity,
                },
            )*
        }

        impl $error_name {
            $(
                #[inline]
                #[doc = concat!("创建", stringify!($variant_name), "错误")]
                pub fn $variant_name($($field: $field_ty),*) -> Self {
                    Self::$variant_name {
                        $(
                            $field,
                        )*
                        severity: $crate::error::ErrorSeverity::Error,
                    }
                }
            )*

            /// 获取错误的严重级别
            pub fn severity(&self) -> $crate::error::ErrorSeverity {
                match self {
                    $(
                        Self::$variant_name { severity, .. } => *severity,
                    )*
                }
            }

            /// 检查错误是否可恢复
            pub fn is_recoverable(&self) -> bool {
                self.severity() < $crate::error::ErrorSeverity::Critical
            }
        }
    };
}

/// 为错误类型实现分类方法的宏
///
/// # 示例
///
/// ```rust
/// use crate::error::error_macros::impl_error_categories;
///
/// impl_error_categories!(MyError, MyCategory);
///
/// // 生成方法
/// impl MyError {
///     pub fn category(&self) -> ErrorCategory {
///         ErrorCategory::MyCategory
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_error_categories {
    ($error_name:ty, $category:expr) => {
        impl $error_name {
            /// 获取错误分类
            pub fn category(&self) -> $crate::error::ErrorCategory {
                $category
            }
        }
    };
}

/// 为错误类型实现分类检查方法的宏
///
/// # 示例
///
/// ```rust
/// use crate::error::error_macros::impl_category_checks;
///
/// impl_category_checks!(MyError {
///     is_file_related => [NotFound, LoadFailed, InvalidFormat],
///     is_network_related => [Download, Upload, Streaming],
/// });
/// ```
#[macro_export]
macro_rules! impl_category_checks {
    (
        $error_name:ident {
            $(
                $check_method:ident => [$($variant:ident),* $(,)?],
            )*
        }
    ) => {
        impl $error_name {
            $(
                #[doc = concat!("检查是否为", stringify!($check_method), "错误")]
                pub fn $check_method(&self) -> bool {
                    matches!(
                        self,
                        $(
                            Self::$variant { .. }
                        ),* |
                        Self::General { .. }  // General错误总是返回true
                    )
                }
            )*
        }
    };
}

/// 定义带有自定义严重级别的错误变体
///
/// 当某些错误变体需要特定的严重级别时使用此宏。
///
/// # 示例
///
/// ```rust
/// use crate::error::{ErrorSeverity, error_macros::define_error_with_custom_severity};
///
/// define_error_with_custom_severity!(
///     pub enum CustomError {
///         #[error("Critical failure")]
///         CriticalFailure { message: String } [Critical],
///
///         #[error("Warning")]
///         Warning { message: String } [Warning],
///     }
/// );
/// ```
#[macro_export]
macro_rules! define_error_with_custom_severity {
    (
        $(#[$enum_meta:meta])*
        pub enum $error_name:ident {
            $(
                $(#[$variant_meta:meta])*
                $variant_name:ident {
                    $($field:ident : $field_ty:ty),* $(,)?
                } [$default_severity:ident],
            )*
        }
    ) => {
        $(#[$enum_meta])*
        pub enum $error_name {
            $(
                $(#[$variant_meta])*
                $variant_name {
                    $(
                        $field : $field_ty
                    ),*
                    severity: $crate::error::ErrorSeverity,
                },
            )*
        }

        impl $error_name {
            $(
                #[inline]
                #[doc = concat!("创建", stringify!($variant_name), "错误")]
                pub fn $variant_name($($field: $field_ty),*) -> Self {
                    Self::$variant_name {
                        $(
                            $field,
                        )*
                        severity: $crate::error::ErrorSeverity::$default_severity,
                    }
                }

                #[inline]
                #[doc = concat!("创建带有自定义严重级别的", stringify!($variant_name), "错误")]
                pub fn with_severity(variant: $error_name, severity: $crate::error::ErrorSeverity) -> Self {
                    match variant {
                        $(
                            Self::$variant_name { $($field),* .. } => Self::$variant_name {
                                $(
                                    $field,
                                )*
                                severity,
                            }
                        )*
                    }
                }
            )*

            /// 获取错误的严重级别
            pub fn severity(&self) -> $crate::error::ErrorSeverity {
                match self {
                    $(
                        Self::$variant_name { severity, .. } => *severity,
                    )*
                }
            }

            /// 检查错误是否可恢复
            pub fn is_recoverable(&self) -> bool {
                self.severity() < $crate::error::ErrorSeverity::Critical
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorSeverity;

    // 测试基本错误定义宏
    define_error_variants!(
        #[derive(Debug, Clone, thiserror::Error)]
        pub enum TestError {
            #[error("Item not found: {id}")]
            NotFound { id: String },

            #[error("Operation failed: {message}")]
            OperationFailed { message: String },

            #[error("Invalid parameter: {param} = {value}")]
            InvalidParameter { param: String, value: String },
        }
    );

    impl_error_categories!(TestError, ErrorCategory::Test);

    impl_category_checks!(TestError {
        is_item_related => [NotFound],
        is_operation_related => [OperationFailed, InvalidParameter],
    });

    #[test]
    fn test_error_macro_creation() {
        let err = TestError::not_found("item_123".to_string());
        assert!(err.is_item_related());
        assert_eq!(err.severity(), ErrorSeverity::Error);
        assert!(err.is_recoverable());
    }

    #[test]
    fn test_error_macro_multi_field() {
        let err = TestError::invalid_parameter("mass".to_string(), "-1.0".to_string());
        assert!(err.is_operation_related());
        assert_eq!(err.severity(), ErrorSeverity::Error);
    }

    #[test]
    fn test_custom_severity_macro() {
        define_error_with_custom_severity!(
            pub enum CustomSeverityError {
                #[error("Critical failure")]
                CriticalFailure { message: String } [Critical],

                #[error("Warning")]
                Warning { message: String } [Warning],
            }
        );

        let crit = CustomSeverityError::critical_failure("System crash".to_string());
        assert_eq!(crit.severity(), ErrorSeverity::Critical);
        assert!(!crit.is_recoverable());

        let warn = CustomSeverityError::warning("Deprecated API".to_string());
        assert_eq!(warn.severity(), ErrorSeverity::Warning);
        assert!(warn.is_recoverable());
    }
}
