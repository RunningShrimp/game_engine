//! 音频插件
//!
//! 提供音频播放功能，支持2D和3D空间音频。

use crate::impl_default;
use crate::plugins::{EnginePlugin, App, PluginVersion, PluginDependency};
use crate::audio::{AudioState, AudioService, audio_playback_system_v2, audio_cleanup_system_v2, audio_gc_system_v2};
use bevy_ecs::prelude::*;

/// 音频插件配置
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// 主音量 (0.0 - 2.0)
    pub master_volume: f32,
    /// 是否启用空间音频
    pub enable_spatial_audio: bool,
    /// 最大同时播放音频数量
    pub max_concurrent_sounds: usize,
}

impl_default!(AudioConfig {
    master_volume: 1.0,
    enable_spatial_audio: true,
    max_concurrent_sounds: 32,
});

/// 音频插件
pub struct AudioPlugin {
    config: AudioConfig,
}

impl AudioPlugin {
    /// 创建音频插件
    pub fn new() -> Self {
        Self {
            config: AudioConfig::default(),
        }
    }

    /// 使用自定义配置创建音频插件
    pub fn with_config(config: AudioConfig) -> Self {
        Self { config }
    }
}

impl EnginePlugin for AudioPlugin {
    fn name(&self) -> &'static str {
        "AudioPlugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Provides audio playback functionality with 2D and 3D spatial audio support"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            // 音频插件依赖于核心ECS系统
        ]
    }

    fn build(&self, app: &mut App) {
        // 插入音频状态资源
        let mut audio_state = AudioState::new();
        AudioService::set_master_volume(&mut audio_state, self.config.master_volume);

        app.insert_resource(audio_state);

        // 添加音频系统
        app.add_systems(audio_playback_system_v2);
        app.add_systems(audio_cleanup_system_v2);
        app.add_systems(audio_gc_system_v2);

        // 如果启用空间音频，添加空间音频系统
        if self.config.enable_spatial_audio {
            // 这里可以添加空间音频系统
            // app.add_systems(spatial_audio_update_system);
        }
    }

    fn startup(&self, world: &mut bevy_ecs::world::World) {
        println!("Audio plugin started with master volume: {}", self.config.master_volume);
        if self.config.enable_spatial_audio {
            println!("Spatial audio enabled");
        }
    }

    fn update(&self, _world: &mut bevy_ecs::world::World) {
        // 音频更新逻辑已在系统函数中处理
    }

    fn shutdown(&self, _world: &mut bevy_ecs::world::World) {
        println!("Audio plugin shutting down");
    }
}