# Game Engine Examples

本目录包含游戏引擎的各种示例项目，展示引擎的核心功能和使用方法。

## 运行示例

使用以下命令运行示例：

```bash
# Hello World 示例
cargo run --example hello_world

# 渲染示例
cargo run --example rendering

# 物理示例
cargo run --example physics

# 动画示例
cargo run --example animation

# 多人游戏示例
cargo run --example multiplayer
```

## 示例说明

### hello_world
最简单的引擎使用示例，展示如何：
- 初始化引擎
- 创建实体
- 运行主循环

### rendering
展示高级渲染功能：
- PBR材质
- 相机设置
- 后处理效果

### physics
展示物理系统：
- 刚体物理
- 碰撞检测
- 物理模拟

### animation
展示动画系统：
- 关键帧动画
- 骨骼动画
- 动画播放

### multiplayer
展示多人游戏功能：
- 网络同步
- 客户端预测
- 服务器权威

## 完整游戏示例

`examples/game/` 目录包含一个完整的游戏示例，展示如何组合使用引擎的各种功能。

