#!/bin/bash
# P1-6 验证脚本
# 自动化运行所有验证检查

set -e  # 遇到错误立即退出

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 项目根目录
PROJECT_ROOT="/Users/didi/Desktop/game_engine"
cd "$PROJECT_ROOT"

# 日志目录
LOG_DIR="$PROJECT_ROOT/docs/code-quality/verification-logs"
mkdir -p "$LOG_DIR"

# 时间戳
TIMESTAMP=$(date +"%Y%m%d-%H%M%S")

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  P1-6 代码质量改进 - 验证脚本${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "开始时间: $(date)"
echo -e "日志目录: $LOG_DIR"
echo ""

# ============================================================================
# 1. 编译检查
# ============================================================================
echo -e "${YELLOW}[1/4] 运行编译检查...${NC}"
echo "cargo check --lib -p game_engine"

if cargo check --lib -p game_engine 2>&1 | tee "$LOG_DIR/cargo-check-$TIMESTAMP.log"; then
    echo -e "${GREEN}✓ 编译检查通过${NC}"
else
    echo -e "${RED}✗ 编译检查失败${NC}"
    echo -e "${RED}请查看日志: $LOG_DIR/cargo-check-$TIMESTAMP.log${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo ""

# ============================================================================
# 2. 测试验证
# ============================================================================
echo -e "${YELLOW}[2/4] 运行测试套件...${NC}"
echo "cargo test -p game_engine --lib"

if cargo test -p game_engine --lib 2>&1 | tee "$LOG_DIR/cargo-test-$TIMESTAMP.log"; then
    echo -e "${GREEN}✓ 测试套件通过${NC}"

    # 提取测试统计
    TEST_COUNT=$(grep -o "test result: ok" "$LOG_DIR/cargo-test-$TIMESTAMP.log" | wc -l)
    echo -e "${BLUE}测试模块数: $TEST_COUNT${NC}"
else
    echo -e "${RED}✗ 测试套件失败${NC}"
    echo -e "${RED}请查看日志: $LOG_DIR/cargo-test-$TIMESTAMP.log${NC}"
    exit 1
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo ""

# ============================================================================
# 3. Clippy检查
# ============================================================================
echo -e "${YELLOW}[3/4] 运行Clippy检查...${NC}"
echo "cargo clippy -p game_engine --lib"

if cargo clippy -p game_engine --lib 2>&1 | tee "$LOG_DIR/cargo-clippy-$TIMESTAMP.log"; then
    echo -e "${GREEN}✓ Clippy检查通过${NC}"

    # 统计警告数量
    WARNING_COUNT=$(grep -c "warning:" "$LOG_DIR/cargo-clippy-$TIMESTAMP.log" || echo "0")
    echo -e "${BLUE}警告数量: $WARNING_COUNT${NC}"
else
    echo -e "${YELLOW}⚠ Clippy发现警告${NC}"
    echo -e "${YELLOW}请查看日志: $LOG_DIR/cargo-clippy-$TIMESTAMP.log${NC}"
    # 不退出，继续执行
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo ""

# ============================================================================
# 4. 覆盖率报告
# ============================================================================
echo -e "${YELLOW}[4/4] 生成覆盖率报告...${NC}"
echo "cargo tarpaulin --lib -p game_engine --out Html"

if command -v cargo-tarpaulin &> /dev/null; then
    if cargo tarpaulin --lib -p game_engine --out Html 2>&1 | tee "$LOG_DIR/tarpaulin-$TIMESTAMP.log"; then
        echo -e "${GREEN}✓ 覆盖率报告生成成功${NC}"

        # 复制报告到文档目录
        COVERAGE_DIR="$PROJECT_ROOT/docs/coverage/post-p1-6"
        mkdir -p "$COVERAGE_DIR"
        cp -r "$PROJECT_ROOT/target/tarpaulin/"* "$COVERAGE_DIR/" 2>/dev/null || true

        echo -e "${BLUE}覆盖率报告: $COVERAGE_DIR/index.html${NC}"
    else
        echo -e "${YELLOW}⚠ 覆盖率报告生成失败（tarpaulin可能未安装）${NC}"
        echo -e "${YELLOW}安装命令: cargo install cargo-tarpaulin${NC}"
    fi
else
    echo -e "${YELLOW}⚠ cargo-tarpaulin未安装，跳过覆盖率报告${NC}"
    echo -e "${YELLOW}安装命令: cargo install cargo-tarpaulin${NC}"
fi

echo ""
echo -e "${GREEN}========================================${NC}"
echo ""

# ============================================================================
# 5. 生成验证摘要
# ============================================================================
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  验证摘要${NC}"
echo -e "${BLUE}========================================${NC}"

# 创建摘要报告
SUMMARY_FILE="$LOG_DIR/verification-summary-$TIMESTAMP.md"
cat > "$SUMMARY_FILE" << EOF
# P1-6 验证摘要报告

**验证时间**: $(date)
**验证人**: 自动化脚本
**项目**: 游戏引擎代码质量改进

## 验证结果

### 1. 编译检查
EOF

if grep -q "Finished" "$LOG_DIR/cargo-check-$TIMESTAMP.log" 2>/dev/null; then
    echo "- ✅ 通过" >> "$SUMMARY_FILE"
else
    echo "- ❌ 失败" >> "$SUMMARY_FILE"
fi

cat >> "$SUMMARY_FILE" << EOF

### 2. 测试套件
EOF

if grep -q "test result: ok" "$LOG_DIR/cargo-test-$TIMESTAMP.log" 2>/dev/null; then
    echo "- ✅ 通过" >> "$SUMMARY_FILE"
    TEST_COUNT=$(grep "test result: ok" "$LOG_DIR/cargo-test-$TIMESTAMP.log" | wc -l)
    echo "- 测试模块数: $TEST_COUNT" >> "$SUMMARY_FILE"
else
    echo "- ❌ 失败" >> "$SUMMARY_FILE"
fi

cat >> "$SUMMARY_FILE" << EOF

### 3. Clippy检查
EOF

if [ -f "$LOG_DIR/cargo-clippy-$TIMESTAMP.log" ]; then
    WARNING_COUNT=$(grep -c "warning:" "$LOG_DIR/cargo-clippy-$TIMESTAMP.log" 2>/dev/null || echo "0")
    if [ "$WARNING_COUNT" -eq 0 ]; then
        echo "- ✅ 无警告" >> "$SUMMARY_FILE"
    else
        echo "- ⚠️  警告数: $WARNING_COUNT" >> "$SUMMARY_FILE"
    fi
else
    echo "- ⚠️  未执行" >> "$SUMMARY_FILE"
fi

cat >> "$SUMMARY_FILE" << EOF

### 4. 覆盖率报告
EOF

if [ -f "$COVERAGE_DIR/index.html" ]; then
    echo "- ✅ 已生成" >> "$SUMMARY_FILE"
    echo "- 报告位置: \`docs/coverage/post-p1-6/index.html\`" >> "$SUMMARY_FILE"
else
    echo "- ⚠️  未生成（tarpaulin未安装）" >> "$SUMMARY_FILE"
fi

cat >> "$SUMMARY_FILE" << EOF

## 详细日志

- 编译日志: \`$(basename $LOG_DIR/cargo-check-$TIMESTAMP.log)\`
- 测试日志: \`$(basename $LOG_DIR/cargo-test-$TIMESTAMP.log)\`
- Clippy日志: \`$(basename $LOG_DIR/cargo-clippy-$TIMESTAMP.log)\`
- 覆盖率日志: \`$(basename $LOG_DIR/tarpaulin-$TIMESTAMP.log)\`

## 下一步

1. 检查所有日志文件
2. 修复任何发现的问题
3. 更新文档
4. 提交代码

---

**验证完成时间**: $(date)
EOF

echo -e "${BLUE}验证摘要: $SUMMARY_FILE${NC}"
cat "$SUMMARY_FILE"

echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  验证完成！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "完成时间: $(date)"
echo -e "所有日志已保存到: $LOG_DIR"
echo ""
echo -e "${BLUE}下一步:${NC}"
echo -e "1. 查看验证摘要: cat $SUMMARY_FILE"
echo -e "2. 检查覆盖率报告: $COVERAGE_DIR/index.html"
echo -e "3. 更新主README"
echo -e "4. 创建迁移指南"
echo ""

exit 0
