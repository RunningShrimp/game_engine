# 依赖升级流程指南

本文档描述了游戏引擎项目的依赖升级流程和最佳实践。

## 自动化依赖更新

### Dependabot配置

项目使用 [Dependabot](https://docs.github.com/en/code-security/dependabot) 自动检测依赖更新。

**配置文件**: `.github/dependabot.yml`

**功能特性**:
- ✅ 每周自动检测依赖更新
- ✅ 自动创建Pull Request
- ✅ 支持Cargo、Python、Docker、GitHub Actions
- ✅ 自动分配审查者
- ✅ 智能版本更新策略

### 更新计划

| 生态系统 | 频率 | 日期 |
|---------|------|------|
| Cargo (主项目) | 每周 | 周一 |
| Cargo (profiling) | 每周 | 周一 |
| Cargo (examples) | 每周 | 周一 |
| GitHub Actions | 每周 | 周一 |
| Docker | 每周 | 周三 |
| Python (scripts) | 每月 | 第一天 |

### 版本更新策略

#### 自动合并规则

以下类型的更新可以自动合并：
- ✅ 直接依赖
- ✅ 生产依赖
- ✅ 补丁版本更新 (x.x.X)

#### 手动审查规则

以下类型的更新需要手动审查：
- ⚠️ 主要版本更新 (X.x.x)
- ⚠️ 次要版本更新 (x.X.x)
- ⚠️ 开发依赖
- ⚠️ 破坏性变更

## 手动依赖升级

### 步骤1: 检查依赖版本

```bash
# 检查过期的依赖
cargo outdated

# 检查特定依赖
cargo search wgpu
```

### 步骤2: 更新依赖

```bash
# 更新所有依赖到最新兼容版本
cargo update

# 更新特定依赖
cargo update wgpu

# 更新到特定版本
cargo update -p wgpu:0.20
```

### 步骤3: 测试更改

```bash
# 运行所有测试
cargo test --all

# 运行特定测试
cargo test --package game_engine

# 检查编译
cargo check --all
```

### 步骤4: 性能回归检测

```bash
# 运行性能基准测试
cargo bench --bench main

# 对比性能变化
cargo bench --bench main | benchcmp main.json
```

### 步骤5: 提交更改

```bash
git add Cargo.toml Cargo.lock
git commit -m "chore(deps): update wgpu to 0.20.0

- 更新wgpu核心库
- 修复API变更
- 更新相关示例

测试: 所有测试通过 ✅
性能: 无显著影响 ✅
"
```

## 关键依赖管理

### 渲染依赖 (wgpu)

**当前版本**: 0.20.x
**更新策略**: 手动审查 + 完整测试

**原因**: wgpu是核心渲染依赖，主要版本更新可能包含破坏性API变更。

**升级清单**:
- [ ] 阅读Release Notes
- [ ] 检查breaking changes
- [ ] 更新渲染代码
- [ ] 测试所有渲染示例
- [ ] 性能基准测试
- [ ] 平台兼容性测试

### ECS依赖 (bevy_ecs)

**当前版本**: 0.13.x
**更新策略**: 次要版本手动审查

**原因**: ECS是核心系统，需要确保性能和兼容性。

### 序列化依赖 (serde)

**当前版本**: 1.0.x
**更新策略**: 跟随最新稳定版

**原因**: serde是Rust生态标准库，通常向后兼容。

## 依赖版本策略

### 语义化版本 (SemVer)

- **Major (X.0.0)**: 破坏性变更
- **Minor (0.X.0)**: 新功能，向后兼容
- **Patch (0.0.X)**: Bug修复

### 我们的策略

| 类型 | 策略 | 说明 |
|-----|------|-----|
| **Patch** | 自动更新 | Bug修复，安全更新 |
| **Minor** | 审查后更新 | 新功能，检查兼容性 |
| **Major** | 延迟更新 | 破坏性变更，等待稳定 |

### 依赖锁定

**Cargo.lock** 文件锁定了所有间接依赖的精确版本，确保：

1. ✅ 可重现构建
2. ✅ 团队成员使用相同版本
3. ✅ CI/CD使用相同版本

**重要**: 始终提交 `Cargo.lock` 文件！

## 回滚策略

### 快速回滚

如果依赖更新导致问题：

```bash
# 1. 回滚Cargo.lock
git checkout HEAD~1 -- Cargo.lock

# 2. 恢复依赖
cargo build

# 3. 验证问题已解决
cargo test
```

### 创建回滚PR

```bash
# 1. 创建回滚分支
git checkout -b rollback-dep-update

# 2. 回滚到之前版本
git revert <merge-commit>

# 3. 提交并创建PR
git push origin rollback-dep-update
```

## CI/CD集成

### 自动化测试流程

每次依赖更新都会触发CI流程：

```yaml
# .github/workflows/dependency-review.yml
name: Dependency Review

on:
  pull_request:
    paths:
      - 'Cargo.toml'
      - 'Cargo.lock'

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run tests
        run: |
          cargo test --all

      - name: Check compilation
        run: |
          cargo check --all

      - name: Run benchmarks
        run: |
          cargo bench --all
```

### 性能回归检测

如果性能下降超过10%，PR将被标记：

```python
# scripts/check_performance.py
import subprocess
import json

def check_performance():
    # 获取基准性能
    baseline = get_baseline_performance()

    # 运行当前性能测试
    current = run_benchmarks()

    # 对比性能
    for name, (base_time, curr_time) in zip(baseline, current):
        degradation = (curr_time - base_time) / base_time

        if degradation > 0.10:  # 10%阈值
            print(f"⚠️  {name}: 性能下降 {degradation:.1%}")
            return False

    return True
```

## 安全更新

### 安全漏洞扫描

```bash
# 使用cargo-audit检查安全漏洞
cargo install cargo-audit
cargo audit
```

### 自动安全更新

Dependabot会自动创建安全更新PR，优先级高于常规更新。

**响应时间**: ⚡ 尽快合并安全更新

## 监控和告警

### 依赖状态监控

定期检查依赖健康状态：

```bash
# 检查过期的依赖
cargo outdated

# 检查未使用的依赖
cargo machete

# 检查依赖大小
cargo sort
```

### 告警配置

配置以下告警：
- 🚨 安全漏洞（立即）
- ⚠️ 许可证变更（24小时内）
- 📊 性能回归（每次PR）

## 最佳实践

### DO ✅

1. **保持依赖更新** - 定期更新到最新稳定版本
2. **阅读Release Notes** - 了解新功能和breaking changes
3. **测试完整** - 运行完整测试套件
4. **监控性能** - 检查性能回归
5. **提交Cargo.lock** - 确保可重现构建

### DON'T ❌

1. **不要盲目更新** - 大版本跳转需要仔细审查
2. **不要忽略警告** - 编译警告可能预示问题
3. **不要批量更新** - 一次更新多个依赖难以定位问题
4. **不要跳过测试** - 信任但要验证
5. **不要提交build目录** - 只提交源文件和Cargo.lock

## 故障排除

### 常见问题

**Q: 依赖更新后编译失败**

A: 检查是否是API破坏性变更：
1. 查看依赖的Release Notes
2. 搜索GitHub Issues
3. 检查是否有迁移指南

**Q: 性能下降**

A: 运行性能分析：
1. `cargo bench` 对比前后性能
2. 使用`flamegraph`分析热点
3. 向依赖仓库提交issue

**Q: 循环依赖**

A: 这是Cargo依赖解析的bug：
1. 尝试`cargo update -p <package>`
2. 清理缓存：`cargo clean`
3. 更新Cargo到最新版本

## 资源链接

- [Dependabot文档](https://docs.github.com/en/code-security/dependabot)
- [Cargo依赖管理](https://doc.rust-lang.org/cargo/guide/)
- [语义化版本](https://semver.org/)
- [Rust安全数据库](https://rustsec.org/)

## 变更日志

### 2025-01-01
- 配置Dependabot自动更新
- 创建依赖升级流程文档
- 添加性能回归检测
