# Physics API

This document describes the physics system API.

## Overview

The physics system provides realistic physical simulation.

## Key Components

### RigidBody

```rust
use game_engine::physics::RigidBody;

let body = RigidBody::dynamic();
world.add_component(entity, body);
```

### Collider

```rust
use game_engine::physics::Collider;

let collider = Collider::cuboid(1.0, 1.0, 1.0);
world.add_component(entity, collider);
```

## See Also

- [Physics System](../physics_system.md)
- [Engine API](./engine.md)
