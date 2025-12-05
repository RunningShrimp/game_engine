#!/bin/bash
# 设置测试覆盖率工具

set -e

echo "Installing cargo-tarpaulin..."

# 检查是否已安装
if command -v cargo-tarpaulin &> /dev/null; then
    echo "cargo-tarpaulin is already installed"
    cargo-tarpaulin --version
else
    echo "Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin --locked
fi

echo "Setup complete!"



