#!/bin/bash
# Plugin Validation Script
# Validates plugin structure and metadata

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PLUGIN_PATH="$1"

if [ -z "$PLUGIN_PATH" ]; then
    echo -e "${RED}Error: Plugin path required${NC}"
    echo "Usage: $0 <plugin-path>"
    exit 1
fi

if [ ! -d "$PLUGIN_PATH" ]; then
    echo -e "${RED}Error: Directory not found: $PLUGIN_PATH${NC}"
    exit 1
fi

echo "Validating plugin at: $PLUGIN_PATH"
echo ""

# Error counter
ERRORS=0
WARNINGS=0

# Check function
check() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✓${NC} $2"
    else
        echo -e "${RED}✗${NC} $2"
        ERRORS=$((ERRORS + 1))
    fi
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
    WARNINGS=$((WARNINGS + 1))
}

# Determine plugin type
if [ -f "$PLUGIN_PATH/Cargo.toml" ]; then
    PLUGIN_TYPE="rust"
elif [ -f "$PLUGIN_PATH/package.json" ]; then
    PLUGIN_TYPE="typescript"
elif [ -f "$PLUGIN_PATH/plugin.lua" ]; then
    PLUGIN_TYPE="lua"
else
    PLUGIN_TYPE="unknown"
fi

echo "Plugin type: $PLUGIN_TYPE"
echo ""

# Rust plugin validation
if [ "$PLUGIN_TYPE" = "rust" ]; then
    check -d "$PLUGIN_PATH/src" "Source directory exists"

    if [ -f "$PLUGIN_PATH/src/lib.rs" ]; then
        check 1 -n "$(grep -c 'impl Plugin' "$PLUGIN_PATH/src/lib.rs" 2>/dev/null || true)" "Implements Plugin trait"

        if grep -q 'name()' "$PLUGIN_PATH/src/lib.rs"; then
            check 1 -n "$(grep -c 'fn name()' "$PLUGIN_PATH/src/lib.rs")" "Has name() method"
        else
            check 0 -n "" "Has name() method"
        fi

        if grep -q 'version()' "$PLUGIN_PATH/src/lib.rs"; then
            check 1 -n "$(grep -c 'fn version()' "$PLUGIN_PATH/src/lib.rs")" "Has version() method"
        else
            check 0 -n "" "Has version() method"
        fi

        if grep -q 'on_load()' "$PLUGIN_PATH/src/lib.rs"; then
            check 1 -n "$(grep -c 'fn on_load' "$PLUGIN_PATH/src/lib.rs")" "Has on_load() method"
        else
            check 0 -n "" "Has on_load() method"
        fi
    else
        check 0 -n "" "Has lib.rs"
    fi

    if grep -q 'crate-type = \["cdylib"\]' "$PLUGIN_PATH/Cargo.toml"; then
        check 1 -n "" "Cargo.toml configured for cdylib"
    else
        check 0 -n "" "Cargo.toml configured for cdylib"
    fi

    # Check for export_plugin macro
    if [ -f "$PLUGIN_PATH/src/lib.rs" ]; then
        if grep -q 'export_plugin!' "$PLUGIN_PATH/src/lib.rs"; then
            check 1 -n "" "Plugin is exported"
        else
            check 0 -n "" "Plugin is exported"
        fi
    fi
fi

# TypeScript plugin validation
if [ "$PLUGIN_TYPE" = "typescript" ]; then
    check -f "$PLUGIN_PATH/src/plugin.ts" "plugin.ts exists"
    check -f "$PLUGIN_PATH/package.json" "package.json exists"
    check -f "$PLUGIN_PATH/tsconfig.json" "tsconfig.json exists"

    if [ -f "$PLUGIN_PATH/src/plugin.ts" ]; then
        if grep -q 'registerPlugin' "$PLUGIN_PATH/src/plugin.ts"; then
            check 1 -n "" "Calls registerPlugin()"
        else
            check 0 -n "" "Calls registerPlugin()"
        fi
    fi
fi

# Lua plugin validation
if [ "$PLUGIN_TYPE" = "lua" ]; then
    check -f "$PLUGIN_PATH/plugin.lua" "plugin.lua exists"

    if [ -f "$PLUGIN_PATH/plugin.lua" ]; then
        if grep -q 'return plugin' "$PLUGIN_PATH/plugin.lua"; then
            check 1 -n "" "Returns plugin table"
        else
            check 0 -n "" "Returns plugin table"
        fi

        if grep -q 'on_load' "$PLUGIN_PATH/plugin.lua"; then
            check 1 -n "" "Has on_load function"
        else
            warn "Missing on_load function"
        fi
    fi
fi

# General checks
if [ -f "$PLUGIN_PATH/README.md" ]; then
    check 1 -n "" "Has README.md"
else
    warn "Missing README.md"
fi

if [ -f "$PLUGIN_PATH/plugin.toml" ]; then
    check 1 -n "" "Has plugin.toml"
else
    warn "Missing plugin.toml"
fi

echo ""
echo -e "${GREEN}Validation complete${NC}"
echo "Errors: $ERRORS"
echo "Warnings: $WARNINGS"

if [ $ERRORS -gt 0 ]; then
    exit 1
fi

exit 0
