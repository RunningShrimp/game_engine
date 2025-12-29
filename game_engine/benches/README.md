# 游戏引擎性能基准测试

本目录包含游戏引擎各个子系统的性能基准测试，用于持续监控和优化性能。

## 基准测试覆盖范围

### ✅ 已实现基准测试

1. **数学运算基准测试** (`math_benchmarks.rs`)
   - 向量运算（加法、点积、叉积）
   - SIMD加速版本性能对比
   - 矩阵变换操作

2. **ECS系统基准测试** (`ecs_benchmarks.rs`)
   - 实体创建和销毁
   - 组件查询和迭代
   - 系统调度性能

3. **物理系统基准测试** (`physics_benchmarks.rs`)
   - 物理世界创建
   - 刚体动力学模拟
   - 碰撞检测性能

4. **网络系统基准测试** (`network_benchmarks.rs`)
   - 消息序列化/反序列化
   - 网络消息处理
   - 连接管理性能

5. **资源管理系统基准测试** (`resource_benchmarks.rs`)
   - 资源缓存操作
   - 哈希表性能
   - 内存管理效率

### ✅ 新增基准测试

6. **渲染系统基准测试** (`render_benchmarks.rs`)
   - **状态**: ✅ 已完全重写和增强
   - **内容**:
     - 视锥剔除 (100-10000 对象)
     - 变换计算性能
     - 渲染排序优化
     - 批处理性能
     - MVP矩阵计算
     - 骨骼动画计算

7. **物理系统基准测试** (`physics_benchmarks.rs`)
   - **状态**: ✅ 已完全重写，使用Rapier3D API
   - **内容**:
     - 物理步进 (10-500 刚体)
     - 碰撞检测性能
     - 空间查询 (射线投射)
     - 刚体创建性能
     - 物理ECS集成
     - 连续碰撞检测 (CCD)

8. **序列化基准测试** (`serialization_benchmarks.rs`)
   - **状态**: ✅ 新增
   - **内容**:
     - 网络消息序列化/反序列化
     - 场景保存/加载
     - JSON vs Bincode对比
     - 存档系统性能
     - 压缩性能 (Gzip/Deflate)

9. **内存基准测试** (`memory_benchmarks.rs`)
   - **状态**: ✅ 新增
   - **内容**:
     - 实体/组件内存分配
     - 实体池重用
     - 组件布局效率 (小组件 vs 大组件)
     - 查询内存访问模式
     - 批量操作内存效率
     - 资源内存使用
     - 内存碎片分析

## 使用方法

### 快速开始

```bash
# 运行所有基准测试
cargo bench --workspace

# 查看HTML报告
open game_engine/benches/results/report/index.html

# 保存性能基线
cargo bench --workspace -- --save-baseline main

# 与基线对比
cargo bench --workspace -- --baseline main
```

### 运行特定基准测试

```bash
# ECS性能测试
cargo bench --bench ecs_benchmarks

# 物理性能测试
cargo bench --bench physics_benchmarks

# 渲染性能测试
cargo bench --bench render_benchmarks

# 序列化性能测试
cargo bench --bench serialization_benchmarks

# 内存性能测试
cargo bench --bench memory_benchmarks

# 数学运算测试
cargo bench --bench math_benchmarks

# 网络性能测试
cargo bench --bench network_benchmarks

# 资源管理测试
cargo bench --bench resource_benchmarks

# 路径查找测试
cargo bench --bench pathfinding_benchmarks
```

### 使用脚本

```bash
# 运行完整基准测试套件
./scripts/run_benchmarks.sh

# 运行性能回归检测
./scripts/performance_regression.sh

# 查看性能对比结果
cat target/performance_results.json
```

## 基准测试配置

### Criterion配置
- 默认采样次数: 100次迭代
- 预热时间: 自动
- 测量单位: 纳秒/操作

### 性能阈值
- 警告阈值: 10%性能下降
- 严重阈值: 20%性能下降

## 持续集成

基准测试集成到CI/CD流程中：

1. **编译验证**: 确保所有基准测试能够编译
2. **性能回归检测**: 自动检测性能下降
3. **结果归档**: 保存历史性能数据

## 基准测试开发指南

### 添加新的基准测试

1. 在`benches/`目录下创建新的基准测试文件
2. 使用`criterion`框架编写测试函数
3. 在`criterion_group!`宏中注册新的测试函数
4. 更新`scripts/verify_benchmarks.sh`验证脚本

### 示例代码结构

```rust
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

fn bench_my_feature(c: &mut Criterion) {
    let mut group = c.benchmark_group("my_feature");

    for input_size in [100, 1000, 10000].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(input_size),
            input_size,
            |b, &size| {
                b.iter(|| {
                    // 执行被测操作
                    black_box(my_operation(size))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_my_feature);
criterion_main!(benches);
```

## 性能优化建议

基于基准测试结果的常见优化方向：

1. **减少内存分配**: 使用对象池和预分配
2. **SIMD优化**: 利用SIMD指令加速向量运算
3. **缓存友好**: 优化数据布局和访问模式
4. **异步处理**: 将阻塞操作移到后台线程
5. **批处理**: 减少系统调用和状态切换

## 故障排除

### 基准测试编译失败
- 检查依赖版本兼容性
- 确认API接口没有发生破坏性变更
- 更新到最新的依赖版本

### 性能结果不稳定
- 增加采样次数
- 运行在稳定的系统负载下
- 使用专用的测试机器

### 内存不足
- 减少测试数据集大小
- 使用更小的迭代次数
- 优化测试代码的内存使用
