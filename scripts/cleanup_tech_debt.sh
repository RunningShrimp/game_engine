#!/bin/bash

# 技术债务清理脚本
#
# 自动清理代码库中的 TODO、FIXME、XXX、HACK 标记
# 生成技术债务报告
# 运行方式: ./scripts/cleanup_tech_debt.sh [directory]

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 参数检查
SCAN_DIR="${1:-game_engine/src}"
OUTPUT_FILE="tech_debt_report.md"
DRY_RUN="${DRY_RUN:-false}"

# 统计变量
TODO_COUNT=0
FIXME_COUNT=0
XXX_COUNT=0
HACK_COUNT=0
TOTAL_COUNT=0

# 按模块统计
declare -A MODULE_DEBT
MODULE_DEBT[""]=0

# 输出文件
OUTPUT_DIR="reports"
mkdir -p "$OUTPUT_DIR"

echo -e "${BLUE}=== 游戏引擎技术债务清理工具 ===${NC}\n"
echo -e "${YELLOW}扫描目录: $SCAN_DIR${NC}\n"
echo -e "${YELLOW}输出文件: $OUTPUT_DIR/$OUTPUT_FILE${NC}\n"
echo -e "${YELLOW}清理模式: $([ "$DRY_RUN" = "true" ] && echo "干运行（不修改文件）" || echo "清理模式（会标记已清理项）")${NC}\n"

# 扫描函数
scan_file() {
    local file="$1"
    local module=$(echo "$file" | sed 's|/|.|g' | cut -d'.' -f3)
    
    # 统计每种标记
    local todo=$(grep -c "TODO" "$file" 2>/dev/null || echo 0)
    local fixme=$(grep -c "FIXME" "$file" 2>/dev/null || echo 0)
    local xxx=$(grep -c "XXX" "$file" 2>/dev/null || echo 0)
    local hack=$(grep -c "HACK" "$file" 2>/dev/null || echo 0)
    
    local total=$((todo + fixme + xxx + hack))
    
    # 累计总数
    TODO_COUNT=$((TODO_COUNT + todo))
    FIXME_COUNT=$((FIXME_COUNT + fixme))
    XXX_COUNT=$((XXX_COUNT + xxx))
    HACK_COUNT=$((HACK_COUNT + hack))
    TOTAL_COUNT=$((TOTAL_COUNT + total))
    
    # 模块级别统计
    MODULE_DEBT["$module"]=$((MODULE_DEBT["$module"] + total))
    
    # 如果有技术债务，输出详细信息
    if [ $total -gt 0 ]; then
        echo -e "  ${RED}✗${NC} $module: ${YELLOW}$total${NC} 个标记 (TODO:$todo, FIXME:$fixme, XXX:$xxx, HACK:$hack)"
    fi
    
    # 如果不是干运行，清理已处理的项
    if [ "$DRY_RUN" = "false" ] && [ $total -gt 0 ]; then
        # 将已处理的标记添加到 .debt-cleaned 后缀
        # 使用临时文件避免原地修改问题
        local temp_file="${file}.tmp"
        
        # 复制原文件
        cp "$file" "$temp_file"
        
        # 清理标记（在注释中标记为已处理）
        sed -i '' -E 's/(TODO|FIXME|XXX|HACK)/\1 (debt-cleaned) :/\1/g' "$temp_file"
        
        # 如果成功，替换原文件
        if [ $? -eq 0 ]; then
            mv "$temp_file" "$file"
            echo -e "    ${GREEN}✓${NC} 已清理 $module 中的标记"
        else
            rm "$temp_file"
            echo -e "    ${RED}✗${NC} 清理 $module 失败"
        fi
    fi
}

# 递归扫描目录
scan_directory() {
    local dir="$1"
    
    echo -e "${BLUE}扫描目录: $dir${NC}\n"
    
    # 扫描所有 .rs 和 .wgsl 文件
    find "$dir" -type f \( -name "*.rs" -o -name "*.wgsl" \) | while read -r file; do
        scan_file "$file"
    done
    
    # 递归扫描子目录
    find "$dir" -type d | while read -r subdir; do
        scan_directory "$subdir"
    done
}

