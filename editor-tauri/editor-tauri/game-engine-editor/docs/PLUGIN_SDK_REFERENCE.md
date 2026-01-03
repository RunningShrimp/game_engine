# Plugin SDK Reference

Complete API reference for developing plugins for the Game Engine Editor.

## Table of Contents

1. [Rust SDK](#rust-sdk)
2. [WASM SDK](#wasm-sdk)
3. [TypeScript SDK](#typescript-sdk)
4. [Lua SDK](#lua-sdk)
5. [Common Patterns](#common-patterns)
6. [API Reference](#api-reference)

## Rust SDK

### Installation

Add to `Cargo.toml`:

```toml
[dependencies]
game-engine-editor = "0.1"
```

### Basic Plugin

```rust
use game_engine_editor::plugin::api::{Plugin, PluginContext};
use game_engine_editor::plugin::export_plugin;

#[derive(Default)]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn name(&self) -> &str {
        "my_plugin"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn on_load(&mut self, context: PluginContext) -> game_engine_editor::plugin::Result<()> {
        println!("Plugin loaded!");
        Ok(())
    }
}

export_plugin!(MyPlugin);
```

### Advanced Features

#### Custom State

```rust
pub struct MyPlugin {
    counter: i32,
    data: HashMap<String, String>,
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self {
            counter: 0,
            data: HashMap::new(),
        }
    }
}
```

#### Event Handling

```rust
fn on_event(&mut self, event: &PluginEvent) {
    match event {
        PluginEvent::SceneLoaded { path } => {
            println!("Scene loaded: {}", path);
        }
        PluginEvent::Tick { delta_time } => {
            // Handle frame updates
        }
        _ => {}
    }
}
```

#### Configuration

```rust
fn on_load(&mut self, context: PluginContext) -> Result<()> {
    // Get configuration value
    if let Ok(setting) = context.config.get::<String>("my_setting") {
        println!("Setting: {}", setting);
    }

    // Set configuration value
    context.config.set("key".to_string(), "value")?;

    Ok(())
}
```

#### Dependencies

```rust
fn dependencies(&self) -> &[&str] {
    &["required_plugin_1", "required_plugin_2"]
}
```

#### Capabilities

```rust
fn capabilities(&self) -> &[PluginCapability] {
    &[
        PluginCapability::Render,
        PluginCapability::Audio,
        PluginCapability::FileSystem,
    ]
}
```

### Macros

#### `plugin!` macro

```rust
plugin!(MyPlugin, "my_plugin", "0.1.0");
```

#### `export_plugin!` macro

```rust
export_plugin!(MyPlugin);
```

### Error Handling

```rust
use game_engine_editor::plugin::PluginError;

fn on_load(&mut self, context: PluginContext) -> Result<()> {
    // Return error
    Err(PluginError::Other("Something went wrong".to_string()))?;

    Ok(())
}
```

## WASM SDK

### Installation

```toml
[dependencies]
wasm-bindgen = "0.2"
```

### Basic Plugin

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn name() -> String {
    "my_wasm_plugin".to_string()
}

#[wasm_bindgen]
pub fn version() -> String {
    "0.1.0".to_string()
}

#[wasm_bindgen]
pub fn on_load() -> i32 {
    // Plugin initialization
    0 // Success
}

#[wasm_bindgen]
pub fn on_update(delta_time: f32) {
    // Update logic
}

#[wasm_bindgen]
pub fn on_unload() -> i32 {
    // Cleanup
    0 // Success
}
```

### Building

```bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/my_plugin.wasm --out-dir ./dist
```

### WASM-Specific Considerations

- No direct file system access
- Limited memory (64KB pages)
- No threading
- Different error handling
- Requires WASM-compatible dependencies

## TypeScript SDK

### Installation

```bash
npm install --save-dev @types/game-engine-editor
```

### Basic Plugin

```typescript
const plugin = {
    name: "my-plugin",
    version: "0.1.0",
    apiVersion: "0.1.0",

    async onLoad(context: PluginContext) {
        console.log("Plugin loaded!");
    },

    onUpdate(context: PluginContext, deltaTime: number) {
        // Update logic
    },

    onUnload(context: PluginContext) {
        console.log("Plugin unloaded!");
    }
};

registerPlugin(plugin);
```

### Type Definitions

```typescript
interface PluginContext {
    engineApi: EngineApi;
    resourceManager: ResourceManager;
    config: PluginConfig;
}

interface EngineApi {
    getVersion(): string;
    getActiveScene(): Scene;
    registerComponent(definition: ComponentDefinition): void;
    addEventListener(event: string, handler: (event: PluginEvent) => void): void;
}

interface PluginEvent {
    type: string;
    data?: any;
    timestamp?: number;
}

interface Scene {
    readonly name: string;
    readonly path: string;
    getRootNodes(): Node[];
    findNodeByName(name: string): Node | null;
    traverse(callback: (node: Node) => void): void;
}
```

### Advanced TypeScript Plugin

```typescript
interface MyPluginState {
    counter: number;
    lastUpdate: number;
}

const plugin: Plugin = {
    name: "my-advanced-plugin",
    version: "0.1.0",
    description: "Advanced TypeScript plugin",
    author: "Your Name",

    state: {
        counter: 0,
        lastUpdate: Date.now()
    } as MyPluginState,

    async onLoad(context: PluginContext) {
        // Subscribe to events
        context.engineApi.addEventListener("scene.load", (event) => {
            console.log("Scene loaded:", event.data);
        });

        // Access resources
        const assets = await context.resourceManager.listAssets();
        console.log("Assets:", assets.length);
    },

    onUpdate(context: PluginContext, deltaTime: number) {
        this.state.counter++;

        if (this.state.counter % 60 === 0) {
            console.log(`Updated ${this.state.counter} times`);
        }
    },

    onEvent(event: PluginEvent) {
        console.log("Event:", event.type, event.data);
    }
};

registerPlugin(plugin);
```

### Building

```bash
npm run build
```

tsconfig.json:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "outDir": "./dist",
    "strict": true,
    "esModuleInterop": true
  }
}
```

## Lua SDK

### Basic Plugin

```lua
local plugin = {
    name = "my_lua_plugin",
    version = "0.1.0",
    apiVersion = "0.1.0"
}

function plugin:on_load(context)
    print("Plugin loaded!")
end

function plugin:on_update(context, delta_time)
    -- Update logic
end

function plugin:on_unload(context)
    print("Plugin unloaded!")
end

return plugin
```

### Lua API

```lua
-- Engine API
engine_api:get_version()
engine_api:get_active_scene()
engine_api:register_component(definition)
engine_api:add_event_listener(event_name, handler)

-- Resource Manager
resource_manager:load_asset(path)
resource_manager:save_asset(path, data)
resource_manager:list_assets()

-- Utility functions
utils.log(message)
utils.warn(message)
utils.error(message)
utils.format(fmt, ...)
```

### Advanced Lua Plugin

```lua
local plugin = {
    name = "my_advanced_lua_plugin",
    version = "0.1.0",
    description = "Advanced Lua plugin",

    state = {
        counter = 0,
        last_update = 0
    }
}

function plugin:on_load(context)
    print("Advanced plugin loaded!")

    -- Subscribe to events
    context.engine_api:add_event_listener("scene.load", function(event)
        print("Scene loaded:", event.data.path)
    end)

    -- Register component
    context.engine_api:register_component({
        type = "LuaScript",
        properties = {
            script_file = {type = "string", default = ""},
            auto_start = {type = "boolean", default = true}
        }
    })
end

function plugin:on_update(context, delta_time)
    self.state.counter = self.state.counter + 1

    if self.state.counter % 60 == 0 then
        print(string.format("Updated %d times", self.state.counter))
    end
end

function plugin:on_event(event)
    print("Event:", event.type, event.data)
end

function plugin:on_unload(context)
    print(string.format("Total updates: %d", self.state.counter))
end

return plugin
```

## Common Patterns

### Singleton Pattern

```rust
pub struct MyPlugin {
    // Use Arc<Mutex<T>> for thread-safe shared state
    state: Arc<Mutex<PluginState>>,
}
```

### Observer Pattern

```rust
fn on_load(&mut self, context: PluginContext) -> Result<()> {
    let event_bus = context.engine_api.event_bus();
    let mut subscriber = event_bus.subscribe();

    tokio::spawn(async move {
        while let Ok(event) = subscriber.recv().await {
            // Handle events
        }
    });

    Ok(())
}
```

### Factory Pattern

```rust
pub trait ComponentFactory {
    fn create(&self, config: &ComponentConfig) -> Box<dyn Component>;
}

pub struct MyComponentFactory;

impl ComponentFactory for MyComponentFactory {
    fn create(&self, config: &ComponentConfig) -> Box<dyn Component> {
        Box::new(MyComponent::new(config))
    }
}
```

### Builder Pattern

```rust
pub struct PluginBuilder {
    name: String,
    version: String,
    capabilities: Vec<PluginCapability>,
}

impl PluginBuilder {
    pub fn new(name: String, version: String) -> Self {
        Self {
            name,
            version,
            capabilities: Vec::new(),
        }
    }

    pub fn with_capability(mut self, cap: PluginCapability) -> Self {
        self.capabilities.push(cap);
        self
    }

    pub fn build(self) -> PluginMetadata {
        // ...
    }
}
```

## API Reference

### Plugin Trait

```rust
pub trait Plugin: Any {
    // Required
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn on_load(&mut self, context: PluginContext) -> Result<()>;

    // Optional (with defaults)
    fn api_version(&self) -> &str { "0.1.0" }
    fn description(&self) -> &str { "" }
    fn author(&self) -> &str { "" }
    fn dependencies(&self) -> &[&str] { &[] }
    fn capabilities(&self) -> &[PluginCapability] { &[] }

    fn on_unload(&mut self, context: PluginContext) -> Result<()> {
        Ok(())
    }

    fn on_update(&mut self, context: PluginContext, delta_time: f32) {}
    fn on_event(&mut self, event: &PluginEvent) {}
    fn config(&self) -> Option<PluginConfig> { None }
}
```

### PluginContext

```rust
pub struct PluginContext {
    pub engine_api: EngineApi,
    pub resource_manager: ResourceManager,
    pub config: PluginConfig,
    pub data: HashMap<String, String>,
}
```

#### Methods

- `get_data(&self, key: &str) -> Option<&String>`
- `set_data(&mut self, key: String, value: String)`

### PluginConfig

```rust
pub struct PluginConfig {
    pub settings: HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub auto_load: bool,
    pub hot_reload: bool,
}
```

#### Methods

- `new() -> Self`
- `with_setting(key: String, value: Value) -> Self`
- `get<T>(&self, key: &str) -> Result<Option<T>>`
- `set<T>(&mut self, key: String, value: T) -> Result<()>`

### PluginEvent

```rust
pub enum PluginEvent {
    PluginLoaded { name: String },
    PluginUnloaded { name: String },
    PluginError { name: String, error: String },
    SceneLoaded { path: String },
    SceneSaved { path: String },
    AssetImported { path: String },
    Tick { delta_time: f32 },
    Custom { type_: String, data: serde_json::Value },
}
```

### PluginCapability

```rust
pub enum PluginCapability {
    Render,
    Audio,
    Physics,
    Network,
    FileSystem,
    UserInterface,
    SceneModification,
    AssetPipeline,
    Custom(String),
}
```

### PluginPermission

```rust
pub enum PluginPermission {
    Read,
    Write,
    Network,
    Filesystem,
    Custom(String),
}
```

### Error Types

```rust
pub enum PluginError {
    NotFound(String),
    LoadFailed(String),
    IncompatibleVersion { required: String, found: String },
    DependencyNotFound(String),
    PermissionDenied(String),
    SandboxViolation(String),
    AbiMismatch(String),
    EventError(String),
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Other(String),
}
```

## Utility Functions

### Logging

```rust
// In plugin
fn on_load(&mut self, context: PluginContext) -> Result<()> {
    println!("INFO: Plugin loaded");
    eprintln!("ERROR: Something went wrong");
    Ok(())
}
```

### Time

```rust
use std::time::Instant;

let start = Instant::now();
// ... do work ...
let elapsed = start.elapsed();
```

### Threading

```rust
use tokio::task::spawn_blocking;

let result = spawn_blocking(|| {
    // CPU-intensive work
    42
}).await?;
```

## Performance Tips

1. **Avoid allocations in hot paths**
2. **Use `Arc` for shared data**
3. **Leverage async for I/O operations**
4. **Cache expensive computations**
5. **Profile with `cargo flamegraph`**

## Debugging

### Enable Logging

```rust
env_logger::init();
```

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin() {
        let plugin = MyPlugin;
        assert_eq!(plugin.name(), "my_plugin");
    }
}
```

### Integration Tests

```bash
cargo test --test integration_tests
```

## Best Practices

1. Always handle errors properly
2. Clean up resources in `on_unload()`
3. Use semantic versioning
4. Document public APIs
5. Write comprehensive tests
6. Keep `on_update()` fast
7. Request minimal permissions
8. Use type-safe APIs

## Additional Resources

- [Plugin System Guide](./PLUGIN_SYSTEM_GUIDE.md)
- [Example Plugins](../examples/)
- [Community Plugins](https://github.com/game-engine/plugins)
- [API Documentation](https://docs.game-engine.dev)
