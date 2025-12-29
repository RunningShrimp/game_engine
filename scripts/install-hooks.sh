#!/bin/bash
# install-hooks.sh
# 安装pre-commit hooks
#
# 用法:
#   ./scripts/install-hooks.sh

set -e

echo "🔧 Installing pre-commit hooks..."

# 检查Python是否安装
if ! command -v python3 &> /dev/null; then
    echo "❌ Error: Python 3 is not installed"
    echo "Please install Python 3 first"
    exit 1
fi

# 安装pre-commit
echo "📦 Installing pre-commit..."
pip3 install pre-commit --upgrade

# 检查.pre-commit-config.yaml是否存在
if [ ! -f ".pre-commit-config.yaml" ]; then
    echo "❌ Error: .pre-commit-config.yaml not found"
    exit 1
fi

# 安装hooks
echo "🔗 Installing git hooks..."
pre-commit install

# 可选: 安装commit-msg hook (用于检查commit message格式)
# pre-commit install --hook-type commit-msg

echo "✅ Pre-commit hooks installed successfully!"
echo ""
echo "📝 Available commands:"
echo "  pre-commit run --all-files    # 运行所有检查"
echo "  pre-commit run --files FILE   # 运行特定文件的检查"
echo "  pre-commit autoupdate         # 更新hooks版本"
echo "  SKIP=id git commit            # 跳过特定hook"
echo ""
echo "🎯 Hooks will run automatically on each commit"
echo "   To skip hooks temporarily: git commit --no-verify"
