#!/bin/bash
# 文档服务器启动脚本
# 启动mdBook本地服务器用于预览文档

set -e

echo "📚 Game Engine Documentation Server"
echo "======================================"
echo ""

# 检查mdbook是否安装
if ! command -v mdbook &> /dev/null; then
    echo "❌ mdBook not found!"
    echo ""
    echo "Please install mdBook:"
    echo "  macOS:   brew install mdbook"
    echo "  Linux:   cargo install mdbook"
    echo "  Windows: cargo install mdbook"
    echo ""
    exit 1
fi

# 切换到docs目录
cd "$(dirname "$0")"
echo "📁 Working directory: $(pwd)"
echo ""

# 检查SUMMARY.md是否存在
if [ ! -f "SUMMARY.md" ]; then
    echo "❌ SUMMARY.md not found!"
    echo "Please run this script from the docs/ directory"
    exit 1
fi

echo "✅ Configuration found"
echo ""
echo "🚀 Starting mdBook server..."
echo ""
echo "Documentation will be available at: http://localhost:3000"
echo ""
echo "Press Ctrl+C to stop the server"
echo ""

# 启动mdBook服务器
mdbook serve --open
