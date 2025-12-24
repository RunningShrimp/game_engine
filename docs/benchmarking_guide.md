# 基准测试和性能回归检测指南

本指南说明如何运行基准测试和进行性能回归检测。

## 概述

基准测试用于测量代码性能，性能回归检测用于确保代码更改不会导致性能下降。

## 快速开始

### 运行基准测试和回归检测

```bash
# 运行所有基准测试并检测性能回归
./scripts/run_benchmarks_and_regression.sh
```

脚本会自动：
1. 运行所有基准测试
2. 收集性能数据
3. 与基线进行比较
4. 生成报告
5. 检测性能回归

## 基准测试列表

游戏引擎包含以下基准测试：

### 1. 数学运算基准测试 (`math_benchmarks`)

测试向量、矩阵、四元数等数学运算的性能：
- 向量运算（加法、乘法、点积、叉积）
- 矩阵运算（乘法、转置、逆矩阵）
- SIMD优化版本对比

### 2. ECS基准测试 (`ecs_benchmarks`)

测试实体组件系统的性能：
- 实体创建和销毁
- 组件添加和移除
- 系统执行性能
- 查询性能

### 3. 渲染基准测试 (`render_benchmarks`)

测试渲染系统的性能：
- 绘制调用性能
- 批处理性能
- GPU驱动渲染性能
- 后处理效果性能

### 4. 寻路基准测试 (`pathfinding_benchmarks`)

测试寻路算法的性能：
- A*算法性能
- 并行寻路性能
- 异步寻路性能

### 5. 资源管理基准测试 (`resource_benchmarks`)

测试资源管理的性能：
- 资源加载性能
- 资源缓存性能
- 内存分配性能

## 手动运行基准测试

### 运行单个基准测试

```bash
# 运行数学运算基准测试
cargo bench --package game_engine --bench math_benchmarks

# 运行ECS基准测试
cargo bench --package game_engine --bench ecs_benchmarks

# 运行渲染基准测试
cargo bench --package game_engine --bench render_benchmarks
```

### 运行所有基准测试

```bash
# 使用脚本
./scripts/run_benchmarks.sh

# 或手动运行
cargo bench --package game_engine
```

### 基准测试选项

```bash
# 快速模式（较少采样）
cargo bench --package game_engine --bench math_benchmarks -- --sample-size 20

# 详细模式（更多采样）
cargo bench --package game_engine --bench math_benchmarks -- --sample-size 100

# 不生成图表（加快速度）
cargo bench --package game_engine --bench math_benchmarks -- --noplot

# 详细输出
cargo bench --package game_engine --bench math_benchmarks -- --verbose
```

## 性能回归检测

### 建立基线

首次运行时会自动建立基线：

```bash
./scripts/run_benchmarks_and_regression.sh
```

基线文件保存在：`target/benchmark_results/performance_baseline.json`

### 手动建立基线

```bash
# 运行基准测试
./scripts/run_benchmarks.sh

# 保存基线
cp target/benchmark_results/performance_current.json \
   target/benchmark_results/performance_baseline.json
```

### 检测回归

运行回归检测脚本：

```bash
./scripts/performance_regression.sh
```

或使用综合脚本：

```bash
./scripts/run_benchmarks_and_regression.sh
```

### 回归阈值

默认阈值：
- **警告阈值**: 10%性能下降
- **严重阈值**: 20%性能下降

修改阈值（编辑脚本）：
```bash
THRESHOLD_WARNING=10   # 警告阈值
THRESHOLD_CRITICAL=20  # 严重阈值
```

## 查看结果

### Criterion报告

基准测试结果保存在 `target/criterion/` 目录：

```bash
# 查看HTML报告
open target/criterion/math_benchmarks/report/index.html
```

### JSON结果

性能数据保存在JSON文件中：

```bash
# 查看当前结果
cat target/benchmark_results/performance_current.json | jq '.'

# 查看基线
cat target/benchmark_results/performance_baseline.json | jq '.'
```

