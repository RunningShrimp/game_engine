# Examples

The game engine includes numerous examples demonstrating various features. This page provides an overview of all available examples and how to use them.

## Running Examples

### Basic Usage

```bash
# Run a specific example
cargo run --example hello_world

# Run with release optimizations
cargo run --release --example hello_world

# Run with logging
RUST_LOG=debug cargo run --example hello_world
```

### List All Examples

```bash
# List all available examples
cargo run --example --help

# Or check the examples directory
ls game_engine/examples/
```

## Beginner Examples

### hello_world

The simplest example demonstrating basic engine setup.

```bash
cargo run --example hello_world
```

**What you'll learn:**
- Basic engine initialization
- Creating a simple window
- Basic game loop

**Source:** [examples/hello_world.rs](../game_engine/examples/hello_world.rs)

### ecs_basics

Introduction to the Entity Component System.

```bash
cargo run --example ecs_basics
```

**What you'll learn:**
- Creating entities
- Adding components
- Querying entities
- Basic systems

**Source:** [examples/ecs_basics.rs](../game_engine/examples/ecs_basics.rs)

### rendering

Basic rendering demonstration.

```bash
cargo run --example rendering
```

**What you'll learn:**
- Setting up a camera
- Creating 3D meshes
- Basic lighting
- Rendering loop

**Source:** [examples/rendering.rs](../game_engine/examples/rendering.rs)

### physics

Physics simulation example.

```bash
cargo run --example physics
```

**What you'll learn:**
- Rigid bodies
- Colliders
- Gravity
- Physical interactions

**Source:** [examples/physics.rs](../game_engine/examples/physics.rs)

### audio

Audio playback example.

```bash
cargo run --example audio
```

**What you'll learn:**
- Playing sound effects
- Background music
- Volume control
- 3D audio positioning

**Source:** [examples/audio.rs](../game_engine/examples/audio.rs)

## Intermediate Examples

### render_advanced

Advanced rendering techniques.

```bash
cargo run --example render_advanced
```

**Features:**
- Multiple lights
- Shadow mapping
- Post-processing effects
- Custom shaders

**Source:** [examples/render_advanced.rs](../game_engine/examples/render_advanced.rs)

### input_handling

Comprehensive input handling demonstration.

```bash
cargo run --example input_handling
```

**Features:**
- Keyboard input
- Mouse input
- Gamepad support
- Input mapping

**Source:** [examples/input_handling.rs](../game_engine/examples/input_handling.rs)

### resources

Resource management example.

```bash
cargo run --example resources
```

**Features:**
- Loading textures
- Loading 3D models
- Resource caching
- Hot reloading

**Source:** [examples/resources.rs](../game_engine/examples/resources.rs)

### ui

User interface example.

```bash
cargo run --example ui
```

**Features:**
- Text rendering
- Buttons
- Layouts
- Event handling

**Source:** [examples/ui.rs](../game_engine/examples/ui.rs)

## Advanced Examples

### domain

Domain-Driven Design example.

```bash
cargo run --example domain
```

**Features:**
- Domain entities
- Value objects
- Aggregates
- Domain events

**Source:** [examples/domain.rs](../game_engine/examples/domain.rs)

### cqrs_example

CQRS pattern implementation.

```bash
cargo run --example cqrs_example
```

**Features:**
- Command handling
- Query handling
- Event sourcing
- Read/write separation

**Source:** [examples/cqrs_example.rs](../game_engine/examples/cqrs_example.rs)

### event_sourcing_example

Event sourcing demonstration.

```bash
cargo run --example event_sourcing_example
```

**Features:**
- Event store
- Event replay
- Snapshotting
- Event versioning

**Source:** [examples/event_sourcing_example.rs](../game_engine/examples/event_sourcing_example.rs)

## Networking Examples

### multiplayer

Basic multiplayer example.

```bash
# Terminal 1 - Server
cargo run --example multiplayer -- server

# Terminal 2 - Client
cargo run --example multiplayer -- client 127.0.0.1
```

**Features:**
- Server-client architecture
- Network serialization
- State synchronization
- Latency compensation

**Source:** [examples/multiplayer.rs](../game_engine/examples/multiplayer.rs)

### network_multiplayer

Advanced multiplayer with prediction.

```bash
# Terminal 1 - Server
cargo run --example network_multiplayer -- server

# Terminal 2 - Client 1
cargo run --example network_multiplayer -- client

# Terminal 3 - Client 2
cargo run --example network_multiplayer -- client
```

**Features:**
- Client-side prediction
- Server reconciliation
- Entity interpolation
- Lag compensation

**Source:** [examples/network_multiplayer.rs](../game_engine/examples/network_multiplayer.rs)

## Performance Examples

### performance_benchmark_example

Performance benchmarking demo.

```bash
cargo run --example performance_benchmark_example
```

**Features:**
- FPS counter
- Frame time graphs
- Entity count benchmark
- Memory profiling

**Source:** [examples/performance_benchmark_example.rs](../game_engine/examples/performance_benchmark_example.rs)

### stress_test

Stress testing the engine.

```bash
cargo run --example stress_test
```

**Features:**
- 10,000 entities
- Complex rendering
- Physics simulation
- Performance metrics

