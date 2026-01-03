# C# 游戏示例集合

这个目录包含了使用游戏引擎C#脚本系统创建的完整游戏示例。

## 🎮 示例列表

### 1. 2D Brick Breaker（打砖块游戏）
**目录**: `brick_breaker/`
**难度**: ⭐ 入门
**特性**:
- C# 脚本组件
- 碰撞检测
- 分数系统
- 音效反馈
- 粒子效果

**运行**:
```bash
cd brick_breaker
cargo run --features csharp
```

### 2. 3D First Person Shooter（第一人称射击）
**目录**: `fps_demo/`
**难度**: ⭐⭐⭐ 中级
**特性**:
- 3D 渲染
- 第一人称控制器
- 武器系统
- 敌人AI
- 生命值系统

**运行**:
```bash
cd fps_demo
cargo run --features csharp
```

### 3. Multiplayer Tank Battle（多人坦克对战）
**目录**: `tank_battle/`
**难度**: ⭐⭐⭐⭐ 高级
**特性**:
- 网络同步
- 多人对战
- 实时状态复制
- RPC调用
- 延迟补偿

**运行**:
```bash
# 服务器
cd tank_battle
cargo run --bin server --features csharp,networking

# 客户端
cargo run --bin client --features csharp,networking
```

## 📚 学习路径

1. **从Brick Breaker开始**: 学习基础的C#脚本组件和游戏循环
2. **进阶到FPS Demo**: 理解3D渲染、控制器和AI系统
3. **挑战Tank Battle**: 掌握网络编程和多人游戏同步

## 🔧 前置要求

- Rust 1.70+
- .NET SDK 8.0+
- 游戏引擎依赖

### 安装 .NET SDK

```bash
# macOS
brew install --cask dotnet-sdk

# Linux (Ubuntu)
wget https://packages.microsoft.com/config/ubuntu/20.04/packages-microsoft-prod.deb -O packages-microsoft-prod.deb
sudo dpkg -i packages-microsoft-prod.deb
sudo apt-get update
sudo apt-get install -y dotnet-sdk-8.0

# Windows
# 下载安装程序: https://dotnet.microsoft.com/download
```

## 📖 代码结构

每个游戏示例遵循以下结构：

```
game_name/
├── scripts/           # C# 脚本文件
│   ├── Components/   # 组件定义
│   ├── Systems/      # 系统逻辑
│   └── Game/         # 游戏逻辑
├── assets/           # 游戏资源
│   ├── textures/
│   ├── models/
│   └── sounds/
├── src/              # Rust 主程序
│   └── main.rs
├── Cargo.toml        # Rust 依赖
└── README.md         # 游戏说明
```

## 🎯 C# 脚本基础

### 创建组件

```csharp
using GameEngine;
using GameEngine.ECS;

public class PlayerController : Component
{
    public float Speed = 5.0f;

    public void Update(float deltaTime)
    {
        // 获取输入
        var input = Input.GetAxis("Horizontal");

        // 移动
        Transform.Position += new Vector3(input * Speed * deltaTime, 0, 0);
    }
}
```

### 创建系统

```csharp
using GameEngine;
using GameEngine.ECS;

public class PhysicsSystem : System
{
    public void Update(World world, float deltaTime)
    {
        // 查询所有有物理组件的实体
        var query = world.Query<Transform, Rigidbody>();

        foreach (var (transform, rigidbody) in query)
        {
            // 应用物理
            rigidbody.Velocity += Vector3.Down * 9.81f * deltaTime;
            transform.Position += rigidbody.Velocity * deltaTime;
        }
    }
}
```

## 🚀 性能提示

1. **使用编译缓存**: 脚本首次编译后会缓存，后续加载更快
2. **避免频繁的跨边界调用**: 尽量减少Rust和C#之间的调用次数
3. **使用对象池**: 重用游戏对象而不是频繁创建销毁
4. **启用JIT**: 使用JIT编译可以提高脚本性能

## 🐛 调试技巧

### 启用详细日志

```rust
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("game_engine=debug,csharp=trace")
        .init();
}
```

### 查看C#异常

```csharp
try {
    // 你的代码
} catch (Exception ex) {
    Debug.LogError($"C# Error: {ex.Message}");
    Debug.LogError($"Stack Trace: {ex.StackTrace}");
}
```

## 📚 更多资源

- [C# SDK文档](../docs/csharp_sdk.md)
- [API参考](../docs/api_reference.md)
- [教程](../docs/tutorials/)
- [社区示例](https://github.com/game-engine/examples)

## 💡 贡献

欢迎提交新的游戏示例！请确保：
- 代码清晰易懂
- 包含详细注释
- 提供README说明
- 遵循项目代码规范

## 📄 许可证

这些示例使用与游戏引擎相同的许可证。
