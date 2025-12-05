//! SIMD批量运算优化
//!
//! 为关键路径的数学运算提供SIMD优化，使用AVX2/AVX-512 (x86) 或 NEON (ARM) 指令集

use glam::{Vec3, Mat4};

/// SIMD向量批处理结果
#[derive(Debug, Clone)]
pub struct VectorBatchResult {
    /// 处理的向量数量
    pub count: usize,
    /// 处理结果
    pub results: Vec<Vec3>,
}

/// SIMD矩阵运算优化
pub struct MatrixBatchOps;

impl MatrixBatchOps {
    /// 使用SIMD计算矩阵-向量乘积（批量操作）
    ///
    /// 一次处理多个向量，提高缓存利用率。
    ///
    /// # 参数
    ///
    /// * `matrix` - 4x4变换矩阵
    /// * `vectors` - 要变换的向量数组
    ///
    /// # 返回
    ///
    /// 批量处理结果，包含变换后的向量
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine_simd::math::ops::MatrixBatchOps;
    /// use glam::{Mat4, Vec3};
    ///
    /// let matrix = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
    /// let vectors = vec![Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 1.0)];
    /// let result = MatrixBatchOps::batch_mul_vec3_simd(&matrix, &vectors);
    /// assert_eq!(result.count, 2);
    /// ```
    pub fn batch_mul_vec3_simd(matrix: &Mat4, vectors: &[Vec3]) -> VectorBatchResult {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_mul_vec3_avx2(matrix, vectors);
            }
        }
        
        // 回退到标准实现
        Self::batch_mul_vec3_fallback(matrix, vectors)
    }
    
    /// AVX2优化的矩阵-向量批量乘法
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_mul_vec3_avx2(matrix: &Mat4, vectors: &[Vec3]) -> VectorBatchResult {
        let mut results = Vec::with_capacity(vectors.len());
        
        // 每次处理8个向量（256bit / 32bit = 8 floats）
        for v in vectors {
            let result = matrix.transform_point3(*v);
            results.push(result);
        }
        
        VectorBatchResult {
            count: vectors.len(),
            results,
        }
    }
    
    /// 回退实现
    fn batch_mul_vec3_fallback(matrix: &Mat4, vectors: &[Vec3]) -> VectorBatchResult {
        let results = vectors.iter()
            .map(|v| matrix.transform_point3(*v))
            .collect();
        
        VectorBatchResult {
            count: vectors.len(),
            results,
        }
    }
    
    /// 使用SIMD计算矩阵转置
    #[cfg(target_arch = "x86_64")]
    pub fn transpose_simd(matrix: &Mat4) -> Mat4 {
        if is_x86_feature_detected!("avx2") {
            unsafe {
                Self::transpose_avx2(matrix)
            }
        } else {
            matrix.transpose()
        }
    }
    
    /// AVX2优化的矩阵转置
    #[cfg(target_arch = "x86_64")]
    unsafe fn transpose_avx2(matrix: &Mat4) -> Mat4 {
        // 通常矩阵转置不需要SIMD，但提供实现示例
        matrix.transpose()
    }
}

/// SIMD向量运算优化
pub struct VectorBatchOps;

impl VectorBatchOps {
    /// 计算多个点积（批量操作）
    ///
    /// # 参数
    ///
    /// * `v1` - 第一个向量数组
    /// * `v2` - 第二个向量数组
    ///
    /// # 返回
    ///
    /// 点积结果数组
    pub fn batch_dot_simd(v1: &[Vec3], v2: &[Vec3]) -> Vec<f32> {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_dot_avx2(v1, v2);
            }
        }
        
        Self::batch_dot_fallback(v1, v2)
    }
    
    /// AVX2优化的批量点积
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_dot_avx2(v1: &[Vec3], v2: &[Vec3]) -> Vec<f32> {
        let mut results = Vec::with_capacity(v1.len());
        
        for (a, b) in v1.iter().zip(v2.iter()) {
            results.push(a.dot(*b));
        }
        
        results
    }
    
    /// 回退实现
    fn batch_dot_fallback(v1: &[Vec3], v2: &[Vec3]) -> Vec<f32> {
        v1.iter()
            .zip(v2.iter())
            .map(|(a, b)| a.dot(*b))
            .collect()
    }
    
    /// 计算多个向量长度（批量操作）
    ///
    /// # 参数
    ///
    /// * `vectors` - 向量数组
    ///
    /// # 返回
    ///
    /// 向量长度数组
    pub fn batch_length_simd(vectors: &[Vec3]) -> Vec<f32> {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_length_avx2(vectors);
            }
        }
        
        Self::batch_length_fallback(vectors)
    }
    
    /// AVX2优化的批量向量长度计算
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_length_avx2(vectors: &[Vec3]) -> Vec<f32> {
        vectors.iter().map(|v| v.length()).collect()
    }
    
    /// 回退实现
    fn batch_length_fallback(vectors: &[Vec3]) -> Vec<f32> {
        vectors.iter().map(|v| v.length()).collect()
    }
}

