#!/bin/bash
# 简单的文档验证脚本
# 验证SUMMARY.md中列出的文件是否存在

set -e

SUMMARY_FILE="docs/SUMMARY.md"
MISSING_FILES=0

echo "Checking SUMMARY.md for missing files..."
echo ""

# 从SUMMARY.md提取所有.md文件链接
grep -oE '\[.*\]\(([^)]+\.md)' "$SUMMARY_FILE" | sed 's/.*(\(.*\))/\1/' | while read -r link; do
    # 去除锚点
    file_path=$(echo "$link" | cut -d'#' -f1)

    # 跳过外部链接
    if [[ "$file_path" == http://* ]] || [[ "$file_path" == https://* ]]; then
        continue
    fi

    # 检查文件是否存在
    if [ ! -f "docs/$file_path" ]; then
        echo "❌ Missing: docs/$file_path"
        MISSING_FILES=$((MISSING_FILES + 1))
    fi
done

echo ""
if [ $MISSING_FILES -eq 0 ]; then
    echo "✅ All files in SUMMARY.md exist!"
    exit 0
else
    echo "❌ $MISSING_FILES file(s) missing"
    exit 1
fi
