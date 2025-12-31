# Game Engine LSP Server

Language Server Protocol implementation for the game engine, providing intelligent IDE support.

## Features

### 1. Code Completion
- **Component Completion**: Auto-complete engine components (Transform, Velocity, Health, etc.)
- **System Query Completion**: Complete system queries with Query<>, Res<>, ResMut<>
- **Resource Completion**: Complete resource access patterns
- **Field & Method Completion**: Complete component fields and methods

### 2. Hover Information
- Rich markdown documentation for all engine types
- Type information for components, resources, and systems
- Field and method signatures with descriptions
- Usage examples

### 3. Diagnostics
- Real-time error checking for unknown components
- Validation of system query parameters
- Resource access validation
- Warnings for mutable resource usage

### 4. Go to Definition
- Navigate to component definitions
- Navigate to resource definitions
- Navigate to system definitions (future enhancement)

## Installation

### Build the LSP Server

```bash
# Build with LSP feature
cargo build --bin game-engine-lsp --features lsp --release

# The binary will be at:
# target/release/game-engine-lsp
```

### IDE Configuration

#### VS Code

Create or edit `.vscode/settings.json`:

```json
{
  "gameEngine.lsp.path": "/path/to/target/release/game-engine-lsp"
}
```

Or install the VS Code extension (when available):

```bash
code --install-extension game-engine-lsp
```

#### Neovim

Add to your `init.lua`:

```lua
require('lspconfig').game_engine_lsp.setup {
  cmd = { '/path/to/target/release/game-engine-lsp' },
  filetypes = { 'rust' },
  root_dir = require('lspconfig').util.root_pattern('Cargo.toml', '.git'),
}
```

#### Emacs

Add to your configuration:

```elisp
(use-package lsp-mode
  :config
  (lsp-register-custom-settings
   '(("game-engine-lsp.path" "/path/to/target/release/game-engine-lsp"))))
```

## Usage

### Starting the Server

The LSP server is typically started automatically by your IDE. For testing:

```bash
# Direct stdin/stdout communication
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}' | game-engine-lsp
```

### Supported File Types

- `*.rs` - Rust source files using the game engine

### Example Usage

```rust
use game_engine::ecs::{Transform, Velocity, Health};

// Type "Query<" and get completion for all components
fn system(query: Query<(Transform, Velocity)>) {
    // Hover over "Transform" to see documentation
    for (transform, velocity) in query.iter() {
        // Auto-complete fields
        let pos = transform.position;
    }
}

// Type "Res<" and get completion for resources
fn another_system(time: Res<Time>) {
    let delta = time.delta_time(); // Hover to see method docs
}
```

## API Registry

The LSP server maintains a registry of all engine APIs:

### Default Components
- **Transform**: Position, rotation, and scale
- **Velocity**: Linear and angular velocity
- **Health**: Health component for game entities

### Default Systems
- **PhysicsSystem**: Updates physics simulation
- **RenderSystem**: Renders all visible entities

### Default Resources
- **Time**: Global time resource
- **AssetServer**: Asset loading and management

## Development

### Running Tests

```bash
# Run LSP tests
cargo test --package game_engine --features lsp --lib tools::lsp

# Run integration tests
cargo test --package game_engine --features lsp --test lsp_tests
```

### Adding New Components

To register a new component with the LSP server:

```rust
use game_engine::tools::lsp::registry::{ComponentDefinition, FieldDefinition};

// In your initialization code
let component = ComponentDefinition {
    name: "MyComponent".to_string(),
    module: "my_module::components".to_string(),
    description: "Description of my component".to_string(),
    fields: vec![
        FieldDefinition {
            name: "my_field".to_string(),
            type_name: "f32".to_string(),
            description: "Field description".to_string(),
            is_public: true,
        },
    ],
    methods: vec![],
    documentation: "# MyComponent\n\nFull documentation...".to_string(),
};

registry.register_component(component).await;
```

## Architecture

### Module Structure

```
src/tools/lsp/
├── mod.rs          # Module exports
├── registry.rs     # Engine API registry
├── completion.rs   # Code completion provider
├── hover.rs        # Hover information provider
├── diagnostics.rs  # Diagnostic provider
└── server.rs       # LSP server implementation
```

### Key Components

1. **EngineAPIRegistry**: Central registry for all engine APIs
2. **CompletionProvider**: Provides code completion suggestions
3. **HoverProvider**: Provides hover information
4. **DiagnosticProvider**: Analyzes code and provides diagnostics
5. **GameEngineLSP**: Main LSP server implementing tower-lsp traits

## Protocol Support

The server implements the following LSP capabilities:

- `textDocument synchronization`: Incremental
- `completion`: Trigger characters: `.`, `<`, `,`
- `hover`: Full markdown support
- `definition`: Go to definition
- `diagnostics`: Full document diagnostics

## Troubleshooting

### Server Not Starting

1. Check that the binary path is correct
2. Verify the LSP feature is enabled: `cargo build --features lsp`
3. Check IDE logs for error messages

### Completion Not Working

1. Ensure the file is recognized as Rust
2. Check that the server is running (see IDE logs)
3. Verify trigger characters (try typing `.` or `<`)

### Diagnostics Not Showing

1. Ensure the file is saved (some editors require save)
2. Check that diagnostics are enabled in your IDE
3. Verify the document is properly opened in the LSP server

## Performance

- Startup time: ~100ms (with async registry population)
- Memory usage: ~5-10MB base + cached document data
- Latency: <10ms for completion/hover requests

## Future Enhancements

- [ ] Add semantic tokens support
- [ ] Implement code actions
- [ ] Add workspace symbols support
- [ ] Support for custom macros and derive macros
- [ ] Integration with rust-analyzer for full Rust support
- [ ] Code lens for entity/component counts
- [ ] Inlay hints for query parameters

## License

MIT License - see main LICENSE file for details.

## Contributing

Contributions welcome! Please see CONTRIBUTING.md for guidelines.
