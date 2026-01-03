//! # Rust Plugin SDK
//!
//! Tools and utilities for developing Rust plugins.

use crate::plugin::api::{Plugin, PluginCapability, PluginConfig, PluginContext, PluginMetadata};
use crate::plugin::{export_plugin, plugin, Result};
use std::default::Default;

/// Rust plugin helper trait
pub trait RustPlugin: Plugin + Default {
    /// Get plugin metadata
    fn metadata() -> PluginMetadata
    where
        Self: Sized;

    /// Export the plugin
    fn export() where Self: Sized {
        // This would be used in build scripts or macros
    }
}

/// Macro to simplify Rust plugin creation
#[macro_export]
macro_rules! rust_plugin {
    ($struct_name:ident, $name:expr, $version:expr) => {
        #[derive(Default)]
        pub struct $struct_name;

        impl $crate::plugin::api::Plugin for $struct_name {
            fn name(&self) -> &str {
                $name
            }

            fn version(&self) -> &str {
                $version
            }

            fn on_load(&mut self, _context: $crate::plugin::api::PluginContext) -> $crate::plugin::Result<()> {
                Ok(())
            }
        }

        $crate::plugin::export_plugin!($struct_name);
    };
}

/// Macro to create a plugin with metadata
#[macro_export]
macro_rules! rust_plugin_full {
    (
        $struct_name:ident,
        $name:expr,
        $version:expr,
        description = $desc:expr,
        author = $author:expr
    ) => {
        #[derive(Default)]
        pub struct $struct_name;

        impl $crate::plugin::api::Plugin for $struct_name {
            fn name(&self) -> &str {
                $name
            }

            fn version(&self) -> &str {
                $version
            }

            fn description(&self) -> &str {
                $desc
            }

            fn author(&self) -> &str {
                $author
            }

            fn on_load(&mut self, _context: $crate::plugin::api::PluginContext) -> $crate::plugin::Result<()> {
                Ok(())
            }
        }

        $crate::plugin::export_plugin!($struct_name);
    };
}

/// Builder for creating Rust plugins
pub struct RustPluginBuilder {
    metadata: PluginMetadata,
}

impl RustPluginBuilder {
    /// Create a new plugin builder
    pub fn new(name: String, version: String) -> Self {
        Self {
            metadata: PluginMetadata::new(name, version),
        }
    }

    /// Set description
    pub fn description(mut self, description: String) -> Self {
        self.metadata.description = description;
        self
    }

    /// Set author
    pub fn author(mut self, author: String) -> Self {
        self.metadata.author = author;
        self
    }

    /// Add capability
    pub fn capability(mut self, capability: PluginCapability) -> Self {
        self.metadata.capabilities.push(capability);
        self
    }

    /// Add dependency
    pub fn dependency(mut self, dependency: String) -> Self {
        self.metadata.dependencies.push(dependency);
        self
    }

    /// Build the metadata
    pub fn build(self) -> PluginMetadata {
        self.metadata
    }
}

/// Template code for a minimal Rust plugin
pub const MINIMAL_PLUGIN_TEMPLATE: &str = r#"
use game_engine_editor::plugin::api::{Plugin, PluginContext, PluginError};

#[derive(Default)]
struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn on_load(&mut self, _context: PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        println!("Plugin loaded!");
        Ok(())
    }

    fn on_update(&mut self, _context: PluginContext, delta_time: f32) {
        // Update logic here
    }
}

// Export the plugin
game_engine_editor::plugin::export_plugin!(MyPlugin);
"#;

/// Template code for an advanced Rust plugin
pub const ADVANCED_PLUGIN_TEMPLATE: &str = r#"
use game_engine_editor::plugin::api::{
    Plugin, PluginContext, PluginCapability, PluginMetadata
};
use std::sync::{Arc, Mutex};

struct MyAdvancedPlugin {
    counter: Arc<Mutex<i32>>,
}

impl Default for MyAdvancedPlugin {
    fn default() -> Self {
        Self {
            counter: Arc::new(Mutex::new(0)),
        }
    }
}

impl Plugin for MyAdvancedPlugin {
    fn name(&self) -> &str {
        "my_advanced_plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "An advanced plugin example"
    }

    fn author(&self) -> &str {
        "Your Name"
    }

    fn capabilities(&self) -> &[PluginCapability] {
        &[PluginCapability::Render, PluginCapability::Audio]
    }

    fn on_load(&mut self, context: PluginContext) -> Result<(), Box<dyn std::error::Error>> {
        println!("Advanced plugin loaded!");
        println!("Config: {:?}", context.config);
        Ok(())
    }

    fn on_update(&mut self, context: PluginContext, delta_time: f32) {
        let mut counter = self.counter.lock().unwrap();
        *counter += 1;

        if *counter % 60 == 0 {
            println!("Plugin has been updated {} times", *counter);
        }
    }
}

game_engine_editor::plugin::export_plugin!(MyAdvancedPlugin);
"#;

/// Generate Cargo.toml for a plugin
pub fn generate_cargo_toml(name: &str, version: &str) -> String {
    format!(
        r#"
[package]
name = "{}"
version = "{}"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
game-engine-editor = "0.1"
"#,
        name, version
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_builder() {
        let metadata = RustPluginBuilder::new("test".to_string(), "0.1.0".to_string())
            .description("Test plugin".to_string())
            .author("Test Author".to_string())
            .capability(PluginCapability::Render)
            .dependency("other_plugin".to_string())
            .build();

        assert_eq!(metadata.name, "test");
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(metadata.description, "Test plugin");
        assert_eq!(metadata.author, "Test Author");
        assert_eq!(metadata.capabilities.len(), 1);
        assert_eq!(metadata.dependencies.len(), 1);
    }

    #[test]
    fn test_generate_cargo_toml() {
        let toml = generate_cargo_toml("my_plugin", "0.1.0");
        assert!(toml.contains("my_plugin"));
        assert!(toml.contains("0.1.0"));
        assert!(toml.contains("cdylib"));
    }
}
