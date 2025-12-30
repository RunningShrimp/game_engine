#!/bin/bash
# Quick development environment setup
#
# Usage: ./scripts/dev.sh

set -e

echo "🚀 Starting development environment..."

# Run cargo watch in background
if command -v cargo-watch &> /dev/null; then
    echo "👀 Starting cargo watch..."
    cargo watch -x 'check --workspace' &
    WATCH_PID=$!
else
    echo "⚠️  cargo-watch not installed, skipping"
    WATCH_PID=""
fi

# Trap to kill background processes
trap "kill $WATCH_PID 2>/dev/null || true" EXIT

echo "✅ Development environment ready!"
echo "Press Ctrl+C to stop"

# If watch is running, wait for it
if [ -n "$WATCH_PID" ]; then
    wait $WATCH_PID
else
    echo "💡 Run 'cargo run' to start the game"
fi
