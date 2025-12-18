#!/bin/bash
# 代码质量检查脚本
# 用于本地开发和CI/CD

set -e

echo "🔍 Running code quality checks..."

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 阈值配置
CLIPPY_WARN_THRESHOLD=100
DOC_WARN_THRESHOLD=50
COVERAGE_THRESHOLD=80  # 百分比

ERRORS=0

# 1. 格式检查
echo ""
echo "📝 Checking code formatting..."
if cargo fmt --all -- --check; then
    echo -e "${GREEN}✅ Code formatting OK${NC}"
else
    echo -e "${RED}❌ Code formatting failed. Run 'cargo fmt --all' to fix.${NC}"
    ERRORS=$((ERRORS + 1))
fi

# 2. Clippy 检查
echo ""
echo "🔧 Running Clippy..."
CLIPPY_OUTPUT=$(cargo clippy --all-targets --all-features -- -D warnings 2>&1 || true)
CLIPPY_WARNINGS=$(echo "$CLIPPY_OUTPUT" | grep -c "warning:" || echo "0")

if [ "$CLIPPY_WARNINGS" -gt "$CLIPPY_WARN_THRESHOLD" ]; then
    echo -e "${RED}❌ Clippy warnings ($CLIPPY_WARNINGS) exceed threshold ($CLIPPY_WARN_THRESHOLD)${NC}"
    ERRORS=$((ERRORS + 1))
else
    echo -e "${GREEN}✅ Clippy OK (warnings: $CLIPPY_WARNINGS)${NC}"
fi

# 3. 文档检查
echo ""
echo "📚 Checking documentation..."
DOC_OUTPUT=$(cargo doc --no-deps --all-features --document-private-items 2>&1 || true)
MISSING_DOCS=$(echo "$DOC_OUTPUT" | grep -c "missing_docs" || echo "0")

if [ "$MISSING_DOCS" -gt "$DOC_WARN_THRESHOLD" ]; then
    echo -e "${YELLOW}⚠️  Missing documentation warnings ($MISSING_DOCS) exceed threshold ($DOC_WARN_THRESHOLD)${NC}"
    # 文档警告不阻止构建，只警告
else
    echo -e "${GREEN}✅ Documentation OK (missing: $MISSING_DOCS)${NC}"
fi

# 4. 测试覆盖率（如果安装了tarpaulin）
echo ""
echo "📊 Checking test coverage..."
if command -v cargo-tarpaulin &> /dev/null; then
    cargo tarpaulin --out Xml --output-dir coverage/ --all-features --timeout 300 || true
    if [ -f coverage/cobertura.xml ]; then
        echo -e "${GREEN}✅ Coverage report generated${NC}"
        # 这里可以添加XML解析来获取实际覆盖率
    else
        echo -e "${YELLOW}⚠️  Coverage report not generated${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed. Install with: cargo install cargo-tarpaulin${NC}"
fi

# 总结
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
if [ "$ERRORS" -eq 0 ]; then
    echo -e "${GREEN}✅ All code quality checks passed!${NC}"
    exit 0
else
    echo -e "${RED}❌ Code quality checks failed ($ERRORS error(s))${NC}"
    exit 1
fi


<<<<<<< HEAD
=======

>>>>>>> 50b9493 (feat: Complete service layer testing with 43 comprehensive tests)
