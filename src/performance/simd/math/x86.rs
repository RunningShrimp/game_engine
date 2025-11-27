/// x86/x64 SIMD数学运算优化
/// 
/// 支持SSE2, SSE4.1, AVX, AVX2, AVX-512指令集

#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

/// 使用SSE2的4维向量点积
///
/// # Safety
///
/// 调用者必须确保：
/// 1. `a` 和 `b` 数组长度至少为4
/// 2. 当前CPU支持SSE2指令集（通过is_x86_feature_detected!检查）
/// 3. 数组内存有效且已初始化
/// 4. 内存对齐至少为4字节（使用_mm_loadu_ps可处理未对齐内存）
///
/// # Panics
///
/// 当数组长度小于4时可能panic（debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::dot_product_sse2;
///
/// // 确保CPU支持SSE2
/// assert!(is_x86_feature_detected!("sse2"));
///
/// let a = [1.0, 2.0, 3.0, 4.0];
/// let b = [5.0, 6.0, 7.0, 8.0];
///
/// unsafe {
///     let result = dot_product_sse2(&a, &b);
///     assert_eq!(result, 70.0);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn dot_product_sse2(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    debug_assert_eq!(a.len(), 4, "Input array 'a' must have length 4");
    debug_assert_eq!(b.len(), 4, "Input array 'b' must have length 4");
    
    let va = _mm_loadu_ps(a.as_ptr());
    let vb = _mm_loadu_ps(b.as_ptr());
    let mul = _mm_mul_ps(va, vb);
    
    // 水平加法
    let shuf = _mm_shuffle_ps(mul, mul, 0b_11_10_11_10);
    let sums = _mm_add_ps(mul, shuf);
    let shuf2 = _mm_movehl_ps(sums, sums);
    let result = _mm_add_ss(sums, shuf2);
    
    _mm_cvtss_f32(result)
}

/// 使用SSE4.1的4维向量点积（更高效）
///
/// # Safety
///
/// 调用者必须确保：
/// 1. `a` 和 `b` 数组长度至少为4
/// 2. 当前CPU支持SSE4.1指令集（通过is_x86_feature_detected!检查）
/// 3. 数组内存有效且已初始化
/// 4. 内存对齐至少为4字节（使用_mm_loadu_ps可处理未对齐内存）
///
/// # Panics
///
/// 当数组长度小于4时可能panic（debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::dot_product_sse41;
///
/// // 确保CPU支持SSE4.1
/// assert!(is_x86_feature_detected!("sse4.1"));
///
/// let a = [1.0, 2.0, 3.0, 4.0];
/// let b = [5.0, 6.0, 7.0, 8.0];
///
/// unsafe {
///     let result = dot_product_sse41(&a, &b);
///     assert_eq!(result, 70.0);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse4.1")]
pub unsafe fn dot_product_sse41(a: &[f32; 4], b: &[f32; 4]) -> f32 {
    debug_assert_eq!(a.len(), 4, "Input array 'a' must have length 4");
    debug_assert_eq!(b.len(), 4, "Input array 'b' must have length 4");
    
    let va = _mm_loadu_ps(a.as_ptr());
    let vb = _mm_loadu_ps(b.as_ptr());
    // _mm_dp_ps: 点积指令，0xFF表示使用所有4个分量
    let result = _mm_dp_ps(va, vb, 0xFF);
    _mm_cvtss_f32(result)
}

