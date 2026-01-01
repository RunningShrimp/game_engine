//  脚本系统完整实现
//
//  提供Lua、TypeScript、Python和Rust脚本集成，支持运行时脚本执行、热重载和跨语言互操作。

use crate::impl_default;
/// 脚本API模块
pub mod api;
/// 音频系统脚本API模块
pub mod audio_api;
/// C#支持模块
#[cfg(feature = "csharp")]
pub mod csharp;
/// C#生命周期钩子模块
#[cfg(feature = "csharp")]
pub mod csharp_lifecycle;
/// ECS脚本绑定模块
pub mod ecs_bindings;
/// 脚本引擎模块
pub mod engine;
/// 实体API模块
pub mod entity_api;
/// 扩展绑定模块
pub mod extended_bindings;
/// 跨语言FFI层
pub mod ffi;
/// 图形UI绑定模块
pub mod graphics_ui_bindings;
/// 输入事件分发模块
pub mod input_dispatcher;
/// JavaScript生命周期钩子模块
pub mod javascript_lifecycle;
/// 生命周期钩子系统
pub mod lifecycle;
/// Lua生命周期钩子模块
pub mod lua_lifecycle;
/// Lua支持模块
pub mod lua_support;
/// 移动平台服务脚本API模块
pub mod mobile_api;
/// 网络API绑定模块
pub mod network_api;
/// 物理系统脚本API模块
pub mod physics_api;
/// 物理音频绑定模块
pub mod physics_audio_bindings;
/// Python支持模块
#[cfg(feature = "pyo3")]
pub mod python;
/// Python生命周期钩子模块
#[cfg(feature = "pyo3")]
pub mod python_lifecycle;
/// 资源管理脚本API模块
pub mod resource_api;
/// Rust脚本模块
pub mod rust_scripting;
/// 脚本系统模块
pub mod system;
/// 线程安全模块
pub mod thread_safe;
/// TypeScript支持模块
#[cfg(feature = "typescript")]
pub mod typescript;
/// TypeScript生命周期钩子模块
#[cfg(feature = "typescript")]
pub mod typescript_lifecycle;

#[cfg(test)]
mod lua_tests;

#[cfg(test)]
mod compatibility_tests;

pub use audio_api::AudioScriptApi;
pub use engine::*;
pub use entity_api::{EntityApi, EntityQueryBuilder, EntityTemplate, TemplateComponent};
pub use javascript_lifecycle::{JavaScriptLifecycleHooks, JavaScriptLifecycleHooksFactory};
pub use lifecycle::{LifecycleHooks, LifecycleHooksComponent, LifecyclePhase, LifecycleScheduler};
#[cfg(feature = "mlua")]
pub use lua_lifecycle::{LuaLifecycleHooks, LuaLifecycleHooksFactory};
pub use lua_support::{LuaContext, LuaEngine, LuaValue};
pub use network_api::{NetworkApi, NetworkScriptContext};
pub use physics_api::PhysicsScriptApi;
pub use rust_scripting::{RustScriptContext, RustScriptContextAdapter, RustScriptEngine};
pub use system::{JavaScriptContext, PythonContext};
pub use system::{ScriptContext, ScriptLanguage, ScriptResult, ScriptSystem, ScriptValue};

#[cfg(feature = "typescript")]
pub use typescript::{TypeScriptContext, TypeScriptRuntime};
#[cfg(feature = "typescript")]
pub use typescript_lifecycle::{TypeScriptLifecycleHooks, TypeScriptLifecycleHooksFactory};

#[cfg(feature = "pyo3")]
pub use python::{PythonContextImpl, PythonRuntime};
#[cfg(feature = "pyo3")]
pub use python_lifecycle::{PythonLifecycleHooks, PythonLifecycleHooksFactory};

#[cfg(feature = "csharp")]
pub use csharp::{CSharpConfig, CSharpContext, CSharpRuntime};
#[cfg(feature = "csharp")]
pub use csharp_lifecycle::{CSharpLifecycleHooks, CSharpLifecycleHooksFactory};

use bevy_ecs::prelude::*;

/// 脚本系统配置
#[derive(Debug, Clone, bevy_ecs::prelude::Resource)]
pub struct ScriptingConfig {
    /// 是否启用Lua脚本
    pub enable_lua: bool,
    /// 是否启用Rust脚本
    pub enable_rust: bool,
    /// 是否启用JavaScript脚本
    pub enable_javascript: bool,
    /// 是否启用TypeScript脚本
    pub enable_typescript: bool,
    /// 是否启用Python脚本
    pub enable_python: bool,
    /// 是否启用C#脚本
    pub enable_csharp: bool,
    /// 脚本热重载
    pub hot_reload: bool,
    /// 脚本执行超时 (毫秒)
    pub execution_timeout_ms: u64,
}

