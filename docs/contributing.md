# Contributing

Thank you for your interest in contributing to the game engine!

## How to Contribute

### Reporting Issues

1. Check existing issues
2. Create a new issue with:
   - Clear title
   - Detailed description
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details

### Submitting Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Update documentation
6. Submit a pull request

### Code Style

- Follow Rust naming conventions
- Use `cargo fmt` for formatting
- Pass `cargo clippy` checks
- Write tests for new features
- Document public APIs

### Documentation

- Update relevant docs
- Add examples for new features
- Update CHANGELOG.md
- Add inline comments

## Development Workflow

```bash
# 1. Fork and clone
git clone https://github.com/yourusername/game_engine.git
cd game_engine

# 2. Create branch
git checkout -b feature/my-feature

# 3. Make changes
# ... edit code ...

# 4. Format and check
cargo fmt
cargo clippy -- -D warnings

# 5. Test
cargo test --workspace
cargo build --release

# 6. Commit and push
git add .
git commit -m "Add my feature"
git push origin feature/my-feature

# 7. Create pull request
```

## Testing

```bash
# Run all tests
cargo test --workspace

# Run with coverage
cargo tarpaulin --workspace

# Run benchmarks
cargo bench --workspace
```

## Documentation

```bash
# Build documentation
cargo doc --workspace --open

# Check documentation
./scripts/check_docs.sh
```

## Project Structure

```
game_engine/
├── game_engine/     # Core engine library
├── examples/        # Example programs
├── docs/           # Documentation
├── scripts/        # Utility scripts
└── tests/          # Integration tests
```

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

## See Also

- [Code of Conduct](./CODE_OF_CONDUCT.md)
- [Development Guide](./best_practices.md)
- [Architecture](./architecture.md)