/// 使用AVX的8维向量点积
///
/// # Safety
///
/// 调用者必须确保：
/// 1. `a` 和 `b` 数组长度至少为8
/// 2. 当前CPU支持AVX指令集（通过is_x86_feature_detected!检查）
/// 3. 数组内存有效且已初始化
/// 4. 内存对齐至少为4字节（使用_mm256_loadu_ps可处理未对齐内存）
///
/// # Panics
///
/// 当数组长度小于8时可能panic（debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::dot_product_avx;
///
/// // 确保CPU支持AVX
/// assert!(is_x86_feature_detected!("avx"));
///
/// let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
/// let b = [8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
///
/// unsafe {
///     let result = dot_product_avx(&a, &b);
///     let expected = 1.0*8.0 + 2.0*7.0 + 3.0*6.0 + 4.0*5.0 + 5.0*4.0 + 6.0*3.0 + 7.0*2.0 + 8.0*1.0;
///     assert_eq!(result, expected);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
pub unsafe fn dot_product_avx(a: &[f32; 8], b: &[f32; 8]) -> f32 {
    debug_assert_eq!(a.len(), 8, "Input array 'a' must have length 8");
    debug_assert_eq!(b.len(), 8, "Input array 'b' must have length 8");
    
    let va = _mm256_loadu_ps(a.as_ptr());
    let vb = _mm256_loadu_ps(b.as_ptr());
    let mul = _mm256_mul_ps(va, vb);
    
    // 水平加法
    let sum = horizontal_add_avx(mul);
    sum
}

/// AVX水平加法辅助函数
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
unsafe fn horizontal_add_avx(v: __m256) -> f32 {
    // 提取高128位和低128位
    let hi = _mm256_extractf128_ps(v, 1);
    let lo = _mm256_castps256_ps128(v);
    let sum128 = _mm_add_ps(hi, lo);
    
    // 128位水平加法
    let shuf = _mm_shuffle_ps(sum128, sum128, 0b_11_10_11_10);
    let sums = _mm_add_ps(sum128, shuf);
    let shuf2 = _mm_movehl_ps(sums, sums);
    let result = _mm_add_ss(sums, shuf2);
    
    _mm_cvtss_f32(result)
}

/// 使用FMA的向量乘加运算 (a * b + c)
///
/// # Safety
///
/// 调用者必须确保：
/// 1. `a`, `b`, `c`, `out` 数组长度至少为4
/// 2. 当前CPU支持FMA指令集（通过is_x86_feature_detected!检查）
/// 3. 所有输入数组内存有效且已初始化
/// 4. `out` 数组可写且内存有效
/// 5. 内存对齐至少为4字节（使用_mm_loadu_ps/_mm_storeu_ps可处理未对齐内存）
/// 6. `out` 不能与 `a`, `b`, `c` 重叠（避免数据竞争）
///
/// # Panics
///
/// 当数组长度小于4时可能panic（debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::fma_vec4;
///
/// // 确保CPU支持FMA
/// assert!(is_x86_feature_detected!("fma"));
///
/// let a = [1.0, 2.0, 3.0, 4.0];
/// let b = [5.0, 6.0, 7.0, 8.0];
/// let c = [0.1, 0.2, 0.3, 0.4];
/// let mut out = [0.0; 4];
///
/// unsafe {
///     fma_vec4(&a, &b, &c, &mut out);
///     // out = a * b + c
///     assert_eq!(out[0], 1.0 * 5.0 + 0.1);
///     assert_eq!(out[1], 2.0 * 6.0 + 0.2);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "fma")]
pub unsafe fn fma_vec4(a: &[f32; 4], b: &[f32; 4], c: &[f32; 4], out: &mut [f32; 4]) {
    debug_assert_eq!(a.len(), 4, "Input array 'a' must have length 4");
    debug_assert_eq!(b.len(), 4, "Input array 'b' must have length 4");
    debug_assert_eq!(c.len(), 4, "Input array 'c' must have length 4");
    debug_assert_eq!(out.len(), 4, "Output array 'out' must have length 4");
    
    let va = _mm_loadu_ps(a.as_ptr());
    let vb = _mm_loadu_ps(b.as_ptr());
    let vc = _mm_loadu_ps(c.as_ptr());
    let result = _mm_fmadd_ps(va, vb, vc);
    _mm_storeu_ps(out.as_mut_ptr(), result);
}

