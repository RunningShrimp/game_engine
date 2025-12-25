//! 平台检测工具
//!
//! 集中管理所有平台相关的检测逻辑，提供统一的平台检测接口。
//! 这个模块减少了代码中散落的平台检测代码，使跨平台开发更加清晰和可维护。
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::platform::detection;
//!
//! if detection::is_mobile() {
//!     // 移动平台特定代码
//! }
//!
//! if detection::is_web() {
//!     // Web平台特定代码
//! }
//!
//! if detection::supports_simd() {
//!     // SIMD优化代码
//! }
//! ```

/// 检测是否为移动平台
///
/// 返回 `true` 如果当前目标是 Android 或 iOS。
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_mobile() {
///     println!("Running on mobile platform");
/// }
/// ```
#[inline]
pub fn is_mobile() -> bool {
    cfg!(any(target_os = "android", target_os = "ios"))
}

/// 检测是否为桌面平台
///
/// 返回 `true` 如果当前目标是 Windows、macOS 或 Linux。
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_desktop() {
///     println!("Running on desktop platform");
/// }
/// ```
#[inline]
pub fn is_desktop() -> bool {
    cfg!(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux"
    ))
}

/// 检测是否为控制台平台
///
/// 返回 `true` 如果当前目标是游戏主机平台。
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_console() {
///     println!("Running on console platform");
/// }
/// ```
#[inline]
pub fn is_console() -> bool {
    cfg!(any(
        target_os = "psp",
        target_os = "horizon",
        target_os = "psx"
    ))
}

/// 检测是否为Web平台
///
/// 返回 `true` 如果当前目标是 WebAssembly。
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_web() {
///     println!("Running on Web platform");
/// }
/// ```
#[inline]
pub fn is_web() -> bool {
    cfg!(target_arch = "wasm32")
}

/// 检测是否支持SIMD指令
///
/// 返回 `true` 如果当前架构支持SIMD指令（x86_64 或 aarch64）。
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::supports_simd() {
///     println!("SIMD instructions are available");
/// }
/// ```
#[inline]
pub fn supports_simd() -> bool {
    cfg!(any(target_arch = "x86_64", target_arch = "aarch64"))
}

/// 检测是否为Windows平台
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_windows() {
///     println!("Running on Windows");
/// }
/// ```
#[inline]
pub fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// 检测是否为macOS平台
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_macos() {
///     println!("Running on macOS");
/// }
/// ```
#[inline]
pub fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// 检测是否为Linux平台
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_linux() {
///     println!("Running on Linux");
/// }
/// ```
#[inline]
pub fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// 检测是否为Android平台
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_android() {
///     println!("Running on Android");
/// }
/// ```
#[inline]
pub fn is_android() -> bool {
    cfg!(target_os = "android")
}

/// 检测是否为iOS平台
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_ios() {
///     println!("Running on iOS");
/// }
/// ```
#[inline]
pub fn is_ios() -> bool {
    cfg!(target_os = "ios")
}

/// 检测是否为x86_64架构
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_x86_64() {
///     println!("Running on x86_64 architecture");
/// }
/// ```
#[inline]
pub fn is_x86_64() -> bool {
    cfg!(target_arch = "x86_64")
}

/// 检测是否为aarch64架构
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_aarch64() {
///     println!("Running on aarch64 architecture");
/// }
/// ```
#[inline]
pub fn is_aarch64() -> bool {
    cfg!(target_arch = "aarch64")
}

/// 检测是否为wasm32架构
///
/// # 示例
///
/// ```rust
/// use game_engine::platform::detection;
///
/// if detection::is_wasm32() {
///     println!("Running on wasm32 architecture");
/// }
/// ```
#[inline]
pub fn is_wasm32() -> bool {
    cfg!(target_arch = "wasm32")
}

/// 平台信息结构
///
/// 包含当前平台的详细信息。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlatformInfo {
    /// 是否为移动平台
    pub is_mobile: bool,
    /// 是否为桌面平台
    pub is_desktop: bool,
    /// 是否为控制台平台
    pub is_console: bool,
    /// 是否为Web平台
    pub is_web: bool,
    /// 是否支持SIMD
    pub supports_simd: bool,
    /// 操作系统名称
    pub os: &'static str,
    /// 架构名称
    pub arch: &'static str,
}

impl PlatformInfo {
    /// 获取当前平台信息
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::platform::detection;
    ///
    /// let info = detection::PlatformInfo::current();
    /// println!("OS: {}, Arch: {}", info.os, info.arch);
    /// ```
    pub fn current() -> Self {
        Self {
            is_mobile: is_mobile(),
            is_desktop: is_desktop(),
            is_console: is_console(),
            is_web: is_web(),
            supports_simd: supports_simd(),
            os: current_os(),
            arch: current_arch(),
        }
    }
}

/// 获取当前操作系统名称
///
/// 返回当前目标操作系统的名称字符串。
#[inline]
pub fn current_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "android") {
        "android"
    } else if cfg!(target_os = "ios") {
        "ios"
    } else if cfg!(target_os = "psp") {
        "psp"
    } else if cfg!(target_os = "horizon") {
        "horizon"
    } else if cfg!(target_os = "psx") {
        "psx"
    } else {
        "unknown"
    }
}

/// 获取当前架构名称
///
/// 返回当前目标架构的名称字符串。
#[inline]
pub fn current_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "wasm32") {
        "wasm32"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_platform_detection() {
        // 测试平台检测函数不会panic
        let _ = is_mobile();
        let _ = is_desktop();
        let _ = is_console();
        let _ = is_web();
        let _ = supports_simd();
    }

    #[test]
    fn test_platform_info() {
        let info = PlatformInfo::current();
        // 验证平台信息的一致性
        assert_eq!(info.is_mobile, is_mobile());
        assert_eq!(info.is_desktop, is_desktop());
        assert_eq!(info.is_console, is_console());
        assert_eq!(info.is_web, is_web());
        assert_eq!(info.supports_simd, supports_simd());
    }

    #[test]
    fn test_current_os() {
        let os = current_os();
        // 验证返回的是有效的操作系统名称
        assert!(!os.is_empty());
    }

    #[test]
    fn test_current_arch() {
        let arch = current_arch();
        // 验证返回的是有效的架构名称
        assert!(!arch.is_empty());
    }
}

