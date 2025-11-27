#!/bin/bash

# Unsafe代码审查脚本
# 用于检查src/performance/simd/目录下的unsafe代码安全文档完整性

set -e

echo "=== Rust游戏引擎 Unsafe代码安全审查脚本 ==="
echo "检查目标: src/performance/simd/"
echo

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 计数器
TOTAL_UNSAFE_FUNCS=0
DOCUMENTED_FUNCS=0
MISSING_DOCS_FUNCS=0

# 检查函数是否有完整的安全文档
check_function_documentation() {
    local file=$1
    local func_name=$2
    local line_num=$3
    
    echo -e "${BLUE}检查函数: ${func_name} (${file}:${line_num})${NC}"
    
    # 检查是否有Safety章节 - 使用更宽松的搜索
    if ! sed -n "${line_num},/^pub unsafe fn ${func_name}/p" "$file" | grep -q "# Safety"; then
        echo -e "${RED}  ❌ 缺少 # Safety 章节${NC}"
        return 1
    fi
    
    # 检查是否有Panics章节
    if ! sed -n "${line_num},/^pub unsafe fn ${func_name}/p" "$file" | grep -q "# Panics"; then
        echo -e "${RED}  ❌ 缺少 # Panics 章节${NC}"
        return 1
    fi
    
    # 检查是否有Examples章节
    if ! sed -n "${line_num},/^pub unsafe fn ${func_name}/p" "$file" | grep -q "# Examples"; then
        echo -e "${RED}  ❌ 缺少 # Examples 章节${NC}"
        return 1
    fi
    
    # 检查是否有debug_assert
    if ! grep -A 50 "pub unsafe fn ${func_name}" "$file" | grep -q "debug_assert"; then
        echo -e "${YELLOW}  ⚠️  建议添加 debug_assert 检查${NC}"
    fi
    
    echo -e "${GREEN}  ✅ 文档完整${NC}"
    return 0
}

# 查找所有pub unsafe函数
echo "正在扫描pub unsafe函数..."
echo

# 扫描目标目录
for file in src/performance/simd/**/*.rs; do
    if [ -f "$file" ]; then
        echo -e "${BLUE}处理文件: $file${NC}"
        
        # 使用grep查找pub unsafe函数定义
        while IFS= read -r line; do
            if [[ $line =~ pub[[:space:]]+unsafe[[:space:]]+fn[[:space:]]+([a-zA-Z_][a-zA-Z0-9_]*) ]]; then
                func_name="${BASH_REMATCH[1]}"
                line_num=$(grep -n "pub unsafe fn $func_name" "$file" | cut -d: -f1)
                
                TOTAL_UNSAFE_FUNCS=$((TOTAL_UNSAFE_FUNCS + 1))
                
                if check_function_documentation "$file" "$func_name" "$line_num"; then
                    DOCUMENTED_FUNCS=$((DOCUMENTED_FUNCS + 1))
                else
                    MISSING_DOCS_FUNCS=$((MISSING_DOCS_FUNCS + 1))
                fi
                
                echo
            fi
        done < <(grep -n "pub unsafe fn" "$file")
    fi
done

# 生成报告
echo "=== 审查报告 ==="
echo -e "总pub unsafe函数数量: ${BLUE}$TOTAL_UNSAFE_FUNCS${NC}"
echo -e "文档完整的函数: ${GREEN}$DOCUMENTED_FUNCS${NC}"
echo -e "文档不完整的函数: ${RED}$MISSING_DOCS_FUNCS${NC}"

if [ $MISSING_DOCS_FUNCS -eq 0 ]; then
    echo -e "${GREEN}🎉 所有pub unsafe函数都有完整的安全文档！${NC}"
    exit_code=0
else
    echo -e "${RED}⚠️  还有 $MISSING_DOCS_FUNCS 个函数需要完善文档${NC}"
    exit_code=1
fi

# 检查编译状态
echo
echo "=== 编译检查 ==="
echo "正在检查代码编译状态..."

if command -v cargo >/dev/null 2>&1; then
    if cargo check --lib; then
    echo -e "${GREEN}✅ 代码编译检查通过${NC}"
    compile_status=0
else
    echo -e "${RED}❌ 代码编译检查失败${NC}"
    compile_status=1
fi

echo
echo "=== 总结 ==="
echo "1. 文档完整性: $DOCUMENTED_FUNCS/$TOTAL_UNSAFE_FUNCS 函数有完整文档"
echo "2. 编译状态: $([ $compile_status -eq 0 ] && echo '通过' || echo '失败')"

# 设置退出码
if [ $MISSING_DOCS_FUNCS -eq 0 ] && [ $compile_status -eq 0 ]; then
    echo -e "${GREEN}🎯 P1级别修复任务完成！${NC}"
    exit 0
else
    echo -e "${RED}❌ 还有问题需要解决${NC}"
    exit 1
fi