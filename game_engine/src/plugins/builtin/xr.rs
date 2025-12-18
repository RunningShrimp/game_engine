//! XR插件
//!
//! 提供VR/AR功能扩展，支持OpenXR标准。

use crate::impl_default;
use crate::plugins::{EnginePlugin, App, PluginVersion, PluginDependency};

/// XR插件配置
#[derive(Debug, Clone)]
pub struct XrConfig {
    /// XR模式
    pub mode: XrMode,
    /// 是否启用手跟踪
    pub hand_tracking: bool,
    /// 是否启用眼跟踪
    pub eye_tracking: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum XrMode {
    /// 禁用XR
    Disabled,
    /// VR模式
    Vr,
    /// AR模式
    Ar,
}

impl_default!(XrConfig {
    mode: XrMode::Disabled,
    hand_tracking: false,
    eye_tracking: false,
});

/// XR插件
pub struct XrPlugin {
    config: XrConfig,
}

impl XrPlugin {
    /// 创建XR插件
    pub fn new() -> Self {
        Self {
            config: XrConfig::default(),
        }
    }

    /// 使用自定义配置创建XR插件
    pub fn with_config(config: XrConfig) -> Self {
        Self { config }
    }
}

impl EnginePlugin for XrPlugin {
    fn name(&self) -> &'static str {
        "XrPlugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Provides VR/AR capabilities with OpenXR support"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            PluginDependency {
                name: "RenderPlugin".to_string(),
                version_requirement: ">=1.0.0".to_string(),
            },
        ]
    }

    fn build(&self, app: &mut App) {
        // 插入XR配置
        app.insert_resource(self.config.clone());

        // 这里可以添加XR相关的系统
        // app.add_systems(xr_update_system);
    }

    fn startup(&self, _world: &mut bevy_ecs::world::World) {
        match self.config.mode {
            XrMode::Disabled => println!("XR plugin started (disabled)"),
            XrMode::Vr => println!("XR plugin started in VR mode"),
            XrMode::Ar => println!("XR plugin started in AR mode"),
        }
    }

    fn update(&self, _world: &mut bevy_ecs::world::World) {
        // XR更新逻辑
    }

    fn shutdown(&self, _world: &mut bevy_ecs::world::World) {
        println!("XR plugin shutting down");
    }
}