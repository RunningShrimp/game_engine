//! # C# Lifecycle Hooks
//!
//! Provides Unity-style lifecycle hooks for C# scripts.

use crate::platform::{KeyCode, MouseButton};
use crate::scripting::lifecycle::LifecycleHooks;
use crate::scripting::{ScriptContext, ScriptResult, ScriptValue};
use bevy_ecs::prelude::*;
use std::sync::{Arc, Mutex};

/// C#生命周期钩子工厂
pub struct CSharpLifecycleHooksFactory;

impl CSharpLifecycleHooksFactory {
    pub fn new() -> Self {
        Self
    }

    /// 为C#脚本创建生命周期钩子
    pub fn create_hooks(
        &self,
        context: Arc<Mutex<Box<dyn ScriptContext>>>,
        script_source: String,
    ) -> CSharpLifecycleHooks {
        CSharpLifecycleHooks {
            context,
            script_source,
        }
    }
}

impl Default for CSharpLifecycleHooksFactory {
    fn default() -> Self {
        Self::new()
    }
}

/// C#生命周期钩子实现
pub struct CSharpLifecycleHooks {
    context: Arc<Mutex<Box<dyn ScriptContext>>>,
    script_source: String,
}

impl LifecycleHooks for CSharpLifecycleHooks {
    fn on_enable(&mut self, entity: Entity) {
        let _ =
            self.call_csharp_function("OnEnable", &[ScriptValue::Number(entity.to_bits() as f64)]);
    }

    fn on_disable(&mut self, entity: Entity) {
        let _ =
            self.call_csharp_function("OnDisable", &[ScriptValue::Number(entity.to_bits() as f64)]);
    }

    fn on_destroy(&mut self, entity: Entity) {
        let _ =
            self.call_csharp_function("OnDestroy", &[ScriptValue::Number(entity.to_bits() as f64)]);
    }

    fn on_start(&mut self, entity: Entity) {
        let _ =
            self.call_csharp_function("OnStart", &[ScriptValue::Number(entity.to_bits() as f64)]);
    }

    fn on_shutdown(&mut self, entity: Entity) {
        let _ = self.call_csharp_function(
            "OnShutdown",
            &[ScriptValue::Number(entity.to_bits() as f64)],
        );
    }

    fn on_update(&mut self, entity: Entity, delta_time: f32) {
        let _ = self.call_csharp_function(
            "OnUpdate",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(delta_time as f64),
            ],
        );
    }

    fn on_fixed_update(&mut self, entity: Entity, fixed_delta_time: f32) {
        let _ = self.call_csharp_function(
            "OnFixedUpdate",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(fixed_delta_time as f64),
            ],
        );
    }

    fn on_late_update(&mut self, entity: Entity, delta_time: f32) {
        let _ = self.call_csharp_function(
            "OnLateUpdate",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(delta_time as f64),
            ],
        );
    }

    fn on_collision_enter(&mut self, entity: Entity, other: Entity) {
        let _ = self.call_csharp_function(
            "OnCollisionEnter",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(other.to_bits() as f64),
            ],
        );
    }

    fn on_collision_stay(&mut self, entity: Entity, other: Entity) {
        let _ = self.call_csharp_function(
            "OnCollisionStay",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(other.to_bits() as f64),
            ],
        );
    }

    fn on_collision_exit(&mut self, entity: Entity, other: Entity) {
        let _ = self.call_csharp_function(
            "OnCollisionExit",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(other.to_bits() as f64),
            ],
        );
    }

    fn on_trigger_enter(&mut self, entity: Entity, other: Entity) {
        let _ = self.call_csharp_function(
            "OnTriggerEnter",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(other.to_bits() as f64),
            ],
        );
    }

    fn on_trigger_stay(&mut self, entity: Entity, other: Entity) {
        let _ = self.call_csharp_function(
            "OnTriggerStay",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(other.to_bits() as f64),
            ],
        );
    }

    fn on_trigger_exit(&mut self, entity: Entity, other: Entity) {
        let _ = self.call_csharp_function(
            "OnTriggerExit",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::Number(other.to_bits() as f64),
            ],
        );
    }

    fn on_key_down(&mut self, entity: Entity, key: KeyCode) {
        let key_str = format!("{:?}", key);
        let _ = self.call_csharp_function(
            "OnKeyDown",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::String(key_str),
            ],
        );
    }

    fn on_key_up(&mut self, entity: Entity, key: KeyCode) {
        let key_str = format!("{:?}", key);
        let _ = self.call_csharp_function(
            "OnKeyUp",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::String(key_str),
            ],
        );
    }

    fn on_mouse_down(&mut self, entity: Entity, button: MouseButton) {
        let button_str = format!("{:?}", button);
        let _ = self.call_csharp_function(
            "OnMouseDown",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::String(button_str),
            ],
        );
    }

    fn on_mouse_up(&mut self, entity: Entity, button: MouseButton) {
        let button_str = format!("{:?}", button);
        let _ = self.call_csharp_function(
            "OnMouseUp",
            &[
                ScriptValue::Number(entity.to_bits() as f64),
                ScriptValue::String(button_str),
            ],
        );
    }

    fn on_pause(&mut self, entity: Entity) {
        let _ =
            self.call_csharp_function("OnPause", &[ScriptValue::Number(entity.to_bits() as f64)]);
    }

    fn on_resume(&mut self, entity: Entity) {
        let _ =
            self.call_csharp_function("OnResume", &[ScriptValue::Number(entity.to_bits() as f64)]);
    }
}

impl CSharpLifecycleHooks {
    /// 调用C#函数
    fn call_csharp_function(&mut self, function_name: &str, args: &[ScriptValue]) -> () {
        let mut context = self.context.lock().unwrap();

        // 检查函数是否存在
        if !context.has_function(function_name) {
            // 函数不存在，静默返回（可选的生命周期钩子）
            return;
        }

        // 调用函数，忽略结果
        let _ = context.call(function_name, args);
    }
}
