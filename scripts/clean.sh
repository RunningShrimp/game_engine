#!/bin/bash
# Clean build artifacts
#
# Usage: ./scripts/clean.sh

echo "🧹 Cleaning build artifacts..."

# Cargo clean
cargo clean

# Clean target directories
find . -type d -name "target" -exec rm -rf {} + 2>/dev/null || true

# Clean backup files
find . -name "*.bak" -delete 2>/dev/null || true
find . -name "*.backup" -delete 2>/dev/null || true

# Clean profiling data
find . -name "flamegraph.svg" -delete 2>/dev/null || true

echo "✅ Clean complete!"
