//! JavaScript Binding Adapter using rquickjs
//! 
//! This adapter provides JavaScript scripting support using QuickJS.

use rquickjs::{Context, Runtime, Function, Object, Value};
use super::protocol::{BindingAdapter, BindingCommand, BindingEvent, BindingResult, ComponentData};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

/// Shared command queue for JS -> Engine communication
#[derive(Default)]
pub struct CommandQueue {
    commands: VecDeque<BindingCommand>,
}

impl CommandQueue {
    pub fn push(&mut self, cmd: BindingCommand) {
        self.commands.push_back(cmd);
    }
    
    pub fn drain(&mut self) -> Vec<BindingCommand> {
        self.commands.drain(..).collect()
    }
}

pub struct JsBindingAdapter {
    #[allow(dead_code)]
    runtime: Runtime,
    context: Context,
    command_queue: Arc<Mutex<CommandQueue>>,
}

impl Default for JsBindingAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl JsBindingAdapter {
    pub fn new() -> Self {
        let runtime = Runtime::new().expect("Failed to create JS runtime");
        let context = Context::full(&runtime).expect("Failed to create JS context");
        let command_queue = Arc::new(Mutex::new(CommandQueue::default()));
        
        Self {
            runtime,
            context,
            command_queue,
        }
    }
    
    fn bind_engine_api(&self) {
        let queue = Arc::clone(&self.command_queue);
        
        self.context.with(|ctx| {
            let global = ctx.globals();
            
            // Create 'Engine' namespace
            let engine_obj = Object::new(ctx.clone()).unwrap();
            
            // Engine.log(msg)
            engine_obj.set("log", Function::new(ctx.clone(), |msg: String| {
                println!("[JS]: {}", msg);
            })).unwrap();
            
            // Engine.spawn(components_json) -> entity_id
            let q = Arc::clone(&queue);
            engine_obj.set("spawn", Function::new(ctx.clone(), move |json: String| -> u64 {
                if let Ok(components) = serde_json::from_str::<Vec<ComponentData>>(&json) {
                    q.lock().unwrap().push(BindingCommand::SpawnEntity { components });
                }
                0 // Actual entity ID will be returned via event
            })).unwrap();
            
            // Engine.despawn(entity_id)
            let q = Arc::clone(&queue);
            engine_obj.set("despawn", Function::new(ctx.clone(), move |entity_id: u64| {
                q.lock().unwrap().push(BindingCommand::DespawnEntity { entity_id });
            })).unwrap();
            
            // Engine.setPosition(entity_id, x, y, z)
            let q = Arc::clone(&queue);
            engine_obj.set("setPosition", Function::new(ctx.clone(), move |entity_id: u64, x: f32, y: f32, z: f32| {
                q.lock().unwrap().push(BindingCommand::SetPosition { entity_id, x, y, z });
            })).unwrap();
            
            // Engine.playSound(name, path, volume, looped)
            let q = Arc::clone(&queue);
            engine_obj.set("playSound", Function::new(ctx.clone(), move |name: String, path: String, volume: f32, looped: bool| {
                q.lock().unwrap().push(BindingCommand::PlaySound { name, path, volume, looped });
            })).unwrap();
            
            // Engine.stopSound(name)
            let q = Arc::clone(&queue);
            engine_obj.set("stopSound", Function::new(ctx.clone(), move |name: String| {
                q.lock().unwrap().push(BindingCommand::StopSound { name });
            })).unwrap();
            
            global.set("Engine", engine_obj).unwrap();
            
            // Create 'Input' namespace
            let input_obj = Object::new(ctx.clone()).unwrap();
            
            // Input.isKeyPressed(key) - placeholder, actual impl needs input state
            input_obj.set("isKeyPressed", Function::new(ctx.clone(), |_key: String| -> bool {
                false // TODO: Connect to actual input system
            })).unwrap();
            
            global.set("Input", input_obj).unwrap();
            
            // Create 'console' object for compatibility
            let console_obj = Object::new(ctx.clone()).unwrap();
            console_obj.set("log", Function::new(ctx.clone(), |msg: String| {
                println!("[JS console]: {}", msg);
            })).unwrap();
            global.set("console", console_obj).unwrap();
        });
    }
    
    pub fn execute_script(&self, code: &str) -> Result<(), String> {
        self.context.with(|ctx| {
            ctx.eval::<(), _>(code).map_err(|e| format!("{:?}", e))
        })
    }
    
    pub fn call_function(&self, name: &str, args_json: &str) -> Result<String, String> {
        self.context.with(|ctx| {
            let global = ctx.globals();
            if let Ok(func) = global.get::<_, Function>(name) {
                // Parse args from JSON and call function
                // For simplicity, we pass the JSON string directly
                match func.call::<_, Value>((args_json,)) {
                    Ok(result) => {
                        // Try to serialize result back to JSON
                        Ok(format!("{:?}", result))
                    }
                    Err(e) => Err(format!("{:?}", e))
                }
            } else {
                Err(format!("Function '{}' not found", name))
            }
        })
    }
}

impl BindingAdapter for JsBindingAdapter {
    fn init(&mut self) {
        self.bind_engine_api();
    }
    
    fn execute_command(&mut self, cmd: BindingCommand) -> BindingResult {
        // This is for engine -> script direction (less common for commands)
        // Usually we poll_commands for script -> engine
        match cmd {
            BindingCommand::Custom { name, data } => {
                // Execute custom JS callback
                let code = format!("if (typeof {} === 'function') {}('{}');", name, name, data);
                match self.execute_script(&code) {
                    Ok(_) => BindingResult::Success,
                    Err(e) => BindingResult::Error(e),
                }
            }
            _ => BindingResult::Error("Command not supported in JS->Engine direction".to_string())
        }
    }
    
    fn dispatch_event(&mut self, event: BindingEvent) {
        let event_json = serde_json::to_string(&event).unwrap_or_default();
        let code = format!(
            "if (typeof __onEngineEvent === 'function') __onEngineEvent({});",
            event_json
        );
        let _ = self.execute_script(&code);
    }
    
    fn poll_commands(&mut self) -> Vec<BindingCommand> {
        self.command_queue.lock().unwrap().drain()
    }
    
    fn shutdown(&mut self) {
        // QuickJS cleanup is automatic via Drop
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
        let result = adapter.execute_script("Engine.log('Hello from test!');");
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_js_command_queue() {
        let mut adapter = JsBindingAdapter::new();
        adapter.init();
        
        // Execute script that queues a command
        let _ = adapter.execute_script("Engine.playSound('test', 'test.mp3', 1.0, false);");
        
        // Poll commands
        let commands = adapter.poll_commands();
        assert_eq!(commands.len(), 1);
        
        match &commands[0] {
            BindingCommand::PlaySound { name, .. } => {
                assert_eq!(name, "test");
            }
            _ => panic!("Expected PlaySound command"),
        }
    }
}
