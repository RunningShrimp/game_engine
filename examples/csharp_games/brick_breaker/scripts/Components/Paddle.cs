using GameEngine;
using GameEngine.ECS;

namespace BrickBreaker
{
    /// <summary>
    /// 挡板组件 - 控制玩家挡板的移动
    /// </summary>
    public class Paddle : Component
    {
        // ========== 公共属性 ==========

        /// <summary>挡板移动速度</summary>
        public float Speed { get; set; } = 10.0f;

        /// <summary>挡板宽度</summary>
        public float Width { get; set; } = 2.0f;

        /// <summary>挡板高度</summary>
        public float Height { get; set; } = 0.3f;

        /// <summary>是否使用鼠标控制</summary>
        public bool UseMouseControl { get; set; } = true;

        // ========== 私有字段 ==========

        private Vector3 initialPosition;

        // ========== 生命周期方法 ==========

        public void Awake()
        {
            initialPosition = new Vector3(0, -5, 0);
        }

        public void Start()
        {
            Transform.Position = initialPosition;
        }

        public void Update(float deltaTime)
        {
            if (UseMouseControl)
            {
                UpdateWithMouse();
            }
            else
            {
                UpdateWithKeyboard(deltaTime);
            }

            // 限制在屏幕范围内
            ClampPosition();
        }

        // ========== 私有方法 ==========

        private void UpdateWithKeyboard()
        {
            float horizontal = Input.GetAxis("Horizontal");

            Vector3 pos = Transform.Position;
            pos.x += horizontal * Speed * Time.deltaTime;
            Transform.Position = pos;
        }

        private void UpdateWithMouse()
        {
            // 将鼠标位置转换为世界坐标
            Vector3 mousePos = Input.mousePosition;
            Vector3 worldPos = Camera.Main.ScreenToWorldPoint(mousePos);

            // 只更新X位置
            Vector3 pos = Transform.Position;
            pos.x = worldPos.x;
            Transform.Position = pos;
        }

        private void ClampPosition()
        {
            Vector3 pos = Transform.Position;

            // 计算边界
            float halfWidth = Width / 2f;
            float minX = -8.5f + halfWidth;
            float maxX = 8.5f - halfWidth;

            // 限制位置
            pos.x = Mathf.Clamp(pos.x, minX, maxY);

            Transform.Position = pos;
        }

        // ========== 公共方法 ==========

        /// <summary>加宽挡板</summary>
        public void Widen(float multiplier = 1.5f)
        {
            Width *= multiplier;
            Width = Mathf.Min(Width, 4.0f); // 限制最大宽度
            UpdateSize();
        }

        /// <summary>缩小挡板</summary>
        public void Shrink(float multiplier = 0.7f)
        {
            Width *= multiplier;
            Width = Mathf.Max(Width, 1.0f); // 限制最小宽度
            UpdateSize();
        }

        /// <summary>重置挡板大小</summary>
        public void ResetSize()
        {
            Width = 2.0f;
            UpdateSize();
        }

        private void UpdateSize()
        {
            // 更新视觉表示
            Transform.Scale = new Vector3(Width, Height, 1);
        }

        /// <summary>使挡板具有粘性（抓住球）</summary>
        public void MakeSticky(float duration = 5.0f)
        {
            StartCoroutine(StickyRoutine(duration));
        }

        private IEnumerator StickyRoutine(float duration)
        {
            // 查找球并附加
            var ball = FindObjectOfType<Ball>();
            if (ball != null)
            {
                ball.AttachedToPaddle = true;
            }

            yield return new WaitForSeconds(duration);

            // 恢复正常
            if (ball != null)
            {
                ball.Launch();
            }
        }
    }
}
