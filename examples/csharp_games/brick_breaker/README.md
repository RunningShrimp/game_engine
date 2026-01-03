# 2D Brick Breaker - 打砖块游戏

一个使用C#脚本系统构建的经典打砖块游戏示例。

## 🎮 游戏特性

- ✅ **C# 脚本组件**: 所有游戏逻辑用C#编写
- ✅ **物理碰撞**: 球和砖块、挡板的碰撞检测
- ✅ **分数系统**: 击碎砖块获得分数
- ✅ **音效反馈**: 击球、击碎砖块的音效
- ✅ **粒子效果**: 击碎砖块时的粒子爆炸
- ✅ **关卡系统**: 多个难度递增的关卡
- ✅ **生命系统**: 3条命，球掉落会损失生命

## 🚀 运行游戏

### 前置要求

```bash
# 安装 .NET SDK 8.0
# macOS
brew install --cask dotnet-sdk

# 验证安装
dotnet --version  # 应该显示 8.0.x
```

### 编译运行

```bash
cd examples/csharp_games/brick_breaker

# 编译Rust主程序
cargo build --release --features csharp

# 运行游戏
cargo run --release --features csharp
```

## 🎯 游戏操作

- **← →** 或 **A D**: 移动挡板
- **空格键**: 发射球/暂停游戏
- **R**: 重新开始
- **ESC**: 退出游戏

## 📁 文件结构

```
brick_breaker/
├── scripts/
│   ├── Components/
│   │   ├── Ball.cs           # 球组件
│   │   ├── Paddle.cs         # 挡板组件
│   │   ├── Brick.cs          # 砖块组件
│   │   └── GameManager.cs    # 游戏管理器
│   ├── Systems/
│   │   ├── PhysicsSystem.cs  # 物理系统
│   │   ├── CollisionSystem.cs # 碰撞系统
│   │   └── ScoreSystem.cs    # 分数系统
│   └── Game/
│       └── GameLogic.cs      # 游戏主逻辑
├── assets/
│   ├── textures/             # 纹理资源
│   └── sounds/               # 音频资源
├── src/
│   └── main.rs               # Rust主程序
├── Cargo.toml
└── README.md
```

## 🔧 C# 脚本详解

### Ball.cs - 球组件

```csharp
using GameEngine;
using GameEngine.ECS;

public class Ball : Component
{
    public Vector2 Velocity;
    public float Speed = 300.0f;

    public void Update(float deltaTime)
    {
        // 移动球
        Transform.Position += new Vector3(Velocity.X, Velocity.Y, 0) * Speed * deltaTime;

        // 墙壁碰撞
        if (Transform.Position.X < -8 || Transform.Position.X > 8)
            Velocity.X = -Velocity.X;

        if (Transform.Position.Y > 6)
            Velocity.Y = -Velocity.Y;

        // 球掉落底部
        if (Transform.Position.Y < -6)
        {
            GameManager.Instance.LoseLife();
        }
    }
}
```

### Paddle.cs - 挡板组件

```csharp
using GameEngine;
using GameEngine.ECS;

public class Paddle : Component
{
    public float Speed = 10.0f;
    public float Width = 2.0f;

    public void Update(float deltaTime)
    {
        // 获取输入
        float horizontal = Input.GetAxis("Horizontal");

        // 移动挡板
        Vector3 pos = Transform.Position;
        pos.x += horizontal * Speed * deltaTime;

        // 限制在屏幕范围内
        pos.x = Mathf.Clamp(pos.x, -7.5f + Width / 2, 7.5f - Width / 2);

        Transform.Position = pos;
    }
}
```

### Brick.cs - 砖块组件

```csharp
using GameEngine;
using GameEngine.ECS;

public class Brick : Component
{
    public int Points = 10;
    public int Hits = 1;
    public Color Color;

    public void OnHit()
    {
        Hits--;

        if (Hits <= 0)
        {
            // 播放破碎音效
            Audio.PlaySound("brick_break");

            // 创建粒子效果
            ParticleSystem.Spawn(Transform.Position, Color, 20);

            // 增加分数
            GameManager.Instance.AddScore(Points);

            // 销毁砖块
            World.DestroyEntity(Entity);
        }
        else
        {
            // 改变颜色表示受损
            Color.a *= 0.7f;
        }
    }
}
```

### GameManager.cs - 游戏管理器