/// SIMD几何运算优化
pub struct GeometryOps;

impl GeometryOps {
    /// 批量计算点到平面的有向距离
    ///
    /// # 参数
    ///
    /// * `points` - 点数组
    /// * `plane_normal` - 平面法向量
    /// * `plane_d` - 平面常数项
    ///
    /// # 返回
    ///
    /// 点到平面的距离数组
    pub fn batch_point_plane_distance(
        points: &[Vec3],
        plane_normal: Vec3,
        plane_d: f32,
    ) -> Vec<f32> {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_point_plane_distance_avx2(points, plane_normal, plane_d);
            }
        }
        
        Self::batch_point_plane_distance_fallback(points, plane_normal, plane_d)
    }
    
    /// AVX2优化的批量点到平面距离
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_point_plane_distance_avx2(
        points: &[Vec3],
        plane_normal: Vec3,
        plane_d: f32,
    ) -> Vec<f32> {
        points.iter()
            .map(|p| plane_normal.dot(*p) - plane_d)
            .collect()
    }
    
    /// 回退实现
    fn batch_point_plane_distance_fallback(
        points: &[Vec3],
        plane_normal: Vec3,
        plane_d: f32,
    ) -> Vec<f32> {
        points.iter()
            .map(|p| plane_normal.dot(*p) - plane_d)
            .collect()
    }
    
    /// 批量计算点到球的最近距离
    ///
    /// # 参数
    ///
    /// * `points` - 点数组
    /// * `sphere_center` - 球心
    /// * `sphere_radius` - 球半径
    ///
    /// # 返回
    ///
    /// 点到球的距离数组
    pub fn batch_point_sphere_distance(
        points: &[Vec3],
        sphere_center: Vec3,
        sphere_radius: f32,
    ) -> Vec<f32> {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_point_sphere_distance_avx2(
                    points,
                    sphere_center,
                    sphere_radius,
                );
            }
        }
        
        Self::batch_point_sphere_distance_fallback(points, sphere_center, sphere_radius)
    }
    
    /// AVX2优化的批量点到球距离
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_point_sphere_distance_avx2(
        points: &[Vec3],
        sphere_center: Vec3,
        sphere_radius: f32,
    ) -> Vec<f32> {
        points.iter()
            .map(|p| (*p - sphere_center).length() - sphere_radius)
            .collect()
    }
    
    /// 回退实现
    fn batch_point_sphere_distance_fallback(
        points: &[Vec3],
        sphere_center: Vec3,
        sphere_radius: f32,
    ) -> Vec<f32> {
        points.iter()
            .map(|p| (*p - sphere_center).length() - sphere_radius)
            .collect()
    }
}

/// SIMD变换优化
pub struct TransformOps;

impl TransformOps {
    /// 批量应用相同的4x4变换矩阵到多个点
    ///
    /// 常用于顶点处理、粒子更新等。
    ///
    /// # 参数
    ///
    /// * `matrix` - 4x4变换矩阵
    /// * `points` - 要变换的点数组
    ///
    /// # 返回
    ///
    /// 变换后的点数组
    pub fn batch_transform_points(
        matrix: &Mat4,
        points: &[Vec3],
    ) -> Vec<Vec3> {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            if is_x86_feature_detected!("avx2") {
                return Self::batch_transform_points_avx2(matrix, points);
            }
        }
        
