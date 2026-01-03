//! {{plugin-name}}
//!
//! {{description}}

use game_engine_editor::plugin::api::{Plugin, PluginContext};
use game_engine_editor::plugin::export_plugin;

#[derive(Default)]
pub struct {{PluginStruct}};

impl Plugin for {{PluginStruct}} {
    fn name(&self) -> &str {
        "{{plugin-name}}"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "{{description}}"
    }

    fn author(&self) -> &str {
        "{{author}}"
    }

    fn on_load(&mut self, context: PluginContext) -> game_engine_editor::plugin::Result<()> {
        println!("✓ Plugin '{}' loaded!", self.name());
        println!("  Engine version: {}", context.engine_api.version());

        Ok(())
    }

    fn on_update(&mut self, _context: PluginContext, delta_time: f32) {
        // Called every frame
        // TODO: Implement update logic
        let _ = delta_time;
    }

    fn on_unload(&mut self, _context: PluginContext) -> game_engine_editor::plugin::Result<()> {
        println!("✓ Plugin '{}' unloaded!", self.name());
        Ok(())
    }
}

// Export the plugin for dynamic loading
export_plugin!({{PluginStruct}});
