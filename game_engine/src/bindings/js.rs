// JavaScript Binding Adapter using rquickjs
//
// This adapter provides JavaScript scripting support using QuickJS.
//
// Design Note: Due to QuickJS not being Send/Sync, this adapter uses
// a single-threaded design with a command queue pattern for thread-safe
// interaction with the engine.
//
use super::protocol::{
    BindingAdapter, BindingCommand, BindingEvent, BindingResult,
};
use rquickjs::{Context, Function, Object, Runtime};
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

// SAFETY: JsBindingAdapter is NOT Send + Sync
//
// QuickJS Runtime and Context are NOT thread-safe and cannot be safely shared across threads.
// The rquickjs library explicitly does not implement Send/Sync for these types.
//
// Attempting to make this type Send/Sync is INCORRECT and UNSAFE because:
// - Runtime/Context contain raw pointers and internal state that is not protected by mutexes
// - QuickJS was designed for single-threaded use only
// - Cross-thread access would cause data races and undefined behavior
//
// Correct approach for multi-threading:
// - Keep JsBindingAdapter on a single thread (e.g., the main thread or a dedicated JS thread)
// - Use channels (mpsc/oneshot) for thread-safe communication
// - The existing Arc<Mutex<CommandQueue>> is a good pattern for this
//
// DO NOT add unsafe impl Send/Sync here. Instead, use a thread-local design or message passing.
//
// See: https://docs.rs/rquickjs/latest/rquickjs/ (note lack of Send/Sync)

impl Default for JsBindingAdapter {
    fn default() -> Self {
        let runtime = match Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                tracing::error!("Failed to create JS runtime: {}", e);
                panic!("Critical: Cannot initialize JavaScript runtime: {}", e);
            }
        };
        let context = match Context::full(&runtime) {
            Ok(ctx) => ctx,
            Err(e) => {
                tracing::error!("Failed to create JS context: {}", e);
                panic!("Critical: Cannot initialize JavaScript context: {}", e);
            }
        };
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

    fn bind_engine_api_internal(&mut self) {
        self.context.with(|ctx| {
            // Create 'Engine' namespace object
            let engine_obj = match Object::new(ctx.clone()) {
                Ok(obj) => obj,
                Err(e) => {
                    tracing::error!("Failed to create Engine object: {}", e);
                    return;
                }
            };

            // Engine.log(msg)
            if let Err(e) = engine_obj.set(
                "log",
                Function::new(ctx.clone(), |msg: String| {
                    println!("[JS]: {}", msg);
                }),
            ) {
                tracing::error!("Failed to set Engine.log: {}", e);
                return;
            }

            // Store engine reference globally
            if let Err(e) = ctx.globals().set("Engine", engine_obj) {
                tracing::error!("Failed to set Engine global: {}", e);
            }
        });
    }

    fn execute_command_internal(&mut self, cmd: BindingCommand) -> BindingResult {
        // Queue command for processing
        let mut queue = match self.command_queue.lock() {
            Ok(q) => q,
            Err(e) => {
                tracing::error!("Failed to acquire command queue lock: {}", e);
                return BindingResult::Error {
                    message: "Command queue lock failed".to_string(),
                };
            }
        };
        queue.push(cmd);
        BindingResult::Success {
            data: Some("Command queued".to_string()),
        }
    }

    fn dispatch_event_internal(&mut self, event: BindingEvent) {
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
            if let Ok(engine_obj) = ctx.globals().get::<_, Object>("Engine") {
                let _ = engine_obj.set("__onEngineEvent", event_handler_wrapper);
            }
        });
    }

    fn poll_commands_internal(&mut self) -> Vec<BindingCommand> {
        // Drain queued commands for engine to process
        if let Ok(mut queue) = self.command_queue.lock() {
            queue.drain()
        } else {
            Vec::new()
        }
    }

    fn shutdown_internal(&mut self) {
        // QuickJS cleanup - runtime is automatically cleaned up on Drop
    }
}

impl BindingAdapter for JsBindingAdapter {
    fn init(&mut self) {
        // Note: In a single-threaded design, init() just sets up the binding
        // Actual script API binding happens when engine calls bind_engine_api()
    }

    fn bind_engine_api(&mut self) {
        self.bind_engine_api_internal();
    }

    fn execute_command(&mut self, cmd: BindingCommand) -> BindingResult {
        self.execute_command_internal(cmd)
    }

    fn dispatch_event(&mut self, event: BindingEvent) {
        self.dispatch_event_internal(event);
    }

    fn poll_commands(&mut self) -> Vec<BindingCommand> {
        self.poll_commands_internal()
    }

    fn shutdown(&mut self) {
        self.shutdown_internal();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_binding_init() {
        let mut adapter = JsBindingAdapter::new();
        adapter.init();

        // Test basic command execution
        let result = adapter.execute_command(BindingCommand::PlaySound {
            sound_id: 1u64,
            volume: 1.0,
            pitch: 1.0,
        });
        assert!(matches!(result, BindingResult::Success { .. }));
    }

    #[test]
    fn test_js_command_queue() {
        let mut adapter = JsBindingAdapter::new();
        adapter.init();

        // Queue a command
        let _ = adapter.execute_command(BindingCommand::PlaySound {
            sound_id: 1u64,
            volume: 1.0,
            pitch: 1.0,
        });

        // Poll commands
        let commands = adapter.poll_commands();
        assert_eq!(commands.len(), 1);

        match &commands[0] {
            BindingCommand::PlaySound { sound_id, .. } => {
                assert_eq!(*sound_id, 1u64);
            }
            _ => panic!("Expected PlaySound command"),
        }
    }
}
