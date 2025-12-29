# CI/CD Quality Gate 使用指南

本文档描述游戏引擎的CI/CD质量门禁体系。

## 目录

1. [概述](#概述)
2. [创建的文件](#创建的文件)
3. [GitHub Actions Workflows](#github-actions-workflows)
4. [本地测试](#本地测试)
5. [质量检查项](#质量检查项)
6. [模板使用](#模板使用)
7. [故障排除](#故障排除)

## 概述

质量门禁体系确保代码在合并前满足质量标准，包括：
- 代码格式检查
- Clippy静态分析
- 单元测试和集成测试
- 文档生成和测试
- 代码覆盖率检查
- 安全审计
- 依赖检查
- 示例编译检查
- 代码复杂度检查

## 创建的文件

### 1. GitHub Actions Workflows
```
.github/workflows/quality-gate.yml          # 主质量门禁workflow
```

### 2. PR和Issue模板
```
.github/PULL_REQUEST_TEMPLATE.md           # Pull Request模板
.github/ISSUE_TEMPLATE/bug_report.md       # Bug报告模板
.github/ISSUE_TEMPLATE/feature_request.md  # 功能请求模板
.github/ISSUE_TEMPLATE/performance_issue.md # 性能问题模板
```

### 3. Pre-commit配置
```
.pre-commit-config.yaml                     # Pre-commit hooks配置
scripts/install-hooks.sh                    # Hook安装脚本
```

### 4. 质量检查脚本
```
scripts/quality-report.sh                   # 生成详细质量报告
scripts/ci-check.sh                         # 快速CI检查
```

## GitHub Actions Workflows

### Quality Gate Workflow

**触发条件:**
- Push到main或develop分支
- Pull Request到main或develop分支
- 手动触发 (workflow_dispatch)

**包含的Jobs:**

1. **format** - 代码格式检查
   - 检查所有Rust代码是否符合rustfmt标准
   - 状态: 必须通过

2. **clippy** - Clippy静态分析
   - 运行clippy检查代码质量问题
   - 警告数量阈值: 10个
   - 状态: 必须通过且警告数<=10

3. **test** - 单元测试
   - 运行所有单元测试
   - 运行所有集成测试
   - 状态: 必须通过

4. **doc** - 文档检查
   - 生成项目文档
   - 运行文档测试
   - 检查文档链接是否完整
   - 状态: 必须通过

5. **coverage** - 代码覆盖率
   - 使用cargo-llvm-cov生成覆盖率
   - 覆盖率阈值: 50%
   - 上传到Codecov
   - 状态: 建议通过 (>=50%)

6. **audit** - 安全审计
   - 使用cargo-audit检查依赖漏洞
   - 状态: 警告级别 (不阻塞PR)

7. **outdated** - 依赖检查
   - 检查是否有过期依赖
   - 状态: 信息级别 (不阻塞PR)

8. **examples** - 示例编译
   - 编译所有示例代码
   - 状态: 必须通过

9. **complexity** - 复杂度检查
   - 使用cargo-complexity分析代码复杂度
   - 复杂度阈值: 20
   - 状态: 建议通过 (不阻塞PR)

10. **quality-summary** - 质量汇总
    - 汇总所有检查结果
    - 生成质量报告摘要
    - 决定质量门禁最终状态

### 质量门禁规则

**必须通过的检查 (阻塞):**
- ✅ format (代码格式)
- ✅ clippy (静态分析)
- ✅ test (单元测试)
- ✅ doc (文档)
- ✅ examples (示例编译)

**建议通过的检查 (警告):**
- ⚠️ coverage (覆盖率 >= 50%)
- ⚠️ complexity (复杂度 < 20)
- ⚠️ audit (无安全漏洞)
- ⚠️ outdated (依赖最新)

## 本地测试

在提交代码前，建议在本地运行质量检查以避免CI失败。

### 方法1: 快速CI检查

```bash
# 运行所有检查
./scripts/ci-check.sh

# 快速模式 (跳过覆盖率)
./scripts/ci-check.sh --fast

# 自动修复可修复的问题
./scripts/ci-check.sh --fix
```

### 方法2: 生成详细质量报告

```bash
# 生成质量报告
./scripts/quality-report.sh

# 输出到指定文件
./scripts/quality-report.sh --output my-report.md

# CI模式 (非零退出码表示失败)
./scripts/quality-report.sh --ci
```

### 方法3: 手动运行各项检查

```bash
# 1. 格式检查
cargo fmt --all -- --check

# 2. Clippy检查
cargo clippy --workspace --all-targets -- -D warnings

# 3. 单元测试
cargo test --workspace --lib

# 4. 文档生成
cargo doc --workspace --no-deps

# 5. 覆盖率 (需要cargo-llvm-cov)
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --summary-only

# 6. 安全审计
cargo install cargo-audit
cargo audit

# 7. 示例编译
cargo build --examples
```

## 质量检查项

### 1. 代码格式化 (Format)

**目的:** 确保代码风格一致

**工具:** rustfmt

**检查命令:**
```bash
cargo fmt --all -- --check
```

**自动修复:**
```bash
cargo fmt --all
```

**CI状态:** 必须通过

### 2. Clippy静态分析 (Clippy)

**目的:** 发现代码质量问题和潜在bug

**工具:** clippy

**检查命令:**
```bash
cargo clippy --workspace --all-targets -- -D warnings
```

**常见警告类型:**
- `clippy::all` - 所有lints
- `clippy::pedantic` - 更严格的检查
- `clippy::cargo` - Cargo配置检查

**警告阈值:** 10个警告

**CI状态:** 必须通过且警告数<=10

### 3. 单元测试 (Test)

**目的:** 验证代码功能正确性

**工具:** cargo test

**检查命令:**
```bash
# 单元测试
cargo test --workspace --lib

# 集成测试
cargo test --workspace --test '*'

# 特定测试
cargo test --package <package> --test <test_name>
```

**CI状态:** 必须通过

### 4. 文档检查 (Doc)

**目的:** 确保文档完整且正确

**工具:** rustdoc

**检查命令:**
```bash
# 生成文档
cargo doc --workspace --no-deps

# 文档测试
cargo test --workspace --doc

# 检查文档链接
cargo doc --workspace --no-deps 2>&1 | grep "cannot be resolved"
```

**CI状态:** 必须通过

### 5. 代码覆盖率 (Coverage)

**目的:** 衡量测试覆盖程度

**工具:** cargo-llvm-cov 或 cargo-tarpaulin

**检查命令:**
```bash
# 安装工具
cargo install cargo-llvm-cov

# 生成覆盖率
cargo llvm-cov --workspace --summary-only

# HTML报告
cargo llvm-cov --workspace --html
```

**覆盖率阈值:** 50%

**CI状态:** 建议通过 (>=50%)

### 6. 安全审计 (Audit)

**目的:** 检查依赖的安全漏洞

**工具:** cargo-audit

**检查命令:**
```bash
# 安装工具
cargo install cargo-audit

# 运行审计
cargo audit

# JSON输出
cargo audit --json
```

**CI状态:** 警告级别 (不阻塞PR)

### 7. 依赖检查 (Outdated)

**目的:** 发现过期的依赖

**工具:** cargo-outdated

**检查命令:**
```bash
# 安装工具
cargo install cargo-outdated

# 检查过期依赖
cargo outdated --workspace
```

**CI状态:** 信息级别 (不阻塞PR)

### 8. 示例编译 (Examples)

**目的:** 确保示例代码可用

**检查命令:**
```bash
# 编译所有示例
for example in examples/*.rs; do
    cargo build --example $(basename $example .rs)
done
```

**CI状态:** 必须通过

### 9. 代码复杂度 (Complexity)

**目的:** 控制代码复杂度

**工具:** cargo-complexity

**检查命令:**
```bash
# 安装工具
cargo install cargo-complexity

# 检查复杂度
cargo complexity --workspace --threshold 20
```

**复杂度阈值:** 20

**CI状态:** 建议通过 (<20)

## 模板使用

### Pull Request模板

创建PR时，系统会自动加载模板。模板包含以下部分：

**必填项:**
- 描述: PR的目的
- 改动类型: Bug修复/新功能/性能改进等
- 测试: 测试情况
- 检查清单: 质量检查确认

**可选项:**
- 变更内容: 主要文件变更
- API变更: 公共API改动
- 性能影响: 性能变化评估
- 相关Issue: 关联的issue
- 截图/演示: 视觉展示

**示例:**

```markdown
## 描述
添加了新的物理引擎模块，支持刚体碰撞检测

## 改动类型
- [x] 新功能
- [ ] Bug修复

## 测试
- [x] 所有现有测试通过
- [x] 添加了新测试 (physics/tests/collision_test.rs)
- [x] 手动测试完成

## 检查清单
- [x] 代码遵循项目风格指南
- [x] 没有Clippy警告
- [x] 所有测试通过
```

### Issue模板

项目提供3种Issue模板：

**1. Bug报告 (`bug_report.md`)**
- Bug描述
- 复现步骤
- 期望行为 vs 实际行为
- 环境信息
- 错误日志

**2. 功能请求 (`feature_request.md`)**
- 功能描述
- 问题背景
- 提议的解决方案
- 替代方案
- 实现复杂度评估

**3. 性能问题 (`performance_issue.md`)**
- 性能问题描述
- 性能指标 (期望vs实际)
- 复现场景
- Profiling数据
- 硬件配置

## Pre-commit Hooks

Pre-commit hooks在每次提交前自动运行质量检查。

### 安装

```bash
# 安装pre-commit
./scripts/install-hooks.sh

# 或手动安装
pip3 install pre-commit
pre-commit install
```

### 使用

```bash
# 正常提交 (自动运行hooks)
git commit -m "message"

# 跳过hooks (不推荐)
git commit --no-verify -m "message"

# 手动运行所有hooks
pre-commit run --all-files

# 运行特定hook
pre-commit run rust-fmt --all-files
```

### 配置的Hooks

1. **rust-fmt** - 格式检查
2. **rust-clippy** - Clippy检查
3. **rust-test** - 测试运行
4. **rust-doc** - 文档检查
5. **cargo-toml-fmt** - Cargo.toml格式化
6. **yaml-format** - YAML格式化
7. **markdownlint** - Markdown检查
8. **trailing-whitespace** - 尾随空格检查
9. **end-of-file-fixer** - 文件结束符检查
10. **detect-private-key** - 私钥检测
11. **cargo-audit** - 安全审计

## 故障排除

### 问题1: CI中format检查失败

**症状:**
```
Error: Code formatting issues found
```

**解决:**
```bash
# 本地运行format
cargo fmt --all

# 验证
cargo fmt --all -- --check

# 提交修复
git add .
git commit -m "fix: format code"
```

### 问题2: Clippy警告过多

**症状:**
```
Error: Too many warnings: 15 > 10
```

**解决:**
```bash
# 本地运行clippy
cargo clippy --workspace --all-targets -- -D warnings

# 逐个修复警告，或允许特定警告
// #![allow(clippy::too_many_arguments)]

# 重新检查
cargo clippy --workspace --all-targets -- -D warnings
```

### 问题3: 测试失败

**症状:**
```
Error: Test failed
```

**解决:**
```bash
# 本地运行测试
cargo test --workspace

# 运行特定测试
cargo test --package <package> test_name

# 带输出的测试
cargo test --workspace -- --nocapture

# 过滤测试
cargo test --workspace physics
```

### 问题4: 文档链接错误

**症状:**
```
Error: Found unresolved doc links
```

**解决:**
```bash
# 本地生成文档并检查链接
cargo doc --workspace --no-deps --all-features 2>&1 | grep "cannot be resolved"

# 修复链接 (确保路径正确)
// 修复前: [Struct](crate::module::Struct)
// 修复后: [Struct](../module/struct.Struct.html)

# 重新检查
cargo doc --workspace --no-deps
```

### 问题5: 覆盖率低于阈值

**症状:**
```
Warning: Coverage below 50% threshold
```

**解决:**
```bash
# 生成本地覆盖率报告
cargo llvm-cov --workspace --html

# 在浏览器中查看
open target/llvm-cov/html/index.html

# 为未覆盖的代码添加测试
// 在tests/目录添加测试用例
```

### 问题6: Pre-commit hooks失败

**症状:**
```
Error: Pre-commit hook failed
```

**解决:**
```bash
# 手动运行hooks查看详情
pre-commit run --all-files

# 跳过特定hook (临时)
SKIP=rust-test git commit -m "message"

# 更新hooks
pre-commit autoupdate
```

### 问题7: 依赖安全漏洞

**症状:**
```
Warning: Found 3 vulnerabilities
```

**解决:**
```bash
# 查看详细信息
cargo audit

# 更新依赖
cargo update

# 如果无法更新，评估风险
# 可能需要联系维护者或等待修复
```

## 最佳实践

### 1. 提交前检查

```bash
# 运行快速检查
./scripts/ci-check.sh

# 自动修复可修复的问题
./scripts/ci-check.sh --fix
```

### 2. 分支策略

```bash
# 为每个功能创建分支
git checkout -b feature/my-feature

# 定期同步main
git fetch origin main
git rebase origin/main

# 推送前运行检查
./scripts/ci-check.sh
```

### 3. PR工作流

```bash
# 1. 创建功能分支
git checkout -b feature/awesome-feature

# 2. 开发并测试
cargo test --workspace
cargo clippy --workspace -- -D warnings

# 3. 提交
git add .
git commit -m "feat: add awesome feature"

# 4. 推送
git push origin feature/awesome-feature

# 5. 创建PR (使用模板)
# 在GitHub上创建PR，填写模板内容
```

### 4. 持续改进

- 定期更新依赖: `cargo outdated`
- 监控覆盖率趋势
- 定期审查Clippy警告
- 保持示例代码更新

## 相关资源

- [Rustfmt文档](https://rust-lang.github.io/rustfmt/)
- [Clippy文档](https://rust-lang.github.io/rust-clippy/)
- [Cargo文档](https://doc.rust-lang.org/cargo/)
- [Pre-commit文档](https://pre-commit.com/)
- [GitHub Actions文档](https://docs.github.com/en/actions)

## 总结

CI/CD质量门禁确保代码质量和项目健康。通过：

1. ✅ 自动化质量检查
2. ✅ 标准化PR/Issue流程
3. ✅ Pre-commit hooks本地验证
4. ✅ 快速反馈循环

帮助团队保持高质量代码标准。

如有问题，请查阅本文档或联系维护团队。
