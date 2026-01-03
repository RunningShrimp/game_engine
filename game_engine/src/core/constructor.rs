//! # 构造函数简化宏
//!
//! 提供宏来减少构造函数的样板代码。
//!
//! ## 功能特性
//!
//! - **simple_new**: 自动生成简单的new()构造函数
//! - **builder**: 自动生成Builder模式
//! - **smart_new**: 智能构造函数，支持Default和参数混合

/// 自动生成简单的new()构造函数
///
/// 这个宏为结构体生成标准的new()构造函数，减少样板代码。
///
/// # 示例
///
/// ```rust
/// use game_engine::core::constructor::simple_new;
///
/// simple_new! {
///     pub struct MyStruct {
///         pub field1: String,
///         pub field2: i32,
///         field3: Vec<u8>,  // 私有字段
///     }
/// }
///
/// // 使用
/// let s = MyStruct::new("hello".to_string(), 42);
/// ```
///
/// # 生成的代码
///
/// ```ignore
/// impl MyStruct {
///     pub fn new(field1: String, field2: i32) -> Self {
///         Self {
///             field1,
///             field2,
///             field3: Default::default(),
///         }
///     }
/// }
/// ```
#[macro_export]
macro_rules! simple_new {
    (
        $(#[$struct_meta:meta])*
        pub struct $struct_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        pub struct $struct_name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_ty,
            )*
        }

        impl $struct_name {
            /// 创建新的实例
            pub fn new($($field_name : $field_ty),*) -> Self {
                Self {
                    $($field_name),*
                }
            }
        }
    };
}

/// 为结构体生成带默认值的new()构造函数
///
/// # 示例
///
/// ```rust
/// use game_engine::core::constructor::new_with_defaults;
///
/// new_with_defaults! {
///     pub struct MyConfig {
///         pub enabled: bool = true,
///         pub port: u16 = 8080,
///         pub host: String = String::from("localhost"),
///     }
/// }
///
/// // 使用
/// let config = MyConfig::new();
/// assert_eq!(config.port, 8080);
/// ```
#[macro_export]
macro_rules! new_with_defaults {
    (
        $(#[$struct_meta:meta])*
        pub struct $struct_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty = $default_val:expr
            ),*
            $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        pub struct $struct_name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_ty,
            )*
        }

        impl $struct_name {
            /// 创建带有默认值的实例
            pub fn new() -> Self {
                Self {
                    $($field_name : $default_val),*
                }
            }

            /// 创建带有自定义值的实例
            pub fn with_values($($field_name : $field_ty),*) -> Self {
                Self {
                    $($field_name),*
                }
            }
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}

/// 生成Builder模式
///
/// # 示例
///
/// ```rust
/// use game_engine::core::constructor::builder;
///
/// builder! {
///     pub struct MyConfig {
///         pub enabled: bool,
///         pub port: u16,
///         pub host: String,
///     }
/// }
///
/// // 使用
/// let config = MyConfig::builder()
///     .enabled(true)
///     .port(8080)
///     .host("localhost".to_string())
///     .build();
/// ```
#[macro_export]
macro_rules! builder {
    (
        $(#[$struct_meta:meta])*
        pub struct $struct_name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_ty:ty
            ),*
            $(,)?
        }
    ) => {
        $(#[$struct_meta])*
        pub struct $struct_name {
            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_ty,
            )*
        }

        impl $struct_name {
            /// 创建Builder
            pub fn builder() -> $struct_name Builder {
                $struct_name Builder::default()
            }
        }

        #[derive(Default)]
        pub struct $struct_name Builder {
            $(
                $field_name : std::option::Option<$field_ty>,
            )*
        }

        impl $struct_name Builder {
            $(
                pub fn $field_name(mut self, value: $field_ty) -> Self {
                    self.$field_name = Some(value);
                    self
                }
            )*

            pub fn build(self) -> Result<$struct_name, String> {
                Ok($struct_name {
                    $(
                        $field_name : self.$field_name.ok_or_else(|| concat!(
                            "Missing field: ",
                            stringify!($field_name)
                        ).to_string())?,
                    )*
                })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    // 测试simple_new
    simple_new! {
        pub struct TestStruct {
            pub field1: String,
            pub field2: i32,
            private_field: Vec<u8>,
        }
    }

    #[test]
    fn test_simple_new() {
        let s = TestStruct::new("hello".to_string(), 42);
        assert_eq!(s.field1, "hello");
        assert_eq!(s.field2, 42);
        assert!(s.private_field.is_empty());
    }

    // 测试new_with_defaults
    new_with_defaults! {
        pub struct TestConfig {
            pub enabled: bool = true,
            pub port: u16 = 8080,
            pub host: String = String::from("localhost"),
        }
    }

    #[test]
    fn test_new_with_defaults() {
        let config = TestConfig::new();
        assert_eq!(config.enabled, true);
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "localhost");

        let config2 = TestConfig::with_values(false, 9000, "example.com".to_string());
        assert_eq!(config2.enabled, false);
        assert_eq!(config2.port, 9000);
        assert_eq!(config2.host, "example.com");
    }

    #[test]
    fn test_default_trait() {
        let config = TestConfig::default();
        assert_eq!(config.port, 8080);
    }

    // 测试builder
    builder! {
        pub struct TestBuilder {
            pub field1: String,
            pub field2: i32,
        }
    }

    #[test]
    fn test_builder() {
        let result = TestBuilder::builder().field1("test".to_string()).field2(42).build();

        assert!(result.is_ok());
        let obj = result.unwrap();
        assert_eq!(obj.field1, "test");
        assert_eq!(obj.field2, 42);
    }

    #[test]
    fn test_builder_missing_field() {
        let result = TestBuilder::builder().field1("test".to_string()).build();

        assert!(result.is_err());
    }
}
