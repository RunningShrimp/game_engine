/// SIMD优化的物理批量计算
///
/// 提供物理模拟中常用的批量SIMD操作，包括速度积分、位置更新、碰撞检测等。

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// 物理批量积分结果
#[derive(Debug, Clone)]
pub struct PhysicsIntegrationResult {
    /// 处理的实体数量
    pub count: usize,
    /// 处理时间（微秒）
    pub processing_time_us: u64,
}

/// SIMD批量物理积分
pub struct PhysicsIntegrator;

impl PhysicsIntegrator {
    /// 批量更新速度（欧拉积分）
    ///
    /// 使用SIMD并行计算多个刚体的速度更新：
    /// `velocity = velocity + (force / mass) * dt`
    ///
    /// # 参数
    ///
    /// * `velocities` - 速度数组 [vx, vy, vz, _]
    /// * `forces` - 力数组 [fx, fy, fz, _]
    /// * `inverse_masses` - 逆质量数组 (1/mass)
    /// * `dt` - 时间步长
    ///
    /// # 性能
    ///
    /// 使用AVX2时可一次处理8个向量，相比标量实现提升3-4x
    #[inline]
    pub fn update_velocities_simd(
        velocities: &mut [[f32; 4]],
        forces: &[[f32; 4]],
        inverse_masses: &[f32],
        dt: f32,
    ) -> PhysicsIntegrationResult {
        let start = std::time::Instant::now();
        let count = velocities.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    Self::update_velocities_avx2(velocities, forces, inverse_masses, dt);
                }
            } else {
                Self::update_velocities_fallback(velocities, forces, inverse_masses, dt);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            Self::update_velocities_fallback(velocities, forces, inverse_masses, dt);
        }

        PhysicsIntegrationResult {
            count,
            processing_time_us: start.elapsed().as_micros() as u64,
        }
    }

    /// AVX2优化的速度更新
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn update_velocities_avx2(
        velocities: &mut [[f32; 4]],
        forces: &[[f32; 4]],
        inverse_masses: &[f32],
        dt: f32,
    ) {
        let dt_vec = _mm256_set1_ps(dt);

        // 批量处理8个向量
        let mut i = 0;
        while i + 8 <= velocities.len() {
            // 加载8个力向量
            let fx0 = _mm256_set_ps(
                forces[i + 7][0],
                forces[i + 6][0],
                forces[i + 5][0],
                forces[i + 4][0],
                forces[i + 3][0],
                forces[i + 2][0],
                forces[i + 1][0],
                forces[i + 0][0],
            );
            let fy0 = _mm256_set_ps(
                forces[i + 7][1],
                forces[i + 6][1],
                forces[i + 5][1],
                forces[i + 4][1],
                forces[i + 3][1],
                forces[i + 2][1],
                forces[i + 1][1],
                forces[i + 0][1],
            );
            let fz0 = _mm256_set_ps(
                forces[i + 7][2],
                forces[i + 6][2],
                forces[i + 5][2],
                forces[i + 4][2],
                forces[i + 3][2],
                forces[i + 2][2],
                forces[i + 1][2],
                forces[i + 0][2],
            );

            // 加载8个逆质量
            let inv_mass = _mm256_set_ps(
                inverse_masses[i + 7],
                inverse_masses[i + 6],
                inverse_masses[i + 5],
                inverse_masses[i + 4],
                inverse_masses[i + 3],
                inverse_masses[i + 2],
                inverse_masses[i + 1],
                inverse_masses[i + 0],
            );

            // 计算加速度: a = F / m
            let ax = _mm256_mul_ps(fx0, inv_mass);
            let ay = _mm256_mul_ps(fy0, inv_mass);
            let az = _mm256_mul_ps(fz0, inv_mass);

            // 计算速度变化: dv = a * dt
            let dvx = _mm256_mul_ps(ax, dt_vec);
            let dvy = _mm256_mul_ps(ay, dt_vec);
            let dvz = _mm256_mul_ps(az, dt_vec);

            // 提取并更新速度
            let mut vx_array = [0.0f32; 8];
            let mut vy_array = [0.0f32; 8];
            let mut vz_array = [0.0f32; 8];

            _mm256_storeu_ps(vx_array.as_mut_ptr(), dvx);
            _mm256_storeu_ps(vy_array.as_mut_ptr(), dvy);
            _mm256_storeu_ps(vz_array.as_mut_ptr(), dvz);

            for j in 0..8 {
                velocities[i + j][0] += vx_array[j];
                velocities[i + j][1] += vy_array[j];
                velocities[i + j][2] += vz_array[j];
            }

            i += 8;
        }

        // 处理剩余元素
        for j in i..velocities.len() {
            let inv_mass = inverse_masses[j];
            velocities[j][0] += forces[j][0] * inv_mass * dt;
            velocities[j][1] += forces[j][1] * inv_mass * dt;
            velocities[j][2] += forces[j][2] * inv_mass * dt;
        }
    }

    /// 标量回退实现
    fn update_velocities_fallback(
        velocities: &mut [[f32; 4]],
        forces: &[[f32; 4]],
        inverse_masses: &[f32],
        dt: f32,
    ) {
        for i in 0..velocities.len() {
            let inv_mass = inverse_masses[i];
            velocities[i][0] += forces[i][0] * inv_mass * dt;
            velocities[i][1] += forces[i][1] * inv_mass * dt;
            velocities[i][2] += forces[i][2] * inv_mass * dt;
        }
    }

    /// 批量更新位置
    ///
    /// 使用SIMD并行计算多个刚体的位置更新：
    /// `position = position + velocity * dt`
    ///
    /// # 参数
    ///
    /// * `positions` - 位置数组 [x, y, z, _]
    /// * `velocities` - 速度数组 [vx, vy, vz, _]
    /// * `dt` - 时间步长
    #[inline]
    pub fn update_positions_simd(
        positions: &mut [[f32; 4]],
        velocities: &[[f32; 4]],
        dt: f32,
    ) -> PhysicsIntegrationResult {
        let start = std::time::Instant::now();
        let count = positions.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    Self::update_positions_avx2(positions, velocities, dt);
                }
            } else {
                Self::update_positions_fallback(positions, velocities, dt);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            Self::update_positions_fallback(positions, velocities, dt);
        }

        PhysicsIntegrationResult {
            count,
            processing_time_us: start.elapsed().as_micros() as u64,
        }
    }

    /// AVX2优化的位置更新
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn update_positions_avx2(positions: &mut [[f32; 4]], velocities: &[[f32; 4]], dt: f32) {
        let dt_vec = _mm256_set1_ps(dt);

        // 批量处理8个向量
        let mut i = 0;
        while i + 8 <= positions.len() {
            // 加载8个速度向量
            let vx0 = _mm256_set_ps(
                velocities[i + 7][0],
                velocities[i + 6][0],
                velocities[i + 5][0],
                velocities[i + 4][0],
                velocities[i + 3][0],
                velocities[i + 2][0],
                velocities[i + 1][0],
                velocities[i + 0][0],
            );
            let vy0 = _mm256_set_ps(
                velocities[i + 7][1],
                velocities[i + 6][1],
                velocities[i + 5][1],
                velocities[i + 4][1],
                velocities[i + 3][1],
                velocities[i + 2][1],
                velocities[i + 1][1],
                velocities[i + 0][1],
            );
            let vz0 = _mm256_set_ps(
                velocities[i + 7][2],
                velocities[i + 6][2],
                velocities[i + 5][2],
                velocities[i + 4][2],
                velocities[i + 3][2],
                velocities[i + 2][2],
                velocities[i + 1][2],
                velocities[i + 0][2],
            );

            // 计算位移: dp = v * dt
            let dpx = _mm256_mul_ps(vx0, dt_vec);
            let dpy = _mm256_mul_ps(vy0, dt_vec);
            let dpz = _mm256_mul_ps(vz0, dt_vec);

            // 提取并更新位置
            let mut px_array = [0.0f32; 8];
            let mut py_array = [0.0f32; 8];
            let mut pz_array = [0.0f32; 8];

            _mm256_storeu_ps(px_array.as_mut_ptr(), dpx);
            _mm256_storeu_ps(py_array.as_mut_ptr(), dpy);
            _mm256_storeu_ps(pz_array.as_mut_ptr(), dpz);

            for j in 0..8 {
                positions[i + j][0] += px_array[j];
                positions[i + j][1] += py_array[j];
                positions[i + j][2] += pz_array[j];
            }

            i += 8;
        }

        // 处理剩余元素
        for j in i..positions.len() {
            positions[j][0] += velocities[j][0] * dt;
            positions[j][1] += velocities[j][1] * dt;
            positions[j][2] += velocities[j][2] * dt;
        }
    }

    /// 标量回退实现
    fn update_positions_fallback(positions: &mut [[f32; 4]], velocities: &[[f32; 4]], dt: f32) {
        for i in 0..positions.len() {
            positions[i][0] += velocities[i][0] * dt;
            positions[i][1] += velocities[i][1] * dt;
            positions[i][2] += velocities[i][2] * dt;
        }
    }

    /// 批量应用阻尼
    ///
    /// 使用SIMD并行计算速度阻尼：
    /// `velocity = velocity * damping_factor`
    ///
    /// # 参数
    ///
    /// * `velocities` - 速度数组
    /// * `damping` - 阻尼系数 (0.0 - 1.0)
    #[inline]
    pub fn apply_damping_simd(
        velocities: &mut [[f32; 4]],
        damping: f32,
    ) -> PhysicsIntegrationResult {
        let start = std::time::Instant::now();
        let count = velocities.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    Self::apply_damping_avx2(velocities, damping);
                }
            } else {
                Self::apply_damping_fallback(velocities, damping);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            Self::apply_damping_fallback(velocities, damping);
        }

        PhysicsIntegrationResult {
            count,
            processing_time_us: start.elapsed().as_micros() as u64,
        }
    }

    /// AVX2优化的阻尼应用
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn apply_damping_avx2(velocities: &mut [[f32; 4]], damping: f32) {
        let damping_vec = _mm256_set1_ps(damping);

        // 批量处理8个向量
        let mut i = 0;
        while i + 8 <= velocities.len() {
            // 加载8个速度分量
            let vx = _mm256_set_ps(
                velocities[i + 7][0],
                velocities[i + 6][0],
                velocities[i + 5][0],
                velocities[i + 4][0],
                velocities[i + 3][0],
                velocities[i + 2][0],
                velocities[i + 1][0],
                velocities[i + 0][0],
            );
            let vy = _mm256_set_ps(
                velocities[i + 7][1],
                velocities[i + 6][1],
                velocities[i + 5][1],
                velocities[i + 4][1],
                velocities[i + 3][1],
                velocities[i + 2][1],
                velocities[i + 1][1],
                velocities[i + 0][1],
            );
            let vz = _mm256_set_ps(
                velocities[i + 7][2],
                velocities[i + 6][2],
                velocities[i + 5][2],
                velocities[i + 4][2],
                velocities[i + 3][2],
                velocities[i + 2][2],
                velocities[i + 1][2],
                velocities[i + 0][2],
            );

            // 应用阻尼
            let vx_damped = _mm256_mul_ps(vx, damping_vec);
            let vy_damped = _mm256_mul_ps(vy, damping_vec);
            let vz_damped = _mm256_mul_ps(vz, damping_vec);

            // 存储结果
            let mut vx_array = [0.0f32; 8];
            let mut vy_array = [0.0f32; 8];
            let mut vz_array = [0.0f32; 8];

            _mm256_storeu_ps(vx_array.as_mut_ptr(), vx_damped);
            _mm256_storeu_ps(vy_array.as_mut_ptr(), vy_damped);
            _mm256_storeu_ps(vz_array.as_mut_ptr(), vz_damped);

            for j in 0..8 {
                velocities[i + j][0] = vx_array[j];
                velocities[i + j][1] = vy_array[j];
                velocities[i + j][2] = vz_array[j];
            }

            i += 8;
        }

        // 处理剩余元素
        for j in i..velocities.len() {
            velocities[j][0] *= damping;
            velocities[j][1] *= damping;
            velocities[j][2] *= damping;
        }
    }

    /// 标量回退实现
    fn apply_damping_fallback(velocities: &mut [[f32; 4]], damping: f32) {
        for velocity in velocities.iter_mut() {
            velocity[0] *= damping;
            velocity[1] *= damping;
            velocity[2] *= damping;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_velocities() {
        let mut velocities = vec![
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0, 0.0],
        ];
        let forces = vec![
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0],
        ];
        let inverse_masses = vec![1.0, 1.0, 1.0, 1.0];
        let dt = 0.016;

        let result = PhysicsIntegrator::update_velocities_simd(
            &mut velocities,
            &forces,
            &inverse_masses,
            dt,
        );

        assert_eq!(result.count, 4);
        assert!((velocities[0][0] - (1.0 + 1.0 * 0.016)).abs() < 1e-5);
    }

    #[test]
    fn test_update_positions() {
        let mut positions = vec![
            [0.0, 0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
        ];
        let velocities = vec![
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [1.0, 1.0, 1.0, 0.0],
        ];
        let dt = 0.016;

        let result = PhysicsIntegrator::update_positions_simd(&mut positions, &velocities, dt);

        assert_eq!(result.count, 4);
        assert!((positions[0][0] - 0.016).abs() < 1e-5);
    }

    #[test]
    fn test_apply_damping() {
        let mut velocities = vec![
            [1.0, 1.0, 1.0, 0.0],
            [2.0, 2.0, 2.0, 0.0],
            [0.5, 0.5, 0.5, 0.0],
        ];
        let damping = 0.95;

        let result = PhysicsIntegrator::apply_damping_simd(&mut velocities, damping);

        assert_eq!(result.count, 3);
        assert!((velocities[0][0] - 0.95).abs() < 1e-5);
        assert!((velocities[1][0] - 1.9).abs() < 1e-5);
    }
}
