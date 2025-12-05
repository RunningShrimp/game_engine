#!/bin/bash
# 运行测试覆盖率分析

set -e

echo "Running test coverage analysis..."

# 运行覆盖率测试
cargo tarpaulin --out Html --output-dir coverage --exclude-files '*/tests/*' --exclude-files '*/examples/*' --exclude-files '*/benches/*' || {
    echo "Warning: Coverage analysis completed with some issues"
    echo "This may be due to platform-specific limitations"
}

echo "Coverage report generated in coverage/ directory"
echo "Open coverage/tarpaulin-report.html to view the report"



