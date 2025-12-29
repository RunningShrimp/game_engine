# Physics System

This document describes the physics system implementation.

## Overview

The physics system provides realistic physical simulation for game objects.

## Features

- Rigid body dynamics
- Collision detection
- Constraint solving
- Raycasting

## Usage

```rust
// Add physics to an entity
world.add_component(entity, RigidBody::dynamic());
world.add_component(entity, Collider::cuboid(1.0, 1.0, 1.0));
```

## See Also

- [Physics API](./api/physics.md)
- [Examples](./examples.md)
