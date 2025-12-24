#!/bin/bash

set -e

echo "=== 代码覆盖率测试脚本 ==="
echo ""
echo "该脚本用于生成代码覆盖率报告"
echo ""
echo "用法:"
echo "  ./coverage.sh              # 生成覆盖率报告"
echo "  ./coverage.sh --html        # 生成HTML格式报告"
echo "  ./coverage.sh --clean       # 清理覆盖率数据"
echo ""

# 清理选项
if [ "$1" == "--clean" ]; then
    echo "清理覆盖率数据..."
    cargo clean
    rm -rf target/coverage/
    echo "清理完成"
    exit 0
fi

# 检查 tarpaulin 是否安装
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "错误: cargo-tarpaulin 未安装"
    echo ""
    echo "请运行以下命令安装:"
    echo "  cargo install cargo-tarpaulin"
    echo ""
    exit 1
fi

# 生成覆盖率报告
echo "正在生成代码覆盖率报告..."

if [ "$1" == "--html" ]; then
    cargo tarpaulin \
        --workspace \
        --out Html \
        --output-dir target/coverage \
        --exclude-files "*/tests/*" \
        --exclude-files "*/benches/*" \
        --exclude-files "*/examples/*" \
        --timeout 300
else
    cargo tarpaulin \
        --workspace \
        --out Lcov \
        --output-dir target/coverage \
        --exclude-files "*/tests/*" \
        --exclude-files "*/benches/*" \
        --exclude-files "*/examples/*" \
        --timeout 300
fi

echo ""
echo "覆盖率报告已生成到: target/coverage/"
echo ""
echo "使用以下命令查看报告:"
if [ "$1" == "--html" ]; then
    echo "  open target/coverage/index.html"
else
    echo "  genhtml target/coverage/lcov.info -o target/coverage/html"
fi
