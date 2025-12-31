# Game Engine CLI Tool - User Guide

## Overview

The Game Engine CLI tool is a command-line interface for quickly scaffolding new game projects using pre-built templates. It simplifies the setup process and provides a standardized project structure.

## Installation

The CLI tool is built automatically when you compile the game engine with the `cli` feature:

```bash
cargo build --release --features cli
```

This creates the `game-engine` binary in `target/release/`.

You can also add it to your PATH:

```bash
export PATH="$PATH:/path/to/game_engine/target/release"
```

## Available Commands

### 1. Create a New Project

Creates a new game project from a template:

```bash
game-engine new <project-name> --template <template-name>
```

**Examples:**

```bash
# Create a basic game project
game-engine new my-game --template basic

# Create a 2D platformer
game-engine new platformer-game --template 2d-platformer

# Create a 3D FPS game
game-engine new shooter --template 3d-fps

# Create in a specific directory
game-engine new my-game --template basic --output ~/projects
```

**Options:**
- `--template <name>`: Specify the template to use (basic, 2d-platformer, 3d-fps)
- `--output <path>`: Specify output directory (default: current directory)
- `--interactive`: Interactive mode to select template from a list

### 2. List Templates

Lists all available project templates:

```bash
game-engine template list
```

**Options:**
- `--search <keyword>`: Search templates by keyword
- `--detailed`: Show detailed information about each template

**Examples:**

```bash
# List all templates
game-engine template list

# List with details
game-engine template list --detailed

# Search templates
game-engine template list --search platformer
```

### 3. Template Information

Shows detailed information about a specific template:

```bash
game-engine template info <template-name>
```

**Example:**

```bash
game-engine template info 2d-platformer
```

### 4. Initialize Existing Project

Initializes the current directory as a game engine project:

```bash
game-engine init
```

**Options:**
- `--force`: Force initialization even if project files exist

This creates:
- `Cargo.toml` - Project configuration
- `src/main.rs` - Entry point
- `.gitignore` - Git ignore file
- `assets/` - Assets directory
- `scripts/` - Scripts directory

### 5. Engine Information

Displays version and configuration information:

```bash
game-engine info
```

## Available Templates

### Basic Template (`basic`)

A minimal game template with:
- Basic window and rendering setup
- Simple game loop
- Minimal ECS setup
- Example scene

**Use case:** Learning the engine basics, small prototypes

### 2D Platformer Template (`2d-platformer`)

A complete 2D platformer game template with:
- 2D rendering system
- Platform physics
- Player controller with movement and jumping
- Tile map support
- Sprite animation system
- Example level JSON

**Use case:** 2D platformer games, side-scrollers

### 3D FPS Template (`3d-fps`)

A 3D first-person shooter template with:
- 3D rendering with PBR lighting
- First-person camera controller
- Weapon system with shooting mechanics
- Enemy AI with state machine
- 3D physics simulation
- Arena map structure

**Use case:** 3D shooters, action games

## Project Structure

Generated projects follow this structure:

```
my-game/
├── assets/              # Game assets
│   ├── textures/       # Textures and images
│   ├── models/         # 3D models (GLTF/FBX)
│   ├── audio/          # Sound effects and music
│   └── levels/         # Level data (JSON)
├── scripts/            # Game scripts (Lua)
│   └── main.lua        # Main game script
├── src/                # Rust source code
│   └── main.rs         # Entry point
├── .vscode/            # VS Code configuration
│   ├── settings.json   # Editor settings
│   └── extensions.json # Recommended extensions
├── .gitignore          # Git ignore file
├── Cargo.toml          # Project configuration
└── README.md           # Project documentation
```

## Template Variables

Templates use Handlebars syntax with the following variables:

- `{{name}}` - Project name (kebab-case)
- `{{name_title}}` - Project name in Title Case
- `{{name_upper}}` - Project name in SCREAMING_SNAKE_CASE
- `{{name_kebab}}` - Project name in kebab-case
- `{{template_name}}` - Template name
- `{{template_description}}` - Template description
- `{{engine_version}}` - Engine version
- `{{year}}` - Current year

## Next Steps After Creating a Project

1. **Navigate to the project:**
   ```bash
   cd my-game
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run the project:**
   ```bash
   cargo run
   ```

4. **Start developing:**
   - Edit `src/main.rs` for game logic
   - Add assets to `assets/`
   - Write scripts in `scripts/`
   - Configure in `Cargo.toml`

## Customizing Templates

You can create custom templates by:

1. Creating a new directory in `templates/`
2. Adding template files with `.hbs` extension for Handlebars processing
3. Using template variables in your files
4. Adding your template to `ProjectTemplate` enum in `src/tools/cli/template.rs`

## Troubleshooting

### Template Not Found

If you get "Template directory not found" error:

1. Check that templates exist in `game_engine/templates/`
2. Verify template name matches available templates (use `game-engine template list`)
3. Ensure you're running the CLI from the correct directory

### Project Already Exists

If the project directory already exists:

1. Choose a different project name
2. Remove or rename the existing directory
3. Use a different output directory with `--output`

### Compilation Errors

If the generated project doesn't compile:

1. Check Rust version (requires 2021 edition)
2. Ensure engine dependencies are correctly specified
3. Verify all features are enabled in `Cargo.toml`

## Examples

### Example 1: Create a Simple Game

```bash
game-engine new simple-game --template basic
cd simple-game
cargo run
```

### Example 2: Create a 2D Platformer

```bash
game-engine new platformer --template 2d-platformer
cd platformer
# Edit assets/levels/level1.json
cargo run
```

### Example 3: Create a 3D FPS with Custom Output

```bash
game-engine new my-shooter --template 3d-fps --output ~/games
cd ~/games/my-shooter
cargo run
```

### Example 4: Interactive Template Selection

```bash
game-engine new my-game --interactive
# Select from the list
# Follow the prompts
```

## Contributing

To add new templates or improve existing ones:

1. Edit template files in `templates/`
2. Update `ProjectTemplate` enum in `src/tools/cli/template.rs`
3. Update `TemplateMetadata` to reflect new features
4. Test with `game-engine new test-project --template your-template`

## License

MIT OR Apache-2.0

## Support

For issues and feature requests, please visit:
https://github.com/username/game_engine/issues
