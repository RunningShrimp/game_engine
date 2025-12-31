# 性能基准测试指南

## 概述

本目录包含游戏引擎的性能基准测试，用于验证和监控关键组件的性能指标。

## 基准测试分类

### 1. 验证框架基准 (`validation_benchmarks.rs`)

测试验证框架在各种场景下的性能表现。

#### 测试项目

- **简单值对象验证**
  - `bench_position_validation` - 位置验证
  - `bench_velocity_validation` - 速度验证
  - `bench_mass_validation` - 质量验证

- **批量验证**
  - `bench_batch_position_validation_100` - 批量验证100个位置
  - `bench_batch_mass_validation_1000` - 批量验证1000个质量

- **复杂对象验证**
  - `bench_entity_validation` - 游戏实体验证（包含多个值对象）

- **验证器函数性能**
  - `bench_validate_finite` - 有限数验证
  - `bench_validate_range` - 范围验证
  - `bench_validate_non_negative` - 非负数验证

- **游戏循环模拟**
  - `bench_game_loop_validation_60_frames` - 模拟60帧游戏循环中的验证

- **创建和验证**
  - `bench_create_and_validate_position` - 创建并验证位置
  - `bench_create_and_validate_mass` - 创建并验证质量

- **集合验证**
  - `bench_vec_validation_10` - 验证10个元素的向量
  - `bench_vec_validation_100` - 验证100个元素的向量

### 2. AI系统基准 (`ai_benchmarks.rs`)

测试行为树和覆盖图系统的性能表现。

#### 测试项目

- **覆盖图传播**
  - `bench_influence_propagate_50x50_5_iterations` - 50x50网格5次迭代
  - `bench_influence_propagate_100x100_10_iterations` - 100x100网格10次迭代

- **战术覆盖图更新**
  - `bench_tactical_map_update` - 战术地图更新

- **位置分析**
  - `bench_analyze_position` - 位置分析
  - `bench_find_best_position` - 查找最佳位置

- **高斯平滑**
  - `bench_gaussian_smooth_50x50` - 50x50网格高斯平滑

- **行为树执行**
  - `bench_sequence_5_nodes` - 5节点序列执行
  - `bench_selector_5_nodes` - 5节点选择器执行

- **实时AI决策循环**
  - `bench_realtime_ai_decision_10_steps` - 10步实时AI决策

- **多单位分析**
  - `bench_multi_unit_analysis_10_units` - 10个单位的分析

- **覆盖图操作**
  - `bench_add_source` - 添加影响力源
  - `bench_get_value` - 获取值
  - `bench_find_max` - 查找最大值
  - `bench_find_min` - 查找最小值

## 运行基准测试

### 运行所有基准测试

```bash
cargo bench
```

### 运行特定基准测试文件

```bash
# 只运行验证框架基准
cargo bench --bench validation_benchmarks

# 只运行AI系统基准
cargo bench --bench ai_benchmarks
```

### 运行特定测试

```bash
cargo bench bench_position_validation
```

## 性能指标解读

### 验证框架

期望的性能指标（参考值）：

- **简单值对象验证**: < 100ns/操作
  - 位置验证: ~50ns
  - 速度验证: ~60ns
  - 质量验证: ~40ns

- **批量验证**: < 10μs/100个对象
  - 100个位置: ~8μs
  - 1000个质量: ~80μs

- **游戏循环**: < 1ms/60帧
  - 60帧验证: ~500μs

### AI系统

期望的性能指标（参考值）：

- **覆盖图传播**:
  - 50x50, 5次迭代: ~5ms
  - 100x100, 10次迭代: ~40ms

- **战术分析**:
  - 位置分析: < 1μs
  - 查找最佳位置: ~10ms (100x100网格)

- **行为树**:
  - 5节点序列: ~100ns
  - 5节点选择器: ~150ns

## 性能优化建议

### 验证框架

1. **批量操作**: 使用向量批量验证而非逐个验证
2. **构造时验证**: 在值对象构造时验证，避免重复验证
3. **缓存结果**: 对不变的对象缓存验证结果

### AI系统

1. **网格大小**: 根据实际需求选择合适的网格大小
2. **迭代次数**: 减少传播迭代次数，使用更高的衰减系数
3. **增量更新**: 只更新变化区域而非整个网格
4. **空间分区**: 对大型地图使用空间分区技术

## 持续集成

基准测试应作为CI/CD流程的一部分：

```yaml
# .github/workflows/benchmark.yml
name: Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: cargo bench
      - name: Store benchmark result
        uses: benchmark-action/github-action-benchmark@v1
        with:
          tool: 'cargo'
          output-file-path: benchmark.txt
```

## 性能回归检测

使用 `cargo critcompare` 比较不同版本的基准测试结果：

```bash
# 安装 critcmp
cargo install critcmp

# 保存基准结果
cargo bench --bench validation_benchmarks | tee baseline.txt

# 修改代码后再次运行
cargo bench --bench validation_benchmarks | tee new.txt

# 比较结果
critcmp baseline.txt new.txt
```

## 故障排查

### 基准测试超时

如果基准测试超时，可能是：

1. 测试规模过大（减小迭代次数或网格大小）
2. 系统资源不足（关闭其他程序）
3. 编译优化未启用（确保使用 `--release` 模式）

### 结果波动大

如果结果波动较大：

1. 增加迭代次数（修改 `bench` 函数的迭代参数）
2. 关闭后台程序
3. 使用稳定的CPU频率（禁用Turbo Boost）
4. 多次运行取平均值

## 贡献指南

添加新的基准测试时：

1. 选择有意义的测试场景
2. 提供清晰的文档说明
3. 包含期望的性能指标
4. 确保测试可重复执行
5. 添加到本README的相应分类中

## 相关文档

- [验证框架文档](../docs/VALIDATION_FRAMEWORK_GUIDE.md)
- [AI系统文档](../docs/AI_SYSTEMS_GUIDE.md)
- [性能优化指南](../docs/PERFORMANCE_OPTIMIZATION.md)
