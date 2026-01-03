using GameEngine;
using GameEngine.ECS;
using System.Collections;

namespace Components
{
    /// <summary>
    /// 敌人AI - 状态机驱动的AI行为
    /// </summary>
    public class Enemy : Component
    {
        [Header("Stats")]
        public int MaxHealth = 100;
        public int Damage = 10;
        public float MoveSpeed = 3.0f;
        public float DetectionRange = 20.0f;
        public float AttackRange = 5.0f;
        public float AttackRate = 1.0f;

        [Header("AI")]
        public float PatrolRadius = 10.0f;
        public Transform[] PatrolPoints;

        [Header("Visual")]
        public Material NormalMaterial;
        public Material HitMaterial;
        public Material DeadMaterial;

        // 状态枚举
        private enum State
        {
            Idle,
            Patrol,
            Chase,
            Attack,
            Dead
        }

        // 当前状态
        private State currentState = State.Idle;

        // 组件引用
        private Transform player;
        private NavMeshAgent agent;
        private Health healthComponent;
        private Renderer renderer;
        private Animator animator;

        // 巡逻逻辑
        private int currentPatrolIndex = 0;
        private Vector3 randomPatrolPoint;

        // 战斗逻辑
        private float lastAttackTime;
        private PlayerController lastHitBy;

        // 效果
        private Coroutine hitEffectCoroutine;

        private void Start()
        {
            // 获取组件
            agent = GetComponent<NavMeshAgent>();
            healthComponent = GetComponent<Health>();
            renderer = GetComponentInChildren<Renderer>();
            animator = GetComponent<Animator>();

            // 查找玩家
            player = FindObjectOfType<PlayerController>()?.Transform;

            // 配置导航代理
            if (agent != null)
            {
                agent.speed = MoveSpeed;
                agent.stoppingDistance = AttackRange * 0.8f;
            }

            // 初始状态
            if (PatrolPoints != null && PatrolPoints.Length > 0)
            {
                currentState = State.Patrol;
            }
            else
            {
                currentState = State.Idle;
                SetRandomPatrolPoint();
            }
        }

        private void Update(float deltaTime)
        {
            // 检查死亡
            if (healthComponent != null && healthComponent.CurrentHealth <= 0)
            {
                if (currentState != State.Dead)
                {
                    Die();
                }
                return;
            }

            // 状态机
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

                case State.Dead:
                    // 死亡状态不更新
                    break;
            }

            // 更新动画
            UpdateAnimations();
        }

        private void UpdateIdle()
        {
            // 检测玩家
            if (IsPlayerInRange())
            {
                TransitionToState(State.Chase);
                return;
            }

            // 偶尔切换到巡逻
            if (Random.value < 0.01f)
            {
                if (PatrolPoints != null && PatrolPoints.Length > 0)
                {
                    TransitionToState(State.Patrol);
                }
                else
                {
                    SetRandomPatrolPoint();
                    TransitionToState(State.Patrol);
                }
            }
        }

        private void UpdatePatrol()
        {
            // 移动到下一个巡逻点
            Vector3 targetPoint = GetPatrolPoint();

            if (agent != null)
            {
                agent.SetDestination(targetPoint);

                // 检查是否到达
                if (!agent.pathPending && agent.remainingDistance < 0.5f)
                {
                    OnPatrolPointReached();
                }
            }

            // 检测玩家
            if (IsPlayerInRange())
            {
                TransitionToState(State.Chase);
            }
        }

        private void UpdateChase()
        {
            if (player == null)
            {
                TransitionToState(State.Patrol);
                return;
            }

            // 追逐玩家
            if (agent != null)
            {
                agent.SetDestination(player.position);
            }

            // 检查攻击范围
            float distance = Vector3.Distance(Transform.position, player.position);
            if (distance <= AttackRange)
            {
                TransitionToState(State.Attack);
            }

            // 检查是否丢失玩家
            if (distance > DetectionRange * 1.5f)
            {
                TransitionToState(State.Patrol);
            }
        }

        private void UpdateAttack()
        {
            if (player == null)
            {
                TransitionToState(State.Patrol);
                return;
            }

            // 面向玩家
            Transform.LookAt(player);

            // 攻击
            float distance = Vector3.Distance(Transform.position, player.position);

            if (distance <= AttackRange)
            {
                if (Time.time - lastAttackTime >= AttackRate)
                {
                    PerformAttack();
                    lastAttackTime = Time.time;
                }
            }
            else
            {
                // 玩家离开攻击范围，追逐
                TransitionToState(State.Chase);
            }
        }

        private Vector3 GetPatrolPoint()
        {
            if (PatrolPoints != null && PatrolPoints.Length > 0)
            {
                return PatrolPoints[currentPatrolIndex].position;
            }
            return randomPatrolPoint;
        }

        private void OnPatrolPointReached()
        {
            if (PatrolPoints != null && PatrolPoints.Length > 0)
            {
                // 移动到下一个巡逻点
                currentPatrolIndex = (currentPatrolIndex + 1) % PatrolPoints.Length;
            }
            else
            {
                // 设置新的随机巡逻点
                SetRandomPatrolPoint();
            }

            // 停留一小段时间
            if (agent != null)
            {
                agent.isStopped = true;
                StartCoroutine(ResumePatrolAfterDelay(1.0f));
            }
        }

