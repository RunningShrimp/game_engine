#!/bin/bash
# quality-report.sh
# 生成代码质量报告
#
# 用法:
#   ./scripts/quality-report.sh              # 生成到quality-report.md
#   ./scripts/quality-report.sh --output report.md  # 输出到指定文件
#   ./scripts/quality-report.sh --ci         # CI模式(退出码指示失败)

set -e

# 配置
OUTPUT_FILE="quality-report.md"
CI_MODE=false
COLOR_OUTPUT=true

# 解析参数
while [[ $# -gt 0 ]]; do
    case $1 in
        --output)
            OUTPUT_FILE="$2"
            shift 2
            ;;
        --ci)
            CI_MODE=true
            shift
            ;;
        --no-color)
            COLOR_OUTPUT=false
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# 颜色输出
if [ "$COLOR_OUTPUT" = true ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    BLUE='\033[0;34m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    NC=''
fi

# 打印带颜色的消息
print_section() {
    echo -e "${BLUE}▶ $1${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# 开始报告
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Game Engine Quality Report${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# 创建报告文件
cat > "$OUTPUT_FILE" << EOF
# Game Engine Quality Report

**Generated:** $(date '+%Y-%m-%d %H:%M:%S')
**Commit:** $(git rev-parse --short HEAD 2>/dev/null || echo "N/A")
**Branch:** $(git branch --show-current 2>/dev/null || echo "N/A")

---

## Table of Contents
1. [Code Formatting](#code-formatting)
2. [Clippy Warnings](#clippy-warnings)
3. [Test Results](#test-results)
4. [Documentation](#documentation)
5. [Code Coverage](#code-coverage)
6. [Security Audit](#security-audit)
7. [Build Status](#build-status)
8. [Summary](#summary)

---

EOF

# 计数器
FAILURES=0
WARNINGS=0

# 1. 格式检查
print_section "Checking code formatting..."
echo "## Code Formatting" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if cargo fmt --all -- --check > /tmp/fmt_check.txt 2>&1; then
    print_success "Code formatting check passed"
    echo "✅ **Status:** PASSED" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
else
    print_error "Code formatting issues found"
    echo "❌ **Status:** FAILED" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo '```diff' >> "$OUTPUT_FILE"
    cat /tmp/fmt_check.txt >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    FAILURES=$((FAILURES + 1))
fi

# 2. Clippy检查
print_section "Running Clippy lints..."
echo "## Clippy Warnings" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

CARGO_TERM_COLOR=never cargo clippy --workspace --all-targets -- -D warnings > /tmp/clippy_output.txt 2>&1 || true
CLIPPY_WARNINGS=$(grep -c "warning:" /tmp/clippy_output.txt 2>/dev/null || echo "0")

if [ "$CLIPPY_WARNINGS" -eq 0 ]; then
    print_success "No clippy warnings"
    echo "✅ **Status:** PASSED - 0 warnings" >> "$OUTPUT_FILE"
else
    print_warning "Found $CLIPPY_WARNINGS clippy warning(s)"
    echo "⚠️ **Status:** WARNING - $CLIPPY_WARNINGS warning(s)" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    head -50 /tmp/clippy_output.txt >> "$OUTPUT_FILE"
    if [ $(wc -l < /tmp/clippy_output.txt) -gt 50 ]; then
        echo "" >> "$OUTPUT_FILE"
        echo "... (truncated, see full output for details)" >> "$OUTPUT_FILE"
    fi
    echo '```' >> "$OUTPUT_FILE"
    WARNINGS=$((WARNINGS + CLIPPY_WARNINGS))
fi
echo "" >> "$OUTPUT_FILE"

# 3. 测试
print_section "Running tests..."
echo "## Test Results" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if cargo test --workspace --lib --quiet 2>&1 | tee /tmp/test_output.txt > /dev/null; then
    TEST_COUNT=$(grep -o "test result: ok" /tmp/test_output.txt | wc -l)
    print_success "All tests passed ($TEST_COUNT test suites)"
    echo "✅ **Status:** PASSED - $TEST_COUNT test suites" >> "$OUTPUT_FILE"
else
    print_error "Some tests failed"
    echo "❌ **Status:** FAILED" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    cat /tmp/test_output.txt >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    FAILURES=$((FAILURES + 1))
fi
echo "" >> "$OUTPUT_FILE"

# 4. 文档
print_section "Checking documentation..."
echo "## Documentation" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if cargo doc --workspace --no-deps --document-private-items --quiet 2>&1 | tee /tmp/doc_output.txt > /dev/null; then
    print_success "Documentation generated successfully"
    echo "✅ **Status:** PASSED" >> "$OUTPUT_FILE"

    # 检查是否有未解析的链接
    if grep -i "cannot be resolved" /tmp/doc_output.txt > /dev/null 2>&1; then
        BROKEN_LINKS=$(grep -c "cannot be resolved" /tmp/doc_output.txt)
        print_warning "Found $BROKEN_LINKS broken doc link(s)"
        echo "" >> "$OUTPUT_FILE"
        echo "⚠️ **Warning:** $BROKEN_LINKS unresolved doc link(s)" >> "$OUTPUT_FILE"
    fi
else
    print_error "Documentation generation failed"
    echo "❌ **Status:** FAILED" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    cat /tmp/doc_output.txt >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    FAILURES=$((FAILURES + 1))
fi
echo "" >> "$OUTPUT_FILE"

# 5. 覆盖率
print_section "Checking code coverage..."
echo "## Code Coverage" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if command -v cargo-tarpaulin &> /dev/null; then
    if cargo tarpaulin --workspace --out Stdout --timeout 120 2>&1 | tee /tmp/coverage_output.txt > /dev/null; then
        COVERAGE=$(grep -oP 'Overall.*\K\d+\.\d+' /tmp/coverage_output.txt | head -1 || echo "0")
        print_success "Coverage: ${COVERAGE}%"
        echo "**Coverage:** ${COVERAGE}%" >> "$OUTPUT_FILE"

        if (( $(echo "$COVERAGE < 50" | bc -l 2>/dev/null || echo "1") )); then
            print_warning "Coverage below 50% threshold"
            echo "⚠️ **Status:** Below target (50%)" >> "$OUTPUT_FILE"
        else
            print_success "Coverage meets threshold"
            echo "✅ **Status:** Meets target (50%)" >> "$OUTPUT_FILE"
        fi
    else
        print_warning "Coverage check failed"
        echo "⚠️ **Status:** Unable to calculate coverage" >> "$OUTPUT_FILE"
    fi
elif command -v cargo-llvm-cov &> /dev/null; then
    if cargo llvm-cov --workspace --summary-only 2>&1 | tee /tmp/coverage_output.txt > /dev/null; then
        COVERAGE=$(grep -oP 'Total.*\K\d+\.\d+' /tmp/coverage_output.txt | head -1 || echo "0")
        print_success "Coverage: ${COVERAGE}%"
        echo "**Coverage:** ${COVERAGE}%" >> "$OUTPUT_FILE"
    else
        print_warning "Coverage check failed"
        echo "⚠️ **Status:** Unable to calculate coverage" >> "$OUTPUT_FILE"
    fi
else
    print_warning "Coverage tool not installed (install cargo-tarpaulin or cargo-llvm-cov)"
    echo "⚠️ **Status:** Coverage tools not available" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo "Install with:" >> "$OUTPUT_FILE"
    echo "```bash" >> "$OUTPUT_FILE"
    echo "cargo install cargo-tarpaulin" >> "$OUTPUT_FILE"
    echo "# or" >> "$OUTPUT_FILE"
    echo "cargo install cargo-llvm-cov" >> "$OUTPUT_FILE"
    echo "```" >> "$OUTPUT_FILE"
fi
echo "" >> "$OUTPUT_FILE"

# 6. 安全审计
print_section "Running security audit..."
echo "## Security Audit" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if command -v cargo-audit &> /dev/null || command -v cargo &> /dev/null; then
    if cargo audit 2>&1 | tee /tmp/audit_output.txt > /dev/null; then
        print_success "No vulnerabilities found"
        echo "✅ **Status:** PASSED - No vulnerabilities" >> "$OUTPUT_FILE"
    else
        VULN_COUNT=$(grep -o "vulnerability" /tmp/audit_output.txt | wc -l)
        if [ "$VULN_COUNT" -gt 0 ]; then
            print_warning "Found $VULN_COUNT vulnerabilities"
            echo "⚠️ **Status:** $VULN_COUNT vulnerability/ies found" >> "$OUTPUT_FILE"
            echo "" >> "$OUTPUT_FILE"
            echo '```' >> "$OUTPUT_FILE"
            cat /tmp/audit_output.txt | head -20 >> "$OUTPUT_FILE"
            echo '```' >> "$OUTPUT_FILE"
        else
            print_warning "Security audit completed with warnings"
            echo "⚠️ **Status:** Completed with warnings" >> "$OUTPUT_FILE"
        fi
    fi
else
    print_warning "cargo-audit not installed"
    echo "⚠️ **Status:** cargo-audit not available" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo "Install with: \`cargo install cargo-audit\`" >> "$OUTPUT_FILE"
fi
echo "" >> "$OUTPUT_FILE"

# 7. 构建检查
print_section "Checking build..."
echo "## Build Status" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

if cargo build --workspace 2>&1 | tee /tmp/build_output.txt > /dev/null; then
    print_success "Build successful"
    echo "✅ **Status:** PASSED" >> "$OUTPUT_FILE"
else
    print_error "Build failed"
    echo "❌ **Status:** FAILED" >> "$OUTPUT_FILE"
    echo "" >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    tail -50 /tmp/build_output.txt >> "$OUTPUT_FILE"
    echo '```' >> "$OUTPUT_FILE"
    FAILURES=$((FAILURES + 1))
fi
echo "" >> "$OUTPUT_FILE"

# 汇总
echo "## Summary" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"
echo "| Metric | Value |" >> "$OUTPUT_FILE"
echo "|--------|-------|" >> "$OUTPUT_FILE"
echo "| Failures | $FAILURES |" >> "$OUTPUT_FILE"
echo "| Warnings | $WARNINGS |" >> "$OUTPUT_FILE"
echo "| Overall Status | $(if [ $FAILURES -eq 0 ]; then echo "✅ PASSED"; else echo "❌ FAILED"; fi) |" >> "$OUTPUT_FILE"
echo "" >> "$OUTPUT_FILE"

# 打印汇总到控制台
echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}   Quality Report Summary${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
if [ $FAILURES -eq 0 ]; then
    print_success "Quality check PASSED"
else
    print_error "Quality check FAILED - $FAILURES failure(s)"
fi

if [ $WARNINGS -gt 0 ]; then
    print_warning "$WARNINGS warning(s) found"
fi

echo ""
echo "Full report saved to: $OUTPUT_FILE"
echo ""

# CI模式下的退出码
if [ "$CI_MODE" = true ]; then
    if [ $FAILURES -gt 0 ]; then
        exit 1
    else
        exit 0
    fi
fi

# 清理临时文件
rm -f /tmp/fmt_check.txt /tmp/clippy_output.txt /tmp/test_output.txt /tmp/doc_output.txt /tmp/coverage_output.txt /tmp/audit_output.txt /tmp/build_output.txt
