# Scripting API

This document describes the scripting system API.

## Overview

The scripting system allows runtime scripting of game logic.

## Supported Languages

- Lua (planned)
- Python (planned)
- Rust hot-reloading (experimental)

## Usage

```rust
use game_engine::scripting::Script;

let script = Script::from_file("player_logic.lua")?;
world.add_component(entity, script);
```

## See Also

- [Hot Reloading](../hot_reloading.md)
- [Engine API](./engine.md)
