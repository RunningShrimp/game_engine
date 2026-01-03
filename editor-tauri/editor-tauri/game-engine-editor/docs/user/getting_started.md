# 快速入门指南
# Getting Started Guide

**版本**: v0.3.0
**预计学习时间**: 10分钟
**最后更新**: 2026-01-03

---

## 📋 目录

1. [安装](#安装)
2. [创建第一个游戏](#创建第一个游戏)
3. [游戏引擎基础](#游戏引擎基础)
4. [添加游戏逻辑](#添加游戏逻辑)
5. [C#脚本](#c脚本)
6. [运行和调试](#运行和调试)
7. [下一步](#下一步)

---

## 安装

### 前置要求

- **Rust**: 1.70 或更高版本
- **.NET SDK**: 8.0（可选，用于C#脚本）
- **Git**: 版本控制
- **VS Code**: 推荐的代码编辑器

### 安装步骤

#### macOS

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装.NET SDK（可选）
brew install --cask dotnet-sdk

# 安装游戏引擎CLI
cargo install game-engine-cli

# 安装VS Code扩展
code --install-extension game-engine.game-engine-vscode
```

#### Windows

```powershell
# 安装Rust
# 访问 https://rustup.rs/ 下载并运行安装程序

# 安装.NET SDK（可选）
# 访问 https://dotnet.microsoft.com/download 下载并安装

# 安装游戏引擎CLI
cargo install game-engine-cli

# 安装VS Code扩展
code --install-extension game-engine.game-engine-vscode
```

#### Linux

```bash
# 安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装.NET SDK（可选）
wget https://dot.net/v1/dotnet-install.sh
chmod +x dotnet-install.sh
./dotnet-install.sh --channel 8.0

# 安装游戏引擎CLI
cargo install game-engine-cli

# 安装VS Code扩展
code --install-extension game-engine.game-engine-vscode
```

### 验证安装

```bash
# 检查Rust版本
rustc --version

# 检查引擎CLI版本
game-engine --version

# 检查.NET版本（如果安装）
dotnet --version
```

---

## 创建第一个游戏

### 使用CLI创建项目

```bash
# 创建新的3D游戏项目
game-engine new my-first-game --template 3d-game

# 进入项目目录
cd my-first-game
```

### 项目结构

```
my-first-game/
├── src/
│   ├── main.rs          # 游戏入口点
│   └── lib.rs           # 库入口
├── assets/              # 游戏资源
│   ├── models/          # 3D模型
│   ├── textures/        # 纹理贴图
│   └── audio/           # 音频文件
├── scripts/             # C#脚本（可选）
├── Cargo.toml           # Rust项目配置
├── game-engine.toml     # 引擎配置
└── README.md            # 项目说明
```

### 构建项目

```bash
# 调试模式构建（快速编译）
game-engine build

# 发布模式构建（优化性能）
game-engine build --release
```

---

## 游戏引擎基础

### ECS架构

游戏引擎使用实体组件系统（ECS）架构：

- **Entity（实体）**: 游戏对象的唯一标识
- **Component（组件）**: 数据（位置、旋转、纹理等）
- **System（系统）**: 逻辑（移动、渲染、物理等）

### 基本概念

```rust
use game_engine::prelude::*;

fn main() {
    // 创建游戏引擎实例
    let mut engine = GameEngine::new();

    // 创建场景
    let mut scene = Scene::new("My Scene");

    // 创建实体
    let player = Entity::new("Player");

    // 添加组件
    player.add_component(Transform::default());
    player.add_component(Sprite::new("player.png"));

    // 添加到场景
    scene.add_entity(player);

    // 运行游戏
    engine.run(scene);
}
```

### 常用组件

| 组件 | 描述 | 用途 |
|------|------|------|
| `Transform` | 位置、旋转、缩放 | 所有游戏对象 |
| `Sprite` | 2D精灵渲染 | 2D游戏 |
| `Mesh` | 3D网格渲染 | 3D游戏 |
| `RigidBody` | 物理模拟 | 物理对象 |
| `Collider` | 碰撞检测 | 可碰撞对象 |
| `AudioSource` | 音频播放 | 音效 |
| `Light` | 光照 | 3D场景 |
| `Camera` | 视口控制 | 渲染视图 |

---

## 添加游戏逻辑

### 游戏循环

```rust
use game_engine::prelude::*;

struct GameState {
    player: Entity,
    score: u32,
}

impl GameLoop for GameState {
    fn update(&mut self, ctx: &mut Context) {
        // 游戏逻辑更新
        self.handle_input(ctx);
        self.update_physics(ctx);
    }

    fn render(&mut self, ctx: &mut Context) {
        // 渲染游戏画面
        ctx.draw_sprite(&self.player);
    }
}

fn main() {
    let mut game = GameState {
        player: Entity::new("Player"),
        score: 0,
    };

    run_game(game);
}
```

### 输入处理

```rust
fn handle_input(&mut self, ctx: &mut Context) {
    let input = ctx.input();

    // 键盘输入
    if input.is_key_down(KeyCode::W) {
        self.move_player(Vector3::FORWARD);
    }
    if input.is_key_down(KeyCode::S) {
        self.move_player(Vector3::BACK);
    }

    // 鼠标输入
    if input.is_mouse_button_down(MouseButton::Left) {
        self.shoot();
    }

    // 手柄输入
    if let Some(gamepad) = input.gamepad(0) {
        let left_stick = gamepad.left_stick();
        if left_stick.x > 0.1 {
            self.move_player(Vector3::RIGHT);
        }
    }
}
```

### 简单的移动逻辑

```rust
fn move_player(&mut self, direction: Vector3) {
    let transform = self.player.get_component::<Transform>().unwrap();
    let speed = 5.0 * delta_time();
    transform.translate(direction * speed);
}
```

### 碰撞检测

```rust
fn check_collisions(&mut self, ctx: &mut Context) {
    // 查询所有碰撞体
    let collisions = ctx.world().query::<(&Collider, &Transform)>();

    for (collider1, transform1) in collisions.iter() {
        for (collider2, transform2) in collisions.iter() {
            if collider1.intersects(collider2, transform1, transform2) {
                self.on_collision(collider1, collider2);
            }
        }
    }
}

fn on_collision(&mut self, a: &Collider, b: &Collider) {
    println!("Collision between {:?} and {:?}", a, b);
}
```

---

## C#脚本

### 为什么使用C#？

- 快速迭代，无需重新编译
- 丰富的库支持
- 易于学习和使用

### 创建C#脚本

在 `scripts/` 目录创建 `player_controller.cs`:

```csharp
using GameEngine;
using GameEngine.Math;

public class PlayerController : MonoBehaviour
{
    // 公共字段在编辑器中可配置
    public float speed = 5.0f;
    public Vector3 velocity;

    // Start方法在游戏开始时调用一次
    void Start()
    {
        velocity = Vector3.Zero;
    }

    // Update方法每帧调用
    void Update()
    {
        // 获取输入
        if (Input.GetKey(KeyCode.W))
        {
            velocity.z = 1.0f;
        }
        else if (Input.GetKey(KeyCode.S))
        {
            velocity.z = -1.0f;
        }
        else
        {
            velocity.z = 0.0f;
        }

        // 移动玩家
        Transform.Position += velocity * speed * Time.deltaTime;
    }

    // OnTriggerEnter在碰撞时调用
    void OnTriggerEnter(Collider other)
    {
        if (other.CompareTag("Enemy"))
        {
            TakeDamage(10);
        }
    }

    void TakeDamage(int damage)
    {
        // 处理伤害逻辑
    }
}
```

### 在Rust中加载C#脚本

```rust
use game_engine::scripting::CSharpRuntime;

fn main() {
    let mut engine = GameEngine::new();
    let mut csharp_runtime = CSharpRuntime::new();

    // 加载C#脚本
    csharp_runtime.load_script("scripts/player_controller.cs");

    // 运行游戏
    engine.run_with_scripting(csharp_runtime);
}
```

---

## 运行和调试

### 运行游戏

```bash
# 调试模式运行
game-engine run

# 发布模式运行
game-engine run --release

# 全屏运行
game-engine run -- --fullscreen

# 禁用垂直同步
game-engine run -- --no-vsync
```

### 调试

#### VS Code调试

创建 `.vscode/launch.json`:

```json
{
    "version": "0.2.0",
    "configurations": [
        {
            "type": "lldb",
            "request": "launch",
            "name": "Debug Game",
            "cargo": {
                "args": [
                    "build",
                    "--package=my-first-game"
                ],
                "filter": {
                    "name": "my-first-game",
                    "kind": "bin"
                }
            },
            "args": [],
            "cwd": "${workspaceFolder}"
        }
    ]
}
```

#### 日志输出

```rust
use game_engine::log::*;

fn main() {
    // 初始化日志
    init_log();

    info!("游戏启动");
    debug!("调试信息");
    warn!("警告信息");
    error!("错误信息");

    run_game();
}
```

#### 性能分析

```bash
# 启用性能分析
game-engine run --profile

# 生成火焰图
game-engine run --flamegraph
```

---

## 下一步

### 教程

1. **创建2D平台游戏** - 学习2D游戏开发基础
2. **创建3D射击游戏** - 学习3D游戏开发
3. **添加物理系统** - 学习物理模拟
4. **添加AI敌人** - 学习AI导航
5. **多人游戏** - 学习网络同步

### 示例项目

- `examples/2d-platformer` - 2D平台游戏
- `examples/3d-fps` - 3D第一人称射击
- `examples/racing-game` - 赛车游戏
- `examples/puzzle-game` - 益智游戏

### 文档

- [API文档](../api/) - 完整的API参考
- [教程文档](./tutorials.md) - 详细教程
- [最佳实践](./best_practices.md) - 开发建议
- [故障排除](./troubleshooting.md) - 常见问题

### 社区

- **Discord**: [discord.gg/game-engine](https://discord.gg/game-engine)
- **GitHub**: [github.com/game-engine/game-engine](https://github.com/game-engine/game-engine)
- **论坛**: [forum.game-engine.dev](https://forum.game-engine.dev)

---

## 常见问题

### Q: 如何学习Rust？

A: 推荐《Rust程序设计语言》和《通过例子学Rust》：
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)

### Q: 性能如何？

A: 引擎针对现代硬件进行了优化：
- 10000+实体流畅运行（60+ FPS）
- 低延迟输入响应（<16ms）
- 高效的内存管理

### Q: 支持哪些平台？

A: 跨平台支持：
- **桌面**: Windows、macOS、Linux
- **Web**: 通过WASM
- **移动**: Android、iOS
- **游戏机**: Nintendo Switch、PlayStation、Xbox

### Q: 如何发布游戏？

A: 使用发布构建和资源打包：
```bash
# 构建发布版本
game-engine build --release

# 打包资源
game-engine asset bundle --compression=high

# 创建安装包
game-engine package --platform=windows
```

### Q: 可以商用吗？

A: 可以！引擎使用MIT许可证，允许商业使用。

---

## 进阶主题

### 自定义组件

```rust
#[derive(Component)]
struct Health {
    current: u32,
    max: u32,
}

impl Health {
    fn new(max: u32) -> Self {
        Self {
            current: max,
            max,
        }
    }

    fn take_damage(&mut self, amount: u32) {
        self.current = self.current.saturating_sub(amount);
    }
}
```

### 自定义系统

```rust
#[system]
fn health_regeneration(world: &mut World, delta_time: f32) {
    let mut query = world.query::<&mut Health>();

    for health in query.iter_mut() {
        if health.current < health.max {
            health.current = (health.current + 1).min(health.max);
        }
    }
}
```

### 资源热重载

```toml
[game-engine]
hot_reload = true
watch_assets = true
```

---

**恭喜！** 你已经掌握了游戏引擎的基础知识。现在可以开始创建你的第一个游戏了！

**祝开发愉快！** 🎮✨

---

**文档版本**: v1.0
**最后更新**: 2026-01-03
**维护者**: Game Engine Team
