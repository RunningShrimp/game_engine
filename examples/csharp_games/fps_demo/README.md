# 3D FPS Demo - 第一人称射击演示

一个使用C#脚本系统构建的3D第一人称射击游戏示例，展示游戏引擎的3D渲染、物理和网络能力。

## 🎮 游戏特性

- ✅ **3D渲染**: 完整的3D场景和模型
- ✅ **第一人称控制器**: WASD移动 + 鼠标视角
- ✅ **武器系统**: 多种武器，射击和换弹
- ✅ **敌人AI**: 基础的敌人AI行为
- ✅ **生命值系统**: 玩家受伤和医疗
- ✅ **物理模拟**: 射弹道、碰撞检测
- ✅ **音效系统**: 3D空间音效

## 🚀 运行游戏

### 前置要求

```bash
# 安装 .NET SDK 8.0
dotnet --version  # 应该显示 8.0.x
```

### 编译运行

```bash
cd examples/csharp_games/fps_demo

# 编译Rust主程序
cargo build --release --features csharp

# 运行游戏
cargo run --release --features csharp
```

## 🎯 游戏操作

### 基础控制

- **W A S D**: 移动
- **鼠标**: 视角控制
- **Space**: 跳跃
- **Shift**: 冲刺
- **Ctrl**: 蹲下

### 武器操作

- **鼠标左键**: 射击
- **R**: 换弹
- **1/2/3**: 切换武器
- **滚轮**: 武器滚动

### 游戏功能

- **Esc**: 暂停菜单
- **Tab**: 记分板
- **F1**: 帮助信息

## 📁 文件结构

```
fps_demo/
├── scripts/
│   ├── Components/
│   │   ├── PlayerController.cs  # 玩家控制器
│   │   ├── FirstPersonCamera.cs  # 第一人称相机
│   │   ├── Weapon.cs             # 武器系统
│   │   ├── Health.cs             # 生命值组件
│   │   ├── Enemy.cs              # 敌人AI
│   │   └── Bullet.cs             # 子弹
│   ├── Systems/
│   │   ├── WeaponSystem.cs       # 武器管理系统
│   │   ├── DamageSystem.cs       # 伤害计算系统
│   │   ├── AINavigationSystem.cs # AI导航系统
│   │   └── PhysicsSystem.cs      # 物理模拟系统
│   └── Game/
│       ├── GameMode.cs           # 游戏模式
│       ├── GameRules.cs          # 游戏规则
│       └── SpawnSystem.cs        # 生成系统
├── assets/
│   ├── models/                   # 3D模型
│   ├── textures/                 # 纹理
│   └── sounds/                   # 音效
├── src/
│   └── main.rs                   # Rust主程序
├── Cargo.toml
└── README.md
```

## 🔧 C# 脚本详解

### PlayerController.cs - 玩家控制器

