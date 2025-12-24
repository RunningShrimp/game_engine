# 代码覆盖率测试指南

本指南说明如何运行代码覆盖率测试并生成覆盖率报告。

## 概述

代码覆盖率测试用于衡量测试套件对代码库的覆盖程度，帮助识别未测试的代码区域。

## 工具选择

游戏引擎支持两种覆盖率工具：

### 1. cargo-tarpaulin (推荐)

**优点**:
- 安装简单
- 支持多种输出格式（HTML、LCOV、JSON、XML）
- 快速执行
- 支持增量覆盖率

**安装**:
```bash
cargo install cargo-tarpaulin
```

### 2. grcov

**优点**:
- 基于LLVM，更准确
- 支持分支覆盖率
- 支持多种输出格式

**安装**:
```bash
cargo install grcov
```

## 快速开始

### 使用脚本（推荐）

```bash
# 运行覆盖率测试并生成报告
./scripts/run_coverage_report.sh
```

脚本会自动：
1. 检查工具是否安装
2. 运行测试套件
3. 生成覆盖率报告（HTML、LCOV、JSON）
4. 显示报告位置

### 手动运行

#### 使用 cargo-tarpaulin

```bash
# 生成HTML报告
cargo tarpaulin \
    --workspace \
    --out Html \
    --output-dir target/coverage \
    --exclude-files "*/tests/*" \
    --exclude-files "*/benches/*" \
    --exclude-files "*/examples/*" \
    --timeout 300

# 查看报告
open target/coverage/index.html
```

#### 使用 grcov

```bash
# 设置环境变量
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Cinstrument-coverage"
export LLVM_PROFILE_FILE="target/coverage/cargo-test-%p-%m.profraw"

# 清理并重新构建
cargo clean
cargo build --workspace --all-features

# 运行测试
cargo test --workspace --lib --all-features

# 生成HTML报告
grcov . \
    --binary-path ./target/debug/deps \
    -s . \
    -t html \
    --branch \
    --ignore-not-existing \
    --ignore "*/tests/*" \
    --ignore "*/benches/*" \
    --ignore "*/examples/*" \
    -o target/coverage/html

# 查看报告
open target/coverage/html/index.html

# 恢复环境变量
unset CARGO_INCREMENTAL
unset RUSTFLAGS
unset LLVM_PROFILE_FILE
```

## 报告格式

### HTML报告

最直观的报告格式，包含：
- 总体覆盖率统计
- 按文件分组的覆盖率
- 行级别的覆盖率详情
- 未覆盖代码高亮

**查看方式**:
```bash
open target/coverage/index.html
# 或
open target/coverage/html/index.html
```

### LCOV报告

标准格式，可用于：
- CI/CD集成
- 与外部工具集成
- 生成其他格式报告

**文件位置**: `target/coverage/lcov.info`

**转换为HTML** (需要安装lcov):
```bash
genhtml target/coverage/lcov.info -o target/coverage/html
```

### JSON报告

机器可读格式，用于：
- 自动化分析
- CI/CD集成
- 覆盖率趋势分析

**文件位置**: `target/coverage/cobertura.json`

### XML报告 (Cobertura)

用于CI/CD系统集成：
- Jenkins
- GitLab CI
- GitHub Actions

**文件位置**: `target/coverage/cobertura.xml`

## 覆盖率目标

### 推荐覆盖率

- **核心模块**: 80%+
- **工具模块**: 70%+
- **示例代码**: 不要求覆盖率
- **基准测试**: 不要求覆盖率

### 当前覆盖率

运行覆盖率测试后，查看报告获取当前覆盖率：
```bash
./scripts/run_coverage_report.sh
```

## 排除文件

以下文件默认排除在覆盖率统计之外：
- `*/tests/*` - 测试文件
- `*/benches/*` - 基准测试文件
- `*/examples/*` - 示例代码
- `*/target/*` - 构建输出

### 自定义排除

在脚本中添加 `--exclude-files` 参数：
```bash
cargo tarpaulin \
    --workspace \
    --exclude-files "*/custom_path/*" \
    --out Html \
    --output-dir target/coverage
```

## CI/CD集成

### GitHub Actions

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install cargo-tarpaulin
        run: cargo install cargo-tarpaulin
      - name: Generate coverage
        run: cargo tarpaulin --out Xml --output-dir coverage
      - name: Upload to Codecov
        uses: codecov/codecov-action@v2
        with:
          files: ./coverage/cobertura.xml
```

### GitLab CI

```yaml
coverage:
  stage: test
  image: rust:latest
  before_script:
    - cargo install cargo-tarpaulin
  script:
    - cargo tarpaulin --out Xml --output-dir coverage
  coverage: '/^\s*lines:\s*\d+\.\d+%/'
  artifacts:
    reports:
      cobertura: coverage/cobertura.xml
```

## 提高覆盖率

### 1. 识别未覆盖代码

查看HTML报告，找出未覆盖的代码区域：
- 红色：未覆盖
- 绿色：已覆盖
- 黄色：部分覆盖

### 2. 添加测试

为未覆盖的代码添加测试：
- 单元测试：测试单个函数
- 集成测试：测试模块交互
- 边界测试：测试边界条件

### 3. 测试策略

- **优先测试核心功能**: 确保核心功能有高覆盖率
- **测试错误路径**: 不仅测试正常路径，也要测试错误处理
- **测试边界条件**: 测试边界值和极端情况
- **避免过度测试**: 不要为了覆盖率而测试，关注代码质量

## 常见问题

### Q: 覆盖率报告显示0%？

**A**: 可能原因：
1. 测试未运行
2. 工具配置错误
3. 所有代码被排除

**解决方案**:
- 检查测试是否运行成功
- 检查排除文件配置
- 查看工具日志

### Q: 覆盖率报告不准确？

**A**: 可能原因：
1. 使用了错误的工具
2. 环境变量未正确设置
3. 增量编译问题

**解决方案**:
- 使用 `cargo clean` 清理
- 检查环境变量设置
- 尝试使用不同的工具

### Q: 覆盖率测试很慢？

**A**: 优化建议：
1. 使用 `--skip-clean` 跳过清理
2. 只测试特定包：`--package game_engine`
3. 增加超时时间：`--timeout 600`
4. 使用增量覆盖率

### Q: 如何查看历史覆盖率趋势？

**A**: 
1. 保存JSON报告到版本控制
2. 使用CI/CD系统跟踪趋势
3. 使用Codecov等工具可视化趋势

## 最佳实践

1. **定期运行**: 在每次重要提交后运行覆盖率测试
2. **设置目标**: 为不同模块设置合理的覆盖率目标
3. **关注质量**: 覆盖率是工具，不是目标，关注测试质量
4. **持续改进**: 逐步提高覆盖率，不要一次性要求100%
5. **审查报告**: 定期审查覆盖率报告，识别测试盲点

## 相关文档

- [测试指南](../guides/getting_started_guide.md#测试)
- [CI/CD配置](../architecture.md#cicd集成)
- [代码质量检查脚本](../scripts/check_code_quality.sh)

