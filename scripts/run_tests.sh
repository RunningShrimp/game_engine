#!/bin/bash
# 测试运行脚本

set -e

echo "========================================="
echo "Game Engine Test Runner"
echo "========================================="
echo ""

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# 解析参数
COVERAGE=false
BENCHMARKS=false
VERBOSE=false
SKIP_SLOW=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --coverage)
            COVERAGE=true
            shift
            ;;
        --bench)
            BENCHMARKS=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --skip-slow)
            SKIP_SLOW=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# 运行单元测试
run_tests() {
    echo -e "${YELLOW}Running unit tests...${NC}"
    
    if [ "$VERBOSE" = true ]; then
        cargo test --workspace --verbose
    else
        cargo test --workspace
    fi
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Unit tests passed${NC}"
    else
        echo -e "${RED}✗ Unit tests failed${NC}"
        exit 1
    fi
}

# 运行覆盖率测试
run_coverage() {
    echo -e "${YELLOW}Running coverage tests...${NC}"
    
    if ! command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}Installing cargo-tarpaulin...${NC}"
        cargo install cargo-tarpaulin
    fi
    
    cargo tarpaulin --workspace \
        --out Html \
        --out Xml \
        --output-dir target/coverage \
        --verbose \
        --timeout 300 \
        --exclude-files "*/tests/*" \
        --exclude-files "*_test.rs" \
        --exclude-files "benches/*"
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Coverage report generated${NC}"
        echo -e "HTML report: target/coverage/html/index.html"
    else
        echo -e "${RED}✗ Coverage tests failed${NC}"
        exit 1
    fi
}

# 运行基准测试
run_benchmarks() {
    echo -e "${YELLOW}Running benchmarks...${NC}"
    cargo bench --workspace
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Benchmarks completed${NC}"
    else
        echo -e "${RED}✗ Benchmarks failed${NC}"
        exit 1
    fi
}

# Clippy检查
run_clippy() {
    echo -e "${YELLOW}Running clippy...${NC}"
    cargo clippy --workspace --all-targets -- -D warnings
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Clippy checks passed${NC}"
    else
        echo -e "${RED}✗ Clippy checks failed${NC}"
        exit 1
    fi
}

# 格式检查
run_fmt_check() {
    echo -e "${YELLOW}Checking code formatting...${NC}"
    cargo fmt --all -- --check
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Code formatting OK${NC}"
    else
        echo -e "${RED}✗ Code formatting issues found${NC}"
        echo "Run 'cargo fmt' to fix"
        exit 1
    fi
}

# 主流程
main() {
    echo -e "${GREEN}Starting test suite...${NC}"
    echo ""
    
    # 运行测试
    run_tests
    
    # 可选：覆盖率
    if [ "$COVERAGE" = true ]; then
        echo ""
        run_coverage
    fi
    
    # 可选：基准测试
    if [ "$BENCHMARKS" = true ]; then
        echo ""
        run_benchmarks
    fi
    
    # Clippy和格式检查
    echo ""
    run_clippy
    run_fmt_check
    
    echo ""
    echo -e "${GREEN}=========================================${NC}"
    echo -e "${GREEN}All tests passed! ✓${NC}"
    echo -e "${GREEN}=========================================${NC}"
}

main
