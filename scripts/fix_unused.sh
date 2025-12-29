#!/bin/bash
# 修复unused变量的辅助脚本
set -e

echo "=== Unused Variables Fix Script ==="
echo ""

# Step 1: 备份当前状态
echo "Step 1: Creating backup..."
BACKUP_DIR="/Users/didi/Desktop/game_engine/.backup_$(date +%Y%m%d_%H%M%S)"
mkdir -p "$BACKUP_DIR"
cp -r /Users/didi/Desktop/game_engine/game_engine/src "$BACKUP_DIR/"
echo "✓ Backup created at: $BACKUP_DIR"
echo ""

# Step 2: 统计潜在问题
echo "Step 2: Counting potential issues..."
echo "Files with 'unused' naming patterns:"
grep -r "unused" /Users/didi/Desktop/game_engine/game_engine/src --include="*.rs" 2>/dev/null | wc -l || echo "0"
echo ""

# Step 3: 生成修复清单
echo "Step 3: Generating fix list..."
cat > /tmp/unused_fix_list.txt << 'EOF'
# Unused Variables Fix List - Generated 2025-12-28

## Priority 1: Core modules (检查这些文件)
- [ ] game_engine/src/lib.rs
- [ ] game_engine/src/core/engine/mod.rs
- [ ] game_engine/src/ecs/mod.rs

## Priority 2: Feature modules
- [ ] game_engine/src/physics/mod.rs
- [ ] game_engine/src/render/mod.rs

## Priority 3: Other modules
EOF
echo "✓ Fix list generated at /tmp/unused_fix_list.txt"
echo ""

# Step 4: 查找可能的unused模式
echo "Step 4: Searching for unused patterns..."
echo ""
echo "=== Potential unused parameters ==="
grep -rn "fn.*unused" /Users/didi/Desktop/game_engine/game_engine/src --include="*.rs" 2>/dev/null | head -10 || echo "No patterns found"
echo ""

echo "=== Potential unused variables ==="
grep -rn "let unused" /Users/didi/Desktop/game_engine/game_engine/src --include="*.rs" 2>/dev/null | head -10 || echo "No patterns found"
echo ""

# Step 5: 生成修复建议
echo "=== Manual Fix Guidelines ==="
cat << 'EOF'
For each file:

1. Add _ prefix to truly unused parameters:
   fn process(&self, data: Data, _unused: i32)

2. For API compatibility, add local allow:
   #[allow(unused_variables)]
   fn process(&self, data: Data, unused: i32) {
       let _ = unused;  // Explicitly ignore
   }

3. Remove mut when not needed:
   let data = Data::new();  // not mut

4. Test after each fix:
   cargo build
   cargo test
EOF
echo ""

echo "=== Script Complete ==="
echo "Next steps:"
echo "  1. Review /tmp/unused_fix_list.txt"
echo "  2. Manually fix each file following guidelines"
echo "  3. Run: cargo build && cargo test"
echo "  4. Remove unused_variables from lib.rs when done"