```csharp
using GameEngine;
using GameEngine.ECS;

public class PlayerController : Component
{
    [Header("Movement")]
    public float WalkSpeed = 5.0f;
    public float SprintSpeed = 8.0f;
    public float JumpForce = 5.0f;

    [Header("Mouse Look")]
    public float MouseSensitivity = 100.0f;
    public float MaxPitchAngle = 89.0f;

    private CharacterController controller;
    private FirstPersonCamera camera;
    private float pitch = 0f;
    private float yaw = 0f;

    public void Start()
    {
        controller = GetComponent<CharacterController>();
        camera = FindObjectOfType<FirstPersonCamera>();

        // 锁定鼠标
        Cursor.LockState = CursorLockMode.Locked;
        Cursor.Visible = false;
    }

    public void Update(float deltaTime)
    {
        HandleMovement();
        HandleMouseLook();
        HandleJump();
    }

    private void HandleMovement()
    {
        // 获取输入
        Vector3 moveDirection = Vector3.Zero;

        if (Input.GetKey(KeyCode.W)) moveDirection += Vector3.Forward;
        if (Input.GetKey(KeyCode.S)) moveDirection += Vector3.Back;
        if (Input.GetKey(KeyCode.A)) moveDirection += Vector3.Left;
        if (Input.GetKey(KeyCode.D)) moveDirection += Vector3.Right;

        // 应用冲刺
        float speed = Input.GetKey(KeyCode.LeftShift) ? SprintSpeed : WalkSpeed;

        // 移动
        controller.Move(moveDirection.normalized * speed * deltaTime);
    }

    private void HandleMouseLook()
    {
        // 鼠标输入
        float mouseX = Input.GetAxis("Mouse X") * MouseSensitivity;
        float mouseY = Input.GetAxis("Mouse Y") * MouseSensitivity;

        // 水平旋转（身体）
        yaw += mouseX * Time.deltaTime;

        // 垂直旋转（头部）
        pitch -= mouseY * Time.deltaTime;
        pitch = Mathf.Clamp(pitch, -MaxPitchAngle, MaxPitchAngle);

        // 应用旋转
        Transform.rotation = Quaternion.Euler(0, yaw, 0);
        camera.Transform.localRotation = Quaternion.Euler(pitch, 0, 0);
    }

    private void HandleJump()
    {
        if (Input.GetKeyDown(KeyCode.Space) && controller.IsGrounded)
        {
            controller.Jump(JumpForce);
        }
    }
}
```

### Weapon.cs - 武器系统

```csharp
using GameEngine;
using GameEngine.ECS;
using System.Collections;

public class Weapon : Component
{
    public enum WeaponType
    {
        Pistol,
        Rifle,
        Shotgun,
        Sniper
    }

    public WeaponType Type;

    [Header("Stats")]
    public int Damage = 10;
    public float FireRate = 0.1f;
    public int MagazineSize = 30;
    public int ReserveAmmo = 120;

    [Header("Recoil")]
    public float RecoilForce = 0.5f;
    public float RecoilRecovery = 5.0f;

    private int currentAmmo;
    private int currentReserve;
    private float lastFireTime;
    private float currentRecoil;

    public void Awake()
    {
        currentAmmo = MagazineSize;
        currentReserve = ReserveAmmo;
    }

    public void Update(float deltaTime)
    {
        // 自动射击
        if (Input.GetMouseButton(0))
        {
            TryFire();
        }

        // 手动装弹
        if (Input.GetKeyDown(KeyCode.R))
        {
            Reload();
        }

        // 恢复后坐力
        currentRecoil = Mathf.MoveTowards(currentRecoil, 0, RecoilRecovery * deltaTime);
    }

    public void TryFire()
    {
        // 检查射速
        if (Time.time - lastFireTime < FireRate)
            return;

        // 检查弹药
        if (currentAmmo <= 0)
        {
            Reload();
            return;
        }

        // 射击
        Fire();
        lastFireTime = Time.time;
        currentAmmo--;

        // 应用后坐力
        ApplyRecoil();
    }

    private void Fire()
    {
        // 创建子弹
        Bullet bullet = Instantiate(BulletPrefab,
            camera.Transform.position,
            camera.Transform.rotation);

        bullet.Damage = Damage;
        bullet.Owner = gameObject;

        // 播放射击音效
        Audio.PlayShotSound(Type);

        // 枪口火焰效果
        MuzzleFlash.Spawn();
    }

    private void ApplyRecoil()
    {
        currentRecoil += RecoilForce;

        // 应用后坐力到相机
        FirstPersonCamera camera = GetComponentInChildren<FirstPersonCamera>();
        if (camera != null)
        {
            camera.AddRecoil(currentRecoil);
        }
    }

    public void Reload()
    {
        if (currentReserve <= 0 || currentAmmo == MagazineSize)
            return;

        // 装弹动画
        Animator animator = GetComponent<Animator>();
        if (animator != null)
        {
            animator.SetTrigger("Reload");
        }

        // 等待装弹完成
        StartCoroutine(ReloadCoroutine());
    }

    private IEnumerator ReloadCoroutine()
    {
        yield return new WaitForSeconds(2.0f);

        // 计算装弹数量
        int needed = MagazineSize - currentAmmo;
        int available = Mathf.Min(needed, currentReserve);

        currentAmmo += available;
        currentReserve -= available;

        UI.UpdateAmmo(currentAmmo, currentReserve);
    }
}
```

