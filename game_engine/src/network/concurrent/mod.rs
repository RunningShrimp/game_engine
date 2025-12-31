//! 网络并发容器抽象层
//!
//! 本模块提供trait抽象来替代条件编译，减少代码重复。

pub mod client_registry;

// 重新导出主要类型（trait始终可用）
pub use client_registry::{ClientRegistry, DefaultClientRegistry};

// DashMap和Mutex实现是feature-gated
#[cfg(feature = "dashmap")]
pub use client_registry::DashMapClientRegistry;

#[cfg(not(feature = "dashmap"))]
pub use client_registry::MutexClientRegistry;
