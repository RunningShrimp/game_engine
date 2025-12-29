# Networking API

This document describes the networking system API.

## Overview

The networking system enables multiplayer functionality.

## Key Components

### Server

```rust
use game_engine::network::Server;

let server = Server::bind("0.0.0.0:8080")?;
server.start()?;
```

### Client

```rust
use game_engine::network::Client;

let client = Client::connect("127.0.0.1:8080")?;
```

## See Also

- [Networking System](../networking_system.md)
- [Engine API](./engine.md)
