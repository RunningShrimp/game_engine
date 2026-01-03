using GameEngine;
using GameEngine.ECS;
using System.Collections;

namespace Components
{
    /// <summary>
    /// 武器系统 - 处理射击、换弹和后坐力
    /// </summary>
    public class Weapon : Component
    {
        public enum WeaponType
        {
            Pistol,
            Rifle,
            Shotgun,
            Sniper
        }

        [Header("Weapon Type")]
        public WeaponType Type;

        [Header("Stats")]
        public int Damage = 10;
        public float FireRate = 0.1f;
        public int MagazineSize = 30;
        public int ReserveAmmo = 120;
        public float Range = 1000.0f;

        [Header("Recoil")]
        public float RecoilForce = 0.5f;
        public float RecoilRecovery = 5.0f;

        [Header("Spread")]
        public float BaseSpread = 0.01f;
        public float MaxSpread = 0.05f;
        public float SpreadIncrease = 0.005f;
        public float SpreadDecrease = 0.01f;

        [Header("Prefabs")]
        public GameObject BulletPrefab;
        public GameObject MuzzleFlashPrefab;
        public GameObject ImpactEffectPrefab;

        // 状态
        private int currentAmmo;
        private int currentReserve;
        private float lastFireTime;
        private float currentSpread;
        private bool isReloading;

        // 组件引用
        private FirstPersonCamera camera;
        private AudioSource audioSource;

        private void Awake()
        {
            currentAmmo = MagazineSize;
            currentReserve = ReserveAmmo;
            currentSpread = BaseSpread;
        }

        private void Start()
        {
            camera = GetComponentInParent<FirstPersonCamera>();
            audioSource = GetComponent<AudioSource>();
        }

        private void Update(float deltaTime)
        {
            if (isReloading)
            {
                return;
            }

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

            // 恢复散射
            if (!Input.GetMouseButton(0))
            {
                currentSpread = Mathf.MoveTowards(
                    currentSpread,
                    BaseSpread,
                    SpreadDecrease * deltaTime
                );
            }
        }

        public void TryFire()
        {
            // 检查射速
            if (Time.time - lastFireTime < FireRate)
            {
                return;
            }

            // 检查装弹状态
            if (isReloading)
            {
                return;
            }

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

            // 增加散射
            currentSpread = Mathf.Min(currentSpread + SpreadIncrease, MaxSpread);

            // 应用后坐力
            ApplyRecoil();

            // 更新UI
            UI.UpdateAmmo(currentAmmo, currentReserve);
        }

        private void Fire()
        {
            if (camera == null)
            {
                return;
            }

            // 获取射击方向（应用散射）
            Vector3 shootDirection = ApplySpread(camera.GetForwardVector());

            // 射线检测
            Ray ray = new Ray(camera.GetPosition(), shootDirection);

            if (Physics.Raycast(ray, out RaycastHit hit, Range))
            {
                ProcessHit(hit);
            }

            // 创建子弹视觉效果
            CreateBulletVisual(ray, hit);

            // 播放射击音效
            PlayShotSound();

            // 枪口火焰效果
            SpawnMuzzleFlash();
        }

        private Vector3 ApplySpread(Vector3 direction)
        {
            // 在XY平面应用随机偏移
            float spreadAngle = currentSpread * 0.5f; // 半散射角
            float angleX = Random.Range(-spreadAngle, spreadAngle);
            float angleY = Random.Range(-spreadAngle, spreadAngle);

            // 创建旋转偏移
            Quaternion spreadRotation = Quaternion.Euler(angleX, angleY, 0.0f);
            return spreadRotation * direction;
        }

