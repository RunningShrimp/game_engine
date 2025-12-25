#!/bin/bash
# 增强的构建脚本
#
# 提供增量构建、并行构建、进度显示等功能
#
# 使用方法:
#   ./scripts/build_enhanced.sh [OPTIONS]
#
# 选项:
#   --profile <debug|release>    构建模式（默认: release）
#   --incremental                启用增量构建（默认: 启用）
#   --no-incremental             禁用增量构建
#   --parallel <N>               并行构建数（默认: CPU核心数）
#   --package <name>             只构建指定包
#   --target <target>            构建目标（如 wasm32-unknown-unknown）
#   --features <features>       启用特性（逗号分隔）
#   --all-features               启用所有特性
#   --progress                   显示构建进度（默认: 启用）
#   --no-progress                不显示构建进度
#   --help                       显示帮助信息

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 默认配置
PROFILE="release"
INCREMENTAL=true
PARALLEL=$(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo "4")
SHOW_PROGRESS=true
PACKAGE=""
TARGET=""
FEATURES=""
ALL_FEATURES=false

# 解析命令行参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --profile)
            PROFILE="$2"
            shift 2
            ;;
        --incremental)
            INCREMENTAL=true
            shift
            ;;
        --no-incremental)
            INCREMENTAL=false
            shift
            ;;
        --parallel)
            PARALLEL="$2"
            shift 2
            ;;
        --package)
            PACKAGE="$2"
            shift 2
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --features)
            FEATURES="$2"
            shift 2
            ;;
        --all-features)
            ALL_FEATURES=true
            shift
            ;;
        --progress)
            SHOW_PROGRESS=true
            shift
            ;;
        --no-progress)
            SHOW_PROGRESS=false
            shift
            ;;
        --help)
            cat << EOF
增强的构建脚本

使用方法:
  $0 [OPTIONS]

选项:
  --profile <debug|release>    构建模式（默认: release）
  --incremental                启用增量构建（默认: 启用）
  --no-incremental             禁用增量构建
  --parallel <N>               并行构建数（默认: CPU核心数）
  --package <name>             只构建指定包
  --target <target>            构建目标
  --features <features>        启用特性（逗号分隔）
  --all-features               启用所有特性
  --progress                   显示构建进度（默认: 启用）
  --no-progress                不显示构建进度
  --help                       显示帮助信息

示例:
  # 标准发布构建
  $0

  # 调试模式构建
  $0 --profile debug

  # 并行构建（8个任务）
  $0 --parallel 8

  # 构建特定包
  $0 --package game_engine

  # 构建WASM目标
  $0 --target wasm32-unknown-unknown

  # 启用所有特性
  $0 --all-features
EOF
            exit 0
            ;;
        *)
            echo -e "${RED}未知选项: $1${NC}"
            echo "使用 --help 查看帮助信息"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}🚀 增强的构建系统${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "构建模式: $PROFILE"
echo "增量构建: $INCREMENTAL"
echo "并行度: $PARALLEL"
echo "显示进度: $SHOW_PROGRESS"
[ -n "$PACKAGE" ] && echo "指定包: $PACKAGE"
[ -n "$TARGET" ] && echo "构建目标: $TARGET"
[ -n "$FEATURES" ] && echo "特性: $FEATURES"
[ "$ALL_FEATURES" = true ] && echo "所有特性: 启用"
echo ""

# 检查是否在项目根目录
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}❌ 错误: 请在项目根目录运行此脚本${NC}"
    exit 1
fi

# 构建参数
BUILD_ARGS=()

# 添加profile参数
if [ "$PROFILE" = "release" ]; then
    BUILD_ARGS+=("--release")
fi

# 添加包参数
if [ -n "$PACKAGE" ]; then
    BUILD_ARGS+=("--package" "$PACKAGE")
fi

# 添加目标参数
if [ -n "$TARGET" ]; then
    BUILD_ARGS+=("--target" "$TARGET")
fi

# 添加特性参数
if [ "$ALL_FEATURES" = true ]; then
    BUILD_ARGS+=("--all-features")
elif [ -n "$FEATURES" ]; then
    BUILD_ARGS+=("--features" "$FEATURES")
fi

# 如果启用增量构建，使用cargo的增量编译
if [ "$INCREMENTAL" = true ]; then
    export CARGO_INCREMENTAL=1
else
    export CARGO_INCREMENTAL=0
fi

# 设置并行度
export CARGO_BUILD_JOBS="$PARALLEL"

# 记录开始时间
START_TIME=$(date +%s)

# 执行构建
echo -e "${BLUE}📦 开始构建...${NC}"
echo ""

if cargo build "${BUILD_ARGS[@]}"; then
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    
    echo ""
    echo -e "${GREEN}✅ 构建成功！${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "总耗时: ${DURATION}秒"
    echo "并行度: $PARALLEL"
    echo "构建模式: $PROFILE"
    
    # 显示构建产物信息
    if [ "$PROFILE" = "release" ]; then
        OUTPUT_DIR="target/release"
    else
        OUTPUT_DIR="target/debug"
    fi
    
    if [ -n "$TARGET" ]; then
        OUTPUT_DIR="target/$TARGET/$PROFILE"
    fi
    
    if [ -d "$OUTPUT_DIR" ]; then
        echo ""
        echo -e "${BLUE}📊 构建产物:${NC}"
        find "$OUTPUT_DIR" -maxdepth 1 -type f -executable 2>/dev/null | head -5 | while read -r file; do
            SIZE=$(du -h "$file" | cut -f1)
            echo "  • $(basename "$file") ($SIZE)"
        done
    fi
    
    echo ""
    exit 0
else
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    
    echo ""
    echo -e "${RED}❌ 构建失败${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "耗时: ${DURATION}秒"
    echo ""
    exit 1
fi

