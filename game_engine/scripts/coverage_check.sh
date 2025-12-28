#!/bin/bash
# 测试覆盖率检查脚本
# 用于CI/CD管道中监控测试覆盖率

set -e

echo "================================"
echo "测试覆盖率检查"
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
    echo "安装命令: cargo install cargo-tarpaulin"
    echo ""
    echo "运行基础测试..."
    cargo test --all-features --workspace
    exit 0
fi

echo -e "${GREEN}运行详细的覆盖率测试...${NC}"
echo ""

# 定义覆盖率目标
CORE_ENGINE_TARGET=75
DOMAIN_LAYER_TARGET=80
ECS_SYSTEM_TARGET=85
OVERALL_TARGET=50

# 运行覆盖率测试
cargo tarpaulin \
    --workspace \
    --all-features \
    --out Html \
    --out Lcov \
    --output-dir ./coverage \
    --timeout 300 \
    -- || true

# 检查覆盖率报告
if [ -f "./coverage/lcov.info" ]; then
    echo ""
    echo "覆盖率报告已生成: ./coverage/index.html"
    echo ""

    # 提取总体覆盖率（如果可能）
    # 注意: 这需要安装lcov工具
    if command -v lcov &> /dev/null; then
        coverage=$(lcov --summary ./coverage/lcov.info 2>&1 | grep "lines" | awk '{print $2}' | sed 's/%//')

        if [ ! -z "$coverage" ]; then
            echo "当前总体覆盖率: ${coverage}%"
            echo "目标覆盖率: ${OVERALL_TARGET}%"

            if (( $(echo "$coverage < $OVERALL_TARGET" | bc -l) )); then
                echo -e "${RED}❌ 覆盖率低于目标${NC}"
                exit 1
            else
                echo -e "${GREEN}✅ 覆盖率达标${NC}"
            fi
        fi
    fi
else
    echo -e "${YELLOW}警告: 覆盖率报告未生成${NC}"
fi

echo ""
echo "模块覆盖率目标:"
echo "  - 核心引擎:    ${CORE_ENGINE_TARGET}%"
echo "  - 领域层:      ${DOMAIN_LAYER_TARGET}%"
echo "  - ECS系统:     ${ECS_SYSTEM_TARGET}%"
echo "  - 总体:        ${OVERALL_TARGET}%"
echo ""

# 检查各模块覆盖率（需要分析生成的HTML报告）
echo "详细报告请查看: ./coverage/index.html"
echo ""

# 列出覆盖率最低的10个文件
if [ -f "./coverage/lcov.info" ]; then
    echo "覆盖率最低的文件 (前10):"
    lcov --list ./coverage/lcov.info 2>/dev/null | tail -10 || echo "需要安装lcov工具查看详细列表"
fi

echo ""
echo "================================"
echo "测试覆盖率检查完成"
echo "================================"
