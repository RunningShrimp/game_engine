# Plugin Development Guide

Complete guide for creating and publishing plugins for the game engine marketplace.

## Quick Start

### 1. Create Plugin Structure

```bash
my-plugin/
├── plugin.json           # Plugin manifest
├── src/                  # Source code
│   ├── lib.rs
│   └── ...
├── assets/               # Assets
│   ├── textures/
│   ├── models/
│   └── sounds/
├── README.md             # Documentation
├── LICENSE               # License file
└── scripts/              # Optional scripts
    ├── post_install.sh
    └── pre_uninstall.sh
```

### 2. Create Plugin Manifest

`plugin.json`:

```json
{
  "name": "my-plugin",
  "display_name": "My Awesome Plugin",
  "version": "1.0.0",
  "description": "A brief description of what your plugin does",
  "entry_point": "src/lib.rs",
  "permissions": [
    "network",
    "filesystem"
  ],
  "resources": [
    {
      "path": "assets/*",
      "resource_type": "static",
      "description": "Plugin assets"
    }
  ],
  "commands": [
    {
      "id": "my-plugin.open",
      "title": "Open My Plugin",
      "category": "Tools",
      "icon": "icon.png",
      "keybinding": "Ctrl+Shift+M"
    }
  ],
  "settings": [
    {
      "key": "api_key",
      "title": "API Key",
      "description": "Enter your API key",
      "setting_type": "string",
      "default_value": "",
      "options": []
    },
    {
      "key": "enable_feature",
      "title": "Enable Feature",
      "description": "Enable the advanced feature",
      "setting_type": "boolean",
      "default_value": true,
      "options": []
    },
    {
      "key": "quality",
      "title": "Quality Level",
      "description": "Select quality level",
      "setting_type": "enum",
      "default_value": "high",
      "options": [
        {"label": "Low", "value": "low"},
        {"label": "Medium", "value": "medium"},
        {"label": "High", "value": "high"}
      ]
    }
  ],
  "compatibility": {
    "engine_version_min": "1.0.0",
    "engine_version_max": null,
    "platforms": ["windows", "macos", "linux"],
    "features": ["vulkan", "mesh_shading"]
  }
}
```

### 3. Implement Plugin

`src/lib.rs`:

```rust
use game_engine::plugin::*;

pub struct MyPlugin {
    settings: PluginSettings,
}

impl Plugin for MyPlugin {
    fn new(settings: PluginSettings) -> Result<Self, PluginError> {
        Ok(Self { settings })
    }

    fn on_load(&mut self) -> Result<(), PluginError> {
        // Initialize plugin
        println!("MyPlugin loaded!");
        Ok(())
    }

    fn on_unload(&mut self) -> Result<(), PluginError> {
        // Cleanup
        println!("MyPlugin unloaded!");
        Ok(())
    }

    fn on_update(&mut self, delta_time: f32) -> Result<(), PluginError> {
        // Per-frame update
        Ok(())
    }
}

// Export plugin
game_engine_export_plugin!(MyPlugin);
```

### 4. Build and Package

```bash
# Build plugin
cargo build --release

# Create package
plugin-cli package ./my-plugin

# Output: my-plugin-1.0.0.tar.gz
```

### 5. Publish to Marketplace

```bash
# Login first
plugin-cli login

# Publish plugin
plugin-cli publish ./my-plugin

# Or publish as draft
plugin-cli publish ./my-plugin --draft

# Update existing plugin
plugin-cli publish ./my-plugin --update
```

## Plugin API Reference

### Lifecycle Methods

```rust
pub trait Plugin {
    /// Called when plugin is loaded
    fn on_load(&mut self) -> Result<(), PluginError>;

    /// Called when plugin is unloaded
    fn on_unload(&mut self) -> Result<(), PluginError>;

    /// Called every frame
    fn on_update(&mut self, delta_time: f32) -> Result<(), PluginError>;

    /// Called when a command is executed
    fn on_command(&mut self, command: &str, args: &[String]) -> Result<(), PluginError>;

    /// Called when settings are changed
    fn on_setting_changed(&mut self, key: &str, value: &Value) -> Result<(), PluginError>;
}
```

### Engine API Access