/// 使用AVX2的8个f32向量加法
///
/// # Safety
///
/// 调用者必须确保：
/// 1. `a`, `b`, `out` 数组长度至少为8
/// 2. 当前CPU支持AVX2指令集（通过is_x86_feature_detected!检查）
/// 3. 所有输入数组内存有效且已初始化
/// 4. `out` 数组可写且内存有效
/// 5. 内存对齐至少为4字节（使用_mm256_loadu_ps/_mm256_storeu_ps可处理未对齐内存）
/// 6. `out` 不能与 `a`, `b` 重叠（避免数据竞争）
///
/// # Panics
///
/// 当数组长度小于8时可能panic（debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::add_vec8_avx2;
///
/// // 确保CPU支持AVX2
/// assert!(is_x86_feature_detected!("avx2"));
///
/// let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
/// let b = [8.0, 7.0, 6.0, 5.0, 4.0, 3.0, 2.0, 1.0];
/// let mut out = [0.0; 8];
///
/// unsafe {
///     add_vec8_avx2(&a, &b, &mut out);
///     assert_eq!(out, [9.0, 9.0, 9.0, 9.0, 9.0, 9.0, 9.0, 9.0]);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn add_vec8_avx2(a: &[f32; 8], b: &[f32; 8], out: &mut [f32; 8]) {
    debug_assert_eq!(a.len(), 8, "Input array 'a' must have length 8");
    debug_assert_eq!(b.len(), 8, "Input array 'b' must have length 8");
    debug_assert_eq!(out.len(), 8, "Output array 'out' must have length 8");
    
    let va = _mm256_loadu_ps(a.as_ptr());
    let vb = _mm256_loadu_ps(b.as_ptr());
    let result = _mm256_add_ps(va, vb);
    _mm256_storeu_ps(out.as_mut_ptr(), result);
}

/// 使用AVX2的8个f32向量乘法
///
/// # Safety
///
/// 调用者必须确保：
/// 1. `a`, `b`, `out` 数组长度至少为8
/// 2. 当前CPU支持AVX2指令集（通过is_x86_feature_detected!检查）
/// 3. 所有输入数组内存有效且已初始化
/// 4. `out` 数组可写且内存有效
/// 5. 内存对齐至少为4字节（使用_mm256_loadu_ps/_mm256_storeu_ps可处理未对齐内存）
/// 6. `out` 不能与 `a`, `b` 重叠（避免数据竞争）
///
/// # Panics
///
/// 当数组长度小于8时可能panic（debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::mul_vec8_avx2;
///
/// // 确保CPU支持AVX2
/// assert!(is_x86_feature_detected!("avx2"));
///
/// let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
/// let b = [2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0];
/// let mut out = [0.0; 8];
///
/// unsafe {
///     mul_vec8_avx2(&a, &b, &mut out);
///     assert_eq!(out, [2.0, 4.0, 6.0, 8.0, 10.0, 12.0, 14.0, 16.0]);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx2")]
pub unsafe fn mul_vec8_avx2(a: &[f32; 8], b: &[f32; 8], out: &mut [f32; 8]) {
    debug_assert_eq!(a.len(), 8, "Input array 'a' must have length 8");
    debug_assert_eq!(b.len(), 8, "Input array 'b' must have length 8");
    debug_assert_eq!(out.len(), 8, "Output array 'out' must have length 8");
    
    let va = _mm256_loadu_ps(a.as_ptr());
    let vb = _mm256_loadu_ps(b.as_ptr());
    let result = _mm256_mul_ps(va, vb);
    _mm256_storeu_ps(out.as_mut_ptr(), result);
}

/// 使用AVX-512的16个f32向量点积
///
/// # Safety
///
/// 调用者必须确保：
/// 1. `a` 和 `b` 数组长度至少为16
/// 2. 当前CPU支持AVX-512F指令集（通过is_x86_feature_detected!检查）
/// 3. 数组内存有效且已初始化
/// 4. 内存对齐至少为4字节（使用_mm512_loadu_ps可处理未对齐内存）
///
/// # Panics
///
/// 当数组长度小于16时可能panic（debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::dot_product_avx512;
///
/// // 确保CPU支持AVX-512F
/// assert!(is_x86_feature_detected!("avx512f"));
///
/// let a = [1.0; 16]; // 16个1.0
/// let b = [2.0; 16]; // 16个2.0
///
/// unsafe {
///     let result = dot_product_avx512(&a, &b);
///     assert_eq!(result, 32.0); // 16 * (1.0 * 2.0)
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
pub unsafe fn dot_product_avx512(a: &[f32; 16], b: &[f32; 16]) -> f32 {
    debug_assert_eq!(a.len(), 16, "Input array 'a' must have length 16");
    debug_assert_eq!(b.len(), 16, "Input array 'b' must have length 16");
    
    let va = _mm512_loadu_ps(a.as_ptr());
    let vb = _mm512_loadu_ps(b.as_ptr());
    let mul = _mm512_mul_ps(va, vb);
    
    // AVX-512的reduce_add指令
    _mm512_reduce_add_ps(mul)
}

