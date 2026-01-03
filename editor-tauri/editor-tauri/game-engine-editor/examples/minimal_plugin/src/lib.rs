//! Minimal Plugin Example
//!
//! This is a minimal example of a plugin for the game engine editor.
//!
//! To build:
//! ```bash
//! cargo build --release
//! ```
//!
//! The compiled plugin will be at:
//! - macOS: target/release/libminimal_plugin.dylib
//! - Linux: target/release/libminimal_plugin.so
//! - Windows: target/release/minimal_plugin.dll

use game_engine_editor::plugin::api::{Plugin, PluginContext};
use game_engine_editor::plugin::export_plugin;

#[derive(Default)]
pub struct MinimalPlugin;

impl Plugin for MinimalPlugin {
    fn name(&self) -> &str {
        "minimal_plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "A minimal example plugin"
    }

    fn author(&self) -> &str {
        "Game Engine Team"
    }

    fn on_load(&mut self, context: PluginContext) -> game_engine_editor::plugin::Result<()> {
        println!("✓ Minimal plugin loaded!");
        println!("  Engine version: {}", context.engine_api.version());
        println!("  Plugin config: {:?}", context.config);

        Ok(())
    }

    fn on_update(&mut self, _context: PluginContext, delta_time: f32) {
        // This is called every frame
        // For this minimal example, we don't do anything here
        let _ = delta_time;
    }

    fn on_unload(&mut self, _context: PluginContext) -> game_engine_editor::plugin::Result<()> {
        println!("✓ Minimal plugin unloaded!");
        Ok(())
    }
}

// Export the plugin for dynamic loading
export_plugin!(MinimalPlugin);
