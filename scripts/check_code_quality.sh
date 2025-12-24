#!/bin/bash
# 代码质量检查脚本
# 运行格式检查、clippy检查和自定义检查

set -e

echo "🔍 开始代码质量检查..."
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查结果
FORMAT_OK=true
CLIPPY_OK=true
CUSTOM_OK=true

# 1. 格式检查
echo "📝 检查代码格式..."
if cargo fmt --check --all 2>&1 | grep -q "Diff in"; then
    echo -e "${RED}❌ 代码格式不符合规范${NC}"
    echo "运行 'cargo fmt --all' 来修复格式问题"
    FORMAT_OK=false
else
    echo -e "${GREEN}✅ 代码格式检查通过${NC}"
fi
echo ""

# 2. Clippy检查
echo "🔧 运行 Clippy 检查..."
if cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee /tmp/clippy_output.txt | grep -q "error\|warning:"; then
    echo -e "${YELLOW}⚠️  Clippy 发现了一些问题${NC}"
    echo "查看 /tmp/clippy_output.txt 获取详细信息"
    CLIPPY_OK=false
else
    echo -e "${GREEN}✅ Clippy 检查通过${NC}"
fi
echo ""

# 3. 检查未使用的导入
echo "🔍 检查未使用的导入..."
UNUSED_IMPORTS=$(cargo clippy --all-targets -- -W clippy::unused_imports 2>&1 | grep "unused import" | wc -l)
if [ "$UNUSED_IMPORTS" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  发现 $UNUSED_IMPORTS 个未使用的导入${NC}"
    cargo clippy --all-targets -- -W clippy::unused_imports 2>&1 | grep "unused import"
    CUSTOM_OK=false
else
    echo -e "${GREEN}✅ 未发现未使用的导入${NC}"
fi
echo ""

# 4. 检查 unwrap() 使用
echo "🔍 检查 unwrap() 使用..."
if [ -f "scripts/check_unwrap.sh" ]; then
    if bash scripts/check_unwrap.sh 2>&1 | grep -q "found"; then
        echo -e "${YELLOW}⚠️  发现了一些 unwrap() 使用${NC}"
        CUSTOM_OK=false
    else
        echo -e "${GREEN}✅ 未发现不安全的 unwrap() 使用${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  check_unwrap.sh 脚本不存在，跳过检查${NC}"
fi
echo ""

# 5. 检查文档完整性
echo "📚 检查文档完整性..."
MISSING_DOCS=$(cargo doc --no-deps --all-features 2>&1 | grep "missing documentation" | wc -l)
if [ "$MISSING_DOCS" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  发现 $MISSING_DOCS 个缺少文档的项目${NC}"
    CUSTOM_OK=false
else
    echo -e "${GREEN}✅ 文档检查通过${NC}"
fi
echo ""

# 总结
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ "$FORMAT_OK" = true ] && [ "$CLIPPY_OK" = true ] && [ "$CUSTOM_OK" = true ]; then
    echo -e "${GREEN}✅ 所有代码质量检查通过！${NC}"
    exit 0
else
    echo -e "${RED}❌ 代码质量检查未完全通过${NC}"
    echo ""
    echo "问题摘要:"
    [ "$FORMAT_OK" = false ] && echo "  - 代码格式问题"
    [ "$CLIPPY_OK" = false ] && echo "  - Clippy 警告/错误"
    [ "$CUSTOM_OK" = false ] && echo "  - 自定义检查问题"
    echo ""
    echo "修复建议:"
    [ "$FORMAT_OK" = false ] && echo "  - 运行: cargo fmt --all"
    [ "$CLIPPY_OK" = false ] && echo "  - 运行: cargo clippy --fix --all-targets"
    [ "$CUSTOM_OK" = false ] && echo "  - 查看上面的详细输出"
    exit 1
fi

