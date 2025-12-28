#!/bin/bash
# 修复Rust文档中的未解析链接
#
# 问题: 文档中使用了[`TypeName`]: Description格式，
# Rust将Description当作链接解析导致unresolved link警告
#
# 解决: 将冒号(:)替换为破折号(-)

set -e

echo "Fixing Rust documentation links..."

# 查找所有包含文档链接的.rs文件
find src/ -name "*.rs" -type f | while read file; do
    # 替换文档中的]: 为] - （修复链接格式）
    # 例如: [`TypeName`]: Description -> [`TypeName`] - Description
    if grep -q '\`\[.*\`\]:[^/]' "$file" 2>/dev/null; then
        echo "Processing: $file"
        # 使用sed进行替换
        sed -i '' 's/\(\`\[.*\`\]\):/\1 -/g' "$file"
    fi
done

echo "Done! Checking documentation..."

# 重新编译文档检查是否还有问题
cargo doc --no-deps --lib 2>&1 | grep "unresolved link" | wc -l

echo "Documentation links fixed!"