impl_default!(ScriptingConfig {
    enable_lua: true,
    enable_rust: false,
    enable_javascript: false,
    enable_typescript: false,
    enable_python: false,
    enable_csharp: true,
    hot_reload: true,
    execution_timeout_ms: 5000,
});

/// 脚本系统资源
#[derive(Resource)]
pub struct ScriptingResource {
    /// 脚本系统
    pub system: ScriptSystem,
    /// Lua引擎
    pub lua_engine: Option<LuaEngine>,
    /// Rust脚本引擎
    pub rust_engine: Option<RustScriptEngine>,
    /// C#运行时
    #[cfg(feature = "csharp")]
    pub csharp_runtime: Option<CSharpRuntime>,
    /// 脚本系统配置
    pub config: ScriptingConfig,
}

impl Default for ScriptingResource {
    fn default() -> Self {
        Self {
            system: ScriptSystem::new(),
            lua_engine: None,
            rust_engine: None,
            #[cfg(feature = "csharp")]
            csharp_runtime: None,
            config: ScriptingConfig::default(),
        }
    }
}

/// 脚本组件 - 附加到需要脚本行为的实体
#[derive(Component, Debug, Clone)]
pub struct ScriptComponent {
    /// 脚本语言
    pub language: ScriptLanguage,
    /// 脚本名称
    pub script_name: String,
    /// 脚本代码或路径
    pub script_source: String,
    /// 是否启用
    pub enabled: bool,
    /// 执行频率
    pub execution_frequency: ScriptExecutionFrequency,
}

/// 脚本执行频率
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptExecutionFrequency {
    /// 每帧执行
    EveryFrame,
    /// 固定时间间隔 (毫秒)
    FixedInterval(u64),
    /// 事件触发
    OnEvent,
}

/// 脚本系统
pub fn scripting_system(
    mut scripting: ResMut<ScriptingResource>,
    query: Query<(Entity, &ScriptComponent)>,
    time: Res<crate::ecs::Time>,
) {
    // 执行所有启用的脚本组件
    for (entity, script) in query.iter() {
        if !script.enabled {
            continue;
        }

        // 检查执行频率
        if !should_execute_script(script, &time) {
            continue;
        }

        // 执行脚本
        let result = execute_script(&mut scripting, script, entity);
        if let Err(e) = result {
            tracing::error!(target: "scripting", "Script execution error for entity {:?}: {}", entity, e);
        }
    }

    // 执行Lua引擎更新
    if let Some(ref mut lua_engine) = scripting.lua_engine {
        // 这里可以添加Lua特定的更新逻辑
        let _ = lua_engine;
    }

    // 执行Rust脚本引擎更新
    if let Some(ref mut rust_engine) = scripting.rust_engine {
        rust_engine.update();
    }
}

/// 检查脚本是否应该执行
fn should_execute_script(script: &ScriptComponent, time: &crate::ecs::Time) -> bool {
    match script.execution_frequency {
        ScriptExecutionFrequency::EveryFrame => true,
        ScriptExecutionFrequency::FixedInterval(interval_ms) => {
            // 简化实现：使用elapsed时间的取模
            let elapsed_ms = (time.elapsed_seconds * 1000.0) as u64;
            elapsed_ms.is_multiple_of(interval_ms.max(1))
        }
        ScriptExecutionFrequency::OnEvent => false, // 需要事件触发
    }
}

/// 执行脚本
fn execute_script(
    scripting: &mut ScriptingResource,
    script: &ScriptComponent,
    entity: Entity,
) -> Result<(), String> {
    match script.language {
        ScriptLanguage::Lua => {
            if let Some(ref mut lua_engine) = scripting.lua_engine {
                // 设置实体上下文
                lua_engine.context.set_global(
                    "current_entity",
                    lua_support::LuaValue::Number(entity.to_bits() as f64),
                ); // 执行脚本
                lua_engine.execute(&script.script_name, &script.script_source)?;
            } else {
                return Err("Lua engine not available".to_string());
            }
        }
        ScriptLanguage::Rust => {
            if let Some(ref mut rust_engine) = scripting.rust_engine {
                rust_engine.execute_script(&script.script_name, &script.script_source)?;
            } else {
                // Rust脚本引擎未启用，尝试使用通用脚本系统（如果已注册）
                // 检查通用脚本系统是否已注册Rust上下文
                let result = scripting.system.execute_script(
                    &script.script_name,
                    &script.script_source,
                    script.language,
                );
                match result {
                    ScriptResult::Error(e) => {
                        // 如果通用脚本系统也失败，返回错误
                        if scripting.config.enable_rust {
                            return Err(format!("Rust script engine initialization failed: {e}"));
                        } else {
                            return Err(format!(
                                "Rust script engine not enabled. Set enable_rust=true in ScriptingConfig. Error: {e}"
                            ));
                        }
                    }
                    _ => {
                        // 通用脚本系统执行成功
                    }
                }
            }
        }
        ScriptLanguage::JavaScript | ScriptLanguage::TypeScript | ScriptLanguage::Python => {
            // 使用通用脚本系统
            let _result = scripting.system.execute_script(
                &script.script_name,
                &script.script_source,
                script.language,
            );
            // 结果处理已由execute_script内部处理
        }
        ScriptLanguage::CSharp => {
            #[cfg(feature = "csharp")]
            {
                if let Some(ref runtime) = scripting.csharp_runtime {
                    // 创建上下文并执行
                    let mut ctx = runtime.create_context();
                    // 设置实体上下文
                    let _ = ctx.set_global(
                        "current_entity",
                        ScriptValue::Number(entity.to_bits() as f64),
                    );
                    // 执行脚本
                    let result = ctx.execute(&script.script_name, Some(&script.script_source));
                    match result {
                        ScriptResult::Error(e) => {
                            return Err(format!("C# script execution error: {}", e));
                        }
                        _ => {}
                    }
                } else {
                    return Err("C# runtime not available".to_string());
                }
            }
            #[cfg(not(feature = "csharp"))]
            {
                return Err(
                    "C# script support not enabled. Enable with 'csharp' feature.".to_string(),
                );
            }
        }
    }

    Ok(())
}