```csharp
using GameEngine;
using GameEngine.ECS;

public class GameManager : Component
{
    public static GameManager Instance { get; private set; }

    public int Score { get; private set; }
    public int Lives { get; private set; }
    public int Level { get; private set; }

    private Ball ball;
    private Paddle paddle;
    private Brick[] bricks;

    public void Awake()
    {
        Instance = this;
        Lives = 3;
        Score = 0;
        Level = 1;
    }

    public void Start()
    {
        SpawnBall();
        CreateLevel();
    }

    public void AddScore(int points)
    {
        Score += points;
        UI.UpdateScore(Score);

        // 检查是否过关
        if (AllBricksDestroyed())
        {
            NextLevel();
        }
    }

    public void LoseLife()
    {
        Lives--;
        UI.UpdateLives(Lives);

        if (Lives <= 0)
        {
            GameOver();
        }
        else
        {
            ResetBall();
        }
    }

    public void NextLevel()
    {
        Level++;
        CreateLevel();
        ResetBall();
        UI.ShowMessage($"Level {Level}");
    }

    private void SpawnBall()
    {
        // 创建球实体
        var ballEntity = World.CreateEntity();
        ball = ballEntity.AddComponent<Ball>();
        ball.Velocity = new Vector2(1, 1).normalized;
    }

    private void CreateLevel()
    {
        // 根据关卡创建砖块布局
        int rows = Mathf.Min(3 + Level, 8);
        int cols = 10;

        for (int row = 0; row < rows; row++)
        {
            for (int col = 0; col < cols; col++)
            {
                var brickEntity = World.CreateEntity();
                var brick = brickEntity.AddComponent<Brick>();

                // 设置砖块属性
                brick.Points = (rows - row) * 10;
                brick.Color = GetRowColor(row);
                brick.Hits = row < 2 ? 2 : 1; // 前两行需要打两次

                // 设置位置
                Vector3 pos = new Vector3(
                    (col - cols / 2.0f + 0.5f) * 1.5f,
                    4.0f - row * 0.6f,
                    0
                );
                brick.Transform.Position = pos;
            }
        }
    }

    private Color GetRowColor(int row)
    {
        Color[] colors = {
            new Color(1, 0, 0),     // 红
            new Color(1, 0.5f, 0),   // 橙
            new Color(1, 1, 0),     // 黄
            new Color(0, 1, 0),     // 绿
            new Color(0, 0, 1),     // 蓝
            new Color(0.5f, 0, 1),   // 紫
            new Color(1, 0, 1),     // 品红
            new Color(0.5f, 0, 0.5f) // 棕
        };

        return colors[row % colors.Length];
    }

    private bool AllBricksDestroyed()
    {
        return World.FindEntitiesOfType<Brick>().Length == 0;
    }

    private void ResetBall()
    {
        ball.Transform.Position = new Vector3(0, -4, 0);
        ball.Velocity = new Vector2(0, 1);
    }

    private void GameOver()
    {
        UI.ShowGameOver(Score);
        Time.timeScale = 0;
    }
}
```

## 🎓 学习要点

### 1. 组件系统

每个游戏对象都是一个Entity，通过添加Component来赋予功能：

```csharp
// 创建实体
var entity = World.CreateEntity();

// 添加组件
var ball = entity.AddComponent<Ball>();

// 获取组件
var ball = entity.GetComponent<Ball>();

// 移除组件
entity.RemoveComponent<Ball>();
```

### 2. 生命周期方法

组件可以定义生命周期方法：

```csharp
public class MyComponent : Component
{
    public void Awake()       // 组件创建时调用
    public void Start()       // 第一帧前调用
    public void Update(float deltaTime)  // 每帧调用
    public void OnDestroy()   // 销毁时调用
}
```

### 3. 输入处理

```csharp
// 获取轴输入
float horizontal = Input.GetAxis("Horizontal");

// 获取按键
if (Input.GetKey(KeyCode.Space))
{
    // 执行动作
}

// 获取按键按下
if (Input.GetKeyDown(KeyCode.R))
{
    // 重新开始
}
```

### 4. 物理碰撞

使用Physics系统进行碰撞检测：

```csharp
public class CollisionSystem : System
{
    public void Update(World world, float deltaTime)
    {
        var balls = world.FindEntitiesOfType<Ball>();
        var bricks = world.FindEntitiesOfType<Brick>();

        foreach (var ball in balls)
        {
            foreach (var brick in bricks)
            {
                if (CheckCollision(ball, brick))
                {
                    brick.OnHit();
                    HandleBallCollision(ball, brick);
                }
            }
        }
    }
}
```

## 🔍 代码示例

### 创建自定义砖块类型

```csharp
public class ExplosiveBrick : Brick
{
    public float ExplosionRadius = 2.0f;

    public void OnHit()
    {
        // 找到附近的砖块
        var nearbyBricks = World.FindEntitiesOfType<Brick>()
            .Where(b => Vector3.Distance(b.Transform.Position, Transform.Position) < ExplosionRadius);

        // 炸毁它们
        foreach (var brick in nearbyBricks)
        {
            World.DestroyEntity(brick.Entity);
        }

        // 播放爆炸效果
        ParticleSystem.SpawnExplosion(Transform.Position);
    }
}
```

### 添加道具系统

```csharp
public class PowerUp : Component
{
    public PowerUpType Type;

    public void Update(float deltaTime)
    {
        Transform.Position += Vector3.Down * deltaTime;

        // 检测与挡板的碰撞
        if (CheckCollisionWithPaddle())
        {
            ApplyPowerUp();
            World.DestroyEntity(Entity);
        }
    }

    private void ApplyPowerUp()
    {
        switch (Type)
        {
            case PowerUpType.WidenPaddle:
                Paddle.Width *= 1.5f;
                break;
            case PowerUpType.MultiBall:
                SpawnExtraBalls(2);
                break;
            case PowerUpType.SlowBall:
                Ball.Speed *= 0.7f;
                break;
        }
    }
}
```

## 🐛 常见问题

### Q: C# 脚本没有加载？
A: 确保 `--features csharp` 已启用，并检查 .NET SDK 是否安装：

```bash
dotnet --version
```

### Q: 球的速度太快或太慢？
A: 调整 `Ball.cs` 中的 `Speed` 属性：

```csharp
public float Speed = 300.0f;  // 调整这个值
```

### Q: 想修改关卡布局？
A: 修改 `GameManager.CreateLevel()` 方法中的砖块生成逻辑。

## 📚 扩展建议

1. **添加道具系统**: 击碎特殊砖块掉落道具
2. **多种球类型**: 火球（穿透砖块）、冰球（减速）
3. **挡板类型**: 发光挡板、粘性挡板
4. **关卡编辑器**: 可视化编辑关卡布局
5. **多人模式**: 双人合作或对战

## 📝 许可证

MIT License - 可自由学习和修改
