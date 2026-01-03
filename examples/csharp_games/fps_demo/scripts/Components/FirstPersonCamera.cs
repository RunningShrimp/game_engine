using GameEngine;
using GameEngine.ECS;

namespace Components
{
    /// <summary>
    /// 第一人称相机 - 处理相机抖动和后坐力
    /// </summary>
    public class FirstPersonCamera : Component
    {
        [Header("Camera Settings")]
        public float FieldOfView = 70.0f;
        public float NearClipPlane = 0.1f;
        public float FarClipPlane = 1000.0f;

        [Header("Recoil")]
        public float RecoilDamping = 10.0f;
        public float RecoilReturnSpeed = 5.0f;

        private Camera camera;
        private Vector3 recoilRotation;
        private Vector3 recoilVelocity;
        private float baseFov;

        private void Awake()
        {
            camera = GetComponent<Camera>();
            baseFov = FieldOfView;

            if (camera != null)
            {
                camera.FieldOfView = FieldOfView;
                camera.NearClipPlane = NearClipPlane;
                camera.FarClipPlane = FarClipPlane;
            }
        }

        private void Update(float deltaTime)
        {
            // 恢复后坐力
            RecoverRecoil(deltaTime);

            // 应用后坐力到相机
            ApplyRecoil();
        }

        public void AddRecoil(float force)
        {
            // 垂直后坐力（向上）
            float verticalRecoil = force * Random.Range(0.8f, 1.2f);
            // 水平后坐力（随机左右）
            float horizontalRecoil = force * Random.Range(-0.3f, 0.3f);

            recoilRotation += new Vector3(-verticalRecoil, horizontalRecoil, 0.0f);

            // FOV踢起效果
            if (camera != null)
            {
                camera.FieldOfView = baseFov + force * 5.0f;
            }
        }

        private void RecoverRecoil(float deltaTime)
        {
            // 平滑恢复到零
            recoilRotation = Vector3.MoveTowards(
                recoilRotation,
                Vector3.Zero,
                RecoilReturnSpeed * deltaTime
            );

            // 恢复FOV
            if (camera != null)
            {
                camera.FieldOfView = Mathf.MoveTowards(
                    camera.FieldOfView,
                    baseFov,
                    RecoilReturnSpeed * deltaTime
                );
            }
        }

        private void ApplyRecoil()
        {
            if (camera != null)
            {
                // 应用后坐力偏移
                Quaternion currentRotation = camera.Transform.localRotation;
                Quaternion recoilOffset = Quaternion.Euler(recoilRotation);
                camera.Transform.localRotation = currentRotation * recoilOffset;
            }
        }

        public void SetFieldOfView(float fov)
        {
            baseFov = fov;
            if (camera != null)
            {
                camera.FieldOfView = fov;
            }
        }

        public Vector3 GetForwardVector()
        {
            if (camera != null)
            {
                return camera.Transform.forward;
            }
            return Transform.forward;
        }

        public Vector3 GetPosition()
        {
            if (camera != null)
            {
                return camera.Transform.position;
            }
            return Transform.position;
        }
    }
}
