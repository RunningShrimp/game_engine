# Networking System

This document describes the networking system for multiplayer games.

## Overview

The networking system provides client-server architecture for multiplayer gameplay.

## Features

- Client-server model
- State synchronization
- RPC system
- Entity replication
- Latency compensation

## Usage

```rust
// Create server
let server = NetworkServer::new(8080)?;

// Create client
let client = NetworkClient::connect("127.0.0.1:8080")?;
```

## See Also

- [Networking API](./api/networking.md)
- [Multiplayer Examples](./examples.md)
