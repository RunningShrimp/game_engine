using GameEngine;
using GameEngine.ECS;

namespace BrickBreaker
{
    /// <summary>
    /// 球组件 - 控制球的移动和物理行为
    /// </summary>
    public class Ball : Component
    {
        // ========== 公共属性 ==========

        /// <summary>球的速度（向量）</summary>
        public Vector2 Velocity { get; set; } = Vector2.Up;

        /// <summary>球的移动速度</summary>
        public float Speed { get; set; } = 300.0f;

        /// <summary>是否附加到挡板上</summary>
        public bool AttachedToPaddle { get; set; } = true;

        // ========== 生命周期方法 ==========

        /// <summary>每帧更新</summary>
        public void Update(float deltaTime)
        {
            if (AttachedToPaddle)
            {
                // 跟随挡板移动
                var paddle = FindObjectOfType<Paddle>();
                if (paddle != null)
                {
                    Transform.Position = new Vector3(
                        paddle.Transform.Position.x,
                        paddle.Transform.Position.y + 0.3f,
                        0
                    );
                }

                // 等待玩家按空格键发射
                if (Input.GetKeyDown(KeyCode.Space))
                {
                    Launch();
                }

                return;
            }

            // 移动球
            Transform.Position += new Vector3(Velocity.x, Velocity.y, 0) * Speed * deltaTime;

            // 墙壁碰撞检测
            CheckWallCollisions();
        }

        // ========== 公共方法 ==========

        /// <summary>发射球</summary>
        public void Launch()
        {
            if (!AttachedToPaddle) return;

            AttachedToPaddle = false;

            // 随机向左或向右发射
            float angle = Random.Range(-30f, 30f);
            Velocity = Quaternion.Euler(0, 0, angle) * Vector2.Up;
        }

        /// <summary>重置球到挡板上</summary>
        public void Reset()
        {
            AttachedToPaddle = true;
            Velocity = Vector2.Up;
            Speed = 300.0f;
        }

        /// <summary>加速球</summary>
        public void SpeedUp(float multiplier = 1.1f)
        {
            Speed *= multiplier;
            Speed = Mathf.Min(Speed, 800.0f); // 限制最大速度
        }

        // ========== 私有方法 ==========

        private void CheckWallCollisions()
        {
            float x = Transform.Position.x;
            float y = Transform.Position.y;

            // 左右墙壁
            if (x < -8.5f || x > 8.5f)
            {
                Velocity.x = -Velocity.x;
                Transform.Position = new Vector3(
                    Mathf.Clamp(x, -8.5f, 8.5f),
                    y,
                    0
                );

                Audio.PlaySound("wall_bounce");
            }

            // 顶部墙壁
            if (y > 6.0f)
            {
                Velocity.y = -Velocity.y;
                Transform.Position = new Vector3(x, 6.0f, 0);

                Audio.PlaySound("wall_bounce");
            }

            // 底部（掉落）
            if (y < -6.5f)
            {
                OnBallDropped();
            }
        }

        private void OnBallDropped()
        {
            Debug.Log("Ball dropped!");

            // 通知游戏管理器
            var gameManager = FindObjectOfType<GameManager>();
            if (gameManager != null)
            {
                gameManager.LoseLife();
            }

            // 重置球
            Reset();
        }
    }
}