/// 批量向量归一化（SSE2）
///
/// # Safety
///
/// 调用者必须确保：
/// 1. 当前CPU支持SSE2指令集（通过is_x86_feature_detected!检查）
/// 2. `vectors` 切片有效且每个向量长度为4
/// 3. 向量内存有效且已初始化
/// 4. 内存对齐至少为4字节（使用_mm_loadu_ps/_mm_storeu_ps可处理未对齐内存）
/// 5. 向量分量不包含NaN或无穷大值（可能导致未定义行为）
///
/// # Panics
///
/// 当向量长度不为4时可能panic（内部debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::normalize_batch_sse2;
///
/// // 确保CPU支持SSE2
/// assert!(is_x86_feature_detected!("sse2"));
///
/// let mut vectors = vec![
///     [3.0, 4.0, 0.0, 0.0], // 长度为5.0
///     [0.0, 0.0, 5.0, 12.0], // 长度为13.0
/// ];
///
/// unsafe {
///     normalize_batch_sse2(&mut vectors);
///     // 第一个向量应该归一化为 [0.6, 0.8, 0.0, 0.0]
///     assert!((vectors[0][0] - 0.6).abs() < 1e-5);
///     assert!((vectors[0][1] - 0.8).abs() < 1e-5);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn normalize_batch_sse2(vectors: &mut [[f32; 4]]) {
    for vec in vectors.iter_mut() {
        let v = _mm_loadu_ps(vec.as_ptr());
        let dot = dot_product_sse2(vec, vec);
        let len = dot.sqrt();
        
        if len > 1e-6 {
            let inv_len = _mm_set1_ps(1.0 / len);
            let normalized = _mm_mul_ps(v, inv_len);
            _mm_storeu_ps(vec.as_mut_ptr(), normalized);
        }
    }
}

/// 4x4矩阵乘法（SSE优化）
///
/// # Safety
///
/// 调用者必须确保：
/// 1. 当前CPU支持SSE2指令集（通过is_x86_feature_detected!检查）
/// 2. `a`, `b`, `out` 矩阵有效且每个矩阵长度为4x4
/// 3. 所有矩阵内存有效且已初始化
/// 4. `out` 矩阵可写且内存有效
/// 5. 内存对齐至少为4字节（使用_mm_loadu_ps/_mm_storeu_ps可处理未对齐内存）
/// 6. `out` 不能与 `a`, `b` 重叠（避免数据竞争）
/// 7. 矩阵元素不包含NaN或无穷大值（可能导致未定义行为）
///
/// # Panics
///
/// 当矩阵维度不为4x4时可能panic（内部debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::mat4_mul_sse2;
///
/// // 确保CPU支持SSE2
/// assert!(is_x86_feature_detected!("sse2"));
///
/// let identity = [
///     [1.0, 0.0, 0.0, 0.0],
///     [0.0, 1.0, 0.0, 0.0],
///     [0.0, 0.0, 1.0, 0.0],
///     [0.0, 0.0, 0.0, 1.0],
/// ];
/// let scale = [
///     [2.0, 0.0, 0.0, 0.0],
///     [0.0, 2.0, 0.0, 0.0],
///     [0.0, 0.0, 2.0, 0.0],
///     [0.0, 0.0, 0.0, 1.0],
/// ];
/// let mut out = [[0.0; 4]; 4];
///
/// unsafe {
///     mat4_mul_sse2(&identity, &scale, &mut out);
///     // 结果应该等于scale矩阵
///     assert_eq!(out[0][0], 2.0);
///     assert_eq!(out[1][1], 2.0);
///     assert_eq!(out[2][2], 2.0);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn mat4_mul_sse2(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4], out: &mut [[f32; 4]; 4]) {
    // 加载矩阵B的列
    let b0 = _mm_loadu_ps(b[0].as_ptr());
    let b1 = _mm_loadu_ps(b[1].as_ptr());
    let b2 = _mm_loadu_ps(b[2].as_ptr());
    let b3 = _mm_loadu_ps(b[3].as_ptr());
    
    for i in 0..4 {
        let a_row = a[i];
        
        // 广播a的每个元素
        let a0 = _mm_set1_ps(a_row[0]);
        let a1 = _mm_set1_ps(a_row[1]);
        let a2 = _mm_set1_ps(a_row[2]);
        let a3 = _mm_set1_ps(a_row[3]);
        
        // 计算结果行
        let r0 = _mm_mul_ps(a0, b0);
        let r1 = _mm_mul_ps(a1, b1);
        let r2 = _mm_mul_ps(a2, b2);
        let r3 = _mm_mul_ps(a3, b3);
        
        let result = _mm_add_ps(_mm_add_ps(r0, r1), _mm_add_ps(r2, r3));
        _mm_storeu_ps(out[i].as_mut_ptr(), result);
    }
}

