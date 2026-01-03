# Tank Battle - 坦克对战演示

一个使用C#脚本系统构建的多人坦克对战游戏示例，展示游戏引擎的网络同步和RPC能力。

## 🎮 游戏特性

- ✅ **多人对战**: 支持最多8人实时对战
- ✅ **网络同步**: 位置、旋转、生命值同步
- ✅ **RPC系统**: 远程过程调用实现游戏逻辑
- ✅ **坦克控制**: WASD移动 + 鼠标瞄准
- ✅ **武器系统**: 炮弹射击、冷却时间
- ✅ **碰撞检测**: 坦克与炮弹的物理碰撞
- ✅ **游戏模式**: 死斗模式、团队死斗
- ✅ **排行榜**: 实时分数和KDA统计

## 🚀 运行游戏

### 前置要求

```bash
# 安装 .NET SDK 8.0
dotnet --version  # 应该显示 8.0.x

# 确保支持网络功能
cargo build --features csharp,network
```

### 启动服务器

```bash
cd examples/csharp_games/tank_battle

# 启动专用服务器
cargo run --bin server --release
```

### 启动客户端

```bash
# 启动客户端并连接到本地服务器
cargo run --release -- --server 127.0.0.1:27015

# 或连接到远程服务器
cargo run --release -- --server 192.168.1.100:27015
```

## 🎯 游戏操作

### 坦克控制

- **W / S**: 前进 / 后退
- **A / D**: 左转 / 右转
- **鼠标**: 炮塔瞄准
- **鼠标左键**: 发射炮弹
- **Space**: 刹车
- **Shift**: 加速

### 游戏功能

- **Tab**: 排行榜
- **Esc**: 暂停菜单
- **Enter**: 聊天
- **F1**: 帮助信息

## 📁 文件结构

```
tank_battle/
├── scripts/
│   ├── Components/
│   │   ├── TankController.cs    # 坦克控制器
│   │   ├── TankTurret.cs        # 炮塔控制
│   │   ├── TankWeapon.cs        # 武器系统
│   │   ├── TankHealth.cs        # 生命值
│   │   └── Shell.cs             # 炮弹
│   ├── Network/
│   │   ├── NetworkManager.cs    # 网络管理器
│   │   ├── TankNetwork.cs       # 坦克网络同步
│   │   └── GameSync.cs          # 游戏状态同步
│   └── Game/
│       ├── GameManager.cs       # 游戏管理器
│       ├── ScoreManager.cs      # 分数管理器
│       └── GameMode.cs          # 游戏模式
├── src/
│   ├── main.rs                  # 客户端主程序
│   └── server.rs                # 服务器主程序
├── Cargo.toml
└── README.md
```

## 🔧 C# 脚本详解

### TankController.cs - 坦克控制器

