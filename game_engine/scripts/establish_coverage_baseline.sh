#!/bin/bash
# 测试覆盖率基线建立脚本
#
# 用于建立当前项目的测试覆盖率基线，作为后续改进的参考

set -e

echo "================================"
echo "测试覆盖率基线建立"
echo "================================"
echo ""

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 检查是否安装了tarpaulin
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${YELLOW}警告: cargo-tarpaulin 未安装${NC}"
    echo "安装命令:"
    echo "  cargo install cargo-tarpaulin"
    echo ""
    echo "使用基本测试命令代替..."
    echo ""

    # 运行基本测试
    echo "运行基本测试套件..."
    cargo test --workspace --all-features 2>&1 | tee /tmp/test_output.txt

    echo ""
    echo "================================"
    echo "测试基线建立完成（未使用覆盖率工具）"
    echo "================================"
    echo ""
    echo "建议安装cargo-tarpaulin以获取详细覆盖率报告:"
    echo "  cargo install cargo-tarpaulin"
    echo "  ./scripts/establish_coverage_baseline.sh"
    exit 0
fi

echo -e "${GREEN}✓ cargo-tarpaulin 已安装${NC}"
echo ""

# 运行覆盖率测试
echo "运行详细的覆盖率测试..."
echo ""

OUTPUT_DIR="./coverage"
REPORT_FILE="$OUTPUT_DIR/coverage_report.txt"

# 创建输出目录
mkdir -p "$OUTPUT_DIR"

# 运行tarpaulin
cargo tarpaulin \
    --workspace \
    --all-features \
    --out Html \
    --out Lcov \
    --output-dir "$OUTPUT_DIR" \
    --timeout 300 \
    --verbose \
    2>&1 | tee "$REPORT_FILE" || {
    EXIT_CODE=$?

    if [ $EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✓ 覆盖率测试完成${NC}"
    else
        echo -e "${YELLOW}⚠ 覆盖率测试完成，但有失败测试${NC}"
        echo "退出码: $EXIT_CODE"
    fi
}

echo ""
echo "================================"
echo "覆盖率基线建立完成"
echo "================================"
echo ""

# 提取总体覆盖率
if command -v lcov &> /dev/null; then
    echo "总体覆盖率:"
    lcov --summary "$OUTPUT_DIR/lcov.info" 2>/dev/null || echo "无法生成覆盖率摘要"
else
    echo "（安装lcov工具以查看详细摘要）"
fi

echo ""
echo "覆盖率报告已生成: $OUTPUT_DIR/index.html"
echo "详细报告: $REPORT_FILE"
echo ""

# 显示模块覆盖率目标
echo "覆盖率目标:"
echo "  核心引擎:     75%"
echo "  领域层:       80%"
echo "  ECS系统:      85%"
echo "  总体:         50%"
echo ""

# 统计测试数量
echo "测试统计:"
TEST_COUNT=$(cargo test --workspace --all-features --no-run --message-format=short 2>&1 | grep -c "Running" || echo "0")
echo "  测试数量: ~$TEST_COUNT 个"
echo ""

echo "下一步:"
echo "  1. 查看HTML报告: open $OUTPUT_DIR/index.html"
echo "  2. 为覆盖率低的模块添加测试"
echo "  3. 运行 ./scripts/coverage_check.sh 持续监控"
echo ""