/// 初始化脚本系统
pub fn setup_scripting(world: &mut World, config: ScriptingConfig) {
    let mut resource = ScriptingResource::default();
    resource.config = config.clone();

    // 初始化Lua引擎
    if config.enable_lua {
        let mut lua_engine = LuaEngine::new();
        lua_engine.register_engine_api();
        resource.lua_engine = Some(lua_engine);
    }

    // 初始化Rust脚本引擎
    if config.enable_rust {
        let rust_engine = RustScriptEngine::new();
        // 创建适配器包装引擎（使用Arc共享）
        let adapter_engine = RustScriptEngine::new();
        // 同时注册到通用脚本系统
        resource.system.register_context(
            ScriptLanguage::Rust,
            Box::new(RustScriptContextAdapter::new(adapter_engine)),
        );
        resource.rust_engine = Some(rust_engine);
    }

    // 注册Lua上下文到通用脚本系统
    if config.enable_lua {
        resource
            .system
            .register_context(ScriptLanguage::Lua, Box::new(LuaContext::new()));
    }

    // 注册其他脚本上下文
    if config.enable_javascript {
        resource.system.register_context(
            ScriptLanguage::JavaScript,
            Box::new(JavaScriptContext::new()),
        );
    }

    // 注册TypeScript上下文
    #[cfg(feature = "typescript")]
    if config.enable_typescript {
        resource.system.register_context(
            ScriptLanguage::TypeScript,
            Box::new(TypeScriptContext::new()),
        );
    }

    if config.enable_python {
        #[cfg(feature = "pyo3")]
        resource
            .system
            .register_context(ScriptLanguage::Python, Box::new(PythonContextImpl::new()));

        #[cfg(not(feature = "pyo3"))]
        resource
            .system
            .register_context(ScriptLanguage::Python, Box::new(PythonContext::new()));
    }

    // 初始化C#运行时
    #[cfg(feature = "csharp")]
    if config.enable_csharp {
        let csharp_runtime = CSharpRuntime::new();
        resource.csharp_runtime = Some(csharp_runtime);

        // 注册C#上下文到通用脚本系统
        resource
            .system
            .register_context(ScriptLanguage::CSharp, Box::new(CSharpContext::new()));
    }

    world.insert_resource(resource);

    // 设置生命周期系统
    lifecycle::setup_lifecycle_systems(world);
}

/// 便捷函数：创建Lua脚本组件
pub fn create_lua_script(script_name: &str, script_source: &str) -> ScriptComponent {
    ScriptComponent {
        language: ScriptLanguage::Lua,
        script_name: script_name.to_string(),
        script_source: script_source.to_string(),
        enabled: true,
        execution_frequency: ScriptExecutionFrequency::EveryFrame,
    }
}

/// 便捷函数：创建Rust脚本组件
pub fn create_rust_script(script_name: &str, script_source: &str) -> ScriptComponent {
    ScriptComponent {
        language: ScriptLanguage::Rust,
        script_name: script_name.to_string(),
        script_source: script_source.to_string(),
        enabled: true,
        execution_frequency: ScriptExecutionFrequency::EveryFrame,
    }
}

/// 便捷函数：创建C#脚本组件
#[cfg(feature = "csharp")]
pub fn create_csharp_script(script_name: &str, script_source: &str) -> ScriptComponent {
    ScriptComponent {
        language: ScriptLanguage::CSharp,
        script_name: script_name.to_string(),
        script_source: script_source.to_string(),
        enabled: true,
        execution_frequency: ScriptExecutionFrequency::EveryFrame,
    }
}
