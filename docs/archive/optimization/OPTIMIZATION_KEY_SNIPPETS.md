# 条件编译优化 - 核心代码片段

## 文件: asset_loader_trait.rs

```rust
//! 统一资源加载器Trait
//! 核心理念: 使用trait对象替代条件编译

use async_trait::async_trait;
use std::{any::Any, path::Path};

// =============================================================================
// 类型定义
// =============================================================================

/// 资源加载错误
#[derive(Debug, thiserror::Error)]
pub enum AssetLoadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Decode error: {0}")]
    Decode(String),
    #[error("Unsupported format")]
    UnsupportedFormat,
}

/// 资源加载结果 - 类型擦除版本
pub enum BoxedAssetResult {
    Image(image::RgbaImage, bool),      // (image, is_linear)
    Bytes(Vec<u8>),
    Custom(Box<dyn Any + Send + Sync>), // ✅ 支持任意类型，包括GLTF
}

// =============================================================================
// AssetLoader Trait - 核心抽象
// =============================================================================

/// 资源加载器trait - 支持异步加载
#[async_trait]
pub trait AssetLoader: Send + Sync + 'static {
    /// 获取支持的文件扩展名（小写，不含点）
    fn extensions(&self) -> &[&str];

    /// 加载资源（异步）
    async fn load(
        &self,
        path: &Path,
        bytes: Vec<u8>,
    ) -> Result<BoxedAssetResult, AssetLoadError>;

    /// 获取加载器名称（用于调试）
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// 克隆为Box trait对象
    fn clone_box(&self) -> Box<dyn AssetLoader>;
}

// =============================================================================
// 具体加载器实现
// =============================================================================

/// 纹理加载器
#[derive(Clone)]
pub struct TextureAssetLoader;

#[async_trait]
impl AssetLoader for TextureAssetLoader {
    fn extensions(&self) -> &[&str] {
        &["png", "jpg", "jpeg", "bmp", "tga", "gif", "webp"]
    }

    async fn load(
        &self,
        _path: &Path,
        bytes: Vec<u8>,
    ) -> Result<BoxedAssetResult, AssetLoadError> {
        // 在阻塞任务中解码图像
        let image = tokio::task::spawn_blocking(move || {
            image::load_from_memory(&bytes)
                .map_err(|e| AssetLoadError::Decode(e.to_string()))
                .map(|img| img.to_rgba8())
        })
        .await
        .map_err(|e| AssetLoadError::Decode(e.to_string()))??;

        Ok(BoxedAssetResult::Image(image, false))
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

/// 图集加载器
#[derive(Clone)]
pub struct AtlasAssetLoader;

#[async_trait]
impl AssetLoader for AtlasAssetLoader {
    fn extensions(&self) -> &[&str] {
        &["atlas", "json"]
    }

    async fn load(
        &self,
        _path: &Path,
        bytes: Vec<u8>,
    ) -> Result<BoxedAssetResult, AssetLoadError> {
        Ok(BoxedAssetResult::Bytes(bytes))
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

// =============================================================================
// GLTF加载器（可选feature）- 唯一的条件编译位置
// =============================================================================

/// GLTF加载器（可选feature）
#[cfg(feature = "gltf")]
pub struct GltfAssetLoaderWrapper {
    pub inner: super::gltf_assets::GltfAssetLoader,
}

#[cfg(feature = "gltf")]
impl Clone for GltfAssetLoaderWrapper {
    fn clone(&self) -> Self {
        Self {
            inner: super::gltf_assets::GltfAssetLoader,
        }
    }
}

#[cfg(feature = "gltf")]
#[async_trait]
impl AssetLoader for GltfAssetLoaderWrapper {
    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }

    async fn load(
        &self,
        _path: &Path,
        bytes: Vec<u8>,
    ) -> Result<BoxedAssetResult, AssetLoadError> {
        let scene = super::gltf_assets::GltfAssetLoader::load_from_bytes(bytes)
            .await
            .map_err(|e| AssetLoadError::Decode(e))?;

        Ok(BoxedAssetResult::Custom(Box::new(scene)))
    }

    fn clone_box(&self) -> Box<dyn AssetLoader> {
        Box::new(self.clone())
    }
}

// =============================================================================
// 加载器注册表 - 运行时动态管理
// =============================================================================

/// 创建默认加载器集合
/// ✅ 仅此一处条件编译！
pub fn create_default_loaders() -> Vec<Box<dyn AssetLoader>> {
    let mut loaders: Vec<Box<dyn AssetLoader>> = vec![
        Box::new(TextureAssetLoader),
        Box::new(AtlasAssetLoader),
    ];

    // 条件编译：仅在feature启用时添加GLTF加载器
    #[cfg(feature = "gltf")]
    loaders.push(Box::new(GltfAssetLoaderWrapper {
        inner: super::gltf_assets::GltfAssetLoader,
    }));

    loaders
}

/// 加载器注册表
#[derive(Clone)]
pub struct AssetLoaderRegistry {
    loaders: Vec<Box<dyn AssetLoader>>,
    extension_map: std::collections::HashMap<String, usize>,
}

impl AssetLoaderRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self {
            loaders: Vec::new(),
            extension_map: std::collections::HashMap::new(),
        }
    }

    /// 使用默认加载器创建注册表
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();
        for loader in create_default_loaders() {
            registry.register(loader);
        }
        registry
    }

    /// 注册加载器
    pub fn register(&mut self, loader: Box<dyn AssetLoader>) {
        let index = self.loaders.len();
        for ext in loader.extensions() {
            self.extension_map.insert(ext.to_lowercase(), index);
        }
        self.loaders.push(loader);
    }

    /// 根据文件扩展名获取加载器
    pub fn get_loader(&self, path: &Path) -> Option<&dyn AssetLoader> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.extension_map.get(&ext.to_lowercase()))
            .map(|&index| self.loaders[index].as_ref())
    }

    /// 检查是否支持某种扩展名
    pub fn supports(&self, extension: &str) -> bool {
        self.extension_map.contains_key(&extension.to_lowercase())
    }
}

impl Default for AssetLoaderRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}
```

