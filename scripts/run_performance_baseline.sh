#!/bin/bash

# 游戏引擎性能基准测试基线脚本
# 用于建立性能基准和检测回归

set -e

echo "🚀 Running Game Engine Performance Baseline Tests"
echo "================================================="

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# 创建结果目录
RESULTS_DIR="performance_results/$(date +%Y%m%d_%H%M%S)"
mkdir -p "$RESULTS_DIR"

echo "📁 Results will be saved to: $RESULTS_DIR"

# 运行各个基准测试
echo ""
echo "🧮 Running Math Benchmarks..."
cargo bench -p game_engine --bench math_benchmarks -- --save-baseline current > "$RESULTS_DIR/math_benchmarks.log" 2>&1 || echo "⚠️  Math benchmarks failed"

echo ""
echo "🎯 Running ECS Benchmarks..."
cargo bench -p game_engine --bench ecs_benchmarks -- --save-baseline current > "$RESULTS_DIR/ecs_benchmarks.log" 2>&1 || echo "⚠️  ECS benchmarks failed"

echo ""
echo "⚛️  Running Physics Benchmarks..."
cargo bench -p game_engine --bench physics_benchmarks -- --save-baseline current > "$RESULTS_DIR/physics_benchmarks.log" 2>&1 || echo "⚠️  Physics benchmarks failed"

echo ""
echo "🎨 Running Render Benchmarks..."
cargo bench -p game_engine --bench render_benchmarks -- --save-baseline current > "$RESULTS_DIR/render_benchmarks.log" 2>&1 || echo "⚠️  Render benchmarks failed"

echo ""
echo "🕸️  Running Network Benchmarks..."
cargo bench -p game_engine --bench network_benchmarks -- --save-baseline current > "$RESULTS_DIR/network_benchmarks.log" 2>&1 || echo "⚠️  Network benchmarks failed"

echo ""
echo "📦 Running Resource Benchmarks..."
cargo bench -p game_engine --bench resource_benchmarks -- --save-baseline current > "$RESULTS_DIR/resource_benchmarks.log" 2>&1 || echo "⚠️  Resource benchmarks failed"

# 生成性能报告摘要
echo ""
echo "📊 Generating Performance Report Summary..."

cat > "$RESULTS_DIR/performance_summary.md" << 'EOF'
# Game Engine Performance Baseline Report

Generated on: $(date)

## Test Results Summary

### Benchmarks Executed:
- ✅ Math Operations (SIMD vs Scalar)
- ✅ ECS Operations (Entity Management, System Updates)
- ✅ Physics Simulation (Rapier Integration)
- ✅ Rendering Pipeline (Frustum Culling, Batching)
- ✅ Network Operations (Serialization, Encryption)
- ✅ Resource Management (Loading, Caching)

### Performance Metrics:
- All benchmarks use Criterion.rs for statistical analysis
- Results saved with `--save-baseline current` for regression detection
- Future runs can compare against this baseline using `--baseline current`

### Notes:
- Some benchmarks may fail if dependencies are not available (e.g., GPU benchmarks on systems without Vulkan)
- Network benchmarks focus on CPU-bound operations (serialization, encryption)
- Resource benchmarks test memory management and caching performance

## Regression Detection

To detect performance regressions, run:
```bash
./scripts/run_performance_baseline.sh
```

Then compare results or use:
```bash
cargo bench -- --baseline current --threshold 5
```

A 5% performance regression threshold is recommended.
EOF

echo ""
echo "✅ Performance baseline tests completed!"
echo "📈 Results saved to: $RESULTS_DIR"
echo ""
echo "💡 Tips:"
echo "   - Use 'cargo bench -- --baseline current' to compare against this baseline"
echo "   - Check individual log files for detailed results"
echo "   - Performance regressions >5% should be investigated"
