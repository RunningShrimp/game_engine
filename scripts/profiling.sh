#!/bin/bash
# 性能分析脚本
#
# 使用不同的工具进行性能分析

set -e

TOOL=${1:-"flamegraph"}

echo "🔥 性能分析工具: $TOOL"
echo ""

case "$TOOL" in
    flamegraph)
        echo "📊 使用Flamegraph生成火焰图..."

        # 检查是否安装了cargo-flamegraph
        if ! command -v cargo-flamegraph &> /dev/null; then
            echo "📦 安装cargo-flamegraph..."
            cargo install flamegraph
        fi

        # 运行flamegraph
        cargo flamegraph --bin game_engine --example performance_examples

        echo "✅ 火焰图已生成: flamegraph.svg"
        echo "💡 在浏览器中打开查看"
        ;;

    perf)
        echo "📊 使用perf进行性能分析..."

        # 检查perf是否可用
        if ! command -v perf &> /dev/null; then
            echo "❌ perf不可用（仅Linux）"
            exit 1
        fi

        # 运行perf record
        perf record --call-graph dwarf cargo run --example performance_examples

        # 生成报告
        perf report

        echo "✅ perf分析完成"
        ;;

    samply)
        echo "📊 使用samply进行性能分析（跨平台）..."

        # 检查samply是否安装
        if ! command -v samply &> /dev/null; then
            echo "📦 安装samply..."
            cargo install samply
        fi

        # 运行samply
        samply record cargo run --example performance_examples

        echo "✅ samply分析完成（应在浏览器中自动打开）"
        ;;

    tracy)
        echo "📊 使用Tracy进行性能分析..."

        # 需要启用tracy feature
        echo "💡 使用: cargo run --example performance_examples --features tracy"
        cargo run --example performance_examples --features tracy

        echo "✅ Tracy分析完成"
        ;;

    *)
        echo "❌ 未知的性能分析工具: $TOOL"
        echo ""
        echo "可用工具:"
        echo "  - flamegraph: 生成火焰图（默认）"
        echo "  - perf:      Linux perf工具"
        echo "  - samply:    跨平台采样分析器"
        echo "  - tracy:     Tracy性能分析器"
        echo ""
        echo "用法: ./scripts/profiling.sh [tool]"
        exit 1
        ;;
esac

echo ""
echo "💡 性能优化建议:"
echo "  - 查看火焰图找出热点函数"
echo "  - 优化高占用函数"
echo "  - 使用并行处理"
echo "  - 考虑缓存优化"
