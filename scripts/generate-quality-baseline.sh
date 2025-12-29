#!/bin/bash
# 生成代码质量基线报告
# 用法: ./scripts/generate-quality-baseline.sh

set -e

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BASELINE_DIR="$PROJECT_ROOT/docs/coverage/baseline"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

echo "========================================="
echo "  游戏引擎代码质量基线生成工具"
echo "========================================="
echo ""

# 创建基线目录
mkdir -p "$BASELINE_DIR"

# 输出文件
OUTPUT_FILE="$BASELINE_DIR/metrics_$TIMESTAMP.txt"

echo "生成基线报告到: $OUTPUT_FILE"
echo ""

{
    echo "游戏引擎代码质量基线数据"
    echo "========================="
    echo "生成时间: $(date)"
    echo "项目版本: $(grep '^version' "$PROJECT_ROOT/game_engine/Cargo.toml" | head -1 | cut -d'"' -f2)"
    echo ""
    
    echo "1. 编译状态"
    echo "-----------"
    echo "Library编译检查:"
    cd "$PROJECT_ROOT/game_engine"
    if cargo check --lib 2>&1 | tee /tmp/cargo_check.log | grep -q "Finished"; then
        echo "  ✅ PASSED"
    else
        echo "  ❌ FAILED"
    fi
    echo ""
    
    echo "2. Clippy分析"
    echo "-------------"
    echo "警告统计:"
    cargo clippy --lib --message-format=short 2>&1 | \
        grep "warning:" | \
        cut -d':' -f4 | \
        sort | uniq -c | sort -rn | head -10 | \
        sed 's/^/  /'
    echo ""
    
    echo "3. 代码统计"
    echo "----------"
    echo "Rust源文件数:"
    find "$PROJECT_ROOT/game_engine/src" -name "*.rs" | wc -l | sed 's/^/  /'
    echo "总代码行数:"
    find "$PROJECT_ROOT/game_engine/src" -name "*.rs" -exec wc -l {} + | tail -1 | sed 's/^/  /'
    echo ""
    
    echo "4. 条件编译统计"
    echo "---------------"
    echo "高复杂度文件:"
    for file in \
        "profiling/tracy.rs" \
        "network/key_exchange.rs" \
        "scripting/wasm_support.rs" \
        "resources/manager.rs"
    do
        if [ -f "$PROJECT_ROOT/game_engine/src/$file" ]; then
            count=$(grep -c "#\[cfg" "$PROJECT_ROOT/game_engine/src/$file" 2>/dev/null || echo "0")
            echo "  $file: $count"
        fi
    done
    echo ""
    
    echo "5. 测试文件统计"
    echo "---------------"
    echo "测试文件数:"
    find "$PROJECT_ROOT/game_engine/src" -name "test*.rs" -o -name "*test*.rs" -o -name "tests.rs" | wc -l | sed 's/^/  /'
    echo ""
    
    echo "6. unsafe使用统计"
    echo "-----------------"
    echo "unsafe块数量:"
    grep -r "unsafe {" "$PROJECT_ROOT/game_engine/src" --include="*.rs" | wc -l | sed 's/^/  /'
    echo "unwrap使用:"
    grep -r "\.unwrap()" "$PROJECT_ROOT/game_engine/src" --include="*.rs" | wc -l | sed 's/^/  /'
    echo "expect使用:"
    grep -r "\.expect(" "$PROJECT_ROOT/game_engine/src" --include="*.rs" | wc -l | sed 's/^/  /'
    echo ""
    
    echo "7. 文档覆盖率"
    echo "-------------"
    echo "文档注释数量:"
    grep -r "///" "$PROJECT_ROOT/game_engine/src" --include="*.rs" | wc -l | sed 's/^/  /'
    echo "模块注释数量:"
    grep -r "//!" "$PROJECT_ROOT/game_engine/src" --include="*.rs" | wc -l | sed 's/^/  /'
    echo ""
    
} | tee "$OUTPUT_FILE"

echo ""
echo "✅ 基线报告生成成功！"
echo "   报告位置: $OUTPUT_FILE"
echo ""
echo "下一步:"
echo "  1. 查看报告: cat $OUTPUT_FILE"
echo "  2. 对比基线: diff $BASELINE_DIR/metrics_*.txt"
echo "  3. 更新文档: cp $OUTPUT_FILE $BASELINE_DIR/baseline-latest.txt"
