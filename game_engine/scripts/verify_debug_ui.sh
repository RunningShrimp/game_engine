#!/bin/bash
# 验证Debug UI模块的编译

echo "Verifying Debug UI module compilation..."

# 检查语法
cargo check --lib 2>&1 | grep -E "(error|warning)" | head -20

if [ $? -eq 0 ]; then
    echo "Found some issues. Running full check..."
    cargo check --lib
else
    echo "No obvious errors found in initial check"
fi
