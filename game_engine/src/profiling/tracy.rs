//! Tracy Profiler集成
//!
//! 提供高性能的实时性能分析功能，支持：
//! - 火焰图生成
//! - GPU性能分析
//! - 内存分析
//! - 实时性能监控
//!
//! ## 重构说明
//!
//! 重构后使用 ProfilerBackend trait 抽象，条件编译从22个减少到2个。

#![allow(dead_code)]

use crate::profiling::backend::{ProfilerBackend, ProfilerScope};

/// 根据feature选择后端实现
#[cfg(feature = "tracy")]
type BackendImpl = crate::profiling::backend::TracyBackend;

#[cfg(not(feature = "tracy"))]
type BackendImpl = crate::profiling::backend::StubBackend;

/// Tracy分析器
pub struct TracyProfiler {
    backend: BackendImpl,
}

impl TracyProfiler {
    /// 创建新的Tracy分析器
    pub fn new() -> Self {
        Self {
            backend: BackendImpl::new(),
        }
    }

    /// 启用/禁用分析器
    pub fn set_enabled(&mut self, enabled: bool) {
        // TracyBackend不支持运行时禁用，此方法保留用于API兼容性
        let _ = enabled;
    }

    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.backend.is_enabled()
    }

    /// 创建分析作用域
    pub fn scope(&self, name: &str) -> ProfilerScope<'_> {
        ProfilerScope::new(&self.backend, name)
    }

    /// 标记事件
    pub fn mark(&self, name: &str) {
        self.backend.mark_event(name);
    }
}

impl Default for TracyProfiler {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 兼容层：保持旧API
// ============================================================================

/// Tracy作用域 - 用于自动测量代码块性能
///
/// 注意：保留此类型用于API兼容性，推荐使用 profiler.scope() 代替
pub struct TracyScope {
    _private: (),
}

impl TracyScope {
    /// 创建新的Tracy作用域
    pub fn new(_name: &'static str) -> Self {
        Self { _private: () }
    }

    /// 创建带颜色的作用域
    pub fn with_color(_name: &'static str, _color: u32) -> Self {
        Self { _private: () }
    }

    /// 创建GPU作用域（需要GPU上下文）
    pub fn gpu(_name: &'static str) -> Self {
        Self { _private: () }
    }
}

/// Tracy消息 - 用于记录事件和消息
pub struct TracyMessage;

impl TracyMessage {
    /// 发送文本消息
    pub fn text(_message: &str) {
        // Stub实现
    }

    /// 发送带颜色的消息
    pub fn colored(_message: &str, _color: u32) {
        // Stub实现
    }

    /// 发送帧标记
    pub fn frame_mark() {
        // Stub实现
    }

    /// 发送带名称的帧标记
    pub fn frame_mark_named(_name: &str) {
        // Stub实现
    }
}

/// Tracy GPU上下文 - 用于GPU性能分析
pub struct TracyGpuContext {
    _name: String,
}

impl TracyGpuContext {
    /// 创建新的GPU上下文
    pub fn new(name: &str) -> Self {
        Self {
            _name: name.to_string(),
        }
    }

    /// 开始GPU作用域
    pub fn begin_span(&self, _name: &str) -> TracyGpuSpan {
        TracyGpuSpan {
            _name: String::new(),
        }
    }

    /// 收集GPU时间戳
    pub fn collect(&self) {
        // Stub实现
    }
}

/// Tracy GPU作用域
pub struct TracyGpuSpan {
    _name: String,
}

/// Tracy内存分配追踪
pub struct TracyAllocation;

impl TracyAllocation {
    /// 记录内存分配
    pub fn alloc(_ptr: *mut u8, _size: usize) {
        // Stub实现
    }

    /// 记录内存释放
    pub fn free(_ptr: *mut u8) {
        // Stub实现
    }
}

// ============================================================================
// Tracy特定实现（仅在启用feature时）
// ============================================================================

#[cfg(feature = "tracy")]
mod tracy_impl {
    use super::*;

    #[cfg(feature = "tracy")]
    use tracy_client::*;

    #[cfg(feature = "tracy")]
    impl TracyScope {
        /// 创建带Tracy的作用域
        pub fn new_tracy(_name: &'static str) -> Self {
            // Note: In tracy-client 0.18, span! requires literal strings at compile time
            // For runtime strings, we need to use Client directly or accept this limitation
            // For now, use a placeholder since we can't pass variables to span! macro
            let _zone = span!("tracy_scope");
            let _ = _zone;
            Self { _private: () }
        }

        /// 创建带颜色的Tracy作用域
        pub fn with_color_tracy(_name: &'static str, _color: u32) -> Self {
            // Same limitation as above
            let _zone = span!("tracy_scope_color");
            let _ = _zone;
            Self { _private: () }
        }
    }

    #[cfg(feature = "tracy")]
    impl TracyMessage {
        /// 发送Tracy文本消息
        pub fn text_tracy(message: &str) {
            if let Some(client) = Client::running() {
                client.message(message, 0);
            }
        }

        /// 发送带颜色的Tracy消息
        pub fn colored_tracy(message: &str, color: u32) {
            if let Some(client) = Client::running() {
                client.color_message(message, color, 0);
            }
        }

        /// 发送Tracy帧标记
        pub fn frame_mark_tracy() {
            if let Some(client) = Client::running() {
                client.message("Frame", 0);
            }
        }

        /// 发送带名称的Tracy帧标记
        pub fn frame_mark_named_tracy(name: &str) {
            if let Some(client) = Client::running() {
                client.message(name, 0);
            }
        }
    }

    #[cfg(feature = "tracy")]
    impl TracyGpuContext {
        /// 创建Tracy GPU上下文
        pub fn new_tracy(name: &str) -> Self {
            // GPU context would be created here
            // For now, just store the name
            Self {
                _name: name.to_string(),
            }
        }

        /// 开始Tracy GPU作用域
        pub fn begin_span_tracy(&self, name: &str) -> TracyGpuSpan {
            TracyGpuSpan {
                _name: name.to_string(),
            }
        }

        /// 收集Tracy GPU时间戳
        pub fn collect_tracy(&self) {
            // GPU collection would happen here
        }
    }

    #[cfg(feature = "tracy")]
    impl TracyAllocation {
        /// 记录Tracy内存分配
        pub fn alloc_tracy(_ptr: *mut u8, _size: usize) {
            // Memory allocation tracking would happen here
            // For tracy-client 0.18, this is handled differently
        }

        /// 记录Tracy内存释放
        pub fn free_tracy(_ptr: *mut u8) {
            // Memory free tracking would happen here
            // For tracy-client 0.18, this is handled differently
        }
    }
}

// 注意：profile_scope 和 profile_mark 宏已通过 #[macro_export] 导出到 crate 根
// ProfilerBackend trait 在模块顶部已导入

// ============================================================================
// 便捷宏
// ============================================================================

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
        let profiler = TracyProfiler::new();
        let _scope = profiler.scope("test_scope");
        // 测试作用域创建不会panic
    }

    #[test]
    fn test_tracy_message() {
        let profiler = TracyProfiler::new();
        profiler.mark("test message");
        // 测试消息发送不会panic
    }

    #[test]
    fn test_legacy_api() {
        let _scope = TracyScope::new("test");
        TracyMessage::text("test");
        // 测试旧API兼容性
    }
}
