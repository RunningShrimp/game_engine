#!/bin/bash
# 性能回归检测脚本
# 在CI中检测性能是否显著下降

set -e

echo "📊 Performance Regression Detection"
echo "==================================="

cd "$(dirname "$0")/.."

# 配置
THRESHOLD_WARNING=10   # 警告阈值：10%性能下降
THRESHOLD_CRITICAL=20  # 严重阈值：20%性能下降

# 基准测试列表
BENCHMARKS=(
    "math_benchmarks"
    "ecs_benchmarks"
    "physics_benchmarks"
    "render_benchmarks"
)

echo "🔍 Checking performance regression..."

# 存储结果
RESULTS_FILE="target/performance_results.json"
mkdir -p target

# 初始化结果文件
echo '{"timestamp": "'$(date -u +"%Y-%m-%dT%H:%M:%SZ")'", "benchmarks": {}}' > "$RESULTS_FILE"

# 运行基准测试并收集结果
for bench in "${BENCHMARKS[@]}"; do
    echo "Running $bench..."

    # 运行基准测试
    if cargo bench --package game_engine --bench "$bench" -- --verbose --noplot --sample-size 20 >/dev/null 2>&1; then

        # 查找结果文件
        CRITERION_DIR="target/criterion"
        if [ -d "$CRITERION_DIR" ]; then
            find "$CRITERION_DIR" -name "estimates.json" -path "*/$bench/*" | head -1 | while read -r estimates_file; do
                if [ -f "$estimates_file" ]; then
                    mean_estimate=$(jq -r '.mean.point_estimate // empty' "$estimates_file" 2>/dev/null || echo "0")

                    if [ "$mean_estimate" != "0" ] && [ "$mean_estimate" != "null" ]; then
                        echo "  📊 $bench: ${mean_estimate}ns"

                        # 更新结果文件
                        jq --arg bench "$bench" --arg time "$mean_estimate" \
                           '.benchmarks[$bench] = $time' "$RESULTS_FILE" > "${RESULTS_FILE}.tmp"
                        mv "${RESULTS_FILE}.tmp" "$RESULTS_FILE"
                    fi
                fi
            done
        fi
    else
        echo "❌ Failed to run $bench"
        exit 1
    fi
done

echo ""
echo "📋 Performance results saved to: $RESULTS_FILE"

# 检查是否有基线结果进行比较
BASELINE_FILE="target/performance_baseline.json"
if [ -f "$BASELINE_FILE" ]; then
    echo "🔄 Comparing with baseline..."

    # 使用Node.js进行比较（如果可用）
    if command -v node &> /dev/null; then
        node -e "
            const fs = require('fs');
            const baseline = JSON.parse(fs.readFileSync('$BASELINE_FILE', 'utf8'));
            const current = JSON.parse(fs.readFileSync('$RESULTS_FILE', 'utf8'));

            let hasRegression = false;
            const thresholdWarning = $THRESHOLD_WARNING;
            const thresholdCritical = $THRESHOLD_CRITICAL;

            console.log('Performance Comparison:');
            console.log('======================');

            for (const [bench, currentTime] of Object.entries(current.benchmarks)) {
                const baselineTime = baseline.benchmarks[bench];
                if (baselineTime && currentTime) {
                    const regression = ((currentTime - baselineTime) / baselineTime) * 100;
                    const status = regression > thresholdCritical ? '❌ CRITICAL' :
                                 regression > thresholdWarning ? '⚠️  WARNING' : '✅ OK';

                    console.log(\`\${bench}: \${regression.toFixed(1)}% (\${status})\`);

                    if (regression > thresholdWarning) {
                        hasRegression = true;
                    }
                }
            }

            if (hasRegression) {
                console.log('');
                console.log('🚨 Performance regression detected!');
                process.exit(1);
            } else {
                console.log('');
                console.log('✅ No significant performance regression.');
            }
        "
    else
        echo "⚠️  Node.js not available for comparison. Install Node.js to enable regression detection."
    fi
else
    echo "📝 No baseline found. Saving current results as baseline..."
    cp "$RESULTS_FILE" "$BASELINE_FILE"
    echo "💾 Baseline saved to: $BASELINE_FILE"
fi

echo ""
echo "🎉 Performance check completed!"
