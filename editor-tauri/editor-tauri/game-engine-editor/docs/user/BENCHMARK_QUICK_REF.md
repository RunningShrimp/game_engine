# Benchmark Quick Reference

## Quick Commands

```bash
# Run all benchmarks
./scripts/run_benchmarks.sh

# Run specific category
cargo bench --bench gpu_benchmark
cargo bench --bench editor_benchmark

# Save baseline
cargo bench -- --save-baseline main

# Compare with baseline
cargo bench -- --baseline main

# Generate flamegraph
cargo flamegraph --bench gpu_benchmark

# Check regressions
python3 scripts/check_regressions.py
```

## Benchmark Structure

```
benches/
├── gpu/                    # GPU rendering (4 files)
│   ├── culling_bench.rs
│   ├── indirect_draw_bench.rs
│   ├── vram_bench.rs
│   └── rendering_bench.rs
├── editor/                 # Editor ops (3 files)
│   ├── entity_crud_bench.rs
│   ├── undo_redo_bench.rs
│   └── material_bench.rs
├── performance/            # Components (1 file)
│   └── behavior_bench.rs
├── comprehensive/          # E2E tests (1 file)
│   └── full_scenario_bench.rs
└── memory/                 # Memory tests (placeholder)
```

## Performance Targets

| Category | Operation | Target |
|----------|-----------|--------|
| GPU | Frustum culling | >2x speedup |
| GPU | Draw call reduction | >60% |
| GPU | VRAM savings | >40% |
| Editor | Entity CRUD | <100μs |
| Editor | Undo/Redo | <1ms |
| Editor | Scene load (10K) | <1s |

## Critical Benchmarks (Regression Detection)

- entity_create
- entity_read
- undo_operations
- frustum_culling_cpu
- vram_allocation

## CI/CD Integration

**Workflow**: `.github/workflows/benchmark.yml`

**Triggers**:
- Push to main/master/develop
- Pull requests
- Manual dispatch

**Jobs**:
1. Run benchmarks (Ubuntu, macOS, Windows)
2. Compare with baseline
3. Generate flamegraphs

**Regression Thresholds**:
- Warning: >10% slower
- Critical: >20% slower

## Viewing Results

```bash
# HTML Report
open target/criterion/report/index.html

# Flamegraphs
ls target/flamegraph/*.svg

# Regression Report
cat benchmark_regressions.md
```

## Adding New Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_operation(c: &mut Criterion) {
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            my_operation(black_box(input))
        });
    });
}

criterion_group!(benches, bench_my_operation);
criterion_main!(benches);
```

## Troubleshooting

**Slow benchmarks**: Reduce measurement time or sample size
**Inconsistent results**: Close other apps, increase warm-up time
**Memory issues**: Reduce input sizes or run individually

## Resources

- Full Guide: `BENCHMARK_GUIDE.md`
- Implementation: `BENCHMARK_IMPLEMENTATION_REPORT.md`
- Criterion Docs: https://bheisler.github.io/criterion.rs/book/