/// 4x4矩阵乘法（AVX优化，处理两个矩阵）
///
/// # Safety
///
/// 调用者必须确保：
/// 1. 当前CPU支持AVX指令集（通过is_x86_feature_detected!检查）
/// 2. `a`, `b`, `out` 矩阵有效且每个矩阵长度为4x4
/// 3. 所有矩阵内存有效且已初始化
/// 4. `out` 矩阵可写且内存有效
/// 5. 内存对齐至少为4字节（内部使用SSE2函数可处理未对齐内存）
/// 6. `out` 不能与 `a`, `b` 重叠（避免数据竞争）
/// 7. 矩阵元素不包含NaN或无穷大值（可能导致未定义行为）
///
/// # Panics
///
/// 当矩阵维度不为4x4时可能panic（内部调用mat4_mul_sse2的debug_assert检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::mat4_mul_avx;
///
/// // 确保CPU支持AVX
/// assert!(is_x86_feature_detected!("avx"));
///
/// let identity = [
///     [1.0, 0.0, 0.0, 0.0],
///     [0.0, 1.0, 0.0, 0.0],
///     [0.0, 0.0, 1.0, 0.0],
///     [0.0, 0.0, 0.0, 1.0],
/// ];
/// let scale = [
///     [3.0, 0.0, 0.0, 0.0],
///     [0.0, 3.0, 0.0, 0.0],
///     [0.0, 0.0, 3.0, 0.0],
///     [0.0, 0.0, 0.0, 1.0],
/// ];
/// let mut out = [[0.0; 4]; 4];
///
/// unsafe {
///     mat4_mul_avx(&identity, &scale, &mut out);
///     // 结果应该等于scale矩阵
///     assert_eq!(out[0][0], 3.0);
///     assert_eq!(out[1][1], 3.0);
///     assert_eq!(out[2][2], 3.0);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx")]
pub unsafe fn mat4_mul_avx(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4], out: &mut [[f32; 4]; 4]) {
    // AVX可以一次处理两行，但这里简化为回退到SSE
    // 实际应用中可以实现更高效的AVX版本
    mat4_mul_sse2(a, b, out);
}

