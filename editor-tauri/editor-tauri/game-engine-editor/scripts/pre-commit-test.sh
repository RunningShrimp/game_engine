#!/bin/bash
# Pre-commit Test Script
# 在提交代码前运行快速检查

set -e

echo "🔍 Running pre-commit checks..."

# 格式检查
echo "📝 Checking code formatting..."
if ! cargo fmt -- --check; then
    echo "❌ Code not formatted. Run 'cargo fmt' to fix."
    exit 1
fi
echo "✅ Format check passed"

# 快速Clippy检查（不允许警告）
echo "🔍 Running Clippy..."
if ! cargo clippy --all-features -- -D warnings; then
    echo "❌ Clippy found issues. Please fix them before committing."
    exit 1
fi
echo "✅ Clippy check passed"

# 编译检查
echo "🔨 Checking compilation..."
if ! cargo check --all-features; then
    echo "❌ Compilation failed. Please fix the errors."
    exit 1
fi
echo "✅ Compilation check passed"

# 运行单元测试（快速）
echo "🧪 Running unit tests..."
if ! cargo test --lib --all-features; then
    echo "❌ Unit tests failed. Please fix them before committing."
    exit 1
fi
echo "✅ Unit tests passed"

echo ""
echo "✅ All pre-commit checks passed!"
echo "🎉 Ready to commit!"
