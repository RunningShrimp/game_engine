#!/bin/bash
# 基准测试和性能回归检测综合脚本
# 用于定期运行基准测试并检测性能回归

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "🚀 基准测试和性能回归检测"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "开始时间: $(date)"
echo ""

# 配置
THRESHOLD_WARNING=10   # 警告阈值：10%性能下降
THRESHOLD_CRITICAL=20  # 严重阈值：20%性能下降
RESULTS_DIR="target/benchmark_results"
BASELINE_FILE="$RESULTS_DIR/performance_baseline.json"
CURRENT_RESULTS_FILE="$RESULTS_DIR/performance_current.json"
HISTORY_DIR="$RESULTS_DIR/history"

# 创建目录
mkdir -p "$RESULTS_DIR"
mkdir -p "$HISTORY_DIR"

# 检查工具
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}❌ Cargo not found. Please install Rust.${NC}"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}⚠️  jq not found. Installing...${NC}"
    if command -v brew &> /dev/null; then
        brew install jq
    else
        echo -e "${RED}Please install jq manually: https://stedolan.github.io/jq/${NC}"
        exit 1
    fi
fi

# 基准测试列表
BENCHMARKS=(
    "math_benchmarks"
    "ecs_benchmarks"
    "render_benchmarks"
    "pathfinding_benchmarks"
    "resource_benchmarks"
)

echo "📊 基准测试列表: ${#BENCHMARKS[@]}"
for bench in "${BENCHMARKS[@]}"; do
    echo "  - $bench"
done
echo ""

# 初始化结果文件
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
echo "{\"timestamp\": \"$TIMESTAMP\", \"benchmarks\": {}}" > "$CURRENT_RESULTS_FILE"

# 运行基准测试
echo "🏃 运行基准测试..."
echo ""

REGRESSION_DETECTED=false
WARNINGS=0
CRITICALS=0

for bench in "${BENCHMARKS[@]}"; do
    echo -e "${BLUE}运行 $bench...${NC}"
    
    # 运行基准测试（使用较小的采样以加快速度）
    if cargo bench --package game_engine --bench "$bench" -- --verbose --noplot --sample-size 20 2>&1 | tee /tmp/bench_${bench}.log | grep -q "test"; then
        echo -e "${GREEN}✅ $bench 完成${NC}"
        
        # 提取结果
        CRITERION_DIR="target/criterion"
        if [ -d "$CRITERION_DIR" ]; then
            # 查找最新的基准测试结果
            ESTIMATES_FILE=$(find "$CRITERION_DIR" -name "estimates.json" -path "*/$bench/*" | head -1)
            
            if [ -f "$ESTIMATES_FILE" ]; then
                MEAN_ESTIMATE=$(jq -r '.mean.point_estimate // empty' "$ESTIMATES_FILE" 2>/dev/null || echo "0")
                
                if [ "$MEAN_ESTIMATE" != "0" ] && [ "$MEAN_ESTIMATE" != "null" ] && [ "$MEAN_ESTIMATE" != "" ]; then
                    # 转换为可读格式
                    if (( $(echo "$MEAN_ESTIMATE > 1000000" | bc -l 2>/dev/null || echo 0) )); then
                        READABLE_TIME=$(echo "scale=2; $MEAN_ESTIMATE / 1000000" | bc 2>/dev/null || echo "N/A")
                        READABLE_TIME="${READABLE_TIME}ms"
                    elif (( $(echo "$MEAN_ESTIMATE > 1000" | bc -l 2>/dev/null || echo 0) )); then
                        READABLE_TIME=$(echo "scale=2; $MEAN_ESTIMATE / 1000" | bc 2>/dev/null || echo "N/A")
                        READABLE_TIME="${READABLE_TIME}μs"
                    else
                        READABLE_TIME="${MEAN_ESTIMATE}ns"
                    fi
                    
                    echo "  📈 平均时间: $READABLE_TIME"
                    
                    # 更新结果文件
                    jq --arg bench "$bench" --arg time "$MEAN_ESTIMATE" \
                       '.benchmarks[$bench] = $time' "$CURRENT_RESULTS_FILE" > "${CURRENT_RESULTS_FILE}.tmp"
                    mv "${CURRENT_RESULTS_FILE}.tmp" "$CURRENT_RESULTS_FILE"
                else
                    echo -e "${YELLOW}  ⚠️  无法提取性能数据${NC}"
                fi
            else
                echo -e "${YELLOW}  ⚠️  未找到结果文件${NC}"
            fi
        fi
    else
        echo -e "${RED}❌ $bench 失败${NC}"
        # 继续运行其他基准测试，不立即退出
    fi
    
    echo ""
done

# 保存历史记录
HISTORY_FILE="$HISTORY_DIR/$(date +%Y%m%d_%H%M%S).json"
cp "$CURRENT_RESULTS_FILE" "$HISTORY_FILE"
echo "📁 历史记录已保存: $HISTORY_FILE"

# 性能回归检测
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "🔍 性能回归检测"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ -f "$BASELINE_FILE" ]; then
    echo "🔄 与基线进行比较..."
    echo ""
    
    # 使用Python进行比较（如果可用）
    if command -v python3 &> /dev/null; then
        python3 << EOF
