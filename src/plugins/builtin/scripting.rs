//! 脚本插件
//!
//! 提供Lua和Rust脚本集成，支持运行时脚本执行。

use crate::plugins::{EnginePlugin, App, PluginVersion, PluginDependency};
use crate::scripting::{ScriptingConfig, ScriptingResource, scripting_system, setup_scripting, ScriptComponent};
use bevy_ecs::prelude::*;

/// 脚本插件
pub struct ScriptingPlugin {
    config: ScriptingConfig,
}

impl ScriptingPlugin {
    /// 创建脚本插件
    pub fn new() -> Self {
        Self {
            config: ScriptingConfig::default(),
        }
    }

    /// 使用自定义配置创建脚本插件
    pub fn with_config(config: ScriptingConfig) -> Self {
        Self { config }
    }
}

impl EnginePlugin for ScriptingPlugin {
    fn name(&self) -> &'static str {
        "ScriptingPlugin"
    }

    fn version(&self) -> PluginVersion {
        PluginVersion::new(1, 0, 0)
    }

    fn description(&self) -> &'static str {
        "Provides comprehensive scripting capabilities with Lua and Rust integration"
    }

    fn dependencies(&self) -> Vec<PluginDependency> {
        vec![
            // 脚本插件通常是独立的，但可能依赖于核心ECS
        ]
    }

    fn build(&self, app: &mut App) {
        // 初始化脚本系统
        app.world_mut().insert_resource(self.config.clone());

        // 设置脚本系统
        setup_scripting(app.world_mut(), self.config.clone());

        // 添加脚本系统
        app.add_systems(scripting_system);
    }

    fn startup(&self, world: &mut bevy_ecs::world::World) {
        println!("Scripting plugin started:");
        println!("  Lua: {}", self.config.enable_lua);
        println!("  Rust: {}", self.config.enable_rust);
        println!("  JavaScript: {}", self.config.enable_javascript);
        println!("  Python: {}", self.config.enable_python);
        println!("  Hot reload: {}", self.config.hot_reload);

        // 创建示例脚本实体
        if self.config.enable_lua {
            create_example_lua_script(world);
        }

        if self.config.enable_rust {
            create_example_rust_script(world);
        }
    }

    fn update(&self, _world: &mut bevy_ecs::world::World) {
        // 脚本更新逻辑已在scripting_system中处理
    }

    fn shutdown(&self, _world: &mut bevy_ecs::world::World) {
        println!("Scripting plugin shutting down");
    }
}

/// 创建示例Lua脚本
fn create_example_lua_script(world: &mut World) {
    let lua_script = r#"
-- 示例Lua脚本
print("Hello from Lua script!")

-- 访问当前实体
local entity_id = current_entity
print("Current entity ID: " .. tostring(entity_id))

-- 模拟一些游戏逻辑
local health = 100
print("Player health: " .. health)

return
"#;

    let script_component = ScriptComponent {
        language: crate::scripting::ScriptLanguage::Lua,
        script_name: "example_lua".to_string(),
        script_source: lua_script.to_string(),
        enabled: true,
        execution_frequency: crate::scripting::ScriptExecutionFrequency::EveryFrame,
    };

    world.spawn(script_component);
}

/// 创建示例Rust脚本
fn create_example_rust_script(world: &mut World) {
    let rust_script = r#"
// 示例Rust脚本
println!("Hello from Rust script!");
// 简单的游戏逻辑
let mut score = 0;
score += 10;
println!("Score: {}", score);
"#;

    let script_component = ScriptComponent {
        language: crate::scripting::ScriptLanguage::Rust,
        script_name: "example_rust".to_string(),
        script_source: rust_script.to_string(),
        enabled: true,
        execution_frequency: crate::scripting::ScriptExecutionFrequency::EveryFrame,
    };

    world.spawn(script_component);
}