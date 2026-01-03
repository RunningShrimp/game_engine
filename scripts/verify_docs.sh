#!/bin/bash
# 文档验证脚本
#
# 功能：
# - 验证文档完整性
# - 检查文档链接有效性
# - 统计文档覆盖率
# - 检查文档格式一致性

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 统计变量
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# 日志函数
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_CHECKS++))
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_CHECKS++))
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

# 检查计数器
increment_check() {
    ((TOTAL_CHECKS++))
}

# 文档根目录
DOCS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../docs" && pwd)"

cd "$DOCS_DIR"

log_info "开始文档验证..."
log_info "文档目录: $DOCS_DIR"
echo

# ============================================================================
# 检查1：验证核心文档存在性
# ============================================================================

log_info "检查1：验证核心文档存在性..."

CORE_DOCS=(
    "README.md"
    "INDEX.md"
    "ADVANCED_FEATURES_GUIDE.md"
    "POST_PROCESSING_GUIDE.md"
    "PHYSICS_SIMULATION_GUIDE.md"
    "BEST_PRACTICES.md"
    "MAINTENANCE_PLAN.md"
    "FINAL_COMPLETION_REPORT.md"
    "TODO_TRACKING.md"
)

for doc in "${CORE_DOCS[@]}"; do
    increment_check
    if [ -f "$doc" ]; then
        log_success "核心文档存在: $doc"
    else
        log_error "核心文档缺失: $doc"
    fi
done

echo

# ============================================================================
# 检查2：验证API文档目录结构
# ============================================================================

log_info "检查2：验证API文档目录结构..."

increment_check
if [ -d "api" ]; then
    log_success "API目录存在"
    
    # 检查核心API文档
    API_DOCS=(
        "api/ecs.md"
        "api/engine.md"
        "api/physics.md"
        "api/rendering.md"
        "api/resources.md"
        "api/scripting.md"
        "api/audio.md"
        "api/networking.md"
    )
    
    for api_doc in "${API_DOCS[@]}"; do
        increment_check
        if [ -f "$api_doc" ]; then
            log_success "API文档存在: $api_doc"
        else
            log_warning "API文档缺失: $api_doc"
        fi
    done
else
    log_error "API目录不存在"
fi

echo

# ============================================================================
# 检查3：验证架构文档目录结构
# ============================================================================

log_info "检查3：验证架构文档目录结构..."

increment_check
if [ -d "architecture" ]; then
    log_success "架构目录存在"
    
    # 检查核心架构文档
    ARCH_DOCS=(
        "architecture/overview.md"
        "architecture/ecs.md"
    )
    
    for arch_doc in "${ARCH_DOCS[@]}"; do
        increment_check
        if [ -f "$arch_doc" ]; then
            log_success "架构文档存在: $arch_doc"
        else
            log_warning "架构文档缺失: $arch_doc"
        fi
    done
else
    log_warning "架构目录不存在（非必需）"
fi

echo

# ============================================================================
# 检查4：验证指南文档
# ============================================================================

log_info "检查4：验证指南文档..."

GUIDE_DOCS=(
    "quickstart.md"
    "installation.md"
    "configuration.md"
    "best_practices.md"
    "troubleshooting.md"
)

for guide_doc in "${GUIDE_DOCS[@]}"; do
    increment_check
    if [ -f "$guide_doc" ]; then
        log_success "指南文档存在: $guide_doc"
    else
        log_warning "指南文档缺失: $guide_doc"
    fi
done

echo

# ============================================================================
# 检查5：验证教程文档
# ============================================================================

log_info "检查5：验证教程文档..."

increment_check
if [ -d "tutorials" ]; then
    log_success "教程目录存在"
    
    # 检查核心教程
    TUTORIAL_DOCS=(
        "tutorials/getting_started.md"
        "tutorials/ecs_guide.md"
        "tutorials/rendering_guide.md"
    )
    
    for tutorial_doc in "${TUTORIAL_DOCS[@]}"; do
        increment_check
        if [ -f "$tutorial_doc" ]; then
            log_success "教程文档存在: $tutorial_doc"
        else
            log_warning "教程文档缺失: $tutorial_doc"
        fi
    done