```csharp
using GameEngine;
using GameEngine.ECS;
using GameEngine.Network;

namespace Components
{
    /// <summary>
    /// 坦克控制器 - 处理移动和网络同步
    /// </summary>
    [NetworkBehaviour]
    public class TankController : NetworkBehaviour
    {
        [Header("Movement")]
        public float MoveSpeed = 5.0f;
        public float TurnSpeed = 100.0f;
        public float Acceleration = 10.0f;
        public float BrakeForce = 20.0f;

        [Header("Network")]
        [SyncVar]
        public Vector3 Position;
        [SyncVar]
        public Quaternion Rotation;
        [SyncVar]
        public float CurrentSpeed;

        private Rigidbody rb;
        private TankTurret turret;
        private bool isLocalPlayer;

        private void Start()
        {
            rb = GetComponent<Rigidbody>();
            turret = GetComponentInChildren<TankTurret>();
            isLocalPlayer = IsLocalPlayer;

            if (isLocalPlayer)
            {
                // 启用本地玩家的控制
                enabled = true;
            }
            else
            {
                // 禁用远程玩家的控制（由网络同步驱动）
                enabled = false;
            }
        }

        private void Update(float deltaTime)
        {
            if (!isLocalPlayer)
            {
                // 插值同步位置
                Transform.position = Vector3.Lerp(
                    Transform.position,
                    Position,
                    deltaTime * 10.0f
                );
                Transform.rotation = Quaternion.Slerp(
                    Transform.rotation,
                    Rotation,
                    deltaTime * 10.0f
                );
                return;
            }

            HandleMovement();
            SendNetworkUpdate();
        }

        private void HandleMovement()
        {
            // 转向
            float turn = Input.GetAxis("Horizontal");
            float targetTurnSpeed = turn * TurnSpeed;
            rb.angularVelocity = new Vector3(0, targetTurnSpeed, 0);

            // 移动
            float move = Input.GetAxis("Vertical");
            Vector3 moveDirection = Transform.forward * move * MoveSpeed;

            // 应用刹车
            if (Input.GetKey(KeyCode.Space))
            {
                rb.velocity = Vector3.MoveTowards(
                    rb.velocity,
                    Vector3.Zero,
                    BrakeForce * Time.deltaTime
                );
            }
            else
            {
                rb.velocity = Vector3.MoveTowards(
                    rb.velocity,
                    moveDirection,
                    Acceleration * Time.deltaTime
                );
            }

            // 更新同步变量
            CurrentSpeed = rb.velocity.magnitude;
        }

        private void SendNetworkUpdate()
        {
            // 每100ms发送一次网络更新
            if (Time.time % 0.1f < Time.deltaTime)
            {
                CmdUpdateTransform(Transform.position, Transform.rotation);
            }
        }

        [Command]
        private void CmdUpdateTransform(Vector3 position, Quaternion rotation)
        {
            Position = position;
            Rotation = rotation;
        }

        [Server]
        public void TakeDamage(int damage, NetworkIdentity attacker)
        {
            if (!IsServer)
            {
                return;
            }

            TankHealth health = GetComponent<TankHealth>();
            if (health != null)
            {
                health.TakeDamage(damage, attacker);
            }
        }
    }
}
```

### TankTurret.cs - 炮塔控制

```csharp
using GameEngine;
using GameEngine.ECS;
using GameEngine.Network;

namespace Components
{
    /// <summary>
    /// 坦克炮塔 - 独立旋转和瞄准
    /// </summary>
    [NetworkBehaviour]
    public class TankTurret : NetworkBehaviour
    {
        [Header("Turret")]
        public float RotationSpeed = 50.0f;
        public float MinElevation = -10.0f;
        public float MaxElevation = 30.0f;

        [Header("Network")]
        [SyncVar]
        public Quaternion TurretRotation;
        [SyncVar]
        public float GunElevation;

        private Transform turretTransform;
        private Transform gunTransform;
        private bool isLocalPlayer;

        private void Start()
        {
            turretTransform = Transform.Find("Turret");
            gunTransform = turretTransform.Find("Gun");
            isLocalPlayer = IsLocalPlayer;
        }

        private void Update(float deltaTime)
        {
            if (!isLocalPlayer)
            {
                // 应用网络同步的旋转
                turretTransform.localRotation = Quaternion.Slerp(
                    turretTransform.localRotation,
                    TurretRotation,
                    deltaTime * 10.0f
                );

                Vector3 gunRotation = gunTransform.localEulerAngles;
                gunRotation.x = GunElevation;
                gunTransform.localEulerAngles = gunRotation;

                return;
            }

            HandleTurretRotation();
            HandleGunElevation();
        }

        private void HandleTurretRotation()
        {
            // 炮塔跟随鼠标位置
            Ray ray = Camera.main.ScreenPointToRay(Input.mousePosition);
            RaycastHit hit;

            if (Physics.Raycast(ray, out hit, 1000f))
            {
                Vector3 targetPoint = hit.point;
                Vector3 targetDirection = targetPoint - turretTransform.position;
                targetDirection.y = 0; // 保持水平旋转

                Quaternion targetRotation = Quaternion.LookRotation(targetDirection);
                turretTransform.localRotation = Quaternion.RotateTowards(
                    turretTransform.localRotation,
                    targetRotation,
                    RotationSpeed * Time.deltaTime
                );
            }

            // 同步炮塔旋转
            if (Time.time % 0.1f < Time.deltaTime)
            {
                CmdUpdateTurretRotation(turretTransform.localRotation);
            }
        }

        private void HandleGunElevation()
        {
            // 鼠标滚轮控制炮管仰角
            float elevationInput = Input.GetAxis("Mouse ScrollWheel");
            GunElevation = Mathf.Clamp(
                GunElevation + elevationInput * 10.0f,
                MinElevation,
                MaxElevation
            );

            Vector3 gunRotation = gunTransform.localEulerAngles;
            gunRotation.x = GunElevation;
            gunTransform.localEulerAngles = gunRotation;

            // 同步仰角
            if (Time.time % 0.1f < Time.deltaTime)
            {
                CmdUpdateGunElevation(GunElevation);
            }
        }

        [Command]
        private void CmdUpdateTurretRotation(Quaternion rotation)
        {
            TurretRotation = rotation;
        }

        [Command]
        private void CmdUpdateGunElevation(float elevation)
        {
            GunElevation = elevation;
        }

        public Vector3 GetMuzzlePosition()
        {
            return gunTransform.Find("Muzzle").position;
        }

        public Quaternion GetMuzzleRotation()
        {
            return gunTransform.Find("Muzzle").rotation;
        }
    }
}
```

