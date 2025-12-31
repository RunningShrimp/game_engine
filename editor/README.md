# Game Engine Editor

A cross-platform game engine editor built with [Tauri](https://tauri.app/) and [React](https://react.dev/).

## Overview

The Game Engine Editor provides a modern, native GUI for game development, leveraging the power of Rust backend with a responsive React frontend. It offers essential tools for scene editing, entity management, asset browsing, and real-time console logging.

## Features

- **Scene Viewport**: WebGL-powered 3D scene visualization with raycasting for entity selection
- **Entity Hierarchy**: Tree-based view of scene entities with parent-child relationships
- **Property Inspector**: Real-time editing of entity components (Transform, Mesh, Material, etc.)
- **Asset Browser**: Browse and import game assets (models, textures, materials, audio)
- **Console Panel**: Real-time logging with filtering and auto-scroll capabilities
- **Playback Controls**: Play, pause, and stop scene simulation
- **Cross-platform**: Native applications for Windows, macOS, and Linux

## Architecture

```
editor/
├── src-tauri/           # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs      # Tauri application entry point
│   │   ├── commands.rs  # Tauri command handlers
│   │   ├── events.rs    # Event system definitions
│   │   └── state.rs     # Application state management
│   ├── Cargo.toml       # Rust dependencies
│   ├── tauri.conf.json  # Tauri configuration
│   ├── build.rs         # Build script
│   └── icons/           # Application icons
├── src/                 # React frontend
│   ├── components/      # React components
│   │   ├── SceneView.tsx       # Scene viewport with WebGL
│   │   ├── Hierarchy.tsx       # Entity tree view
│   │   ├── Inspector.tsx       # Property editor
│   │   ├── AssetBrowser.tsx    # Asset management
│   │   └── Console.tsx         # Log viewer
│   ├── App.tsx         # Main application component
│   ├── main.tsx        # React entry point
│   ├── index.css       # Global styles
│   └── index.html      # HTML template
├── package.json         # Node.js dependencies
├── tsconfig.json        # TypeScript configuration
├── vite.config.ts       # Vite build configuration
└── README.md           # This file
```

## Prerequisites

### Required

- **Rust**: 1.70 or higher
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- **Node.js**: 18 or higher
  ```bash
  # On macOS
  brew install node

  # On Linux
  sudo apt install nodejs npm

  # On Windows
  # Download from https://nodejs.org/
  ```

### Platform-Specific Dependencies

#### macOS
```bash
# Install Xcode Command Line Tools
xcode-select --install
```

#### Linux (Ubuntu/Debian)
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.0-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

#### Windows
```bash
# Install Visual Studio C++ Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

## Installation

1. **Clone the repository** (if not already done)
   ```bash
   cd /path/to/project
   ```

2. **Install frontend dependencies**
   ```bash
   cd editor
   npm install
   ```

3. **Build Tauri CLI** (first time only)
   ```bash
   cd src-tauri
   cargo build
   cd ..
   ```

## Development

### Start Development Server

```bash
npm run tauri:dev
```

This will:
- Start the Vite development server on `http://localhost:1420`
- Launch the Tauri application
- Enable hot-reload for both frontend and backend changes

### Development Workflow

1. **Frontend Changes**: Edit files in `src/`. Changes will auto-reload.
2. **Backend Changes**: Edit files in `src-tauri/src/`. Save and the app will reload.

### Available Scripts

- `npm run dev` - Start Vite development server only
- `npm run build` - Build frontend for production
- `npm run preview` - Preview production build
- `npm run tauri` - Run Tauri CLI
- `npm run tauri:dev` - Start Tauri in development mode
- `npm run tauri:build` - Build Tauri application for distribution

## Building for Production

### Build for Current Platform

```bash
npm run tauri:build
```

The built application will be in `src-tauri/target/release/bundle/`:

- **macOS**: `.dmg` or `.app` bundle
- **Linux**: `.deb` or `.AppImage` package
- **Windows**: `.msi` or `.exe` installer

### Build for Specific Platform

#### macOS
```bash
npm run tauri:build -- --target universal-apple-darwin
```

#### Linux
```bash
npm run tauri:build -- --target x86_64-unknown-linux-gnu
```

#### Windows
```bash
npm run tauri:build -- --target x86_64-pc-windows-msvc
```

## Usage

### Basic Workflow

1. **Initialize Engine**: Click "Initialize Engine" on startup
2. **Navigate Scene**:
   - Use the **Scene View** to visualize your 3D scene
   - Click on entities to select them
3. **Manage Entities**:
   - Use the **Hierarchy** panel to browse and select entities
   - Create new entities with the `+` button
   - Delete selected entities with the `-` button
4. **Edit Properties**:
   - Select an entity to view its components in the **Inspector**
   - Edit Transform (Position, Rotation, Scale) values
   - Click "Apply Transform" to commit changes
5. **Browse Assets**:
   - Use the **Asset Browser** to view project assets
   - Click "Import" to add new assets
   - Double-click assets to preview (future feature)
6. **Control Playback**:
   - Click `▶` to play the scene
   - Click `⏸` to pause
   - Click `⏹` to stop and reset
7. **Monitor Logs**:
   - View real-time logs in the **Console** panel
   - Filter by log level (Error, Warning, Info, Debug)
   - Use the search box to filter messages

### Keyboard Shortcuts

(Coming soon)

## Tauri Commands

The editor exposes the following Tauri commands to the frontend:

### Engine Management
- `create_engine()` - Initialize engine instance
- `get_entities()` - Get scene entity hierarchy
- `create_entity(name, parent_id)` - Create new entity
- `delete_entity(entity_id)` - Delete entity

### Component Editing
- `get_entity_components(entity_id)` - Get entity components
- `update_component(entity_id, component)` - Update component data
- `update_transform(entity_id, position, rotation, scale)` - Update transform

### Scene Control
- `play_scene()` - Start scene simulation
- `stop_scene()` - Stop scene simulation
- `pause_scene()` - Pause scene simulation

### Interaction
- `raycast(x, y)` - Perform raycast for entity picking

### Asset Management
- `get_assets(asset_type)` - Get asset list
- `import_asset(source_path, asset_type)` - Import asset

### Scene Persistence
- `save_scene(scene_path)` - Save scene to file
- `load_scene(scene_path)` - Load scene from file

### Logging
- `get_console_logs(limit)` - Get console log entries

## Configuration

### Tauri Configuration

Edit `src-tauri/tauri.conf.json` to customize:

- Window size and behavior
- Application metadata (name, version, identifier)
- Security policies (CSP, filesystem access)
- Bundle settings (icons, targets)

### Frontend Configuration

- **Vite**: `vite.config.ts` - Build settings, dev server config
- **TypeScript**: `tsconfig.json` - Compiler options
- **Tailwind**: `tailwind.config.js` - Styling configuration
- **Package**: `package.json` - Dependencies and scripts

## Troubleshooting

### Common Issues

#### "Engine not created" Error
- Ensure you click "Initialize Engine" on startup
- Check console for Rust error messages

#### WebGL Context Not Available
- Ensure your GPU drivers are up to date
- Try enabling hardware acceleration in browser settings

#### Build Fails on macOS
```bash
# Reset Xcode tools
sudo xcode-select --reset
sudo xcode-select --install
```

#### Build Fails on Linux
```bash
# Install missing dependencies
sudo apt install libwebkit2gtk-4.0-dev build-essential
```

#### Build Fails on Windows
- Ensure Visual Studio C++ Build Tools are installed
- Add Rust to PATH: `rustup default stable`

### Debugging

#### Enable Rust Logging
```bash
RUST_LOG=debug npm run tauri:dev
```

#### View Tauri Logs
- macOS: `~/Library/Logs/com.game-engine.editor/`
- Linux: `~/.local/share/com.game-engine.editor/logs/`
- Windows: `%APPDATA%\com.game-engine.editor\logs\`

## Plugin Development

### Adding New Components

1. **Define Component Data Structure** (`src-tauri/src/commands.rs`):
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyComponentData {
    pub field1: String,
    pub field2: f32,
}
```

2. **Add Tauri Command** (`src-tauri/src/commands.rs`):
```rust
#[tauri::command]
pub async fn update_my_component(
    entity_id: u64,
    data: MyComponentData,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    // Implementation
    Ok(())
}
```

3. **Register Command** (`src-tauri/src/main.rs`):
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing commands
    update_my_component,
])
```

4. **Create React Component** (`src/components/MyComponentEditor.tsx`):
```tsx
import { invoke } from '@tauri-apps/api/tauri';

