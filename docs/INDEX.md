# 游戏引擎文档索引

本索引帮助您快速找到所需的文档。

## 快速开始

### 新手入门
1. [README.md](../README.md) - 项目简介和快速开始
2. [快速开始指南](guides/getting_started_guide.md) - 详细入门教程
3. [运行示例](../game_engine/examples/README.md) - 示例代码说明

### 第一次运行
```bash
# 克隆仓库
git clone <repository>
cd game_engine/game_engine

# 运行Hello World
cargo run --example hello_world

# 生成文档
cargo doc --open
```

## 核心概念

### 架构文档
- [整体架构概览](architecture/overview.md) - 引擎架构设计
- [ECS架构详解](architecture/ecs.md) - 实体组件系统
- 渲染管线 - 参见 [render/mod.rs](../game_engine/src/render/mod.rs)
- 物理系统 - 参见 [physics/mod.rs](../game_engine/src/physics/mod.rs)
- Domain层设计 - 参见 [domain/mod.rs](../game_engine/src/domain/mod.rs)

### 设计模式
- [CQRS指南](guides/cqrs_guide.md) - 命令查询职责分离
- [事件溯源指南](guides/event_sourcing_guide.md) - Event Sourcing模式
- [插件系统指南](guides/plugin_system_guide.md) - 插件架构

## API参考

### 核心模块
- [Engine](../game_engine/src/core/engine/engine.rs) - 引擎核心API
- [ECS](../game_engine/src/ecs/mod.rs) - 实体组件系统
- [渲染](../game_engine/src/render/mod.rs) - 渲染系统
- [物理](../game_engine/src/physics/mod.rs) - 物理系统
- [音频](../game_engine/src/audio/mod.rs) - 音频系统
- [网络](../game_engine/src/network/mod.rs) - 网络同步
- [Domain](../game_engine/src/domain/mod.rs) - 领域层

### 工具模块
- [性能监控](../game_engine/src/performance/mod.rs) - 性能分析
- [资源管理](../game_engine/src/resources/mod.rs) - 资源加载
- [配置系统](../game_engine/src/config/mod.rs) - 配置管理

## 示例代码

### 基础示例
| 示例 | 描述 | 运行命令 |
|------|------|----------|
| hello_world | 最简单的示例 | `cargo run --example hello_world` |
| ecs_basics | ECS基础 | `cargo run --example ecs_basics` |
| rendering | 渲染系统 | `cargo run --example rendering` |
| physics | 物理系统 | `cargo run --example physics` |
| audio | 音频系统 | `cargo run --example audio` |

### 高级示例
| 示例 | 描述 | 运行命令 |
|------|------|----------|
| render_advanced | 高级渲染 | `cargo run --example render_advanced` |
| domain | Domain层 | `cargo run --example domain` |
| cqrs_example | CQRS模式 | `cargo run --example cqrs_example` |
| event_sourcing_example | 事件溯源 | `cargo run --example event_sourcing_example` |
| multiplayer | 多人游戏 | `cargo run --example multiplayer` |
| network_multiplayer | 网络多人 | `cargo run --example network_multiplayer` |

### 性能和调试
| 示例 | 描述 | 运行命令 |
|------|------|----------|
| performance_benchmark_example | 性能基准 | `cargo run --example performance_benchmark_example` |
| tracy_profiling | Tracy分析 | `cargo run --example tracy_profiling` |
| world_inspector_example | World检查器 | `cargo run --example world_inspector_example` |

## 指南和教程

### 功能指南
- [服务层指南](guides/service_layer_guide.md)
- [特性标志指南](guides/feature_flags_guide.md)
- [错误处理指南](guides/error_handling_guide.md)
- [对象池使用指南](guides/object_pool_usage_guide.md)

### 高级功能
- [异步寻路指南](guides/async_pathfinding_guide.md)
- [移动平台指南](guides/mobile_platform_guide.md)
- [WASM构建指南](guides/wasm_build_guide.md)
- [后处理API指南](guides/postprocess_api_guide.md)
- [软体物理指南](guides/soft_body_physics_guide.md)

### 性能优化
- [性能调优指南](performance_tuning_guide.md)
- [基准测试指南](benchmarking_guide.md)
- [Tracy性能分析](tracy_profiling_guide.md)
- [Tracy配置](tracy_setup.md)

## 架构决策记录（ADR）

### 核心架构
- [ADR-001: ECS架构](adr/0001-ecs-architecture.md)
- [ADR-002: 领域驱动设计](adr/0002-domain-driven-design.md)
- [ADR-004: 并发模型](adr/0004-concurrency-model.md)
- [ADR-005: Rust语言选择](adr/0005-rust-language-choice.md)

