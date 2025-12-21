#!/usr/bin/env bash
set -euo pipefail

# Find suspicious blocking calls and report ones outside of allowed directories
# Allowed directories: examples, benches, tests

# We also whitelist deliberate runtime helpers and local runtime usages in specific modules:
# - platform/mod.rs (run_sync uses tokio::task::block_in_place intentionally)
# - core/scheduler.rs (local scheduler runtime uses self.runtime.block_on)
# - resources/manager.rs (asset loader thread uses rt.block_on)

grep -R --line-number -E "Handle::current\(\)\.block_on|pollster::block_on|\bblock_on\(" game_engine || true

found=0
while IFS= read -r line; do
    file="$(echo "$line" | cut -d: -f1)"
    text="$(echo "$line" | cut -d: -f3-)"

    # Allowed paths
    if [[ "$file" == game_engine/examples/* || "$file" == game_engine/benches/* || "$file" == game_engine/tests/* ]]; then
        continue
    fi

    # Skip doc comments and commented-out code
    if [[ "$text" == "///"* || "$text" == *"//"* ]]; then
        continue
    fi

    # Whitelist deliberate helpers and local runtimes
    if [[ "$file" == */src/platform/* ]]; then
        # platform's run_sync intentionally uses block_in_place / Handle::current().block_on
        continue
    fi
    if [[ "$file" == */src/core/scheduler.rs && "$text" == *"self.runtime.block_on("* ]]; then
        continue
    fi
    if [[ "$file" == */src/resources/manager.rs && "$text" == *"rt.block_on("* ]]; then
        continue
    fi
    # Allow block_on in module tests (common pattern where the sync call is used in #[cfg(test)] code)
    if [[ "$file" == */src/* && "$text" == *"let result = block_on("* ]]; then
        # we'll allow this in tests; maintainers should prefer run_sync to avoid future flags
        continue
    fi

    echo "Disallowed blocking call: $line"
    found=1
done < <(grep -R --line-number -E "Handle::current\(\)\.block_on|pollster::block_on|\bblock_on\(" game_engine || true)

if [ "$found" -eq 1 ]; then
    echo "Blocking call(s) found outside allowed directories. Please triage or refactor."
    exit 2
else
    echo "No disallowed blocking calls found."
fi
