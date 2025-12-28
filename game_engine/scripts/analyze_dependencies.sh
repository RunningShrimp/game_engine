#!/bin/bash
# 模块依赖分析工具

echo "=== 游戏引擎模块依赖分析 ==="
echo ""

# 定义主要模块
MODULES=(
    "ai"
    "animation"
    "audio"
    "config"
    "core"
    "domain"
    "ecs"
    "editor"
    "network"
    "performance"
    "physics"
    "platform"
    "plugins"
    "profiling"
    "render"
    "resources"
    "scene"
    "scripting"
    "services"
    "ui"
    "xr"
)

echo "## 模块依赖矩阵"
echo ""

# 为每个模块找出它依赖的其他模块
for MODULE in "${MODULES[@]}"; do
    echo "### $MODULE 依赖于:"
    DEPS=$(grep -r "^use crate::$MODULE" /Users/wangbiao/Desktop/project/game_engine/game_engine/src --include="*.rs" 2>/dev/null | \
           sed 's/.*use crate::[a-z_]*:://' | \
           sed 's/::.*//' | \
           sort | uniq | \
           grep -E "^(${MODULES[*]})$" | \
           tr '\n' ' ')
    if [ -n "$DEPS" ]; then
        echo "   $DEPS"
    else
        echo "   (无)"
    fi
    echo ""
done

echo ""
echo "## 可能的循环依赖检查"
echo ""

# 检查render是否依赖domain
if grep -q "^use crate::domain" /Users/wangbiao/Desktop/project/game_engine/game_engine/src/render/*.rs 2>/dev/null; then
    echo "⚠️  render → domain"
fi

# 检查domain是否依赖render
if grep -q "^use crate::render" /Users/wangbiao/Desktop/project/game_engine/game_engine/src/domain/*.rs 2>/dev/null; then
    echo "⚠️  domain → render"
fi

# 检查ecs是否依赖core
if grep -q "^use crate::core" /Users/wangbiao/Desktop/project/game_engine/game_engine/src/ecs/*.rs 2>/dev/null; then
    echo "⚠️  ecs → core"
fi

# 检查core是否依赖ecs
if grep -q "^use crate::ecs" /Users/wangbiao/Desktop/project/game_engine/game_engine/src/core/*.rs 2>/dev/null; then
    echo "⚠️  core → ecs"
fi

echo ""
echo "## 跨层依赖检查"
echo ""

# 检查高层模块是否直接依赖底层实现细节
if grep -q "^use crate::core::engine::" /Users/wangbiao/Desktop/project/game_engine/game_engine/src/domain/*.rs 2>/dev/null; then
    echo "⚠️  domain 直接依赖 core::engine (应该通过抽象)"
fi

if grep -q "^use crate::render::wgpu_modules::" /Users/wangbiao/Desktop/project/game_engine/game_engine/src/core/engine/*.rs 2>/dev/null; then
    echo "⚠️  core::engine 直接依赖 render::wgpu_modules (应该通过抽象)"
fi
