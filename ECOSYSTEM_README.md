# Game Engine Ecosystem

Welcome to the Game Engine ecosystem! This document provides an overview of the extension and migration tools available.

## 📦 Table of Contents

- [Plugin System](#plugin-system)
- [Resource Marketplace](#resource-marketplace)
- [Unity Migration Tools](#unity-migration-tools)
- [Quick Start](#quick-start)
- [Documentation](#documentation)

---

## 🔌 Plugin System

### Overview

The plugin system allows you to extend the engine with custom functionality through dynamically loaded libraries.

### Features

- ✅ **Dynamic Loading** - Load plugins at runtime
- ✅ **Hot Reload** - Update plugins without restarting
- ✅ **Type Safety** - Rust's type system ensures safety
- ✅ **Component Registration** - Add custom ECS components
- ✅ **System Registration** - Add custom game systems
- ✅ **Event Handling** - Respond to engine events
- ✅ **Inter-Plugin Communication** - Message passing system

### Example Plugins

#### 1. Hello World Plugin
A minimal example demonstrating the plugin API.

```rust
pub struct HelloWorldPlugin {
    metadata: PluginMetadata,
    greeting_count: usize,
}

impl Plugin for HelloWorldPlugin {
    fn on_load(&mut self, context: &PluginContext) -> Result<(), Error> {
        println!("Plugin loaded!");
        Ok(())
    }

    fn on_update(&mut self, _context: &PluginContext, delta: f32) {
        // Update logic here
    }
}
```

#### 2. Physics Plugin
Adds physics simulation components and systems.

**Components:**
- Velocity
- Acceleration
- Mass
- PhysicsMaterial

**Systems:**
- PhysicsSystem (gravity, velocity updates)

#### 3. Render Plugin
Adds rendering capabilities.

**Components:**
- Mesh
- Material
- Light (Directional, Point, Spot)
- Camera
- PostProcessing

**Systems:**
- RenderSystem (draw calls, frame tracking)

### Building Plugins

```bash
# Build an example plugin
cd examples/plugins/hello_world_plugin
cargo build --release

# The compiled plugin will be in target/release/
# - Linux: libhello_world_plugin.so
# - macOS: libhello_world_plugin.dylib
# - Windows: hello_world_plugin.dll
```

### Using Plugins

```rust
use game_engine::plugins::PluginManager;

let manager = PluginManager::with_default_config();
manager.initialize()?;

// Update plugins each frame
manager.update(delta_time);
```

**See:** [Plugin Documentation](examples/plugins/README.md)

---

## 🛒 Resource Marketplace

### Overview

The marketplace provides a centralized platform for sharing and discovering game assets, plugins, and templates.

### Package Types

1. **Asset Packs** - Textures, models, audio, shaders
2. **Plugins** - Game systems, tools, extensions
3. **Templates** - Project and scene templates
4. **Scripts** - Reusable Lua scripts

### Package Format

```
example-package/
├── package.toml          # Metadata
├── README.md             # Documentation
├── LICENSE               # License file
├── assets/               # Game assets
├── scripts/              # Scripts
└── install/              # Installation hooks
```

### Metadata Example

```toml
[package]
name = "example-package"
version = "1.0.0"
type = "asset-pack"
description = "An example asset pack"

[engine]
version = ">=0.1.0"

[dependencies]
core-textures = "^1.0.0"

[assets]
textures = ["assets/textures/**/*.png"]
models = ["assets/models/**/*.gltf"]
```

### API Usage

```rust
use game_engine::marketplace::MarketplaceClient;

let client = MarketplaceClient::new(
    "https://marketplace.example.com".to_string(),
    std::path::PathBuf::from("marketplace_cache"),
);

// Search for packages
let query = SearchQuery {
    keywords: vec!["sci-fi textures".to_string()],
    ..Default::default()
};
let results = client.search(query).await?;

// Install a package
client.install_package("texture-pack", Some("1.0.0"), options).await?;

// Check for updates
let updates = client.check_updates().await?;
```

**See:** [Marketplace Design](marketplace/MARKETPLACE_DESIGN.md)

---

## 🔄 Unity Migration Tools

### Overview

Migrate your Unity projects to the game engine with automated tools.

### Supported Conversions

| Unity | Engine | Status |
|-------|--------|--------|
| Scenes (.unity) | Entity definitions | ✅ |
| Prefabs (.prefab) | Entity templates | ✅ |
| Textures (PNG/JPG) | PNG | ✅ |
| Models (FBX/OBJ) | glTF 2.0 | ✅ |
| Audio (WAV/MP3) | WAV | ✅ |
| Scripts (C#) | Lua | ⚠️ Partial |

### Migration Process

```bash
# Run migration
migrate \
  --source /path/to/unity/project \
  --output /path/to/output

# The migration will:
# 1. Parse Unity project structure
# 2. Migrate scenes to entities
# 3. Convert assets to engine formats
# 4. Convert scripts to Lua
# 5. Generate migration report
```

### API Mapping

| Unity | Engine |
|-------|--------|
| `transform.position` | `entity:get_position()` |
| `rigidbody.AddForce()` | `rigidbody:apply_force()` |
| `Debug.Log()` | `print()` |
| `Input.GetAxis()` | `input:get_axis()` |
| `Start()` | `on_start()` |
| `Update()` | `on_update()` |

### Example: C# to Lua

**Unity C#:**
```csharp
public class PlayerController : MonoBehaviour {
    public float speed = 5.0f;

    void Update() {
        float move = Input.GetAxis("Horizontal");
        transform.position += new Vector3(move * speed * Time.deltaTime, 0, 0);
    }
}
```

**Engine Lua:**
```lua
local PlayerController = {
    speed = 5.0,
}

function PlayerController.on_update(self, dt)
    local move = input:get_axis("Horizontal")
    local pos = self.entity:get_position()
    pos.x = pos.x + move * self.speed * dt
    self.entity:set_position(pos)
end

return PlayerController
```

**See:** [Unity Migration Guide](src/tools/migration/docs/UNITY_MIGRATION_GUIDE.md)

---

## 🚀 Quick Start

### Install the Engine

```bash
git clone https://github.com/example/game-engine.git
cd game-engine
cargo build --release
```

### Build a Plugin

```bash
cd examples/plugins/hello_world_plugin
cargo build --release
```

### Migrate a Unity Project

```bash
cargo run --bin migrate -- \
  --source ~/Projects/MyUnityGame \
  --output ~/Projects/MyEngineGame
```

### Create a Resource Package

1. Create package structure
2. Add `package.toml` with metadata
3. Add assets and documentation
4. Validate: `marketplace validate ./package`
5. Publish: `marketplace publish ./package`

---

## 📚 Documentation

### Core Documentation

- [Plugin System API](src/plugins/README.md) - Plugin development guide
- [Marketplace Design](marketplace/MARKETPLACE_DESIGN.md) - Marketplace architecture
- [Package Format](marketplace/PACKAGE_FORMAT.md) - Package structure specification
- [Unity Migration Guide](src/tools/migration/docs/UNITY_MIGRATION_GUIDE.md) - Migration tutorial

### Project Documentation

- [ECS Architecture](docs/P3-1_ECS_REPORT.md) - Entity-Component System
- [Rendering System](docs/P3-2_RENDER_REPORT.md) - Graphics pipeline
- [Resource Management](docs/P3-3_RESOURCE_REPORT.md) - Asset loading
- [Ecosystem Report](docs/P3-4_ECOSYSTEM_REPORT.md) - This ecosystem

### Examples

- [Hello World Plugin](examples/plugins/hello_world_plugin/) - Minimal plugin
- [Physics Plugin](examples/plugins/physics_plugin/) - Physics simulation
- [Render Plugin](examples/plugins/render_plugin/) - Rendering system

---

## 🤝 Contributing

We welcome contributions! Areas needing help:

### Plugin System
- [ ] IDE integration for plugin development
- [ ] Remote plugin loading
- [ ] Plugin sandbox improvements

### Marketplace
- [ ] Web UI implementation
- [ ] Package review system
- [ ] Automated testing

### Migration Tools
- [ ] More Unity component mappings
- [ ] Shader conversion
- [ ] Animation system migration

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Rust community for excellent tooling
- Unity for inspiring the migration tools
- Contributors and testers

---

## 📞 Support

- **Documentation:** [docs.example.com](https://docs.example.com)
- **Forum:** [forum.example.com](https://forum.example.com)
- **Discord:** [discord.gg/engine](https://discord.gg/engine)
- **Issues:** [github.com/engine/issues](https://github.com/engine/issues)

---

**Version:** 1.0.0
**Last Updated:** 2024-12-31
**Status:** ✅ Stable
