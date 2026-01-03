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

        [SyncVar]
        public int PlayerId;

        private Rigidbody rb;
        private TankTurret turret;
        private TankWeapon weapon;
        private bool isLocalPlayer;

        private void Awake()
        {
            rb = GetComponent<Rigidbody>();
        }

        private void Start()
        {
            turret = GetComponentInChildren<TankTurret>();
            weapon = GetComponentInChildren<TankWeapon>();
            isLocalPlayer = IsLocalPlayer;

            if (isLocalPlayer)
            {
                // 设置本地玩家相机
                SetupCamera();
            }

            // 注册到游戏管理器
            GameManager gameManager = FindObjectOfType<GameManager>();
            if (gameManager != null)
            {
                gameManager.RegisterTank(this);
            }
        }

        private void Update(float deltaTime)
        {
            if (!isLocalPlayer)
            {
                // 远程玩家：插值同步位置
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

            // 本地玩家：处理输入
            HandleMovement();
            HandleBrake();

            // 定期发送网络更新
            if (Time.time % 0.1f < Time.deltaTime)
            {
                SendNetworkUpdate();
            }
        }

        private void HandleMovement()
        {
            // 转向
            float turn = Input.GetAxis("Horizontal");
            if (Mathf.Abs(turn) > 0.01f)
            {
                float turnAngle = turn * TurnSpeed * Time.deltaTime;
                Transform.Rotate(0, turnAngle, 0);
            }

            // 移动
            float move = Input.GetAxis("Vertical");
            if (Mathf.Abs(move) > 0.01f)
            {
                Vector3 moveDirection = Transform.forward * move * MoveSpeed;

                rb.velocity = Vector3.MoveTowards(
                    rb.velocity,
                    moveDirection,
                    Acceleration * Time.deltaTime
                );
            }

            CurrentSpeed = rb.velocity.magnitude;
        }

        private void HandleBrake()
        {
            if (Input.GetKey(KeyCode.Space))
            {
                rb.velocity = Vector3.MoveTowards(
                    rb.velocity,
                    Vector3.Zero,
                    BrakeForce * Time.deltaTime
                );
            }
        }

        private void SendNetworkUpdate()
        {
            if (isServer)
            {
                // 服务器直接更新同步变量
                Position = Transform.position;
                Rotation = Transform.rotation;
            }
            else
            {
                // 客户端发送Command到服务器
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

        private void SetupCamera()
        {
            // 查找或创建相机
            Camera camera = FindObjectOfType<Camera>();

            if (camera == null)
            {
                GameObject cameraObj = new GameObject("MainCamera");
                camera = cameraObj.AddComponent<Camera>();
            }

            // 设置相机跟随
            CameraFollow cameraFollow = camera.GetComponent<CameraFollow>();
            if (cameraFollow == null)
            {
                cameraFollow = camera.gameObject.AddComponent<CameraFollow>();
            }

            cameraFollow.target = Transform;
            cameraFollow.offset = new Vector3(0, 10, -15);
            cameraFollow.smoothSpeed = 5.0f;
        }

        private void OnDestroy()
        {
            // 从游戏管理器注销
            GameManager gameManager = FindObjectOfType<GameManager>();
            if (gameManager != null)
            {
                gameManager.UnregisterTank(this);
            }
        }

        public bool IsLocalPlayer
        {
            get { return isLocalPlayer; }
        }

        public NetworkIdentity Identity
        {
            get { return netIdentity; }
        }
    }
}
