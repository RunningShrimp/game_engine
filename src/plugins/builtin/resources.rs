//! 资源管理系统插件
//!
//! 提供统一的资源加载、管理和缓存功能。

use crate::impl_default;
use crate::plugins::{EnginePlugin, App, PluginVersion, PluginDependency};
use crate::resources::{CoroutineAssetLoader, CoroutineLoaderConfig, UploadQueue, StagingBufferPool};

/// 资源插件配置
#[derive(Debug, Clone)]
pub struct ResourceConfig {
    /// 资源缓存大小 (MB)
    pub cache_size_mb: usize,
    /// 是否启用异步加载
    pub async_loading: bool,
    /// 是否启用资源压缩
    pub compression: bool,
    /// 资源热重载
    pub hot_reload: bool,
    /// 协程加载器配置
    pub loader_config: CoroutineLoaderConfig,
}

impl_default!(ResourceConfig {
    cache_size_mb: 512,
    async_loading: true,
    compression: false,
    hot_reload: true,
    loader_config: CoroutineLoaderConfig::default(),
});

/// 资源管理系统插件
pub struct ResourcePlugin {
    config: ResourceConfig,
}

impl ResourcePlugin {
    /// 创建资源插件
    pub fn new() -> Self {
        Self {
            config: ResourceConfig::default(),
        }
    }

    /// 使用自定义配置创建资源插件
    pub fn with_config(config: ResourceConfig) -> Self {
        Self { config }
    }
}

impl EnginePlugin for ResourcePlugin {
    fn name(&self) -> &'static str {
        "ResourcePlugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Provides comprehensive resource loading, management, caching, and hot reloading"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            // 资源插件通常是基础插件
        ]
    }

    fn build(&self, app: &mut App) {
        // 插入资源配置和系统
        app.insert_resource(self.config.clone());
        app.insert_resource(CoroutineAssetLoader::new(self.config.loader_config.clone()));
        app.insert_resource(UploadQueue::new());
        app.insert_resource(StagingBufferPool::new());

        // 添加资源系统
        app.add_systems(resource_loading_system);
        app.add_systems(resource_upload_system);
        app.add_systems(resource_cleanup_system);
    }

    fn startup(&self, world: &mut bevy_ecs::world::World) {
        println!("Resource plugin started:");
        println!("  Cache size: {} MB", self.config.cache_size_mb);
        println!("  Async loading: {}", self.config.async_loading);
        println!("  Compression: {}", self.config.compression);
        println!("  Hot reload: {}", self.config.hot_reload);

        // 初始化资源加载器
        initialize_default_resources(world);
    }

    fn update(&self, _world: &mut bevy_ecs::world::World) {
        // 资源更新逻辑已在系统函数中处理
    }

    fn shutdown(&self, _world: &mut bevy_ecs::world::World) {
        println!("Resource plugin shutting down");
    }
}

/// 资源加载系统
pub fn resource_loading_system(
    mut loader: ResMut<CoroutineAssetLoader>,
    time: Res<crate::ecs::Time>,
) {
    // 更新加载器
    loader.update(time.delta_seconds);
}

/// 资源上传系统
pub fn resource_upload_system(
    mut upload_queue: ResMut<UploadQueue>,
    mut staging_pool: ResMut<StagingBufferPool>,
) {
    // 处理上传队列
    upload_queue.process_uploads(&mut staging_pool);
}

/// 资源清理系统
pub fn resource_cleanup_system(
    mut loader: ResMut<CoroutineAssetLoader>,
) {
    // 清理未使用的资源
    loader.cleanup_unused();
}

/// 初始化默认资源
fn initialize_default_resources(world: &mut bevy_ecs::world::World) {
    let mut loader = world.get_resource_mut::<CoroutineAssetLoader>().unwrap();

    // 加载默认纹理
    let _ = loader.load_texture("default_white", "assets/textures/default_white.png");
    let _ = loader.load_texture("default_normal", "assets/textures/default_normal.png");

    // 加载默认字体
    let _ = loader.load_font("default_font", "assets/fonts/default.ttf");

    // 加载默认着色器
    let _ = loader.load_shader("default_vertex", "assets/shaders/default.vert");
    let _ = loader.load_shader("default_fragment", "assets/shaders/default.frag");

    println!("Initialized default resources");
}