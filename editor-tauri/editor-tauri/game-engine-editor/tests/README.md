# Integration Tests Documentation

## 概述

本测试套件为游戏引擎编辑器提供全面的集成测试，覆盖所有新实现的功能：

1. **平台认证系统** (~1,306行代码)
2. **控制器扩展功能** (~1,293行代码)
3. **GPU管理器优化** (~850行代码)
4. **代码去重工具集** (7个宏 + 平台trait)

## 测试结构

```
tests/
├── integration/              # 集成测试
│   ├── mod.rs               # 测试模块入口
│   ├── platform_certification_tests.rs    # 平台认证测试
│   ├── controller_extended_tests.rs       # 控制器扩展测试
│   ├── gpu_manager_tests.rs              # GPU管理器测试
│   ├── code_tools_tests.rs               # 代码工具测试
│   ├── editor_integration_tests.rs       # 编辑器集成测试
│   └── e2e_scenario_tests.rs             # 端到端场景测试
├── fixtures/                # 测试fixtures
│   ├── mod.rs
│   ├── test_entities.rs    # 测试实体和数据结构
│   ├── test_scenes.rs      # 测试场景
│   └── mock_platforms.rs   # Mock平台实现
└── helpers/                # 测试辅助工具
    ├── mod.rs
    ├── assert_helpers.rs   # 自定义断言
    └── test_helpers.rs     # 测试工具函数
```

## 测试覆盖率目标

- **新代码覆盖率**: >85%
- **核心功能覆盖率**: >95%
- **边界条件覆盖率**: >80%

## 运行测试

### 运行所有测试

```bash
cargo test
```

### 运行特定测试模块

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

### 运行测试并显示输出

```bash
cargo test -- --nocapture
```

### 运行测试并生成覆盖率报告

```bash
# 安装tarpaulin
cargo install cargo-tarpaulin

# 生成覆盖率报告
cargo tarpaulin --out Html --output-dir coverage/
```

## 测试分类

### 1. 平台认证系统测试 (platform_certification_tests.rs)

覆盖范围：
- ✅ 5个平台的基本认证 (PS5, Xbox, Switch, Steam, Epic)
- ✅ 平台特定功能检查 (Trophy, Achievement, Cloud Save等)
- ✅ 自定义认证规则
- ✅ 批量认证检查
- ✅ 并行认证处理
- ✅ 报告生成 (文本/JSON)
- ✅ 边界条件和错误处理

测试数量：30+

### 2. 控制器扩展功能测试 (controller_extended_tests.rs)

覆盖范围：
- ✅ 所有平台的基本控制器功能
- ✅ 振动功能 (PS5, Xbox, Switch)
- ✅ LED控制 (PS5, PS4)
- ✅ 触摸板输入 (PS5, PS4)
- ✅ 运动传感器 (PS5, PS4, Switch)
- ✅ PS5特定功能 (Haptic Feedback, Adaptive Triggers)
- ✅ 多平台控制器兼容性
- ✅ 控制器校准
- ✅ 多控制器支持
- ✅ 性能测试

测试数量：40+

### 3. GPU管理器测试 (gpu_manager_tests.rs)

覆盖范围：
- ✅ VRAM管理 (分配/释放/使用率)
- ✅ 视锥剔除
- ✅ 遮挡剔除 (需要光线追踪支持)
- ✅ 距离剔除
- ✅ 间接绘制 (需要Mesh Shader支持)
- ✅ 多特性组合
- ✅ 自适应质量策略
- ✅ VRAM管理策略
- ✅ 性能基准测试
- ✅ 边界条件 (碎片化/不足/超额)

测试数量：35+

### 4. 代码去重工具测试 (code_tools_tests.rs)

覆盖范围：
- ✅ 7个平台宏的正确性
- ✅ Platform trait实现
- ✅ 代码生成正确性
- ✅ 宏展开正确性
- ✅ 零成本抽象验证
- ✅ 编译时检查
- ✅ 类型安全
- ✅ 性能开销测试
- ✅ 代码量减少验证

测试数量：25+

### 5. 编辑器集成测试 (editor_integration_tests.rs)

覆盖范围：
- ✅ 编辑器与GPU系统集成
- ✅ 编辑器与控制器系统集成
- ✅ 编辑器与平台认证集成
- ✅ 资源导入和GPU内存管理
- ✅ 撤销/重做集成
- ✅ 性能监控集成
- ✅ 多平台适配
- ✅ 并发操作
- ✅ 数据持久化
- ✅ UI交互
- ✅ 错误处理和恢复

测试数量：20+

### 6. 端到端场景测试 (e2e_scenario_tests.rs)