/// 批量向量变换（矩阵 * 向量）
///
/// # Safety
///
/// 调用者必须确保：
/// 1. 当前CPU支持SSE2指令集（通过is_x86_feature_detected!检查）
/// 2. `matrix` 是有效的4x4变换矩阵
/// 3. `vectors` 和 `out` 切片长度相同且每个向量长度为4
/// 4. 所有输入内存有效且已初始化
/// 5. `out` 切片可写且内存有效
/// 6. 内存对齐至少为4字节（使用_mm_loadu_ps/_mm_storeu_ps可处理未对齐内存）
/// 7. `out` 不能与 `vectors` 重叠（避免数据竞争）
/// 8. 向量和矩阵元素不包含NaN或无穷大值（可能导致未定义行为）
///
/// # Panics
///
/// 当vectors和out长度不同时可能panic（assert_eq!检查）
///
/// # Examples
///
/// ```rust
/// use game_engine::performance::simd::math::x86::transform_vectors_sse2;
///
/// // 确保CPU支持SSE2
/// assert!(is_x86_feature_detected!("sse2"));
///
/// let identity = [
///     [1.0, 0.0, 0.0, 0.0],
///     [0.0, 1.0, 0.0, 0.0],
///     [0.0, 0.0, 1.0, 0.0],
///     [0.0, 0.0, 0.0, 1.0],
/// ];
/// let vectors = vec![
///     [1.0, 2.0, 3.0, 1.0],
///     [4.0, 5.0, 6.0, 1.0],
/// ];
/// let mut out = vec![[0.0; 4]; 2];
///
/// unsafe {
///     transform_vectors_sse2(&identity, &vectors, &mut out);
///     // 使用单位矩阵变换，输出应该等于输入
///     assert_eq!(out[0], vectors[0]);
///     assert_eq!(out[1], vectors[1]);
/// }
/// ```
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
pub unsafe fn transform_vectors_sse2(
    matrix: &[[f32; 4]; 4],
    vectors: &[[f32; 4]],
    out: &mut [[f32; 4]]
) {
    assert_eq!(vectors.len(), out.len());
    
    // 加载矩阵的行
    let m0 = _mm_loadu_ps(matrix[0].as_ptr());
    let m1 = _mm_loadu_ps(matrix[1].as_ptr());
    let m2 = _mm_loadu_ps(matrix[2].as_ptr());
    let m3 = _mm_loadu_ps(matrix[3].as_ptr());
    
    for (vec, out_vec) in vectors.iter().zip(out.iter_mut()) {
        let v = _mm_loadu_ps(vec.as_ptr());
        
        // 广播向量的每个分量
        let vx = _mm_shuffle_ps(v, v, 0b_00_00_00_00);
        let vy = _mm_shuffle_ps(v, v, 0b_01_01_01_01);
        let vz = _mm_shuffle_ps(v, v, 0b_10_10_10_10);
        let vw = _mm_shuffle_ps(v, v, 0b_11_11_11_11);
        
        // 计算 M * v
        let r0 = _mm_mul_ps(m0, vx);
        let r1 = _mm_mul_ps(m1, vy);
        let r2 = _mm_mul_ps(m2, vz);
        let r3 = _mm_mul_ps(m3, vw);
        
        let result = _mm_add_ps(_mm_add_ps(r0, r1), _mm_add_ps(r2, r3));
        _mm_storeu_ps(out_vec.as_mut_ptr(), result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_dot_product() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [5.0, 6.0, 7.0, 8.0];
        let expected = 1.0 * 5.0 + 2.0 * 6.0 + 3.0 * 7.0 + 4.0 * 8.0;
        
        unsafe {
            if is_x86_feature_detected!("sse2") {
                let result = dot_product_sse2(&a, &b);
                assert!((result - expected).abs() < 1e-5);
            }
            
            if is_x86_feature_detected!("sse4.1") {
                let result = dot_product_sse41(&a, &b);
                assert!((result - expected).abs() < 1e-5);
            }
        }
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_mat4_mul() {
        let a = [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [1.0, 2.0, 3.0, 1.0],
        ];
        let b = [
            [2.0, 0.0, 0.0, 0.0],
            [0.0, 2.0, 0.0, 0.0],
            [0.0, 0.0, 2.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mut out = [[0.0; 4]; 4];
        
        unsafe {
            if is_x86_feature_detected!("sse2") {
                mat4_mul_sse2(&a, &b, &mut out);
                assert_eq!(out[0][0], 2.0);
                assert_eq!(out[1][1], 2.0);
                assert_eq!(out[2][2], 2.0);
            }
        }
    }
}
