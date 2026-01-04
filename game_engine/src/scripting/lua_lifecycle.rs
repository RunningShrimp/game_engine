// Lua生命周期钩子实现
//
// 集成lifecycle系统与mlua Lua引擎，支持Unity风格的生命周期回调

#[cfg(feature = "mlua")]
#[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
use crate::ecs::Entity;
#[cfg(feature = "mlua")]
#[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
use crate::scripting::{
    lifecycle::{LifecycleHooks, LifecyclePhase},
    lua_support::LuaValue,
    system::{ScriptContext, ScriptLanguage, ScriptResult},
};
#[cfg(feature = "mlua")]
#[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
use mlua::{Function, Lua, Result as LuaResult, Value as LuaValueInternal};
use std::sync::{Arc, Mutex};

/// Lua生命周期钩子
///
/// 为Lua脚本提供Unity风格的生命周期回调支持
#[cfg(feature = "mlua")]
#[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
pub struct LuaLifecycleHooks {
    /// 脚本名称
    script_name: String,
    /// Lua引擎实例
    lua: Arc<Mutex<Lua>>,
    /// 实体ID (用于在Lua中标识当前实体)
    entity_id: String,
    /// 是否已启用
    enabled: Arc<Mutex<bool>>,
}

#[cfg(feature = "mlua")]
#[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
impl LuaLifecycleHooks {
    /// 创建新的Lua生命周期钩子
    ///
    /// # 参数
    ///
    /// - `script_name`: 脚本名称（用于日志和错误报告）
    /// - `lua`: Lua引擎实例（共享引用）
    /// - `entity`: ECS实体ID
    pub fn new(script_name: String, lua: Arc<Mutex<Lua>>, entity: Entity) -> Self {
        Self {
            script_name,
            lua,
            entity_id: entity.to_string(),
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// 调用Lua生命周期函数
    ///
    /// # 参数
    ///
    /// - `function_name`: 要调用的Lua函数名称
    /// - `args`: 传递给Lua函数的参数
    ///
    /// # 返回
    ///
    /// 如果调用成功返回 `Ok(())`，否则返回错误信息
    fn call_lua_function(&self, function_name: &str, args: Vec<LuaValue>) -> Result<(), String> {
        // 检查是否启用
        let enabled = self
            .enabled
            .lock()
            .map_err(|e| format!("Failed to acquire enabled lock: {}", e))?;
        if !*enabled {
            return Ok(());
        }
        drop(enabled);

        let lua = self.lua.lock().map_err(|e| format!("Failed to acquire Lua lock: {}", e))?;

        // 检查函数是否存在
        let globals = lua.globals();
        let function_exists: bool = lua
            .load(&format!("type({}) == \"function\"", function_name))
            .eval()
            .unwrap_or(false);

        if !function_exists {
            // 函数不存在不是错误，只是静默跳过
            return Ok(());
        }

        // 设置当前实体ID（作为全局变量供Lua访问）
        let _: LuaResult<()> = globals.set("__current_entity_id", self.entity_id.clone());

        // 获取函数
        let function: Function = match globals.get(function_name) {
            Ok(f) => f,
            Err(e) => {
                return Err(format!("Failed to get function '{}': {}", function_name, e));
            }
        };

        // 转换参数
        let lua_args: Vec<LuaValueInternal> = args
            .into_iter()
            .map(|v| match v {
                LuaValue::Nil => LuaValueInternal::Nil,
                LuaValue::Boolean(b) => LuaValueInternal::Boolean(b),
                LuaValue::Number(n) => LuaValueInternal::Number(n),
                LuaValue::String(s) => LuaValueInternal::String(s.into()),
                LuaValue::Table(_) => LuaValueInternal::Nil, // 简化：table暂时转为nil
            })
            .collect();

        // 调用函数
        let result: LuaResult<()> = function.call(lua_args);
        match result {
            Ok(_) => Ok(()),
            Err(e) => {
                tracing::error!(
                    target: "scripting",
                    "Lua lifecycle error in '{}' ({}): {}",
                    self.script_name,
                    function_name,
                    e
                );
                Err(format!("{}: {}", function_name, e))
            }
        }
    }

    /// 启用钩子
    pub fn enable(&self) {
        if let Ok(mut enabled) = self.enabled.lock() {
            *enabled = true;
        }
    }

    /// 禁用钩子
    pub fn disable(&self) {
        if let Ok(mut enabled) = self.enabled.lock() {
            *enabled = false;
        }
    }

    /// 将Entity转换为LuaValue
    fn entity_to_value(entity: Entity) -> LuaValue {
        LuaValue::Number(entity.to_bits() as f64)
    }

    /// 将两个Entity转换为参数数组
    fn entities_to_args(entity: Entity, other: Entity) -> Vec<LuaValue> {
        vec![Self::entity_to_value(entity), Self::entity_to_value(other)]
    }
}

#[cfg(feature = "mlua")]
#[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
impl LifecycleHooks for LuaLifecycleHooks {
    fn on_enable(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_lua_function("onEnable", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onEnable for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_disable(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_lua_function("onDisable", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onDisable for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_destroy(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_lua_function("onDestroy", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onDestroy for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_update(&mut self, entity: Entity, delta_time: f32) {
        let args = vec![
            Self::entity_to_value(entity),
            LuaValue::Number(delta_time as f64),
        ];
        if let Err(e) = self.call_lua_function("onUpdate", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onUpdate for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_fixed_update(&mut self, entity: Entity, fixed_delta_time: f32) {
        let args = vec![
            Self::entity_to_value(entity),
            LuaValue::Number(fixed_delta_time as f64),
        ];
        if let Err(e) = self.call_lua_function("onFixedUpdate", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onFixedUpdate for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_late_update(&mut self, entity: Entity, delta_time: f32) {
        let args = vec![
            Self::entity_to_value(entity),
            LuaValue::Number(delta_time as f64),
        ];
        if let Err(e) = self.call_lua_function("onLateUpdate", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onLateUpdate for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_collision_enter(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_lua_function("onCollisionEnter", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onCollisionEnter for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_collision_stay(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_lua_function("onCollisionStay", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onCollisionStay for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_collision_exit(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_lua_function("onCollisionExit", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onCollisionExit for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_trigger_enter(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_lua_function("onTriggerEnter", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onTriggerEnter for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_trigger_stay(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_lua_function("onTriggerStay", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onTriggerStay for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_trigger_exit(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_lua_function("onTriggerExit", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onTriggerExit for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_start(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_lua_function("onStart", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onStart for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_shutdown(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_lua_function("onShutdown", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onShutdown for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_key_down(&mut self, entity: Entity, key: crate::platform::KeyCode) {
        let args = vec![
            Self::entity_to_value(entity),
            LuaValue::String(format!("{:?}", key)),
        ];
        if let Err(e) = self.call_lua_function("onKeyDown", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onKeyDown for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_key_up(&mut self, entity: Entity, key: crate::platform::KeyCode) {
        let args = vec![
            Self::entity_to_value(entity),
            LuaValue::String(format!("{:?}", key)),
        ];
        if let Err(e) = self.call_lua_function("onKeyUp", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onKeyUp for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_mouse_down(&mut self, entity: Entity, button: crate::platform::MouseButton) {
        let args = vec![
            Self::entity_to_value(entity),
            LuaValue::String(format!("{:?}", button)),
        ];
        if let Err(e) = self.call_lua_function("onMouseDown", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onMouseDown for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_mouse_up(&mut self, entity: Entity, button: crate::platform::MouseButton) {
        let args = vec![
            Self::entity_to_value(entity),
            LuaValue::String(format!("{:?}", button)),
        ];
        if let Err(e) = self.call_lua_function("onMouseUp", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onMouseUp for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_pause(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_lua_function("onPause", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onPause for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_resume(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_lua_function("onResume", args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onResume for '{}': {}",
                self.script_name,
                e
            );
        }
    }
}

/// Lua生命周期钩子工厂
///
/// 提供便捷函数来创建Lua生命周期钩子
#[cfg(feature = "mlua")]
#[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
pub struct LuaLifecycleHooksFactory;

#[cfg(feature = "mlua")]
#[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
impl LuaLifecycleHooksFactory {
    /// 为Lua脚本创建生命周期钩子组件
    ///
    /// # 参数
    ///
    /// - `script_name`: 脚本名称
    /// - `script_source`: Lua脚本源代码
    /// - `entity`: ECS实体ID
    /// - `lua`: Lua引擎实例
    ///
    /// # 返回
    ///
    /// 返回实现 `LifecycleHooks` trait 的Lua钩子实例
    pub fn create_hooks(
        script_name: String,
        script_source: String,
        entity: Entity,
        lua: Arc<Mutex<Lua>>,
    ) -> Result<Box<dyn LifecycleHooks>, String> {
        // 执行脚本以注册生命周期函数
        let lua_instance = lua.lock().map_err(|e| format!("Failed to acquire Lua lock: {}", e))?;

        // 注册Engine API
        let globals = lua_instance.globals();
        let _: LuaResult<()> = globals.set("Engine", lua_instance.create_table());

        // 注册Engine.log函数
        let engine_table: mlua::Table = globals.get("Engine")?;
        engine_table.set(
            "log",
            lua_instance.create_function(|_, msg: String| {
                tracing::info!(target: "scripting", "[Lua]: {}", msg);
                Ok(())
            })?,
        )?;

        // 注册Engine.time函数
        engine_table.set(
            "time",
            lua_instance.create_function(|_, ()| -> mlua::Result<f64> {
                Ok(crate::core::utils::current_timestamp_f64())
            })?,
        )?;

        // 执行脚本源代码
        if let Err(e) = lua_instance.load(&script_source).exec() {
            return Err(format!("Failed to execute script '{}': {}", script_name, e));
        }

        drop(lua_instance);

        // 创建并返回钩子实例
        Ok(Box::new(LuaLifecycleHooks::new(script_name, lua, entity)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "mlua")]
    #[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
    use crate::ecs::Entity;
    #[cfg(feature = "mlua")]
    #[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
    use mlua::{Function, Lua, Value as LuaValueInternal};

    #[test]
    #[cfg(feature = "mlua")]
    #[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
    fn test_lua_lifecycle_hooks_creation() {
        let lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new(Lua::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        let hooks = LuaLifecycleHooks::new("test_script".to_string(), lua, entity);

        assert_eq!(hooks.script_name, "test_script");
        assert_eq!(hooks.entity_id, entity.to_string());
    }

    #[test]
    #[cfg(feature = "mlua")]
    #[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
    fn test_lua_lifecycle_hooks_enable_disable() {
        let lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new(Lua::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        let hooks = LuaLifecycleHooks::new("test_script".to_string(), lua, entity);

        // 默认启用
        assert!(*hooks.enabled.lock().unwrap());

        // 禁用
        hooks.disable();
        assert!(!*hooks.enabled.lock().unwrap());

        // 启用
        hooks.enable();
        assert!(*hooks.enabled.lock().unwrap());
    }

    #[test]
    #[cfg(feature = "mlua")]
    #[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
    fn test_lua_lifecycle_hooks_factory() {
        let lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new(Lua::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        // 创建简单的Lua脚本
        let script_source = r#"
            function onEnable(entity)
                Engine.log("Entity enabled: " .. entity)
            end

            function onUpdate(entity, deltaTime)
                Engine.log("Update: " .. deltaTime)
            end
        "#
        .to_string();

        let result = LuaLifecycleHooksFactory::create_hooks(
            "test_script".to_string(),
            script_source,
            entity,
            lua,
        );

        assert!(result.is_ok());
    }

    #[test]
    #[cfg(feature = "mlua")]
    #[allow(unexpected_cfgs, reason = "mlua is a custom feature")]
    fn test_lua_lifecycle_hooks_integration() {
        use crate::scripting::lifecycle::LifecycleHooksComponent;

        let lua: Arc<Mutex<Lua>> = Arc::new(Mutex::new(Lua::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        // 创建完整的生命周期钩子组件
        let script_source = r#"
            local updateCount = 0

            function onEnable(entity)
                Engine.log("onEnable called")
            end

            function onUpdate(entity, deltaTime)
                updateCount = updateCount + 1
                Engine.log("onUpdate called: " .. updateCount)
            end
        "#
        .to_string();

        let hooks = LuaLifecycleHooksFactory::create_hooks(
            "integration_test".to_string(),
            script_source,
            entity,
            lua.clone(),
        )
        .unwrap();

        let component = LifecycleHooksComponent::new(hooks);
        assert!(component.enabled);

        // 测试调用钩子
        component.hooks.on_enable(entity);
        component.hooks.on_update(entity, 0.016);
        component.hooks.on_update(entity, 0.016);
    }
}
