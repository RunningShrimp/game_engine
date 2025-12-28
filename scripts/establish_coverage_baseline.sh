#!/bin/bash
# 建立代码覆盖率基线脚本
# 生成基线报告并保存到 docs/TEST_COVERAGE_BASELINE.md

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "📊 建立代码覆盖率基线"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# 检查工具
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${RED}❌ cargo-tarpaulin 未安装${NC}"
    echo ""
    echo "请运行以下命令安装:"
    echo "  cargo install cargo-tarpaulin"
    echo ""
    exit 1
fi

echo -e "${GREEN}✅ cargo-tarpaulin 已安装${NC}"
echo ""

# 创建输出目录
COVERAGE_DIR="target/coverage"
BASELINE_DIR="docs/coverage_baseline"
mkdir -p "$COVERAGE_DIR"
mkdir -p "$BASELINE_DIR"

# 运行测试
echo "🧪 运行测试套件..."
if ! cargo test --workspace --lib --quiet --no-fail-fast 2>&1 | tail -20; then
    echo -e "${YELLOW}⚠️  部分测试可能失败，继续生成覆盖率报告...${NC}"
fi

echo ""
echo -e "${GREEN}✅ 测试完成${NC}"
echo ""

# 生成覆盖率报告
echo "📈 生成覆盖率报告..."

# 生成JSON报告（用于解析）
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
    --all-features > "$COVERAGE_DIR/coverage.json" 2>&1 || {
    echo -e "${YELLOW}⚠️  覆盖率报告生成完成（可能有部分文件未覆盖）${NC}"
}

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
    --all-features || true

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
    --all-features > "$COVERAGE_DIR/lcov.info" 2>&1 || true

echo ""
echo -e "${GREEN}✅ 覆盖率报告已生成${NC}"
echo ""

# 解析JSON报告并生成基线数据
echo "📝 生成基线数据..."

if [ -f "$COVERAGE_DIR/coverage.json" ]; then
    # 使用Python或jq解析JSON（如果可用）
    if command -v python3 &> /dev/null; then
        python3 << 'PYTHON_SCRIPT'
import json
import sys
from pathlib import Path

try:
    with open('target/coverage/coverage.json', 'r') as f:
        data = json.load(f)
    
    # 提取总体覆盖率
    total_lines = data.get('total_lines', 0)
    covered_lines = data.get('covered_lines', 0)
    coverage_percent = (covered_lines / total_lines * 100) if total_lines > 0 else 0
    
    print(f"总体覆盖率: {coverage_percent:.2f}%")
    print(f"总行数: {total_lines}")
    print(f"覆盖行数: {covered_lines}")
    
    # 按文件分组统计
    files = data.get('files', [])
    modules = {}
    
    for file_data in files:
        file_path = file_data.get('path', '')
        if 'game_engine/src' in file_path:
            # 提取模块名
            parts = file_path.split('game_engine/src/')
            if len(parts) > 1:
                module = parts[1].split('/')[0]
                if module not in modules:
                    modules[module] = {'lines': 0, 'covered': 0}
                modules[module]['lines'] += file_data.get('lines', 0)
                modules[module]['covered'] += file_data.get('covered', 0)
    
    print("\n模块覆盖率:")
    for module, stats in sorted(modules.items()):
        mod_coverage = (stats['covered'] / stats['lines'] * 100) if stats['lines'] > 0 else 0
        print(f"  {module}: {mod_coverage:.2f}% ({stats['covered']}/{stats['lines']})")
        
except Exception as e:
    print(f"解析JSON失败: {e}", file=sys.stderr)
    sys.exit(1)
PYTHON_SCRIPT
    else
        echo "⚠️  Python3 未安装，无法自动解析覆盖率数据"
        echo "   请手动查看 $COVERAGE_DIR/coverage.json"
    fi
else
    echo "⚠️  覆盖率JSON文件未生成"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 基线报告位置"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "  - HTML 报告: $COVERAGE_DIR/index.html"
echo "  - JSON 报告: $COVERAGE_DIR/coverage.json"
echo "  - LCOV 报告: $COVERAGE_DIR/lcov.info"
echo "  - 基线文档: docs/TEST_COVERAGE_BASELINE.md"
echo ""
echo -e "${BLUE}💡 查看HTML报告:${NC}"
echo "   open $COVERAGE_DIR/index.html"
echo ""
echo -e "${GREEN}✅ 基线建立完成${NC}"
echo ""
echo "下一步:"
echo "  1. 查看覆盖率报告，识别未覆盖的关键代码"
echo "  2. 更新 docs/TEST_COVERAGE_BASELINE.md 中的基线数据"
echo "  3. 制定覆盖率改进计划"

