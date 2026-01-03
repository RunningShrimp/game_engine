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

        [Header("Explosion")]
        public float ExplosionRadius = 5.0f;
        public float ExplosionForce = 1000.0f;
        public GameObject ExplosionPrefab;

        [SyncVar]
        private Vector3 syncPosition;

        [SyncVar]
        private Quaternion syncRotation;

        private NetworkIdentity owner;
        private Rigidbody rb;
        private bool hasExploded;

        private void Awake()
        {
            rb = GetComponent<Rigidbody>();
        }

        private void Start()
        {
            if (isServer)
            {
                // 服务器上启动销毁计时器
                StartCoroutine(DestroyAfterLifetime());
            }
            else
            {
                // 客户端禁用物理模拟
                if (rb != null)
                {
                    rb.isKinematic = true;
                }
            }
        }

        private void Update()
        {
            if (isServer)
            {
                // 服务器更新同步位置
                syncPosition = Transform.position;
                syncRotation = Transform.rotation;
            }
            else
            {
                // 客户端插值
                Transform.position = Vector3.Lerp(
                    Transform.position,
                    syncPosition,
                    Time.deltaTime * 10.0f
                );

                Transform.rotation = Quaternion.Slerp(
                    Transform.rotation,
                    syncRotation,
                    Time.deltaTime * 10.0f
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
            if (rb != null)
            {
                rb.velocity = Transform.forward * Speed;
            }
            else
            {
                // 如果没有刚体，直接移动
                Transform.position += Transform.forward * Speed * Time.fixedDeltaTime;
            }
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
            Collider[] colliders = Physics.OverlapSphere(Transform.position, ExplosionRadius);

            foreach (Collider collider in colliders)
            {
                TankHealth tankHealth = collider.GetComponent<TankHealth>();

                if (tankHealth != null)
                {
                    // 根据距离计算衰减伤害
                    float distance = Vector3.Distance(Transform.position, collider.transform.position);
                    float falloff = 1.0f - Mathf.Clamp01(distance / ExplosionRadius);
                    int falloffDamage = Mathf.RoundToInt(Damage * falloff);

                    if (falloffDamage > 0)
                    {
                        tankHealth.TakeDamage(falloffDamage, owner);
                    }
                }

                // 应用爆炸力
                Rigidbody targetRb = collider.GetComponent<Rigidbody>();
                if (targetRb != null)
                {
                    Vector3 direction = (collider.transform.position - Transform.position).normalized;
                    targetRb.AddForce(direction * ExplosionForce * falloff, ForceMode.Impulse);
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
            if (ExplosionSound != null)
            {
                AudioSource.PlayClipAtPoint(ExplosionSound, position);
            }

            // 创建爆炸视觉效果
            if (ExplosionPrefab != null)
            {
                GameObject explosion = Instantiate(ExplosionPrefab, position, Quaternion.identity);
                Destroy(explosion, 3.0f);
            }
            else
            {
                // 默认爆炸效果
                CreateDefaultExplosion(position);
            }

            // 相机震动
            CameraShake cameraShake = Camera.main?.GetComponent<CameraShake>();
            if (cameraShake != null)
            {
                float distance = Vector3.Distance(Camera.main.transform.position, position);
                if (distance < 50.0f)
                {
                    float intensity = 1.0f - (distance / 50.0f);
                    cameraShake.Shake(intensity * 0.5f, 0.3f);
                }
            }
        }

        private void CreateDefaultExplosion(Vector3 position)
        {
            // 创建简单的粒子效果
            GameObject explosion = new GameObject("Explosion");
            explosion.transform.position = position;

            // 光源
            Light light = explosion.AddComponent<Light>();
            light.type = LightType.Point;
            light.color = new Color(1.0f, 0.5f, 0.0f);
            light.range = 10.0f;
            light.intensity = 2.0f;

            // 淡出动画
            MonoBehaviour.StartCoroutine(FadeOutLight(light, 0.5f));

            // 销毁
            Destroy(explosion, 1.0f);
        }

        private IEnumerator FadeOutLight(Light light, float duration)
        {
            float elapsed = 0f;
            float originalIntensity = light.intensity;

            while (elapsed < duration)
            {
                light.intensity = Mathf.Lerp(originalIntensity, 0f, elapsed / duration);
                elapsed += Time.deltaTime;
                yield return null;
            }

            light.intensity = 0f;
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

        public AudioClip ExplosionSound { get; set; }
    }
}