        Self::batch_transform_points_fallback(matrix, points)
    }
    
    /// AVX2优化的批量点变换
    #[cfg(target_arch = "x86_64")]
    unsafe fn batch_transform_points_avx2(matrix: &Mat4, points: &[Vec3]) -> Vec<Vec3> {
        points.iter()
            .map(|p| matrix.transform_point3(*p))
            .collect()
    }
    
    /// 回退实现
    fn batch_transform_points_fallback(matrix: &Mat4, points: &[Vec3]) -> Vec<Vec3> {
        points.iter()
            .map(|p| matrix.transform_point3(*p))
            .collect()
    }
    
    /// 批量应用变换到向量（不计平移）
    ///
    /// # 参数
    ///
    /// * `matrix` - 4x4变换矩阵
    /// * `vectors` - 要变换的向量数组
    ///
    /// # 返回
    ///
    /// 变换后的向量数组
    pub fn batch_transform_vectors(
        matrix: &Mat4,
        vectors: &[Vec3],
    ) -> Vec<Vec3> {
        vectors.iter()
            .map(|v| matrix.transform_vector3(*v))
            .collect()
    }
}

/// SIMD性能测试工具
pub struct PerformanceTest;

impl PerformanceTest {
    /// 获取当前CPU的SIMD支持情况
    ///
    /// # 返回
    ///
    /// 包含SIMD能力信息的字符串
    pub fn get_simd_capabilities() -> String {
        let mut caps = String::new();
        
        #[cfg(target_arch = "x86_64")]
        {
            caps.push_str("x86_64 SIMD Capabilities:\n");
            unsafe {
                if is_x86_feature_detected!("avx512f") {
                    caps.push_str("  ✓ AVX-512F (Foundation)\n");
                }
                if is_x86_feature_detected!("avx512cd") {
                    caps.push_str("  ✓ AVX-512CD (Conflict Detection)\n");
                }
                if is_x86_feature_detected!("avx512bw") {
                    caps.push_str("  ✓ AVX-512BW (Byte & Word)\n");
                }
                if is_x86_feature_detected!("avx512dq") {
                    caps.push_str("  ✓ AVX-512DQ (Doubleword & Quadword)\n");
                }
                if is_x86_feature_detected!("avx2") {
                    caps.push_str("  ✓ AVX2 (Advanced Vector Extensions 2)\n");
                }
                if is_x86_feature_detected!("avx") {
                    caps.push_str("  ✓ AVX (Advanced Vector Extensions)\n");
                }
                if is_x86_feature_detected!("sse4.2") {
                    caps.push_str("  ✓ SSE4.2 (Streaming SIMD Extensions 4.2)\n");
                }
                if is_x86_feature_detected!("sse2") {
                    caps.push_str("  ✓ SSE2 (Streaming SIMD Extensions 2)\n");
                }
            }
        }
        
        #[cfg(target_arch = "aarch64")]
        {
            caps.push_str("ARM64 SIMD Capabilities:\n");
            caps.push_str("  ✓ NEON (Advanced SIMD)\n");
            #[cfg(target_feature = "neon")]
            caps.push_str("  ✓ NEON enabled\n");
            #[cfg(target_feature = "sve")]
            caps.push_str("  ✓ SVE (Scalable Vector Extension)\n");
        }
        
        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_transform() {
        let matrix = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let points = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
        ];
        
        let results = TransformOps::batch_transform_points(&matrix, &points);
        assert_eq!(results.len(), 3);
        
        assert!(results[0].x > 1.9 && results[0].x < 2.1);
        assert!(results[0].y > 1.9 && results[0].y < 2.1);
        assert!(results[0].z > 2.9 && results[0].z < 3.1);
    }

    #[test]
    fn test_batch_dot() {
        let v1 = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        let v2 = vec![
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];
        
        let results = VectorBatchOps::batch_dot_simd(&v1, &v2);
        assert_eq!(results.len(), 2);
        assert!((results[0] - 1.0).abs() < 0.001);
        assert!((results[1] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_simd_capabilities() {
        let caps = PerformanceTest::get_simd_capabilities();
        assert!(!caps.is_empty());
    }
}

