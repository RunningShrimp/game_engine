#!/bin/bash
# 分析TODO/FIXME标记的脚本

set -e

echo "# TODO/FIXME标记分析报告"
echo ""
echo "**生成日期**: $(date '+%Y-%m-%d')"
echo ""

# 统计总数（排除target目录和.d文件）
TOTAL=$(grep -r -E "TODO|FIXME|XXX|HACK|BUG" --include="*.rs" --include="*.md" \
    --exclude-dir=target --exclude-dir=.git \
    src/ docs/ game_engine_simd/src/ game_engine_hardware/src/ 2>/dev/null | \
    grep -v "\.d:" | wc -l | tr -d ' ')

echo "## 总览"
echo ""
echo "- **总标记数**: $TOTAL"
echo ""

# 按类型分类
echo "## 按类型分类"
echo ""
TODO_COUNT=$(grep -r -E "\bTODO\b" --include="*.rs" --include="*.md" \
    --exclude-dir=target --exclude-dir=.git \
    src/ docs/ game_engine_simd/src/ game_engine_hardware/src/ 2>/dev/null | \
    grep -v "\.d:" | wc -l | tr -d ' ')

FIXME_COUNT=$(grep -r -E "\bFIXME\b" --include="*.rs" --include="*.md" \
    --exclude-dir=target --exclude-dir=.git \
    src/ docs/ game_engine_simd/src/ game_engine_hardware/src/ 2>/dev/null | \
    grep -v "\.d:" | wc -l | tr -d ' ')

XXX_COUNT=$(grep -r -E "\bXXX\b" --include="*.rs" --include="*.md" \
    --exclude-dir=target --exclude-dir=.git \
    src/ docs/ game_engine_simd/src/ game_engine_hardware/src/ 2>/dev/null | \
    grep -v "\.d:" | wc -l | tr -d ' ')

HACK_COUNT=$(grep -r -E "\bHACK\b" --include="*.rs" --include="*.md" \
    --exclude-dir=target --exclude-dir=.git \
    src/ docs/ game_engine_simd/src/ game_engine_hardware/src/ 2>/dev/null | \
    grep -v "\.d:" | wc -l | tr -d ' ')

BUG_COUNT=$(grep -r -E "\bBUG\b" --include="*.rs" --include="*.md" \
    --exclude-dir=target --exclude-dir=.git \
    src/ docs/ game_engine_simd/src/ game_engine_hardware/src/ 2>/dev/null | \
    grep -v "\.d:" | wc -l | tr -d ' ')

echo "- **TODO**: $TODO_COUNT"
echo "- **FIXME**: $FIXME_COUNT"
echo "- **XXX**: $XXX_COUNT"
echo "- **HACK**: $HACK_COUNT"
echo "- **BUG**: $BUG_COUNT"
echo ""

# 按模块分类
echo "## 按模块分类"
echo ""
echo "### 主项目 (src/)"
for module in core domain render physics audio ai network xr editor scripting scene resources performance config platform plugins; do
    if [ -d "src/$module" ]; then
        count=$(grep -r -E "TODO|FIXME|XXX|HACK|BUG" --include="*.rs" "src/$module/" 2>/dev/null | wc -l | tr -d ' ')
        if [ "$count" -gt 0 ]; then
            echo "- **$module/**: $count"
        fi
    fi
done

echo ""
echo "### 子项目"
if [ -d "game_engine_simd/src" ]; then
    simd_count=$(grep -r -E "TODO|FIXME|XXX|HACK|BUG" --include="*.rs" "game_engine_simd/src/" 2>/dev/null | wc -l | tr -d ' ')
    if [ "$simd_count" -gt 0 ]; then
        echo "- **game_engine_simd/**: $simd_count"
    fi
fi

if [ -d "game_engine_hardware/src" ]; then
    hw_count=$(grep -r -E "TODO|FIXME|XXX|HACK|BUG" --include="*.rs" "game_engine_hardware/src/" 2>/dev/null | wc -l | tr -d ' ')
    if [ "$hw_count" -gt 0 ]; then
        echo "- **game_engine_hardware/**: $hw_count"
    fi
fi

echo ""
echo "## 阻塞性TODO识别"
echo ""
echo "### 关键词：compile/error/fix/broken/missing/implement"
echo ""

BLOCKING=$(grep -r -E "TODO.*(compile|error|fix|broken|missing|implement|blocking|blocked)|FIXME.*(compile|error|fix|broken|missing|implement|blocking|blocked)" \
    --include="*.rs" --exclude-dir=target --exclude-dir=.git \
    src/ 2>/dev/null | grep -v "\.d:" | head -30)

if [ -z "$BLOCKING" ]; then
    echo "未发现明显的阻塞性TODO。"
else
    echo "$BLOCKING" | while IFS= read -r line; do
        file=$(echo "$line" | cut -d: -f1)
        content=$(echo "$line" | cut -d: -f2-)
        echo "- \`$file\`: $content"
    done
fi

echo ""
echo "## 按优先级分类（估计）"
echo ""
echo "### 功能完善（40%）"
echo "- 脚本系统集成"
echo "- UI系统实现"
echo "- XR功能完善"
echo ""
echo "### 性能优化（30%）"
echo "- GPU优化"
echo "- 并行化"
echo "- 硬件特定优化"
echo ""
echo "### 代码重构（20%）"
echo "- 重复代码清理"
echo "- 架构一致性"
echo "- API标准化"
echo ""
echo "### 文档和测试（10%）"
echo "- 文档补充"
echo "- 测试完善"
echo ""
echo "---"
echo ""
echo "**注意**: 此报告基于关键词匹配生成，实际分类可能需要人工审查。"
