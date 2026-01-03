# 快速测试参考

## 运行测试

```bash
# 所有测试
cargo test --workspace

# 特定模块
cargo test render
cargo test physics
cargo test entity

# 带输出
cargo test -- --nocapture

# 并行测试
cargo test --workspace --test-threads 4
```

## 覆盖率

```bash
# 安装工具
cargo install cargo-tarpaulin

# 生成报告
cargo tarpaulin --workspace --out Html --output-dir target/coverage

# 查看报告
open target/coverage/html/index.html  # macOS
```

## 基准测试

```bash
# 所有基准
cargo bench

# 特定基准
cargo bench --bench render_benchmark
cargo bench --bench physics_benchmark
cargo bench --bench ecs_benchmark
```

## 测试脚本

```bash
# 完整测试流程
./scripts/run_tests.sh

# 带覆盖率
./scripts/run_tests.sh --coverage

# 详细输出
./scripts/run_tests.sh --verbose
```

## 测试数量

- 单元测试: 111个
- 基准测试: 6个套件
- 测试文件: 20个
- 基准文件: 14个

## 覆盖率目标

- 当前: ~42%
- 目标: 50%
- 进度: 84%
