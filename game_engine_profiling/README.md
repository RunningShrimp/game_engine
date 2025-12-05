# Game Engine Profiling

Performance profiling and benchmarking tools for game engines.

## Features

- **Profiling**: Performance profiling tools for measuring and analyzing code performance
- **Benchmarking**: Benchmarking tools for comparing performance across different implementations
- **Monitoring**: System monitoring tools for tracking performance metrics
- **Visualization**: Performance visualization tools for displaying performance data
- **CI/CD**: CI/CD integration tools for automated performance testing

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
game_engine_profiling = { path = "../game_engine_profiling", version = "0.1.0" }
```

## Example

```rust
use game_engine_profiling::{Profiler, Benchmark};

// Create a profiler
let mut profiler = Profiler::new();
profiler.start_scope("my_function");
// ... do work ...
profiler.end_scope("my_function");

// Run a benchmark
let mut benchmark = Benchmark::new("my_benchmark");
benchmark.run(|| {
    // ... code to benchmark ...
});
```

## License

MIT OR Apache-2.0