export const MyComponentEditor: React.FC<{ entityId: number }> = ({ entityId }) => {
  const handleUpdate = async () => {
    await invoke('update_my_component', {
      entityId,
      data: { /* ... */ }
    });
  };

  return (
    <div className="my-component">
      {/* Component UI */}
    </div>
  );
};
```

## Performance Considerations

- **Entity Updates**: Throttle transform updates to avoid excessive IPC calls
- **Log Polling**: Console refresh rate is limited to 1 second
- **Asset Thumbnails**: Lazy-load and cache thumbnails
- **WebGL Rendering**: Use requestAnimationFrame for smooth rendering

## Security

The editor implements the following security measures:

- **Content Security Policy**: Restricts resource loading
- **Filesystem Access**: Scoped to project directories
- **IPC Validation**: All inputs are validated on the Rust side
- **Sandboxing**: Tauri provides OS-level sandboxing

## Future Enhancements

- [ ] Undo/Redo system integration
- [ ] Visual scripting editor
- [ ] Material editor with preview
- [ ] Particle system editor
- [ ] Animation timeline
- [ ] Multi-scene editing
- [ ] Collaborative editing
- [ ] Plugin system
- [ ] Custom layout persistence
- [ ] Theme customization

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style

- **Rust**: Follow `rustfmt` formatting
- **TypeScript/React**: Follow ESLint rules
- **Commit Messages**: Use conventional commits format

## License

MIT OR Apache-2.0

## Acknowledgments

- [Tauri](https://tauri.app/) - Cross-platform desktop framework
- [React](https://react.dev/) - UI library
- [Vite](https://vitejs.dev/) - Build tool
- [Tailwind CSS](https://tailwindcss.com/) - Styling framework
- [WebGPU](https://www.w3.org/TR/webgpu/) - Graphics API

## Support

- **Documentation**: See `/docs` folder
- **Issues**: Report bugs on GitHub Issues
- **Discussions**: Join GitHub Discussions for questions

## Changelog

### v0.1.0 (2024-12-31)
- Initial release
- Basic scene editing
- Entity hierarchy
- Component inspector
- Asset browser
- Console logging
- Playback controls

---

**Built with ❤️ using Tauri and React**
