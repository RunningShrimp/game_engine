#!/bin/bash
# mdbook安装和配置脚本
# 此脚本安装mdbook及其扩展，用于生成文档网站

set -e

echo "=========================================="
echo "Game Engine Documentation Setup"
echo "=========================================="
echo ""

# 检查是否安装了Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo not found. Please install Rust first:"
    echo "   https://rustup.rs/"
    exit 1
fi

echo "✅ Rust/Cargo found"
echo ""

# 安装mdbook核心
echo "📦 Installing mdbook..."
if ! command -v mdbook &> /dev/null; then
    cargo install mdbook
    echo "✅ mdbook installed successfully"
else
    echo "✅ mdbook already installed"
fi
echo ""

# 安装扩展（可选但推荐）
echo "📦 Installing mdbook extensions..."

# mdbook-linkcheck - 检查链接有效性
echo "  - mdbook-linkcheck (link checker)..."
if ! mdbook-linkcheck --version &> /dev/null 2>&1; then
    cargo install mdbook-linkcheck
else
    echo "    Already installed"
fi

# mdbook-catppuccin - 漂亮的主题
echo "  - mdbook-catppuccin (theme)..."
if ! mdbook-catppuccin --version &> /dev/null 2>&1; then
    cargo install mdbook-catppuccin
else
    echo "    Already installed"
fi

# mdbook-mermaid - 图表支持
echo "  - mdbook-mermaid (diagrams)..."
if ! mdbook-mermaid --version &> /dev/null 2>&1; then
    cargo install mdbook-mermaid
else
    echo "    Already installed"
fi

# mdbook-toc - 自动目录
echo "  - mdbook-toc (table of contents)..."
if ! mdbook-toc --version &> /dev/null 2>&1; then
    cargo install mdbook-toc
else
    echo "    Already installed"
fi

echo ""
echo "=========================================="
echo "✅ Installation complete!"
echo "=========================================="
echo ""
echo "Next steps:"
echo "  1. Build documentation:  cd docs && mdbook build"
echo "  2. Serve locally:        cd docs && mdbook serve"
echo "  3. Open browser:         http://localhost:3000"
echo ""
echo "For more information, see:"
echo "  - docs/book.toml (configuration)"
echo "  - docs/SUMMARY.md (table of contents)"
echo ""
