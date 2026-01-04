// JavaScript生命周期钩子实现
//
// 集成lifecycle系统与rquickjs JavaScript引擎，支持Unity风格的生命周期回调

use crate::ecs::Entity;
use crate::scripting::{
    lifecycle::{LifecycleHooks, LifecyclePhase},
    system::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue},
};
use std::sync::{Arc, Mutex};

/// JavaScript生命周期钩子
///
/// 为JavaScript脚本提供Unity风格的生命周期回调支持
pub struct JavaScriptLifecycleHooks {
    /// 脚本名称
    script_name: String,
    /// JavaScript上下文
    context: Arc<Mutex<dyn ScriptContext>>,
    /// 实体ID (用于在JavaScript中标识当前实体)
    entity_id: String,
    /// 是否已启用
    enabled: Arc<Mutex<bool>>,
}

impl JavaScriptLifecycleHooks {
    /// 创建新的JavaScript生命周期钩子
    ///
    /// # 参数
    ///
    /// - `script_name`: 脚本名称（用于日志和错误报告）
    /// - `context`: JavaScript上下文（共享引用）
    /// - `entity`: ECS实体ID
    pub fn new(
        script_name: String,
        context: Arc<Mutex<dyn ScriptContext>>,
        entity: Entity,
    ) -> Self {
        Self {
            script_name,
            context,
            entity_id: entity.to_string(),
            enabled: Arc::new(Mutex::new(true)),
        }
    }