覆盖范围：
- ✅ 创建和测试新游戏项目
- ✅ 多平台游戏开发和测试
- ✅ 复杂场景创建和优化
- ✅ 游戏控制器输入和反馈
- ✅ 平台认证和修复流程
- ✅ GPU性能优化工作流
- ✅ 实时编辑和测试
- ✅ 资源加载和管理
- ✅ 多玩家协同开发
- ✅ 完整游戏开发周期
- ✅ 性能压力测试
- ✅ 错误恢复场景
- ✅ 数据一致性验证

测试数量：13个场景

## 测试辅助工具

### 自定义断言 (assert_helpers.rs)

```rust
assert_approx_eq(a, b, epsilon)           // 浮点数近似相等
assert_vec3_approx_eq(a, b, epsilon)      // 向量近似相等
assert_completes_within(duration, action) // 超时断言
assert_contains(collection, item)         // 包含断言
assert_string_contains(haystack, needle)  // 字符串包含断言
assert_ok(result)                         // Ok断言
assert_err(result)                        // Err断言
assert_some(option)                       // Some断言
assert_none(option)                       // None断言
assert_in_range(value, min, max)         // 范围断言
```

### 测试工具 (test_helpers.rs)

```rust
Timer::new()                              // 性能计时器
retry(attempts, delay, operation)         // 重试机制
ConcurrentTestRunner                      // 并发测试
SharedState<T>                            // 共享状态管理
TestDataGenerator                        // 测试数据生成
MemoryMonitor                             // 内存使用监控 (Linux)
TestLogCollector                          // 日志收集器
```

### Mock对象 (mock_platforms.rs)

```rust
MockCertificationSystem                   // Mock认证系统
MockController                           // Mock控制器
MockGPUManager                           // Mock GPU管理器
MockPlatformManager                      // Mock平台管理器
```

## 性能基准

所有性能测试的目标指标：

| 操作 | 目标时间 |
|------|---------|
| 认证检查 (单次) | <1ms |
| 认证检查 (100次) | <100ms |
| 控制器输入处理 (10000次) | <100ms |
| VRAM分配/释放 (10000次) | <100ms |
| 特性切换 (1000次) | <50ms |
| VRAM使用率计算 (10000次) | <50ms |
| 宏展开 (100000次) | <100ms |

## CI/CD集成

### GitHub Actions配置

```yaml
name: Integration Tests

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main, develop ]

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

    steps:
    - uses: actions/checkout@v2

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run tests
      run: cargo test --all-features

    - name: Generate coverage
      run: cargo tarpaulin --out Xml

    - name: Upload coverage
      uses: codecov/codecov-action@v2
```

### 本地预提交检查

```bash
#!/bin/bash
# scripts/pre-commit-test.sh

echo "Running tests..."
cargo test --quiet

echo "Checking formatting..."
cargo fmt -- --check

echo "Running clippy..."
cargo clippy -- -D warnings

echo "All checks passed!"
```

## 测试最佳实践

1. **测试命名**: 使用描述性的测试名称 `test_<function>_<scenario>`
2. **独立性**: 每个测试应该独立运行，不依赖其他测试
3. **可重复性**: 测试结果应该是确定性的
4. **快速执行**: 单元测试应该快速完成
5. **清晰断言**: 使用自定义断言提供更好的错误消息
6. **边界测试**: 测试正常情况和边界条件
7. **Mock使用**: 使用mock对象隔离外部依赖

## 测试维护

### 添加新测试

1. 在相应的模块中添加测试函数
2. 使用描述性的测试名称
3. 包含文档注释说明测试目的
4. 更新本README文档

### 更新Fixtures

当数据结构变化时：
1. 更新 `test_entities.rs` 中的结构定义
2. 更新 `mock_platforms.rs` 中的mock实现
3. 确保所有使用这些fixture的测试仍然通过

### 性能退化检测

定期运行性能基准测试：
```bash
cargo test --release performance
```

## 已知限制

1. **平台特定测试**: 某些平台特定功能仅在相应平台上可测试
2. **GPU功能测试**: 依赖实际GPU硬件，某些功能在CI中可能无法测试
3. **性能测试**: 在CI环境中可能比本地慢

## 故障排除

### 测试超时

增加超时时间：
```bash
cargo test --test-timeout=300
```

### 内存泄漏检测

使用Valgrind (Linux):
```bash
cargo test --no-run
valgrind --leak-check=full target/debug/deps/*-*
```

### 并发测试失败

禁用测试并行化：
```bash
cargo test -- --test-threads=1
```

## 贡献指南

添加新测试时请确保：
1. ✅ 测试名称清晰描述测试内容
2. ✅ 包含正常和边界情况
3. ✅ 使用适当的辅助工具和fixtures
4. ✅ 测试独立且可重复
5. ✅ 更新相关文档

## 联系方式

测试相关问题请联系测试团队或提交Issue。
