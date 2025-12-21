#!/bin/bash

# 检查阻塞调用的脚本
# 用于确保async边界不被违反

set -e

echo "检查阻塞调用..."

# 查找所有block_on调用
echo "=== 查找所有block_on调用 ==="
find . -name "*.rs" -not -path "./target/*" -exec grep -l "block_on" {} \; | while read file; do
    echo "文件: $file"
    grep -n "block_on" "$file" | head -3
    echo "---"
done

# 查找Handle::current().block_on调用
echo "=== 查找Handle::current().block_on调用 ==="
find . -name "*.rs" -not -path "./target/*" -exec grep -l "Handle::current().block_on" {} \; | while read file; do
    echo "文件: $file"
    grep -n -A1 -B1 "Handle::current().block_on" "$file"
    echo "---"
done

# 检查是否在runtime内部使用不当的阻塞调用
echo "=== 检查潜在的runtime嵌套问题 ==="
echo "注意：以下调用需要检查是否在runtime内部使用："
find . -name "*.rs" -not -path "./target/*" -exec grep -n "block_on" {} \; | grep -v "block_in_place" | grep -v "pollster" | head -10

echo "检查完成。"