### Markdown报告

综合脚本会生成Markdown报告：

```bash
# 查看最新报告
ls -t target/benchmark_results/benchmark_report_*.md | head -1 | xargs cat
```

## 性能分析

### 识别性能瓶颈

1. **查看Criterion报告**:
   - 打开HTML报告
   - 查看各测试的详细统计
   - 识别最慢的操作

2. **对比不同实现**:
   ```bash
   # 运行基准测试对比
   cargo bench --package game_engine --bench math_benchmarks
   # 查看SIMD vs 非SIMD性能对比
   ```

3. **使用性能分析器**:
   ```bash
   # 使用perf (Linux)
   perf record cargo bench --package game_engine --bench math_benchmarks
   perf report
   
   # 使用Instruments (macOS)
   xcrun xctrace record --template "Time Profiler" \
     --launch -- cargo bench --package game_engine --bench math_benchmarks
   ```

### 性能优化建议

1. **SIMD优化**: 使用SIMD指令加速数学运算
2. **并行化**: 使用并行处理加速独立操作
3. **缓存优化**: 减少内存分配，使用对象池
4. **算法优化**: 选择更高效的算法
5. **批处理**: 合并操作减少开销

## CI/CD集成

### GitHub Actions

```yaml
name: Performance Regression

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y jq bc
      - name: Run benchmarks
        run: ./scripts/run_benchmarks_and_regression.sh
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: target/benchmark_results/
```

### GitLab CI

```yaml
benchmark:
  stage: test
  image: rust:latest
  before_script:
    - apt-get update && apt-get install -y jq bc
  script:
    - ./scripts/run_benchmarks_and_regression.sh
  artifacts:
    paths:
      - target/benchmark_results/
    expire_in: 30 days
```

## 最佳实践

### 1. 定期运行基准测试

- **每次重要提交**: 运行关键基准测试
- **每周**: 运行完整基准测试套件
- **发布前**: 运行所有基准测试并检查回归

### 2. 维护基线

- **更新基线**: 当性能改进时更新基线
- **版本化基线**: 为不同版本维护不同基线
- **审查基线**: 定期审查基线是否合理

### 3. 关注关键指标

- **核心功能**: 优先关注核心功能的性能
- **热点路径**: 关注性能热点路径
- **用户体验**: 关注影响用户体验的指标

### 4. 性能测试策略

- **真实场景**: 使用真实场景进行测试
- **多种负载**: 测试不同负载下的性能
- **边界条件**: 测试边界条件和极端情况

### 5. 回归处理

- **调查原因**: 深入调查性能回归的原因
- **权衡考虑**: 考虑功能改进 vs 性能损失
- **文档记录**: 记录性能回归的原因和决策

## 常见问题

### Q: 基准测试结果不稳定？

**A**: 可能原因：
1. 系统负载变化
2. CPU频率变化
3. 缓存未预热

**解决方案**:
- 多次运行取平均值
- 使用固定CPU频率
- 预热缓存
- 使用专用测试机器

### Q: 如何比较不同版本的性能？

**A**: 
1. 保存每个版本的基线
2. 使用历史记录对比
3. 使用性能趋势图

### Q: 基准测试很慢？

**A**: 优化建议：
1. 使用快速模式：`--sample-size 20`
2. 只运行关键基准测试
3. 使用并行执行
4. 跳过图表生成：`--noplot`

### Q: 如何添加新的基准测试？

**A**: 
1. 在 `game_engine/benches/` 创建新文件
2. 使用 `criterion` 框架编写测试
3. 添加到基准测试列表
4. 更新基线

## 相关文档

- [性能调优指南](performance_tuning_guide.md)
- [性能监控增强](../PERFORMANCE_MONITORING_ENHANCEMENTS.md)
- [基准测试验证指南](guides/render_benchmarks_verification.md)
- [Criterion文档](https://docs.rs/criterion/)

