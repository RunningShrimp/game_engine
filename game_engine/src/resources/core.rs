//! 资源管理核心类型
//!
//! 此模块定义了资源管理系统的核心类型，被多个资源管理器共享使用：
//! - `manager.rs`: 异步资源管理器
//! - `optimized_manager.rs`: 优化版资源管理器（使用DashMap/parking_lot）
//!
//! # 设计目标
//!
//! 1. **代码复用**: 消除manager.rs和optimized_manager.rs之间的重复代码
//! 2. **类型安全**: 提供统一的类型接口
//! 3. **性能优化**: 支持不同的并发策略（std::sync::RwLock vs parking_lot::RwLock vs DashMap）
//!
//! # 使用示例
//!
//! ```rust
//! use game_engine::resources::core::{LoadState, AssetContainer, Handle};
//! use std::sync::Arc;
//!
//! // 创建加载句柄
//! let handle = Handle::<String>::new_loading();
//!
//! // 检查加载状态
//! assert!(!handle.is_loaded());
//! ```

use bevy_ecs::prelude::Component;
use std::sync::{Arc, RwLock};

// =============================================================================
// 条件编译：选择锁类型
// =============================================================================

#[cfg(feature = "dashmap")]
pub use parking_lot::RwLock as LockType;

#[cfg(not(feature = "dashmap"))]
pub use std::sync::RwLock as LockType;

// =============================================================================
// 核心类型定义
// =============================================================================

/// 资源加载状态
///
/// 表示资源的当前加载状态，用于异步/同步资源加载。
///
/// # 变体
///
/// - `Loading`: 资源正在加载中
/// - `Loaded(T)`: 资源加载完成，包含加载的数据
/// - `Failed(String)`: 资源加载失败，包含错误信息
///
/// # 示例
///
/// ```
/// use game_engine::resources::core::LoadState;
///
/// let loading = LoadState::<String>::Loading;
/// let loaded = LoadState::Loaded("data".to_string());
/// let failed = LoadState::Failed("error".to_string());
/// ```
#[derive(Clone, Debug)]
pub enum LoadState<T> {
    /// 正在加载
    Loading,
    /// 加载完成
    Loaded(T),
    /// 加载失败
    Failed(String),
}

impl<T> LoadState<T> {
    /// 检查是否已加载
    ///
    /// # 返回
    ///
    /// 如果状态是 `Loaded`，返回 `true`；否则返回 `false`
    #[inline]
    pub fn is_loaded(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }

    /// 获取已加载的数据
    ///
    /// # 返回
    ///
    /// 如果状态是 `Loaded`，返回 `Some(&T)`；否则返回 `None`
    #[inline]
    pub fn get_loaded(&self) -> Option<&T> {
        match self {
            Self::Loaded(v) => Some(v),
            _ => None,
        }
    }

    /// 转换加载状态的数据类型
    ///
    /// # 泛型
    ///
    /// - `U`: 目标类型
    /// - `F`: 转换函数
    pub fn map<U, F>(self, f: F) -> LoadState<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Loading => LoadState::Loading,
            Self::Loaded(v) => LoadState::Loaded(f(v)),
            Self::Failed(e) => LoadState::Failed(e),
        }
    }
}

/// 资源容器
///
/// 内部使用锁保护加载状态，支持并发访问。
///
/// # 性能
///
/// - 使用 `parking_lot::RwLock`（当 `dashmap` feature 启用时）
/// - 使用 `std::sync::RwLock`（默认）
///
/// # 示例
///
/// ```rust
/// use game_engine::resources::core::{AssetContainer, LoadState};
/// use std::sync::Arc;
///
/// let container = Arc::new(AssetContainer::new());
/// ```
#[derive(Debug)]
pub struct AssetContainer<T> {
    /// 加载状态（受锁保护）
    pub state: LockType<LoadState<T>>,
}

impl<T> AssetContainer<T> {
    /// 创建新的资源容器（初始状态为Loading）
    #[inline]
    pub fn new() -> Self {
        Self {
            state: LockType::new(LoadState::Loading),
        }
    }

    /// 创建新的资源容器（带初始状态）
    #[inline]
    pub fn with_state(state: LoadState<T>) -> Self {
        Self {
            state: LockType::new(state),
        }
    }

    /// 创建新的资源容器（已加载状态）
    #[inline]
    pub fn with_loaded(data: T) -> Self {
        Self {
            state: LockType::new(LoadState::Loaded(data)),
        }
    }
}

impl<T: Clone> AssetContainer<T> {
    /// 尝试获取资源的克隆（非阻塞）
    ///
    /// # 返回
    ///
    /// 如果资源已加载，返回 `Some(T)`；否则返回 `None`
    #[inline]
    pub fn try_get_clone(&self) -> Option<T> {
        self.state.read().get_loaded().cloned()
    }
}

