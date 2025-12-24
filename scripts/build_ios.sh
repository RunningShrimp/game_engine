#!/bin/bash
# iOS构建脚本
#
# 此脚本用于构建iOS平台的游戏引擎库
#
# 使用方法:
#   ./scripts/build_ios.sh [--release] [--target <target>]
#
# 参数:
#   --release: 构建发布版本（默认是调试版本）
#   --target: 指定目标架构 (aarch64-apple-ios, x86_64-apple-ios, 或 aarch64-apple-ios-sim)
#
# 示例:
#   ./scripts/build_ios.sh
#   ./scripts/build_ios.sh --release
#   ./scripts/build_ios.sh --target aarch64-apple-ios-sim

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 默认值
BUILD_MODE="debug"
TARGET="aarch64-apple-ios"

# 解析参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --release)
            BUILD_MODE="release"
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}未知参数: $1${NC}"
            echo "使用方法: $0 [--release] [--target <target>]"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}开始构建iOS平台...${NC}"
echo "构建模式: $BUILD_MODE"
echo "目标架构: $TARGET"

# 检查是否在macOS上
if [[ "$OSTYPE" != "darwin"* ]]; then
    echo -e "${RED}错误: iOS构建只能在macOS上进行${NC}"
    exit 1
fi

# 检查Xcode是否安装
if ! command -v xcodebuild &> /dev/null; then
    echo -e "${RED}错误: 未找到Xcode，请先安装Xcode${NC}"
    exit 1
fi

# 检查rustup是否安装
if ! command -v rustup &> /dev/null; then
    echo -e "${RED}错误: 未找到rustup，请先安装Rust${NC}"
    exit 1
fi

# 添加iOS目标
echo -e "${YELLOW}添加iOS目标: $TARGET${NC}"
rustup target add "$TARGET" || {
    echo -e "${RED}错误: 无法添加iOS目标${NC}"
    exit 1
}

# 设置构建参数
BUILD_ARGS="--target $TARGET"
if [[ "$BUILD_MODE" == "release" ]]; then
    BUILD_ARGS="$BUILD_ARGS --release"
fi

# 构建
echo -e "${GREEN}开始构建...${NC}"
cd "$(dirname "$0")/.."

if cargo build $BUILD_ARGS; then
    echo -e "${GREEN}构建成功！${NC}"
    
    # 显示输出路径
    OUTPUT_DIR="target/$TARGET/$BUILD_MODE"
    echo "输出目录: $OUTPUT_DIR"
    
    # 列出生成的文件
    if [[ -d "$OUTPUT_DIR" ]]; then
        echo -e "${GREEN}生成的文件:${NC}"
        find "$OUTPUT_DIR" -name "*.a" -o -name "*.rlib" | head -5
    fi
else
    echo -e "${RED}构建失败${NC}"
    exit 1
fi

echo -e "${GREEN}iOS构建完成！${NC}"

