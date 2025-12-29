#!/bin/bash
# 文档检查脚本
# 检查文档完整性和链接有效性

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
DOCS_DIR="$PROJECT_ROOT/docs"

echo "=========================================="
echo "Documentation Check"
echo "=========================================="
echo ""

ERRORS=0

# 检查mdbook是否安装
if ! command -v mdbook &> /dev/null; then
    echo "❌ mdbook not found. Please run: ./scripts/setup-documentation.sh"
    exit 1
fi

echo "✅ mdbook found"
echo ""

# 检查1: 构建文档
echo "🔍 Check 1: Building documentation..."
cd "$DOCS_DIR"
if mdbook build > /dev/null 2>&1; then
    echo "✅ Documentation builds successfully"
else
    echo "❌ Documentation build failed"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# 检查2: 检查链接
echo "🔍 Check 2: Checking links..."
if command -v mdbook-linkcheck &> /dev/null; then
    if mdbook linkcheck; then
        echo "✅ All links are valid"
    else
        echo "⚠️  Some links are broken or unreachable"
        # 不算错误，因为外部链接可能暂时不可用
    fi
else
    echo "⚠️  mdbook-linkcheck not installed. Skipping link check."
    echo "   Install with: cargo install mdbook-linkcheck"
fi
echo ""

# 检查3: 验证SUMMARY.md
echo "🔍 Check 3: Verifying SUMMARY.md..."

# 创建临时Python脚本来检查SUMMARY.md
python3 << 'PYTHON_SCRIPT'
import sys
import re
from pathlib import Path

docs_dir = Path.cwd()
summary_file = docs_dir / "SUMMARY.md"

if not summary_file.exists():
    print("❌ SUMMARY.md not found")
    sys.exit(1)

with open(summary_file) as f:
    content = f.read()

# 提取所有链接
links = re.findall(r'\(([^)]+\.md)\)', content)
errors = []

for link in links:
    # 去除锚点
    link_path = link.split('#')[0]

    # 跳略外部链接
    if link.startswith('http://') or link.startswith('https://'):
        continue

    # 检查文件是否存在
    file_path = docs_dir / link_path
    if not file_path.exists():
        errors.append(f"  - {link_path} not found")

if errors:
    print("❌ Some files listed in SUMMARY.md are missing:")
    for error in errors:
        print(error)
    sys.exit(1)
else:
    print("✅ All files in SUMMARY.md exist")

sys.exit(0)
PYTHON_SCRIPT

if [ $? -ne 0 ]; then
    ERRORS=$((ERRORS + 1))
fi
echo ""

# 检查4: 代码示例可编译性
echo "🔍 Check 4: Checking code examples..."
cd "$PROJECT_ROOT"
if cargo test --doc --workspace --quiet; then
    echo "✅ Documentation tests pass"
else
    echo "⚠️  Some documentation tests failed"
    ERRORS=$((ERRORS + 1))
fi
echo ""

# 检查5: 检查文档覆盖
echo "🔍 Check 5: Checking documentation coverage..."

# 统计文档数量
DOC_COUNT=$(find "$DOCS_DIR" -name "*.md" -type f | wc -l | tr -d ' ')
echo "   Found $DOC_COUNT markdown files"

if [ "$DOC_COUNT" -lt 20 ]; then
    echo "⚠️  Low documentation count (< 20 files)"
else
    echo "✅ Good documentation count ($DOC_COUNT files)"
fi
echo ""

# 检查6: 检查关键文档
echo "🔍 Check 6: Checking key documentation files..."

KEY_FILES=(
    "INDEX.md"
    "SUMMARY.md"
    "quickstart.md"
    "installation.md"
    "book.toml"
)

MISSING=0
for file in "${KEY_FILES[@]}"; do
    if [ -f "$DOCS_DIR/$file" ]; then
        echo "  ✅ $file"
    else
        echo "  ❌ $file (missing)"
        MISSING=$((MISSING + 1))
    fi
done

if [ $MISSING -gt 0 ]; then
    ERRORS=$((ERRORS + 1))
fi
echo ""

# 总结
echo "=========================================="
if [ $ERRORS -eq 0 ]; then
    echo "✅ All checks passed!"
    echo "=========================================="
    exit 0
else
    echo "❌ Found $ERRORS error(s)"
    echo "=========================================="
    exit 1
fi