### TankWeapon.cs - 武器系统

```csharp
using GameEngine;
using GameEngine.ECS;
using GameEngine.Network;
using System.Collections;

namespace Components
{
    /// <summary>
    /// 坦克武器 - 炮弹发射和管理
    /// </summary>
    [NetworkBehaviour]
    public class TankWeapon : NetworkBehaviour
    {
        [Header("Weapon")]
        public int ShellDamage = 30;
        public float FireRate = 2.0f;
        public float ShellSpeed = 20.0f;
        public float ShellLifetime = 5.0f;

        [Header("Network")]
        [SyncVar]
        public float LastFireTime;

        private TankTurret turret;
        private bool isLocalPlayer;

        private void Start()
        {
            turret = GetComponentInParent<TankTurret>();
            isLocalPlayer = IsLocalPlayer;
        }

        private void Update(float deltaTime)
        {
            if (!isLocalPlayer)
            {
                return;
            }

            // 射击
            if (Input.GetMouseButtonDown(0))
            {
                TryFire();
            }
        }

        private void TryFire()
        {
            // 检查射速
            if (Time.time - LastFireTime < FireRate)
            {
                return;
            }

            // 请求服务器射击
            CmdFire();
        }

        [Command]
        private void CmdFire()
        {
            // 服务器验证射速
            if (Time.time - LastFireTime < FireRate)
            {
                return;
            }

            // 获取炮口位置和方向
            Vector3 muzzlePos = turret.GetMuzzlePosition();
            Quaternion muzzleRot = turret.GetMuzzleRotation();

            // 在服务器上生成炮弹
            GameObject shell = Instantiate(
                ShellPrefab,
                muzzlePos,
                muzzleRot
            );

            // 设置炮弹属性
            Shell shellComponent = shell.GetComponent<Shell>();
            if (shellComponent != null)
            {
                shellComponent.Damage = ShellDamage;
                shellComponent.Owner = netIdentity;
            }

            // 网络生成炮弹
            NetworkServer.Spawn(shell);

            // 更新射击时间
            LastFireTime = Time.time;

            // 播放射击音效
            RpcPlayFireSound();
        }

        [ClientRpc]
        private void RpcPlayFireSound()
        {
            AudioSource audioSource = GetComponent<AudioSource>();
            if (audioSource != null)
            {
                audioSource.Play();
            }

            // 枪口火焰效果
            Instantiate(MuzzleFlashPrefab, turret.GetMuzzlePosition());
        }
    }
}
```

### Shell.cs - 炮弹

