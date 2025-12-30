#!/bin/bash
# 代码覆盖率检查脚本

set -e

echo "📊 生成代码覆盖率报告..."

if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "📦 安装cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

cargo tarpaulin \
    --workspace \
    --out Lcov \
    --output-dir target/coverage \
    --exclude-files "*/tests/*" \
    --exclude-files "*/benches/*" \
    --timeout 300

echo "✅ 覆盖率报告已生成: target/coverage/lcov.info"