else
    log_warning "教程目录不存在（非必需）"
fi

echo

# ============================================================================
# 检查6：检查重复文档
# ============================================================================

log_info "检查6：检查重复文档..."

# 定义可能重复的文档模式
DUPLICATE_PATTERNS=(
    "*COMPLETION_REPORT*.md"
    "*COMPLETION_SUMMARY*.md"
    "*_COMPLETION_REPORT.md"
    "*_IMPLEMENTATION_REPORT.md"
    "P1-*_SUMMARY.md"
    "P1-*-SUMMARY.md"
    "P2-*_SUMMARY.md"
    "P2-*-SUMMARY.md"
    "P3-*_SUMMARY.md"
    "P3-*-SUMMARY.md"
)

found_duplicates=0

for pattern in "${DUPLICATE_PATTERNS[@]}"; do
    matches=$(find . -name "$pattern" -type f 2>/dev/null | wc -l | tr -d ' ')
    if [ "$matches" -gt 0 ]; then
        log_warning "发现可能的重复文档（模式: $pattern）：$matches 个"
        ((found_duplicates++))
    fi
done

increment_check
if [ $found_duplicates -eq 0 ]; then
    log_success "未发现明显的重复文档"
else
    log_error "发现 $found_duplicates 组可能重复的文档模式"
fi

echo

# ============================================================================
# 检查7：检查临时/草稿文档
# ============================================================================

log_info "检查7：检查临时/草稿文档..."

TEMP_PATTERNS=(
    "*_DRAFT.md"
    "*_TEMP.md"
    "*_WIP.md"
    "*TODO*.md"
    "*DRAFT*.md"
    "*TEMP*.md"
    "*WIP*.md"
)

found_temp=0

for pattern in "${TEMP_PATTERNS[@]}"; do
    matches=$(find . -name "$pattern" -type f 2>/dev/null | grep -v "TODO_TRACKING.md" | wc -l | tr -d ' ')
    if [ "$matches" -gt 0 ]; then
        log_warning "发现临时/草稿文档（模式: $pattern）：$matches 个"
        ((found_temp++))
    fi
done

increment_check
if [ $found_temp -eq 0 ]; then
    log_success "未发现临时/草稿文档"
else
    log_warning "发现 $found_temp 组可能需要清理的临时/草稿文档"
fi

echo

# ============================================================================
# 检查8：检查文档大小（过小文档可能需要完善）
# ============================================================================

log_info "检查8：检查文档大小..."

SMALL_DOCS=$(find . -name "*.md" -type f -size -1k 2>/dev/null | head -20)
increment_check

if [ -z "$SMALL_DOCS" ]; then
    log_success "未发现过小文档（< 1KB）"
else
    log_warning "发现以下文档过小（< 1KB），可能需要完善："
    echo "$SMALL_DOCS" | while read -r doc; do
        echo "  - $doc"
    done
fi

echo

# ============================================================================
# 检查9：检查Markdown格式
# ============================================================================

log_info "检查9：检查Markdown格式..."

# 检查是否有未闭合的代码块
UNCLOSED_CODE_BLOCKS=$(grep -r '```' *.md 2>/dev/null | grep -v '\.md.*\.md' || true)
increment_check

if [ -z "$UNCLOSED_CODE_BLOCKS" ]; then
    log_success "未发现未闭合的代码块"
else
    log_warning "发现未闭合的代码块，请检查"
fi

# 检查是否有未闭合的链接
BROKEN_LINKS=$(grep -r '\[.*\](\s*$' *.md 2>/dev/null | grep -v '\.md.*\.md' | head -10 || true)
increment_check

if [ -z "$BROKEN_LINKS" ]; then
    log_success "未发现明显的格式问题"
else
    log_warning "发现潜在的格式问题，请检查："
    echo "$BROKEN_LINKS" | while read -r line; do
        echo "  - $line"
    done | head -5