/// 资源句柄
///
/// 用于异步/同步资源加载的句柄，通过 `Arc` 共享资源容器。
///
/// # 线程安全
///
/// `Handle<T>` 可以在线程间安全地克隆和传递。
///
/// # 泛型
///
/// - `T`: 资源类型（必须满足 `'static + Send + Sync`）
///
/// # 示例
///
/// ```rust
/// use game_engine::resources::core::Handle;
///
/// let handle = Handle::<String>::new_loading();
/// assert!(!handle.is_loaded());
/// ```
#[derive(Clone, Component, Debug)]
pub struct THandle<T: 'static + Send + Sync> {
    /// 资源容器（Arc共享）
    pub container: Arc<AssetContainer<T>>,
}

// 为了向后兼容，提供 Handle 别名
pub use THandle as Handle;

impl<T: 'static + Send + Sync> Handle<T> {
    /// 创建新的加载中句柄
    ///
    /// # 返回
    ///
    /// 返回一个处于 `Loading` 状态的句柄
    #[inline]
    pub fn new_loading() -> Self {
        Self {
            container: Arc::new(AssetContainer::new()),
        }
    }

    /// 从 Arc<AssetContainer> 创建句柄（内部使用）
    #[inline]
    pub(crate) fn from_container(container: Arc<AssetContainer<T>>) -> Self {
        Self { container }
    }

    /// 获取资源（克隆）
    ///
    /// # 返回
    ///
    /// 如果资源已加载，返回 `Some(T)`；否则返回 `None`
    ///
    /// # 性能
    ///
    /// - 需要获取读锁
    /// - 如果 `T` 是 `Arc<U>`，克隆 Arc 只是增加引用计数，非常快
    pub fn get(&self) -> Option<T>
    where
        T: Clone,
    {
        self.container.state.read().get_loaded().cloned()
    }

    /// 检查资源是否已加载
    ///
    /// # 返回
    ///
    /// 如果资源已加载，返回 `true`；否则返回 `false`
    #[inline]
    pub fn is_loaded(&self) -> bool {
        self.container.state.read().is_loaded()
    }

    /// 非阻塞方式获取资源
    ///
    /// 与 `get()` 类似，但使用 `try_read()` 避免阻塞
    pub fn get_non_blocking(&self) -> Option<T>
    where
        T: Clone,
    {
        // parking_lot::RwLock::try_read() 返回 Option，std::sync::RwLock::try_read() 返回 Result
        #[cfg(feature = "dashmap")]
        {
            self.container.state.try_read().and_then(|state| state.get_loaded().cloned())
        }
        #[cfg(not(feature = "dashmap"))]
        {
            self.container
                .state
                .try_read()
                .ok()
                .and_then(|state| state.get_loaded().cloned())
        }
    }

    /// 获取加载状态（非阻塞）
    ///
    /// # 返回
    ///
    /// 返回当前状态的克隆（如果可能获取锁）
    pub fn get_state_non_blocking(&self) -> Option<LoadState<T>>
    where
        T: Clone,
    {
        #[cfg(feature = "dashmap")]
        {
            self.container.state.try_read().map(|state| match &*state {
                LoadState::Loaded(v) => LoadState::Loaded(v.clone()),
                LoadState::Failed(e) => LoadState::Failed(e.clone()),
                LoadState::Loading => LoadState::Loading,
            })
        }
        #[cfg(not(feature = "dashmap"))]
        {
            self.container.state.try_read().ok().map(|state| match &*state {
                LoadState::Loaded(v) => LoadState::Loaded(v.clone()),
                LoadState::Failed(e) => LoadState::Failed(e.clone()),
                LoadState::Loading => LoadState::Loading,
            })
        }
    }

    /// 获取资源状态信息
    ///
    /// # 返回
    ///
    /// 返回状态的字符串描述
    pub fn get_status(&self) -> Result<String, &'static str> {
        #[cfg(feature = "dashmap")]
        {
            match self.container.state.try_read() {
                Some(state) => Ok(match &*state {
                    LoadState::Loading => "loading".to_string(),
                    LoadState::Loaded(_) => "loaded".to_string(),
                    LoadState::Failed(err) => format!("failed: {err}"),
                }),
                None => Err("Lock unavailable"),
            }
        }
        #[cfg(not(feature = "dashmap"))]
        {
            self.container
                .state
                .try_read()
                .map(|state| match &*state {
                    LoadState::Loading => "loading".to_string(),
                    LoadState::Loaded(_) => "loaded".to_string(),
                    LoadState::Failed(err) => format!("failed: {err}"),
                })
                .map_err(|_| "Lock poisoned")
        }
    }
}

// =============================================================================
// 辅助trait和函数
// =============================================================================

/// 资源状态检查trait
///
/// 为资源容器和句柄提供统一的状态检查接口
pub trait ResourceState<T> {
    /// 检查是否已加载
    fn is_loaded(&self) -> bool;

    /// 获取已加载的数据
    fn get_loaded(&self) -> Option<T>
    where
        T: Clone;
}

