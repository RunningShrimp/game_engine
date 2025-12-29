# Benchmark CI集成与性能监控指南

本文档说明如何使用benchmark CI集成系统和性能监控面板。

## 目录

- [概述](#概述)
- [CI/CD集成](#cicd集成)
- [性能监控面板](#性能监控面板)
- [本地监控工具](#本地监控工具)
- [脚本使用指南](#脚本使用指南)
- [故障排除](#故障排除)

## 概述

benchmark监控系统提供以下功能:

1. **自动CI集成** - 在每次push和PR时自动运行benchmark
2. **性能回归检测** - 自动检测超过阈值的性能退化
3. **趋势分析** - 跟踪性能随时间的变化
4. **交互式仪表板** - 可视化性能指标和趋势图
5. **本地监控** - 在开发时持续监控性能

## CI/CD集成

### 工作流概览

`.github/workflows/benchmark.yml` 定义了以下jobs:

#### 1. Benchmark Job

运行所有benchmark并生成报告:

```yaml
- 运行workspace中所有benchmark
- 生成性能报告
- 导出JSON/CSV数据
- 上传artifacts
- 在PR中评论结果
```

**触发条件:**
- Push to main/master
- Pull Request
- Manual workflow dispatch

#### 2. Performance Regression Check Job

在PR中检测性能回归:

```yaml
- 对比PR分支与main分支
- 运行regression检测脚本
- 失败如果检测到显著回归 (>10%)
```

**阈值配置:**
- 回归阈值: 10%
- 改进阈值: 5%
- 统计显著性: 2-sigma

#### 3. Performance Trends Job

在main分支更新时生成趋势数据:

```yaml
- 加载历史数据
- 生成趋势图表
- 部署到GitHub Pages
```

**输出:**
- JSON历史数据
- PNG趋势图表
- HTML仪表板

### CI Artifacts

每次运行生成以下artifacts:

| Artifact | 内容 | 保留期 |
|----------|------|--------|
| benchmark-results | 原始benchmark输出和Criterion数据 | 30天 |
| benchmark-report | Markdown性能报告 | 90天 |
| benchmark-data | JSON/CSV导出数据 | 90天 |

## 性能监控面板

### 访问仪表板

仪表板部署在GitHub Pages (需要配置):

```
https://<username>.github.io/<repository>/benches/trends/
```

### 本地预览

要本地预览仪表板:

```bash
# 1. 生成benchmark数据
cargo bench --workspace

# 2. 导出数据
python3 scripts/export_benchmark_data.py

# 3. 启动HTTP服务器
cd game_engine/benches/trends
python3 -m http.server 8000

# 4. 访问浏览器
open http://localhost:8000
```

### 仪表板功能

#### 指标卡片

显示每个benchmark的:
- 当前性能 (ns)
- 相对baseline的变化百分比
- 状态指示器:
  - 🟢 改进 (>5% 提升)
  - 🔴 回归 (>10% 退化)
  - 🟡 稳定 (±5% 内)

#### 趋势图表

每个benchmark的历史性能曲线:
- X轴: 日期
- Y轴: 执行时间 (ns)
- 蓝色线: 平均值
- 阴影区域: 标准差

#### 摘要统计

显示总体概况:
- 总benchmark数量
- 改进的benchmark数
- 回归的benchmark数
- 稳定的benchmark数

## 本地监控工具

### 持续监控脚本

使用`watch_benchmarks.sh`持续监控性能:

```bash
# 基本用法 (默认5分钟间隔)
./scripts/watch_benchmarks.sh

# 自定义间隔 (2分钟)
BENCHMARK_INTERVAL=120 ./scripts/watch_benchmarks.sh

# 输出到文件
./scripts/watch_benchmarks.sh 2>&1 | tee monitoring.log
```

**功能:**
- 自动初始化baseline (如果需要)
- 周期性运行benchmark
- 实时检测回归
- 生成报告
- 倒计时显示下次运行时间

**终止:** 按 Ctrl+C

### 单次回归检测

快速检测当前代码的回归:

```bash
# 运行benchmark
cargo bench --workspace -- --save-baseline temp

# 检测回归
python3 scripts/detect_regression.py \
  --baseline target/criterion/main \
  --current target/criterion/temp \
  --threshold 10.0 \
  --verbose
```

## 脚本使用指南

### generate_benchmark_report.py

生成Markdown格式的性能报告。

**用法:**

```bash
# 基本用法 (自动查找数据)
python3 scripts/generate_benchmark_report.py

# 报告保存到 benchmark_report.md
```

**输出:**
- 终端: 彩色报告
- 文件: `benchmark_report.md`

**报告内容:**
1. 执行摘要 (改进/回归/稳定数量)
2. 详细结果表格
3. 回归详情 (如果有)
4. 改进详情 (如果有)

**退出码:**
- 0: 无回归
- 1: 检测到回归

### detect_regression.py

检测性能回归的专用工具。

**用法:**

```bash
# 基本用法
python3 scripts/detect_regression.py

# 自定义路径
python3 scripts/detect_regression.py \
  --baseline path/to/baseline \
  --current path/to/current

# 自定义阈值
python3 scripts/detect_regression.py \
  --threshold 15.0 \
  --improvement -10.0

# 显示稳定的benchmark
python3 scripts/detect_regression.py --verbose

# CI模式 (始终返回0)
python3 scripts/detect_regression.py --exit-zero
```

**参数:**

| 参数 | 说明 | 默认值 |
|------|------|--------|
| --baseline, -b | Baseline目录 | target/criterion/main |
| --current, -c | 当前结果目录 | target/criterion |
| --threshold, -t | 回归阈值 (%) | 10.0 |
| --improvement, -i | 改进阈值 (%) | -5.0 |
| --verbose, -v | 显示稳定benchmark | false |
| --exit-zero | 始终返回退出码0 | false |

**输出示例:**

```
Performance Regression Analysis
================================================================================
Total benchmarks analyzed: 42
  - Regressed: 2
  - Improved: 5
  - Stable: 35
================================================================================

❌ PERFORMANCE REGRESSIONS DETECTED
================================================================================

  ❌ ecs_create_entity_1000
     Baseline: 1234.56 ± 45.67 ns
     Current:  1456.78 ± 56.78 ns
     Change:   +18.03% (+222.22 ns)
     Statistically significant

✅ PASS: No regressions detected, 5 benchmark(s) improved
```

### export_benchmark_data.py

导出benchmark数据为JSON/CSV格式。

**用法:**

```bash
# 导出JSON (供仪表板使用)
python3 scripts/export_benchmark_data.py

# 导出JSON + CSV
python3 scripts/export_benchmark_data.py --csv

# 自定义输出路径
python3 scripts/export_benchmark_data.py \
  --output custom/path/data.json

# 漂亮打印到stdout
python3 scripts/export_benchmark_data.py --pretty
```

**输出格式:**

**JSON:**
```json
{
  "metadata": {
    "timestamp": "2025-12-28T22:30:00",
    "count": 42,
    "version": "1.0"
  },
  "benchmarks": {
    "ecs_create_entity_1000": {
      "mean": 1234.56,
      "stddev": 45.67,
      "median": 1220.00,
      "min": 1100.00,
      "max": 1500.00,
      "baseline": 1200.00,
      "baseline_stddev": 40.00,
      "unit": "ns"
    }
  }
}
```

### update_trend_charts.py

生成性能趋势图表。

**用法:**

```bash
# 从benchmark_data.json生成图表
python3 scripts/update_trend_charts.py

# 指定数据文件
python3 scripts/update_trend_charts.py \
  --input path/to/data.json

# 自定义目录
python3 scripts/update_trend_charts.py \
  --trends-dir game_engine/benches/trends \
  --output-dir game_engine/benches/trends

# 不保存到历史
python3 scripts/update_trend_charts.py --no-save
```

**要求:**
```bash
pip install matplotlib
```

**输出:**
- 趋势图表: `{benchmark_name}_trend.png`
- 摘要报告: `trend_summary.md`

### watch_benchmarks.sh

持续监控benchmark性能。

**用法:**

```bash
# 基本用法
./scripts/watch_benchmarks.sh

# 自定义监控间隔 (秒)
BENCHMARK_INTERVAL=300 ./scripts/watch_benchmarks.sh

# 后台运行
nohup ./scripts/watch_benchmarks.sh > monitoring.log 2>&1 &
```

**环境变量:**

| 变量 | 说明 | 默认值 |
|------|------|--------|
| BENCHMARK_INTERVAL | 监控间隔(秒) | 300 (5分钟) |

**输出位置:**
```
benches/monitoring/<timestamp>/
├── run_1.txt
├── run_2.txt
├── report_1.md
└── report_2.md
```

## 故障排除

### 问题: CI中的benchmark失败

**可能原因:**
1. 依赖未安装
2. 编译错误
3. 超时

**解决方案:**

```yaml
# 增加超时时间
timeout-minutes: 90

# 检查依赖
- name: Install dependencies
  run: |
    sudo apt-get update
    sudo apt-get install -y libssl-dev pkg-config
```

### 问题: 趋势图表未生成

**可能原因:**
matplotlib未安装

**解决方案:**

```bash
# CI中添加
- name: Install Python dependencies
  run: pip3 install matplotlib

# 本地
pip3 install matplotlib
```

### 问题: 仪表板显示"Loading data..."

**可能原因:**
1. benchmark_data.json不存在
2. 路径配置错误
3. CORS问题

**解决方案:**

```bash
# 1. 生成数据
cargo bench --workspace
python3 scripts/export_benchmark_data.py

# 2. 检查路径
ls -la game_engine/benches/trends/benchmark_data.json

# 3. 使用HTTP服务器
python3 -m http.server 8000
```

### 问题: 回归检测误报

**可能原因:**
1. 阈值过低
2. 统计波动
3. 系统负载

**解决方案:**

```bash
# 调整阈值
python3 scripts/detect_regression.py \
  --threshold 15.0 \      # 提高到15%
  --improvement -10.0     # 改进也需要10%

# 多次运行取平均
for i in {1..5}; do
  cargo bench --workspace
done

# 使用更稳定的环境
# - 关闭其他应用
# - 使用性能模式
# - 固定CPU频率
```

### 问题: Baseline数据丢失

**可能原因:**
Git缓存清理

**解决方案:**

```bash
# 重新创建baseline
cargo bench --workspace -- --save-baseline main

# 或从历史恢复
git checkout HEAD~1
cargo bench --workspace -- --save-baseline old
git checkout -

# 对比
cargo bench --workspace -- --baseline old
```

## 最佳实践

### 1. Benchmark命名

使用描述性名称:

```rust
// 好
fn bench_ecs_create_entity_1000(c: &mut Criterion) { ... }

// 不好
fn bench_test1(c: &mut Criterion) { ... }
```

### 2. 基准线管理

定期更新baseline:

```bash
# 在稳定版本上
git tag v1.0.0
cargo bench --workspace -- --save-baseline v1.0.0

# 对比当前版本
cargo bench --workspace -- --baseline v1.0.0
```

### 3. CI集成

在PR中强制检测回归:

```yaml
- name: Detect regressions
  run: python3 scripts/detect_regression.py
  # 失败阻止合并
```

### 4. 监控策略

- 开发时: 使用`watch_benchmarks.sh`
- CI/CD: 自动运行
- 发布前: 完整趋势分析
- 定期: 查看仪表板

### 5. 数据保留

- Raw数据: 30天 (通过artifacts)
- 报告: 90天
- 趋势: 永久 (通过GitHub Pages)

## 进阶配置

### 自定义阈值

在`.github/workflows/benchmark.yml`:

```yaml
- name: Detect regressions
  run: |
    python3 scripts/detect_regression.py \
      --threshold ${{ vars.BENCHMARK_THRESHOLD }} \
      --improvement ${{ vars.BENCHMARK_IMPROVEMENT }}
```

### 添加新benchmark

1. 在`game_engine/benches/`创建文件:

```rust
// game_engine/benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_feature(c: &mut Criterion) {
    c.bench_function("my_feature", |b| {
        b.iter(|| {
            // 你的代码
        });
    });
}

criterion_group!(benches, bench_my_feature);
criterion_main!(benches);
```

2. 在`game_engine/Cargo.toml`添加:

```toml
[[bench]]
name = "my_benchmark"
harness = false
```

3. 提交并查看CI结果

## 相关文档

- [Criterion.rs文档](https://bheisler.github.io/criterion.rs/book/)
- [Benchmark快速开始](./BENCHMARK_QUICKSTART.md)
- [Benchmark实现总结](../BENCHMARK_IMPLEMENTATION_SUMMARY.md)
- [GitHub Actions文档](https://docs.github.com/en/actions)

## 支持

如有问题:
1. 查看本文档的故障排除部分
2. 检查CI日志
3. 查看`.github/workflows/benchmark.yml`
4. 提交issue
