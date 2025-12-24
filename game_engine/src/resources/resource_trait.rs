//! 统一资源接口
//!
//! 定义统一的资源trait，用于所有类型的资源（纹理、模型、音频等）。
//! 提供统一的资源生命周期管理和元数据访问接口。

use std::path::PathBuf;
use std::time::SystemTime;

/// 统一资源错误类型
#[derive(Debug, thiserror::Error)]
pub enum ResourceError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Resource not found: {0}")]
    NotFound(String),

    #[error("Decode error: {0}")]
    Decode(String),

    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    #[error("Dependency error: {0}")]
    Dependency(String),

    #[error("Other error: {0}")]
    Other(String),
}

/// 资源元数据
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    /// 资源路径
    pub path: PathBuf,
    /// 资源大小（字节）
    pub size_bytes: usize,
    /// 最后修改时间
    pub last_modified: Option<SystemTime>,
    /// 资源类型
    pub resource_type: String,
    /// 自定义元数据
    pub custom: std::collections::HashMap<String, String>,
}

impl ResourceMetadata {
    /// 创建新的资源元数据
    pub fn new(path: PathBuf, size_bytes: usize, resource_type: impl Into<String>) -> Self {
        Self {
            path,
            size_bytes,
            last_modified: None,
            resource_type: resource_type.into(),
            custom: std::collections::HashMap::new(),
        }
    }
}

/// 统一资源 trait
///
/// 所有资源类型必须实现此trait，提供统一的资源管理接口。
pub trait Resource: Send + Sync + 'static {
    /// 获取资源元数据
    fn metadata(&self) -> &ResourceMetadata;

    /// 获取资源大小（字节）
    fn size_bytes(&self) -> usize {
        self.metadata().size_bytes
    }

    /// 检查资源是否已加载
    fn is_loaded(&self) -> bool {
        true // 默认实现：如果资源对象存在，则认为已加载
    }

    /// 获取资源路径
    fn path(&self) -> &PathBuf {
        &self.metadata().path
    }

    /// 获取资源类型
    fn resource_type(&self) -> &str {
        &self.metadata().resource_type
    }

    /// 类型擦除支持（用于统一资源管理器）
    fn as_any(&self) -> &dyn std::any::Any
    where
        Self: 'static,
    {
        self
    }
}

/// 统一资源加载器 trait
///
/// 定义资源加载的标准接口，支持异步加载和批量预加载。
pub trait ResourceLoader: Send + Sync {
    /// 此加载器处理的资源类型
    type Resource: Resource;

    /// 加载上下文（如渲染器、设备等）
    type Context: Send + Sync;

    /// 异步加载单个资源
    ///
    /// # 参数
    /// - `path`: 资源路径
    /// - `ctx`: 加载上下文
    ///
    /// # 返回
    /// 加载的资源或错误
    async fn load(
        &self,
        path: &std::path::Path,
        ctx: &Self::Context,
    ) -> Result<Self::Resource, ResourceError>;

    /// 批量预加载资源
    ///
    /// # 参数
    /// - `paths`: 资源路径列表
    /// - `ctx`: 加载上下文
    ///
    /// # 返回
    /// 加载结果列表，顺序与输入相同
    async fn preload(
        &self,
        paths: &[PathBuf],
        ctx: &Self::Context,
    ) -> Vec<Result<Self::Resource, ResourceError>> {
        let mut results = Vec::with_capacity(paths.len());
        for path in paths {
            results.push(self.load(path, ctx).await);
        }
        results
    }

    /// 检查资源是否存在
    ///
    /// # 参数
    /// - `path`: 资源路径
    ///
    /// # 返回
    /// 如果资源存在则返回true
    async fn exists(&self, path: &std::path::Path) -> bool {
        std::path::Path::new(path).exists()
    }

    /// 获取资源元数据（不加载完整资源）
    ///
    /// # 参数
    /// - `path`: 资源路径
    ///
    /// # 返回
    /// 资源元数据或错误
    async fn metadata(&self, path: &std::path::Path) -> Result<ResourceMetadata, ResourceError> {
        let path_buf = path.to_path_buf();
        let metadata = std::fs::metadata(path)?;
        let size_bytes = metadata.len() as usize;
        let last_modified = metadata.modified().ok();

        Ok(ResourceMetadata {
            path: path_buf,
            size_bytes,
            last_modified,
            resource_type: "unknown".to_string(),
            custom: std::collections::HashMap::new(),
        })
    }
}

/// 资源加载器注册表
///
/// 管理不同类型的资源加载器，支持动态注册和查找。
pub struct ResourceLoaderRegistry {
    loaders: std::collections::HashMap<String, Box<dyn AnyResourceLoader>>,
}

impl ResourceLoaderRegistry {
    /// 创建新的加载器注册表
    pub fn new() -> Self {
        Self {
            loaders: std::collections::HashMap::new(),
        }
    }

    /// 注册资源加载器
    ///
    /// # 参数
    /// - `resource_type`: 资源类型标识符（如"texture"、"model"等）
    /// - `loader`: 资源加载器
    pub fn register<L: ResourceLoader + 'static>(
        &mut self,
        resource_type: impl Into<String>,
        loader: L,
    ) {
        self.loaders.insert(resource_type.into(), Box::new(LoaderWrapper(loader)));
    }

    /// 获取资源加载器
    ///
    /// # 参数
    /// - `resource_type`: 资源类型标识符
    ///
    /// # 返回
    /// 资源加载器的可选引用
    pub fn get(&self, resource_type: &str) -> Option<&dyn AnyResourceLoader> {
        self.loaders.get(resource_type).map(|l| l.as_ref())
    }
}

impl Default for ResourceLoaderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// 类型擦除的资源加载器trait对象
pub trait AnyResourceLoader: Send + Sync {
    /// 加载资源（类型擦除版本）
    fn load_any(
        &self,
        path: &std::path::Path,
        ctx: &dyn std::any::Any,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Box<dyn std::any::Any>, ResourceError>> + Send>,
    >;
}

/// 资源加载器包装器（用于类型擦除）
struct LoaderWrapper<L: ResourceLoader>(L);

impl<L: ResourceLoader + 'static> AnyResourceLoader for LoaderWrapper<L> {
    fn load_any(
        &self,
        _path: &std::path::Path,
        _ctx: &dyn std::any::Any,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Box<dyn std::any::Any>, ResourceError>> + Send>,
    > {
        // 注意：这是一个简化的实现，实际使用时需要更复杂的类型转换
        Box::pin(async move {
            Err(ResourceError::Other(
                "Type erasure not fully implemented".to_string(),
            ))
        })
    }
}
