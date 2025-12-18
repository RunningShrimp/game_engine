//! 核心宏定义
//!
//! 提供统一的宏来减少代码重复

/// 为结构体实现Default trait的宏
///
/// 使用示例:
/// ```rust
/// struct MyStruct {
///     field1: u32,
///     field2: String,
/// }
///
/// impl_default!(MyStruct {
///     field1: 0,
///     field2: String::new(),
/// });
/// ```
#[macro_export]
macro_rules! impl_default {
    ($struct_name:ident {
        $($field:ident: $value:expr),* $(,)?
    }) => {
        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    $($field: $value),*
                }
            }
        }
    };
}

/// 为结构体实现new()构造函数的宏
///
/// 使用示例:
/// ```rust
/// struct MyStruct {
///     field1: u32,
///     field2: String,
/// }
///
/// impl_new!(MyStruct {
///     field1: 0,
///     field2: String::new(),
/// });
/// ```
#[macro_export]
macro_rules! impl_new {
    ($struct_name:ident {
        $($field:ident: $value:expr),* $(,)?
    }) => {
        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    $($field: $value),*
                }
            }
        }
    };
}

/// 同时实现Default和new()的宏
///
/// 使用示例:
/// ```rust
/// struct MyStruct {
///     field1: u32,
///     field2: String,
/// }
///
/// impl_default_and_new!(MyStruct {
///     field1: 0,
///     field2: String::new(),
/// });
/// ```
#[macro_export]
macro_rules! impl_default_and_new {
    ($struct_name:ident {
        $($field:ident: $value:expr),* $(,)?
    }) => {
        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    $($field: $value),*
                }
            }
        }

        impl $struct_name {
            pub fn new() -> Self {
                Self::default()
            }
        }
    };
}

#[cfg(test)]
mod tests {

    struct TestStruct {
        field1: u32,
        field2: String,
    }

    impl_default_and_new!(TestStruct {
        field1: 0,
        field2: String::new(),
    });

    #[test]
    fn test_impl_default_and_new() {
        let s1 = TestStruct::default();
        let s2 = TestStruct::new();

        assert_eq!(s1.field1, 0);
        assert_eq!(s1.field2, "");
        assert_eq!(s2.field1, 0);
        assert_eq!(s2.field2, "");
    }
}