        private IEnumerator ResumePatrolAfterDelay(float delay)
        {
            yield return new WaitForSeconds(delay);
            if (agent != null)
            {
                agent.isStopped = false;
            }
        }

        private void SetRandomPatrolPoint()
        {
            Vector3 randomDirection = Random.insideUnitSphere * PatrolRadius;
            randomDirection.y = 0;
            randomPatrolPoint = Transform.position + randomDirection;
        }

        private bool IsPlayerInRange()
        {
            if (player == null)
            {
                return false;
            }

            float distance = Vector3.Distance(Transform.position, player.position);
            return distance <= DetectionRange;
        }

        private void TransitionToState(State newState)
        {
            if (currentState == newState)
            {
                return;
            }

            // 退出当前状态
            OnExitState(currentState);

            // 进入新状态
            currentState = newState;
            OnEnterState(newState);
        }

        private void OnEnterState(State state)
        {
            switch (state)
            {
                case State.Chase:
                    // 播放追逐音效
                    PlayAlertSound();
                    break;

                case State.Attack:
                    // 播放攻击音效
                    PlayAttackSound();
                    break;
            }
        }

        private void OnExitState(State state)
        {
            // 状态退出逻辑
        }

        private void PerformAttack()
        {
            if (player == null)
            {
                return;
            }

            // 播放攻击动画
            if (animator != null)
            {
                animator.SetTrigger("Attack");
            }

            // 播放攻击音效
            PlayAttackSound();

            // 对玩家造成伤害
            PlayerController playerController = player.GetComponent<PlayerController>();
            if (playerController != null)
            {
                playerController.TakeDamage(Damage);
            }
        }

        public void TakeDamage(int damage)
        {
            if (currentState == State.Dead)
            {
                return;
            }

            // 扣除生命值
            if (healthComponent != null)
            {
                healthComponent.TakeDamage(damage);
            }

            // 记录最后攻击者
            if (player != null)
            {
                lastHitBy = player.GetComponent<PlayerController>();
            }

            // 显示受击效果
            ShowHitEffect();

            // 如果发现玩家，切换到追逐状态
            if (currentState == State.Idle || currentState == State.Patrol)
            {
                if (IsPlayerInRange())
                {
                    TransitionToState(State.Chase);
                }
            }
        }

        private void ShowHitEffect()
        {
            // 停止之前的受击效果
            if (hitEffectCoroutine != null)
            {
                StopCoroutine(hitEffectCoroutine);
            }

            // 启动新的受击效果
            hitEffectCoroutine = StartCoroutine(HitEffectCoroutine());
        }

        private IEnumerator HitEffectCoroutine()
        {
            if (renderer != null && HitMaterial != null)
            {
                Material originalMaterial = renderer.material;
                renderer.material = HitMaterial;

                yield return new WaitForSeconds(0.1f);

                renderer.material = originalMaterial;
            }

            hitEffectCoroutine = null;
        }

        private void Die()
        {
            TransitionToState(State.Dead);

            // 播放死亡动画
            if (animator != null)
            {
                animator.SetTrigger("Die");
            }

            // 改变材质
            if (renderer != null && DeadMaterial != null)
            {
                renderer.material = DeadMaterial;
            }

            // 播放死亡音效
            PlayDeathSound();

            // 禁用碰撞体
            Collider collider = GetComponent<Collider>();
            if (collider != null)
            {
                collider.enabled = false;
            }

            // 禁用导航代理
            if (agent != null)
            {
                agent.enabled = false;
            }

            // 通知游戏管理器
            GameMode gameManager = FindObjectOfType<GameMode>();
            if (gameManager != null)
            {
                gameManager.OnEnemyKilled(this);
            }

            // 延迟销毁
            Destroy(gameObject, 2.0f);
        }

        private void UpdateAnimations()
        {
            if (animator == null)
            {
                return;
            }

            // 更新移动动画
            float moveSpeed = 0f;
            if (agent != null && !agent.isStopped)
            {
                moveSpeed = agent.velocity.magnitude;
            }

            animator.SetFloat("Speed", moveSpeed);

            // 更新状态动画
            animator.SetInteger("State", (int)currentState);
        }

        private void PlayAlertSound()
        {
            AudioSource audioSource = GetComponent<AudioSource>();
            if (audioSource != null)
            {
                // 播放警报音效
                audioSource.Play();
            }
        }

        private void PlayAttackSound()
        {
            AudioSource audioSource = GetComponent<AudioSource>();
            if (audioSource != null)
            {
                // 播放攻击音效
                audioSource.Play();
            }
        }

        private void PlayDeathSound()
        {
            AudioSource audioSource = GetComponent<AudioSource>();
            if (audioSource != null)
            {
                // 播放死亡音效
                audioSource.Play();
            }
        }

        public PlayerController LastHitBy
        {
            get { return lastHitBy; }
        }

        public State CurrentState
        {
            get { return currentState; }
        }
    }
}
