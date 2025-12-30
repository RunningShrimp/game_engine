#!/bin/bash
# 统计异步操作使用情况

echo "=== 异步操作使用统计 ==="
echo ""

# 统计async fn使用
echo "1. async fn 定义统计:"
grep -r "async fn" game_engine/src/ --include="*.rs" | wc -l | xargs echo "   总数:"

# 按文件统计
echo ""
echo "2. 按文件统计 (top 20):"
grep -r "async fn" game_engine/src/ --include="*.rs" | cut -d: -f1 | sort | uniq -c | sort -rn | head -20 | awk '{print "   " $2 ": " $1 " 处"}'

# 统计await使用
echo ""
echo "3. await 使用统计:"
grep -r "\.await" game_engine/src/ --include="*.rs" | wc -l | xargs echo "   总数:"

# 统计async trait使用
echo ""
echo "4. async trait 定义:"
grep -r "async fn" game_engine/src/ --include="*.rs" -B 5 | grep "trait" | wc -l | xargs echo "   异步trait数:"

# 按模块统计
echo ""
echo "5. 按模块统计:"
for module in domain ecs core engine resources network physics render audio ai animation; do
    if [ -d "game_engine/src/$module" ]; then
        count=$(grep -r "async fn" game_engine/src/$module/ --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
        if [ "$count" -gt 0 ]; then
            echo "   $module/: $count 处"
        fi
    fi
done

echo ""
echo "=== 分析完成 ==="