fi

echo

# ============================================================================
# 检查10：统计文档数量
# ============================================================================

log_info "检查10：统计文档数量..."

TOTAL_MDS=$(find . -name "*.md" -type f | wc -l | tr -d ' ')
increment_check

log_info "总文档数: $TOTAL_MDS"

if [ $TOTAL_MDS -gt 100 ]; then
    log_success "文档数量充足（> 100）"
elif [ $TOTAL_MDS -gt 50 ]; then
    log_warning "文档数量适中（50-100）"
else
    log_error "文档数量偏少（< 50）"
fi

echo

# ============================================================================
# 检查11：检查关键功能文档覆盖
# ============================================================================

log_info "检查11：检查关键功能文档覆盖..."

FEATURE_KEYWORDS=(
    "ECS|Entity Component System"
    "Rendering|Render"
    "Physics|Rapier"
    "Audio|Sound"
    "Networking|Network"
    "Scripting|Script"
    "Ray Tracing|RTX"
    "VXGI|Global Illumination"
    "GPU|CUDA|ROCm"
    "NPU|Inference|ONNX"
)

# 检查主文档是否覆盖所有关键功能
MAIN_DOCS="README.md INDEX.md ADVANCED_FEATURES_GUIDE.md FINAL_COMPLETION_REPORT.md"

missing_features=0

for keyword in "${FEATURE_KEYWORDS[@]}"; do
    found=0
    for doc in $MAIN_DOCS; do
        if [ -f "$doc" ] && grep -qiE "$keyword" "$doc" >/dev/null 2>&1; then
            found=1
            break
        fi
    done
    
    if [ $found -eq 0 ]; then
        log_warning "可能缺失功能文档: $keyword"
        ((missing_features++))
    fi
done

increment_check
if [ $missing_features -eq 0 ]; then
    log_success "关键功能文档覆盖完整"
else
    log_warning "可能缺失 $missing_features 个关键功能的文档"
fi

echo

# ============================================================================
# 检查12：检查README.md质量
# ============================================================================

log_info "检查12：检查README.md质量..."

increment_check
if [ -f "README.md" ]; then
    # 检查关键章节
    REQUIRED_SECTIONS=(
        "# 游戏引擎"
        "## 特性"
        "## 快速开始"
        "## 文档"
    )
    
    missing_sections=0
    for section in "${REQUIRED_SECTIONS[@]}"; do
        if ! grep -q "$section" README.md; then
            ((missing_sections++))
        fi
    done
    
    if [ $missing_sections -eq 0 ]; then
        log_success "README.md 包含所有关键章节"
    else
        log_warning "README.md 缺失 $missing_sections 个关键章节"
    fi
else
    log_error "README.md 不存在"
fi

echo

# ============================================================================
# 生成验证报告
# ============================================================================

echo
log_info "============================================================"
log_info "文档验证完成"
log_info "============================================================"
echo

PASS_RATE=$(awk "BEGIN {printf \"%.1f\", ($PASSED_CHECKS/$TOTAL_CHECKS)*100}")

log_info "总检查数: $TOTAL_CHECKS"
log_success "通过检查: $PASSED_CHECKS"
log_error "失败检查: $FAILED_CHECKS"
log_info "通过率: ${PASS_RATE}%"
echo

# 判断整体质量
if [ $PASS_RATE -ge 90 ]; then
    log_success "文档质量：优秀 🎉"
    EXIT_CODE=0
elif [ $PASS_RATE -ge 75 ]; then
    log_warning "文档质量：良好"
    EXIT_CODE=1
elif [ $PASS_RATE -ge 60 ]; then
    log_warning "文档质量：及格"
    EXIT_CODE=2
else
    log_error "文档质量：需要改进"
    EXIT_CODE=3
fi

echo
log_info "建议："
log_info "  1. 检查失败的检查项"
log_info "  2. 修复缺失的文档"
log_info "  3. 完善文档格式"
log_info "  4. 更新文档索引"
echo

exit $EXIT_CODE
