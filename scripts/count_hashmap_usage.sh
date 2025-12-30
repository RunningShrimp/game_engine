#!/bin/bash
# 统计HashMap使用情况

echo "=== HashMap使用统计 ==="
echo ""

# 统计HashMap使用
echo "1. HashMap使用统计:"
grep -r "HashMap<" game_engine/src/ --include="*.rs" | wc -l | xargs echo "   总数:"

# 按文件统计
echo ""
echo "2. 按文件统计 (top 20):"
grep -r "HashMap<" game_engine/src/ --include="*.rs" | cut -d: -f1 | sort | uniq -c | sort -rn | head -20 | awk '{print "   " $2 ": " $1 " 处"}'

# 统计std::collections::HashMap使用
echo ""
echo "3. std::collections::HashMap使用:"
grep -r "std::collections::HashMap" game_engine/src/ --include="*.rs" | wc -l | xargs echo "   总数:"

# 按模块统计
echo ""
echo "4. 按模块统计:"
for module in domain ecs core engine resources network physics render audio ai animation concurrency config; do
    if [ -d "game_engine/src/$module" ]; then
        count=$(grep -r "HashMap<" game_engine/src/$module/ --include="*.rs" 2>/dev/null | wc -l | tr -d ' ')
        if [ "$count" -gt 0 ]; then
            echo "   $module/: $count 处"
        fi
    fi
done

# 检查DashMap使用
echo ""
echo "5. DashMap使用:"
grep -r "DashMap<" game_engine/src/ --include="*.rs" | wc -l | xargs echo "   当前使用:"

echo ""
echo "=== DashMap应用场景分析 ==="
echo "潜在应用场景（读多写少）:"
echo "  - 资源缓存"
echo "  - ECS查询"
echo "  - 配置管理"
echo "  - 事件总线"

echo ""
echo "=== 分析完成 ==="
