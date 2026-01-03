#!/bin/bash
# Plugin Generator Script
# Creates a new plugin from templates

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Templates directory
TEMPLATE_DIR="$(dirname "$0")/../templates"
SDK_ROOT="$(dirname "$0")/.."

# Print usage
usage() {
    echo "Usage: $0 [OPTIONS] <plugin-name>"
    echo ""
    echo "Options:"
    echo "  -t, --type TYPE      Plugin type: rust, wasm, typescript, lua (default: rust)"
    echo "  -d, --description DESC  Plugin description"
    echo "  -a, --author AUTHOR  Plugin author"
    echo "  -o, --output DIR     Output directory (default: current directory)"
    echo "  -h, --help           Show this help message"
    echo ""
    echo "Example:"
    echo "  $0 -t rust -d 'My awesome plugin' -a 'John Doe' my_plugin"
    exit 1
}

# Parse arguments
PLUGIN_TYPE="rust"
DESCRIPTION="A plugin for Game Engine Editor"
AUTHOR=""
OUTPUT_DIR="."

while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--type)
            PLUGIN_TYPE="$2"
            shift 2
            ;;
        -d|--description)
            DESCRIPTION="$2"
            shift 2
            ;;
        -a|--author)
            AUTHOR="$2"
            shift 2
            ;;
        -o|--output)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        -h|--help)
            usage
            ;;
        *)
            PLUGIN_NAME="$1"
            shift
            ;;
    esac
done

# Check plugin name
if [ -z "$PLUGIN_NAME" ]; then
    echo -e "${RED}Error: Plugin name is required${NC}"
    usage
fi

# Validate plugin type
case $PLUGIN_TYPE in
    rust|wasm|typescript|lua)
        ;;
    *)
        echo -e "${RED}Error: Invalid plugin type '$PLUGIN_TYPE'${NC}"
        echo "Valid types: rust, wasm, typescript, lua"
        exit 1
        ;;
esac

# Check if template exists
TEMPLATE_PATH="$TEMPLATE_DIR/$PLUGIN_TYPE"
if [ ! -d "$TEMPLATE_PATH" ]; then
    echo -e "${RED}Error: Template not found for type '$PLUGIN_TYPE'${NC}"
    exit 1
fi

# Create plugin directory
PLUGIN_DIR="$OUTPUT_DIR/$PLUGIN_NAME"
if [ -d "$PLUGIN_DIR" ]; then
    echo -e "${YELLOW}Warning: Directory '$PLUGIN_DIR' already exists${NC}"
    read -p "Continue? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    mkdir -p "$PLUGIN_DIR"
fi

echo -e "${GREEN}Creating plugin '$PLUGIN_NAME' (type: $PLUGIN_TYPE)...${NC}"

# Copy template files
case $PLUGIN_TYPE in
    rust)
        mkdir -p "$PLUGIN_DIR/src"
        cp "$TEMPLATE_PATH/Cargo.toml" "$PLUGIN_DIR/"
        cp "$TEMPLATE_PATH/README.md" "$PLUGIN_DIR/"
        cp "$TEMPLATE_PATH/plugin.toml" "$PLUGIN_DIR/"

        # Process lib.rs template
        sed -e "s/{{plugin-name}}/$PLUGIN_NAME/g" \
            -e "s/{{description}}/$DESCRIPTION/g" \
            -e "s/{{author}}/$AUTHOR/g" \
            -e "s/{{PluginStruct}}/$(echo $PLUGIN_NAME | sed -r 's/(^|_)([a-z])/\U\2/g')/g" \
            "$TEMPLATE_PATH/src/lib.rs" > "$PLUGIN_DIR/src/lib.rs"
        ;;

    wasm)
        mkdir -p "$PLUGIN_DIR/src"
        cp "$TEMPLATE_PATH/Cargo.toml" "$PLUGIN_DIR/"

        # Process lib.rs template
        sed -e "s/{{plugin-name}}/$PLUGIN_NAME/g" \
            -e "s/{{description}}/$DESCRIPTION/g" \
            -e "s/{{author}}/$AUTHOR/g" \
            "$TEMPLATE_PATH/src/lib.rs" > "$PLUGIN_DIR/src/lib.rs"
        ;;

    typescript)
        mkdir -p "$PLUGIN_DIR/src"
        cp "$TEMPLATE_PATH/package.json" "$PLUGIN_DIR/"
        cp "$TEMPLATE_PATH/tsconfig.json" "$PLUGIN_DIR/"

        # Process plugin.ts template
        sed -e "s/{{plugin-name}}/$PLUGIN_NAME/g" \
            -e "s/{{description}}/$DESCRIPTION/g" \
            -e "s/{{author}}/$AUTHOR/g" \
            "$TEMPLATE_PATH/src/plugin.ts" > "$PLUGIN_DIR/src/plugin.ts"
        ;;

    lua)
        cp "$TEMPLATE_PATH/plugin.toml" "$PLUGIN_DIR/"

        # Process plugin.lua template
        sed -e "s/{{plugin-name}}/$PLUGIN_NAME/g" \
            -e "s/{{description}}/$DESCRIPTION/g" \
            -e "s/{{author}}/$AUTHOR/g" \
            "$TEMPLATE_PATH/plugin.lua" > "$PLUGIN_DIR/plugin.lua"
        ;;
esac

echo -e "${GREEN}✓ Plugin created successfully!${NC}"
echo ""
echo "Location: $PLUGIN_DIR"
echo ""
echo "Next steps:"
case $PLUGIN_TYPE in
    rust)
        echo "  cd $PLUGIN_NAME"
        echo "  cargo build --release"
        ;;
    wasm)
        echo "  cd $PLUGIN_NAME"
        echo "  cargo build --release --target wasm32-unknown-unknown"
        ;;
    typescript)
        echo "  cd $PLUGIN_NAME"
        echo "  npm install"
        echo "  npm run build"
        ;;
    lua)
        echo "  # No build step required for Lua plugins"
        echo "  # Copy $PLUGIN_NAME/plugin.lua to your editor's plugins directory"
        ;;
esac

exit 0
