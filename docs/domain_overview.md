# Domain-Driven Design Overview

This document provides an overview of Domain-Driven Design (DDD) principles as applied to the game engine.

## Overview

The engine uses Domain-Driven Design to separate business logic from infrastructure concerns.

## Key Concepts

### Domain Layer

The domain layer contains core business logic and domain models.

See [Domain-Driven Design ADR](./adr/0002-domain-driven-design.md) for detailed information.

### Bounded Contexts

The engine is organized into bounded contexts:
- Physics
- Rendering
- Audio
- Networking
- Gameplay

### Aggregates

Related domain objects are grouped into aggregates.

## Related Documentation

- [ADR-002: Domain-Driven Design](./adr/0002-domain-driven-design.md)
- [Domain Guide](./guides/getting_started_guide.md)
- [CQRS Pattern](./guides/cqrs_guide.md)
