# GPU Acceleration

This document describes GPU acceleration features.

## Overview

The engine offloads computation to GPU where possible.

## GPU Features

- Compute shaders for physics
- GPU particle systems
- Hardware skinning
- Compute-based AI

## Usage

```rust
// GPU-accelerated physics
physics.enable_gpu_acceleration(true);
```

## See Also

- [Rendering Pipeline](./rendering_pipeline.md)
- [Performance Optimization](./performance_tuning_guide.md)