```csharp
using GameEngine;
using GameEngine.ECS;
using GameEngine.Network;
using System.Collections;

namespace Components
{
    /// <summary>
    /// 炮弹 - 网络同步的抛射物
    /// </summary>
    [NetworkBehaviour]
    public class Shell : NetworkBehaviour
    {
        [Header("Shell")]
        public int Damage = 30;
        public float Speed = 20.0f;
        public float Lifetime = 5.0f;

        [SyncVar]
        private Vector3 syncPosition;

        private NetworkIdentity owner;
        private Rigidbody rb;
        private bool hasExploded;

        private void Start()
        {
            rb = GetComponent<Rigidbody>();

            if (isServer)
            {
                // 服务器上启动销毁计时器
                StartCoroutine(DestroyAfterLifetime());
            }
        }

        private void Update(float deltaTime)
        {
            if (isServer)
            {
                // 服务器更新同步位置
                syncPosition = Transform.position;
            }
            else
            {
                // 客户端插值
                Transform.position = Vector3.Lerp(
                    Transform.position,
                    syncPosition,
                    deltaTime * 10.0f
                );
            }
        }

        private void FixedUpdate()
        {
            if (!isServer)
            {
                return;
            }

            // 服务器物理模拟
            rb.velocity = Transform.forward * Speed;
        }

        private void OnCollisionEnter(Collision collision)
        {
            if (!isServer || hasExploded)
            {
                return;
            }

            // 检查是否击中坦克
            TankHealth tankHealth = collision.gameObject.GetComponent<TankHealth>();

            if (tankHealth != null)
            {
                // 对坦克造成伤害
                tankHealth.TakeDamage(Damage, owner);

                // 销毁炮弹
                Explode();
            }
            else
            {
                // 击中环境，直接销毁
                Explode();
            }
        }

        private void Explode()
        {
            if (hasExploded)
            {
                return;
            }

            hasExploded = true;

            // 范围伤害
            Collider[] colliders = Physics.OverlapSphere(Transform.position, 5.0f);

            foreach (Collider collider in colliders)
            {
                TankHealth tankHealth = collider.GetComponent<TankHealth>();

                if (tankHealth != null)
                {
                    // 根据距离计算衰减伤害
                    float distance = Vector3.Distance(Transform.position, collider.transform.position);
                    int falloffDamage = Mathf.RoundToInt(Damage * (1.0f - distance / 5.0f));

                    if (falloffDamage > 0)
                    {
                        tankHealth.TakeDamage(falloffDamage, owner);
                    }
                }
            }

            // 爆炸效果
            RpcShowExplosion(Transform.position);

            // 销毁炮弹
            NetworkServer.Destroy(gameObject);
        }

        [ClientRpc]
        private void RpcShowExplosion(Vector3 position)
        {
            // 播放爆炸音效
            AudioSource.PlayClipAtPoint(ExplosionSound, position);

            // 创建爆炸视觉效果
            Instantiate(
                ExplosionPrefab,
                position,
                Quaternion.identity
            );
        }

        private IEnumerator DestroyAfterLifetime()
        {
            yield return new WaitForSeconds(Lifetime);

            if (!hasExploded)
            {
                Explode();
            }
        }

        public NetworkIdentity Owner
        {
            get { return owner; }
            set { owner = value; }
        }
    }
}
```

### NetworkManager.cs - 网络管理器

