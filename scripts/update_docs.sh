#!/bin/bash
# 文档更新和预览脚本
# 此脚本生成API文档并启动本地预览服务器

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOCS_DIR="$PROJECT_ROOT/docs"

echo "=========================================="
echo "Documentation Update and Preview"
echo "=========================================="
echo ""

# 检查mdbook是否安装
if ! command -v mdbook &> /dev/null; then
    echo "❌ mdbook not found. Installing..."
    bash "$SCRIPT_DIR/setup-documentation.sh"
fi

echo "📚 Step 1: Generating Rust API documentation..."
echo ""

# 生成Rust API文档
cd "$PROJECT_ROOT"
cargo doc --workspace --no-deps --all-features

echo "✅ Rust API documentation generated"
echo ""

echo "📖 Step 2: Building mdBook site..."
echo ""

# 构建mdbook
cd "$DOCS_DIR"
mdbook build

echo "✅ mdBook site built successfully"
echo ""

echo "🚀 Step 3: Starting local preview server..."
echo ""
echo "Documentation will be available at:"
echo "  → http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# 启动预览服务器
mdbook serve --host 0.0.0.0 --port 3000 --open
