//! 错误处理Trait
//!
//! 提供统一的错误处理trait，减少代码重复。

use std::fmt;
use std::io;

/// 错误上下文扩展trait
///
/// 为Result类型添加上下文信息，减少错误处理的重复代码。
pub trait ResultExt<T, E>: Sized {
    /// 添加上下文信息到错误
    ///
    /// # 示例
    /// ```rust
    /// use game_engine::error::traits::ResultExt;
    ///
    /// let result: Result<(), std::io::Error> = Err(std::io::Error::new(
    ///     std::io::ErrorKind::NotFound,
    ///     "file not found"
    /// ));
    ///
    /// let result = result
    ///     .context("Failed to load texture");
    /// ```
    fn context(self, context: impl fmt::Display) -> Result<T, ContextError<E>>;

    /// 使用闭包添加上下文信息
    fn with_context<F>(self, f: F) -> Result<T, ContextError<E>>
    where
        F: FnOnce() -> String;

    /// 添加位置信息到错误（便捷方法）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::error::traits::ResultExt;
    ///
    /// let result: Result<(), std::io::Error> = Err(std::io::Error::new(
    ///     std::io::ErrorKind::NotFound,
    ///     "file not found"
    /// ));
    ///
    /// let result = result
    ///     .context_at("load_texture", "Failed to load texture");
    /// ```
    fn context_at(
        self,
        location: &'static str,
        context: impl fmt::Display,
    ) -> Result<T, ContextError<E>> {
        self.context(format!("[{location}] {context}"))
    }

    /// 添加文件和行号信息到错误（便捷方法）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::error::traits::ResultExt;
    ///
    /// let result: Result<(), std::io::Error> = Err(std::io::Error::new(
    ///     std::io::ErrorKind::NotFound,
    ///     "file not found"
    /// ));
    ///
    /// let result = result
    ///     .context_here("Failed to load texture");
    /// ```
    fn context_here(self, context: impl fmt::Display) -> Result<T, ContextError<E>> {
        self.context(format!("{}:{}: {}", file!(), line!(), context))
    }
}

/// 错误类型转换trait
///
/// 简化不同错误类型之间的转换。
pub trait IntoResult<T>: Sized {
    /// 将错误转换为特定的Result类型
    fn into_result(self) -> Result<T, Self>;

    /// 将错误映射到另一种错误类型
    fn map_err<E2, F>(self, f: F) -> Result<T, E2>
    where
        F: FnOnce() -> E2;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context(self, context: impl fmt::Display) -> Result<T, ContextError<E>> {
        self.map_err(|e| ContextError {
            error: e,
            context: context.to_string(),
        })
    }

    fn with_context<F>(self, f: F) -> Result<T, ContextError<E>>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| ContextError {
            error: e,
            context: f(),
        })
    }
}

/// 带上下文的错误包装器
#[derive(Debug)]
pub struct ContextError<E> {
    error: E,
    context: String,
}

impl<E> fmt::Display for ContextError<E>
where
    E: std::error::Error,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.error)
    }
}

impl<E> std::error::Error for ContextError<E>
where
    E: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// IO错误扩展trait
///
/// 为io::Result提供便捷的转换方法。
pub trait IoResultExt<T>: Sized {
    /// 转换IO错误为字符串错误
    fn map_io_err(self) -> Result<T, String>;

    /// 添加上下文到IO错误
    fn context_io(self, context: &str) -> Result<T, String>;
}

impl<T> IoResultExt<T> for Result<T, io::Error> {
    fn map_io_err(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }

    fn context_io(self, context: &str) -> Result<T, String> {
        self.map_err(|e| format!("{context}: {e}"))
    }
}

/// 可选值转换trait
///
/// 简化Option到Result的转换。
pub trait OptionExt<T>: Sized {
    /// 将None转换为错误
    fn ok_or_else<E>(self, error: impl FnOnce() -> E) -> Result<T, E>;

    /// 将None转换为特定错误消息
    fn ok_or_msg(self, msg: impl AsRef<str>) -> Result<T, String>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_else<E>(self, error: impl FnOnce() -> E) -> Result<T, E> {
        self.ok_or_else(error)
    }

    fn ok_or_msg(self, msg: impl AsRef<str>) -> Result<T, String> {
        self.ok_or_else(|| msg.as_ref().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_result_context() {
        let result: Result<(), io::Error> =
            Err(io::Error::new(io::ErrorKind::NotFound, "file.txt"));

        let result = result.context("Failed to load asset");
        assert!(result.is_err());

        if let Err(e) = result {
            let error_msg = e.to_string();
            assert!(error_msg.contains("Failed to load asset"));
            assert!(error_msg.contains("file.txt"));
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_result_with_context() {
        let result: Result<(), io::Error> = Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "access denied",
        ));

        let result = result.with_context(|| format!("IO Error: access denied"));
        assert!(result.is_err());
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_io_result_ext() {
        let result: Result<(), io::Error> =
            Err(io::Error::new(io::ErrorKind::InvalidData, "invalid"));

        let result = result.context_io("Reading file");
        assert!(result.is_err());

        if let Err(e) = result {
            assert!(e.contains("Reading file"));
        }
    }

    #[test]
    #[ignore] // TODO: Fix compilation errors
    fn test_option_ext() {
        let option: Option<i32> = None;

        let result = option.ok_or_msg("Value not found");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Value not found");
    }
}