```csharp
using GameEngine;
using GameEngine.Network;
using GameEngine.ECS;

namespace Network
{
    /// <summary>
    /// 网络管理器 - 处理连接和游戏状态
    /// </summary>
    public class NetworkManager : MonoBehaviour
    {
        [Header("Network Settings")]
        public int MaxPlayers = 8;
        public int Port = 27015;
        public string ServerAddress = "127.0.0.1";

        [Header("Prefabs")]
        public GameObject PlayerPrefab;

        public static NetworkManager Instance { get; private set; }
        public bool IsServer { get; private set; }
        public bool IsClient { get; private set; }
        public bool IsConnected { get; private set; }

        private void Awake()
        {
            if (Instance != null && Instance != this)
            {
                Destroy(gameObject);
                return;
            }
            Instance = this;
        }

        private void Start()
        {
            // 检查命令行参数
            string[] args = System.Environment.GetCommandLineArgs();

            for (int i = 0; i < args.Length; i++)
            {
                if (args[i] == "--server" && i + 1 < args.Length)
                {
                    // 启动服务器模式
                    string serverAddress = args[i + 1];
                    StartServer(serverAddress);
                    return;
                }
                else if (args[i] == "--dedicated")
                {
                    // 启动专用服务器
                    StartDedicatedServer();
                    return;
                }
            }

            // 默认启动客户端
            StartClient();
        }

        public void StartServer(string address)
        {
            NetworkServer.Configure(MaxPlayers, Port);
            NetworkServer.RegisterHandler(MsgType.Connect, OnServerConnect);
            NetworkServer.RegisterHandler(MsgType.Disconnect, OnServerDisconnect);
            NetworkServer.RegisterHandler(MsgType.Ready, OnClientReady);

            if (NetworkServer.Listen(address))
            {
                IsServer = true;
                IsConnected = true;
                Debug.Log("服务器启动: " + address);
            }
            else
            {
                Debug.LogError("服务器启动失败");
            }
        }

        public void StartDedicatedServer()
        {
            NetworkServer.Configure(MaxPlayers, Port);
            NetworkServer.RegisterHandler(MsgType.Connect, OnServerConnect);
            NetworkServer.RegisterHandler(MsgType.Disconnect, OnServerDisconnect);

            if (NetworkServer.Listen(Port))
            {
                IsServer = true;
                IsConnected = true;
                Debug.Log("专用服务器启动，端口: " + Port);

                // 加载游戏场景
                SceneManager.LoadScene("GameScene");
            }
            else
            {
                Debug.LogError("专用服务器启动失败");
            }
        }

        public void StartClient()
        {
            NetworkClient.Connect(ServerAddress, Port);
            NetworkClient.RegisterHandler(MsgType.Connect, OnClientConnect);
            NetworkClient.RegisterHandler(MsgType.Disconnect, OnClientDisconnect);

            IsClient = true;
            Debug.Log("连接到服务器: " + ServerAddress);
        }

        private void OnServerConnect(NetworkMessage netMsg)
        {
            Debug.Log("客户端连接: " + netMsg.conn.address);

            // 生成玩家坦克
            GameObject player = Instantiate(PlayerPrefab);
            NetworkServer.AddPlayerForConnection(netMsg.conn, player);
        }

        private void OnServerDisconnect(NetworkMessage netMsg)
        {
            Debug.Log("客户端断开: " + netMsg.conn.address);

            // 销毁玩家坦克
            if (netMsg.conn.playerControllers.Count > 0)
            {
                GameObject player = netMsg.conn.playerControllers[0].gameObject;
                NetworkServer.Destroy(player);
            }
        }

        private void OnClientReady(NetworkMessage netMsg)
        {
            Debug.Log("客户端就绪");
        }

        private void OnClientConnect(NetworkMessage netMsg)
        {
            IsConnected = true;
            Debug.Log("已连接到服务器");

            // 通知服务器客户端就绪
            NetworkClient.Send(MsgType.Ready);

            // 加载游戏场景
            SceneManager.LoadScene("GameScene");
        }

        private void OnClientDisconnect(NetworkMessage netMsg)
        {
            IsConnected = false;
            Debug.Log("与服务器断开连接");

            // 显示断线提示
            UI.ShowDisconnected();
        }

        public void Disconnect()
        {
            if (IsClient && IsConnected)
            {
                NetworkClient.Disconnect();
            }
            else if (IsServer)
            {
                NetworkServer.Shutdown();
            }
        }
    }
}
```

### GameManager.cs - 游戏管理器

