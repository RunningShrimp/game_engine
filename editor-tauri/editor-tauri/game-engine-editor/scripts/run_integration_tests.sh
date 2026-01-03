#!/bin/bash
# Integration Test Runner Script
# 运行所有集成测试并生成报告

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# 解析命令行参数
VERBOSE=false
COVERAGE=false
BENCHMARK=false
SPECIFIC_TEST=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--coverage)
            COVERAGE=true
            shift
            ;;
        -b|--benchmark)
            BENCHMARK=true
            shift
            ;;
        -t|--test)
            SPECIFIC_TEST="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -v, --verbose           Show verbose output"
            echo "  -c, --coverage          Generate coverage report"
            echo "  -b, --benchmark         Run performance benchmarks"
            echo "  -t, --test <NAME>       Run specific test"
            echo "  -h, --help              Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_info "Starting integration tests..."
echo ""

# 检查依赖
print_info "Checking dependencies..."

if ! command -v cargo &> /dev/null; then
    print_error "cargo not found. Please install Rust."
    exit 1
fi

if [ "$COVERAGE" = true ] && ! command -v cargo-tarpaulin &> /dev/null; then
    print_warn "cargo-tarpaulin not found. Installing..."
    cargo install cargo-tarpaulin
fi

print_info "Dependencies OK"
echo ""

# 运行格式检查
print_info "Running format check..."
if cargo fmt -- --check; then
    print_info "Format check passed"
else
    print_error "Format check failed. Run 'cargo fmt' to fix."
    exit 1
fi
echo ""

# 运行Clippy
print_info "Running Clippy..."
if cargo clippy --all-features -- -D warnings; then
    print_info "Clippy passed"
else
    print_error "Clippy failed"
    exit 1
fi
echo ""

# 构建测试
print_info "Building tests..."
if [ "$VERBOSE" = true ]; then
    cargo test --no-run --all-features --verbose
else
    cargo test --no-run --all-features 2>&1 | grep -E "Compiling|Finished"
fi
print_info "Build successful"
echo ""

# 运行测试
print_info "Running tests..."

if [ -n "$SPECIFIC_TEST" ]; then
    print_info "Running specific test: $SPECIFIC_TEST"
    if [ "$VERBOSE" = true ]; then
        cargo test --all-features "$SPECIFIC_TEST" -- --nocapture
    else
        cargo test --all-features "$SPECIFIC_TEST"
    fi
else
    if [ "$VERBOSE" = true ]; then
        cargo test --all-features -- --nocapture --test-threads=1
    else
        cargo test --all-features -- --test-threads=1
    fi
fi

TEST_RESULT=$?

if [ $TEST_RESULT -eq 0 ]; then
    print_info "All tests passed!"
else
    print_error "Some tests failed"
    exit 1
fi
echo ""

# 生成覆盖率报告
if [ "$COVERAGE" = true ]; then
    print_info "Generating coverage report..."
    cargo tarpaulin --all-features --timeout 300 --out Html --output-dir coverage/

    if [ -f "coverage/index.html" ]; then
        print_info "Coverage report generated: coverage/index.html"

        # 提取覆盖率百分比
        COVERAGE_PERCENT=$(cargo tarpaulin --all-features --timeout 300 --out Stdout 2>/dev/null | grep "Overall" | awk '{print $2}')
        print_info "Overall coverage: ${COVERAGE_PERCENT}"
    fi
    echo ""
fi

# 运行性能基准测试
if [ "$BENCHMARK" = true ]; then
    print_info "Running performance benchmarks..."
    cargo test --release performance -- --nocapture

    if [ $? -eq 0 ]; then
        print_info "Performance benchmarks passed"
    else
        print_warn "Some performance tests failed or showed regressions"
    fi
    echo ""
fi

# 生成测试报告
print_info "Generating test report..."

REPORT_FILE="test_report_$(date +%Y%m%d_%H%M%S).txt"

{
    echo "======================================"
    echo "  Integration Test Report"
    echo "======================================"
    echo ""
    echo "Date: $(date)"
    echo "Platform: $(uname -s) $(uname -m)"
    echo "Rust version: $(rustc --version)"
    echo ""
    echo "Test Configuration:"
    echo "  Verbose: $VERBOSE"
    echo "  Coverage: $COVERAGE"
    echo "  Benchmark: $BENCHMARK"
    echo ""
    echo "Test Summary:"
    echo "  Total Tests Run: $(cargo test --all-features --no-run 2>&1 | grep "Running" | wc -l)"
    echo "  Result: PASSED"
    echo ""
    echo "======================================"
} > "$REPORT_FILE"

print_info "Test report saved: $REPORT_FILE"
echo ""

# 总结
print_info "======================================"
print_info "  Integration Tests Complete"
print_info "======================================"
print_info "All tests passed successfully!"
echo ""

if [ "$COVERAGE" = true ]; then
    print_info "View coverage report: file://$(pwd)/coverage/index.html"
fi

exit 0
