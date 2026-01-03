# Performance Benchmark Suite Documentation

## Overview

This comprehensive benchmark suite measures the performance of the Game Engine Editor across multiple dimensions including GPU rendering, editor operations, performance monitoring, and memory usage.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Benchmark Categories](#benchmark-categories)
3. [Running Benchmarks](#running-benchmarks)
4. [Interpreting Results](#interpreting-results)
5. [Performance Targets](#performance-targets)
6. [CI/CD Integration](#cicd-integration)
7. [Adding New Benchmarks](#adding-new-benchmarks)

## Quick Start

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Criterion.rs and benchmarking tools
cargo install cargo-criterion
cargo install cargo-flamegraph
```

### Running All Benchmarks

```bash
cd benches
cargo bench
```

### Running Specific Benchmark Categories

```bash
# GPU benchmarks only
cargo bench --bench gpu_benchmark

# Editor benchmarks only
cargo bench --bench editor_benchmark

# Performance component benchmarks
cargo bench --bench performance_benchmark

# Memory benchmarks
cargo bench --bench memory_benchmark

# Comprehensive end-to-end benchmarks
cargo bench --bench comprehensive_benchmark
```

## Benchmark Categories

### 1. GPU Benchmarks (`gpu/`)

Tests GPU-related operations and rendering performance.

#### Culling Benchmarks
- **Frustum Culling (CPU vs GPU simulated)**
  - Tests: 1K, 5K, 10K, 50K instances
  - Metrics: Culling throughput, time per instance
  - Target: >2x speedup with GPU simulation

- **Occlusion Culling**
  - Tests: Depth buffer testing at various scene sizes
  - Metrics: Culling accuracy and performance

- **Combined Culling**
  - Tests: Frustum + occlusion culling together
  - Metrics: Overall culling efficiency

#### Indirect Drawing Benchmarks
- **Traditional vs Indirect Draw Calls**
  - Tests: 1K, 5K, 10K, 50K instances
  - Metrics: Draw call count reduction
  - Target: >60% reduction in draw calls

#### VRAM Management Benchmarks
- **Memory Allocation/Deallocation**
  - Tests: Various allocation patterns and sizes
  - Metrics: Allocation speed, fragmentation
  - Target: >40% memory savings

- **Memory Pooling**
  - Tests: Texture and mesh streaming patterns
  - Metrics: Memory utilization efficiency

- **Defragmentation**
  - Tests: Fragmented memory scenarios
  - Metrics: Defragmentation performance

#### Rendering Pipeline Benchmarks
- **Shadow Rendering**
  - Tests: 1, 4, 8, 16 light sources
  - Metrics: Shadow pass performance

- **Forward vs Deferred Rendering**
  - Tests: Different scene complexities
  - Metrics: Draw calls, vertex throughput

- **Post-Processing**
  - Tests: Bloom, TAA, SSAO, motion blur, etc.
  - Metrics: Per-effect performance cost

### 2. Editor Benchmarks (`editor/`)

Tests core editor operations and user interactions.

#### Entity CRUD Benchmarks
- **Entity Creation**
  - Tests: 100, 1K, 10K entities
  - Metrics: Creation time per entity
  - Target: <100μs per operation

- **Entity Read (by ID and Name)**
  - Tests: Various entity counts
  - Metrics: Lookup time

- **Entity Update**
  - Tests: Property updates at scale
  - Metrics: Update time per entity

- **Entity Deletion**
  - Tests: Bulk and individual deletions
  - Metrics: Deletion time

- **Search Operations**
  - Tests: Tag-based, name-based searches
  - Metrics: Search performance

#### Undo/Redo Benchmarks
- **Command Execution**
  - Tests: 10, 50, 100, 500, 1K commands
  - Metrics: Execution time

- **Undo Operations**
  - Tests: Various history sizes
  - Metrics: Undo time per command
  - Target: <1ms per operation

- **Redo Operations**
  - Tests: Various history sizes
  - Metrics: Redo time per command

- **Undo/Redo Cycles**
  - Tests: Full undo/redo cycles
  - Metrics: Total cycle time

- **Large History**
  - Tests: 100, 1K, 10K command history
  - Metrics: Memory overhead and performance

#### Material Editor Benchmarks
- **Material Creation**
  - Tests: 10, 50, 100, 500 materials
  - Metrics: Creation time

- **Property Updates**
  - Tests: Real-time property editing
  - Metrics: Update latency

- **Material Clone/Duplicate**
  - Tests: Material duplication performance
  - Metrics: Copy time

- **Shader Switching**
  - Tests: Changing material shaders
  - Metrics: Shader compilation time

### 3. Performance Component Benchmarks (`performance/`)

Tests performance-critical systems.

#### Behavior Tree Benchmarks
- **Tree Execution by Depth**
  - Tests: 5, 10, 15, 20 levels deep
  - Metrics: Execution time per frame

- **Tree Execution by Branching**
  - Tests: 2, 3, 4, 5 branches per node
  - Metrics: Node traversal performance

- **Multiple Trees**
  - Tests: 10, 50, 100, 500 concurrent trees
  - Metrics: Batch execution performance

#### Animation Benchmarks
- **Animation Evaluation**
  - Tests: Different clip lengths and bone counts
  - Metrics: Animation sampling time

- **Blending**
  - Tests: Multiple animation blending
  - Metrics: Blend operation cost

- **State Machine**
  - Tests: State transitions
  - Metrics: Transition performance

### 4. Memory Benchmarks (`memory/`)

Tests memory allocation patterns and efficiency.

#### Allocation Benchmarks
- **Entity Allocations**
  - Tests: Bulk entity creation
  - Metrics: Memory per entity

- **Asset Allocations**
  - Tests: Texture, mesh, material loading
  - Metrics: Memory footprint

#### Cache Benchmarks
- **Entity Cache**
  - Tests: Cache hit rates at different sizes
  - Metrics: Hit/miss ratios

- **Asset Cache**
  - Tests: LRU caching efficiency
  - Metrics: Cache performance

#### Memory Leak Detection
- **Long-running Sessions**
  - Tests: Extended operation periods
  - Metrics: Memory growth over time

### 5. Comprehensive Benchmarks (`comprehensive/`)

End-to-end scenarios combining multiple systems.

#### Scene Creation
- Tests: 1K, 5K, 10K, 50K entity scenes
- Metrics: Total creation time

#### Editing Sessions
- Tests: Mixed operations (create, edit, delete, undo/redo)
- Metrics: Session completion time

#### Full Workflow
- Tests: Complete editing workflow
- Metrics: End-to-end performance

## Running Benchmarks

### Basic Execution

```bash
# Run all benchmarks
cargo bench

# Run with verbose output
cargo bench -- --verbose

# Run with custom measurement time
cargo bench -- --measurement-time 30

# Run with custom sample size
cargo bench -- --sample-size 200
```

### Baseline Comparison

```bash
# Save current results as baseline
cargo bench -- --save-baseline main

# Compare against baseline
cargo bench -- --baseline main

# Compare with detailed output
cargo bench -- --baseline main --verbose
```

### Output Formats

```bash
# Generate HTML report (default)
cargo bench

# Generate quiet output (for CI/CD)
cargo bench -- --output-format quiet

# Save raw data
cargo bench -- --save-baseline custom_name
```

### Flamegraph Profiling

```bash
# Generate flamegraph for specific benchmark
cargo flamegraph --bench gpu_benchmark -- --profile-time 30

# Generate for all benchmarks
for bench in gpu editor performance memory comprehensive; do
    cargo flamegraph --bench ${bench}_benchmark -- --profile-time 30
done
```

## Interpreting Results

### Criterion.rs Output

Criterion generates several types of output:

1. **Command Line Output**
   - Summary of each benchmark
   - Mean execution time
   - Standard deviation
   - Comparison with baseline (if available)

2. **HTML Reports**
   - Located in `target/criterion/report/`
   - Interactive charts and graphs
   - Statistical analysis

3. **JSON Data**
   - Raw benchmark data for custom analysis
   - Located in `target/criterion/<benchmark>/`

### Key Metrics

- **Mean**: Average execution time
- **Std Dev**: Variability in measurements
- **Median**: Middle value (less affected by outliers)
- **Throughput**: Operations per second
- **Comparison**: % change from baseline

### Performance Regression Signs

⚠️ **Warning Signs**:
- >10% increase in execution time
- Increased standard deviation (instability)
- Memory usage growth over time
- Consistent slowdown in CI/CD

🔴 **Critical Regressions**:
- >20% performance degradation
- Failure to complete benchmarks
- Memory leaks detected

## Performance Targets

### GPU Performance

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| Frustum culling (50K instances) | <5ms | TBD | ⏳ |
| Indirect drawing (draw call reduction) | >60% | TBD | ⏳ |
| VRAM savings | >40% | TBD | ⏳ |
| Shadow rendering (4 lights) | <10ms | TBD | ⏳ |

### Editor Performance

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| Entity CRUD | <100μs | TBD | ⏳ |
| Undo/Redo | <1ms | TBD | ⏳ |
| Material update | <10ms | TBD | ⏳ |
| Scene load (10K entities) | <1s | TBD | ⏳ |

### Memory Usage

| Component | Target | Current | Status |
|-----------|--------|---------|--------|
| Per-entity overhead | <1KB | TBD | ⏳ |
| Memory leaks | None | TBD | ⏳ |
| Cache efficiency | >80% hit rate | TBD | ⏳ |

## CI/CD Integration

### GitHub Actions Workflow

The benchmark suite integrates with GitHub Actions via `.github/workflows/benchmark.yml`:

**Triggers**:
- Push to main/master/develop
- Pull requests
- Manual workflow dispatch

**Jobs**:
1. **Benchmark** - Runs all benchmarks on Ubuntu, macOS, Windows
2. **Compare** - Compares PR results with baseline
3. **Flamegraph** - Generates performance flamegraphs

**Automatic Regression Detection**:
- Checks for >10% performance regression
- Posts results as PR comments
- Blocks critical regressions

### Local Testing Before Commit

```bash
# Run full benchmark suite
./scripts/run_benchmarks.sh

# Check for regressions
python3 scripts/check_regressions.py
```

## Adding New Benchmarks

### Template

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_my_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_operation");
    group.measurement_time(std::time::Duration::from_secs(10));

    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &n| {
                let data = create_test_data(n);
                b.iter(|| {
                    my_operation(black_box(&data))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_my_operation);
criterion_main!(benches);
```

### Best Practices

1. **Use `black_box`** to prevent compiler optimization
2. **Set appropriate measurement time** (5-15 seconds typically)
3. **Test multiple input sizes** for scalability analysis
4. **Include warm-up** for JIT-compiled code
5. **Document performance goals** in benchmark comments
6. **Use descriptive names** for benchmark identification
7. **Group related benchmarks** together
8. **Include unit tests** for benchmark fixtures

## Troubleshooting

### Benchmarks are too slow

- Reduce `measurement_time`
- Reduce `sample_size`
- Test with smaller input sizes

### Results are inconsistent

- Close other applications
- Use `--warm-up-time` to stabilize
- Increase sample size for better statistics
- Run on a dedicated machine

### Memory issues

- Reduce input sizes
- Run benchmarks individually
- Check for memory leaks with Valgrind/Sanitizers

## Additional Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Flamegraph Guide](https://github.com/flamegraph-rs/flamegraph)

## Contributing

When adding new benchmarks:

1. Follow the existing structure and naming conventions
2. Document performance expectations
3. Update this README with new benchmarks
4. Add performance targets to the tables above
5. Test on multiple platforms if possible
6. Include regression checks in CI/CD

---

**Last Updated**: 2025-01-02
**Version**: 1.0.0
