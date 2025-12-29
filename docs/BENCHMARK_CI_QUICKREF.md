# Benchmark CI集成 - 快速参考

## 快速开始

### 本地使用

```bash
# 1. 运行benchmark
cargo bench --workspace

# 2. 生成报告
python3 scripts/generate_benchmark_report.py

# 3. 检测回归
python3 scripts/detect_regression.py

# 4. 导出数据
python3 scripts/export_benchmark_data.py --csv

# 5. 查看仪表板
cd game_engine/benches/trends && python3 -m http.server 8000
```

### 持续监控

```bash
# 后台监控 (5分钟间隔)
./scripts/watch_benchmarks.sh

# 自定义间隔
BENCHMARK_INTERVAL=120 ./scripts/watch_benchmarks.sh
```

## CI工作流

### 触发条件

- Push to main/master
- Pull Request
- Manual dispatch

### Jobs

1. **benchmark** - 运行所有benchmark
2. **performance-regression-check** - PR回归检测
3. **trends** - 更新趋势图 (仅main)
4. **benchmark-summary** - 生成摘要

### Artifacts

| 名称 | 内容 | 保留期 |
|------|------|--------|
| benchmark-results | 原始输出 | 30天 |
| benchmark-report | Markdown报告 | 90天 |
| benchmark-data | JSON/CSV | 90天 |

## 脚本速查

### generate_benchmark_report.py

```bash
python3 scripts/generate_benchmark_report.py
# 输出: benchmark_report.md
```

### detect_regression.py

```bash
# 基本检测 (10%阈值)
python3 scripts/detect_regression.py

# 自定义阈值
python3 scripts/detect_regression.py --threshold 15.0

# 详细输出
python3 scripts/detect_regression.py --verbose

# CI模式 (不失败)
python3 scripts/detect_regression.py --exit-zero
```

### export_benchmark_data.py

```bash
# JSON (仪表板用)
python3 scripts/export_benchmark_data.py

# JSON + CSV
python3 scripts/export_benchmark_data.py --csv

# 自定义输出
python3 scripts/export_benchmark_data.py --output path/data.json
```

### update_trend_charts.py

```bash
# 生成趋势图
python3 scripts/update_trend_charts.py

# 指定输入
python3 scripts/update_trend_charts.py --input data.json

# 自定义目录
python3 scripts/update_trend_charts.py \
  --trends-dir game_engine/benches/trends
```

## 性能仪表板

### 访问

- GitHub Pages: `https://<user>.github.io/<repo>/benches/trends/`
- 本地: `http://localhost:8000`

### 功能

- 🟢 改进 (>5% 提升)
- 🔴 回归 (>10% 退化)
- 🟡 稳定 (±5% 内)

## 故障排除

### Benchmark失败

```bash
# 检查依赖
sudo apt-get install libssl-dev pkg-config

# 单独运行
cargo bench --bench ecs_benchmarks
```

### 无趋势图

```bash
# 安装matplotlib
pip3 install matplotlib
```

### 仪表板空白

```bash
# 生成数据
cargo bench --workspace
python3 scripts/export_benchmark_data.py

# 启动服务器
cd game_engine/benches/trends
python3 -m http.server 8000
```

## 文件结构

```
game_engine/
├── .github/workflows/
│   └── benchmark.yml          # CI配置
├── scripts/
│   ├── generate_benchmark_report.py
│   ├── detect_regression.py
│   ├── export_benchmark_data.py
│   ├── update_trend_charts.py
│   └── watch_benchmarks.sh
└── game_engine/benches/
    ├── trends/
    │   ├── index.html         # 仪表板
    │   └── benchmark_data.json
    ├── results/
    └── baseline/
```

## 相关文档

- [完整指南](./BENCHMARK_MONITORING_GUIDE.md)
- [Benchmark快速开始](./BENCHMARK_QUICKSTART.md)
- [实现总结](../BENCHMARK_IMPLEMENTATION_SUMMARY.md)
