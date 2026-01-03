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

        [Header("Prefabs")]
        public GameObject ShellPrefab;
        public GameObject MuzzleFlashPrefab;

        [Header("Network")]
        [SyncVar]
        public float LastFireTime;

        private TankTurret turret;
        private TankController tankController;
        private AudioSource audioSource;
        private bool isLocalPlayer;

        private void Start()
        {
            turret = GetComponentInParent<TankTurret>();
            tankController = GetComponentInParent<TankController>();
            audioSource = GetComponent<AudioSource>();
            isLocalPlayer = tankController != null && tankController.IsLocalPlayer;
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
            if (isServer)
            {
                Fire();
            }
            else
            {
                CmdFire();
            }
        }

        [Command]
        private void CmdFire()
        {
            // 服务器验证射速
            if (Time.time - LastFireTime < FireRate)
            {
                return;
            }

            Fire();
        }

        [Server]
        private void Fire()
        {
            // 获取炮口位置和方向
            Vector3 muzzlePos = turret.GetMuzzlePosition();
            Quaternion muzzleRot = turret.GetMuzzleRotation();

            // 在服务器上生成炮弹
            GameObject shell = Instantiate(ShellPrefab, muzzlePos, muzzleRot);
            shell.name = "Shell_" + netId.ToString();

            // 设置炮弹属性
            Shell shellComponent = shell.GetComponent<Shell>();
            if (shellComponent != null)
            {
                shellComponent.Damage = ShellDamage;
                shellComponent.Speed = ShellSpeed;
                shellComponent.Lifetime = ShellLifetime;
                shellComponent.Owner = tankController.Identity;
            }

            // 网络生成炮弹
            NetworkServer.Spawn(shell);

            // 更新射击时间
            LastFireTime = Time.time;

            // 播放射击音效和效果
            RpcPlayFireEffect();
        }

        [ClientRpc]
        private void RpcPlayFireEffect()
        {
            // 播放射击音效
            if (audioSource != null)
            {
                audioSource.Play();
            }

            // 枪口火焰效果
            if (MuzzleFlashPrefab != null && turret != null)
            {
                GameObject flash = Instantiate(
                    MuzzleFlashPrefab,
                    turret.GetMuzzlePosition(),
                    turret.GetMuzzleRotation()
                );
                Destroy(flash, 0.1f);
            }

            // 后坐力动画
            StartCoroutine(RecoilAnimation());
        }

        private IEnumerator RecoilAnimation()
        {
            if (turret == null)
            {
                yield break;
            }

            Transform gunTransform = turret.Transform.Find("Gun");
            if (gunTransform == null)
            {
                yield break;
            }

            Vector3 originalPos = gunTransform.localPosition;
            Vector3 recoilPos = originalPos - new Vector3(0, 0, 0.5f);

            // 后坐
            float elapsed = 0f;
            while (elapsed < 0.05f)
            {
                gunTransform.localPosition = Vector3.Lerp(
                    originalPos,
                    recoilPos,
                    elapsed / 0.05f
                );
                elapsed += Time.deltaTime;
                yield return null;
            }

            gunTransform.localPosition = recoilPos;

            // 恢复
            elapsed = 0f;
            while (elapsed < 0.2f)
            {
                gunTransform.localPosition = Vector3.Lerp(
                    recoilPos,
                    originalPos,
                    elapsed / 0.2f
                );
                elapsed += Time.deltaTime;
                yield return null;
            }

            gunTransform.localPosition = originalPos;
        }

        public float GetFireRate()
        {
            return FireRate;
        }

        public float GetCooldownRemaining()
        {
            float elapsed = Time.time - LastFireTime;
            return Mathf.Max(0f, FireRate - elapsed);
        }
    }
}
