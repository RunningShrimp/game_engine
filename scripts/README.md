# Game Engine Development Scripts

This directory contains utility scripts for game engine development.

## Available Scripts

### Development Scripts

- `dev.sh` - Quick development environment setup
- `test.sh` - Run all tests
- `clean.sh` - Clean build artifacts
- `bench.sh` - Run benchmarks

### CI/CD Scripts

- `pre-commit-check.sh` - Pre-commit validation
- `ci-test.sh` - CI test runner
- `ci-doc.sh` - Documentation builder

## Usage

All scripts can be run from the repository root:

```bash
# Development
./scripts/dev.sh
./scripts/test.sh

# CI/CD
./scripts/ci-test.sh
./scripts/ci-doc.sh
```

## Adding New Scripts

When adding new scripts:
1. Make them executable: `chmod +x scripts/script-name.sh`
2. Add documentation to this README
3. Follow the naming convention: `verb-object.sh`
