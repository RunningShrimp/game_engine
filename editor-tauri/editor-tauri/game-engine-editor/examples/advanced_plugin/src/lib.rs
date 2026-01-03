//! Advanced Plugin Example
//!
//! This is an advanced example demonstrating various plugin capabilities.
//!
//! Features demonstrated:
//! - Plugin lifecycle management
//! - Event subscription
//! - Configuration handling
//! - Frame updates
//! - Error handling
//! - Statistics tracking

use game_engine_editor::plugin::api::{
    Plugin, PluginCapability, PluginConfig, PluginContext, PluginEvent,
};
use game_engine_editor::plugin::export_plugin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Advanced plugin with state management
pub struct AdvancedPlugin {
    /// Frame counter
    frame_count: Arc<AtomicU64>,

    /// Event counter
    event_count: Arc<AtomicU64>,

    /// Load time
    load_time: Option<chrono::DateTime<chrono::Utc>>,

    /// Configuration
    config: Option<PluginConfig>,
}

impl Default for AdvancedPlugin {
    fn default() -> Self {
        Self {
            frame_count: Arc::new(AtomicU64::new(0)),
            event_count: Arc::new(AtomicU64::new(0)),
            load_time: None,
            config: None,
        }
    }
}

impl Plugin for AdvancedPlugin {
    fn name(&self) -> &str {
        "advanced_plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn description(&self) -> &str {
        "An advanced plugin demonstrating full plugin API capabilities"
    }

    fn author(&self) -> &str {
        "Game Engine Team"
    }

    fn api_version(&self) -> &str {
        "0.1.0"
    }

    fn dependencies(&self) -> &[&str] {
        &[]
    }

    fn capabilities(&self) -> &[PluginCapability] {
        &[
            PluginCapability::Render,
            PluginCapability::Audio,
            PluginCapability::FileSystem,
        ]
    }

    fn on_load(&mut self, context: PluginContext) -> game_engine_editor::plugin::Result<()> {
        self.load_time = Some(chrono::Utc::now());
        self.config = Some(context.config.clone());

        println!("╔════════════════════════════════════════════╗");
        println!("║  Advanced Plugin Loaded                    ║");
        println!("╠════════════════════════════════════════════╣");
        println!("║  Name: {:<34} ║", self.name());
        println!("║  Version: {:<31} ║", self.version());
        println!("║  Author: {:<32} ║", self.author());
        println!("║  Description: {:<28} ║", self.description());
        println!("║  API Version: {:<29} ║", self.api_version());
        println!("║  Capabilities: {:<28} ║", "Render, Audio, FileSystem");
        println!("╠════════════════════════════════════════════╣");
        println!("║  Engine Version: {:<28} ║", context.engine_api.version());
        println!("║  Enabled: {:<34} ║", context.config.enabled);
        println!("║  Auto Load: {:<32} ║", context.config.auto_load);
        println!("║  Hot Reload: {:<31} ║", context.config.hot_reload);
        println!("╚════════════════════════════════════════════╝");

        // Demonstrate configuration access
        if let Ok(custom_value) = context.config.get::<String>("custom_setting") {
            println!("\nCustom setting: {}", custom_value);
        }

        Ok(())
    }

    fn on_update(&mut self, _context: PluginContext, delta_time: f32) {
        let frame = self.frame_count.fetch_add(1, Ordering::Relaxed);

        // Print statistics every 60 frames (approximately once per second at 60 FPS)
        if frame % 60 == 0 {
            let fps = 1.0 / delta_time;
            println!(
                "📊 [Advanced Plugin] Frame: {:>6} | FPS: {:>6.1} | Events: {:>4}",
                frame,
                fps,
                self.event_count.load(Ordering::Relaxed)
            );
        }
    }

    fn on_event(&mut self, event: &PluginEvent) {
        self.event_count.fetch_add(1, Ordering::Relaxed);

        match event {
            PluginEvent::PluginLoaded { name } => {
                println!("🔌 Plugin loaded: {}", name);
            }
            PluginEvent::PluginUnloaded { name } => {
                println!("🔌 Plugin unloaded: {}", name);
            }
            PluginEvent::SceneLoaded { path } => {
                println!("📁 Scene loaded: {}", path);
            }
            PluginEvent::SceneSaved { path } => {
                println!("💾 Scene saved: {}", path);
            }
            PluginEvent::AssetImported { path } => {
                println!("🎨 Asset imported: {}", path);
            }
            PluginEvent::Tick { delta_time } => {
                // Silent tick events (don't log these to avoid spam)
                let _ = delta_time;
            }
            PluginEvent::PluginError { name, error } => {
                println!("❌ Plugin error in {}: {}", name, error);
            }
            PluginEvent::Custom { type_, data } => {
                println!("📨 Custom event: {} | Data: {}", type_, data);
            }
        }
    }

    fn on_unload(&mut self, _context: PluginContext) -> game_engine_editor::plugin::Result<()> {
        let total_frames = self.frame_count.load(Ordering::Relaxed);
        let total_events = self.event_count.load(Ordering::Relaxed);
        let uptime = if let Some(load_time) = self.load_time {
            let duration = chrono::Utc::now() - load_time;
            duration.num_seconds()
        } else {
            0
        };

        println!("╔════════════════════════════════════════════╗");
        println!("║  Advanced Plugin Unloaded                  ║");
        println!("╠════════════════════════════════════════════╣");
        println!("║  Total Frames: {:<29} ║", total_frames);
        println!("║  Total Events: {:<29} ║", total_events);
        println!("║  Uptime: {:<34} ║", format!("{}s", uptime));
        if uptime > 0 {
            let avg_fps = (total_frames as f64) / (uptime as f64);
            println!("║  Avg FPS: {:<33} ║", format!("{:.1}", avg_fps));
        }
        println!("╚════════════════════════════════════════════╝");

        Ok(())
    }

    fn config(&self) -> Option<PluginConfig> {
        self.config.clone()
    }
}

// Export the plugin for dynamic loading
export_plugin!(AdvancedPlugin);
