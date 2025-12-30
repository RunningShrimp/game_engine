# 游戏引擎 (Game Engine)

<div align="center">

**A High-Performance Cross-Platform 2D/3D Game Engine Built with Rust**

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/badge/Tests-525%2B-brightgreen.svg)](tests/)
[![Coverage](https://img.shields.io/badge/Coverage-75%25-brightgreen.svg)](docs/)

[Quick Start](#quick-start) • [Features](#features) • [Performance](#performance) • [Contributing](CONTRIBUTING.md)

</div>

## 简介

一个用Rust构建的高性能跨平台2D/3D游戏引擎，注重性能、安全性和易用性。

### 核心特性

- 🎮 ECS架构（基于bevy_ecs）
- 🚀 现代渲染（wgpu）
- ⚡ 性能优化（parking_lot 2.5x-8x, DashMap 10x-20x）
- 🔊 完整音频（3D空间音频）
- 🧠 AI系统（行为树、寻路）
- 🌐 网络支持（WebSocket、UDP）

## 快速开始

```bash
# 克隆仓库
git clone https://github.com/username/game_engine.git
cd game_engine

# 运行示例
cargo run --example performance_examples

# 运行测试
cargo test --workspace
```

**详细指南**: [QUICKSTART.md](QUICKSTART.md)

## 功能

| 功能 | 描述 | 状态 |
|------|------|------|
| ECS | 实体组件系统 | ✅ |
| 渲染 | wgpu跨平台 | ✅ |
| 物理 | Rapier 2D/3D | ✅ |
| 音频 | 3D空间音频 | ✅ |
| AI | 行为树、寻路 | ✅ |
| 网络 | WebSocket、UDP | ✅ |
| 脚本 | Lua、WASM | ✅ |

## 性能

### 基准测试

```
parking_lot::RwLock:   40ns  (2.5x faster)
std::sync::RwLock:     100ns

DashMap:                100ns (10x faster)
Mutex<HashMap>:        1,000ns
```

**详细报告**: [性能优化报告](docs/PERFORMANCE_BEST_PRACTICES.md)

## 文档

### 核心文档
- [快速开始](QUICKSTART.md)
- [贡献指南](CONTRIBUTING.md)
- [变更日志](CHANGELOG.md)

### 优化文档
- [优化指南](docs/OPTIMIZATION_GUIDE.md) - 综合优化策略和最佳实践
- [性能最佳实践](docs/PERFORMANCE_BEST_PRACTICES.md) - 性能优化详细指南
- [优化状态](docs/OPTIMIZATION_STATUS.md) - 优化进度和性能指标

### 项目文档
- [完成报告](docs/FINAL_COMPLETION_REPORT.md) - 项目完成总结
- [维护计划](docs/MAINTENANCE_PLAN.md) - 维护和升级计划
- [故障排除](docs/TROUBLESHOOTING_GUIDE.md) - 常见问题解决

## 贡献

欢迎贡献！详见[CONTRIBUTING.md](CONTRIBUTING.md)

## 许可证

MIT OR Apache-2.0

---

<div align="center">

**Built with ❤️ and Rust**

</div>
