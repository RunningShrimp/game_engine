# Example Plugins

This directory contains example plugins demonstrating the game engine's plugin system.

## Plugins

### 1. Hello World Plugin (`hello_world_plugin`)

A minimal example that shows:
- Basic plugin structure
- Lifecycle hooks (on_load, on_unload, on_update)
- Event handling
- Logging

**Features:**
- Prints greetings periodically
- Responds to engine events
- Demonstrates plugin metadata

**Build:**
```bash
cd hello_world_plugin
cargo build --release
```

### 2. Physics Plugin (`physics_plugin`)

A physics simulation plugin that demonstrates:
- Custom component registration
- System registration
- Physics calculations

**Features:**
- Velocity component
- Acceleration component
- Mass component
- Physics material component
- Physics system with gravity

**Build:**
```bash
cd physics_plugin
cargo build --release
```

### 3. Render Plugin (`render_plugin`)

A rendering plugin that demonstrates:
- Complex component definitions
- Multiple systems
- Renderer customization

**Features:**
- Mesh component
- Material component
- Light component (directional, point, spot)
- Camera component
- Post-processing component
- Render system with draw call tracking

**Build:**
```bash
cd render_plugin
cargo build --release
```

## Using Plugins

After building, plugins will be compiled as dynamic libraries:
- Linux: `.so` files
- macOS: `.dylib` files
- Windows: `.dll` files

These can be loaded by the engine's plugin loader:

```rust
use game_engine::plugins::PluginManager;

let manager = PluginManager::with_default_config();
manager.initialize()?;
```

## Plugin Structure

All plugins must:
1. Implement the `Plugin` trait
2. Export a `create_plugin` function
3. Be compiled as a `cdylib`

Example:
```rust
use game_engine::plugins::api::Plugin;
use std::any::Any;

pub struct MyPlugin {
    metadata: PluginMetadata,
}

impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn Plugin> {
    Box::new(MyPlugin::new())
}
```

## Testing

Each plugin includes comprehensive test coverage:

```bash
cargo test
```

## Next Steps

- Create your own plugin by copying one of the examples
- Register custom components for your game logic
- Implement systems for game mechanics
- Handle engine events for scene changes
- Use hot-reload for rapid development