impl<T: Clone + Send + Sync> ResourceState<T> for Handle<T> {
    #[inline]
    fn is_loaded(&self) -> bool {
        self.is_loaded()
    }

    #[inline]
    fn get_loaded(&self) -> Option<T> {
        self.get()
    }
}

impl<T: Clone + Send + Sync> ResourceState<T> for Arc<AssetContainer<T>> {
    #[inline]
    fn is_loaded(&self) -> bool {
        self.state.read().is_loaded()
    }

    #[inline]
    fn get_loaded(&self) -> Option<T> {
        self.state.read().get_loaded().cloned()
    }
}

/// 创建已加载的句柄（便捷函数）
///
/// # 参数
///
/// - `data`: 已加载的数据
///
/// # 返回
///
/// 返回一个已加载的句柄
pub fn loaded_handle<T: 'static + Send + Sync>(data: T) -> Handle<T> {
    Handle {
        container: Arc::new(AssetContainer::with_loaded(data)),
    }
}

/// 创建失败的句柄（便捷函数）
///
/// # 参数
///
/// - `error`: 错误信息
///
/// # 返回
///
/// 返回一个加载失败的句柄
pub fn failed_handle<T: 'static + Send + Sync>(error: String) -> Handle<T> {
    Handle {
        container: Arc::new(AssetContainer::with_state(LoadState::Failed(error))),
    }
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_state() {
        let loading = LoadState::<String>::Loading;
        assert!(!loading.is_loaded());
        assert!(loading.get_loaded().is_none());

        let loaded = LoadState::Loaded("data".to_string());
        assert!(loaded.is_loaded());
        assert_eq!(loaded.get_loaded(), Some(&"data".to_string()));

        let failed = LoadState::<String>::Failed("error".to_string());
        assert!(!failed.is_loaded());
        assert!(failed.get_loaded().is_none());
    }

    #[test]
    fn test_load_state_map() {
        let loaded = LoadState::Loaded(42);
        let mapped = loaded.map(|x| x.to_string());

        assert!(mapped.is_loaded());
        assert_eq!(mapped.get_loaded(), Some(&"42".to_string()));
    }

    #[test]
    fn test_asset_container() {
        let container = AssetContainer::<String>::new();
        assert!(!container.state.read().is_loaded());

        let container_with_data = AssetContainer::with_loaded("test".to_string());
        assert!(container_with_data.state.read().is_loaded());
    }

    #[test]
    fn test_handle_creation() {
        let handle = Handle::<String>::new_loading();
        assert!(!handle.is_loaded());
        assert!(handle.get().is_none());
    }

    #[test]
    fn test_handle_loaded() {
        let container = Arc::new(AssetContainer::with_loaded("test".to_string()));
        let handle = Handle::from_container(container);

        assert!(handle.is_loaded());
        assert_eq!(handle.get(), Some("test".to_string()));
    }

    #[test]
    fn test_handle_get_status() {
        let handle = Handle::<String>::new_loading();
        assert_eq!(handle.get_status().unwrap(), "loading");

        let loaded = loaded_handle("data".to_string());
        assert_eq!(loaded.get_status().unwrap(), "loaded");

        let failed = failed_handle::<String>("error".to_string());
        assert!(failed.get_status().unwrap().starts_with("failed:"));
    }

    #[test]
    fn test_convenience_functions() {
        let loaded = loaded_handle(42i32);
        assert!(loaded.is_loaded());
        assert_eq!(loaded.get(), Some(42));

        let failed = failed_handle::<i32>("test error".to_string());
        assert!(!failed.is_loaded());
        assert!(failed.get().is_none());
    }

    #[test]
    fn test_resource_state_trait() {
        let handle = loaded_handle("data".to_string());
        assert!(ResourceState::is_loaded(&handle));
        assert_eq!(ResourceState::get_loaded(&handle), Some("data".to_string()));

        let container = Arc::new(AssetContainer::with_loaded("test".to_string()));
        assert!(ResourceState::is_loaded(&container));
        assert_eq!(
            ResourceState::get_loaded(&container),
            Some("test".to_string())
        );
    }

    #[test]
    fn test_handle_clone() {
        let loaded = loaded_handle(42i32);
        let cloned = loaded.clone();

        assert!(cloned.is_loaded());
        assert_eq!(cloned.get(), Some(42));
    }

    #[test]
    fn test_handle_non_blocking() {
        let handle = Handle::<String>::new_loading();
        assert!(handle.get_non_blocking().is_none());

        let container = Arc::new(AssetContainer::with_loaded("test".to_string()));
        let handle = Handle::from_container(container);
        assert_eq!(handle.get_non_blocking(), Some("test".to_string()));
    }

    #[test]
    fn test_handle_state_non_blocking() {
        let handle = loaded_handle(42i32);
        let state = handle.get_state_non_blocking();

        assert!(state.is_some());
        assert!(state.unwrap().is_loaded());
    }
}
