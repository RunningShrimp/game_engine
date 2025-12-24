// JavaScript Binding Adapter using rquickjs
//
// This adapter provides JavaScript scripting support using QuickJS.
//
// Design Note: Due to QuickJS not being Send/Sync, this adapter uses
// a single-threaded design with a command queue pattern for thread-safe
// interaction with the engine.
//
use super::protocol::{
    BindingAdapter, BindingCommand, BindingError, BindingEvent, BindingResponse, BindingResult,
    ComponentData,
};
use rquickjs::{Context, Function, Object, Runtime, Value};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Shared command queue for JS -> Engine communication
#[derive(Default)]
pub struct CommandQueue {
    commands: VecDeque<BindingCommand>,
}

impl CommandQueue {
    /// 推送命令到队列末尾
    pub fn push(&mut self, cmd: BindingCommand) {
        self.commands.push_back(cmd);
    }

    /// 清空队列并返回所有命令
    pub fn drain(&mut self) -> Vec<BindingCommand> {
        self.commands.drain(..).collect()
    }
}

/// JavaScript 绑定适配器
///
/// 使用 QuickJS 引擎提供 JavaScript 脚本支持
///
/// # Thread Safety
///
/// 使用单线程设计，通过命令队列实现线程安全
pub struct JsBindingAdapter {
    /// QuickJS运行时实例
    #[allow(dead_code)]
    runtime: Runtime,
    /// QuickJS上下文
    #[allow(dead_code)]
    context: Context,
    /// 命令队列，用于线程间通信
    command_queue: Arc<Mutex<CommandQueue>>,
}

impl Default for JsBindingAdapter {
    fn default() -> Self {
        let runtime = Runtime::new().expect("Failed to create JS runtime");
        let context = Context::full(&runtime).expect("Failed to create JS context");
        let command_queue = Arc::new(Mutex::new(CommandQueue::default()));

        Self {
            runtime,
            context,
            command_queue,
        }
    }
}

impl JsBindingAdapter {
    pub fn new() -> Self {
        Self::default()
    }

    fn init(&mut self) {
        // Note: In a single-threaded design, init() just sets up the binding
        // Actual script API binding happens when engine calls bind_engine_api()
    }

    fn bind_engine_api(&mut self) {
        let queue = Arc::clone(&self.command_queue);

        self.context.with(|ctx| {
            let global = ctx.globals();

            // Create 'Engine' namespace object
            let engine_obj = Object::new(ctx.clone()).unwrap();

            // Engine.log(msg)
            engine_obj
                .set(
                    "log",
                    Function::new(ctx.clone(), |msg: String| {
                        println!("[JS]: {}", msg);
                    }),
                )
                .unwrap();

            // Store engine reference globally
            ctx.globals().set("Engine", engine_obj).unwrap();
        });
    }

    fn execute_command(&mut self, cmd: BindingCommand) -> BindingResult {
        // Queue command for processing
        let mut queue = self.command_queue.lock().unwrap();
        queue.push(cmd);
        BindingResult::Success {
            data: Some("Command queued".to_string()),
        }
    }

    fn dispatch_event(&mut self, event: BindingEvent) {
        // Store event in globals for script polling
        self.context.with(|ctx| {
            let event_json = serde_json::to_string(&event).unwrap_or_default();

            // Define event handler wrapper
            let event_handler_wrapper = format!(
                r#"(function(event) {{
                    try {{
                        if (typeof __onEngineEvent === 'function') {{
                            __onEngineEvent(event);
                        }}
                    }} catch (e) {{
                        console.error('Event handler error:', e);
                    }}
                }})({})"#,
                event_json
            );

            // Store handler
            if let Ok(engine_obj) = ctx.globals().get("Engine") {
                engine_obj
                    .set(
                        "__onEngineEvent",
                        Function::new(ctx.clone(), event_handler_wrapper).unwrap(),
                    )
                    .unwrap();
            }
        });
    }

    fn poll_commands(&mut self) -> Vec<BindingCommand> {
        // Drain queued commands for engine to process
        if let Ok(mut queue) = self.command_queue.lock() {
            queue.drain()
        } else {
            Vec::new()
        }
    }

    fn shutdown(&mut self) {
        // QuickJS cleanup - runtime is automatically cleaned up on Drop
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_binding_init() {
        let mut adapter = JsBindingAdapter::new();
        adapter.init();

        // Test basic script execution
        let result = adapter.execute_js_script("Engine.log('Hello from test!');");
        assert!(result.is_ok());
    }

    #[test]
    fn test_js_command_queue() {
        let mut adapter = JsBindingAdapter::new();
        adapter.init();

        // Queue a command
        let _ = adapter.execute_command(BindingCommand::PlaySound {
            sound_id: 1,
            volume: 1.0,
            pitch: 1.0,
        });

        // Poll commands
        let commands = adapter.poll_commands();
        assert_eq!(commands.len(), 1);

        match &commands[0] {
            BindingCommand::PlaySound { sound_id, .. } => {
                assert_eq!(sound_id, 1);
            }
            _ => panic!("Expected PlaySound command"),
        }
    }
}
