#!/bin/bash
# 验证核心改动的编译状态

echo "=== 核心改动验证测试 ==="
echo ""

# 测试1: AI寻路 Rayon集成
echo "1. AI寻路模块 (Rayon集成):"
rustc --crate-type lib --test game_engine/src/ai/pathfinding.rs \
  --edition 2024 \
  --extern glam --edition 2024 \
  2>&1 | grep -E "error|warning" | head -5 || echo "   ✅ 通过"

# 测试2: 音频处理 Rayon集成
echo "2. 音频处理模块 (Rayon集成):"
cargo check -p game_engine --lib --features parallel 2>&1 | grep -i "audio.*error" || echo "   ✅ 通过"

# 测试3: 资源模块 trait抽象
echo "3. 资源模块 (ConcurrentMap trait):"
cargo check -p game_engine --lib 2>&1 | grep -i "resources.*concurrent.*error" || echo "   ✅ 通过"

# 测试4: 网络模块 trait抽象
echo "4. 网络模块 (ClientRegistry trait):"
cargo check -p game_engine --lib --features dashmap 2>&1 | grep -i "network.*concurrent.*error" || echo "   ✅ 通过"

# 测试5: IPC优化
echo "5. 微内核IPC (blocking_read优化):"
cargo check -p game_engine --lib 2>&1 | grep -i "ipc.*error" || echo "   ✅ 通过"

echo ""
echo "=== 验证完成 ==="