### 渲染和物理
- [ADR-003: 渲染管线](adr/0003-rendering-pipeline.md)
- [ADR-012: 软体物理](adr/0012-soft-body-physics.md)

### 高级功能
- [ADR-010: CQRS模式](adr/0010-cqrs-pattern.md)
- [ADR-009: 事件溯源增强](adr/0009-event-sourcing-enhancements.md)
- [ADR-007: 统一资源管理](adr/0007-unified-resource-management.md)
- [ADR-006: 后处理效果管理器](adr/0006-postprocess-effect-manager.md)

### 平台支持
- [ADR-008: WASM优化](adr/0008-wasm-optimization.md)
- [ADR-011: 移动平台支持](adr/0011-mobile-platform-support.md)

## 质量和测试

### 测试指南
- [测试覆盖基线](TEST_COVERAGE_BASELINE.md)
- [覆盖率报告指南](coverage_report_guide.md)

### 质量追踪
- [Clippy迁移追踪](quality-tracker/clippy-migration-tracker.md)
- [未使用代码修复计划](quality-tracker/unused-fix-plan.md)
- [死代码修复计划](quality-tracker/dead-code-fix-plan.md)
- [命名约定修复计划](quality-tracker/naming-conventions-fix-plan.md)

## 开发相关

### 构建和编译
- [条件编译指南](CONDITIONAL_COMPILATION_GUIDE.md)
- [条件编译审计](CONDITIONAL_COMPILATION_AUDIT.md)
- [配置优化报告](../CFG_OPTIMIZATION_REPORT.md)

### CI/CD
- [CI/CD优化](cicd_optimization.md)
- [性能回归CI](PERFORMANCE_REGRESSION_CI.md)

### 最佳实践
- [最佳实践](best_practices.md)
- [API参考](api_reference.md)

## 功能特性

### AI系统
- [AI功能增强](ai_features_enhancement.md)

### 编辑器
- [编辑器功能增强](editor_features_enhancement.md)

### 全局光照
- [全局光照](global_illumination.md)

### 网络和回放
- [回放系统](replay_system.md)

### Ray Tracing
- [光线追踪集成](ray_tracing_integration.md)

## 故障排除

### 问题解决
- [故障排除](troubleshooting.md)

### 实现总结
- [实现总结](implementation_summary.md)

## 项目管理

### 进度追踪
- [执行进度报告](execution-progress-report.md)
- [进度更新2](progress-update-2.md)
- [进度更新3](progress-update-3.md)

### TODO追踪
- [TODO追踪](TODO_TRACKING.md)

## 按主题查找

### 我想学习...
- **ECS基础** → [ECS架构](architecture/ecs.md) + [ecs_basics示例](../game_engine/examples/ecs_basics.rs)
- **渲染** → [渲染模块](../game_engine/src/render/mod.rs) + [rendering示例](../game_engine/examples/rendering.rs)
- **物理** → [物理模块](../game_engine/src/physics/mod.rs) + [physics示例](../game_engine/examples/physics.rs)
- **音频** → [音频模块](../game_engine/src/audio/mod.rs) + [audio示例](../game_engine/examples/audio.rs)
- **网络** → [网络模块](../game_engine/src/network/mod.rs) + [multiplayer示例](../game_engine/examples/multiplayer.rs)
- **DDD** → [Domain模块](../game_engine/src/domain/mod.rs) + [domain示例](../game_engine/examples/domain.rs)

### 我想解决...
- **性能问题** → [性能调优指南](performance_tuning_guide.md) + [Tracy分析](tracy_profiling_guide.md)
- **编译错误** → [故障排除](troubleshooting.md) + [条件编译指南](CONDITIONAL_COMPILATION_GUIDE.md)
- **API使用** → [API参考](api_reference.md) + [示例代码](../game_engine/examples/README.md)

### 我想了解...
- **架构设计** → [架构概览](architecture/overview.md) + [ADR索引](adr/README.md)
- **设计决策** → [ADR文档](adr/)
- **最佳实践** → [最佳实践](best_practices.md)

## 文档贡献

### 如何贡献文档
1. 遵循现有文档风格
2. 添加代码示例
3. 包含使用场景
4. 更新本索引

### 文档模板
参见 [lib.rs](../game_engine/src/lib.rs) 和各模块的 `mod.rs` 文件。

---

**最后更新**: 2025-12-28
**维护者**: Game Engine Team
