# Multi-threading

This document describes the multi-threading architecture.

## Overview

The engine uses multi-threading for parallel processing of game logic.

## Thread Architecture

```mermaid
graph TB
    Main[Main Thread] --> JobSystem[Job System]

    JobSystem --> T1[Thread 1]
    JobSystem --> T2[Thread 2]
    JobSystem --> T3[Thread 3]
    JobSystem --> T4[Thread 4]
```

## Thread Safety

- ECS: Thread-safe queries
- Resources: Atomic reference counting
- Rendering: Command buffers

## See Also

- [ADR-004: Concurrency Model](./adr/0004-concurrency-model.md)
- [Performance Optimization](./performance_tuning_guide.md)
