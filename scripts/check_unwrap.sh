#!/bin/bash
# 检查代码库中新增的unwrap()和expect()使用
# 用于监控代码质量，确保关键路径不使用unwrap()

set -e

echo "检查unwrap()和expect()使用情况..."

# 统计总数
TOTAL=$(grep -r "\.unwrap()\|\.expect(" --include="*.rs" game_engine/src | wc -l | tr -d ' ')

echo "总共发现 $TOTAL 处unwrap()/expect()使用"

# 检查关键文件
echo ""
echo "关键路径检查："
echo "=============="

KEY_FILES=(
    "game_engine/src/core/engine/engine.rs"
    "game_engine/src/resources/manager.rs"
    "game_engine/src/render/wgpu_utils.rs"
    "game_engine/src/network/client.rs"
    "game_engine/src/network/server.rs"
)

for file in "${KEY_FILES[@]}"; do
    if [ -f "$file" ]; then
        COUNT=$(grep -c "\.unwrap()\|\.expect(" "$file" 2>/dev/null || echo "0")
        if [ "$COUNT" -gt 0 ]; then
            echo "  $file: $COUNT 处"
            # 显示具体位置（前5个）
            grep -n "\.unwrap()\|\.expect(" "$file" | head -5 | sed 's/^/    /'
        fi
    fi
done

echo ""
echo "检查完成。"
echo ""
echo "注意："
echo "- 关键路径（引擎初始化、资源加载、网络连接）应避免使用unwrap()"
echo "- 测试代码中的unwrap()可以接受，但应添加注释说明"
echo "- 对于确实需要panic的场景，使用expect()并提供清晰的错误消息"

exit 0

