//! 性能分析后端Trait定义
//!
//! 提供统一的性能分析接口，支持多种后端实现（Tracy、Stub等）

/// 性能分析后端trait
pub trait ProfilerBackend {
    /// 开始性能分析区域
    fn begin_span(&self, name: &str);

    /// 结束性能分析区域
    fn end_span(&self);

    /// 标记一个即时事件
    fn mark_event(&self, name: &str);

    /// 检查是否启用
    fn is_enabled(&self) -> bool;
}

/// 作用域guard - 自动管理分析区域
pub struct ProfilerScope<'a> {
    backend: &'a dyn ProfilerBackend,
    name: String,
}

impl<'a> ProfilerScope<'a> {
    pub fn new(backend: &'a dyn ProfilerBackend, name: &str) -> Self {
        backend.begin_span(name);
        Self {
            backend,
            name: name.to_string(),
        }
    }
}

impl<'a> Drop for ProfilerScope<'a> {
    fn drop(&mut self) {
        self.backend.end_span();
    }
}

// ============================================================================
// Tracy实现
// ============================================================================

#[cfg(feature = "tracy")]
pub use tracy_impl::TracyBackend;

#[cfg(feature = "tracy")]
mod tracy_impl {
    use super::ProfilerBackend;
    use tracy_client::*;

    pub struct TracyBackend;

    impl TracyBackend {
        pub fn new() -> Self {
            // Initialize tracy client on first use
            let _ = Client::running();
            Self
        }
    }

    impl ProfilerBackend for TracyBackend {
        fn begin_span(&self, _name: &str) {
            // Create a zone that ends when dropped
            // Note: In tracy-client 0.18, span! requires literal strings
            // For runtime profiling with dynamic names, we use a placeholder
            // This is a limitation of the macro-based API
            let _zone = span!("profiling_zone");
            let _ = _zone; // Keep until drop
        }

        fn end_span(&self) {
            // Zone automatically ends when dropped
        }

        fn mark_event(&self, name: &str) {
            // Use Client::message to mark events (works with dynamic strings)
            if let Some(client) = Client::running() {
                client.message(name, 0);
            }
        }

        fn is_enabled(&self) -> bool {
            Client::running().is_some()
        }
    }
}

// ============================================================================
// Stub实现（用于非Tracy构建）
// ============================================================================

#[cfg(not(feature = "tracy"))]
pub use stub_impl::StubBackend;

#[cfg(not(feature = "tracy"))]
mod stub_impl {
    use super::ProfilerBackend;

    /// Stub实现 - 零开销
    pub struct StubBackend;

    impl Default for StubBackend {
        fn default() -> Self {
            Self::new()
        }
    }

    impl StubBackend {
        pub fn new() -> Self {
            Self
        }
    }

    impl ProfilerBackend for StubBackend {
        fn begin_span(&self, _name: &str) {
            // 空实现 - 编译器会优化掉
        }

        fn end_span(&self) {
            // 空实现
        }

        fn mark_event(&self, _name: &str) {
            // 空实现
        }

        fn is_enabled(&self) -> bool {
            false
        }
    }
}

// ============================================================================
// 便捷宏
// ============================================================================

/// 创建性能分析作用域的便捷宏
#[macro_export]
macro_rules! profile_scope {
    ($profiler:expr, $name:expr) => {
        let _scope = $profiler.scope($name);
        // _scope会在作用域结束时自动drop，结束分析
    };
}

/// 标记性能事件的便捷宏
#[macro_export]
macro_rules! profile_mark {
    ($profiler:expr, $name:expr) => {
        $profiler.mark($name);
    };
}
