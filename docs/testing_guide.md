# Testing Guide

This document provides testing guidelines and best practices.

## Testing Strategy

The engine uses multiple testing approaches:
- Unit tests for individual components
- Integration tests for subsystems
- Benchmarks for performance verification

## Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run benchmarks
cargo bench --workspace
```

## Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_creation() {
        let world = World::new();
        let entity = world.create_entity();
        assert!(entity.is_valid());
    }
}
```

## See Also

- [Test Coverage Baseline](./TEST_COVERAGE_BASELINE.md)
- [Benchmarking Guide](./benchmarking_guide.md)
