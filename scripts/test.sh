#!/bin/bash
# Run all tests
#
# Usage: ./scripts/test.sh

set -e

echo "🧪 Running tests..."

# Unit tests
echo ""
echo "Running unit tests..."
cargo test --workspace --lib

# Integration tests
echo ""
echo "Running integration tests..."
cargo test --workspace --test '*'

# Doc tests
echo ""
echo "Running doc tests..."
cargo test --workspace --doc

echo ""
echo "✅ All tests passed!"
