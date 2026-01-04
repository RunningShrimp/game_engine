// TypeScript生命周期钩子实现
//
// 集成lifecycle系统与TypeScript/QuickJS引擎，支持类型安全的生命周期回调

use crate::ecs::Entity;
use crate::scripting::{
    javascript_lifecycle::JavaScriptLifecycleHooks,
    lifecycle::{LifecycleHooks, LifecyclePhase},
    system::{ScriptContext, ScriptLanguage, ScriptResult, ScriptValue},
};
use std::sync::{Arc, Mutex};

/// TypeScript生命周期钩子
///
/// 为TypeScript脚本提供Unity风格的生命周期回调支持
/// TypeScript使用QuickJS引擎（通过rquickjs），与JavaScript共享相同的运行时
pub struct TypeScriptLifecycleHooks {
    /// 内部JavaScript钩子实现
    inner: JavaScriptLifecycleHooks,
}

impl TypeScriptLifecycleHooks {
    /// 创建新的TypeScript生命周期钩子
    ///
    /// # 参数
    ///
    /// - `script_name`: 脚本名称（用于日志和错误报告）
    /// - `context`: TypeScript/JavaScript上下文（共享引用）
    /// - `entity`: ECS实体ID
    pub fn new(
        script_name: String,
        context: Arc<Mutex<dyn ScriptContext>>,
        entity: Entity,
    ) -> Self {
        Self {
            inner: JavaScriptLifecycleHooks::new(script_name, context, entity),
        }
    }

    /// 启用钩子
    pub fn enable(&self) {
        self.inner.enable();
    }

    /// 禁用钩子
    pub fn disable(&self) {
        self.inner.disable();
    }
}

impl LifecycleHooks for TypeScriptLifecycleHooks {
    fn on_enable(&mut self, entity: Entity) {
        self.inner.on_enable(entity);
    }

    fn on_disable(&mut self, entity: Entity) {
        self.inner.on_disable(entity);
    }

    fn on_destroy(&mut self, entity: Entity) {
        self.inner.on_destroy(entity);
    }

    fn on_update(&mut self, entity: Entity, delta_time: f32) {
        self.inner.on_update(entity, delta_time);
    }

    fn on_fixed_update(&mut self, entity: Entity, fixed_delta_time: f32) {
        self.inner.on_fixed_update(entity, fixed_delta_time);
    }

    fn on_late_update(&mut self, entity: Entity, delta_time: f32) {
        self.inner.on_late_update(entity, delta_time);
    }

    fn on_collision_enter(&mut self, entity: Entity, other: Entity) {
        self.inner.on_collision_enter(entity, other);
    }

    fn on_collision_stay(&mut self, entity: Entity, other: Entity) {
        self.inner.on_collision_stay(entity, other);
    }

    fn on_collision_exit(&mut self, entity: Entity, other: Entity) {
        self.inner.on_collision_exit(entity, other);
    }

    fn on_trigger_enter(&mut self, entity: Entity, other: Entity) {
        self.inner.on_trigger_enter(entity, other);
    }

    fn on_trigger_stay(&mut self, entity: Entity, other: Entity) {
        self.inner.on_trigger_stay(entity, other);
    }

    fn on_trigger_exit(&mut self, entity: Entity, other: Entity) {
        self.inner.on_trigger_exit(entity, other);
    }

    fn on_start(&mut self, entity: Entity) {
        self.inner.on_start(entity);
    }

    fn on_shutdown(&mut self, entity: Entity) {
        self.inner.on_shutdown(entity);
    }

    fn on_key_down(&mut self, entity: Entity, key: crate::platform::KeyCode) {
        self.inner.on_key_down(entity, key);
    }

    fn on_key_up(&mut self, entity: Entity, key: crate::platform::KeyCode) {
        self.inner.on_key_up(entity, key);
    }

    fn on_mouse_down(&mut self, entity: Entity, button: crate::platform::MouseButton) {
        self.inner.on_mouse_down(entity, button);
    }

    fn on_mouse_up(&mut self, entity: Entity, button: crate::platform::MouseButton) {
        self.inner.on_mouse_up(entity, button);
    }

    fn on_pause(&mut self, entity: Entity) {
        self.inner.on_pause(entity);
    }

