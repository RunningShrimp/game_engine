# 性能基准测试文档

本文档说明如何运行和使用游戏引擎的性能基准测试。

---

## 目录

- [概述](#概述)
- [运行基准测试](#运行基准测试)
- [基准测试列表](#基准测试列表)
- [性能指标解读](#性能指标解读)
- [性能回归检测](#性能回归检测)
- [最佳实践](#最佳实践)

---

## 概述

性能基准测试用于：
- **测量性能**: 量化关键操作的性能指标
- **检测回归**: 在代码变更后检测性能退化
- **优化验证**: 验证优化措施的实际效果
- **性能对比**: 对比不同实现方案的性能

### 基准测试工具

我们使用 [Criterion.rs](https://github.com/bheisler/criterion.rs) 作为基准测试框架，它提供：
- 统计分析（平均值、中位数、标准差）
- HTML报告生成
- 性能回归检测
- 自动对比不同版本

---

## 运行基准测试

### 基本命令

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench math_benchmarks

# 运行特定基准测试函数
cargo bench --bench math_benchmarks vec3_operations

# 生成HTML报告（自动生成在 target/criterion/ 目录）
cargo bench --bench math_benchmarks -- --save-baseline baseline
```

### 环境要求

- **Rust工具链**: stable 或 nightly（推荐nightly以获得更好的性能）
- **系统资源**: 确保有足够的CPU和内存
- **稳定环境**: 关闭其他占用资源的程序，确保测试环境稳定

### 性能基线

首次运行基准测试时，建议保存基线：

```bash
# 保存基线
cargo bench --bench math_benchmarks -- --save-baseline baseline

# 后续运行对比基线
cargo bench --bench math_benchmarks -- --baseline baseline
```

---

## 基准测试列表

### 1. 数学运算基准测试 (`math_benchmarks`)

**位置**: `benches/math_benchmarks.rs`

**测试内容**:
- 向量运算（加法、点积、叉积、归一化、距离）
- 矩阵运算（乘法、变换、逆矩阵）
- 四元数运算（乘法、插值、旋转）
- SIMD优化对比

**运行**:
```bash
cargo bench --bench math_benchmarks
```

**性能目标**:
- 向量运算: < 1ns/操作
- 矩阵乘法: < 10ns/操作
- SIMD批量变换: > 4x 加速（相比标量）

### 2. ECS系统基准测试 (`ecs_benchmarks`)

**位置**: `benches/ecs_benchmarks.rs`

**测试内容**:
- 实体创建和销毁
- 组件添加和移除
- 系统查询性能
- 批量操作性能

**运行**:
```bash
cargo bench --bench ecs_benchmarks
```

**性能目标**:
- 实体创建: < 100ns/实体
- 组件查询: < 10ns/查询
- 批量操作: 线性扩展，无明显性能下降

### 3. 渲染系统基准测试 (`render_benchmarks`)

**位置**: `benches/render_benchmarks.rs`

**测试内容**:
- 批次构建性能
- 实例化渲染性能
- LOD选择性能
- 视锥体剔除性能

**运行**:
```bash
cargo bench --bench render_benchmarks
```

**性能目标**:
- 批次构建: < 1ms（1000个对象）
- LOD选择: < 100ns/对象
- 视锥体剔除: < 50ns/对象

### 4. 物理系统基准测试 (`physics_benchmarks`)

**位置**: `benches/physics_benchmarks.rs`

**测试内容**:
- 物理步进性能
- 碰撞检测性能
- 力应用性能
- 并行物理性能

**运行**:
```bash
cargo bench --bench physics_benchmarks
```

**性能目标**:
- 物理步进: < 16ms（60fps目标）
- 碰撞检测: < 1μs/碰撞对
- 并行物理: > 2x 加速（相比单线程）

---

## 性能指标解读

### 关键指标

1. **时间 (Time)**
   - **平均值 (Mean)**: 多次运行的平均时间
   - **中位数 (Median)**: 50%的运行时间
   - **标准差 (Std Dev)**: 时间波动程度

2. **吞吐量 (Throughput)**
   - 每秒操作数 (ops/sec)
   - 用于对比不同实现的效率

3. **性能变化 (Change)**
   - 相对于基线的性能变化百分比
   - 正值表示性能下降，负值表示性能提升

### 示例输出解读

```
vec3_add          time:   [0.500 ns 0.501 ns 0.502 ns]
                  change: [-0.10% +0.20% +0.50%] (p = 0.15 > 0.05)
```

解读：
- **时间**: 0.500-0.502 纳秒
- **变化**: 相对于基线，性能变化在 -0.10% 到 +0.50% 之间
- **p值**: 0.15 > 0.05，变化不显著（可能是测量误差）

---

## 性能回归检测

### 自动检测

Criterion会自动检测性能回归：

```bash
# 运行并对比基线
cargo bench --bench math_benchmarks -- --baseline baseline
```

如果检测到性能回归（>5%），Criterion会：
- 标记为性能回归
- 在报告中高亮显示
- 提供统计显著性分析

### CI集成

在CI中运行基准测试：

```yaml
# .github/workflows/benchmarks.yml
name: Benchmarks

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  benchmarks:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      
      - name: Run benchmarks
        run: |
          cargo bench --bench math_benchmarks -- --save-baseline ci
          cargo bench --bench ecs_benchmarks -- --save-baseline ci
          cargo bench --bench render_benchmarks -- --save-baseline ci
          cargo bench --bench physics_benchmarks -- --save-baseline ci
      
      - name: Compare with baseline
        run: |
          cargo bench --bench math_benchmarks -- --baseline ci
          cargo bench --bench ecs_benchmarks -- --baseline ci
          cargo bench --bench render_benchmarks -- --baseline ci
          cargo bench --bench physics_benchmarks -- --baseline ci
      
      - name: Upload benchmark results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: target/criterion/
```

### 查看HTML报告

Criterion会自动生成HTML报告：

```bash
# 运行基准测试并生成报告
cargo bench --bench math_benchmarks

# 报告位置
# target/criterion/{benchmark_name}/{function_name}/report/index.html

# 在浏览器中打开报告
open target/criterion/vec3_operations/add/report/index.html
```

报告包含：
- 性能统计图表
- 性能变化趋势
- 与基线的对比
- 详细的统计信息

### 性能阈值

设置性能阈值，超过阈值时失败：

```rust
use criterion::{black_box, Criterion};

fn bench_with_threshold(c: &mut Criterion) {
    c.bench_function("critical_operation", |b| {
        b.iter(|| {
            // 操作
        });
    })
    .sample_size(1000)
    .warm_up_time(std::time::Duration::from_millis(100))
    .measurement_time(std::time::Duration::from_secs(2));
}
```

---

## 最佳实践

### 1. 测试环境

- **稳定环境**: 关闭其他程序，确保CPU和内存可用
- **固定配置**: 使用相同的硬件和软件配置
- **多次运行**: 运行多次取平均值，减少测量误差

### 2. 测试数据

- **代表性数据**: 使用真实场景的数据规模和分布
- **边界情况**: 测试小数据和大数据的情况
- **随机数据**: 使用proptest生成随机测试数据

### 3. 结果分析

- **关注趋势**: 关注性能趋势而非单次结果
- **统计显著性**: 使用p值判断变化是否显著
- **上下文**: 结合代码变更分析性能变化原因

### 4. 性能优化

- **先测量**: 使用基准测试识别瓶颈
- **再优化**: 针对瓶颈进行优化
- **验证效果**: 优化后重新运行基准测试验证

### 5. 文档记录

- **记录基线**: 记录重要版本的性能基线
- **记录变更**: 记录性能优化的原因和效果
- **分享结果**: 在PR中分享性能测试结果

---

## 性能目标总结

| 系统 | 操作 | 目标性能 |
|------|------|----------|
| 数学 | 向量运算 | < 1ns/操作 |
| 数学 | 矩阵乘法 | < 10ns/操作 |
| ECS | 实体创建 | < 100ns/实体 |
| ECS | 组件查询 | < 10ns/查询 |
| 渲染 | 批次构建 | < 1ms（1000对象） |
| 渲染 | LOD选择 | < 100ns/对象 |
| 物理 | 物理步进 | < 16ms（60fps） |
| 物理 | 碰撞检测 | < 1μs/碰撞对 |

---

## 故障排除

### 问题：基准测试结果不稳定

**原因**: 系统负载、CPU频率变化、缓存影响

**解决**:
- 关闭其他程序
- 固定CPU频率（如果可能）
- 增加warm-up时间
- 增加样本数量

### 问题：性能回归误报

**原因**: 测量误差、系统负载变化

**解决**:
- 增加样本数量
- 多次运行取平均值
- 检查统计显著性（p值）

### 问题：基准测试运行缓慢

**原因**: 样本数量过多、测试数据过大

**解决**:
- 减少样本数量（使用`sample_size()`）
- 减少测试数据规模
- 使用`--quick`模式（Criterion的快速模式）

---

## 添加新的基准测试

### 创建基准测试文件

1. 在 `benches/` 目录创建新文件，例如 `benches/my_benchmarks.rs`
2. 添加基准测试函数
3. 在 `Cargo.toml` 中注册基准测试

### 示例：创建新的基准测试

```rust
// benches/my_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_operation(c: &mut Criterion) {
    c.bench_function("my_operation", |b| {
        b.iter(|| {
            // 要测试的操作
            black_box(my_function());
        });
    });
}

criterion_group!(benches, bench_my_operation);
criterion_main!(benches);
```

在 `Cargo.toml` 中添加：

```toml
[[bench]]
name = "my_benchmarks"
harness = false
```

### 基准测试最佳实践

1. **使用 `black_box`**: 防止编译器优化掉测试代码
2. **设置合理的样本大小**: 使用 `sample_size()` 控制测试时间
3. **预热时间**: 使用 `warm_up_time()` 让JIT编译器优化代码
4. **测量时间**: 使用 `measurement_time()` 控制测量时长
5. **参数化测试**: 使用 `bench_with_input()` 测试不同输入规模

### 示例：参数化基准测试

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn bench_scalable_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("scalable_operation");
    
    for size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            size,
            |b, &size| {
                let data = create_test_data(size);
                b.iter(|| {
                    black_box(process_data(&data));
                });
            },
        );
    }
    
    group.finish();
}
```

---

## 参考资源

- [Criterion.rs文档](https://docs.rs/criterion/)
- [Rust性能书](https://nnethercote.github.io/perf-book/)
- [性能分析工具](https://github.com/rust-lang/rustc-perf)

---

## 更新日志

- 2025-12-02: 初始版本

