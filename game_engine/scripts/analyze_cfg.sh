#!/bin/bash
# 分析条件编译使用情况
# 识别可以优化的模式

echo "=== 条件编译分析报告 ==="
echo ""

echo "1. 按feature统计使用次数："
grep -rh "#\[cfg(feature" src/ --include="*.rs" | \
    sed 's/.*feature("\(.*\)").*/\1/' | \
    sort | uniq -c | sort -rn | head -20

echo ""
echo "2. 重复cfg块最多的文件："
for file in $(find src/ -name "*.rs" -type f); do
    count=$(grep -c "#\[cfg(" "$file" 2>/dev/null || echo 0)
    if [ "$count" -gt 10 ]; then
        echo "  $file: $count 处"
    fi
done | sort -t: -k2 -rn | head -10

echo ""
echo "3. 可以合并的相邻cfg块："
for file in $(find src/ -name "*.rs" -type f); do
    # 查找连续的相同cfg
    prev_line=""
    prev_cfg=""
    line_num=0
    while IFS= read -r line; do
        line_num=$((line_num + 1))
        if echo "$line" | grep -q "#\[cfg(feature"; then
            current_cfg=$(echo "$line" | sed 's/.*feature("\(.*\)").*/\1/')
            if [ "$current_cfg" = "$prev_cfg" ] && [ $((line_num - prev_line)) -eq 1 ]; then
                echo "  $file:${prev_line}: 相邻的 cfg(feature = \"$current_cfg\")"
            fi
            prev_cfg="$current_cfg"
            prev_line=$line_num
        fi
    done < "$file"
done | head -20

echo ""
echo "4. 缺失文档注释的features："
grep -E "^secure_key_exchange|^insecure_key_exchange|^tracy|^xr|^gltf|^wasm" Cargo.toml | \
    grep -v "^##" | \
    while read line; do
        feature=$(echo "$line" | cut -d= -f1 | xargs)
        echo "  - $feature"
    done

echo ""
echo "分析完成！"
