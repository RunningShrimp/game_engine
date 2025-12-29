# Resources API

This document describes the resource management system API.

## Overview

The resource system manages loading and caching of game assets.

## Key Functions

### Loading Resources

```rust
use game_engine::resources::ResourceManager;

let texture = engine.resources.load::<Texture>("player.png")?;
let model = engine.resources.load::<Model>("player.gltf")?;
```

### Getting Resources

```rust
if let Some(texture) = engine.resources.get::<Texture>("player.png") {
    // Use texture
}
```

## See Also

- [Engine API](./engine.md)
- [Resource Optimization Plan](../RESOURCE_OPTIMIZATION_PLAN.md)
