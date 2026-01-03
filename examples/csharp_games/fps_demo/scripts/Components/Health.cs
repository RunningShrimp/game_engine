using GameEngine;
using GameEngine.ECS;
using System;

namespace Components
{
    /// <summary>
    /// 生命值组件 - 管理实体生命值
    /// </summary>
    public class Health : Component
    {
        [Header("Health Settings")]
        public int MaxHealth = 100;
        public int CurrentHealth { get; private set; }
        public bool RegenerateEnabled { get; set; }
        public float RegenerationRate { get; set; } = 5.0f;
        public float RegenerationDelay { get; set; } = 3.0f;

        [Header("Death Settings")]
        public bool DestroyOnDeath = true;
        public float DestroyDelay = 2.0f;

        // 事件
        public event Action<int, int> OnHealthChanged; // (current, max)
        public event Action OnDeath;
        public event Action<int> OnDamageTaken; // damage amount

        // 内部状态
        private float lastDamageTime;
        private bool isDead;

        private void Awake()
        {
            CurrentHealth = MaxHealth;
        }

        private void Start()
        {
            // 通知初始生命值
            OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);
        }

        private void Update(float deltaTime)
        {
            if (isDead)
            {
                return;
            }

            // 生命值再生
            if (RegenerateEnabled && CurrentHealth < MaxHealth)
            {
                // 检查是否可以再生（距离上次受伤超过延迟时间）
                if (Time.time - lastDamageTime >= RegenerationDelay)
                {
                    Regenerate(deltaTime);
                }
            }
        }

        public void TakeDamage(int damage)
        {
            if (isDead || damage <= 0)
            {
                return;
            }

            // 扣除生命值
            CurrentHealth = Mathf.Max(0, CurrentHealth - damage);
            lastDamageTime = Time.time;

            // 触发事件
            OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);
            OnDamageTaken?.Invoke(damage);

            // 检查死亡
            if (CurrentHealth <= 0)
            {
                Die();
            }
        }

        public void Heal(int amount)
        {
            if (isDead || amount <= 0)
            {
                return;
            }

            // 增加生命值
            CurrentHealth = Mathf.Min(MaxHealth, CurrentHealth + amount);

            // 触发事件
            OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);
        }

        public void SetHealth(int health)
        {
            if (isDead)
            {
                return;
            }

            CurrentHealth = Mathf.Clamp(health, 0, MaxHealth);

            // 触发事件
            OnHealthChanged?.Invoke(CurrentHealth, MaxHealth);

            // 检查死亡
            if (CurrentHealth <= 0)
            {
                Die();
            }
        }

        private void Regenerate(float deltaTime)
        {
            int regenerationAmount = Mathf.RoundToInt(RegenerationRate * deltaTime);

            if (regenerationAmount > 0)
            {
                Heal(regenerationAmount);
            }
        }

        private void Die()
        {
            if (isDead)
            {
                return;
            }

            isDead = true;

            // 触发死亡事件
            OnDeath?.Invoke();

            // 销毁对象
            if (DestroyOnDeath)
            {
                Destroy(gameObject, DestroyDelay);
            }
        }

        public float GetHealthPercentage()
        {
            return (float)CurrentHealth / MaxHealth;
        }

        public bool IsDead
        {
            get { return isDead; }
        }

        public bool IsFullHealth
        {
            get { return CurrentHealth >= MaxHealth; }
        }
    }
}
