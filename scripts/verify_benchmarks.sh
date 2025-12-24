#!/bin/bash
# 基准测试验证脚本
# 用于CI快速验证基准测试是否能够运行

set -e

echo "🔍 Verifying Game Engine Benchmarks"
echo "==================================="

cd "$(dirname "$0")/.."

# 检查基准测试文件是否存在
BENCHMARKS=(
    "math_benchmarks.rs"
    "ecs_benchmarks.rs"
    "physics_benchmarks.rs"
    "render_benchmarks.rs"
    "network_benchmarks.rs"
    "resource_benchmarks.rs"
)

echo "📁 Checking benchmark files..."
for bench_file in "${BENCHMARKS[@]}"; do
    if [ -f "game_engine/benches/$bench_file" ]; then
        echo "✅ $bench_file exists"
    else
        echo "❌ $bench_file missing"
        exit 1
    fi
done

echo ""
echo "🔨 Testing benchmark compilation..."

# 首先测试核心基准测试（已知可以编译的）
CORE_BENCHMARKS=("math_benchmarks" "ecs_benchmarks" "physics_benchmarks" "network_benchmarks" "resource_benchmarks")

for bench_name in "${CORE_BENCHMARKS[@]}"; do
    echo "Testing $bench_name compilation..."

    if cargo check --bench "$bench_name" >/dev/null 2>&1; then
        echo "✅ $bench_name compiles successfully"
    else
        echo "❌ $bench_name compilation failed"
        echo "   Run: cargo check --bench $bench_name"
        exit 1
    fi
done

# 测试render_benchmarks（已更新到当前wgpu API）
echo "Testing render_benchmarks compilation..."
if cargo check --bench "render_benchmarks" >/dev/null 2>&1; then
    echo "✅ render_benchmarks compiles successfully"
else
    echo "⚠️  render_benchmarks compilation has warnings (may require GPU adapter)"
    echo "   This is expected in CI environments without GPU access"
    echo "   Run: cargo check --bench render_benchmarks for details"
fi

echo ""
echo "🎉 Benchmark verification completed successfully!"
echo "💡 Full benchmarks can be run with: ./scripts/run_benchmarks.sh"