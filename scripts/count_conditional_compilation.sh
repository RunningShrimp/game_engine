#!/bin/bash
# 统计条件编译使用情况

echo "=== 条件编译使用统计 ==="
echo ""

# 统计#[cfg(feature 使用
echo "1. #[cfg(feature 使用统计:"
grep -r "#\[cfg(feature" game_engine/src/ --include="*.rs" | wc -l | xargs echo "   总数:"

# 按文件统计
echo ""
echo "2. 按文件统计 (top 20):"
grep -r "#\[cfg(feature" game_engine/src/ --include="*.rs" | cut -d: -f1 | sort | uniq -c | sort -rn | head -20 | awk '{print "   " $2 ": " $1 " 处"}'

# 统计#[cfg(target 使用
echo ""
echo "3. #[cfg(target 使用统计:"
grep -r "#\[cfg(target" game_engine/src/ --include="*.rs" | wc -l | xargs echo "   总数:"

# 统计feature名称
echo ""
echo "4. 使用的feature名称:"
grep -ro 'feature = "[^"]*"' game_engine/src/ --include="*.rs" | sort | uniq -c | sort -rn | awk '{print "   " $2 ": " $1 " 次"}'

echo ""
echo "=== 分析完成 ==="
