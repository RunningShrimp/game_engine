# CI/CD流水线优化指南

## 概述

本文档介绍游戏引擎的CI/CD流水线优化，包括自动发布、覆盖率报告和性能回归检测。

## 工作流文件

### 1. 自动发布 (`release.yml`)

**功能**:
- 自动创建GitHub Release
- 多平台构建（Linux、macOS、Windows）
- 自动上传构建产物
- 发布到Crates.io

**触发条件**:
- 推送版本标签（如 `v1.0.0`）
- 手动触发（workflow_dispatch）

**使用方式**:
```bash
# 创建版本标签
git tag v1.0.0
git push origin v1.0.0

# 或通过GitHub Actions手动触发
```

### 2. 增强的覆盖率报告 (`coverage-enhanced.yml`)

**功能**:
- 使用 `cargo-llvm-cov` 生成详细覆盖率报告
- 上传到Codecov
- 生成HTML覆盖率报告
- PR评论覆盖率结果
- 覆盖率趋势分析

**触发条件**:
- 推送到main/develop分支
- Pull Request
- 每周一自动运行

**覆盖率目标**:
- 目标覆盖率: 80%+
- 自动检测覆盖率下降

### 3. 增强的性能回归检测 (`performance-regression-enhanced.yml`)

**功能**:
- 运行所有基准测试
- 对比性能基线
- 检测性能回归
- PR评论性能结果
- 自动更新性能基线（main分支）
- 性能趋势分析

**触发条件**:
- 推送到main/develop分支
- Pull Request
- 每天自动运行

**性能阈值**:
- 警告阈值: 10%性能下降
- 严重阈值: 20%性能下降

## 现有工作流

### CI流水线 (`ci.yml`)

包含以下任务：
- 代码质量检查（格式化、Clippy）
- 跨平台测试
- 代码覆盖率
- 构建
- 性能基准测试
- 文档生成

### 跨平台测试 (`cross-platform-test.yml`)

- 支持Linux、macOS、Windows
- 支持WASM目标
- 多Rust版本测试

## 配置要求

### GitHub Secrets

需要配置以下secrets：

1. **CRATES_IO_TOKEN** (可选)
   - 用于发布到Crates.io
   - 获取方式: https://crates.io/me

2. **CODECOV_TOKEN** (可选)
   - 用于上传覆盖率到Codecov
   - 获取方式: https://codecov.io/

### 性能基线文件

确保 `performance_baselines.json` 文件存在并包含最新的性能基线数据。

## 使用建议

### 发布新版本

1. 更新版本号
2. 更新CHANGELOG.md
3. 创建版本标签: `git tag v1.0.0`
4. 推送标签: `git push origin v1.0.0`
5. GitHub Actions会自动创建Release并构建所有平台版本

### 查看覆盖率报告

1. 在PR中查看覆盖率评论
2. 下载HTML覆盖率报告
3. 访问Codecov查看详细报告

### 性能回归处理

1. 如果检测到性能回归，PR中会显示警告
2. 检查基准测试结果
3. 优化性能热点
4. 如果回归是预期的，更新性能基线

## 最佳实践

1. **定期运行性能测试**
   - 每天自动运行性能回归检测
   - 及时发现性能问题

2. **保持覆盖率**
   - 目标覆盖率80%+
   - 新代码应包含测试

3. **版本发布**
   - 使用语义化版本（SemVer）
   - 更新CHANGELOG
   - 创建Release说明

4. **性能基线**
   - 定期更新性能基线
   - 记录性能改进

## 故障排除

### 发布失败

- 检查版本标签格式
- 确保有发布权限
- 检查Crates.io token

### 覆盖率报告不生成

- 检查cargo-llvm-cov安装
- 确保测试能够运行
- 检查Codecov配置

### 性能回归检测失败

- 检查性能基线文件
- 确保基准测试能够运行
- 检查性能阈值设置

## 更多信息

- [GitHub Actions文档](https://docs.github.com/en/actions)
- [Codecov文档](https://docs.codecov.com/)
- [Cargo发布指南](https://doc.rust-lang.org/cargo/reference/publishing.html)