```rust
// Access engine systems
use game_engine::core::*;

struct MyPlugin {
    world: WorldHandle,
    renderer: RendererHandle,
    audio: AudioHandle,
}

impl MyPlugin {
    fn spawn_entity(&self) {
        // Spawn an entity
        let entity = self.world.spawn();

        // Add components
        self.world.add_component(entity, Transform::default());
        self.world.add_component(entity, Mesh::from_file("model.gltf"));
    }

    fn play_sound(&self) {
        self.audio.play("assets/sound.wav");
    }

    fn register_material(&self) {
        self.renderer.register_material("my_material", custom_shader());
    }
}
```

### Event System

```rust
// Subscribe to events
impl Plugin for MyPlugin {
    fn on_load(&mut self) -> Result<(), PluginError> {
        // Subscribe to entity spawn events
        self.subscribe(EntitySpawned::EVENT_ID, Self::on_entity_spawned);
        Ok(())
    }
}

impl MyPlugin {
    fn on_entity_spawned(&mut self, event: &EntitySpawned) {
        println!("Entity spawned: {:?}", event.entity);
    }
}
```

### UI Integration

```rust
// Add custom UI panels
impl Plugin for MyPlugin {
    fn on_load(&mut self) -> Result<(), PluginError> {
        // Register custom panel
        self.register_panel(Panel {
            title: "My Plugin Panel",
            id: "my-plugin-panel",
            render: Box::new(|ui| {
                ui.label("Hello from plugin!");
                if ui.button("Click me") {
                    println!("Button clicked!");
                }
            }),
        });

        Ok(())
    }
}
```

## Plugin Permissions

### Available Permissions

- **`network`**: Allow network requests
- **`filesystem:read`**: Read files from disk
- **`filesystem:write`**: Write files to disk
- **`gpu`**: Access GPU directly
- **`audio`**: Play/record audio
- **`input`**: Capture input events
- **`window`**: Create windows

### Requesting Permissions

```json
{
  "permissions": [
    "network",
    "filesystem:read",
    "gpu"
  ]
}
```

### Using Permissions

```rust
// Check permission
if self.has_permission("network")? {
    // Make network request
    let response = reqwest::get("https://api.example.com").await?;
}

// Request file path
let path = self.request_file_path()?;
let content = std::fs::read_to_string(path)?;
```

## Plugin Settings

### Defining Settings

Settings are defined in `plugin.json` and accessed at runtime:

```rust
// Get setting value
let api_key: String = self.get_setting("api_key")?;
let enable_feature: bool = self.get_setting("enable_feature")?;
let quality: String = self.get_setting("quality")?;

// Set setting value
self.set_setting("api_key", "new-api-key")?;

// Watch for changes
impl Plugin for MyPlugin {
    fn on_setting_changed(&mut self, key: &str, value: &Value) -> Result<(), PluginError> {
        match key {
            "api_key" => {
                let new_key = value.as_str().ok_or(PluginError::InvalidValue)?;
                self.update_api_key(new_key)?;
            }
            _ => {}
        }
        Ok(())
    }
}
```

## Plugin Resources

### Loading Resources

```rust
// Load from plugin directory
let texture = self.load_resource("assets/textures/texture.png")?;
let model = self.load_resource("assets/models/model.gltf")?;
let sound = self.load_resource("assets/sounds/sound.wav")?;

// Load from engine resources
let engine_texture = self.load_engine_resource("engine:textures/default.png")?;
```

### Resource Types

- Textures: PNG, JPG, HDR, EXR
- Models: GLTF, GLB, FBX, OBJ
- Sounds: WAV, MP3, OGG
- Shaders: WGSL, SPIRV
- Scripts: Lua, Python (if enabled)

## Plugin Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_initialization() {
        let settings = PluginSettings::default();
        let plugin = MyPlugin::new(settings).unwrap();
        assert!(plugin.is_loaded());
    }

    #[test]
    fn test_command_handling() {
        let mut plugin = MyPlugin::new(PluginSettings::default()).unwrap();
        plugin.on_command("test", &[]).unwrap();
    }
}
```

### Integration Tests

```bash
# Create test workspace
mkdir test_workspace
cd test_workspace

# Install plugin
plugin-cli install ../my-plugin-1.0.0.tar.gz

