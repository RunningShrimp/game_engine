//! 场景管理器插件
//!
//! 提供场景管理功能，支持场景加载、保存、切换等。

use crate::impl_default;
use crate::plugins::{EnginePlugin, App, PluginVersion, PluginDependency};
use crate::scene::{SceneManager, scene_update_system, scene_load_system, scene_cleanup_system};

/// 场景插件配置
#[derive(Debug, Clone)]
pub struct SceneConfig {
    /// 是否启用场景序列化
    pub enable_serialization: bool,
    /// 最大场景缓存数量
    pub max_cached_scenes: usize,
    /// 默认场景过渡类型
    pub default_transition_duration: f32,
}

impl_default!(SceneConfig {
    enable_serialization: true,
    max_cached_scenes: 10,
    default_transition_duration: 0.5,
});

/// 场景管理器插件
pub struct ScenePlugin {
    config: SceneConfig,
}

impl ScenePlugin {
    /// 创建场景插件
    pub fn new() -> Self {
        Self {
            config: SceneConfig::default(),
        }
    }

    /// 使用自定义配置创建场景插件
    pub fn with_config(config: SceneConfig) -> Self {
        Self { config }
    }
}

impl EnginePlugin for ScenePlugin {
    fn name(&self) -> &'static str {
        "ScenePlugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Provides comprehensive scene management with loading, saving, switching, and transitions"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            // 场景插件可能依赖于资源管理
            PluginDependency {
                name: "ResourcePlugin".to_string(),
                version_requirement: ">=1.0.0".to_string(),
            },
        ]
    }

    fn build(&self, app: &mut App) {
        // 插入场景配置和场景管理器
        app.insert_resource(self.config.clone());
        app.insert_resource(SceneManager::new());

        // 添加场景系统
        app.add_systems(scene_update_system);
        app.add_systems(scene_load_system);
        app.add_systems(scene_cleanup_system);
    }

    fn startup(&self, world: &mut bevy_ecs::world::World) {
        println!("Scene plugin started:");
        println!("  Serialization: {}", self.config.enable_serialization);
        println!("  Max cached scenes: {}", self.config.max_cached_scenes);
        println!("  Default transition: {}s", self.config.default_transition_duration);

        // 创建默认场景
        create_default_scenes(world);
    }

    fn update(&self, _world: &mut bevy_ecs::world::World) {
        // 场景更新逻辑已在系统函数中处理
    }

    fn shutdown(&self, _world: &mut bevy_ecs::world::World) {
        println!("Scene plugin shutting down");
    }
}

/// 创建默认场景
fn create_default_scenes(world: &mut bevy_ecs::world::World) {
    let mut scene_manager = world.get_resource_mut::<SceneManager>().unwrap();

    // 创建主菜单场景
    let main_menu_id = scene_manager.create_scene("Main Menu".to_string());
    scene_manager.set_scene_metadata(main_menu_id, "type".to_string(), "menu".to_string());
    scene_manager.set_scene_metadata(main_menu_id, "music".to_string(), "menu_theme.mp3".to_string());

    // 创建游戏场景
    let game_scene_id = scene_manager.create_scene("Game Level 1".to_string());
    scene_manager.set_scene_metadata(game_scene_id, "type".to_string(), "level".to_string());
    scene_manager.set_scene_metadata(game_scene_id, "difficulty".to_string(), "normal".to_string());

    // 创建暂停场景
    let pause_scene_id = scene_manager.create_scene("Pause Menu".to_string());
    scene_manager.set_scene_metadata(pause_scene_id, "type".to_string(), "overlay".to_string());

    // 设置主菜单为当前场景
    scene_manager.switch_to_scene_immediate(main_menu_id);

    println!("Created default scenes: Main Menu, Game Level 1, Pause Menu");
}