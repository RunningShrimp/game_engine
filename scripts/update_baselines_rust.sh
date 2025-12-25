#!/bin/bash
# 使用Rust程序更新性能基线

set -e

echo "🚀 使用Rust程序更新性能基线..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# 运行Rust示例程序
cargo run --package game_engine --example update_performance_baselines

echo ""
echo "✅ 完成！"

