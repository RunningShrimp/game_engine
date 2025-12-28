# 性能回归检测CI集成指南

本文档说明如何在CI中集成性能回归检测功能。

## 概述

性能回归检测CI工作流会在每次推送和PR时自动运行，检测代码变更是否导致性能下降。

## 工作流配置

### 触发条件

性能回归检测在以下情况下触发：

- **Push事件**: 推送到 `main` 或 `develop` 分支
- **Pull Request**: 创建或更新PR到 `main` 或 `develop` 分支
- **手动触发**: 通过 `workflow_dispatch` 手动运行

### 工作流文件

配置文件位置：`.github/workflows/performance-regression.yml`

## 工作流程

### 1. 环境准备

- 安装Rust工具链（stable版本）
- 安装jq（用于JSON处理）
- 缓存Cargo依赖以加速构建

### 2. 构建项目

```yaml
- name: Build project
  run: cargo build --release --package game_engine
```

### 3. 运行性能回归检测

```yaml
- name: Run performance regression detection
  run: ./scripts/performance_regression.sh
```

脚本会：
1. 运行所有基准测试（math, ecs, physics, render）
2. 收集性能结果
3. 与基线进行比较
4. 检测性能回归

### 4. 结果处理

- **上传报告**: 性能结果和基线文件作为artifact上传
- **PR评论**: 如果检测到回归，自动在PR中添加评论
- **失败处理**: 如果检测到严重回归（>20%），工作流会失败

## 性能阈值

### 警告阈值

- **WARNING**: 性能下降 > 10%
- **CRITICAL**: 性能下降 > 20%

### 基准测试列表

- `math_benchmarks`: 数学运算性能
- `ecs_benchmarks`: ECS系统性能
- `physics_benchmarks`: 物理引擎性能
- `render_benchmarks`: 渲染系统性能

## 基线管理

### 创建初始基线

首次运行时，如果没有基线文件，脚本会自动创建：

```bash
# 基线文件位置
target/performance_baseline.json
```

### 更新基线

基线应该定期更新以反映性能改进：

```bash
# 手动更新基线
./scripts/performance_regression.sh
# 如果检测通过，基线会自动更新
```

### 基线文件格式

```json
{
  "timestamp": "2024-01-01T00:00:00Z",
  "benchmarks": {
    "math_benchmarks": 1234567,
    "ecs_benchmarks": 2345678,
    "physics_benchmarks": 3456789,
    "render_benchmarks": 4567890
  }
}
```

## 本地运行

### 运行性能回归检测

```bash
# 确保脚本可执行
chmod +x scripts/performance_regression.sh

# 运行检测
./scripts/performance_regression.sh
```

### 依赖工具

- **jq**: JSON处理工具
  - Ubuntu/Debian: `sudo apt-get install jq`
  - macOS: `brew install jq`
  - 或使用Node.js作为备选

### 查看结果

```bash
# 查看性能结果
cat target/performance_results.json

# 查看基线
cat target/performance_baseline.json
```

## PR集成

### 自动评论

当检测到性能回归时，工作流会自动在PR中添加评论，包含：

- 性能对比表格
- 回归百分比
- 严重程度（WARNING/CRITICAL）
- 建议修复提示

### 示例评论

```markdown
## ⚠️ Performance Regression Detected

### Performance Comparison

| Benchmark | Baseline | Current | Change | Status |
|-----------|----------|---------|--------|--------|
| ecs_benchmarks | 2345678.00ns | 2600000.00ns | +10.8% | ⚠️ WARNING |
| render_benchmarks | 4567890.00ns | 5500000.00ns | +20.4% | ❌ CRITICAL |

⚠️ **Please review and address these performance regressions before merging.**
```

## 故障排除

### 问题：基线文件不存在

**解决方案**: 首次运行会自动创建基线。确保脚本有写入权限。

### 问题：jq未安装

**解决方案**: CI工作流会自动安装jq。本地运行需要手动安装。

### 问题：基准测试失败

**解决方案**: 
1. 检查基准测试代码是否正确
2. 确保所有依赖已安装
3. 查看详细错误日志

### 问题：误报回归

**解决方案**:
1. 检查系统负载是否影响结果
2. 多次运行取平均值
3. 调整阈值（修改脚本中的`THRESHOLD_WARNING`和`THRESHOLD_CRITICAL`）

## 最佳实践

### 1. 定期更新基线

基线应该反映当前最佳性能状态，建议：
- 每次重大性能优化后更新
- 每月定期审查和更新
- 在发布新版本前更新

### 2. 关注关键指标

优先关注以下性能指标：
- ECS查询性能
- 渲染帧率
- 物理模拟性能
- 内存分配性能

### 3. 分析回归原因

检测到回归时：
1. 查看最近的代码变更
2. 分析性能profiling数据
3. 使用性能分析工具（如Tracy）定位瓶颈
4. 实施修复并验证

### 4. CI/CD集成

将性能回归检测集成到CI/CD流程：
- 作为PR检查的一部分
- 阻止有严重回归的PR合并
- 定期运行完整基准测试套件

## 相关文档

- [基准测试指南](./benchmarking_guide.md)
- [性能调优指南](./performance_tuning_guide.md)
- [CI/CD优化](./cicd_optimization.md)

