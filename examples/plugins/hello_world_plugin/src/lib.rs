//! Hello World Plugin
//!
//! A simple example plugin that demonstrates basic plugin functionality.

use game_engine::plugins::api::*;
use game_engine::ecs::{ComponentRegistry, SystemScheduler};
use game_engine::error::Error;
use std::any::Any;

/// Hello World Plugin
pub struct HelloWorldPlugin {
    metadata: PluginMetadata,
    greeting_count: usize,
}

impl HelloWorldPlugin {
    /// Create a new Hello World plugin
    pub fn new() -> Self {
        let metadata = PluginMetadata::new("hello_world".to_string(), "0.1.0".to_string())
            .with_author("Game Engine Team".to_string())
            .with_description("A simple Hello World plugin".to_string())
            .with_website("https://example.com".to_string())
            .with_license("MIT".to_string());

        Self {
            metadata,
            greeting_count: 0,
        }
    }
}

impl Default for HelloWorldPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for HelloWorldPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn on_load(&mut self, context: &PluginContext) -> Result<(), Error> {
        println!("[HelloWorldPlugin] Loaded successfully!");
        println!("[HelloWorldPlugin] Engine version: {}", context.engine_version());
        println!("[HelloWorldPlugin] Data directory: {}", context.data_dir().display());
        println!("[HelloWorldPlugin] Config directory: {}", context.config_dir().display());
        println!("[HelloWorldPlugin] Hot-reload enabled: {}", context.is_hot_reload());

        Ok(())
    }

    fn on_unload(&mut self, _context: &PluginContext) -> Result<(), Error> {
        println!("[HelloWorldPlugin] Unloaded. Total greetings: {}", self.greeting_count);
        Ok(())
    }

    fn on_update(&mut self, _context: &PluginContext, delta: f32) {
        // Print a greeting every second (approximately)
        self.greeting_count += 1;

        if self.greeting_count % 60 == 0 {
            println!("[HelloWorldPlugin] Hello, World! Delta: {:.3}s", delta);
        }
    }

    fn on_event(&mut self, _context: &PluginContext, event: &PluginEvent) {
        match event {
            PluginEvent::EngineInit => {
                println!("[HelloWorldPlugin] Engine initialized!");
            }
            PluginEvent::EngineShutdown => {
                println!("[HelloWorldPlugin] Engine shutting down!");
            }
            PluginEvent::SceneLoading(name) => {
                println!("[HelloWorldPlugin] Scene loading: {}", name);
            }
            PluginEvent::SceneLoaded(name) => {
                println!("[HelloWorldPlugin] Scene loaded: {}", name);
            }
            _ => {}
        }
    }

    fn register_components(&self, registry: &mut ComponentRegistry) {
        println!("[HelloWorldPlugin] Registering custom components...");
        // Components would be registered here
    }

    fn register_systems(&self, scheduler: &mut SystemScheduler) {
        println!("[HelloWorldPlugin] Registering custom systems...");
        // Systems would be registered here
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Plugin creation function (exported for dynamic loading)
#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(HelloWorldPlugin::new())
}
