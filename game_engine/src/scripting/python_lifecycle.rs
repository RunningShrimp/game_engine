// Python生命周期钩子实现
//
// 集成lifecycle系统与Python引擎（pyo3），支持Unity风格的生命周期回调

use crate::ecs::Entity;
use crate::scripting::{
    lifecycle::{LifecycleHooks, LifecyclePhase},
    system::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue},
};
use std::sync::{Arc, Mutex};

/// Python生命周期钩子
///
/// 为Python脚本提供Unity风格的生命周期回调支持
pub struct PythonLifecycleHooks {
    /// 脚本名称
    script_name: String,
    /// Python上下文
    context: Arc<Mutex<dyn ScriptContext>>,
    /// 实体ID (用于在Python中标识当前实体)
    entity_id: String,
    /// 是否已启用
    enabled: Arc<Mutex<bool>>,
}

impl PythonLifecycleHooks {
    /// 创建新的Python生命周期钩子
    ///
    /// # 参数
    ///
    /// - `script_name`: 脚本名称（用于日志和错误报告）
    /// - `context`: Python上下文（共享引用）
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

    /// 调用Python生命周期函数
    ///
    /// # 参数
    ///
    /// - `function_name`: 要调用的Python函数名称
    /// - `args`: 传递给Python函数的参数
    ///
    /// # 返回
    ///
    /// 如果调用成功返回 `Ok(())`，否则返回错误信息
    fn call_python_function(
        &self,
        function_name: &str,
        args: &[ScriptValue],
    ) -> Result<(), String> {
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
        let mut ctx = self
            .context
            .lock()
            .map_err(|e| format!("Failed to acquire context lock: {}", e))?;

        // Python使用callable()检查函数是否存在
        let has_function = match ctx.eval(&format!("callable({})", function_name)) {
            ScriptResult::Success(ScriptValue::Boolean(true)) => true,
            _ => false,
        };

        if !has_function {
            // 函数不存在不是错误，只是静默跳过
            return Ok(());
        }

        // 设置当前实体ID（作为全局变量供Python访问）
        let _ = ctx.set_global(
            "__current_entity_id",
            ScriptValue::String(self.entity_id.clone()),
        );

        // 调用Python函数
        let result = ctx.call(function_name, args);

        match result {
            ScriptResult::Success(_) | ScriptResult::Void => Ok(()),
            ScriptResult::Error(e) => {
                tracing::error!(
                    target: "scripting",
                    "Python lifecycle error in '{}' ({}): {}",
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

impl LifecycleHooks for PythonLifecycleHooks {
    fn on_enable(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_python_function("on_enable", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_enable for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_disable(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_python_function("on_disable", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_disable for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_destroy(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_python_function("on_destroy", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_destroy for '{}': {}",
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
        if let Err(e) = self.call_python_function("on_update", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_update for '{}': {}",
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
        if let Err(e) = self.call_python_function("on_fixed_update", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_fixed_update for '{}': {}",
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
        if let Err(e) = self.call_python_function("on_late_update", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_late_update for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_collision_enter(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_python_function("on_collision_enter", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_collision_enter for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_collision_stay(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_python_function("on_collision_stay", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_collision_stay for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_collision_exit(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_python_function("on_collision_exit", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_collision_exit for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_trigger_enter(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_python_function("on_trigger_enter", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_trigger_enter for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_trigger_stay(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_python_function("on_trigger_stay", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_trigger_stay for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_trigger_exit(&mut self, entity: Entity, other: Entity) {
        let args = Self::entities_to_args(entity, other);
        if let Err(e) = self.call_python_function("on_trigger_exit", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_trigger_exit for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_start(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_python_function("on_start", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_start for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_shutdown(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_python_function("on_shutdown", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_shutdown for '{}': {}",
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
        if let Err(e) = self.call_python_function("on_key_down", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_key_down for '{}': {}",
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
        if let Err(e) = self.call_python_function("on_key_up", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_key_up for '{}': {}",
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
        if let Err(e) = self.call_python_function("on_mouse_down", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_mouse_down for '{}': {}",
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
        if let Err(e) = self.call_python_function("on_mouse_up", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_mouse_up for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_pause(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_python_function("on_pause", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_pause for '{}': {}",
                self.script_name,
                e
            );
        }
    }

    fn on_resume(&mut self, entity: Entity) {
        let args = vec![Self::entity_to_value(entity)];
        if let Err(e) = self.call_python_function("on_resume", &args) {
            tracing::warn!(
                target: "scripting",
                "Failed to call on_resume for '{}': {}",
                self.script_name,
                e
            );
        }
    }
}

/// Python生命周期钩子工厂
///
/// 提供便捷函数来创建Python生命周期钩子
pub struct PythonLifecycleHooksFactory;

impl PythonLifecycleHooksFactory {
    /// 为Python脚本创建生命周期钩子组件
    ///
    /// # 参数
    ///
    /// - `script_name`: 脚本名称
    /// - `script_source`: Python脚本源代码
    /// - `entity`: ECS实体ID
    /// - `context`: Python上下文
    ///
    /// # 返回
    ///
    /// 返回实现 `LifecycleHooks` trait 的Python钩子实例
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
                return Err(format!(
                    "Failed to execute Python script '{}': {}",
                    script_name, e
                ));
            }
            _ => {}
        }

        drop(ctx);

        // 创建并返回钩子实例
        Ok(Box::new(PythonLifecycleHooks::new(
            script_name,
            context,
            entity,
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::system::PythonContext;

    #[test]
    fn test_python_lifecycle_hooks_creation() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(PythonContext::new()));
        let entity = Entity::from_raw(1);

        let hooks = PythonLifecycleHooks::new("test_script".to_string(), context, entity);

        assert_eq!(hooks.script_name, "test_script");
        assert_eq!(hooks.entity_id, entity.to_string());
    }

    #[test]
    fn test_python_lifecycle_hooks_enable_disable() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(PythonContext::new()));
        let entity = Entity::from_raw(1);

        let hooks = PythonLifecycleHooks::new("test_script".to_string(), context, entity);

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
    fn test_python_lifecycle_hooks_factory() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(PythonContext::new()));
        let entity = Entity::from_raw(1);

        // 创建简单的Python脚本
        let script_source = r#"
def on_enable(entity):
    print(f"Entity enabled: {entity}")

def on_update(entity, delta_time):
    print(f"Update: {delta_time}")
        "#
        .to_string();

        let result = PythonLifecycleHooksFactory::create_hooks(
            "test_script".to_string(),
            script_source,
            entity,
            context,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_python_lifecycle_hooks_integration() {
        use crate::scripting::lifecycle::LifecycleHooksComponent;

        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(PythonContext::new()));
        let entity = Entity::from_raw(1);

        // 创建完整的生命周期钩子组件
        let script_source = r#"
update_count = 0

def on_enable(entity):
    print("on_enable called")

def on_update(entity, delta_time):
    global update_count
    update_count += 1
    print(f"on_update called: {update_count}")
        "#
        .to_string();

        let hooks = PythonLifecycleHooksFactory::create_hooks(
            "integration_test".to_string(),
            script_source,
            entity,
            context.clone(),
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
