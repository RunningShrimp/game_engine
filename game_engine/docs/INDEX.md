# 游戏引擎文档索引

欢迎来到游戏引擎文档中心！本文档提供了所有可用文档的索引和导航。

## 📚 快速导航

- [快速开始](#快速开始)
- [用户指南](#用户指南)
- [API 文档](#api-文档)
- [架构文档](#架构文档)
- [教程](#教程)
- [任务报告](#任务报告)
- [技术决策记录](#技术决策记录)

## 🚀 快速开始

### 新手入门
- [快速开始指南](tutorials/getting_started.md) - 快速上手游戏引擎
- [ECS 指南](tutorials/ecs_guide.md) - 实体组件系统详解
- [渲染指南](tutorials/rendering_guide.md) - 渲染系统使用指南

### 安装和构建
- [XMake 构建指南](xmake_build_guide.md) - 使用 XMake 构建项目
- [CLI 工具指南](cli_tool_guide.md) - 命令行工具使用

## 👥 用户指南

### 核心功能
- [Entity API 快速开始](entity_api_quick_start.md) - 实体 API 快速入门
- [Entity API 实现文档](entity_api_implementation.md) - 实体 API 实现细节
- [Entity API 验收文档](entity_api_acceptance.md) - 实体 API 验收标准

### 调试系统
- [LSP 服务器文档](LSP_SERVER.md) - Language Server Protocol 支持

### 开发工具
- [样式指南](STYLE_GUIDE.md) - 代码风格和规范

## 📖 API 文档

### 脚本系统
#### Entity API
- [高级示例](../examples/entity_api_advanced.lua) - Entity API 高级用法
- [游戏示例](../examples/entity_api_game.lua) - Entity API 游戏示例
- [Lua 示例](../examples/entity_api_lua.lua) - Lua 集成示例

#### TypeScript 集成
- [TypeScript 示例](../examples/TYPESCRIPT_EXAMPLE.md) - TypeScript 使用说明

### 平台支持
- [移动平台](../src/platform/mobile/README.md) - iOS 和 Android 平台支持

## 🏗️ 架构文档

### 架构决策记录 (ADR)
1. [001-为什么选择 ECS 架构](adr/001-why-ecs.md)
2. [002-为什么选择 WebGPU](adr/002-why-webgpu.md)
3. [003-异步设计决策](adr/003-async-design.md)

### 设计文档
- [错误处理宏指南](ERROR_MACROS_GUIDE.md) - 错误处理最佳实践
- [错误处理改进计划](ERROR_HANDLING_IMPROVEMENT_PLAN.md) - 错误处理系统演进

## 📚 教程

### 基础教程
1. [快速开始](tutorials/getting_started.md)
2. [ECS 系统](tutorials/ecs_guide.md)
3. [渲染系统](tutorials/rendering_guide.md)

### 高级教程
1. [调试 UI 实现](P1-3-debug-ui-implementation.md)
2. [热重载优化](P1-5-hot-reload-optimization.md)

## 📊 任务报告

### P0 阶段（核心功能）
- [测试覆盖率分析](TEST_COVERAGE_ANALYSIS.md) - 测试覆盖情况报告

### P1 阶段（高优先级）
#### P1-1: UI 系统
- [完成总结](P1-1_completion_summary.md)

#### P1-2: 动画系统
- [实现报告](P1-2_IMPLEMENTATION_REPORT.md)
- [完成总结](P1-2_COMPLETION_SUMMARY.md)

#### P1-3: 调试系统
- [完成总结](P1-3-completion-report.md)
- [文件列表](P1-3-file-list.md)
- [快速开始](P1-3-quick-start.md)
- [README 补充](P1-3-README-ADDITION.md)

#### P1-4: Unity 迁移
- (待补充)

#### P1-5: 性能分析
- [完成总结](P1-5_COMPLETION_SUMMARY.md)
- [验证报告](P1-5_VERIFICATION_REPORT.md)
- [快速开始](P1-5-QUICK_START.md)

### P2 阶段（中优先级）
#### P2-1: LLM 集成
- [任务完成报告](P2-1_Task_Completion_Report.md)
- [README](LLM_Integration_README.md)
- [总结](P2-1_LLM_Integration_Summary.md)

#### P2-4: DCC 工具
- [总结](P2-4_DCC_TOOLS_SUMMARY.md)

#### P2-5: 构建系统
- [XMake 实现报告](reports/P2-5_xmake_implementation_report.md)

### P3 阶段（低优先级）
- [P3-1 文件列表](P3-1_FILE_LIST.md)

### 综合报告
- [阶段 1 总结](PHASE_1_SUMMARY.md)
- [技术债务清理](TECHNICAL_DEBT_CLEANUP.md)
- [技术债务最终报告](TECHNICAL_DEBT_FINAL_REPORT.md)
- [文档改进报告](DOCUMENTATION_IMPROVEMENT_REPORT.md)
- [项目状态更新](PROJECT_STATUS_UPDATE.md)

### 开发进度
- [Clippy 会话 1](PHASE_1_CLIPPY_SESSION_1.md)
- [Clippy 会话 2](PHASE_1_CLIPPY_SESSION_2.md)
- [Clippy 会话 3](PHASE_1_CLIPPY_SESSION_3.md)
- [Clippy 会话 4](PHASE_1_CLIPPY_SESSION_4.md)
- [Clippy 会话 5](PHASE_1_CLIPPY_SESSION_5.md)
- [Clippy 进度](PHASE_1_CLIPPY_PROGRESS.md)

### 编译和测试
- [编译错误修复](COMPILATION_ERROR_FIXES.md)
- [编译错误最终报告](COMPILATION_ERROR_FINAL_REPORT.md)
- [测试编译修复](TEST_COMPILATION_FIXES.md)
- [测试修复进度](TEST_FIX_PROGRESS_REPORT.md)
- [警告清理报告](WARNING_CLEANUP_REPORT.md)

### 特定任务报告
- [任务 5.1 总结](TASK_5.1_SUMMARY.md)
- [任务 5.2 总结](TASK_5.2_SUMMARY.md)
- [任务 5.3 总结](TASK_5.3_SUMMARY.md)

### 文档迁移
- [文档迁移进度](DOC_MIGRATION_PROGRESS.md)

### 条件编译
- [条件编译分析](CONDITIONAL_COMPILATION_ANALYSIS.md)

## 🔧 维护文档

### 代码质量
- [样式指南](STYLE_GUIDE.md) - 编码规范和最佳实践
- [测试指南](TESTING_GUIDE.md) - 测试编写指南（待创建）
- [贡献指南](CONTRIBUTING.md) - 贡献流程指南（待创建）

## 🗺️ 文档结构

```
docs/
├── INDEX.md                        # 本文件 - 文档索引
├── adr/                            # 架构决策记录
│   ├── 001-why-ecs.md
│   ├── 002-why-webgpu.md
│   └── 003-async-design.md
├── tutorials/                       # 教程
│   ├── getting_started.md
│   ├── ecs_guide.md
│   └── rendering_guide.md
├── reports/                        # 详细报告
│   └── P2-5_xmake_implementation_report.md
├── *.md                            # 各种报告和指南
└── P*-*.md                         # 阶段性任务报告
```

## 📝 文档贡献

如果您想为文档做出贡献，请遵循以下步骤：

1. **确定文档类型**：是新教程、API 文档、还是修复现有文档
2. **遵循样式指南**：参考 [样式指南](STYLE_GUIDE.md)
3. **使用清晰的标题结构**：使用 Markdown 标题（#, ##, ###）
4. **添加代码示例**：使用 \```rust 等代码块
5. **更新索引**：在本文件中添加新文档的链接

## 🔍 文档搜索

使用以下方式搜索文档：
- **按主题**：查看上面的分类章节
- **按阶段**：查看任务报告部分
- **按关键词**：使用浏览器的查找功能（Ctrl+F 或 Cmd+F）

## 📞 获取帮助

如果您在文档中找不到所需信息：
1. 查看 [快速开始指南](tutorials/getting_started.md)
2. 检查 [API 文档](#api-文档)
3. 查看 [常见问题](FAQ.md)（待创建）
4. 提交 Issue 到项目仓库

## 🔄 文档更新

文档最后更新：2026-01-02

引擎版本：v2.0.0

---

**提示**：将此页面加入书签，作为快速访问所有文档的入口点！
