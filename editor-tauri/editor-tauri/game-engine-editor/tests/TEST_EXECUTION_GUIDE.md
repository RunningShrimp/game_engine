# Test Execution Guide

## 快速开始

### 运行所有测试

```bash
# 基础测试运行
cargo test

# 详细输出
cargo test -- --nocapture

# 单线程运行（避免并发问题）
cargo test -- --test-threads=1
```

### 运行特定测试

```bash
# 平台认证测试
cargo test platform_certification

# 控制器测试
cargo test controller

# GPU管理器测试
cargo test gpu_manager

# 代码工具测试
cargo test code_tools

# 编辑器集成测试
cargo test editor_integration

# 端到端测试
cargo test e2e
```

### 运行单个测试

```bash
cargo test test_name::test_function_name
```

## 使用测试脚本

### 完整测试运行

```bash
# 运行完整测试套件
./scripts/run_integration_tests.sh

# 详细输出
./scripts/run_integration_tests.sh --verbose

# 生成覆盖率报告
./scripts/run_integration_tests.sh --coverage

# 运行性能基准测试
./scripts/run_integration_tests.sh --benchmark

# 运行特定测试
./scripts/run_integration_tests.sh --test platform_certification

# 查看帮助
./scripts/run_integration_tests.sh --help
```

### 预提交检查

```bash
# 快速预提交检查
./scripts/pre-commit-test.sh
```

## 覆盖率报告

### 生成覆盖率

```bash
# 安装tarpaulin（如果未安装）
cargo install cargo-tarpaulin

# 生成HTML报告
cargo tarpaulin --all-features --out Html --output-dir coverage/

# 生成XML报告（用于CI）
cargo tarpaulin --all-features --out Xml --output-dir coverage/

# 生成终端报告
cargo tarpaulin --all-features --out Stdout
```

### 查看覆盖率

```bash
# 在浏览器中打开
open coverage/index.html  # macOS
xdg-open coverage/index.html  # Linux
start coverage/index.html  # Windows
```

## 性能测试

### 运行性能测试

```bash
# 运行所有性能测试
cargo test performance

# 运行性能测试并显示输出
cargo test performance -- --nocapture

# Release模式运行（更准确）
cargo test --release performance -- --nocapture
```

### 性能基准

当前性能基准：

| 测试类别 | 目标时间 |
|---------|---------|
| 认证检查 (单次) | <1ms |
| 认证检查 (100次) | <100ms |
| 控制器输入 (10000次) | <100ms |
| VRAM操作 (10000次) | <100ms |
| 特性切换 (1000次) | <50ms |
| VRAM计算 (10000次) | <50ms |
| 宏展开 (100000次) | <100ms |

## 故障排除

### 测试超时

```bash
# 增加超时时间
cargo test --test-timeout=300
```

### 并发问题

```bash
# 单线程运行
cargo test -- --test-threads=1
```

### 只运行失败的测试

```bash
# 编辑tests/目录，临时禁用其他测试
# 或使用：
cargo test -- --exact 'test_name'
```

### 查看测试输出

```bash
# 显示所有输出
cargo test -- --nocapture

# 显示测试捕获的输出
cargo test -- --show-output
```

## CI/CD集成

### GitHub Actions

测试会在以下情况自动运行：
- 推送到main或develop分支
- 创建Pull Request
- 手动触发（workflow_dispatch）

### 本地预提交

配置Git钩子：

```bash
# 复制预提交脚本
cp scripts/pre-commit-test.sh .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

## 测试分类

### 单元测试

快速、隔离的测试：
- helpers模块测试
- fixtures模块测试
- 单个函数测试

### 集成测试

跨模块集成测试：
- platform_certification_tests
- controller_extended_tests
- gpu_manager_tests
- code_tools_tests
- editor_integration_tests

### 端到端测试

完整场景测试：
- e2e_scenario_tests

### 性能测试

性能基准测试：
- 所有performance_*测试

## 持续监控

### 定期运行

建议定期运行：

```bash
# 每日完整测试
./scripts/run_integration_tests.sh --coverage --benchmark

# 每次提交前
./scripts/pre-commit-test.sh
```

### 性能退化检测

```bash
# 运行性能测试并保存结果
cargo test --release performance -- --nocapture > perf_baseline.txt

# 比较结果
diff perf_baseline.txt perf_new.txt
```

## 最佳实践

1. **运行测试前**: 确保代码已编译
2. **提交代码前**: 运行预提交检查
3. **推送代码前**: 运行完整测试套件
4. **添加新功能**: 同时添加测试
5. **修复Bug**: 先添加失败的测试，然后修复

## 获取帮助

- 查看测试文档: `tests/README.md`
- 查看测试报告: `tests/INTEGRATION_TEST_REPORT.md`
- 运行帮助: `./scripts/run_integration_tests.sh --help`
