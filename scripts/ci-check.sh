#!/bin/bash
# ci-check.sh
# 快速CI检查脚本 - 在提交前运行
#
# 用法:
#   ./scripts/ci-check.sh              # 运行所有检查
#   ./scripts/ci-check.sh --fast       # 快速模式(跳过coverage)
#   ./scripts/ci-check.sh --fix        # 自动修复问题

set -e

FAST_MODE=false
AUTO_FIX=false

# 解析参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --fast)
            FAST_MODE=true
            shift
            ;;
        --fix)
            AUTO_FIX=true
            shift
            ;;
        *)
            echo "Usage: $0 [--fast] [--fix]"
            exit 1
            ;;
    esac
done

# 颜色
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Quick CI Check${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

FAILURES=0

# 1. Format
echo -e "${BLUE}[1/5]${NC} Checking formatting..."
if [ "$AUTO_FIX" = true ]; then
    cargo fmt --all
    echo -e "${GREEN}✓ Code formatted${NC}"
else
    if cargo fmt --all -- --check; then
        echo -e "${GREEN}✓ Formatting OK${NC}"
    else
        echo -e "${RED}✗ Formatting issues found${NC}"
        echo "  Run 'cargo fmt --all' to fix"
        FAILURES=$((FAILURES + 1))
    fi
fi

# 2. Clippy
echo ""
echo -e "${BLUE}[2/5]${NC} Running clippy..."
if cargo clippy --workspace --all-targets -- -D warnings; then
    echo -e "${GREEN}✓ No clippy warnings${NC}"
else
    echo -e "${RED}✗ Clippy found issues${NC}"
    FAILURES=$((FAILURES + 1))
fi

# 3. Tests
echo ""
echo -e "${BLUE}[3/5]${NC} Running tests..."
if cargo test --workspace --lib --quiet; then
    echo -e "${GREEN}✓ Tests passed${NC}"
else
    echo -e "${RED}✗ Tests failed${NC}"
    FAILURES=$((FAILURES + 1))
fi

# 4. Docs
echo ""
echo -e "${BLUE}[4/5]${NC} Checking docs..."
if cargo doc --workspace --no-deps --quiet; then
    echo -e "${GREEN}✓ Documentation OK${NC}"
else
    echo -e "${RED}✗ Documentation errors${NC}"
    FAILURES=$((FAILURES + 1))
fi

# 5. Coverage (skip in fast mode)
if [ "$FAST_MODE" = false ]; then
    echo ""
    echo -e "${BLUE}[5/5]${NC} Checking coverage..."
    if command -v cargo-llvm-cov &> /dev/null; then
        if cargo llvm-cov --workspace --summary-only; then
            echo -e "${GREEN}✓ Coverage generated${NC}"
        else
            echo -e "${YELLOW}⚠ Coverage check failed${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ cargo-llvm-cov not installed, skipping coverage${NC}"
    fi
else
    echo ""
    echo -e "${YELLOW}⊘ Skipping coverage (fast mode)${NC}"
fi

# Result
echo ""
echo -e "${BLUE}========================================${NC}"
if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    exit 0
else
    echo -e "${RED}✗ $FAILURES check(s) failed${NC}"
    exit 1
fi