---

## 文件: manager.rs 优化部分

### 1. 资源任务枚举 - 无条件编译

```rust
/// 资源加载任务 - 统一接口（无特征依赖）
pub enum AssetTask {
    Texture {
        path: PathBuf,
        handle: Arc<AssetContainer<u32>>,
        is_linear: bool,
        start: std::time::Instant,
    },
    Atlas {
        path: PathBuf,
        handle: Arc<AssetContainer<Atlas>>,
        start: std::time::Instant,
    },
    /// ✅ 通用加载任务（使用动态加载器，无需条件编译）
    Generic {
        path: PathBuf,
        handle_type_id: TypeId,
        handle: Arc<AssetContainer<Box<dyn Any + Send + Sync>>>,
        start: std::time::Instant,
    },
}

/// 资源加载结果 - 统一接口（无特征依赖）
pub enum AssetResult {
    Image(image::RgbaImage),
    Bytes(Vec<u8>),
    Custom(Box<dyn Any + Send + Sync>), // ✅ 支持任意类型
}
```

### 2. 资源统计 - 无特征依赖

```rust
/// 资源统计信息（无特征依赖）
#[derive(Debug, Default, Clone)]
pub struct AssetStats {
    pub loaded_textures: usize,
    pub loaded_atlases: usize,
    pub loaded_custom: usize,  // ✅ 替代 loaded_gltf_scenes
    pub failed_textures: usize,
    pub failed_atlases: usize,
    pub failed_custom: usize,  // ✅ 替代 gltf 失败统计
    pub total_memory_bytes: usize,
    pub average_load_time_ms: f64,
}
```

### 3. 资源事件 - 无特征依赖

```rust
/// 资源事件（无特征依赖）
#[derive(Clone, Debug)]
pub enum AssetEvent {
    TextureLoaded(Handle<u32>, f32),
    AtlasLoaded(Handle<Atlas>, f32),
    CustomLoaded {  // ✅ 替代 GltfLoaded
        type_name: String,
        handle: Arc<AssetContainer<Box<dyn Any + Send + Sync>>>,
        time_ms: f32,
    },
    TextureFailed(Handle<u32>, String),
    AtlasFailed(Handle<Atlas>, String),
    CustomFailed {
        type_name: String,
        error: String,
    },
}
```

### 4. AssetServer结构 - 使用加载器注册表

```rust
#[derive(Resource)]
pub struct AssetServer {
    tx: mpsc::UnboundedSender<AssetTask>,
    rx: mpsc::UnboundedReceiver<(AssetTask, Result<AssetResult, String>)>,
    worker_handle: Option<std::thread::JoinHandle<()>>,
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// ✅ 加载器注册表 - 运行时动态管理
    loader_registry: Arc<RwLock<AssetLoaderRegistry>>,
    texture_count: std::sync::atomic::AtomicUsize,
    stats: std::sync::RwLock<AssetStats>,
}

impl AssetServer {
    pub fn new() -> Self {
        let (task_tx, task_rx) = mpsc::unbounded_channel::<AssetTask>();
        let (done_tx, done_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // ✅ 创建加载器注册表（自动包含所有可用加载器）
        let loader_registry = Arc::new(RwLock::new(AssetLoaderRegistry::with_defaults()));

        // ... 工作线程创建 ...

        Self {
            tx: task_tx,
            rx: done_rx,
            worker_handle: Some(worker_handle),
            shutdown_tx: Some(shutdown_tx),
            loader_registry,  // ✅ 保存注册表引用
            texture_count: std::sync::atomic::AtomicUsize::new(0),
            stats: std::sync::RwLock::new(AssetStats::default()),
        }
    }
}
```

### 5. 任务处理 - 无条件编译的通用逻辑