        private void ProcessHit(RaycastHit hit)
        {
            // 检查是否击中实体
            GameObject hitObject = hit.collider.gameObject;

            // 尝试获取生命值组件
            Health health = hitObject.GetComponent<Health>();
            if (health != null)
            {
                health.TakeDamage(Damage);

                // 显示伤害数字
                UI.ShowDamageNumber(hit.point, Damage);

                return;
            }

            // 检查是否击中其他敌人
            Enemy enemy = hitObject.GetComponent<Enemy>();
            if (enemy != null)
            {
                enemy.TakeDamage(Damage);

                // 显示伤害数字
                UI.ShowDamageNumber(hit.point, Damage);
            }

            // 创建命中效果
            if (ImpactEffectPrefab != null)
            {
                GameObject impact = Instantiate(
                    ImpactEffectPrefab,
                    hit.point,
                    Quaternion.LookAt(hit.normal)
                );
                Destroy(impact, 2.0f);
            }
        }

        private void CreateBulletVisual(Ray ray, RaycastHit hit)
        {
            // 创建子弹轨迹
            Vector3 endPoint = hit.collider != null ? hit.point : ray.origin + ray.direction * Range;

            // 使用LineRenderer或粒子系统绘制轨迹
            GameObject tracer = new GameObject("BulletTracer");
            LineRenderer line = tracer.AddComponent<LineRenderer>();
            line.SetPosition(0, ray.origin);
            line.SetPosition(1, endPoint);
            line.startWidth = 0.02f;
            line.endWidth = 0.02f;
            line.material = new Material(Shader.Find("Unlit/Color"));
            line.material.color = new Color(1.0f, 0.8f, 0.3f, 0.5f);

            // 淡出效果
            StartCoroutine(FadeTracer(line));
        }

        private IEnumerator FadeTracer(LineRenderer line)
        {
            float duration = 0.2f;
            float elapsed = 0.0f;

            while (elapsed < duration)
            {
                elapsed += Time.deltaTime;
                float alpha = 1.0f - (elapsed / duration);
                Color color = line.material.color;
                color.a = alpha;
                line.material.color = color;
                yield return null;
            }

            Destroy(line.gameObject);
        }

        private void ApplyRecoil()
        {
            if (camera != null)
            {
                camera.AddRecoil(RecoilForce);
            }
        }

        private void PlayShotSound()
        {
            if (audioSource != null)
            {
                // 根据武器类型播放不同音效
                AudioClip clip = GetShotSoundClip();
                audioSource.PlayOneShot(clip);
            }
        }

        private AudioClip GetShotSoundClip()
        {
            // 这里应该从资源管理器加载音效
            // 简化示例，返回null
            return null;
        }

        private void SpawnMuzzleFlash()
        {
            if (MuzzleFlashPrefab != null)
            {
                GameObject flash = Instantiate(
                    MuzzleFlashPrefab,
                    Transform.position,
                    Transform.rotation
                );
                Destroy(flash, 0.1f);
            }
        }

        public void Reload()
        {
            if (isReloading)
            {
                return;
            }

            if (currentReserve <= 0 || currentAmmo == MagazineSize)
            {
                return;
            }

            StartCoroutine(ReloadCoroutine());
        }

        private IEnumerator ReloadCoroutine()
        {
            isReloading = true;

            // 播放装弹动画
            Animator animator = GetComponent<Animator>();
            if (animator != null)
            {
                animator.SetTrigger("Reload");
            }

            // 播放装弹音效
            if (audioSource != null)
            {
                AudioClip reloadClip = GetReloadSoundClip();
                audioSource.PlayOneShot(reloadClip);
            }

            // UI提示
            UI.ShowReloading();

            // 等待装弹完成
            yield return new WaitForSeconds(2.0f);

            // 计算装弹数量
            int needed = MagazineSize - currentAmmo;
            int available = Mathf.Min(needed, currentReserve);

            currentAmmo += available;
            currentReserve -= available;

            // 更新UI
            UI.UpdateAmmo(currentAmmo, currentReserve);

            isReloading = false;
        }

        private AudioClip GetReloadSoundClip()
        {
            // 从资源管理器加载音效
            return null;
        }

        public int GetCurrentAmmo()
        {
            return currentAmmo;
        }

        public int GetReserveAmmo()
        {
            return currentReserve;
        }

        public bool IsReloading()
        {
            return isReloading;
        }
    }
}
