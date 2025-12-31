#!/bin/bash
# 清理不必要的文档和文件

echo "=== 开始清理项目文件 ==="

# 文档根目录
DOCS_DIR="/Users/didi/Desktop/game_engine/docs"
cd "$DOCS_DIR" || exit 1

echo "1. 清理中间进度报告..."
# 删除中间阶段进度报告，只保留最终版本
rm -f \
  SESSION_PROGRESS_2025-12-31.md \
  SESSION_PROGRESS_2025-12-31_FINAL.md \
  PHASE_2_COMPLETE_REPORT_2025-12-30.md \
  OPTIMIZATION_PHASE_PROGRESS_REPORT.md \
  PHASE_5_ACTION_PLAN.md \
  PHASE_5_COMPLETION_REPORT.md

echo "2. 清理重复的总结文档..."
rm -f \
  P3_PHASE_PROGRESS_2025-12-31_FINAL.md \
  IMPLEMENTATION_SUMMARY_2025-12-28.md \
  TASK_COMPLETION_REPORT_2025-12-28.md \
  CONTINUOUS_IMPROVEMENT_REPORT_2025-12-28.md

echo "3. 清理过时的计划文档..."
rm -f \
  AI_ENHANCEMENT_PLAN.md \
  AUDIO_ENHANCEMENT_PLAN.md \
  OPTIMIZATION_FINAL_SUMMARY.md \
  OPTIMIZATION_JOURNEY_REPORT.md \
  OPTIMIZATION_FINAL_CELEBRATION.md

echo "4. 清理中间测试报告..."
rm -f \
  NETWORK_TEST_FIX_REPORT.md \
  TEST_FIX_SESSION_REPORT.md \
  TEST_FIX_ROUND3_REPORT.md \
  TEST_FIX_ROUND4_REPORT.md \
  TEST_FIX_ROUND5_REPORT.md

echo "5. 清理其他临时文档..."
rm -f \
  v0.2.0_COMPARISON.md \
  status-summary.md \
  3D_FORMAT_LOADER_SUMMARY.md \
  PARALLEL_OPTIMIZATION_REPORT.md \
  BENCHMARK_DASHBOARD_PREVIEW.md \
  BENCHMARK_CI_QUICKREF.md

echo "6. 清理tasks目录下的中间文件..."
rm -f tasks/P0-2_IMPLEMENTATION_SUMMARY.md
rm -f tasks/P3-2-TASK-COMPLETE.md
rm -f tasks/P3-3_DEPENDENCY_CLEANUP_SUMMARY.md

echo "7. 清理adr目录下的过时文档..."
rm -f adr/0004-concurrency-model.md
rm -f adr/0007-unified-resource-management.md
rm -f adr/0008-wasm-optimization.md

echo "8. 清理code-quality目录..."
rm -rf code-quality

echo "9. 清理benchmarks目录..."
rm -rf benchmarks

echo "10. 清理其他临时目录..."
rm -rf adr/GUEST_BOOK.md 2>/dev/null || true

echo "=== 清理完成 ==="
echo ""
echo "保留的核心文档:"
echo "- FINAL_COMPLETION_REPORT_2025-12-31.md (最终完成报告)"
echo "- P2-*/P3-*/ (各阶段总结文档)"
echo "- architecture/, api/, rendering_pipeline.md 等 (技术文档)"
echo ""
