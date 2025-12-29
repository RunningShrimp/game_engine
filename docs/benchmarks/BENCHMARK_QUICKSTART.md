# 性能基准测试快速入门

## 5分钟快速开始

### 1. 运行所有基准测试

```bash
# 在项目根目录
cargo bench --workspace
```

这将:
- 编译所有benchmark
- 运行性能测试
- 生成HTML报告
- 保存结果到 `game_engine/benches/results/`

### 2. 查看结果

```bash
# 在macOS上
open game_engine/benches/results/report/index.html

# 在Linux上
xdg-open game_engine/benches/results/report/index.html

# 在Windows上
start game_engine/benches/results/report/index.html
```

### 3. 建立性能基线

```bash
# 保存当前性能作为baseline
cargo bench --workspace -- --save-baseline main
```

## 常用命令

### 运行特定benchmark

```bash
# 只测试ECS性能
cargo bench --bench ecs_benchmarks

# 只测试物理性能
cargo bench --bench physics_benchmarks

# 只测试渲染性能
cargo bench --bench render_benchmarks
```

### 性能对比

```bash
# 与baseline对比
cargo bench --bench ecs_benchmarks -- --baseline main

# 查看性能变化
cargo bench --workspace -- --baseline main
```

### 调试模式

```bash
# 运行特定benchmark（快速迭代）
cargo bench --bench ecs_benchmarks spawn_entities

# 查看详细输出
cargo bench --workspace -- --verbose
```

## 理解基准测试结果

### 命令行输出示例

```
spawn_entities/100
                        time:   [2.3456 ms 2.3789 ms 2.4123 ms]
                        change: [-2.3% -1.8% -1.2%] (p = 0.00 < 0.05)
                        Performance has improved.
```

**解读:**
- `time`: 平均运行时间（中位数）
- `change`: 与baseline对比的变化
- `Performance has improved`: 性能提升

### HTML报告

打开HTML报告后，你可以看到:
- **时间趋势图**: 性能随时间变化
- **对比图**: 不同实现的性能对比
- **详细统计**: 均值、中位数、标准差

## 基准测试文件结构

```
game_engine/benches/
├── ecs_benchmarks.rs              # ECS性能测试
├── physics_benchmarks.rs          # 物理性能测试
├── render_benchmarks.rs           # 渲染性能测试
├── serialization_benchmarks.rs    # 序列化性能测试
├── memory_benchmarks.rs           # 内存性能测试
├── math_benchmarks.rs             # 数学运算测试
├── network_benchmarks.rs          # 网络性能测试
├── resource_benchmarks.rs         # 资源管理测试
├── pathfinding_benchmarks.rs      # 路径查找测试
└── results/                       # 测试结果目录
    ├── report/
    │   └── index.html             # 主报告
    └── <benchmark-name>/
        └── new/
            └── estimates.json     # 原始数据
```

## 编写你的第一个benchmark

### 示例: 测试一个函数性能

```rust
// benches/my_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn my_function(n: usize) -> usize {
    (0..n).sum()
}

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // 使用black_box防止编译器优化掉代码
            black_box(my_function(1000))
        });
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

### 注册benchmark

在 `Cargo.toml` 中:

```toml
[[bench]]
name = "my_benchmark"
harness = false
```

### 运行

```bash
cargo bench --bench my_benchmark
```

## 性能回归检测

### 在CI中自动检测

项目已配置GitHub Actions自动运行benchmark:

1. **Push到main**: 运行并保存baseline
2. **创建PR**: 与baseline对比
3. **性能下降**: CI失败并通知

### 本地检测

```bash
# 修改代码后
git checkout -b my-feature
# ... 进行修改 ...
cargo bench --workspace -- --baseline main

# 如果性能下降超过5%，CI会失败
```

## 性能优化工作流

### 1. 建立baseline

```bash
git checkout main
cargo bench --workspace -- --save-baseline main
```

### 2. 优化代码

```bash
git checkout -b optimize-something
# ... 进行优化 ...
```

### 3. 验证改进

```bash
cargo bench --workspace -- --baseline main

# 查看HTML报告确认改进
open game_engine/benches/results/report/index.html
```

### 4. 提交PR

```bash
git add .
git commit -m "Optimize: Improve ECS query performance by 20%"
git push origin optimize-something
```

## 常见问题

### Q: Benchmark运行太慢怎么办?

**A**: 减少测试规模或运行特定benchmark:

```bash
# 只运行一个benchmark
cargo bench --bench ecs_benchmarks spawn_entities/1000

# 减少采样次数（快速但结果不太准确）
cargo bench --workspace -- --sample-size 10
```

### Q: 为什么结果不稳定?

**A**: 确保系统负载稳定:

```bash
# 关闭其他应用
# 确保没有后台更新
# 增加采样时间
cargo bench --workspace -- --measurement-time 10
```

### Q: 如何优化内存分配?

**A**: 使用内存benchmark:

```bash
cargo bench --bench memory_benchmarks

# 查看内存分配报告
open game_engine/benches/results/memory_benchmarks/report/index.html
```

### Q: Benchmark失败了怎么办?

**A**: 检查错误信息:

```bash
# 查看详细错误
cargo bench --workspace -- --nocapture

# 检查是否是API变更
cargo build --benches
```

## 性能目标参考

### 当前性能基准（参考值）

**ECS:**
- 实体创建: ~2-3 μs/entity
- 查询迭代: ~10-100 ns/entity
- 系统调度: ~1-10 μs/system

**Physics:**
- 物理步进: ~1-5 ms/100 bodies
- 碰撞检测: ~0.5-2 ms/100 pairs
- 射线投射: ~1-10 μs/raycast

**Render:**
- 视锥剔除: ~10-100 ns/object
- 变换计算: ~5-50 ns/transform
- 批处理: ~1-10 μs/batch

**Serialization:**
- 消息序列化: ~100-500 ns/message
- 场景保存: ~1-10 ms/scene
- 压缩: ~10-100 ms/MB

**Memory:**
- 实体分配: ~100-500 bytes/entity
- 组件分配: ~10-100 bytes/component
- 内存碎片: <10%

## 下一步

1. **阅读完整文档**: `docs/benchmark_infrastructure.md`
2. **探索现有benchmark**: 查看benches/目录下的代码
3. **编写自己的benchmark**: 为你的功能添加性能测试
4. **监控性能**: 定期运行benchmark并跟踪性能趋势

## 获取帮助

- **GitHub Issues**: 报告benchmark问题
- **文档**: `docs/benchmark_infrastructure.md`
- **示例**: `benches/*.rs`

## 贡献

欢迎贡献新的benchmark!

请确保:
- 使用criterion框架
- 遵循命名规范
- 测试多个规模
- 更新文档
