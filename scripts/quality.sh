#!/bin/bash
# 代码质量检查脚本
#
# 运行所有代码质量检查：格式、Lint、测试

set -e

echo "🔍 运行代码质量检查..."

# 1. 代码格式检查
echo ""
echo "1️⃣ 检查代码格式..."
if cargo fmt -- --check; then
    echo "✅ 代码格式检查通过"
else
    echo "❌ 代码格式有问题！"
    echo "💡 运行 'cargo fmt' 修复格式"
    exit 1
fi

# 2. Clippy检查
echo ""
echo "2️⃣ 运行Clippy检查..."
if cargo clippy --workspace -- -D warnings; then
    echo "✅ Clippy检查通过"
else
    echo "❌ Clippy检查失败！"
    echo "💡 请修复Clippy警告"
    exit 1
fi

# 3. 单元测试
echo ""
echo "3️⃣ 运行单元测试..."
if cargo test --workspace --lib; then
    echo "✅ 单元测试通过"
else
    echo "❌ 单元测试失败！"
    exit 1
fi

# 4. 集成测试
echo ""
echo "4️⃣ 运行集成测试..."
if cargo test --workspace --test '*'; then
    echo "✅ 集成测试通过"
else
    echo "❌ 集成测试失败！"
    exit 1
fi

# 5. 文档测试
echo ""
echo "5️⃣ 运行文档测试..."
if cargo test --workspace --doc; then
    echo "✅ 文档测试通过"
else
    echo "❌ 文档测试失败！"
    exit 1
fi

# 6. 生成文档
echo ""
echo "6️⃣ 生成文档..."
if cargo doc --workspace --no-deps; then
    echo "✅ 文档生成成功"
else
    echo "❌ 文档生成失败！"
    exit 1
fi

echo ""
echo "🎉 所有代码质量检查通过！"
echo ""
echo "📊 检查总结:"
echo "  ✅ 代码格式正确"
echo "  ✅ Clippy检查通过"
echo "  ✅ 单元测试通过"
echo "  ✅ 集成测试通过"
echo "  ✅ 文档测试通过"
echo "  ✅ 文档生成成功"
