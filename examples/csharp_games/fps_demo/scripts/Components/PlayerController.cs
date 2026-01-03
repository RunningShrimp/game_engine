using GameEngine;
using GameEngine.ECS;

namespace Components
{
    /// <summary>
    /// 玩家控制器 - 处理第一人称移动和输入
    /// </summary>
    public class PlayerController : Component
    {
        [Header("Movement")]
        public float WalkSpeed = 5.0f;
        public float SprintSpeed = 8.0f;
        public float JumpForce = 5.0f;

        [Header("Mouse Look")]
        public float MouseSensitivity = 100.0f;
        public float MaxPitchAngle = 89.0f;

        private CharacterController controller;
        private FirstPersonCamera camera;
        private float pitch = 0f;
        private float yaw = 0f;

        private void Start()
        {
            controller = GetComponent<CharacterController>();
            camera = FindObjectOfType<FirstPersonCamera>();

            // 锁定鼠标
            Cursor.LockState = CursorLockMode.Locked;
            Cursor.Visible = false;
        }

        private void Update(float deltaTime)
        {
            HandleMovement();
            HandleMouseLook();
            HandleJump();
        }

        private void HandleMovement()
        {
            // 获取输入
            Vector3 moveDirection = Vector3.Zero;

            if (Input.GetKey(KeyCode.W)) moveDirection += Vector3.Forward;
            if (Input.GetKey(KeyCode.S)) moveDirection += Vector3.Back;
            if (Input.GetKey(KeyCode.A)) moveDirection += Vector3.Left;
            if (Input.GetKey(KeyCode.D)) moveDirection += Vector3.Right;

            // 应用冲刺
            float speed = Input.GetKey(KeyCode.LeftShift) ? SprintSpeed : WalkSpeed;

            // 移动（相对于玩家朝向）
            moveDirection = Transform.rotation * moveDirection;
            controller.Move(moveDirection.normalized * speed * deltaTime);
        }

        private void HandleMouseLook()
        {
            // 鼠标输入
            float mouseX = Input.GetAxis("Mouse X") * MouseSensitivity;
            float mouseY = Input.GetAxis("Mouse Y") * MouseSensitivity;

            // 水平旋转（身体）
            yaw += mouseX * Time.deltaTime;

            // 垂直旋转（头部）
            pitch -= mouseY * Time.deltaTime;
            pitch = Mathf.Clamp(pitch, -MaxPitchAngle, MaxPitchAngle);

            // 应用旋转
            Transform.rotation = Quaternion.Euler(0, yaw, 0);
            if (camera != null)
            {
                camera.Transform.localRotation = Quaternion.Euler(pitch, 0, 0);
            }
        }

        private void HandleJump()
        {
            if (Input.GetKeyDown(KeyCode.Space) && controller.IsGrounded)
            {
                controller.Jump(JumpForce);
            }
        }

        public void TakeDamage(float damage)
        {
            // 获取生命值组件
            Health health = GetComponent<Health>();
            if (health != null)
            {
                health.TakeDamage(damage);
            }

            // 屏幕血红效果
            UI.ShowDamageEffect();
        }

        public void AddScore(int points)
        {
            GameMode gameMode = FindObjectOfType<GameMode>();
            if (gameMode != null)
            {
                gameMode.AddScore(points);
            }
        }
    }
}