    fn on_resume(&mut self, entity: Entity) {
        self.inner.on_resume(entity);
    }
}

/// TypeScript生命周期钩子工厂
///
/// 提供便捷函数来创建TypeScript生命周期钩子
pub struct TypeScriptLifecycleHooksFactory;

impl TypeScriptLifecycleHooksFactory {
    /// 为TypeScript脚本创建生命周期钩子组件
    ///
    /// # 参数
    ///
    /// - `script_name`: 脚本名称
    /// - `script_source`: TypeScript脚本源代码
    /// - `entity`: ECS实体ID
    /// - `context`: TypeScript上下文
    ///
    /// # 返回
    ///
    /// 返回实现 `LifecycleHooks` trait 的TypeScript钩子实例
    pub fn create_hooks(
        script_name: String,
        script_source: String,
        entity: Entity,
        context: Arc<Mutex<dyn ScriptContext>>,
    ) -> Result<Box<dyn LifecycleHooks>, String> {
        // TypeScript代码会被编译为JavaScript，然后执行
        // 这里我们直接执行TypeScript源码（QuickJS会处理）
        let mut ctx =
            context.lock().map_err(|e| format!("Failed to acquire context lock: {}", e))?;

        // 执行脚本源代码
        match ctx.execute(&script_source, Some(&script_name)) {
            ScriptResult::Error(e) => {
                return Err(format!(
                    "Failed to execute TypeScript script '{}': {}",
                    script_name, e
                ));
            }
            _ => {}
        }

        drop(ctx);

        // 创建并返回钩子实例（包装JavaScript实现）
        Ok(Box::new(TypeScriptLifecycleHooks::new(
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
    fn test_typescript_lifecycle_hooks_creation() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(JavaScriptContext::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        let hooks = TypeScriptLifecycleHooks::new("test_script".to_string(), context, entity);

        // TypeScript hooks应该正确包装JavaScript hooks
        assert_eq!(hooks.inner.script_name, "test_script");
        assert_eq!(hooks.inner.entity_id, entity.to_string());
    }

    #[test]
    fn test_typescript_lifecycle_hooks_enable_disable() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(JavaScriptContext::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        let hooks = TypeScriptLifecycleHooks::new("test_script".to_string(), context, entity);

        // 默认启用
        assert!(*hooks.inner.enabled.lock().unwrap());

        // 禁用
        hooks.disable();
        assert!(!*hooks.inner.enabled.lock().unwrap());

        // 启用
        hooks.enable();
        assert!(*hooks.inner.enabled.lock().unwrap());
    }

    #[test]
    fn test_typescript_lifecycle_hooks_factory() {
        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(JavaScriptContext::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        // 创建简单的TypeScript脚本
        let script_source = r#"
            // TypeScript类型注解示例
            interface Entity {
                id: number;
                active: boolean;
            }

            function onEnable(entity: number): void {
                Engine.log("Entity enabled: " + entity);
            }

            function onUpdate(entity: number, deltaTime: number): void {
                Engine.log("Update: " + deltaTime);
            }
        "#
        .to_string();

        let result = TypeScriptLifecycleHooksFactory::create_hooks(
            "test_script".to_string(),
            script_source,
            entity,
            context,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_typescript_lifecycle_hooks_integration() {
        use crate::scripting::lifecycle::LifecycleHooksComponent;

        let context: Arc<Mutex<dyn ScriptContext>> = Arc::new(Mutex::new(JavaScriptContext::new()));
        let entity = Entity::from_raw_u32(1).unwrap();

        // 创建完整的生命周期钩子组件
        let script_source = r#"
            // TypeScript示例：带类型的组件
            class PlayerController {
                private updateCount: number = 0;

                onEnable(): void {
                    Engine.log("PlayerController enabled");
                }

                onUpdate(entity: number, deltaTime: number): void {
                    this.updateCount++;
                    Engine.log("onUpdate called: " + this.updateCount);
                }
            }

            const controller = new PlayerController();

            // 导出到全局作用域
            function onEnable(entity: number): void {
                controller.onEnable();
            }

            function onUpdate(entity: number, deltaTime: number): void {
                controller.onUpdate(entity, deltaTime);
            }
        "#
        .to_string();

        let hooks = TypeScriptLifecycleHooksFactory::create_hooks(
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
