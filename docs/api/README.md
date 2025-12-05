# API 参考文档

本文档提供游戏引擎的完整 API 参考。

## 目录结构

- [核心模块](./core.md) - 引擎核心功能
- [渲染系统](./render.md) - 2D/3D 渲染
- [物理系统](./physics.md) - 物理模拟
- [音频系统](./audio.md) - 音频播放
- [动画系统](./animation.md) - 动画播放
- [网络系统](./network.md) - 多人游戏支持
- [XR 系统](./xr.md) - VR/AR 支持
- [编辑器](./editor.md) - 编辑器工具

## 快速开始

### 生成文档

使用 `cargo doc` 生成完整的 API 文档：

```bash
# 生成文档
cargo doc --no-deps --open

# 生成文档（包含私有项）
cargo doc --document-private-items --no-deps --open
```

### 查看在线文档

文档已发布到 [docs.rs](https://docs.rs/game_engine)。

## 文档规范

所有公共 API 都应包含：
- 模块级文档（`//!`）
- 类型文档（`///`）
- 函数文档（`///`）
- 使用示例（`# Example`）
- 错误说明（`# Errors`）
- 性能说明（`# Performance`）

## 贡献

添加新 API 时，请确保：
1. 添加完整的文档注释
2. 包含使用示例
3. 说明错误情况
4. 更新本文档

