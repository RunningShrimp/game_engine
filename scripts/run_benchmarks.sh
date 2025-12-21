#!/bin/bash
# 游戏引擎基准测试运行脚本
# 用于CI/CD和本地性能回归检测

set -e

echo "🚀 Running Game Engine Benchmarks"
echo "================================="

# 检查是否安装了必要的工具
if ! command -v cargo &> /dev/null; then
    echo "❌ Cargo not found. Please install Rust."
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo "⚠️  jq not found. Installing..."
    # 尝试安装jq (macOS)
    if command -v brew &> /dev/null; then
        brew install jq
    else
        echo "Please install jq manually: https://stedolan.github.io/jq/"
        exit 1
    fi
fi

cd "$(dirname "$0")/.."

# 基准测试列表
BENCHMARKS=(
    "math_benchmarks"
    "ecs_benchmarks"
    "physics_benchmarks"
    "render_benchmarks"
    "network_benchmarks"
    "resource_benchmarks"
    "pathfinding_benchmarks"
)

echo "📊 Available benchmarks: ${#BENCHMARKS[@]}"
printf '%s\n' "${BENCHMARKS[@]}"
echo ""

# 创建基准测试结果目录
RESULTS_DIR="target/benchmark_results"
mkdir -p "$RESULTS_DIR"

# 运行所有基准测试
echo "🏃 Running benchmarks..."
for bench in "${BENCHMARKS[@]}"; do
    echo "Running $bench..."

    # 使用采样模式运行基准测试（更快）
    if cargo bench --package game_engine --bench "$bench" -- --verbose --noplot; then
        echo "✅ $bench completed successfully"

        # 提取结果摘要
        CRITERION_DIR="target/criterion"
        if [ -d "$CRITERION_DIR" ]; then
            # 查找最新的基准测试结果
            find "$CRITERION_DIR" -name "estimates.json" -path "*/$bench/*" | head -1 | while read -r estimates_file; do
                if [ -f "$estimates_file" ]; then
                    mean_estimate=$(jq -r '.mean.point_estimate // empty' "$estimates_file" 2>/dev/null || echo "N/A")
                    if [ "$mean_estimate" != "N/A" ] && [ "$mean_estimate" != "null" ]; then
                        # 转换为毫秒或合适的单位
                        if (( $(echo "$mean_estimate > 1000000" | bc -l 2>/dev/null) )); then
                            readable_time="$(echo "scale=2; $mean_estimate / 1000000" | bc 2>/dev/null || echo "N/A")ms"
                        else
                            readable_time="${mean_estimate}ns"
                        fi
                        echo "  📈 Mean time: $readable_time"
                    fi
                fi
            done
        fi
    else
        echo "❌ $bench failed"
        exit 1
    fi

    echo ""
done

echo "🎉 All benchmarks completed successfully!"
echo ""
echo "📁 Benchmark results saved in: target/criterion/"
echo "🔍 Use 'cargo run --bin compare_performance' to compare against baselines"
