# CI/CD Quality Gate - 快速参考

## 📋 一分钟速查

### 本地检查命令

```bash
# 快速检查 (推荐)
./scripts/ci-check.sh

# 详细报告
./scripts/quality-report.sh

# 安装hooks
./scripts/install-hooks.sh
```

### 手动运行检查

```bash
# Format
cargo fmt --all -- --check

# Clippy
cargo clippy --workspace --all-targets -- -D warnings

# Test
cargo test --workspace --lib

# Doc
cargo doc --workspace --no-deps

# Coverage
cargo llvm-cov --workspace --summary-only
```

## 🔍 质量门禁检查项

| 检查项 | 工具 | 状态 | 阈值 |
|--------|------|------|------|
| Format | rustfmt | ✅ 必须 | 通过 |
| Clippy | clippy | ✅ 必须 | ≤10警告 |
| Test | cargo test | ✅ 必须 | 100%通过 |
| Doc | rustdoc | ✅ 必须 | 无错误 |
| Coverage | llvm-cov | ⚠️ 建议 | ≥50% |
| Audit | cargo-audit | ⚠️ 警告 | 无漏洞 |
| Examples | cargo build | ✅ 必须 | 编译成功 |
| Complexity | cargo-complexity | ⚠️ 建议 | <20 |

## 📝 PR检查清单

在创建PR前确认:

- [ ] `cargo fmt --all` 已运行
- [ ] `cargo clippy` 无警告
- [ ] `cargo test --workspace` 全部通过
- [ ] `cargo doc --workspace` 无错误
- [ ] 新增功能有测试
- [ ] 更新了文档

## 🚨 常见问题速解

### Format失败
```bash
cargo fmt --all
```

### Clippy警告过多
```bash
cargo clippy --workspace --all-targets -- -D warnings
# 修复警告或添加 #[allow(clippy::warning_name)]
```

### 测试失败
```bash
cargo test --workspace -- --nocapture
# 查看详细输出
```

### 覆盖率低
```bash
cargo llvm-cov --workspace --html
# 在target/llvm-cov/html/查看报告
```

## 📚 完整文档

详细文档: [CI_CD_QUALITY_GATE_GUIDE.md](CI_CD_QUALITY_GATE_GUIDE.md)

## 🔧 工具安装

```bash
# 必需工具
rustup component add rustfmt clippy

# 可选工具
cargo install cargo-llvm-cov    # 覆盖率
cargo install cargo-audit       # 安全审计
cargo install cargo-outdated    # 依赖检查
cargo install cargo-complexity  # 复杂度

# Pre-commit
pip3 install pre-commit
./scripts/install-hooks.sh
```

## 💡 工作流建议

```bash
# 1. 创建功能分支
git checkout -b feature/my-feature

# 2. 开发并频繁检查
./scripts/ci-check.sh

# 3. 提交
git add .
git commit -m "feat: add my feature"

# 4. 推送
git push origin feature/my-feature

# 5. 创建PR (填写模板)
```

## 🎯 质量目标

- **代码格式**: 100%符合rustfmt
- **静态分析**: ≤10个clippy警告
- **测试通过率**: 100%
- **文档完整性**: 无缺失/错误链接
- **代码覆盖率**: ≥50%
- **代码复杂度**: <20

## 📞 获取帮助

- 文档: `docs/CI_CD_QUALITY_GATE_GUIDE.md`
- Bug: 使用`.github/ISSUE_TEMPLATE/bug_report.md`
- 问题: 使用`.github/ISSUE_TEMPLATE/feature_request.md`

---

**记住**: 运行`./scripts/ci-check.sh`在提交前可以避免大多数CI失败! 🚀