### Enemy.cs - 敌人AI

```csharp
using GameEngine;
using GameEngine.ECS;
using System.Collections;

public class Enemy : Component
{
    [Header("Stats")]
    public int Health = 100;
    public int Damage = 10;
    public float MoveSpeed = 3.0f;
    public float DetectionRange = 20.0f;
    public float AttackRange = 5.0f;

    [Header("AI")]
    public float PatrolRadius = 10.0f;
    public Transform[] PatrolPoints;

    private enum State
    {
        Idle,
        Patrol,
        Chase,
        Attack,
        Dead
    }

    private State currentState = State.Idle;
    private Transform player;
    private int currentPatrolIndex = 0;
    private NavMeshAgent agent;

    public void Start()
    {
        agent = GetComponent<NavMeshAgent>();
        player = FindObjectOfType<PlayerController>()?.Transform;

        // 初始状态
        if (PatrolPoints.Length > 0)
        {
            currentState = State.Patrol;
        }
    }

    public void Update(float deltaTime)
    {
        if (Health <= 0)
        {
            Die();
            return;
        }

        switch (currentState)
        {
            case State.Idle:
                UpdateIdle();
                break;
            case State.Patrol:
                UpdatePatrol();
                break;
            case State.Chase:
                UpdateChase();
                break;
            case State.Attack:
                UpdateAttack();
                break;
        }
    }

    private void UpdateIdle()
    {
        // 检测玩家
        if (IsPlayerInRange())
        {
            currentState = State.Chase;
        }
    }

    private void UpdatePatrol()
    {
        // 移动到下一个巡逻点
        if (PatrolPoints.Length == 0)
            return;

        Transform targetPoint = PatrolPoints[currentPatrolIndex];
        agent.SetDestination(targetPoint.position);

        // 检查是否到达
        if (agent.ReachedDestination)
        {
            currentPatrolIndex = (currentPatrolIndex + 1) % PatrolPoints.Length;
        }

        // 检测玩家
        if (IsPlayerInRange())
        {
            currentState = State.Chase;
        }
    }

    private void UpdateChase()
    {
        if (player == null)
        {
            currentState = State.Patrol;
            return;
        }

        // 追逐玩家
        agent.SetDestination(player.position);

        // 检查攻击范围
        float distance = Vector3.Distance(Transform.position, player.position);
        if (distance <= AttackRange)
        {
            currentState = State.Attack;
        }
    }

    private void UpdateAttack()
    {
        // 面向玩家
        Transform.LookAt(player);

        // 攻击
        PlayerController playerController = player.GetComponent<PlayerController>();
        if (playerController != null)
        {
            playerController.TakeDamage(Damage * Time.deltaTime);
        }

        // 检查是否离开攻击范围
        float distance = Vector3.Distance(Transform.position, player.position);
        if (distance > AttackRange * 1.5f)
        {
            currentState = State.Chase;
        }
    }

    private bool IsPlayerInRange()
    {
        if (player == null)
            return false;

        float distance = Vector3.Distance(Transform.position, player.position);
        return distance <= DetectionRange;
    }

    public void TakeDamage(int damage)
    {
        Health -= damage;

        // 显示伤害效果
        StartCoroutine(FlashRed());

        if (Health <= 0)
        {
            Die();
        }
    }

    private IEnumerator FlashRed()
    {
        Renderer renderer = GetComponentInChildren<Renderer>();
        if (renderer != null)
        {
            renderer.material.color = Color.red;
            yield return new WaitForSeconds(0.1f);
            renderer.material.color = Color.white;
        }
    }

    private void Die()
    {
        currentState = State.Dead;

        // 播放死亡动画
        Animator animator = GetComponent<Animator>();
        if (animator != null)
        {
            animator.SetTrigger("Die");
        }

        // 销毁敌人
        Destroy(gameObject, 2.0f);

        // 通知游戏管理器
        GameManager.Instance?.OnEnemyKilled(this);
    }
}
```

