# Game Engine VS Code Extension

[![Version](https://img.shields.io/visual-studio-marketplace/v/game-engine.game-engine-vscode)](https://marketplace.visualstudio.com/items?itemName=game-engine.game-engine-vscode)
[![Installs](https://img.shields.io/visual-studio-marketplace/i/game-engine.game-engine-vscode)](https://marketplace.visualstudio.com/items?itemName=game-engine.game-engine-vscode)
[![Rating](https://img.shields.io/visual-studio-marketplace/r/game-engine.game-engine-vscode)](https://marketplace.visualstudio.com/items?itemName=game-engine.game-engine-vscode)

Intelligent code support for **Game Engine** projects in Visual Studio Code.

## Features

✨ **Intelligent Code Completion**
- Context-aware suggestions for engine components, systems, and resources
- Auto-completion for ECS queries and component access
- Smart suggestions for engine APIs

💡 **Rich Hover Information**
- Detailed documentation for all engine types and functions
- Parameter hints for function calls
- Type information for variables and expressions

🔍 **Go-to-Definition**
- Navigate to symbol definitions across your project
- Find all references to symbols
- View document symbols and workspace symbols

⚡ **Real-time Diagnostics**
- Instant error detection as you type
- Validation of engine API usage
- Performance warnings and suggestions

🐛 **Integrated Debugging**
- Debug adapter protocol (DAP) support
- Breakpoint management
- Variable inspection and watch windows

## Installation

### From Marketplace (Recommended)

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search for "Game Engine"
4. Click "Install"

### From Source

1. Clone this repository
2. Install dependencies:
   ```bash
   npm install
   ```
3. Compile the extension:
   ```bash
   npm run compile
   ```
4. Press F5 in VS Code to launch a new Extension Development Host window

## Requirements

- [Game Engine](https://github.com/game-engine/game-engine) v0.3.0 or later
- [Rust toolchain](https://rustup.rs/) (stable or nightly)
- VS Code 1.80.0 or later

## Configuration

The extension can be configured via VS Code settings (`settings.json`):

```json
{
  // Enable/disable the LSP server
  "gameEngine.lsp.enabled": true,

  // Path to the LSP server executable (optional)
  "gameEngine.lsp.path": "/path/to/game-engine-lsp",

  // Additional arguments for the LSP server (optional)
  "gameEngine.lsp.args": [],

  // Traces the communication between VS Code and the language server
  "gameEngine.lsp.trace.server": "off",

  // Maximum number of problems to report
  "gameEngine.lsp.maxNumberOfProblems": 100,

  // Enable code completion
  "gameEngine.completion.enable": true,

  // Enable hover information
  "gameEngine.hover.enable": true,

  // Enable real-time diagnostics
  "gameEngine.diagnostics.enable": true
}
```

## Usage

### Basic Usage

1. Open a Rust project that uses Game Engine
2. The LSP server will start automatically
3. Start coding with intelligent suggestions!

### Commands

The extension provides the following commands:

- **Game Engine: Restart LSP Server** - Restart the language server
- **Game Engine: Show Documentation** - Open online documentation
- **Game Engine: Open Playground** - Create a new playground file
- **Game Engine: Run Diagnostics** - Force diagnostic run
- **Game Engine: Show Performance** - Display LSP server performance stats

### Keyboard Shortcuts

| Command | Shortcut | Description |
|---------|----------|-------------|
| Trigger Suggest | Ctrl+Space | Show code completion |
| Go to Definition | F12 | Jump to symbol definition |
| Peek Definition | Alt+F12 | Peek symbol definition |
| Find All References | Shift+F12 | Find all symbol references |
| Rename Symbol | F2 | Rename symbol across project |
| Show Hover | Ctrl+Space | Show hover information |

## Development

### Project Structure

```
vscode-extension/
├── src/
│   └── extension.ts       # Main extension entry point
├── package.json           # Extension manifest
├── tsconfig.json          # TypeScript configuration
├── README.md              # This file
└── out/                   # Compiled output (generated)
```

### Building

```bash
# Install dependencies
npm install

# Compile TypeScript
npm run compile

# Watch for changes
npm run watch

# Run linter
npm run lint

# Run tests
npm run test
```

### Publishing

```bash
# Install vsce (VS Code Extension Manager)
npm install -g @vscode/vsce

# Package the extension
vsce package

# Publish to marketplace
vsce publish
```

## Troubleshooting

### LSP Server Not Starting

1. Check that `game-engine-lsp` is installed and in your PATH
2. Verify the path in settings: `"gameEngine.lsp.path"`
3. Check the output channel "Game Engine LSP" for error messages

### Code Completion Not Working

1. Make sure the LSP server is running (check output channels)
2. Verify completion is enabled: `"gameEngine.completion.enable": true`
3. Try restarting the LSP server: `Game Engine: Restart LSP Server`

### Diagnostics Not Showing

1. Check that diagnostics are enabled: `"gameEngine.diagnostics.enable": true`
2. Open the Problems panel (View → Problems)
3. Try restarting the LSP server

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## License

This extension is licensed under the [MIT License](../../LICENSE).

## Acknowledgments

- Built with [vscode-languageclient](https://github.com/microsoft/vscode-languageserver-nodejs)
- Powered by [Game Engine](https://github.com/game-engine/game-engine)

## Support

- 📖 [Documentation](https://docs.game-engine.dev)
- 💬 [Discord](https://discord.gg/game-engine)
- 🐛 [Issue Tracker](https://github.com/game-engine/vscode-extension/issues)
- ✉️ [Email](mailto:support@game-engine.dev)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.
