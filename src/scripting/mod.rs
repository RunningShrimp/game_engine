pub mod engine;
pub mod system;
pub mod api;
pub mod ecs_bindings;
pub mod extended_bindings;
pub mod physics_audio_bindings;
pub mod graphics_ui_bindings;

pub use engine::*;
pub use system::{ScriptSystem, ScriptLanguage, ScriptValue, ScriptResult, ScriptContext};
pub use system::{JavaScriptContext, PythonContext};

use bevy_ecs::prelude::*;

/// 脚本系统资源
#[derive(Resource)]
pub struct ScriptingResource {
    pub system: ScriptSystem,
}

impl Default for ScriptingResource {
    fn default() -> Self {
        Self {
            system: ScriptSystem::new(),
        }
    }
}

/// 脚本系统占位
pub fn scripting_system() {
    // TODO: 实现脚本执行逻辑
}

/// 初始化脚本系统
pub fn setup_scripting(world: &mut World) {
    // 创建脚本系统资源
    let mut resource = ScriptingResource::default();
    
    // 注册默认的JavaScript上下文
    resource.system.register_context(
        ScriptLanguage::JavaScript,
        Box::new(JavaScriptContext::new()),
    );
    
    // 注册默认的Python上下文
    resource.system.register_context(
        ScriptLanguage::Python,
        Box::new(PythonContext::new()),
    );
    
    world.insert_resource(resource);
}
