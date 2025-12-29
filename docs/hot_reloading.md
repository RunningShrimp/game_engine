# Hot Reloading

This document describes hot reloading capabilities for rapid development.

## Overview

Hot reloading allows you to modify assets and code without restarting the game.

## Supported Features

- Asset hot reloading (textures, models, audio)
- Shader hot reloading
- Script hot reloading

## Usage

```rust
// Enable hot reloading in dev mode
#[cfg(debug_assertions)]
engine.enable_hot_reloading(true);
```

## See Also

- [Resource Management](./api/resources.md)
- [Development Tools](./best_practices.md)