import json
import sys

baseline_file = "$BASELINE_FILE"
current_file = "$CURRENT_RESULTS_FILE"
threshold_warning = $THRESHOLD_WARNING
threshold_critical = $THRESHOLD_CRITICAL

try:
    with open(baseline_file, 'r') as f:
        baseline = json.load(f)
    with open(current_file, 'r') as f:
        current = json.load(f)
    
    print("性能对比:")
    print("=" * 60)
    print(f"{'基准测试':<30} {'变化':<15} {'状态':<10}")
    print("-" * 60)
    
    has_regression = False
    warnings = 0
    criticals = 0
    
    for bench, current_time in current.get('benchmarks', {}).items():
        baseline_time = baseline.get('benchmarks', {}).get(bench)
        if baseline_time and current_time:
            try:
                baseline_time = float(baseline_time)
                current_time = float(current_time)
                
                if baseline_time > 0:
                    regression = ((current_time - baseline_time) / baseline_time) * 100
                    
                    if regression > threshold_critical:
                        status = "❌ CRITICAL"
                        has_regression = True
                        criticals += 1
                    elif regression > threshold_warning:
                        status = "⚠️  WARNING"
                        has_regression = True
                        warnings += 1
                    else:
                        status = "✅ OK"
                    
                    change_str = f"{regression:+.1f}%"
                    print(f"{bench:<30} {change_str:<15} {status:<10}")
            except (ValueError, TypeError):
                print(f"{bench:<30} {'N/A':<15} {'⚠️  ERROR':<10}")
        else:
            if baseline_time:
                print(f"{bench:<30} {'基线缺失':<15} {'⚠️  WARNING':<10}")
            else:
                print(f"{bench:<30} {'新测试':<15} {'ℹ️  NEW':<10}")
    
    print("=" * 60)
    print(f"\n统计: {warnings} 个警告, {criticals} 个严重问题")
    
    if has_regression:
        print("\n🚨 检测到性能回归!")
        sys.exit(1)
    else:
        print("\n✅ 未检测到显著性能回归")
        sys.exit(0)
except FileNotFoundError as e:
    print(f"错误: 文件未找到 - {e}")
    sys.exit(1)
except json.JSONDecodeError as e:
    print(f"错误: JSON解析失败 - {e}")
    sys.exit(1)
except Exception as e:
    print(f"错误: {e}")
    sys.exit(1)
EOF
        
        COMPARISON_EXIT=$?
        if [ $COMPARISON_EXIT -eq 1 ]; then
            REGRESSION_DETECTED=true
        fi
    else
        echo -e "${YELLOW}⚠️  Python3 不可用，跳过详细比较${NC}"
        echo "安装 Python3 以启用性能回归检测"
    fi
else
    echo "📝 未找到基线文件，保存当前结果作为基线..."
    cp "$CURRENT_RESULTS_FILE" "$BASELINE_FILE"
    echo -e "${GREEN}💾 基线已保存: $BASELINE_FILE${NC}"
    echo ""
    echo "提示: 下次运行此脚本时将进行性能回归检测"
fi

# 生成报告
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 生成报告"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 生成Markdown报告
REPORT_FILE="$RESULTS_DIR/benchmark_report_$(date +%Y%m%d_%H%M%S).md"
cat > "$REPORT_FILE" << EOF
# 基准测试报告

**生成时间**: $(date)
**基线文件**: $BASELINE_FILE
**当前结果**: $CURRENT_RESULTS_FILE

## 基准测试结果

EOF

if [ -f "$CURRENT_RESULTS_FILE" ]; then
    echo "### 性能数据" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "\`\`\`json" >> "$REPORT_FILE"
    cat "$CURRENT_RESULTS_FILE" | jq '.' >> "$REPORT_FILE" || cat "$CURRENT_RESULTS_FILE" >> "$REPORT_FILE"
    echo "\`\`\`" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi

if [ -f "$BASELINE_FILE" ] && [ "$REGRESSION_DETECTED" = true ]; then
    echo "### ⚠️ 性能回归检测" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
    echo "检测到性能回归，请查看详细对比。" >> "$REPORT_FILE"
    echo "" >> "$REPORT_FILE"
fi

echo "报告已保存: $REPORT_FILE"

# 总结
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📋 总结"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "开始时间: $(date)"
echo "结束时间: $(date)"
echo ""
echo "结果文件:"
echo "  - 当前结果: $CURRENT_RESULTS_FILE"
if [ -f "$BASELINE_FILE" ]; then
    echo "  - 基线文件: $BASELINE_FILE"
fi
echo "  - 历史记录: $HISTORY_DIR"
echo "  - 报告文件: $REPORT_FILE"
echo ""

if [ "$REGRESSION_DETECTED" = true ]; then
    echo -e "${RED}❌ 检测到性能回归${NC}"
    echo ""
    echo "建议:"
    echo "  1. 查看详细报告: $REPORT_FILE"
    echo "  2. 检查最近的代码更改"
    echo "  3. 如果回归是预期的，更新基线: cp $CURRENT_RESULTS_FILE $BASELINE_FILE"
    exit 1
else
    echo -e "${GREEN}✅ 基准测试完成，未检测到性能回归${NC}"
    exit 0
fi

