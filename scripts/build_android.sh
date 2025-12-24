#!/bin/bash
# Android构建脚本
#
# 此脚本用于构建Android平台的游戏引擎库
#
# 使用方法:
#   ./scripts/build_android.sh [--release] [--target <target>] [--api-level <level>]
#
# 参数:
#   --release: 构建发布版本（默认是调试版本）
#   --target: 指定目标架构 (aarch64-linux-android, armv7-linux-androideabi, i686-linux-android, x86_64-linux-android)
#   --api-level: Android API级别 (默认: 29)
#
# 示例:
#   ./scripts/build_android.sh
#   ./scripts/build_android.sh --release
#   ./scripts/build_android.sh --target aarch64-linux-android --api-level 30

set -e

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 默认值
BUILD_MODE="debug"
TARGET="aarch64-linux-android"
API_LEVEL="29"

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
        --api-level)
            API_LEVEL="$2"
            shift 2
            ;;
        *)
            echo -e "${RED}未知参数: $1${NC}"
            echo "使用方法: $0 [--release] [--target <target>] [--api-level <level>]"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}开始构建Android平台...${NC}"
echo "构建模式: $BUILD_MODE"
echo "目标架构: $TARGET"
echo "API级别: $API_LEVEL"

# 检查rustup是否安装
if ! command -v rustup &> /dev/null; then
    echo -e "${RED}错误: 未找到rustup，请先安装Rust${NC}"
    exit 1
fi

# 检查cargo-ndk是否安装
if ! command -v cargo-ndk &> /dev/null; then
    echo -e "${YELLOW}警告: 未找到cargo-ndk，尝试安装...${NC}"
    if cargo install cargo-ndk; then
        echo -e "${GREEN}cargo-ndk安装成功${NC}"
    else
        echo -e "${RED}错误: 无法安装cargo-ndk，请手动安装: cargo install cargo-ndk${NC}"
        exit 1
    fi
fi

# 检查Android NDK
if [[ -z "$ANDROID_NDK_HOME" ]] && [[ -z "$ANDROID_NDK_ROOT" ]]; then
    echo -e "${YELLOW}警告: 未设置ANDROID_NDK_HOME或ANDROID_NDK_ROOT环境变量${NC}"
    echo "请设置Android NDK路径，例如:"
    echo "  export ANDROID_NDK_HOME=/path/to/android-ndk"
    echo "或者使用cargo-ndk的自动检测功能"
fi

# 添加Android目标
echo -e "${YELLOW}添加Android目标: $TARGET${NC}"
rustup target add "$TARGET" || {
    echo -e "${RED}错误: 无法添加Android目标${NC}"
    exit 1
}

# 设置构建参数
BUILD_ARGS="ndk --target $TARGET --platform $API_LEVEL build"
if [[ "$BUILD_MODE" == "release" ]]; then
    BUILD_ARGS="$BUILD_ARGS --release"
fi

# 构建
echo -e "${GREEN}开始构建...${NC}"
cd "$(dirname "$0")/.."

if cargo $BUILD_ARGS; then
    echo -e "${GREEN}构建成功！${NC}"
    
    # 显示输出路径
    OUTPUT_DIR="target/$TARGET/$BUILD_MODE"
    echo "输出目录: $OUTPUT_DIR"
    
    # 列出生成的文件
    if [[ -d "$OUTPUT_DIR" ]]; then
        echo -e "${GREEN}生成的文件:${NC}"
        find "$OUTPUT_DIR" -name "*.so" -o -name "*.a" | head -5
    fi
else
    echo -e "${RED}构建失败${NC}"
    exit 1
fi

echo -e "${GREEN}Android构建完成！${NC}"

