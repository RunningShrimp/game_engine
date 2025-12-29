#!/bin/bash
# 监控benchmark性能变化
# 这个脚本会持续运行benchmark并监控性能变化

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Benchmark Monitoring System${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 获取脚本所在目录
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_ROOT"

# 检查是否已经初始化baseline
BASELINE_DIR="target/criterion/main"
if [ ! -d "$BASELINE_DIR" ]; then
    echo -e "${YELLOW}Warning: No baseline found. Creating initial baseline...${NC}"
    cargo bench --workspace -- --save-baseline main --output-format bencher
    echo -e "${GREEN}✓ Baseline created${NC}"
    echo ""
fi

# 设置监控间隔（秒）
INTERVAL=${BENCHMARK_INTERVAL:-300}  # 默认5分钟

echo -e "${BLUE}Configuration:${NC}"
echo "  - Monitor interval: ${INTERVAL}s"
echo "  - Project root: ${PROJECT_ROOT}"
echo ""

# 创建结果目录
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULT_DIR="benches/monitoring/${TIMESTAMP}"
mkdir -p "$RESULT_DIR"

echo -e "${BLUE}Starting benchmark monitoring...${NC}"
echo -e "${YELLOW}Press Ctrl+C to stop${NC}"
echo ""

RUN_COUNT=0

while true; do
    RUN_COUNT=$((RUN_COUNT + 1))
    RUN_TIMESTAMP=$(date +"%Y-%m-%d %H:%M:%S")

    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}Run #${RUN_COUNT} - ${RUN_TIMESTAMP}${NC}"
    echo -e "${BLUE}========================================${NC}"

    # 运行benchmark与baseline对比
    echo -e "${GREEN}Running benchmarks...${NC}"

    if cargo bench --workspace -- --baseline main --output-format bencher 2>&1 | tee "$RESULT_DIR/run_${RUN_COUNT}.txt"; then
        echo -e "${GREEN}✓ Benchmarks completed successfully${NC}"
    else
        echo -e "${RED}✗ Benchmarks failed${NC}"
    fi

    echo ""

    # 生成性能报告
    echo -e "${GREEN}Generating performance report...${NC}"
    if python3 "$SCRIPT_DIR/generate_benchmark_report.py" > "$RESULT_DIR/report_${RUN_COUNT}.md" 2>&1; then
        echo -e "${GREEN}✓ Report generated${NC}"

        # 显示摘要
        if grep -q "No performance regressions detected" "$RESULT_DIR/report_${RUN_COUNT}.md"; then
            echo -e "${GREEN}✓ No regressions detected${NC}"
        else
            echo -e "${RED}⚠ Performance regression detected!${NC}"
            grep -A 2 "Performance Regressions" "$RESULT_DIR/report_${RUN_COUNT}.md" || true
        fi
    else
        echo -e "${RED}✗ Report generation failed${NC}"
    fi

    echo ""

    # 导出数据
    if [ -f "$SCRIPT_DIR/export_benchmark_data.py" ]; then
        echo -e "${GREEN}Exporting benchmark data...${NC}"
        python3 "$SCRIPT_DIR/export_benchmark_data.py"
    fi

    echo ""

    # 如果不是第一次运行，等待指定间隔
    if [ $RUN_COUNT -gt 0 ]; then
        echo -e "${YELLOW}Waiting ${INTERVAL}s until next run...${NC}"
        echo ""

        # 显示倒计时
        for ((i=$INTERVAL; i>0; i--)); do
            printf "\r${BLUE}Next run in: ${NC}%02d:%02d " $((i/60)) $((i%60))
            sleep 1
        done

        echo ""
        echo ""
    fi
done
