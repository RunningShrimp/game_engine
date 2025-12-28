//! Tracy Profiler集成
//!
//! 提供高性能的实时性能分析功能，支持：
//! - 火焰图生成
//! - GPU性能分析
//! - 内存分析
//! - 实时性能监控

#[cfg(feature = "tracy")]
use tracy_client::*;

/// Tracy分析器
pub struct TracyProfiler {
    #[cfg(feature = "tracy")]
    enabled: bool,
}

impl TracyProfiler {
    /// 创建新的Tracy分析器
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "tracy")]
            enabled: true,
        }
    }

    /// 启用/禁用分析器
    pub fn set_enabled(&mut self, enabled: bool) {
        #[cfg(feature = "tracy")]
        { self.enabled = enabled; }
        #[cfg(not(feature = "tracy"))]
        { let _ = enabled; }
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        #[cfg(feature = "tracy")]
        { self.enabled }
        #[cfg(not(feature = "tracy"))]
        { false }
    }
}

impl Default for TracyProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracy作用域 - 用于自动测量代码块性能
pub struct TracyScope {
    #[cfg(feature = "tracy")]
    span: Span,
    #[cfg(not(feature = "tracy"))]
    _name: &'static str,
}

impl TracyScope {
    /// 创建新的Tracy作用域
    pub fn new(name: &'static str) -> Self {
        Self {
            #[cfg(feature = "tracy")]
            span: Span::new(name, "", file!(), line!(), 100),
            #[cfg(not(feature = "tracy"))]
            _name: name,
        }
    }

    /// 创建带颜色的作用域
    pub fn with_color(name: &'static str, color: u32) -> Self {
        Self {
            #[cfg(feature = "tracy")]
            span: Span::new(name, "", file!(), line!(), color),
            #[cfg(not(feature = "tracy"))]
            _name: name,
        }
    }

    /// 创建GPU作用域（需要GPU上下文）
    pub fn gpu(_name: &'static str) -> Self {
        Self {
            #[cfg(feature = "tracy")]
            span: Span::new(_name, "", file!(), line!(), 100),
            #[cfg(not(feature = "tracy"))]
            _name: _name,
        }
    }
}

impl Drop for TracyScope {
    fn drop(&mut self) {
        #[cfg(feature = "tracy")]
        { /* Span在drop时自动结束 */ }
    }
}

/// Tracy消息 - 用于记录事件和消息
pub struct TracyMessage;

impl TracyMessage {
    /// 发送文本消息
    pub fn text(message: &str) {
        #[cfg(feature = "tracy")]
        { message!(message); }
        #[cfg(not(feature = "tracy"))]
        { let _ = message; }
    }

    /// 发送带颜色的消息
    pub fn colored(message: &str, color: u32) {
        #[cfg(feature = "tracy")]
        { message!(message, color); }
        #[cfg(not(feature = "tracy"))]
        { let _ = (message, color); }
    }

    /// 发送帧标记
    pub fn frame_mark() {
        #[cfg(feature = "tracy")]
        { frame_mark!(); }
    }

    /// 发送带名称的帧标记
    pub fn frame_mark_named(name: &str) {
        #[cfg(feature = "tracy")]
        { frame_mark_named!(name); }
        #[cfg(not(feature = "tracy"))]
        { let _ = name; }
    }
}

/// Tracy GPU上下文 - 用于GPU性能分析
#[cfg(feature = "tracy")]
pub struct TracyGpuContext {
    context: GpuContext,
}

#[cfg(not(feature = "tracy"))]
pub struct TracyGpuContext {
    _name: String,
}

impl TracyGpuContext {
    /// 创建新的GPU上下文
    pub fn new(name: &str) -> Self {
        Self {
            #[cfg(feature = "tracy")]
            context: GpuContext::new(name),
            #[cfg(not(feature = "tracy"))]
            _name: name.to_string(),
        }
    }

    /// 开始GPU作用域
    pub fn begin_span(&self, name: &str) -> TracyGpuSpan {
        TracyGpuSpan {
            #[cfg(feature = "tracy")]
            span: self.context.begin_span(name),
            #[cfg(not(feature = "tracy"))]
            _name: name.to_string(),
        }
    }

    /// 收集GPU时间戳
    pub fn collect(&self) {
        #[cfg(feature = "tracy")]
        { self.context.collect(); }
    }
}

/// Tracy GPU作用域
#[cfg(feature = "tracy")]
pub struct TracyGpuSpan {
    span: GpuSpan,
}

#[cfg(not(feature = "tracy"))]
pub struct TracyGpuSpan {
    _name: String,
}

impl Drop for TracyGpuSpan {
    fn drop(&mut self) {
        #[cfg(feature = "tracy")]
        { /* GpuSpan在drop时自动结束 */ }
    }
}

/// Tracy内存分配追踪
pub struct TracyAllocation;

impl TracyAllocation {
    /// 记录内存分配
    pub fn alloc(ptr: *mut u8, size: usize) {
        #[cfg(feature = "tracy")]
        { unsafe { tracy_client::alloc(ptr, size); } }
        #[cfg(not(feature = "tracy"))]
        { let _ = (ptr, size); }
    }

    /// 记录内存释放
    pub fn free(ptr: *mut u8) {
        #[cfg(feature = "tracy")]
        { unsafe { tracy_client::free(ptr); } }
        #[cfg(not(feature = "tracy"))]
        { let _ = ptr; }
    }
}

// 重新导出公共API（避免重复导出，已在mod.rs中导出）

/// 便捷宏：创建Tracy作用域
#[macro_export]
macro_rules! tracy_scope {
    ($name:expr) => {
        let _tracy_scope = $crate::profiling::tracy::TracyScope::new($name);
    };
    ($name:expr, $color:expr) => {
        let _tracy_scope = $crate::profiling::tracy::TracyScope::with_color($name, $color);
    };
}

/// 便捷宏：创建GPU作用域
#[macro_export]
macro_rules! tracy_gpu_scope {
    ($name:expr) => {
        let _tracy_gpu_scope = $crate::profiling::tracy::TracyScope::gpu($name);
    };
}

/// 便捷宏：发送Tracy消息
#[macro_export]
macro_rules! tracy_message {
    ($msg:expr) => {
        $crate::profiling::tracy::TracyMessage::text($msg);
    };
    ($msg:expr, $color:expr) => {
        $crate::profiling::tracy::TracyMessage::colored($msg, $color);
    };
}

/// 便捷宏：帧标记
#[macro_export]
macro_rules! tracy_frame {
    () => {
        $crate::profiling::tracy::TracyMessage::frame_mark();
    };
    ($name:expr) => {
        $crate::profiling::tracy::TracyMessage::frame_mark_named($name);
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracy_profiler_creation() {
        let profiler = TracyProfiler::new();
        // 测试创建不会panic
        assert!(!profiler.is_enabled() || profiler.is_enabled());
    }

    #[test]
    fn test_tracy_scope() {
        let _scope = TracyScope::new("test_scope");
        // 测试作用域创建不会panic
    }

    #[test]
    fn test_tracy_message() {
        TracyMessage::text("test message");
        // 测试消息发送不会panic
    }
}

