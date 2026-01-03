using GameEngine;
using GameEngine.ECS;

namespace Game
{
    /// <summary>
    /// 生成点 - 敌人和玩家的生成位置
    /// </summary>
    public class SpawnPoint : Component
    {
        [Header("Spawn Settings")]
        public SpawnType Type = SpawnType.Enemy;
        public bool IsAvailable { get; set; } = true;
        public float CooldownTime = 2.0f;

        [Header("Visualization")]
        public bool ShowGizmo = true;
        public float GizmoRadius = 0.5f;
        public Color GizmoColor = new Color(0.0f, 1.0f, 0.0f, 0.5f);

        private float lastSpawnTime;
        private Coroutine cooldownCoroutine;

        public enum SpawnType
        {
            Player,
            Enemy,
            Both
        }

        private void Start()
        {
            // 自动注册到GameMode
            GameMode gameMode = GameMode.Instance;
            if (gameMode != null)
            {
                // GameMode会在Start时查找所有SpawnPoint
            }
        }

        public GameObject Spawn(GameObject prefab)
        {
            if (prefab == null)
            {
                Debug.LogWarning("生成点: 预制体为空");
                return null;
            }

            if (!IsAvailable)
            {
                Debug.LogWarning("生成点: 不可用");
                return null;
            }

            // 创建对象
            GameObject spawnedObject = Instantiate(prefab, Transform.position, Transform.rotation);

            // 开始冷却
            StartCooldown();

            log_info("生成: {0} at {1}", prefab.name, Transform.position);

            return spawnedObject;
        }

        public GameObject Spawn(GameObject prefab, Quaternion rotation)
        {
            if (prefab == null)
            {
                return null;
            }

            if (!IsAvailable)
            {
                return null;
            }

            // 创建对象
            GameObject spawnedObject = Instantiate(prefab, Transform.position, rotation);

            // 开始冷却
            StartCooldown();

            return spawnedObject;
        }

        public GameObject Spawn(GameObject prefab, Vector3 positionOffset)
        {
            if (prefab == null)
            {
                return null;
            }

            if (!IsAvailable)
            {
                return null;
            }

            // 创建对象
            Vector3 spawnPosition = Transform.position + positionOffset;
            GameObject spawnedObject = Instantiate(prefab, spawnPosition, Transform.rotation);

            // 开始冷却
            StartCooldown();

            return spawnedObject;
        }

        private void StartCooldown()
        {
            IsAvailable = false;

            if (cooldownCoroutine != null)
            {
                StopCoroutine(cooldownCoroutine);
            }

            cooldownCoroutine = StartCoroutine(CooldownCoroutine());
        }

        private System.Collections.IEnumerator CooldownCoroutine()
        {
            yield return new WaitForSeconds(CooldownTime);

            IsAvailable = true;
            cooldownCoroutine = null;
        }

        public void ForceAvailable()
        {
            if (cooldownCoroutine != null)
            {
                StopCoroutine(cooldownCoroutine);
                cooldownCoroutine = null;
            }

            IsAvailable = true;
        }

        public float GetCooldownProgress()
        {
            if (IsAvailable)
            {
                return 1.0f;
            }

            float elapsed = Time.time - lastSpawnTime;
            return Mathf.Clamp01(elapsed / CooldownTime);
        }

        private void OnDrawGizmos()
        {
            if (!ShowGizmo)
            {
                return;
            }

            // 绘制生成点
            Gizmos.color = GizmoColor;
            Gizmos.DrawSphere(Transform.position, GizmoRadius);

            // 绘制方向指示器
            Gizmos.color = new Color(1.0f, 1.0f, 1.0f, 0.5f);
            Vector3 direction = Transform.forward * 2.0f;
            Gizmos.DrawLine(Transform.position, Transform.position + direction);

            // 绘制类型标识
            Gizmos.color = GetGizmoColorForType();
            Gizmos.DrawWireSphere(Transform.position, GizmoRadius * 1.2f);
        }

        private Color GetGizmoColorForType()
        {
            switch (Type)
            {
                case SpawnType.Player:
                    return new Color(0.0f, 0.5f, 1.0f, 0.5f); // 蓝色
                case SpawnType.Enemy:
                    return new Color(1.0f, 0.0f, 0.0f, 0.5f); // 红色
                case SpawnType.Both:
                    return new Color(0.5f, 1.0f, 0.0f, 0.5f); // 绿色
                default:
                    return GizmoColor;
            }
        }

        private void log_info(string message)
        {
            Debug.Log("[SpawnPoint] " + name + ": " + message);
        }
    }
}
