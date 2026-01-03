# 测试指南

本指南提供游戏引擎项目的完整测试策略、工具和最佳实践。

## 目录

- [概述](#概述)
- [测试架构](#测试架构)
- [运行测试](#运行测试)
- [编写测试](#编写测试)
- [性能基准测试](#性能基准测试)
- [覆盖率报告](#覆盖率报告)
- [CI/CD集成](#cicd集成)
- [最佳实践](#最佳实践)

---

## 概述

### 目标

- **代码覆盖率**: 从19%提升至50%
- **测试类型**: 单元测试、集成测试、性能测试
- **核心模块**: 渲染、物理、ECS、数学库

### 测试框架

- **单元测试**: Rust内置测试框架
- **覆盖率**: cargo-tarpaulin
- **基准测试**: Criterion.rs
- **属性测试**: Proptest

---

## 测试架构

### 目录结构

```
game_engine/
├── tests/                      # 集成测试
│   ├── test_infrastructure/   # 测试基础设施
│   ├── render/               # 渲染系统测试
│   ├── physics/              # 物理系统测试
│   ├── entity/               # ECS系统测试
│   └── math/                 # 数学库测试
└── benches/                  # 性能基准测试
```

---

## 运行测试

### 基本测试命令

\`\`\`bash
# 运行所有测试
cargo test --workspace

# 运行特定模块测试
cargo test --package game_engine --lib render

# 运行基准测试
cargo bench --workspace
\`\`\`

---

## 编写测试

### 单元测试示例

\`\`\`rust
#[test]
fn test_basic_functionality() {
    let result = some_function();
    assert_eq!(result, expected);
}
\`\`\`

---

## 覆盖率报告

### 生成覆盖率报告

\`\`\`bash
cargo tarpaulin --workspace --out Html --output-dir target/coverage
\`\`\`

---

## CI/CD集成

自动测试通过GitHub Actions配置，详见`.github/workflows/test.yml`。

---

## 最佳实践

1. 测试命名清晰
2. 测试独立隔离
3. 使用自定义断言
4. 测试边界情况
5. 性能测试

---

更多详情请参考完整的测试文档。