```rust
impl AssetServer {
    /// 处理任务 - 使用加载器注册表（无特征依赖）
    async fn process_task(
        task: &AssetTask,
        registry: &Arc<RwLock<AssetLoaderRegistry>>,
    ) -> Result<AssetResult, String> {
        match task {
            AssetTask::Texture { path, .. } => {
                match tokio::fs::read(path).await {
                    Ok(bytes) => {
                        let decode_res = tokio::task::spawn_blocking(move || {
                            image::load_from_memory(&bytes)
                                .map(|img| AssetResult::Image(img.to_rgba8()))
                                .map_err(|e| e.to_string())
                        }).await;
                        decode_res.unwrap_or(Err("Decode failed".to_string()))
                    },
                    Err(e) => Err(e.to_string()),
                }
            }
            AssetTask::Atlas { path, .. } => {
                tokio::fs::read(path).await
                    .map(AssetResult::Bytes)
                    .map_err(|e| e.to_string())
            }
            AssetTask::Generic { path, .. } => {
                // ✅ 使用加载器注册表动态加载
                let registry = registry.read().map_err(|e| e.to_string())?;
                let loader = registry.get_loader(path)
                    .ok_or_else(|| format!("No loader found for: {}", path.display()))?;

                let bytes = tokio::fs::read(path).await
                    .map_err(|e| e.to_string())?;

                match loader.load(path, bytes).await {
                    Ok(BoxedAssetResult::Image(img, _)) => Ok(AssetResult::Image(img)),
                    Ok(BoxedAssetResult::Bytes(bytes)) => Ok(AssetResult::Bytes(bytes)),
                    Ok(BoxedAssetResult::Custom(custom)) => Ok(AssetResult::Custom(custom)),
                    Err(e) => Err(e.to_string()),
                }
            }
        }
    }
}
```

### 6. 向后兼容接口 - 仅2处条件编译

```rust
// =============================================================================
// GLTF 特定接口（保持向后兼容）
// =============================================================================
// ✅ 仅此2处需要条件编译（为保持向后兼容）
#[cfg(feature = "gltf")]
pub async fn load_gltf_async(&self, path: &Path) -> Result<Handle<GltfScene>, String> {
    let handle = Handle::new_loading();
    let container: Arc<AssetContainer<Box<dyn Any + Send + Sync>>> =
        unsafe { std::mem::transmute(handle.container.clone()) };

    let task = AssetTask::Generic {
        path: path.to_path_buf(),
        handle_type_id: TypeId::of::<GltfScene>(),
        handle: container,
        start: std::time::Instant::now(),
    };

    let _ = self.tx.send(task);

    // 等待加载完成后转换类型
    let handle_any: Handle<Box<dyn Any + Send + Sync>> =
        unsafe { std::mem::transmute(handle) };
    self.wait_for_load(&handle_any).await?;

    unsafe { std::mem::transmute(handle_any) }
}

#[cfg(feature = "gltf")]
pub fn load_gltf(&self, path: &Path) -> Handle<GltfScene> {
    let handle = Handle::new_loading();
    let container: Arc<AssetContainer<Box<dyn Any + Send + Sync>>> =
        unsafe { std::mem::transmute(handle.container.clone()) };

    let task = AssetTask::Generic {
        path: path.to_path_buf(),
        handle_type_id: TypeId::of::<GltfScene>(),
        handle: container,
        start: std::time::Instant::now(),
    };

    let _ = self.tx.send(task);
    unsafe { std::mem::transmute(handle) }
}
```

### 7. 注册自定义加载器

```rust
impl AssetServer {
    /// 注册自定义加载器
    pub fn register_loader(&self, loader: Box<dyn AssetLoader>) {
        if let Ok(mut registry) = self.loader_registry.write() {
            registry.register(loader);
        }
    }
}
```

---

## 使用示例

```rust
// 创建服务器（自动注册所有加载器）
let server = AssetServer::new();

// 纹理加载
let texture = server.load_texture(Path::new("player.png")).await?;

// 图集加载
let atlas = server.load_atlas(Path::new("ui.atlas")).await?;

// GLTF加载（feature="gltf"时可用）
#[cfg(feature = "gltf")]
let gltf = server.load_gltf(Path::new("scene.gltf")).await?;

// 注册自定义加载器
server.register_loader(Box::new(MyCustomLoader::new()));

// 使用自定义加载器加载
let custom = server.load_async::<MyCustomType>(Path::new("file.custom")).await?;
```

---

## 对比总结

### 原始代码 (13处条件编译)
```rust
// 遍布多个位置的条件编译
#[cfg(feature = "gltf")] pub use ...;
#[cfg(feature = "gltf")] Gltf { ... },
#[cfg(feature = "gltf")] pub loaded_gltf_scenes: usize,
#[cfg(feature = "gltf")] GltfLoaded(...),
// ... 等等，共13处
```

### 优化后代码 (3处条件编译)
```rust
// 仅在3处保留条件编译
// 1. asset_loader_trait.rs - 加载器创建
#[cfg(feature = "gltf")]
loaders.push(Box::new(GltfAssetLoaderWrapper { ... }));

// 2-3. manager.rs - 向后兼容方法
#[cfg(feature = "gltf")]
pub async fn load_gltf_async(...) { ... }

#[cfg(feature = "gltf")]
pub fn load_gltf(...) { ... }
```

**核心思想**: 使用trait对象和运行时注册表将条件编译集中在加载器创建处，其他代码完全解耦。