```csharp
using GameEngine;
using GameEngine.ECS;
using GameEngine.Network;

namespace Game
{
    /// <summary>
    /// 游戏管理器 - 游戏流程和规则
    /// </summary>
    public class GameManager : NetworkBehaviour
    {
        public enum GameMode
        {
            FreeForAll,    // 自由混战
            TeamDeathmatch // 团队死斗
        }

        [Header("Game Settings")]
        public GameMode CurrentMode = GameMode.FreeForAll;
        public int ScoreLimit = 50;
        public int GameDuration = 600; // 10分钟

        [Header("Spawn Points")]
        public Transform[] SpawnPoints;

        private float gameStartTime;
        private bool gameInProgress;

        public static GameManager Instance { get; private set; }

        private void Awake()
        {
            if (Instance != null && Instance != this)
            {
                Destroy(gameObject);
                return;
            }
            Instance = this;
        }

        public override void OnStartServer()
        {
            // 服务器上启动游戏
            StartGame();
        }

        private void StartGame()
        {
            gameStartTime = Time.time;
            gameInProgress = true;

            Debug.Log("游戏开始: " + CurrentMode);

            // 通知所有客户端游戏开始
            RpcGameStart();
        }

        [ClientRpc]
        private void RpcGameStart()
        {
            UI.ShowMessage("游戏开始!");
            UI.HideLobby();
        }

        public void OnPlayerKilled(NetworkIdentity victim, NetworkIdentity killer)
        {
            if (!isServer)
            {
                return;
            }

            // 更新分数
            ScoreManager scoreManager = FindObjectOfType<ScoreManager>();
            if (scoreManager != null)
            {
                scoreManager.AddScore(killer, 1);
            }

            // 检查胜利条件
            if (scoreManager.GetScore(killer) >= ScoreLimit)
            {
                EndGame(killer);
            }
        }

        private void EndGame(NetworkIdentity winner)
        {
            gameInProgress = false;

            Debug.Log("游戏结束，获胜者: " + winner.netId);

            // 通知所有客户端游戏结束
            RpcGameEnd(winner);
        }

        [ClientRpc]
        private void RpcGameEnd(NetworkIdentity winner)
        {
            string winnerName = winner.name;
            UI.ShowGameOver(winnerName);
        }

        public Transform GetSpawnPoint()
        {
            if (SpawnPoints == null || SpawnPoints.Length == 0)
            {
                return null;
            }

            // 随机选择一个生成点
            int index = Random.Range(0, SpawnPoints.Length);
            return SpawnPoints[index];
        }

        public float GetRemainingTime()
        {
            if (!gameInProgress)
            {
                return 0f;
            }

            float elapsed = Time.time - gameStartTime;
            return Mathf.Max(0f, GameDuration - elapsed);
        }
    }
}
```

## 🎓 学习要点

### 1. 网络同步

```csharp
// 同步变量
[SyncVar]
public Vector3 Position;

// Command（客户端→服务器）
[Command]
private void CmdUpdateTransform(Vector3 position, Quaternion rotation)
{
    // 服务器执行
    Position = position;
    Rotation = rotation;
}

// ClientRpc（服务器→客户端）
[ClientRpc]
private void RpcShowExplosion(Vector3 position)
{
    // 所有客户端执行
    Instantiate(ExplosionPrefab, position);
}
```

### 2. 网络行为

```csharp
[NetworkBehaviour]
public class TankController : NetworkBehaviour
{
    public bool IsLocalPlayer { get; }
    public bool isServer { get; }
    public bool isClient { get; }

    private void Update()
    {
        if (!IsLocalPlayer)
        {
            return; // 只处理本地玩家输入
        }

        HandleInput();
    }
}
```

### 3. 远程过程调用

```csharp
// 客户端调用服务器方法
[Command]
private void CmdFire()
{
    // 服务器验证并执行
    if (Time.time - LastFireTime >= FireRate)
    {
        SpawnShell();
        LastFireTime = Time.time;

        // 回调客户端显示效果
        RpcPlayFireEffect();
    }
}
```

### 4. 网络对象生成

```csharp
// 服务器上生成
[Server]
public void SpawnShell()
{
    GameObject shell = Instantiate(ShellPrefab, position, rotation);
    NetworkServer.Spawn(shell); // 网络同步生成
}
```

## 🐛 常见问题

### Q: 客户端无法连接服务器？
A: 检查：
1. 防火墙设置
2. 服务器地址和端口
3. 网络连接
4. 服务器是否已启动

### Q: 位置同步卡顿？
A: 优化：
1. 增加插值速度
2. 使用网络压缩
3. 降低同步频率
4. 启用客户端预测

### Q: RPC调用失败？
A: 确保：
1. 已添加[Command]/[ClientRpc]属性
2. 在正确的对象上调用
3. 网络连接正常
4. 权限设置正确

## 📚 扩展建议

1. **增强网络功能**
   - 客户端预测
   - 服务器回滚
   - 延迟补偿
   - 丢包重传

2. **添加游戏模式**
   - 夺旗模式
   - 推送模式
   - 占领模式
   - 护送模式

3. **改进排行榜**
   - 全球排行
   - 成就系统
   - 赛季系统
   - 奖励机制

4. **优化性能**
   - 对象池
   - 网络压缩
   - 帧同步
   - LOD系统

---

**状态**: 🚧 框架完成，需要填充实现细节

**下一步**: 添加完整的系统脚本和Rust集成代码