    /// 调用JavaScript生命周期函数
    ///
    /// # 参数
    ///
    /// - `function_name`: 要调用的JavaScript函数名称
    /// - `args`: 传递给JavaScript函数的参数
    ///
    /// # 返回
    ///
    /// 如果调用成功返回 `Ok(())`，否则返回错误信息
    fn call_js_function(&self, function_name: &str, args: &[ScriptValue]) -> Result<(), String> {
        // 检查是否启用
        let enabled = self
            .enabled
            .lock()
            .map_err(|e| format!("Failed to acquire enabled lock: {}", e))?;
        if !*enabled {
            return Ok(());
        }
        drop(enabled);

        // 检查函数是否存在
        let has_function = self
            .context
            .lock()
            .map_err(|e| format!("Failed to acquire context lock: {}", e))?
            .has_function(function_name);

        if !has_function {
            // 函数不存在不是错误，只是静默跳过
            return Ok(());
        }

        // 设置当前实体ID（作为全局变量供JavaScript访问）
        let _ = self
            .context
            .lock()
            .map_err(|e| format!("Failed to acquire context lock: {}", e))?
            .set_global(
                "__current_entity_id",
                ScriptValue::String(self.entity_id.clone()),
            );

        // 调用JavaScript函数
        let result = self
            .context
            .lock()
            .map_err(|e| format!("Failed to acquire context lock: {}", e))?
            .call(function_name, args);

        match result {
            ScriptResult::Success(_) | ScriptResult::Void => Ok(()),
            ScriptResult::Error(e) => {
                tracing::error!(
                    target: "scripting",
                    "JavaScript lifecycle error in '{}' ({}): {}",
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

    /// 将Entity转换为ScriptValue
    fn entity_to_value(entity: Entity) -> ScriptValue {
        ScriptValue::Integer(entity.to_bits() as i64)
    }

    /// 将两个Entity转换为参数数组
    fn entities_to_args(entity: Entity, other: Entity) -> Vec<ScriptValue> {
        vec![Self::entity_to_value(entity), Self::entity_to_value(other)]
    }
}

impl LifecycleHooks for JavaScriptLifecycleHooks {
    fn on_enable(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_js_function("onEnable", &args) {
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
        if let Err(e) = self.call_js_function("onDisable", &args) {
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
        if let Err(e) = self.call_js_function("onDestroy", &args) {
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
            ScriptValue::Number(delta_time as f64),
        ];
        if let Err(e) = self.call_js_function("onUpdate", &args) {
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
            ScriptValue::Number(fixed_delta_time as f64),
        ];
        if let Err(e) = self.call_js_function("onFixedUpdate", &args) {
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
            ScriptValue::Number(delta_time as f64),
        ];
        if let Err(e) = self.call_js_function("onLateUpdate", &args) {
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
        if let Err(e) = self.call_js_function("onCollisionEnter", &args) {
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
        if let Err(e) = self.call_js_function("onCollisionStay", &args) {
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
        if let Err(e) = self.call_js_function("onCollisionExit", &args) {
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
        if let Err(e) = self.call_js_function("onTriggerEnter", &args) {
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
        if let Err(e) = self.call_js_function("onTriggerStay", &args) {
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
        if let Err(e) = self.call_js_function("onTriggerExit", &args) {
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
        if let Err(e) = self.call_js_function("onStart", &args) {
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
        if let Err(e) = self.call_js_function("onShutdown", &args) {
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
            ScriptValue::String(format!("{:?}", key)),
        ];
        if let Err(e) = self.call_js_function("onKeyDown", &args) {
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
            ScriptValue::String(format!("{:?}", key)),
        ];
        if let Err(e) = self.call_js_function("onKeyUp", &args) {
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
            ScriptValue::String(format!("{:?}", button)),
        ];
        if let Err(e) = self.call_js_function("onMouseDown", &args) {
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
            ScriptValue::String(format!("{:?}", button)),
        ];
        if let Err(e) = self.call_js_function("onMouseUp", &args) {
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
        if let Err(e) = self.call_js_function("onPause", &args) {
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
        if let Err(e) = self.call_js_function("onResume", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call onResume for '{}': {}",
                self.script_name,
                e
            );
        }
    }
}

/// JavaScript生命周期钩子工厂
///
/// 提供便捷函数来创建JavaScript生命周期钩子
pub struct JavaScriptLifecycleHooksFactory;

impl JavaScriptLifecycleHooksFactory {
    /// 为JavaScript脚本创建生命周期钩子组件
    ///
    /// # 参数
    ///
    /// - `script_name`: 脚本名称
    /// - `script_source`: JavaScript脚本源代码
    /// - `entity`: ECS实体ID
    /// - `context`: JavaScript上下文
    ///
    /// # 返回
    ///
    /// 返回实现 `LifecycleHooks` trait 的JavaScript钩子实例
    pub fn create_hooks(
        script_name: String,
        script_source: String,
        entity: Entity,
        context: Arc<Mutex<dyn ScriptContext>>,
    ) -> Result<Box<dyn LifecycleHooks>, String> {
        // 执行脚本以注册生命周期函数
        let mut ctx =
            context.lock().map_err(|e| format!("Failed to acquire context lock: {}", e))?;

        // 执行脚本源代码
        match ctx.execute(&script_source, Some(&script_name)) {
            ScriptResult::Error(e) => {
                return Err(format!("Failed to execute script '{}': {}", script_name, e));
            }
            _ => {}
        }

        drop(ctx);

        // 创建并返回钩子实例
        Ok(Box::new(JavaScriptLifecycleHooks::new(
            script_name,
            context,
            entity,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::system::JavaScriptContext;

    #[test]
    fn test_javascript_lifecycle_hooks_creation() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(JavaScriptContext::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        let hooks = JavaScriptLifecycleHooks::new("test_script".to_string(), context, entity);

        assert_eq!(hooks.script_name, "test_script");
        assert_eq!(hooks.entity_id, entity.to_string());
    }

    #[test]
    fn test_javascript_lifecycle_hooks_enable_disable() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(JavaScriptContext::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        let hooks = JavaScriptLifecycleHooks::new("test_script".to_string(), context, entity);

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
    fn test_javascript_lifecycle_hooks_factory() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(JavaScriptContext::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        // 创建简单的JavaScript脚本
        let script_source = r#"
            function onEnable(entity) {
                Engine.log("Entity enabled: " + entity);
            }

            function onUpdate(entity, deltaTime) {
                Engine.log("Update: " + deltaTime);
            }
        "#
        .to_string();

        let result = JavaScriptLifecycleHooksFactory::create_hooks(
            "test_script".to_string(),
            script_source,
            entity,
            context,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_javascript_lifecycle_hooks_integration() {
        use crate::scripting::lifecycle::LifecycleHooksComponent;

        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(JavaScriptContext::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        // 创建完整的生命周期钩子组件
        let script_source = r#"
            let updateCount = 0;

            function onEnable(entity) {
                Engine.log("onEnable called");
            }

            function onUpdate(entity, deltaTime) {
                updateCount++;
                Engine.log("onUpdate called: " + updateCount);
            }
        "#
        .to_string();

        let hooks = JavaScriptLifecycleHooksFactory::create_hooks(
            "integration_test".to_string(),
            script_source,
            entity,
            context.clone(),
        )
        .unwrap();

        let mut component = LifecycleHooksComponent::new(hooks);
        assert!(component.enabled);

        // 测试调用钩子
        component.hooks.on_enable(entity);
        component.hooks.on_update(entity, 0.016);
        component.hooks.on_update(entity, 0.016);
    }
}
