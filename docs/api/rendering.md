# Rendering API

This document describes the rendering system API.

## Overview

The rendering system handles all graphics operations.

## Key Components

### Renderer

```rust
use game_engine::render::Renderer;

let renderer = &mut engine.renderer;
renderer.set_clear_color(Color::new(0.1, 0.1, 0.1, 1.0));
```

### Camera

```rust
use game_engine::Camera;

let camera = Camera::new();
camera.set_position(Vector3::new(0.0, 5.0, 10.0));
camera.look_at(Vector3::new(0.0, 0.0, 0.0));
```

### Light

```rust
use game_engine::Light;

let light = Light::directional(Vector3::new(1.0, -1.0, -1.0));
```

## See Also

- [Rendering Pipeline](../rendering_pipeline.md)
- [Engine API](./engine.md)
