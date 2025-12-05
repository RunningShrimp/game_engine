# CI/CD 代码质量监控设置指南

**创建日期**: 2025-12-01  
**目标**: 建立持续集成的代码质量监控

---

## 概述

本项目已配置 GitHub Actions 工作流，用于自动化代码质量检查、测试和文档验证。

---

## 工作流配置

### 1. `.github/workflows/ci.yml`

**主要CI工作流**，包含以下检查：

- ✅ **代码格式检查** (`fmt`): 使用 `cargo fmt` 检查代码格式
- ✅ **Clippy检查** (`clippy`): 使用 `cargo clippy` 检查代码质量
- ✅ **编译检查** (`build`): 在多个平台上编译（Linux, Windows, macOS）
- ✅ **单元测试** (`test`): 运行所有单元测试
- ✅ **文档检查** (`docs`): 检查文档生成
- ✅ **测试覆盖率** (`coverage`): 使用 `cargo-tarpaulin` 生成覆盖率报告

**触发条件**:
- Push 到 `main` 或 `develop` 分支
- Pull Request 到 `main` 或 `develop` 分支

---

### 2. `.github/workflows/code-quality.yml`

**代码质量专项检查**，包含：

- ✅ **Clippy警告阈值**: 超过100个警告时失败
- ✅ **文档覆盖率检查**: 超过50个缺失文档警告时警告
- ✅ **测试覆盖率**: 生成覆盖率报告并上传到 Codecov
- ✅ **质量报告**: 在 GitHub Actions 中生成质量报告

**触发条件**:
- Push 到 `main` 或 `develop` 分支
- Pull Request 到 `main` 或 `develop` 分支
- 每天 UTC 2:00 自动运行（定时检查）

---

## 质量阈值

### Clippy 警告阈值

- **阈值**: 100 个警告
- **行为**: 超过阈值时 CI 失败
- **调整**: 修改 `.github/workflows/code-quality.yml` 中的 `CLIPPY_WARN_THRESHOLD`

### 文档覆盖率阈值

- **阈值**: 50 个缺失文档警告
- **行为**: 超过阈值时显示警告（不阻止构建）
- **调整**: 修改 `.github/workflows/code-quality.yml` 中的 `DOC_WARN_THRESHOLD`

### 测试覆盖率阈值

- **目标**: 80%+ 覆盖率
- **当前**: 覆盖率报告生成，但未设置硬性阈值
- **调整**: 可以在 `code-quality.yml` 中添加覆盖率百分比检查

---

## 本地使用

### 运行代码质量检查脚本

```bash
# 运行所有质量检查
./scripts/check_code_quality.sh
```

脚本会检查：
1. 代码格式
2. Clippy 警告
3. 文档覆盖率
4. 测试覆盖率（如果安装了 tarpaulin）

### 手动运行检查

```bash
# 格式检查
cargo fmt --all -- --check

# Clippy 检查
cargo clippy --all-targets --all-features -- -D warnings

# 文档检查
cargo doc --no-deps --all-features --document-private-items

# 测试覆盖率（需要先安装 tarpaulin）
cargo install cargo-tarpaulin
cargo tarpaulin --out Xml --output-dir coverage/ --all-features
```

---

## 依赖安装

### Linux (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y libasound2-dev libudev-dev
```

### macOS

```bash
# 通常不需要额外依赖
```

### Windows

```bash
# 通常不需要额外依赖
```

---

## Codecov 集成

测试覆盖率报告会自动上传到 Codecov（如果配置了 Codecov token）。

**设置 Codecov**:
1. 访问 https://codecov.io
2. 连接 GitHub 仓库
3. 获取 token（可选，GitHub Actions 可以自动检测）

---

## 调整阈值

### 修改 Clippy 阈值

编辑 `.github/workflows/code-quality.yml`:

```yaml
if [ "$WARNINGS" -gt 100 ]; then  # 修改这里的数字
```

### 修改文档阈值

编辑 `.github/workflows/code-quality.yml`:

```yaml
if [ "$MISSING_DOCS" -gt 50 ]; then  # 修改这里的数字
```

### 修改覆盖率阈值

编辑 `scripts/check_code_quality.sh`:

```bash
COVERAGE_THRESHOLD=80  # 修改这里的百分比
```

---

## 故障排除

### Clippy 检查失败

1. 运行 `cargo clippy --all-targets --all-features` 查看具体警告
2. 修复警告或添加 `#[allow(clippy::warning_name)]` 注释
3. 如果警告过多，可以临时提高阈值

### 文档检查失败

1. 运行 `cargo doc --no-deps --all-features` 查看缺失文档
2. 为公共 API 添加文档注释
3. 如果缺失文档过多，可以临时提高阈值

### 覆盖率报告未生成

1. 确保安装了 `cargo-tarpaulin`: `cargo install cargo-tarpaulin`
2. 检查测试是否正常运行: `cargo test`
3. 查看 `coverage/` 目录是否生成

---

## 最佳实践

1. **提交前运行检查**: 在提交前运行 `./scripts/check_code_quality.sh`
2. **逐步提高阈值**: 不要一次性设置过高的阈值，逐步提高
3. **关注警告**: 即使不阻止构建，也要关注警告并逐步修复
4. **定期审查**: 定期审查质量报告，识别需要改进的地方

---

### 3. `.github/workflows/benchmarks.yml`

**性能基准测试工作流**，包含：

- ✅ **数学运算基准测试**: 向量、矩阵运算性能
- ✅ **ECS系统基准测试**: 实体创建、组件添加、系统执行性能
- ✅ **物理系统基准测试**: 物理世界更新、碰撞检测性能
- ✅ **渲染系统基准测试**: 视锥剔除、LOD计算、批渲染性能
- ✅ **寻路系统基准测试**: A*寻路算法和并行寻路性能
- ✅ **性能回归检测**: 自动检测性能下降并报告

**触发条件**:
- Push 到 `main` 或 `develop` 分支
- Pull Request 到 `main` 或 `develop` 分支
- 每天 UTC 2:00 自动运行（定时检查）

**性能阈值**:
- **回归阈值**: 200% (性能下降超过2倍时失败)
- **行为**: 超过阈值时在PR中评论并标记为失败

---

## 性能基准测试

### 运行基准测试

```bash
# 运行所有基准测试
cargo bench

# 运行特定基准测试
cargo bench --bench math_benchmarks
cargo bench --bench ecs_benchmarks
cargo bench --bench physics_benchmarks
cargo bench --bench render_benchmarks
cargo bench --bench pathfinding_benchmarks
```

### 查看基准测试结果

基准测试结果会保存在 `target/criterion/` 目录中，可以使用浏览器打开HTML报告：

```bash
# 生成并打开基准测试报告
cargo bench --bench math_benchmarks -- --save-baseline main
open target/criterion/math_benchmarks/report/index.html
```

### 性能回归检测

基准测试使用 `criterion` 库，支持：

- **基线比较**: 与之前的基准测试结果比较
- **统计显著性**: 自动检测性能变化是否显著
- **HTML报告**: 生成详细的性能报告

**设置基线**:

```bash
# 设置基线（在性能优化前）
cargo bench --bench math_benchmarks -- --save-baseline before_optimization

# 运行基准测试并与基线比较
cargo bench --bench math_benchmarks -- --baseline before_optimization
```

---

## 下一步

- [x] 集成性能基准测试到 CI
- [ ] 配置 Codecov token（可选）
- [ ] 根据实际情况调整阈值
- [ ] 添加更多质量检查（如安全扫描）

---

**文档状态**: 完成  
**最后更新**: 2025-12-01

