# Plugin SDK

Software Development Kit for creating plugins for the Game Engine Editor.

## Overview

This SDK provides everything you need to create plugins:

- **Templates**: Ready-to-use project templates for Rust, WASM, TypeScript, and Lua
- **Tools**: Scripts for plugin generation and validation
- **Documentation**: Comprehensive guides and API references
- **Examples**: Sample plugins demonstrating best practices

## Quick Start

### Create a New Plugin

Use the plugin generator:

```bash
./plugin-sdk/tools/create-plugin.sh -t rust -d "My awesome plugin" -a "Your Name" my_plugin
```

Options:
- `-t, --type`: Plugin type (rust, wasm, typescript, lua)
- `-d, --description`: Plugin description
- `-a, --author`: Plugin author
- `-o, --output`: Output directory

### Validate a Plugin

Check if your plugin meets all requirements:

```bash
./plugin-sdk/tools/validate-plugin.sh /path/to/plugin
```

## Directory Structure

```
plugin-sdk/
├── templates/           # Plugin templates
│   ├── rust/           # Rust plugin template
│   ├── wasm/           # WASM plugin template
│   ├── typescript/     # TypeScript plugin template
│   └── lua/            # Lua plugin template
├── tools/              # Development tools
│   ├── create-plugin.sh
│   └── validate-plugin.sh
└── docs/               # Documentation
    ├── README.md
    ├── PLUGIN_SYSTEM_GUIDE.md
    └── PLUGIN_SDK_REFERENCE.md
```

## Plugin Types

### 1. Rust Plugins (Native)

**Best for**: High-performance plugins, full system access

**Pros**:
- Maximum performance
- Full engine API access
- Type safety
- Direct memory access

**Cons**:
- Requires compilation
- Platform-specific binaries
- Steeper learning curve

**Use when**: You need maximum performance or low-level system access

### 2. WASM Plugins

**Best for**: Cross-platform plugins, web deployment

**Pros**:
- Cross-platform
- Sandboxed execution
- Fast startup
- Small binary size

**Cons**:
- Limited API access
- No threading
- Memory constraints

**Use when**: You want portability and security

### 3. TypeScript Plugins

**Best for**: Editor extensions, UI plugins

**Pros**:
- Easy to develop
- Large ecosystem
- Hot reload
- No compilation needed

**Cons**:
- Lower performance
- JavaScript runtime
- Limited access

**Use when**: Building editor UI or tooling

### 4. Lua Plugins

**Best for**: Scripting, game logic, quick prototypes

**Pros**:
- Very simple
- Instant reload
- Easy to learn
- Embedded

**Cons**:
- Slowest performance
- Limited tooling
- Dynamic typing

**Use when**: Rapid prototyping or simple scripting

## Templates

### Rust Template

```bash
./tools/create-plugin.sh -t rust my_plugin
cd my_plugin
cargo build --release
```

Output:
- macOS: `target/release/libmy_plugin.dylib`
- Linux: `target/release/libmy_plugin.so`
- Windows: `target/release/my_plugin.dll`

### WASM Template

```bash
./tools/create-plugin.sh -t wasm my_plugin
cd my_plugin
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen target/wasm32-unknown-unknown/release/my_plugin.wasm --out-dir ./dist
```

### TypeScript Template

```bash
./tools/create-plugin.sh -t typescript my_plugin
cd my_plugin
npm install
npm run build
```

### Lua Template

```bash
./tools/create-plugin.sh -t lua my_plugin
# No build step required
# Copy plugin.lua to editor's plugins directory
```

## Development Workflow

1. **Create plugin** from template
2. **Implement** your plugin logic
3. **Test** locally
4. **Validate** with validation script
5. **Build** for release
6. **Distribute** to users

## Best Practices

1. **Start Simple**: Begin with minimal functionality
2. **Test Often**: Use the validation script
3. **Document**: Keep README.md updated
4. **Version**: Follow semantic versioning
5. **Error Handling**: Always handle errors gracefully
6. **Performance**: Profile before optimizing
7. **Security**: Request minimal permissions

## Common Tasks

### Add Configuration

Edit `plugin.toml`:

```toml
[config.settings]
my_setting = "value"
number_setting = 42
```

Access in plugin:

```rust
let value = context.config.get::<String>("my_setting")?;
```

### Subscribe to Events

```rust
fn on_load(&mut self, context: PluginContext) -> Result<()> {
    let event_bus = context.engine_api.event_bus();
    // Subscribe to events
    Ok(())
}
```

### Access Resources

```rust
fn on_load(&mut self, context: PluginContext) -> Result<()> {
    let assets = context.resource_manager.list_assets()?;
    Ok(())
}
```

## Troubleshooting

### Plugin Not Loading

1. Check file extension (`.so`, `.dylib`, `.dll`)
2. Verify plugin manifest (`plugin.toml`)
3. Check dependencies are satisfied
4. Review error messages in editor console

### Build Errors

1. Ensure Rust version is compatible
2. Update dependencies: `cargo update`
3. Clean and rebuild: `cargo clean && cargo build`

### Validation Errors

Run validation script to identify issues:
```bash
./tools/validate-plugin.sh /path/to/plugin
```

## Resources

- [Plugin System Guide](../docs/PLUGIN_SYSTEM_GUIDE.md)
- [SDK Reference](../docs/PLUGIN_SDK_REFERENCE.md)
- [Example Plugins](../examples/)
- [Community Plugins](https://github.com/game-engine/plugins)

## Support

- GitHub: https://github.com/game-engine/editor/issues
- Discord: https://discord.gg/game-engine
- Email: support@game-engine.dev

## License

MIT License - see LICENSE file for details
