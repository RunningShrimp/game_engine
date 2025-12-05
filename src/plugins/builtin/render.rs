//! 渲染插件
//!
//! 提供基于wgpu的渲染功能，支持2D/3D渲染、PBR材质、阴影等。

use crate::impl_default;
use crate::plugins::{EnginePlugin, App, PluginVersion, PluginDependency};
use crate::render::wgpu::WgpuRenderer;
use bevy_ecs::prelude::*;
use winit::window::Window;

/// 渲染插件配置
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// 是否启用PBR渲染
    pub enable_pbr: bool,
    /// 是否启用阴影
    pub enable_shadows: bool,
    /// 是否启用后处理
    pub enable_postprocessing: bool,
    /// MSAA采样数
    pub msaa_samples: u32,
    /// 是否启用GPU驱动渲染
    pub enable_gpu_driven: bool,
}

impl_default!(RenderConfig {
    enable_pbr: true,
    enable_shadows: true,
    enable_postprocessing: false,
    msaa_samples: 4,
    enable_gpu_driven: false,
});

/// 渲染插件
pub struct RenderPlugin {
    config: RenderConfig,
    window: Option<std::sync::Arc<Window>>,
}

impl RenderPlugin {
    /// 创建渲染插件
    pub fn new() -> Self {
        Self {
            config: RenderConfig::default(),
            window: None,
        }
    }

    /// 使用自定义配置创建渲染插件
    pub fn with_config(config: RenderConfig) -> Self {
        Self {
            config,
            window: None,
        }
    }

    /// 设置窗口（需要在插件构建前调用）
    pub fn with_window(mut self, window: std::sync::Arc<Window>) -> Self {
        self.window = Some(window);
        self
    }
}

impl EnginePlugin for RenderPlugin {
    fn name(&self) -> &'static str {
        "RenderPlugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Provides comprehensive rendering capabilities using wgpu"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            // 渲染插件依赖于核心ECS系统
        ]
    }

    fn build(&self, app: &mut App) {
        // 需要窗口来创建渲染器
        if let Some(window) = &self.window {
            // 在实际应用中，这里需要异步创建渲染器
            // 由于插件系统是同步的，我们在这里只插入占位符
            // 实际的渲染器创建应该在应用启动后进行

            // 插入渲染配置资源
            app.insert_resource(self.config.clone());

            // 这里可以添加渲染相关的系统
            // app.add_systems(render_system);
        } else {
            eprintln!("Warning: RenderPlugin requires a window to be set before building");
        }
    }

    fn startup(&self, world: &mut bevy_ecs::world::World) {
        println!("Render plugin started with PBR: {}, Shadows: {}",
                 self.config.enable_pbr, self.config.enable_shadows);

        // 在这里可以创建实际的渲染器
        // let renderer = pollster::block_on(WgpuRenderer::new(window));
        // world.insert_resource(renderer);
    }

    fn update(&self, _world: &mut bevy_ecs::world::World) {
        // 渲染更新逻辑在渲染系统中处理
    }

    fn shutdown(&self, _world: &mut bevy_ecs::world::World) {
        println!("Render plugin shutting down");
    }
}