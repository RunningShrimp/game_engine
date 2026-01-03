# Contributing to Game Engine

Thank you for your interest in contributing to Game Engine! This guide will help you get started.

---

## 🚀 Quick Start

1. **Fork the repository** on GitHub
2. **Clone your fork** locally
3. **Set up development environment**
4. **Make your changes**
5. **Test thoroughly**
6. **Submit a pull request**

---

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Documentation](#documentation)
- [Pull Request Process](#pull-request-process)
- [Community Guidelines](#community-guidelines)

---

## 🤝 Code of Conduct

### Our Pledge

We are committed to making participation in our project a harassment-free experience for everyone, regardless of level of experience, gender, gender identity and expression, sexual orientation, disability, personal appearance, body size, race, ethnicity, age, religion, or nationality.

### Our Standards

**Positive behavior includes**:
- Using welcoming and inclusive language
- Being respectful of differing viewpoints and experiences
- Gracefully accepting constructive criticism
- Focusing on what is best for the community
- Showing empathy towards other community members

**Unacceptable behavior includes**:
- The use of sexualized language or imagery
- Trolling, insulting/derogatory comments, or personal/political attacks
- Public or private harassment
- Publishing others' private information (e.g., physical or electronic address) without explicit permission
- Other unethical or unprofessional conduct

### Enforcement

Project maintainers have the right and responsibility to remove, edit, or reject comments, commits, code, wiki edits, issues, and other contributions that are not aligned with this Code of Conduct.

---

## 🛠️ Getting Started

### Prerequisites

**Required**:
- **Rust**: 1.70 or later ([Install Rust](https://www.rust-lang.org/tools/install))
- **Git**: For version control
- **Cargo**: Comes with Rust

**Optional but Recommended**:
- **.NET SDK 8.0**: For C# scripting development
- **VS Code**: For development with LSP support
- **Node.js 18+**: For web platform development

### Fork and Clone

1. **Fork the repository**:
   - Go to https://github.com/game-engine/game-engine
   - Click "Fork" button in top-right corner

2. **Clone your fork**:
   ```bash
   git clone https://github.com/YOUR_USERNAME/game-engine.git
   cd game-engine
   ```

3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/game-engine/game-engine.git
   ```

### Development Setup

1. **Install dependencies**:
   ```bash
   # Install Rust toolchain
   rustup install stable
   rustup default stable

   # Install development tools
   cargo install cargo-watch
   cargo install cargo-edit
   ```

2. **Build the project**:
   ```bash
   cargo build --release
   ```

3. **Run tests**:
   ```bash
   cargo test --all
   ```

4. **Install CLI tools**:
   ```bash
   cargo install --path cli
   ```

### Development Environment

#### VS Code Setup

1. **Install VS Code**:
   - Download from [https://code.visualstudio.com/](https://code.visualstudio.com/)

2. **Install extensions**:
   - rust-analyzer (Rust language server)
   - CodeLLDB (debugger)
   - Even Better TOML (for Cargo.toml)

3. **Configure settings** (`.vscode/settings.json`):
   ```json
   {
     "rust-analyzer.cargo.features": "all",
     "rust-analyzer.checkOnSave.command": "clippy",
     "editor.formatOnSave": true,
     "editor.defaultFormatter": "rust-lang.rust-analyzer"
   }
   ```

---

## 🔄 Development Workflow

### Branch Strategy

We use a simplified Git flow:

- **`master`**: Main development branch (always stable)
- **`feature/*`**: Feature branches
- **`bugfix/*`**: Bug fix branches
- **`hotfix/*`**: Urgent production fixes
- **`release/*`**: Release preparation branches

### Creating a Branch

1. **Sync with upstream**:
   ```bash
   git fetch upstream
   git checkout master
   git merge upstream/master
   ```

2. **Create your branch**:
   ```bash
   # For new features
   git checkout -b feature/your-feature-name

   # For bug fixes
   git checkout -b bugfix/issue-number-description

   # For hotfixes
   git checkout -b hotfix/critical-fix-description
   ```

### Making Changes

1. **Write code following our standards** (see [Coding Standards](#coding-standards))

2. **Test your changes**:
   ```bash
   # Run all tests
   cargo test --all

   # Run specific test
   cargo test test_name

   # Run with output
   cargo test -- --nocapture

   # Watch mode (development)
   cargo watch -x test
   ```

3. **Format code**:
   ```bash
   # Format all code
   cargo fmt --all

   # Check formatting
   cargo fmt --all -- --check
   ```

4. **Run linter**:
   ```bash
   # Check for issues
   cargo clippy --all --all-targets -- -D warnings

   # Fix issues automatically
   cargo clippy --all --all-targets --fix
   ```

5. **Build successfully**:
   ```bash
   cargo build --release
   ```

### Committing Changes

1. **Stage your changes**:
   ```bash
   git add .
   # Or selectively
   git add path/to/file.rs
   ```

2. **Write a good commit message**:
   ```bash
   git commit -m "feat: add NavMesh pathfinding system

   - Implement A* algorithm with heap-based priority queue
   - Add path smoothing and string pulling
   - Support parallel pathfinding (4-8x speedup)
   - Build time <5ms for 1000 nodes

   Fixes #123
   "
   ```

3. **Commit message format**:
   ```
   <type>(<scope>): <subject>

   <body>

   <footer>
   ```

   **Types**:
   - `feat`: New feature
   - `fix`: Bug fix
   - `docs`: Documentation changes
   - `style`: Code style changes (formatting)
   - `refactor`: Code refactoring
   - `perf`: Performance improvements
   - `test`: Test additions/changes
   - `chore`: Build process or tooling changes
   - `ci`: CI/CD changes

### Syncing with Upstream

Keep your branch up-to-date with upstream:

```bash
# Fetch upstream changes
git fetch upstream

# Rebase your branch on top of upstream/master
git rebase upstream/master

# Or merge (if you prefer)
git merge upstream/master
```

---

## 📐 Coding Standards

### Rust Code Style

#### Naming Conventions

```rust
// Modules: snake_case
mod physics_system;

// Types: PascalCase
struct GameObject;
enum GameState;

// Functions: snake_case
fn calculate_physics() {}

// Variables: snake_case
let player_position = Vector3::new(0.0, 0.0, 0.0);

// Constants: SCREAMING_SNAKE_CASE
const MAX_PLAYERS: usize = 100;

// Lifetime parameters: short, lowercase
fn parse<'a>(input: &'a str) -> &'a str {}
```

#### Code Organization

```rust
// 1. Imports (grouped and sorted)
use std::collections::HashMap;
use crate::physics::{PhysicsEngine, RigidBody};

// 2. Types and structs
pub struct PlayerController {
    position: Vector3,
    velocity: Vector3,
}

// 3. Implementation
impl PlayerController {
    // 3a. Constructor
    pub fn new() -> Self {
        Self {
            position: Vector3::zero(),
            velocity: Vector3::zero(),
        }
    }

    // 3b. Public methods
    pub fn update(&mut self, dt: f32) {
        // ...
    }

    // 3c. Private methods
    fn calculate_movement(&self) -> Vector3 {
        // ...
    }
}

// 4. Trait implementations
impl Default for PlayerController {
    fn default() -> Self {
        Self::new()
    }
}

// 5. Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_movement() {
        // ...
    }
}
```

#### Error Handling

```rust
// Use Result for fallible operations
pub fn load_texture(path: &str) -> Result<Texture, LoadError> {
    let data = std::fs::read(path)
        .map_err(|e| LoadError::IoError(path.to_string(), e))?;

    Texture::from_data(&data)
}

// Use Option for optional values
pub fn find_entity(id: u32) -> Option<Entity> {
    entities.get(&id).copied()
}

// Use ? operator for propagation
pub fn load_scene(path: &str) -> Result<Scene, LoadError> {
    let data = std::fs::read_to_string(path)?;
    let scene: Scene = serde_json::from_str(&data)?;
    Ok(scene)
}
```

#### Documentation

```rust
//! # Game Engine Core Module
//!
//! This module provides the core functionality for the game engine.

/// Represents a 3D vector with x, y, z components.
///
/// # Examples
///
/// ```
/// use game_engine::math::Vector3;
///
/// let v = Vector3::new(1.0, 2.0, 3.0);
/// assert_eq!(v.magnitude(), 3.742);
/// ```
///
/// # Panics
///
/// Panics if used in a context that requires a normalized vector
/// when the vector is zero-length.
///
/// # Errors
///
/// This function does not return errors, but related functions
/// may return [`MathError`] for invalid operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    /// Creates a new Vector3 with the given components.
    ///
    /// # Arguments
    ///
    /// * `x` - The x component
    /// * `y` - The y component
    /// * `z` - The z component
    ///
    /// # Examples
    ///
    /// ```
    /// # use game_engine::math::Vector3;
    /// let v = Vector3::new(1.0, 2.0, 3.0);
    /// ```
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    /// Calculates the magnitude (length) of the vector.
    ///
    /// # Returns
    ///
    /// The magnitude as a `f32` value.
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
}
```

#### Performance Guidelines

```rust
// ✅ Good: Use references for large types
pub fn process_mesh(mesh: &Mesh) -> ProcessedMesh {
    // ...
}

// ❌ Bad: Unnecessary copy
pub fn process_mesh(mesh: Mesh) -> ProcessedMesh {
    // ...
}

// ✅ Good: Use iterators
let sum: i32 = numbers.iter().sum();

// ❌ Bad: Imperative loop
let mut sum = 0;
for n in &numbers {
    sum += n;
}

// ✅ Good: Use with_capacity for collections
let mut entities = Vec::with_capacity(1000);

// ❌ Bad: Let it grow dynamically
let mut entities = Vec::new();

// ✅ Good: Use efficient data structures
use std::collections::HashMap;
let map = HashMap::with_capacity(100);

// ❌ Bad: Use Vec for lookups
let pairs: Vec<(String, i32)> = Vec::new();
```

#### Unsafe Code

```rust
// Avoid unsafe code when possible
// If used, document why it's safe

/// # Safety
///
/// This function is safe because:
/// 1. The pointer is guaranteed to be valid for the lifetime 'a
/// 2. The memory is aligned for type T
/// 3. No mutable references exist while this reference is alive
pub unsafe fn from_raw_ptr<'a, T>(ptr: *const T) -> &'a T {
    &*ptr
}

// Or use safe wrappers
pub fn from_raw_ptr_safe<T>(ptr: *const T) -> Option<&T> {
    if ptr.is_null() {
        None
    } else {
        unsafe { Some(&*ptr) }
    }
}
```

### C# Code Style

If contributing C# code:

```csharp
// Naming conventions
public class PlayerController  // PascalCase for classes
{
    private int _health;        // _camelCase for private fields
    public int MaxHealth { get; set; }  // PascalCase for properties
    public void Update() {}     // PascalCase for methods
    const int MaxPlayers = 100; // PascalCase for constants
}

// XML documentation
/// <summary>
/// Controls player movement and input.
/// </summary>
/// <example>
/// <code>
/// var player = new PlayerController();
/// player.Update();
/// </code>
/// </example>
public class PlayerController
{
    /// <summary>
    /// Updates the player state.
    /// </summary>
    /// <param name="deltaTime">Time since last frame in seconds.</param>
    public void Update(float deltaTime)
    {
        // Implementation
    }
}
```

---

## 🧪 Testing Guidelines

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;

    // Unit tests
    #[test]
    fn test_vector_addition() {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        let result = v1 + v2;
        assert_eq!(result, Vector3::new(5.0, 7.0, 9.0));
    }

    // Property-based tests
    #[test]
    fn test_vector_commutative() {
        let v1 = Vector3::random();
        let v2 = Vector3::random();
        assert_eq!(v1 + v2, v2 + v1);
    }

    // Error cases
    #[test]
    fn test_invalid_input_returns_error() {
        let result = parse_texture("");
        assert!(matches!(result, Err(ParseError::EmptyInput)));
    }

    // Async tests
    #[tokio::test]
    async fn test_network_connection() {
        let server = start_test_server().await;
        let client = connect_to_server(&server.addr).await;
        assert!(client.is_connected());
    }

    // Benchmarks (in benches/ directory)
    #[bench]
    fn bench_vector_addition(b: &mut Bencher) {
        let v1 = Vector3::new(1.0, 2.0, 3.0);
        let v2 = Vector3::new(4.0, 5.0, 6.0);
        b.iter(|| v1 + v2);
    }
}
```

### Testing Best Practices

1. **Write tests first** (Test-Driven Development):
   ```rust
   #[test]
   fn test_player_health_reduces_on_damage() {
       let mut player = Player::new(100);
       player.take_damage(10);
       assert_eq!(player.health(), 90);
   }
   ```

2. **Test edge cases**:
   ```rust
   #[test]
   fn test_zero_health() {
       let player = Player::new(0);
       assert!(!player.is_alive());
   }

   #[test]
   fn test_negative_damage() {
       let mut player = Player::new(100);
       player.take_damage(-10);
       assert_eq!(player.health(), 100); // Should not increase
   }
   ```

3. **Use test helpers**:
   ```rust
   fn create_test_player() -> Player {
       Player::new(100)
           .with_weapon(Weapon::Pistol)
           .with_position(Vector3::new(0.0, 0.0, 0.0))
   }

   #[test]
   fn test_player_firing() {
       let player = create_test_player();
       player.fire();
       assert!(player.has_fired());
   }
   ```

4. **Integration tests** (in `tests/` directory):
   ```rust
   // tests/integration/game_loop_test.rs
   use game_engine::prelude::*;

   #[test]
   fn test_complete_game_loop() {
       let mut engine = GameEngine::new();
       let scene = create_test_scene();
       engine.run_scene_once(scene);
       // Assertions...
   }
   ```

5. **Performance tests**:
   ```rust
   #[test]
   fn test_physics_performance() {
       let start = Instant::now();
       simulate_physics(10000);
       let duration = start.elapsed();
       assert!(duration.as_millis() < 16); // < 16ms (60 FPS)
   }
   ```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_vector_addition

# Run tests in a specific crate
cargo test -p game_engine

# Run integration tests only
cargo test --test integration

# Run ignored tests
cargo test -- --ignored

# Run tests with specific filter
cargo test physics

# Run benchmarks
cargo test --release --benches
```

---

## 📚 Documentation

### Code Documentation

Every public API must have documentation:

```rust
/// Renders a mesh to the screen.
///
/// This function takes a mesh and renders it using the current
/// rendering pipeline and camera settings.
///
/// # Arguments
///
/// * `mesh` - The mesh to render
/// * `transform` - The transform (position, rotation, scale) to apply
/// * `material` - The material to use for rendering
///
/// # Returns
///
/// Returns `Ok(())` if rendering succeeded, or an error if the
/// mesh could not be rendered.
///
/// # Errors
///
/// This function will return an error if:
/// - The mesh has no vertices
/// - The material shader is invalid
/// - The graphics device is lost
///
/// # Examples
///
/// ```
/// use game_engine::render::*;
///
/// # fn render_example(mesh: Mesh, transform: Transform, material: Material) -> Result<(), RenderError> {
/// render_mesh(&mesh, &transform, &material)?;
/// # Ok(())
/// # }
/// ```
///
/// # Performance
///
/// This function is optimized for batch rendering. When rendering
/// multiple meshes with the same material, use [`render_mesh_batch`]
/// for better performance.
///
/// # See Also
///
/// - [`render_mesh_batch`] - Batch rendering
/// - [`Mesh`] - Mesh structure
/// - [`Material`] - Material system
pub fn render_mesh(
    mesh: &Mesh,
    transform: &Transform,
    material: &Material
) -> Result<(), RenderError> {
    // Implementation...
}
```

### README Documentation

Keep README.md files up-to-date:
- Project overview
- Installation instructions
- Quick start guide
- Basic usage examples
- Links to full documentation

### API Documentation

Generate and check API docs:

```bash
# Generate documentation
cargo doc --all --no-deps --open

# Check documentation coverage
cargo doc --all --no-deps
```

### Examples

Provide runnable examples:

```rust
// In examples/basic_rendering.rs
use game_engine::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create engine
    let mut engine = GameEngine::new();

    // Create scene
    let mut scene = Scene::new("Basic Scene");

    // Add cube
    let cube = Entity::new("Cube");
    cube.add_component(Mesh::cube());
    cube.add_component(Material::default());
    scene.add_entity(cube);

    // Run
    engine.run(scene)?;

    Ok(())
}
```

Run examples:
```bash
cargo run --example basic_rendering
```

---

## 🔀 Pull Request Process

### Before Submitting

1. **Check all requirements**:
   - [ ] Code follows style guidelines
   - [ ] All tests pass
   - [ ] Documentation updated
   - [ ] Commit messages follow convention
   - [ ] No merge conflicts with master
   - [ ] Changes are minimal and focused

2. **Update documentation**:
   - API docs for new functions
   - README for new features
   - Changelog for user-facing changes

3. **Add tests**:
   - Unit tests for new functionality
   - Integration tests for workflows
   - Update existing tests if needed

### Creating a Pull Request

1. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

2. **Create PR on GitHub**:
   - Go to your fork on GitHub
   - Click "Pull Request" button
   - Base repository: `game-engine/game-engine`
   - Base: `master`
   - Compare: `your-username:feature/your-feature-name`
   - Click "Create pull request"

3. **PR title format**:
   ```
   feat: add NavMesh pathfinding system
   fix: resolve memory leak in texture cache
   docs: update API documentation for rendering
   ```

4. **PR description template**:
   ```markdown
   ## Summary
   Brief description of changes (1-2 sentences)

   ## Changes
   - [ ] Added NavMesh generation
   - [ ] Implemented A* pathfinding
   - [ ] Added path smoothing
   - [ ] Performance optimizations

   ## Testing
   - [ ] Unit tests pass
   - [ ] Integration tests pass
   - [ ] Manual testing completed

   ## Documentation
   - [ ] API docs updated
   - [ ] Examples added
   - [ ] README updated

   ## Checklist
   - [ ] Code follows style guidelines
   - [ ] Self-review completed
   - [ ] Comments added to complex code
   - [ ] Documentation updated
   - [ ] No new warnings generated
   - [ ] Tests added/updated
   - [ ] All tests pass
   - [ ] PR description complete

   ## Issues
   Fixes #123
   Related to #456

   ## Screenshots (if applicable)
   [Add screenshots/gifs for UI changes]

   ## Performance Impact
   - Before: 10ms per frame
   - After: 5ms per frame (50% improvement)
   ```

### PR Review Process

1. **Automated checks**:
   - CI builds must pass
   - All tests must pass
   - Code coverage must not decrease

2. **Code review**:
   - At least one maintainer approval required
   - Address all review comments
   - Make requested changes

3. **Testing**:
   - Maintainers test your changes
   - Provide feedback or approval

4. **Merge**:
   - Squash and merge to master
   - Delete branch after merge

### After Merge

1. **Delete your branch**:
   ```bash
   git branch -d feature/your-feature-name
   git push origin --delete feature/your-feature-name
   ```

2. **Sync with upstream**:
   ```bash
   git fetch upstream
   git checkout master
   git merge upstream/master
   ```

3. **Celebrate!** 🎉

---

## 👥 Community Guidelines

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: General questions and ideas
- **Discord**: Real-time chat and community support
- **Forum**: In-depth technical discussions

### Asking Questions

1. **Search first**:
   - Check existing issues
   - Read documentation
   - Search discussions

2. **Provide context**:
   - Engine version
   - OS and platform
   - Error messages
   - Minimal reproduction case

3. **Use appropriate channel**:
   - Bug? → GitHub Issue
   - Question? → GitHub Discussion or Discord
   - Feature request? → GitHub Discussion

### Reporting Issues

Use the issue template:

```markdown
## Bug Description
Clear description of the bug

## Reproduction Steps
1. Step 1
2. Step 2
3. Step 3

## Expected Behavior
What should happen

## Actual Behavior
What actually happens

## Environment
- Engine version: 0.3.0
- OS: macOS 14.0
- Rust version: 1.70.0

## Additional Context
Logs, screenshots, etc.
```

### Feature Requests

```markdown
## Feature Description
What feature you want

## Use Case
Why you need it and how you'd use it

## Proposed Solution
How you think it should work

## Alternatives Considered
Other approaches you thought about

## Additional Context
Examples, references, etc.
```

---

## 🎯 Good First Issues

Looking for something to work on? Check these labels:

- **good first issue**: Good for newcomers
- **help wanted**: Maintainers need help
- **documentation**: Documentation improvements
- **performance**: Performance optimizations

### Contribution Ideas

1. **Documentation**:
   - Improve API docs
   - Add more examples
   - Write tutorials

2. **Tests**:
   - Increase test coverage
   - Add integration tests
   - Add benchmarks

3. **Features**:
   - Implement pending features
   - Add new components
   - Enhance existing systems

4. **Performance**:
   - Profile and optimize
   - Reduce memory usage
   - Improve algorithms

5. **Tools**:
   - Improve CLI tools
   - Enhance editor
   - Add debugging utilities

---

## 📜 License

By contributing, you agree that your contributions will be licensed under the **MIT License**.

---

## 🆘 Getting Help

If you need help:

1. **Read documentation**: Check docs/ directory
2. **Search issues**: Look for similar problems
3. **Ask on Discord**: Get real-time help
4. **Create discussion**: Ask questions on GitHub
5. **Contact maintainers**: For urgent issues

---

## 🙏 Thank You!

Thank you for contributing to Game Engine! Every contribution helps make the engine better for everyone.

**Together we're building an amazing game engine!** 🎮✨

---

*This document is maintained by the Game Engine team. Last updated: 2026-01-03*

---

**Quick Links**:
- [Documentation](docs/)
- [API Reference](docs/api/)
- [Issues](https://github.com/game-engine/game-engine/issues)
- [Discussions](https://github.com/game-engine/game-engine/discussions)
- [Discord](https://discord.gg/game-engine)

---

*Generated with [Claude Code](https://claude.com/claude-code)*
*Co-Authored-By: Claude Sonnet 4 <noreply@anthropic.com>*