//  音频插件
//
//  提供音频播放功能，支持2D和3D空间音频。

use crate::impl_default;
use crate::plugins::{App, EnginePlugin, PluginDependency, PluginVersion};
use bevy_ecs::prelude::*;

/// 音频状态资源
///
/// 管理全局音频状态，包括主音量和播放中的音频数量。
#[derive(Debug, Clone, bevy_ecs::prelude::Resource)]
pub struct AudioState {
    /// 主音量 (0.0 - 2.0)
    pub master_volume: f32,
    /// 当前播放中的音频数量
    pub active_sounds: usize,
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            active_sounds: 0,
        }
    }
}

/// 音频更新系统
///
/// 每帧更新音频状态，可用于清理已完成的音频播放。
pub fn audio_update_system(audio_state: ResMut<AudioState>) {
    // 这里可以添加音频清理逻辑
    // 例如：检查并移除已完成的音频播放
    // 目前仅作为占位符系统，确保音频状态被更新

    // 更新活动音频计数（实际应用中应从AudioService获取）
    if audio_state.active_sounds > 0 {
        // 模拟音频播放结束的逻辑
        // 实际实现需要与AudioService集成
    }
}

/// 音频插件配置
#[derive(Debug, Clone, bevy_ecs::prelude::Resource)]
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

impl Default for AudioPlugin {
    fn default() -> Self {
        Self::new()
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
        // 插入音频配置和状态
        let audio_state = AudioState {
            master_volume: self.config.master_volume,
            ..Default::default()
        };
        app.insert_resource(self.config.clone());
        app.insert_resource(audio_state);

        // 添加音频更新系统到调度器
        app.add_systems(|schedule| {
            schedule.add_systems(audio_update_system);
        });

        // 注意：空间音频系统需要在应用中单独注册
        // 因为它们需要额外的参数（如监听器位置）
        if self.config.enable_spatial_audio {
            tracing::info!("Spatial audio enabled - register update_listener_system separately");
        }
    }

    fn startup(&self, _world: &mut bevy_ecs::world::World) {
        println!(
            "Audio plugin started with master volume: {}",
            self.config.master_volume
        );
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