### GameMode.cs - 游戏模式

```csharp
using GameEngine;
using GameEngine.ECS;
using System.Collections.Generic;

public class GameMode : Component
{
    [Header("Game Settings")]
    public int MaxPlayers = 10;
    public int GameDuration = 600; // 10分钟
    public int ScoreLimit = 50;

    [Header("Spawn Settings")]
    public float SpawnInterval = 5.0f;
    public int MaxEnemies = 20;

    private float gameTime = 0f;
    private float spawnTimer = 0f;
    private List<PlayerController> players = new List<PlayerController>();
    private List<Enemy> enemies = new List<Enemy>();

    public enum GameState
    {
        WaitingToStart,
        InProgress,
        GameOver
    }

    public GameState State { get; private set; }

    public void Start()
    {
        State = GameState.WaitingToStart;

        // 等待玩家
        StartCoroutine(StartGameWhenReady());
    }

    public void Update(float deltaTime)
    {
        if (State == GameState.InProgress)
        {
            UpdateGame(deltaTime);
        }
    }

    private IEnumerator StartGameWhenReady()
    {
        // 等待至少1个玩家
        while (players.Count == 0)
        {
            yield return new WaitForSeconds(1.0f);
        }

        StartGame();
    }

    public void StartGame()
    {
        State = GameState.InProgress;

        // 重置游戏
        gameTime = 0f;
        spawnTimer = 0f;

        // 生成初始敌人
        for (int i = 0; i < 5; i++)
        {
            SpawnEnemy();
        }

        // 显示游戏开始消息
        UI.ShowMessage("FIGHT!");
    }

    private void UpdateGame(float deltaTime)
    {
        gameTime += deltaTime;

        // 检查游戏结束条件
        if (gameTime >= GameDuration)
        {
            EndGame();
            return;
        }

        // 生成敌人
        spawnTimer += deltaTime;
        if (spawnTimer >= SpawnInterval && enemies.Count < MaxEnemies)
        {
            SpawnEnemy();
            spawnTimer = 0f;
        }

        // 更新UI
        UI.UpdateTime(GameDuration - gameTime);
        UI.UpdateScore(players.Count, enemies.Count);
    }

    private void SpawnEnemy()
    {
        // 从生成点选择
        Transform spawnPoint = GetRandomSpawnPoint();

        // 创建敌人
        GameObject enemy = Instantiate(EnemyPrefab, spawnPoint.position, spawnPoint.rotation);
        Enemy enemyComponent = enemy.GetComponent<Enemy>();
        enemies.Add(enemyComponent);
    }

    private Transform GetRandomSpawnPoint()
    {
        // 从预定义的生成点中随机选择
        Transform[] spawnPoints = GameObject.FindObjectsOfType<SpawnPoint>();
        return spawnPoints[Random.Range(0, spawnPoints.Length)].transform;
    }

    public void OnPlayerKilled(PlayerController player)
    {
        players.Remove(player);

        // 检查是否所有玩家都被消灭
        if (players.Count == 0)
        {
            EndGame();
        }
    }

    public void OnEnemyKilled(Enemy enemy)
    {
        enemies.Remove(enemy);

        // 增加分数
        PlayerController killer = enemy.LastHitBy;
        if (killer != null)
        {
            killer.AddScore(100);
        }
    }

    private void EndGame()
    {
        State = GameState.GameOver;

        // 显示结果
        UI.ShowGameOver(players.Count, enemies.Count);

        // 重启游戏
        StartCoroutine(RestartGameDelayed());
    }

    private IEnumerator RestartGameDelayed()
    {
        yield return new WaitForSeconds(5.0f);

        // 清理场景
        foreach (Enemy enemy in enemies)
        {
            if (enemy != null)
            {
                Destroy(enemy.gameObject);
            }
        }
        enemies.Clear();

        // 重新开始
        StartGame();
    }
}
```