# 生成Markdown报告
generate_report() {
    local report_file="$OUTPUT_DIR/$OUTPUT_FILE"
    
    echo "# 技术债务报告" > "$report_file"
    echo "" >> "$report_file"
    echo "**生成时间**: $(date '+%Y-%m-%d %H:%M:%S')" >> "$report_file"
    echo "**扫描目录**: $SCAN_DIR" >> "$report_file"
    echo "" >> "$report_file"
    echo "## 统计摘要" >> "$report_file"
    echo "" >> "$report_file"
    echo "| 标记类型 | 数量 | 占比 |" >> "$report_file"
    echo "|---------|------|------|" >> "$report_file"
    echo "| TODO | $TODO_COUNT | $(awk "BEGIN {print (\"$TODO_COUNT\"/\"$TOTAL_COUNT\")*100}" <<< "" | awk '{printf \"%.1f%%\", $1}')" |" >> "$report_file"
    echo "| FIXME | $FIXME_COUNT | $(awk "BEGIN {print (\"$FIXME_COUNT\"/\"$TOTAL_COUNT\")*100}" <<< "" | awk '{printf \"%.1f%%\", $1}')" |" >> "$report_file"
    echo "| XXX | $XXX_COUNT | $(awk "BEGIN {print (\"$XXX_COUNT\"/\"$TOTAL_COUNT\")*100}" <<< "" | awk '{printf \"%.1f%%\", $1}')" |" >> "$report_file"
    echo "| HACK | $HACK_COUNT | $(awk "BEGIN {print (\"$HACK_COUNT\"/\"$TOTAL_COUNT\")*100}" <<< "" | awk '{printf \"%.1f%%\", $1}')" |" >> "$report_file"
    echo "| **总计** | **$TOTAL_COUNT** | **100%** |" >> "$report_file"
    echo "" >> "$report_file"
    echo "## 模块级统计" >> "$report_file"
    echo "" >> "$report_file"
    echo "| 模块 | 债务数量 |" >> "$report_file"
    echo "|------|-----------|" >> "$report_file"
    
    # 排序模块
    for module in $(echo "${!MODULE_DEBT[@]}" | tr ' ' '\n' | sort -t '=' -k2 -nr); do
        echo "| $module | ${MODULE_DEBT[$module]} |" >> "$report_file"
    done
    
    echo "" >> "$report_file"
    echo "## 优先级分类" >> "$report_file"
    echo "" >> "$report_file"
    echo "### 高优先级（FIXME）" >> "$report_file"
    echo "- 需要立即处理的已知问题" >> "$report_file"
    echo "- 影响稳定性和性能的代码" >> "$report_file"
    echo "" >> "$report_file"
    echo "### 中优先级（TODO）" >> "$report_file"
    echo "- 功能改进项" >> "$report_file"
    echo "- 未来的优化计划" >> "$report_file"
    echo "" >> "$report_file"
    echo "### 低优先级（XXX/HACK）" >> "$report_file"
    echo "- 代码注释" >> "$report_file"
    echo "- 临时解决方案" >> "$report_file"
    echo "" >> "$report_file"
    echo "## 建议和最佳实践" >> "$report_file"
    echo "" >> "$report_file"
    echo "1. **定期清理技术债务**" >> "$report_file"
    echo "   - 每次发布前运行此脚本" >> "$report_file"
    echo "   - 设置技术债务清理的Sprint周期" >> "$report_file"
    echo "" >> "$report_file"
    echo "2. **改进代码质量**" >> "$report_file"
    echo "   - 减少HACK的使用" >> "$report_file"
    echo "   - 添加足够的测试覆盖" >> "$report_file"
    echo "   - 完善代码文档" >> "$report_file"
    echo "" >> "$report_file"
    echo "3. **建立代码审查流程**" >> "$report_file"
    echo "   - 所有PR必须检查技术债务" >> "$report_file"
    echo "   - 重大变更需要团队审查" >> "$report_file"
    echo "" >> "$report_file"
    echo "## 清理进度" >> "$report_file"
    echo "" >> "$report_file"
    
    if [ "$DRY_RUN" = "false" ]; then
        echo "✅ 已执行清理操作" >> "$report_file"
        echo "- TODO标记已添加 `.debt-cleaned` 后缀" >> "$report_file"
        echo "- FIXME标记已添加 `.debt-cleaned` 后缀" >> "$report_file"
        echo "- XXX标记已添加 `.debt-cleaned` 后缀" >> "$report_file"
        echo "- HACK标记已添加 `.debt-cleaned` 后缀" >> "$report_file"
    else
        echo "📊 仅扫描，未执行清理操作" >> "$report_file"
    fi
}

# 主执行流程
main() {
    echo -e "${GREEN}开始技术债务扫描...${NC}\n"
    
    # 检查目录是否存在
    if [ ! -d "$SCAN_DIR" ]; then
        echo -e "${RED}错误: 目录不存在: $SCAN_DIR${NC}\n"
        exit 1
    fi
    
    # 扫描目录
    scan_directory "$SCAN_DIR"
    
    # 生成报告
    generate_report
    
    # 输出统计
    echo -e "\n${GREEN}=== 扫描完成 ===${NC}\n"
    echo -e "${BLUE}统计结果:${NC}\n"
    echo -e "  TODO:    ${YELLOW}$TODO_COUNT${NC}"
    echo -e "  FIXME:  ${YELLOW}$FIXME_COUNT${NC}"
    echo -e "  XXX:     ${YELLOW}$XXX_COUNT${NC}"
    echo -e "  HACK:    ${YELLOW}$HACK_COUNT${NC}"
    echo -e "  总计:    ${GREEN}$TOTAL_COUNT${NC}"
    echo -e "\n${BLUE}报告已生成: $OUTPUT_DIR/$OUTPUT_FILE${NC}\n"
    
    # 显示模块级统计（Top 10）
    echo -e "\n${BLUE}技术债务最多的模块（Top 10）:${NC}\n"
    for module in $(echo "${!MODULE_DEBT[@]}" | tr ' ' '\n' | sort -t '=' -k2 -nr | head -10); do
        echo -e "  ${YELLOW}${MODULE_DEBT[$module]}${NC} 个标记"
    done
    
    # 显示清理模式
    if [ "$DRY_RUN" = "false" ]; then
        echo -e "\n${YELLOW}提示: 运行 ./scripts/cleanup_tech_debt.sh $SCAN_DIR clean 来清理标记${NC}\n"
    fi
    
    echo -e "${GREEN}✓ 扫描完成！${NC}\n"
}

# 脚本入口点
main "$@"

