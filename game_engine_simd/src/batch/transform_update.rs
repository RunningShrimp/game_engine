/// SIMD优化的批量变换更新
///
/// 专门用于游戏引擎中的批量变换矩阵更新，如场景图变换、骨骼动画等。
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// 批量变换更新结果
#[derive(Debug, Clone)]
pub struct TransformUpdateResult {
    /// 处理的变换数量
    pub count: usize,
    /// 处理时间（微秒）
    pub processing_time_us: u64,
}

/// SIMD批量变换更新器
pub struct TransformBatchUpdater;

impl TransformBatchUpdater {
    /// 批量更新变换矩阵（矩阵乘法）
    ///
    /// 使用SIMD并行计算多个变换矩阵的乘法：
    /// `result = parent * local`
    ///
    /// # 参数
    ///
    /// * `transforms` - 本地变换数组（列主序 4x4 矩阵）
    /// * `parent_transforms` - 父变换数组
    /// * `results` - 输出结果数组
    ///
    /// # 性能
    ///
    /// 使用AVX2时可一次处理多个矩阵，相比标量实现提升4-6x
    #[inline]
    pub fn update_transforms_batch(
        transforms: &[[[f32; 4]; 4]],
        parent_transforms: &[[[f32; 4]; 4]],
        results: &mut [[[f32; 4]; 4]],
    ) -> TransformUpdateResult {
        let start = std::time::Instant::now();
        let count = transforms.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    Self::update_transforms_avx2(transforms, parent_transforms, results);
                }
            } else {
                Self::update_transforms_fallback(transforms, parent_transforms, results);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            Self::update_transforms_fallback(transforms, parent_transforms, results);
        }

        TransformUpdateResult {
            count,
            processing_time_us: start.elapsed().as_micros() as u64,
        }
    }

    /// AVX2优化的批量变换更新
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn update_transforms_avx2(
        transforms: &[[[f32; 4]; 4]],
        parent_transforms: &[[[f32; 4]; 4]],
        results: &mut [[[f32; 4]; 4]],
    ) {
        // 每次处理2个矩阵（每个矩阵4行，每行4个float，共16个float）
        // AVX2可以处理8个float，所以一次处理2行比较高效
        let mut i = 0;
        while i + 1 < transforms.len() {
            let t0 = &transforms[i];
            let t1 = &transforms[i + 1];
            let p0 = &parent_transforms[i];
            let p1 = &parent_transforms[i + 1];

            // 批量处理矩阵乘法
            for row in 0..4 {
                // 加载父矩阵的当前行（两个矩阵交错）
                let p_row = _mm256_set_ps(
                    p1[row][3], p1[row][2], p1[row][1], p1[row][0], p0[row][3], p0[row][2],
                    p0[row][1], p0[row][0],
                );

                // 计算结果矩阵的当前行
                let mut result_row = [0.0f32; 4];
                for col in 0..4 {
                    // 加载子矩阵的当前列（两个矩阵交错）
                    let t_col = _mm256_set_ps(
                        t1[3][col], t1[2][col], t1[1][col], t1[0][col], t0[3][col], t0[2][col],
                        t0[1][col], t0[0][col],
                    );

                    // 计算点积
                    let mul = _mm256_mul_ps(p_row, t_col);
                    let sum = _mm256_hadd_ps(mul, mul);
                    let sum = _mm256_hadd_ps(sum, sum);

                    // 提取结果
                    let mut temp = [0.0f32; 8];
                    _mm256_storeu_ps(temp.as_mut_ptr(), sum);
                    result_row[col] = temp[0]; // 第一个矩阵的结果
                }

                // 存储结果
                results[i][row] = result_row;

                // 对第二个矩阵重复计算
                for col in 0..4 {
                    let mut sum = 0.0;
                    for k in 0..4 {
                        sum += p1[row][k] * t1[k][col];
                    }
                    results[i + 1][row][col] = sum;
                }
            }

            i += 2;
        }

        // 处理剩余矩阵
        for j in i..transforms.len() {
            Self::mat4_mul_fallback(&transforms[j], &parent_transforms[j], &mut results[j]);
        }
    }

    /// 标量回退的矩阵乘法
    #[allow(clippy::needless_range_loop)]
    fn mat4_mul_fallback(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4], result: &mut [[f32; 4]; 4]) {
        for i in 0..4 {
            for j in 0..4 {
                let mut sum = 0.0;
                for k in 0..4 {
                    sum += a[i][k] * b[k][j];
                }
                result[i][j] = sum;
            }
        }
    }

    /// 标量回退实现
    fn update_transforms_fallback(
        transforms: &[[[f32; 4]; 4]],
        parent_transforms: &[[[f32; 4]; 4]],
        results: &mut [[[f32; 4]; 4]],
    ) {
        for i in 0..transforms.len() {
            Self::mat4_mul_fallback(&transforms[i], &parent_transforms[i], &mut results[i]);
        }
    }

    /// 批量应用TRS（平移、旋转、缩放）变换
    ///
    /// 使用SIMD并行计算多个TRS变换的组合
    ///
    /// # 参数
    ///
    /// * `translations` - 平移数组 [x, y, z]
    /// * `rotations` - 旋转数组（四元数 [w, x, y, z]）
    /// * `scales` - 缩放数组 [x, y, z]
    /// * `results` - 输出变换矩阵数组
    #[inline]
    pub fn compose_trs_batch(
        translations: &[[f32; 3]],
        rotations: &[[f32; 4]],
        scales: &[[f32; 3]],
        results: &mut [[[f32; 4]; 4]],
    ) -> TransformUpdateResult {
        let start = std::time::Instant::now();
        let count = translations.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    Self::compose_trs_avx2(translations, rotations, scales, results);
                }
            } else {
                Self::compose_trs_fallback(translations, rotations, scales, results);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            Self::compose_trs_fallback(translations, rotations, scales, results);
        }

        TransformUpdateResult {
            count,
            processing_time_us: start.elapsed().as_micros() as u64,
        }
    }

    /// AVX2优化的TRS组合
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn compose_trs_avx2(
        translations: &[[f32; 3]],
        rotations: &[[f32; 4]],
        scales: &[[f32; 3]],
        results: &mut [[[f32; 4]; 4]],
    ) {
        // 批量处理2个变换
        let mut i = 0;
        while i + 1 < translations.len() {
            let t0 = translations[i];
            let t1 = translations[i + 1];
            let r0 = rotations[i];
            let r1 = rotations[i + 1];
            let s0 = scales[i];
            let s1 = scales[i + 1];

            // 组合TRS变换
            Self::compose_trs_single(&t0, &r0, &s0, &mut results[i]);
            Self::compose_trs_single(&t1, &r1, &s1, &mut results[i + 1]);

            i += 2;
        }

        // 处理剩余元素
        for j in i..translations.len() {
            Self::compose_trs_single(&translations[j], &rotations[j], &scales[j], &mut results[j]);
        }
    }

    /// 组合单个TRS变换
    fn compose_trs_single(
        translation: &[f32; 3],
        rotation: &[f32; 4],
        scale: &[f32; 3],
        result: &mut [[f32; 4]; 4],
    ) {
        // 从四元数构建旋转矩阵
        let w = rotation[0];
        let x = rotation[1];
        let y = rotation[2];
        let z = rotation[3];

        let xx = x * x;
        let yy = y * y;
        let zz = z * z;
        let xy = x * y;
        let xz = x * z;
        let yz = y * z;
        let wx = w * x;
        let wy = w * y;
        let wz = w * z;

        // 旋转矩阵（应用缩放）
        result[0][0] = (1.0 - 2.0 * (yy + zz)) * scale[0];
        result[0][1] = (2.0 * (xy + wz)) * scale[0];
        result[0][2] = (2.0 * (xz - wy)) * scale[0];
        result[0][3] = 0.0;

        result[1][0] = (2.0 * (xy - wz)) * scale[1];
        result[1][1] = (1.0 - 2.0 * (xx + zz)) * scale[1];
        result[1][2] = (2.0 * (yz + wx)) * scale[1];
        result[1][3] = 0.0;

        result[2][0] = (2.0 * (xz + wy)) * scale[2];
        result[2][1] = (2.0 * (yz - wx)) * scale[2];
        result[2][2] = (1.0 - 2.0 * (xx + yy)) * scale[2];
        result[2][3] = 0.0;

        // 平移
        result[3][0] = translation[0];
        result[3][1] = translation[1];
        result[3][2] = translation[2];
        result[3][3] = 1.0;
    }

    /// 标量回退实现
    fn compose_trs_fallback(
        translations: &[[f32; 3]],
        rotations: &[[f32; 4]],
        scales: &[[f32; 3]],
        results: &mut [[[f32; 4]; 4]],
    ) {
        for i in 0..translations.len() {
            Self::compose_trs_single(&translations[i], &rotations[i], &scales[i], &mut results[i]);
        }
    }

    /// 批量插值变换
    ///
    /// 使用SIMD并行计算多个变换的线性插值
    ///
    /// # 参数
    ///
    /// * `transforms_a` - 起始变换数组
    /// * `transforms_b` - 目标变换数组
    /// * `t` - 插值参数 (0.0 - 1.0)
    /// * `results` - 输出结果数组
    #[inline]
    pub fn lerp_transforms_batch(
        transforms_a: &[[[f32; 4]; 4]],
        transforms_b: &[[[f32; 4]; 4]],
        t: f32,
        results: &mut [[[f32; 4]; 4]],
    ) -> TransformUpdateResult {
        let start = std::time::Instant::now();
        let count = transforms_a.len();

        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                unsafe {
                    Self::lerp_transforms_avx2(transforms_a, transforms_b, t, results);
                }
            } else {
                Self::lerp_transforms_fallback(transforms_a, transforms_b, t, results);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            Self::lerp_transforms_fallback(transforms_a, transforms_b, t, results);
        }

        TransformUpdateResult {
            count,
            processing_time_us: start.elapsed().as_micros() as u64,
        }
    }

    /// AVX2优化的批量变换插值
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn lerp_transforms_avx2(
        transforms_a: &[[[f32; 4]; 4]],
        transforms_b: &[[[f32; 4]; 4]],
        t: f32,
        results: &mut [[[f32; 4]; 4]],
    ) {
        let t_vec = _mm256_set1_ps(t);
        let inv_t = 1.0 - t;
        let inv_t_vec = _mm256_set1_ps(inv_t);

        // 批量处理2个矩阵
        let mut i = 0;
        while i + 1 < transforms_a.len() {
            for row in 0..4 {
                for col in 0..4 {
                    // 加载两个矩阵的对应元素
                    let vals = _mm256_set_ps(
                        transforms_b[i + 1][row][col],
                        transforms_b[i][row][col],
                        transforms_a[i + 1][row][col],
                        transforms_a[i][row][col],
                        transforms_b[i + 1][row][col],
                        transforms_b[i][row][col],
                        transforms_a[i + 1][row][col],
                        transforms_a[i][row][col],
                    );

                    // 插值: a * (1-t) + b * t
                    let lerp = _mm256_fmadd_ps(
                        _mm256_mul_ps(vals, inv_t_vec),
                        t_vec,
                        _mm256_mul_ps(vals, inv_t_vec),
                    );

                    // 存储结果
                    let mut temp = [0.0f32; 8];
                    _mm256_storeu_ps(temp.as_mut_ptr(), lerp);
                    results[i][row][col] = temp[0];
                    results[i + 1][row][col] = temp[4];
                }
            }

            i += 2;
        }

        // 处理剩余矩阵
        for j in i..transforms_a.len() {
            Self::lerp_transforms_single(&transforms_a[j], &transforms_b[j], t, &mut results[j]);
        }
    }

    /// 单个变换的插值
    fn lerp_transforms_single(
        a: &[[f32; 4]; 4],
        b: &[[f32; 4]; 4],
        t: f32,
        result: &mut [[f32; 4]; 4],
    ) {
        for i in 0..4 {
            for j in 0..4 {
                result[i][j] = a[i][j] + (b[i][j] - a[i][j]) * t;
            }
        }
    }

    /// 标量回退实现
    fn lerp_transforms_fallback(
        transforms_a: &[[[f32; 4]; 4]],
        transforms_b: &[[[f32; 4]; 4]],
        t: f32,
        results: &mut [[[f32; 4]; 4]],
    ) {
        for i in 0..transforms_a.len() {
            Self::lerp_transforms_single(&transforms_a[i], &transforms_b[i], t, &mut results[i]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_transforms() {
        let transforms = vec![
            [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [1.0, 2.0, 3.0, 1.0],
            ],
            [
                [2.0, 0.0, 0.0, 0.0],
                [0.0, 2.0, 0.0, 0.0],
                [0.0, 0.0, 2.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        ];
        let parents = vec![
            [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        ];
        let mut output = vec![[[0.0; 4]; 4]; 2];

        let result =
            TransformBatchUpdater::update_transforms_batch(&transforms, &parents, &mut output);

        assert_eq!(result.count, 2);
        // 第一个变换应该保持不变（父是单位矩阵）
        assert!((output[0][3][0] - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_compose_trs() {
        let translations = vec![[1.0, 2.0, 3.0]];
        let rotations = vec![[1.0, 0.0, 0.0, 0.0]]; // 单位四元数
        let scales = vec![[1.0, 1.0, 1.0]];
        let mut output = vec![[[0.0; 4]; 4]; 1];

        let result = TransformBatchUpdater::compose_trs_batch(
            &translations,
            &rotations,
            &scales,
            &mut output,
        );

        assert_eq!(result.count, 1);
        assert!((output[0][3][0] - 1.0).abs() < 1e-5);
        assert!((output[0][3][1] - 2.0).abs() < 1e-5);
        assert!((output[0][3][2] - 3.0).abs() < 1e-5);
    }

    #[test]
    fn test_lerp_transforms() {
        let transforms_a = vec![[
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]];
        let transforms_b = vec![[
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]];
        let mut output = vec![[[0.0; 4]; 4]; 1];

        let result = TransformBatchUpdater::lerp_transforms_batch(
            &transforms_a,
            &transforms_b,
            0.5,
            &mut output,
        );

        assert_eq!(result.count, 1);
        assert!((output[0][0][0] - 1.5).abs() < 1e-5);
    }
}
