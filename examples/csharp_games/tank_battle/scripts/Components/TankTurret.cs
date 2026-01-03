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
        private Transform muzzleTransform;
        private TankController tankController;
        private bool isLocalPlayer;

        private void Start()
        {
            // 查找子对象
            turretTransform = Transform.Find("Turret");
            if (turretTransform == null)
            {
                turretTransform = Transform;
            }

            gunTransform = turretTransform.Find("Gun");
            if (gunTransform == null)
            {
                gunTransform = turretTransform;
            }

            muzzleTransform = gunTransform.Find("Muzzle");
            if (muzzleTransform == null)
            {
                // 创建默认炮口位置
                GameObject muzzle = new GameObject("Muzzle");
                muzzle.transform.SetParent(gunTransform);
                muzzle.transform.localPosition = new Vector3(0, 0, 2);
                muzzleTransform = muzzle.transform;
            }

            tankController = GetComponentInParent<TankController>();
            isLocalPlayer = tankController != null && tankController.IsLocalPlayer;
        }

        private void Update(float deltaTime)
        {
            if (!isLocalPlayer)
            {
                // 远程玩家：应用网络同步的旋转
                if (turretTransform != null)
                {
                    turretTransform.localRotation = Quaternion.Slerp(
                        turretTransform.localRotation,
                        TurretRotation,
                        deltaTime * 10.0f
                    );
                }

                if (gunTransform != null)
                {
                    Vector3 gunRotation = gunTransform.localEulerAngles;
                    gunRotation.x = GunElevation;
                    gunTransform.localEulerAngles = gunRotation;
                }

                return;
            }

            // 本地玩家：处理输入
            HandleTurretRotation();
            HandleGunElevation();

            // 定期发送网络更新
            if (Time.time % 0.1f < Time.deltaTime)
            {
                SendNetworkUpdate();
            }
        }

        private void HandleTurretRotation()
        {
            if (turretTransform == null)
            {
                return;
            }

            // 炮塔跟随鼠标位置
            Camera camera = Camera.main;
            if (camera == null)
            {
                return;
            }

            Ray ray = camera.ScreenPointToRay(Input.mousePosition);
            RaycastHit hit;

            if (Physics.Raycast(ray, out hit, 1000f))
            {
                Vector3 targetPoint = hit.point;
                Vector3 targetDirection = targetPoint - turretTransform.position;
                targetDirection.y = 0; // 保持水平旋转

                if (targetDirection.sqrMagnitude > 0.001f)
                {
                    Quaternion targetRotation = Quaternion.LookRotation(targetDirection);
                    turretTransform.localRotation = Quaternion.RotateTowards(
                        turretTransform.localRotation,
                        targetRotation,
                        RotationSpeed * Time.deltaTime
                    );
                }
            }
        }

        private void HandleGunElevation()
        {
            if (gunTransform == null)
            {
                return;
            }

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
        }

        private void SendNetworkUpdate()
        {
            if (isServer)
            {
                // 服务器直接更新
                if (turretTransform != null)
                {
                    TurretRotation = turretTransform.localRotation;
                }
            }
            else
            {
                // 客户端发送Command
                if (turretTransform != null)
                {
                    CmdUpdateTurretRotation(turretTransform.localRotation);
                }
            }

            if (!isServer)
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
            if (muzzleTransform != null)
            {
                return muzzleTransform.position;
            }

            if (gunTransform != null)
            {
                return gunTransform.position + gunTransform.forward * 2.0f;
            }

            return Transform.position + Transform.forward * 2.0f;
        }

        public Quaternion GetMuzzleRotation()
        {
            if (muzzleTransform != null)
            {
                return muzzleTransform.rotation;
            }

            if (gunTransform != null)
            {
                return gunTransform.rotation;
            }

            return Transform.rotation;
        }
    }
}
