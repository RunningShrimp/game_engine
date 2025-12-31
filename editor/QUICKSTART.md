# Quick Start Guide

Get the Game Engine Editor up and running in 5 minutes!

## Step 1: Install Dependencies

### macOS
```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Node.js
brew install node

# Install Xcode Command Line Tools
xcode-select --install
```

### Linux (Ubuntu/Debian)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install Tauri dependencies
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

### Windows
```powershell
# Install Rust
# Download and run from: https://rustup.rs/

# Install Node.js
# Download from: https://nodejs.org/

# Install Visual Studio C++ Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/
```

## Step 2: Install Project Dependencies

```bash
cd /Users/wangbiao/Desktop/project/game_engine/editor
npm install
```

## Step 3: Start Development Server

```bash
npm run tauri:dev
```

This will:
1. Start Vite dev server on `http://localhost:1420`
2. Compile Rust backend
3. Open the Game Engine Editor window

## Step 4: Explore the Editor

### First Launch
1. Click **"Initialize Engine"** to start the engine
2. The editor interface will load with panels:
   - **Left**: Entity Hierarchy & Asset Browser
   - **Center**: Scene View & Console
   - **Right**: Property Inspector

### Basic Operations

#### Select Entities
- Click entities in the **Hierarchy** panel
- Click directly in the **Scene View** (raycast selection)

#### Edit Transforms
1. Select an entity
2. In the **Inspector**, edit Position/Rotation/Scale
3. Click **"Apply Transform"** to save changes

#### Create/Delete Entities
- Click **`+`** in Hierarchy to create new entity
- Select entity and click **`-`** to delete

#### Playback Scene
- Click **`▶`** to play the scene
- Click **`⏸`** to pause
- Click **`⏹`** to stop

#### View Console
- Logs appear in the bottom panel
- Filter by level: All, Errors, Warnings, Info, Debug
- Search box to filter messages

## Common Issues

### "Engine not created" Error
**Solution**: Click "Initialize Engine" button on startup screen

### Port 1420 Already in Use
**Solution**: Stop other services using port 1420 or edit `vite.config.ts`

### macOS: "xcode-select not found"
**Solution**: Run `xcode-select --install`

### Linux: "libwebkit not found"
**Solution**: Install dependencies from Step 1

### Windows: Build Fails
**Solution**:
1. Install Visual Studio C++ Build Tools
2. Restart terminal
3. Run `rustup default stable`

## Next Steps

### Development
- Edit React components in `src/components/`
- Edit Rust commands in `src-tauri/src/commands.rs`
- Changes will auto-reload

### Build for Production
```bash
npm run tauri:build
```
Output in `src-tauri/target/release/bundle/`

### Learn More
- Read full [README.md](./README.md)
- Check Tauri docs: https://tauri.app/
- React docs: https://react.dev/

## Keyboard Shortcuts

(Coming soon!)

## Tips

1. **Use the Console** - Check logs for errors and warnings
2. **Save Your Work** - Use Save button in menu bar
3. **Check Inspector** - Selected entity properties appear here
4. **Asset Browser** - Import assets with the Import button

## Getting Help

- Check `/docs` folder for detailed documentation
- Report issues on GitHub
- Join discussions for questions

---

**Happy Editing! 🎮**