**Source:** [examples/stress_test.rs](../game_engine/examples/stress_test.rs)

## Debugging Examples

### tracy_profiling

Tracy profiler integration.

```bash
# Install Tracy first
# Then run with profiling enabled
cargo run --example tracy_profiling
```

**Features:**
- CPU profiling
- Memory profiling
- Frame graphs
- Real-time metrics

**Source:** [examples/tracy_profiling.rs](../game_engine/examples/tracy_profiling.rs)

### world_inspector_example

World inspector for debugging.

```bash
cargo run --example world_inspector_example
```

**Features:**
- Entity browser
- Component inspector
- Real-time editing
- Entity spawning

**Source:** [examples/world_inspector_example.rs](../game_engine/examples/world_inspector_example.rs)

### logging_example

Comprehensive logging demonstration.

```bash
RUST_LOG=debug cargo run --example logging_example
```

**Features:**
- Log levels
- Custom loggers
- Structured logging
- Log filtering

**Source:** [examples/logging_example.rs](../game_engine/examples/logging_example.rs)

## Platform-Specific Examples

### wasm_example

WebAssembly browser example.

```bash
# Build for WASM
cargo install wasm-pack
wasm-pack build --target web

# Serve with local server
python -m http.server 8000
# Open http://localhost:8000
```

**Features:**
- WebGL rendering
- Browser input
- Canvas integration
- Web Audio

**Source:** [examples/wasm_example.rs](../game_engine/examples/wasm_example.rs)

### android_example

Android platform example.

```bash
# Build APK
./scripts/build_android.sh

# Install on device
adb install target/android apk/game engine.apk
```

**Features:**
- Touch controls
- Android lifecycle
- Permissions
- Native integration

**Source:** [examples/android_example.rs](../game_engine/examples/android_example.rs)

## Specialized Examples

### particle_system

Particle effects demonstration.

```bash
cargo run --example particle_system
```

**Features:**
- Particle emitters
- Particle physics
- Trail effects
- GPU acceleration

**Source:** [examples/particle_system.rs](../game_engine/examples/particle_system.rs)

### animation_example

Animation system demonstration.

```bash
cargo run --example animation_example
```

**Features:**
- Skeletal animation
- Morph targets
- Animation blending
- Animation states

**Source:** [examples/animation_example.rs](../game_engine/examples/animation_example.rs)

### pathfinding_example

Pathfinding algorithms demonstration.

```bash
cargo run --example pathfinding_example
```

**Features:**
- A* pathfinding
- Navigation mesh
- Dynamic obstacles
- Path smoothing

**Source:** [examples/pathfinding_example.rs](../game_engine/examples/pathfinding_example.rs)

### ai_behavior_example

AI behavior trees demonstration.

```bash
cargo run --example ai_behavior_example
```

**Features:**
- Behavior trees
- State machines
- Utility AI
- GOAP (Goal-Oriented Action Planning)

**Source:** [examples/ai_behavior_example.rs](../game_engine/examples/ai_behavior_example.rs)

## Complete Game Examples

### platformer_example

Simple platformer game.

```bash
cargo run --example platformer_example
```

**Features:**
- Platformer physics
- Player character
- Collectibles
- Level design

**Source:** [examples/platformer_example.rs](../game_engine/examples/platformer_example.rs)

### fps_example

First-person shooter example.

```bash
cargo run --example fps_example
```

**Features:**
- First-person camera
- Weapon system
- Enemy AI
- Health system

**Source:** [examples/fps_example.rs](../game_engine/examples/fps_example.rs)

### racing_example

Racing game example.

```bash
cargo run --example racing_example
```

**Features:**
- Vehicle physics
- Track design
- Lap timing
- AI opponents

**Source:** [examples/racing_example.rs](../game_engine/examples/racing_example.rs)

## Example Categories Summary

| Category | Examples | Description |
|----------|----------|-------------|
| **Beginner** | 5 | Basic engine features |
| **Intermediate** | 4 | Specific subsystems |
| **Advanced** | 3 | Design patterns |
| **Networking** | 2 | Multiplayer games |
| **Performance** | 2 | Profiling & optimization |
| **Debugging** | 3 | Development tools |
| **Platform** | 2 | WASM & Android |
| **Specialized** | 4 | Advanced features |
| **Games** | 3 | Complete game demos |

## Learning Path

We recommend following this path:

1. **Start with basics:**
   - hello_world
   - ecs_basics
   - rendering

2. **Explore subsystems:**
   - physics
   - audio
   - input_handling

3. **Learn patterns:**
   - domain
   - cqrs_example
   - event_sourcing_example

4. **Build multiplayer:**
   - multiplayer
   - network_multiplayer

5. **Optimize:**
   - performance_benchmark_example
   - tracy_profiling

## Contributing Examples

Have a cool example? Contributions are welcome!

1. Create a new file in `game_engine/examples/`
2. Follow the existing code style
3. Add comments explaining the concepts
4. Update this documentation
5. Submit a pull request

## Getting Help

- Check the inline comments in each example
- Review the [API documentation](./api_reference.md)
- Read the [guides](./guides/getting_started_guide.md)
- Ask questions on [GitHub Issues](https://github.com/yourusername/game_engine/issues)

Happy coding!
