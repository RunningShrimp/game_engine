# 性能基准测试指南

**创建日期**: 2025-12-03  
**目的**: 文档化性能基准测试的使用和最佳实践

---

## 概述

项目使用 `criterion` 框架进行性能基准测试。基准测试位于 `benches/` 目录。

---

## 现有基准测试

### 1. 数学运算基准测试

**文件**: `benches/math_benchmarks.rs`

测试内容：
- 向量运算性能
- 矩阵运算性能
- SIMD优化效果

运行方式：
```bash
cargo bench --bench math_benchmarks
```

### 2. ECS系统基准测试

**文件**: `benches/ecs_benchmarks.rs`

测试内容：
- 实体创建和销毁性能
- 组件查询性能
- 系统执行性能

运行方式：
```bash
cargo bench --bench ecs_benchmarks
```

### 3. 物理系统基准测试

**文件**: `benches/physics_benchmarks.rs`

测试内容：
- 物理模拟性能
- 碰撞检测性能
- 并行物理计算性能

运行方式：
```bash
cargo bench --bench physics_benchmarks
```

### 4. 渲染系统基准测试

**文件**: `benches/render_benchmarks.rs`

测试内容：
- 渲染管线性能
- 批渲染性能
- GPU驱动剔除性能

运行方式：
```bash
cargo bench --bench render_benchmarks
```

---

## 性能回归检测

### 建立基线

1. 运行基准测试：
```bash
cargo bench
```

2. 保存基线结果：
```bash
cargo bench -- --save-baseline baseline
```

### 检测回归

1. 运行基准测试并对比基线：
```bash
cargo bench -- --baseline baseline
```

2. 查看性能变化报告

---

## 性能优化最佳实践

### 1. 测量优先

- 在优化前先测量性能
- 识别真正的性能瓶颈
- 使用性能分析工具（如 `perf`、`cargo flamegraph`）

### 2. 渐进优化

- 一次优化一个方面
- 每次优化后测量效果
- 保留性能测试结果

### 3. 关注关键路径

- 优化热点代码路径
- 避免过早优化
- 平衡代码可读性和性能

### 4. 使用SIMD

- 利用SIMD指令加速数学运算
- 使用 `game_engine_simd` crate
- 自动检测CPU特性

### 5. 内存管理

- 使用对象池减少分配
- 使用Arena分配器
- 避免不必要的内存拷贝

---

## 性能指标

### 关键指标

- **帧时间**: 目标 < 16.7ms (60 FPS)
- **绘制调用**: 尽量减少
- **内存使用**: 监控内存占用
- **CPU使用**: 监控CPU负载

### 性能分析工具

- `cargo flamegraph`: 生成火焰图
- `perf`: Linux性能分析工具
- `cargo bench`: 基准测试
- `cargo profdata`: 性能数据收集

---

## 更新记录

- 2025-12-03: 创建性能基准测试指南

