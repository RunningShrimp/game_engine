#!/bin/bash
# 代码覆盖率测试和报告生成脚本
# 支持多种覆盖率工具：cargo-tarpaulin, grcov

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "📊 代码覆盖率测试和报告生成"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 创建输出目录
COVERAGE_DIR="target/coverage"
mkdir -p "$COVERAGE_DIR"

# 检查工具
TARPAULIN_AVAILABLE=false
GRCOV_AVAILABLE=false

if command -v cargo-tarpaulin &> /dev/null; then
    TARPAULIN_AVAILABLE=true
    echo -e "${GREEN}✅ cargo-tarpaulin 已安装${NC}"
else
    echo -e "${YELLOW}⚠️  cargo-tarpaulin 未安装${NC}"
fi

if command -v grcov &> /dev/null; then
    GRCOV_AVAILABLE=true
    echo -e "${GREEN}✅ grcov 已安装${NC}"
else
    echo -e "${YELLOW}⚠️  grcov 未安装${NC}"
fi

echo ""

# 如果没有工具，提供安装说明
if [ "$TARPAULIN_AVAILABLE" = false ] && [ "$GRCOV_AVAILABLE" = false ]; then
    echo -e "${RED}❌ 未找到覆盖率工具${NC}"
    echo ""
    echo "请安装以下工具之一："
    echo ""
    echo "1. cargo-tarpaulin (推荐):"
    echo "   cargo install cargo-tarpaulin"
    echo ""
    echo "2. grcov:"
    echo "   cargo install grcov"
    echo ""
    exit 1
fi

# 运行测试
echo "🧪 运行测试套件..."
cargo test --workspace --lib --quiet --no-fail-fast 2>&1 | tail -20
TEST_EXIT_CODE=${PIPESTATUS[0]}

if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo -e "${RED}❌ 测试失败，无法生成覆盖率报告${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 测试通过${NC}"
echo ""

# 生成覆盖率报告
if [ "$TARPAULIN_AVAILABLE" = true ]; then
    echo "📈 使用 cargo-tarpaulin 生成覆盖率报告..."
    
    # 生成HTML报告
    cargo tarpaulin \
        --workspace \
        --out Html \
        --output-dir "$COVERAGE_DIR" \
        --exclude-files "*/tests/*" \
        --exclude-files "*/benches/*" \
        --exclude-files "*/examples/*" \
        --exclude-files "*/target/*" \
        --timeout 300 \
        --skip-clean \
        --all-features || {
        echo -e "${YELLOW}⚠️  覆盖率报告生成完成（可能有部分文件未覆盖）${NC}"
    }
    
    # 生成LCOV报告
    cargo tarpaulin \
        --workspace \
        --out Lcov \
        --output-dir "$COVERAGE_DIR" \
        --exclude-files "*/tests/*" \
        --exclude-files "*/benches/*" \
        --exclude-files "*/examples/*" \
        --exclude-files "*/target/*" \
        --timeout 300 \
        --skip-clean \
        --all-features || true
    
    # 生成JSON报告
    cargo tarpaulin \
        --workspace \
        --out Json \
        --output-dir "$COVERAGE_DIR" \
        --exclude-files "*/tests/*" \
        --exclude-files "*/benches/*" \
        --exclude-files "*/examples/*" \
        --exclude-files "*/target/*" \
        --timeout 300 \
        --skip-clean \
        --all-features || true
    
    echo -e "${GREEN}✅ 覆盖率报告已生成${NC}"
    
elif [ "$GRCOV_AVAILABLE" = true ]; then
    echo "📈 使用 grcov 生成覆盖率报告..."
    
    # 清理之前的覆盖率数据
    rm -rf "$COVERAGE_DIR"/*
    
    # 设置环境变量
    export CARGO_INCREMENTAL=0
    export RUSTFLAGS="-Cinstrument-coverage"
    export LLVM_PROFILE_FILE="$COVERAGE_DIR/cargo-test-%p-%m.profraw"
    
    # 重新构建和测试
    echo "  - 重新构建（带覆盖率插桩）..."
    cargo clean
    cargo build --workspace --all-features
    
    echo "  - 运行测试（收集覆盖率数据）..."
    cargo test --workspace --lib --all-features --no-fail-fast
    
    # 生成HTML报告
    echo "  - 生成HTML报告..."
    grcov . \
        --binary-path ./target/debug/deps \
        -s . \
        -t html \
        --branch \
        --ignore-not-existing \
        --ignore "*/tests/*" \
        --ignore "*/benches/*" \
        --ignore "*/examples/*" \
        --ignore "*/target/*" \
        -o "$COVERAGE_DIR/html" || {
        echo -e "${YELLOW}⚠️  覆盖率报告生成完成（可能有部分文件未覆盖）${NC}"
    }
    
    # 生成LCOV报告
    echo "  - 生成LCOV报告..."
    grcov . \
        --binary-path ./target/debug/deps \
        -s . \
        -t lcov \
        --branch \
        --ignore-not-existing \
        --ignore "*/tests/*" \
        --ignore "*/benches/*" \
        --ignore "*/examples/*" \
        --ignore "*/target/*" \
        -o "$COVERAGE_DIR/lcov.info" || true
    
    # 恢复环境变量
    unset CARGO_INCREMENTAL
    unset RUSTFLAGS
    unset LLVM_PROFILE_FILE
    
    echo -e "${GREEN}✅ 覆盖率报告已生成${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 覆盖率报告摘要"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 尝试提取覆盖率数据
if [ -f "$COVERAGE_DIR/cobertura.xml" ]; then
    echo "  - Cobertura XML: $COVERAGE_DIR/cobertura.xml"
fi

if [ -f "$COVERAGE_DIR/lcov.info" ]; then
    echo "  - LCOV 报告: $COVERAGE_DIR/lcov.info"
    
    # 尝试计算覆盖率百分比
    if command -v lcov &> /dev/null; then
        TOTAL_LINES=$(lcov --summary "$COVERAGE_DIR/lcov.info" 2>/dev/null | grep "lines" | awk '{print $2}' || echo "N/A")
        COVERED_LINES=$(lcov --summary "$COVERAGE_DIR/lcov.info" 2>/dev/null | grep "lines" | awk '{print $4}' || echo "N/A")
        echo "  - 总行数: $TOTAL_LINES"
        echo "  - 覆盖行数: $COVERED_LINES"
    fi
fi

if [ -f "$COVERAGE_DIR/index.html" ]; then
    echo "  - HTML 报告: $COVERAGE_DIR/index.html"
    echo ""
    echo -e "${BLUE}💡 查看HTML报告:${NC}"
    echo "   open $COVERAGE_DIR/index.html"
elif [ -d "$COVERAGE_DIR/html" ]; then
    echo "  - HTML 报告: $COVERAGE_DIR/html/index.html"
    echo ""
    echo -e "${BLUE}💡 查看HTML报告:${NC}"
    echo "   open $COVERAGE_DIR/html/index.html"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ 覆盖率测试完成${NC}"
echo ""

