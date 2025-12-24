# Profiling模块职责划分

## 概述

游戏引擎的profiling功能分布在多个crate中，本文档明确了各模块的职责边界和推荐使用方式。

## Crate层级结构

```
game_engine (核心引擎)
  ├── performance/ (性能优化，依赖profiling)
  │   ├── profiling/ (核心profiling实现)
  │   ├── tracing_metrics/ (tracing + metrics集成)
  │   └── metrics_storage/ (metrics存储系统)
  └── profiling/ (性能监控和分析工具)

game_engine_performance (性能工具集)
  ├── benchmarking/ (基准测试工具)
  ├── cicd/ (CI/CD集成)
  ├── gpu/ (GPU性能优化)
  ├── memory/ (内存优化)
  ├── monitoring/ (系统监控)
  ├── optimization/ (特定领域优化)
  ├── rendering/ (渲染优化)
  ├── sync/ (同步工具)
  └── profiling/ (已废弃，保留用于向后兼容)

game_engine_profiling (性能分析工具)
  ├── benchmarking/ (基准测试工具)
  ├── cicd/ (CI/CD集成)
  ├── monitoring/ (系统监控)
  ├── profiling/ (性能分析工具)
  └── visualization/ (性能可视化)
```

## 模块职责划分

### game_engine::profiling (核心profiling实现)

**职责**：提供核心性能监控和分析功能

**子模块**：
- `metrics` - 性能指标定义和基础类型
- `collector` - 高性能指标收集器
- `storage` - 指标存储和持久化
- `dashboard` - Web监控面板 (需`profiling`特性)
- `visualization` - 性能数据可视化
- `alerting` - 性能告警系统
- `service` - Profiling服务接口

**高级分析工具**（从game_engine_performance迁移）：
- `advanced_profiler` - 高级性能分析器
- `bottleneck_detector` - 性能瓶颈检测
- `continuous_profiler` - 持续性能分析
- `frame_analyzer` - 帧分析工具
- `memory_profiler` - 内存分析器
- `performance_analyzer` - 性能分析器
- `profiler` - 基础分析器

**推荐使用方式**：
```rust
use game_engine::profiling::{
    Profiler, ContinuousProfiler, BottleneckDetector,
    PerformanceAnalyzer, FrameAnalyzer
};
```

### game_engine::performance::tracing_metrics (tracing + metrics集成)

**职责**：统一tracing spans和metrics收集接口

**主要功能**：
- Tracing span管理
- Metrics记录
- 性能分析集成
- 系统监控集成

**推荐使用方式**：
```rust
use game_engine::performance::tracing_metrics::TracingMetricsManager;

let manager = TracingMetricsManager::new();
manager.record_metric("frame_time", 16.67);
```

### game_engine::performance::metrics_storage (metrics存储系统)

**职责**：提供统一的metrics存储和查询接口

**主要功能**：
- 时间序列数据存储
- 数据点聚合统计
- 时间窗口查询
- 数据清理和归档

**推荐使用方式**：
```rust
use game_engine::performance::metrics_storage::MetricsStorage;

let storage = MetricsStorage::new(1000);
storage.record("frame_time", 16.67, None);
let agg = storage.aggregate("frame_time", None);
```

### game_engine_performance (性能工具集)

**职责**：提供独立的性能优化和分析工具

**子模块**：
- `benchmarking/` - 基准测试工具
  - `benchmark` - 基准测试核心
  - `benchmark_baselines` - 基准基线管理
  - `benchmark_runner` - 基准运行器
  - `critical_path_benchmarks` - 关键路径基准
  - `gpu_comparative_benchmark` - GPU比较基准
  - `optimization_validation` - 优化验证
  - `regression_testing` - 回归测试

- `cicd/` - CI/CD集成工具
  - `cicd_manager` - CI/CD管理器

- `gpu/` - GPU性能优化
  - `gpu_compute` - GPU计算优化
  - `gpu_physics` - GPU物理加速
  - `wgpu_integration` - WGPU集成

- `memory/` - 内存优化
  - `arena` - Arena分配器
  - `bump` - Bump分配器
  - `memory_optimization` - 内存优化策略
  - `object_pool` - 对象池

- `monitoring/` - 系统监控
  - `monitoring_legacy` - 遗留监控工具
  - `system_monitor` - 系统性能监控

- `optimization/` - 特定领域优化
  - `ai_pathfinding` - AI路径查找优化
  - `audio_pipeline` - 音频流水线优化

- `rendering/` - 渲染优化
  - `batch_renderer` - 批处理渲染器

- `sync/` - 同步工具
  - `synchronized` - 同步原语

**推荐使用方式**：
```rust
use game_engine_performance::{
    benchmarking::Benchmark,
    memory::ObjectPool,
    gpu::GpuCompute
};
```

### game_engine_profiling (性能分析工具)

**职责**：提供专业的性能分析和可视化工具

**子模块**：
- `benchmarking/` - 基准测试工具
- `cicd/` - CI/CD集成
- `monitoring/` - 系统监控
- `profiling/` - 性能分析
- `visualization/` - 性能可视化

**推荐使用方式**：
```rust
use game_engine_profiling::{Profiler, Benchmark, PerformanceDashboard};
```

## 依赖关系图

```
┌─────────────────────────────────────────────────────┐
│          game_engine_profiling                      │
│  (独立性能分析工具，无依赖循环)                   │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│          game_engine_performance                    │
│  (独立性能工具集，无依赖循环)                      │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│              game_engine                         │
│  ┌───────────────────────────────────────────┐   │
│  │          performance/                      │   │
│  │  ┌──────────────┐  ┌──────────────┐ │   │
│  │  │   profiling  │  │ tracing_     │ │   │
│  │  │  (核心实现)  │  │ metrics       │ │   │
│  │  └──────────────┘  └──────────────┘ │   │
│  └───────────────────────────────────────────┘   │
│                                                  │
│  ┌───────────────────────────────────────────┐   │
│  │          profiling/                       │   │
│  │  (监控和分析工具)                         │   │
│  └───────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

## 使用建议

### 1. 新项目推荐

**使用game_engine::profiling作为主要profiling入口**
- 提供最完整的性能分析功能
- 与引擎核心紧密集成
- 支持所有高级分析特性

### 2. 基准测试需求

**使用game_engine_performance::benchmarking**
- 独立的基准测试框架
- 支持回归测试
- 可与CI/CD集成

### 3. 专业性能分析

**使用game_engine_profiling**
- 专业的性能分析工具
- 可视化面板
- 适用于深度性能调优

### 4. 轻量级监控

**使用game_engine::performance::tracing_metrics**
- 集成tracing spans
- 统一的metrics接口
- 最小性能开销

## 迁移指南

### 从game_engine_performance::profiling迁移

```rust
// 旧代码（已废弃）
use game_engine_performance::profiling::Profiler;

// 新代码（推荐）
use game_engine::profiling::Profiler;
```

### 从game_engine::performance迁移（重复模块）

```rust
// 旧代码（render_optimization）
use game_engine_performance::rendering::render_optimization::FrustumCulling;

// 新代码（推荐）
use game_engine::performance::rendering::FrustumCulling;
```

## 注意事项

1. **避免循环依赖**：不要在game_engine核心中依赖game_engine_performance或game_engine_profiling
2. **职责清晰**：核心profiling功能在game_engine::profiling，工具在独立crate
3. **向后兼容**：保留game_engine_performance的profiling模块作为空导出
4. **文档更新**：使用新路径时，确保文档同步更新