## 🎓 学习要点

### 1. 3D空间操作

```csharp
// 3D向量操作
Vector3 position = new Vector3(0, 1, 0);
Vector3 forward = Transform.forward;
Vector3 right = Transform.right;

// 旋转
Quaternion rotation = Quaternion.Euler(0, 90, 0);
Transform.rotation = rotation;

// 距离计算
float distance = Vector3.Distance(a.position, b.position);
```

### 2. 相机控制

```csharp
// 第一人称视角
void Update()
{
    // 鼠标控制视角
    float mouseX = Input.GetAxis("Mouse X") * sensitivity;
    float mouseY = Input.GetAxis("Mouse Y") * sensitivity;

    // 水平旋转
    Transform.Rotate(0, mouseX, 0);

    // 垂直旋转（限制角度）
    pitch -= mouseY;
    pitch = Mathf.Clamp(pitch, -89, 89);

    camera.localRotation = Quaternion.Euler(pitch, 0, 0);
}
```

### 3. 物理模拟

```csharp
// 射弹道
void Shoot()
{
    Ray ray = new Ray(camera.position, camera.forward);

    if (Physics.Raycast(ray, out RaycastHit hit, 1000f))
    {
        // 命中目标
        Health health = hit.collider.GetComponent<Health>();
        if (health != null)
        {
            health.TakeDamage(damage);
        }

        // 创建命中效果
        Instantiate(hitEffect, hit.point, Quaternion.identity);
    }
}
```

### 4. AI导航

```csharp
// NavMesh导航
void Update()
{
    // 设置目标
    agent.SetDestination(target.position);

    // 检查是否到达
    if (agent.remainingDistance < 0.5f)
    {
        // 到达目标
    }

    // 检查路径状态
    if (agent.isPathStale)
    {
        // 重新计算路径
        agent.CalculatePath(target.position);
    }
}
```

## 🐛 常见问题

### Q: 3D模型没有显示？
A: 确保：
1. 模型文件在`assets/models/`目录
2. 模型格式被支持（GLTF, FBX等）
3. 材质和纹理正确加载

### Q: 鼠标视角不工作？
A: 检查：
1. 鼠标是否锁定（Cursor.lockState）
2. 鼠标敏感度设置
3. 是否有相机组件

### Q: 敌人AI不移动？
A: 确保：
1. NavMesh已烘焙
2. NavMeshAgent组件已添加
3. 目标点有效

### Q: 射击没有效果？
A: 检查：
1. 武器是否有弹药
2. 射击方向是否正确
3. 目标是否有碰撞体

## 📚 扩展建议

1. **添加更多武器**
   - 冲锋枪
   - 狙击步枪
   - 火箭发射器
   - 近战武器

2. **增强敌人AI**
   - 不同类型的敌人
   - 小队战术
   - 掩体系统
   - 手雷投掷

3. **多人模式**
   - 团队死斗
   - 夺旗模式
   - 搜索与摧毁

4. **视觉效果**
   - 粒子效果
   - 弹孔贴花
   - 动态光照
   - 环境反射

## 📝 性能优化

```csharp
// 对象池
public class BulletPool : MonoBehaviour
{
    private Stack<Bullet> pool = new Stack<Bullet>();

    public Bullet Get()
    {
        if (pool.Count > 0)
        {
            return pool.Pop();
        }

        return Instantiate(bulletPrefab);
    }

    public void Return(Bullet bullet)
    {
        bullet.gameObject.SetActive(false);
        pool.Push(bullet);
    }
}

// 射线优化
public class Weapon : MonoBehaviour
{
    public LayerMask hitLayers;

    void Shoot()
    {
        // 只检测特定层
        if (Physics.Raycast(ray, out hit, 1000f, hitLayers))
        {
            // ...
        }
    }
}
```

---

**状态**: 🚧 框架完成，需要填充实现细节

**下一步**: 添加完整的系统脚本和资源文件
