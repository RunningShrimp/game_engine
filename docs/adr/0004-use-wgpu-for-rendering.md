# ADR-0004: 使用wgpu进行渲染

**状态**: 已采用  
**日期**: 2025-12-01  
**决策者**: 架构团队

## 上下文

游戏引擎需要跨平台渲染支持（Windows、macOS、Linux、Web），需要一个统一的渲染API。

## 决策

使用 `wgpu` 作为渲染后端：
- wgpu是Rust的WebGPU实现
- 支持多平台（Vulkan、Metal、DirectX 12、WebGPU）
- 提供统一的API，无需平台特定代码
- 性能接近原生API

## 后果

### 优点
- **跨平台**: 一套代码支持多个平台
- **现代化**: 使用现代GPU API（WebGPU）
- **性能**: 接近原生API性能
- **安全性**: Rust类型系统保证内存安全

### 缺点
- **API限制**: WebGPU API可能不如原生API灵活
- **学习曲线**: 团队需要学习WebGPU概念
- **调试**: 跨平台调试可能更困难

### 实施细节
- 渲染系统位于 `src/render/` 目录
- 使用 `wgpu` crate 进行渲染
- 支持PBR渲染、后处理效果等

## 替代方案

1. **Vulkan**: 使用原生Vulkan API
   - 拒绝原因: 需要平台特定代码，维护成本高

2. **OpenGL**: 使用OpenGL
   - 拒绝原因: 已过时，性能较差

3. **DirectX 12**: 使用DirectX 12
   - 拒绝原因: 仅支持Windows

## 相关决策

- ADR-0005: 使用Bevy ECS进行实体管理



