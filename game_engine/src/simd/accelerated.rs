// SIMD加速模块
//
// 提供向量运算的SIMD加速版本和运行时CPU特性检测

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;
use std::mem;

// ============================================================================
// CPU特性检测
// ============================================================================

/// CPU特性
#[derive(Debug, Clone, Copy)]
pub struct CpuFeatures {
    /// SSE支持
    pub sse: bool,
    /// SSE2支持
    pub sse2: bool,
    /// SSE3支持
    pub sse3: bool,
    /// SSE4.1支持
    pub sse4_1: bool,
    /// SSE4.2支持
    pub sse4_2: bool,
    /// AVX支持
    pub avx: bool,
    /// AVX2支持
    pub avx2: bool,
    /// AVX-512F支持
    pub avx512f: bool,
    /// NEON支持（ARM）
    pub neon: bool,
}

impl CpuFeatures {
    /// 检测CPU特性
    pub fn detect() -> Self {
        #[cfg(target_arch = "x86_64")]
        {
            Self::detect_x86()
        }

        #[cfg(target_arch = "aarch64")]
        {
            Self::detect_arm()
        }

        #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
        {
            Self::none()
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn detect_x86() -> Self {
        let mut features = Self::none();

        // 检测SSE
        if is_x86_feature_detected!("sse") {
            features.sse = true;
        }

        // 检测SSE2
        if is_x86_feature_detected!("sse2") {
            features.sse2 = true;
        }

        // 检测SSE3
        if is_x86_feature_detected!("sse3") {
            features.sse3 = true;
        }

        // 检测SSE4.1
        if is_x86_feature_detected!("sse4.1") {
            features.sse4_1 = true;
        }

        // 检测SSE4.2
        if is_x86_feature_detected!("sse4.2") {
            features.sse4_2 = true;
        }

        // 检测AVX
        if is_x86_feature_detected!("avx") {
            features.avx = true;
        }

        // 检测AVX2
        if is_x86_feature_detected!("avx2") {
            features.avx2 = true;
        }

        // 检测AVX-512F
        if is_x86_feature_detected!("avx512f") {
            features.avx512f = true;
        }

        features
    }

    #[cfg(target_arch = "aarch64")]
    fn detect_arm() -> Self {
        // ARM64总是支持NEON
        Self {
            sse: false,
            sse2: false,
            sse3: false,
            sse4_1: false,
            sse4_2: false,
            avx: false,
            avx2: false,
            avx512f: false,
            neon: true,
        }
    }

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    fn none() -> Self {
        Self {
            sse: false,
            sse2: false,
            sse3: false,
            sse4_1: false,
            sse4_2: false,
            avx: false,
            avx2: false,
            avx512f: false,
            neon: false,
        }
    }

    /// 获取最佳可用的SIMD指令集
    pub fn best_simd(&self) -> SimdInstructionSet {
        if self.avx512f {
            SimdInstructionSet::AVX512F
        } else if self.avx2 {
            SimdInstructionSet::AVX2
        } else if self.avx {
            SimdInstructionSet::AVX
        } else if self.sse4_2 {
            SimdInstructionSet::SSE4_2
        } else if self.sse4_1 {
            SimdInstructionSet::SSE4_1
        } else if self.sse3 {
            SimdInstructionSet::SSE3
        } else if self.sse2 {
            SimdInstructionSet::SSE2
        } else if self.sse {
            SimdInstructionSet::SSE
        } else if self.neon {
            SimdInstructionSet::NEON
        } else {
            SimdInstructionSet::Scalar
        }
    }
}

/// SIMD指令集
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SimdInstructionSet {
    /// 标量（无SIMD）
    Scalar,
    /// SSE
    SSE,
    /// SSE2
    SSE2,
    /// SSE3
    SSE3,
    /// SSE4.1
    SSE4_1,
    /// SSE4.2
    SSE4_2,
    /// AVX
    AVX,
    /// AVX2
    AVX2,
    /// AVX-512F
    AVX512F,
    /// NEON (ARM)
    NEON,
}

impl SimdInstructionSet {
    /// 获取向量宽度（位数）
    pub fn vector_width(&self) -> usize {
        match self {
            SimdInstructionSet::Scalar => 32,
            SimdInstructionSet::SSE => 128,
            SimdInstructionSet::SSE2 => 128,
            SimdInstructionSet::SSE3 => 128,
            SimdInstructionSet::SSE4_1 => 128,
            SimdInstructionSet::SSE4_2 => 128,
            SimdInstructionSet::AVX => 256,
            SimdInstructionSet::AVX2 => 256,
            SimdInstructionSet::AVX512F => 512,
            SimdInstructionSet::NEON => 128,
        }
    }

    /// 获取f32向量元素数量
    pub fn f32_lanes(&self) -> usize {
        self.vector_width() / 32
    }
}

// ============================================================================
// SIMD向量运算
// ============================================================================

/// SIMD向量运算
pub struct SimdVecOps {
    features: CpuFeatures,
}

impl SimdVecOps {
    /// 创建新的SIMD向量运算
    pub fn new() -> Self {
        Self {
            features: CpuFeatures::detect(),
        }
    }

    /// 获取CPU特性
    pub fn features(&self) -> &CpuFeatures {
        &self.features
    }

    /// 向量加法（SIMD加速）
    pub fn add_vec3(&self, a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        #[cfg(target_arch = "x86_64")]
        {
            if self.features.avx {
                self.add_vec3_avx(a, b)
            } else if self.features.sse2 {
                self.add_vec3_sse2(a, b)
            } else {
                self.add_vec3_scalar(a, b)
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.add_vec3_scalar(a, b)
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx")]
    unsafe fn add_vec3_avx(&self, a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        let a_vec = _mm_set_ps(0.0, a[2], a[1], a[0]);
        let b_vec = _mm_set_ps(0.0, b[2], b[1], b[0]);
        let result = _mm_add_ps(a_vec, b_vec);

        let mut output = [0.0; 3];
        _mm_storeu_ps(output.as_mut_ptr(), result);
        output
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn add_vec3_sse2(&self, a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        let a_vec = _mm_set_ps(0.0, a[2], a[1], a[0]);
        let b_vec = _mm_set_ps(0.0, b[2], b[1], b[0]);
        let result = _mm_add_ps(a_vec, b_vec);

        let mut output = [0.0; 3];
        _mm_storeu_ps(output.as_mut_ptr(), result);
        output
    }

    fn add_vec3_scalar(&self, a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
    }

    /// 向量点积（SIMD加速）
    pub fn dot_vec3(&self, a: [f32; 3], b: [f32; 3]) -> f32 {
        #[cfg(target_arch = "x86_64")]
        {
            if self.features.sse3 {
                self.dot_vec3_sse3(a, b)
            } else if self.features.sse2 {
                self.dot_vec3_sse2(a, b)
            } else {
                self.dot_vec3_scalar(a, b)
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.dot_vec3_scalar(a, b)
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse3")]
    unsafe fn dot_vec3_sse3(&self, a: [f32; 3], b: [f32; 3]) -> f32 {
        let a_vec = _mm_set_ps(0.0, a[2], a[1], a[0]);
        let b_vec = _mm_set_ps(0.0, b[2], b[1], b[0]);
        let result = _mm_dp_ps(a_vec, b_vec, 0x71);

        _mm_cvtss_f32(result)
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn dot_vec3_sse2(&self, a: [f32; 3], b: [f32; 3]) -> f32 {
        let a_vec = _mm_set_ps(0.0, a[2], a[1], a[0]);
        let b_vec = _mm_set_ps(0.0, b[2], b[1], b[0]);
        let mul = _mm_mul_ps(a_vec, b_vec);

        // 水平求和
        let shuffle = _mm_movehdup_ps(mul);
        let sums = _mm_add_ps(mul, shuffle);
        let shuffle2 = _mm_movehl_ps(shuffle, shuffle);
        let result = _mm_add_ss(sums, shuffle2);

        _mm_cvtss_f32(result)
    }

    fn dot_vec3_scalar(&self, a: [f32; 3], b: [f32; 3]) -> f32 {
        a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
    }

    /// 向量长度（SIMD加速）
    pub fn length_vec3(&self, v: [f32; 3]) -> f32 {
        self.dot_vec3(v, v).sqrt()
    }

    /// 向量归一化（SIMD加速）
    pub fn normalize_vec3(&self, v: [f32; 3]) -> [f32; 3] {
        let len = self.length_vec3(v);
        if len > 0.0 {
            [v[0] / len, v[1] / len, v[2] / len]
        } else {
            [0.0, 0.0, 0.0]
        }
    }

    /// 批量向量加法（SIMD加速）
    pub fn add_vec3_batch(&self, a: &[[f32; 3]], b: &[[f32; 3]], dest: &mut [[f32; 3]]) {
        assert_eq!(a.len(), b.len());
        assert_eq!(a.len(), dest.len());

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.avx2 {
                unsafe { self.add_vec3_batch_avx2(a, b, dest) }
            } else {
                self.add_vec3_batch_scalar(a, b, dest);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.add_vec3_batch_scalar(a, b, dest);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn add_vec3_batch_avx2(
        &self,
        a: &[[f32; 3]],
        b: &[[f32; 3]],
        dest: &mut [[f32; 3]],
    ) {
        for i in 0..a.len() {
            let a_vec = _mm256_set_ps(0.0, 0.0, a[i][2], a[i][1], a[i][0], 0.0, 0.0, 0.0);
            let b_vec = _mm256_set_ps(0.0, 0.0, b[i][2], b[i][1], b[i][0], 0.0, 0.0, 0.0);
            let result = _mm256_add_ps(a_vec, b_vec);

            let mut temp = [0.0f32; 8];
            _mm256_storeu_ps(temp.as_mut_ptr(), result);
            dest[i] = [temp[0], temp[1], temp[2]];
        }
    }

    fn add_vec3_batch_scalar(&self, a: &[[f32; 3]], b: &[[f32; 3]], dest: &mut [[f32; 3]]) {
        for i in 0..a.len() {
            dest[i] = [a[i][0] + b[i][0], a[i][1] + b[i][1], a[i][2] + b[i][2]];
        }
    }
}

impl Default for SimdVecOps {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// SIMD矩阵运算
// ============================================================================

/// SIMD矩阵运算
pub struct SimdMatrixOps {
    features: CpuFeatures,
}

impl SimdMatrixOps {
    /// 创建新的SIMD矩阵运算
    pub fn new() -> Self {
        Self {
            features: CpuFeatures::detect(),
        }
    }

    /// 4x4矩阵乘法（SIMD加速）
    pub fn mul_mat4(&self, a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        #[cfg(target_arch = "x86_64")]
        {
            if self.features.avx {
                self.mul_mat4_avx(a, b)
            } else if self.features.sse2 {
                self.mul_mat4_sse2(a, b)
            } else {
                self.mul_mat4_scalar(a, b)
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.mul_mat4_scalar(a, b)
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx")]
    unsafe fn mul_mat4_avx(&self, a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        // 转置矩阵B以提高缓存效率
        let mut bt = [[0.0; 4]; 4];
        for i in 0..4 {
            for j in 0..4 {
                bt[j][i] = b[i][j];
            }
        }

        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            let row_a = _mm256_loadu_ps(a[i].as_ptr());

            for j in 0..4 {
                let row_b = _mm256_loadu_ps(bt[j].as_ptr());
                let mul = _mm256_mul_ps(row_a, row_b);

                // 水平求和
                let shuffle1 = _mm256_permute2f128_ps(mul, mul, 0x01);
                let sum1 = _mm256_add_ps(mul, shuffle1);
                let shuffle2 = _mm256_shuffle_ps(sum1, sum1, 0xB1);
                let sum2 = _mm256_add_ps(sum1, shuffle2);
                let final_sum = _mm256_add_ps(sum2, _mm256_movehdup_ps(sum2));

                result[i][j] = _mm_cvtss_f32(_mm256_castps256_ps128(final_sum));
            }
        }

        result
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn mul_mat4_sse2(&self, a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                let mut sum = _mm_setzero_ps();

                for k in 0..4 {
                    let a_elem = _mm_set1_ps(a[i][k]);
                    let b_row = _mm_loadu_ps(b[k].as_ptr());
                    let mul = _mm_mul_ps(a_elem, b_row);
                    sum = _mm_add_ps(sum, mul);
                }

                // 水平求和
                let shuffle1 = _mm_movehdup_ps(sum);
                let sum1 = _mm_add_ps(sum, shuffle1);
                let shuffle2 = _mm_movehl_ps(shuffle1, shuffle1);
                let sum2 = _mm_add_ps(sum1, shuffle2);

                result[i][j] = _mm_cvtss_f32(sum2);
            }
        }

        result
    }

    fn mul_mat4_scalar(&self, a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
        let mut result = [[0.0; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result[i][j] += a[i][k] * b[k][j];
                }
            }
        }

        result
    }

    /// 批量矩阵-向量乘法（SIMD加速）
    pub fn transform_vec3_batch(
        &self,
        matrices: &[[[f32; 4]; 4]],
        vectors: &[[f32; 3]],
        dest: &mut [[f32; 3]],
    ) {
        assert_eq!(matrices.len(), vectors.len());
        assert_eq!(vectors.len(), dest.len());

        #[cfg(target_arch = "x86_64")]
        {
            if self.features.avx2 {
                unsafe { self.transform_vec3_batch_avx2(matrices, vectors, dest) }
            } else if self.features.sse2 {
                unsafe { self.transform_vec3_batch_sse2(matrices, vectors, dest) }
            } else {
                self.transform_vec3_batch_scalar(matrices, vectors, dest);
            }
        }

        #[cfg(not(target_arch = "x86_64"))]
        {
            self.transform_vec3_batch_scalar(matrices, vectors, dest);
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn transform_vec3_batch_avx2(
        &self,
        matrices: &[[[f32; 4]; 4]],
        vectors: &[[f32; 3]],
        dest: &mut [[f32; 3]],
    ) {
        for i in 0..matrices.len() {
            let m = &matrices[i];
            let v = vectors[i];

            let x = m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2] + m[0][3];
            let y = m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2] + m[1][3];
            let z = m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2] + m[2][3];

            dest[i] = [x, y, z];
        }
    }

    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "sse2")]
    unsafe fn transform_vec3_batch_sse2(
        &self,
        matrices: &[[[f32; 4]; 4]],
        vectors: &[[f32; 3]],
        dest: &mut [[f32; 3]],
    ) {
        for i in 0..matrices.len() {
            let m = &matrices[i];
            let v = vectors[i];

            let vec3 = _mm_set_ps(0.0, v[2], v[1], v[0]);
            let mut result = _mm_setzero_ps();

            for j in 0..3 {
                let row = _mm_loadu_ps(m[j].as_ptr());
                let broadcast = _mm_shuffle_ps(vec3, vec3, j as i32);
                let mul = _mm_mul_ps(row, broadcast);
                result = _mm_add_ps(result, mul);
            }

            let mut temp = [0.0; 4];
            _mm_storeu_ps(temp.as_mut_ptr(), result);
            dest[i] = [temp[0], temp[1], temp[2]];
        }
    }

    fn transform_vec3_batch_scalar(
        &self,
        matrices: &[[[f32; 4]; 4]],
        vectors: &[[f32; 3]],
        dest: &mut [[f32; 3]],
    ) {
        for i in 0..matrices.len() {
            let m = &matrices[i];
            let v = vectors[i];

            dest[i][0] = m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2] + m[0][3];
            dest[i][1] = m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2] + m[1][3];
            dest[i][2] = m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2] + m[2][3];
        }
    }
}

impl Default for SimdMatrixOps {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 运行时分发器
// ============================================================================

/// SIMD加速分发器
pub struct SimdDispatcher {
    features: CpuFeatures,
    vec_ops: SimdVecOps,
    matrix_ops: SimdMatrixOps,
}

impl SimdDispatcher {
    /// 创建新的分发器
    pub fn new() -> Self {
        let features = CpuFeatures::detect();
        Self {
            features,
            vec_ops: SimdVecOps::new(),
            matrix_ops: SimdMatrixOps::new(),
        }
    }

    /// 获取CPU特性
    pub fn features(&self) -> &CpuFeatures {
        &self.features
    }

    /// 获取最佳SIMD指令集
    pub fn best_instruction_set(&self) -> SimdInstructionSet {
        self.features.best_simd()
    }

    /// 获取向量运算
    pub fn vec_ops(&self) -> &SimdVecOps {
        &self.vec_ops
    }

    /// 获取矩阵运算
    pub fn matrix_ops(&self) -> &SimdMatrixOps {
        &self.matrix_ops
    }

    /// 打印CPU信息
    pub fn print_info(&self) {
        println!("CPU Features:");
        println!("  SSE: {}", self.features.sse);
        println!("  SSE2: {}", self.features.sse2);
        println!("  SSE3: {}", self.features.sse3);
        println!("  SSE4.1: {}", self.features.sse4_1);
        println!("  SSE4.2: {}", self.features.sse4_2);
        println!("  AVX: {}", self.features.avx);
        println!("  AVX2: {}", self.features.avx2);
        println!("  AVX-512F: {}", self.features.avx512f);
        println!("  NEON: {}", self.features.neon);
        println!("\nBest SIMD: {:?}", self.best_instruction_set());
    }
}

impl Default for SimdDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_features_detect() {
        let features = CpuFeatures::detect();
        // 至少应该有一些特性
        assert!(features.sse || features.neon);
    }

    #[test]
    fn test_vec3_add() {
        let ops = SimdVecOps::new();
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let result = ops.add_vec3(a, b);
        assert_eq!(result, [5.0, 7.0, 9.0]);
    }

    #[test]
    fn test_vec3_dot() {
        let ops = SimdVecOps::new();
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];
        let result = ops.dot_vec3(a, b);
        assert_eq!(result, 32.0);
    }

    #[test]
    fn test_vec3_length() {
        let ops = SimdVecOps::new();
        let v = [3.0, 4.0, 0.0];
        let result = ops.length_vec3(v);
        assert_eq!(result, 5.0);
    }

    #[test]
    fn test_vec3_normalize() {
        let ops = SimdVecOps::new();
        let v = [3.0, 4.0, 0.0];
        let result = ops.normalize_vec3(v);
        let len = ops.length_vec3(result);
        assert!((len - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_mat4_mul() {
        let ops = SimdMatrixOps::new();

        let identity = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];

        let result = ops.mul_mat4(&identity, &identity);
        assert_eq!(result, identity);
    }

    #[test]
    fn test_batch_operations() {
        let vec_ops = SimdVecOps::new();

        let a = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]];
        let b = [[7.0, 8.0, 9.0], [10.0, 11.0, 12.0]];
        let mut dest = [[0.0; 3]; 2];

        vec_ops.add_vec3_batch(&a, &b, &mut dest);

        assert_eq!(dest[0], [8.0, 10.0, 12.0]);
        assert_eq!(dest[1], [14.0, 16.0, 18.0]);
    }

    #[test]
    fn test_simd_dispatcher() {
        let dispatcher = SimdDispatcher::new();
        let features = dispatcher.features();

        // 验证检测到的特性
        assert!(features.sse || features.neon || !features.sse);

        let best = dispatcher.best_instruction_set();
        assert!(best != SimdInstructionSet::Scalar || (!features.sse && !features.neon));
    }
}
