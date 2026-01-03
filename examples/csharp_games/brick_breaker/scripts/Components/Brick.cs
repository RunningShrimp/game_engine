using GameEngine;
using GameEngine.ECS;
using System.Collections;

namespace BrickBreaker
{
    /// <summary>
    /// 砖块组件 - 定义砖块属性和行为
    /// </summary>
    public class Brick : Component
    {
        // ========== 公共属性 ==========

        /// <summary>砖块分数值</summary>
        public int Points { get; set; } = 10;

        /// <summary>需要击打的次数</summary>
        public int Hits { get; set; } = 1;

        /// <summary>砖块颜色</summary>
        public Color Color { get; set; } = Color.White;

        /// <summary>砖块类型</summary>
        public BrickType Type { get; set; } = BrickType.Normal;

        /// <summary>是否不可破坏</summary>
        public bool Indestructible { get; set; } = false;

        // ========== 私有字段 ==========

        private int currentHits;
        private SpriteRenderer spriteRenderer;
        private float shakeIntensity = 0.2f;

        // ========== 生命周期方法 ==========

        public void Awake()
        {
            currentHits = Hits;
            spriteRenderer = GetComponent<SpriteRenderer>();

            // 设置颜色
            if (spriteRenderer != null)
            {
                spriteRenderer.Color = Color;
            }
        }

        // ========== 公共方法 ==========

        /// <summary>被球击中时调用</summary>
        public void OnHit()
        {
            if (Indestructible)
            {
                PlayHitSound();
                return;
            }

            currentHits--;

            if (currentHits <= 0)
            {
                DestroyBrick();
            }
            else
            {
                TakeDamage();
            }
        }

        /// <summary>销毁砖块</summary>
        private void DestroyBrick()
        {
            // 播放音效
            Audio.PlaySound("brick_break");

            // 创建粒子效果
            ParticleSystem.Spawn(Transform.Position, Color, 20);

            // 增加分数
            var gameManager = FindObjectOfType<GameManager>();
            if (gameManager != null)
            {
                gameManager.AddScore(Points);
            }

            // 根据类型触发特殊效果
            if (Type == BrickType.Explosive)
            {
                Explode();
            }
            else if (Type == BrickType.PowerUp)
            {
                SpawnPowerUp();
            }

            // 销毁实体
            World.DestroyEntity(Entity);
        }

        /// <summary>砖块受损（改变外观）</summary>
        private void TakeDamage()
        {
            // 播放音效
            Audio.PlaySound("brick_hit");

            // 震动效果
            StartCoroutine(ShakeRoutine());

            // 改变透明度
            Color.a *= 0.7f;
            if (spriteRenderer != null)
            {
                spriteRenderer.Color = Color;
            }
        }

        /// <summary>爆炸效果（爆炸型砖块）</summary>
        private void Explode()
        {
            float explosionRadius = 2.0f;

            // 找到附近的砖块
            Brick[] nearbyBricks = FindObjectsOfType<Brick>();
            foreach (Brick brick in nearbyBricks)
            {
                if (brick == this) continue;

                float distance = Vector3.Distance(Transform.Position, brick.Transform.Position);
                if (distance < explosionRadius)
                {
                    brick.OnHit();
                }
            }

            // 播放爆炸音效和效果
            Audio.PlaySound("explosion");
            ParticleSystem.SpawnExplosion(Transform.Position);
        }

        /// <summary>生成道具</summary>
        private void SpawnPowerUp()
        {
            // 随机选择道具类型
            PowerUpType powerUpType = GetRandomPowerUp();

            // 创建道具实体
            Entity powerUpEntity = World.CreateEntity();
            powerUpEntity.Transform.Position = Transform.Position;

            PowerUp powerUp = powerUpEntity.AddComponent<PowerUp>();
            powerUp.Type = powerUpType;

            // 设置视觉
            SpriteRenderer spriteRenderer = powerUpEntity.AddComponent<SpriteRenderer>();
            spriteRenderer.Sprite = Resources.Load<Sprite>("PowerUps/" + powerUpType.ToString());
        }

        private PowerUpType GetRandomPowerUp()
        {
            float random = Random.value;

            if (random < 0.3f)
                return PowerUpType.WidenPaddle;
            else if (random < 0.5f)
                return PowerUpType.MultiBall;
            else if (random < 0.7f)
                return PowerUpType.SlowBall;
            else if (random < 0.85f)
                return PowerUpType.StickyPaddle;
            else
                return PowerUpType.ExtraLife;
        }

        /// <summary>震动效果</summary>
        private IEnumerator ShakeRoutine()
        {
            float elapsed = 0f;
            float duration = 0.2f;

            Vector3 originalPosition = Transform.Position;

            while (elapsed < duration)
            {
                elapsed += Time.deltaTime;

                Vector3 offset = Random.insideUnitSphere * shakeIntensity;
                offset.z = 0;
                Transform.Position = originalPosition + offset;

                yield return null;
            }

            Transform.Position = originalPosition;
        }

        private void PlayHitSound()
        {
            Audio.PlaySound("brick_metal");
        }
    }

    /// <summary>
    /// 砖块类型枚举
    /// </summary>
    public enum BrickType
    {
        /// <summary>普通砖块</summary>
        Normal,

        /// <summary>爆炸砖块（连锁破坏）</summary>
        Explosive,

        /// <summary>道具砖块（掉落道具）</summary>
        PowerUp,

        /// <summary>多段砖块（需要多次击打）</summary>
        MultiHit,

        /// <summary>不可破坏的砖块</summary>
        Indestructible
    }

    /// <summary>
    /// 道具类型枚举
    /// </summary>
    public enum PowerUpType
    {
        /// <summary>加宽挡板</summary>
        WidenPaddle,

        /// <summary>多球</summary>
        MultiBall,

        /// <summary>减速球</summary>
        SlowBall,

        /// <summary>粘性挡板</summary>
        StickyPaddle,

        /// <summary>额外生命</summary>
        ExtraLife,

        /// <summary>激光</summary>
        Laser,

        /// <summary>穿透球</summary>
        PiercingBall
    }
}
