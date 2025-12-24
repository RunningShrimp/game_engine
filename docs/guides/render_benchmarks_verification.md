# 渲染基准测试验证指南

## 概述

本文档说明如何验证渲染系统基准测试的正确性和性能。

## 验证步骤

### 1. 编译验证

首先验证基准测试能够编译：

```bash
# 检查编译
cargo check --bench render_benchmarks

# 如果成功，应该看到类似输出：
# Checking game_engine v0.1.0 (...)
# Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### 2. 运行基准测试

#### 完整基准测试套件

```bash
# 运行所有渲染基准测试
cargo bench --bench render_benchmarks

# 运行特定基准测试
cargo bench --bench render_benchmarks -- frustum_culling
cargo bench --bench render_benchmarks -- gpu_indirect_draw
cargo bench --bench render_benchmarks -- gpu_culling
```

#### 快速验证（仅编译检查）

```bash
# 使用验证脚本
./scripts/verify_benchmarks.sh
```

### 3. 基准测试内容

#### 视锥剔除测试 (`bench_frustum_culling`)

测试不同对象数量下的视锥剔除性能：
- 100 个对象
- 1,000 个对象
- 10,000 个对象

**预期性能**：
- 100 对象: < 0.1ms
- 1,000 对象: < 1ms
- 10,000 对象: < 10ms

#### LOD 计算测试 (`bench_lod_calculation`)

测试不同对象数量下的 LOD 计算性能：
- 100 个对象
- 1,000 个对象
- 10,000 个对象

**预期性能**：
- 100 对象: < 0.1ms
- 1,000 对象: < 1ms
- 10,000 对象: < 10ms

#### 批处理分组测试 (`bench_batch_grouping`)

测试不同对象数量下的批处理分组性能：
- 100 个对象
- 1,000 个对象
- 10,000 个对象

**预期性能**：
- 100 对象: < 0.5ms
- 1,000 对象: < 5ms
- 10,000 对象: < 50ms

#### GPU 间接绘制测试 (`bench_gpu_indirect_draw`)

**注意**: 此测试需要实际的 GPU 设备。在 CI 环境中可能会跳过。

测试不同实例数量下的 GPU 间接绘制性能：
- 1,000 个实例
- 10,000 个实例
- 50,000 个实例

**预期性能**：
- 1,000 实例: < 2ms
- 10,000 实例: < 10ms
- 50,000 实例: < 50ms

#### GPU 剔除测试 (`bench_gpu_culling`)

**注意**: 此测试需要实际的 GPU 设备。在 CI 环境中可能会跳过。

测试不同实例数量下的 GPU 剔除性能：
- 1,000 个实例
- 10,000 个实例
- 50,000 个实例

**预期性能**：
- 1,000 实例: < 1ms
- 10,000 实例: < 5ms
- 50,000 实例: < 25ms

## 环境要求

### 必需依赖

- Rust 工具链（最新稳定版）
- Criterion（基准测试框架，通过 Cargo 自动安装）
- wgpu（图形 API，通过 Cargo 自动安装）

### 可选依赖

- GPU 设备（用于 GPU 相关基准测试）
  - 如果无 GPU，相关测试会自动跳过

### CI/CD 环境

在 CI/CD 环境中：
- GPU 测试可能会被跳过（这是正常的）
- 只验证 CPU 相关的基准测试（视锥剔除、LOD 计算、批处理分组）

## 性能基准线

基准测试的预期性能值存储在 `performance_baselines.json` 中：

```json
{
  "render_benchmarks": {
    "description": "渲染管线性能基准测试",
    "baseline": {
      "draw_call_batch": "1.5 ms/frame",
      "shader_compilation": "25.6 ms/shader",
      "texture_upload": "3.2 ms/texture"
    },
    "threshold": 1.1
  }
}
```

## 性能回归检测

### 自动检测

使用性能回归检测脚本：

```bash
# 运行性能回归检测
./scripts/performance_regression.sh

# 或使用 CI/CD 集成
# GitHub Actions 会自动运行性能回归检测
```

### 手动对比

1. 运行基准测试并保存结果：
   ```bash
   cargo bench --bench render_benchmarks -- --output-format json > current_results.json
   ```

2. 与基线对比：
   ```bash
   # 使用性能回归检测工具
   cargo run --bin performance_regression_check -- \
     --baseline performance_baselines.json \
     --output regression_report.json
   ```

## 故障排除

### 问题：编译失败

**可能原因**：
- wgpu API 版本不匹配
- 缺少依赖

**解决方案**：
```bash
# 更新依赖
cargo update

# 检查 wgpu 版本
cargo tree | grep wgpu
```

### 问题：GPU 测试被跳过

**可能原因**：
- 无 GPU 设备（CI 环境）
- GPU 驱动问题

**解决方案**：
- 这是正常的，GPU 测试在无 GPU 环境中会自动跳过
- 如果需要测试 GPU 功能，请在本地有 GPU 的环境中运行

### 问题：性能结果异常

**可能原因**：
- 系统负载高
- 其他进程占用资源
- 硬件配置不同

**解决方案**：
- 关闭其他应用程序
- 多次运行取平均值
- 在相同硬件配置下对比

## 更新基准线

如果性能有显著改进，需要更新基准线：

```bash
# 1. 运行基准测试
cargo bench --bench render_benchmarks

# 2. 分析结果，确定新的基准值

# 3. 更新 performance_baselines.json
# 编辑文件，更新 render_benchmarks 部分的 baseline 值

# 4. 提交更改
git add performance_baselines.json
git commit -m "Update render benchmarks baseline"
```

## 相关文档

- [性能基准测试 README](../../game_engine/benches/README.md)
- [性能回归检测指南](./performance_regression_guide.md)
- [wgpu API 文档](https://docs.rs/wgpu/)