# Run tests
plugin-cli test my-plugin
```

## Best Practices

### 1. Error Handling

```rust
// Always return detailed errors
pub fn my_function() -> Result<(), PluginError> {
    let data = load_data()
        .map_err(|e| PluginError::IoError(format!("Failed to load data: {}", e)))?;

    Ok(())
}
```

### 2. Resource Management

```rust
// Clean up resources
impl Drop for MyPlugin {
    fn drop(&mut self) {
        // Release resources
        self.cleanup();
    }
}
```

### 3. Performance

```rust
// Cache expensive operations
impl MyPlugin {
    fn get_cached_data(&mut self) -> &Data {
        if self.cache.is_none() {
            self.cache = Some(expensive_computation());
        }
        self.cache.as_ref().unwrap()
    }
}
```

### 4. Thread Safety

```rust
// Use thread-safe containers
use std::sync::{Arc, Mutex};

pub struct MyPlugin {
    data: Arc<Mutex<Vec<Item>>>,
}
```

### 5. Logging

```rust
// Use engine logging
log::info!("Plugin initialized");
log::warn!("Deprecated feature used");
log::error!("Operation failed: {}", error);
```

## Publishing Checklist

Before publishing your plugin:

- [ ] Plugin compiles without errors
- [ ] All tests pass
- [ ] Documentation is complete
- [ ] README includes installation and usage instructions
- [ ] License file is included
- [ ] Plugin manifest is valid
- [ ] Screenshots are provided (min 3, max 10)
- [ ] Chelog is up to date
- [ ] Version follows semantic versioning
- [ ] Compatibility requirements are accurate
- [ ] Settings have descriptions
- [ ] Commands have clear titles
- [ ] Permissions are minimal and justified

## Versioning

Use semantic versioning:

- **MAJOR**: Breaking changes
- **MINOR**: New features, backwards compatible
- **PATCH**: Bug fixes, backwards compatible

```
1.0.0 → 1.0.1 (bug fix)
1.0.1 → 1.1.0 (new feature)
1.1.0 → 2.0.0 (breaking change)
```

## Monetization

### Paid Plugins

Set up pricing in manifest:

```json
{
  "pricing": {
    "pricing_type": "paid",
    "price": 29.99,
    "currency": "USD",
    "trial_available": true,
    "trial_days": 14
  }
}
```

### Subscription Plugins

```json
{
  "pricing": {
    "pricing_type": "subscription",
    "subscription": {
      "monthly": 9.99,
      "yearly": 99.99,
      "currency": "USD"
    }
  }
}
```

### License Key Validation

```rust
impl Plugin for MyPlugin {
    fn on_load(&mut self) -> Result<(), PluginError> {
        // Validate license
        let license_key = self.get_license_key()?;
        if !self.validate_license(&license_key)? {
            return Err(PluginError::LicenseInvalid);
        }

        Ok(())
    }
}
```

## Support and Updates

### Providing Support

Include contact information in README:

```markdown
# Support

- Email: support@example.com
- Discord: https://discord.gg/xxxxx
- Issues: https://github.com/user/my-plugin/issues
```

### Handling Updates

```rust
// Notify users of updates
impl Plugin for MyPlugin {
    fn on_load(&mut self) -> Result<(), PluginError> {
        if self.has_update()? {
            self.notify("A new version is available!");
        }
        Ok(())
    }
}
```

## Example Plugins

See the `examples/` directory for complete plugin examples:

- `simple-plugin/`: Minimal plugin template
- `custom-renderer/`: Custom rendering pipeline
- `physics-extension/`: Custom physics integration
- `ai-behavior/`: AI behavior tree plugin
- `networking/`: Multiplayer networking plugin

## Additional Resources

- [Plugin API Reference](./API_REFERENCE.md)
- [Marketplace Guidelines](./MARKETPLACE_GUIDELINES.md)
- [Community Forum](https://forum.gameengine.com)
- [Developer Discord](https://discord.gg/gameengine)

## Troubleshooting

### Common Issues

**Plugin not loading**
- Check manifest syntax is valid
- Verify engine version compatibility
- Check console logs for errors

**Permission denied**
- Add required permissions to manifest
- Request permissions at runtime

**Crash on load**
- Check for null pointer dereferences
- Verify resource paths are correct
- Enable debug mode for detailed logs

For more help, visit the community forums or open an issue on GitHub.
