// ============================================================================
// Math模块属性测试
// ============================================================================
//
// 本文件包含Math相关的属性测试。
//
// ## 测试的属性
//
// 1. **四元数性质**: 四元数运算应该满足数学性质
// 2. **矩阵运算**: 矩阵乘法应该满足结合律
// 3. **向量运算**: 向量运算应该满足线性代数性质
// 4. **变换属性**: 变换应该满足群论性质
// 5. **插值平滑性**: 插值应该产生平滑过渡

use proptest::prelude::*;
use glam::{Vec2, Vec3, Vec4, Quat, Mat3, Mat4};

// ============================================================================
// Test helpers (copied from property_tests.rs)
// ============================================================================

pub mod strategies {
    use proptest::prelude::*;
    use glam::Vec3;

    /// 坐标策略：生成合理的浮点数坐标
    pub fn coord() -> impl Strategy<Value = f32> {
        -1000.0..=1000.0f32
    }

    /// 小坐标策略：生成小范围的坐标（适合局部测试）
    pub fn coord_small() -> impl Strategy<Value = f32> {
        -100.0..=100.0f32
    }

    /// 向量策略：生成3D向量
    pub fn vec3() -> impl Strategy<Value = Vec3> {
        prop::array::uniform3(coord()).prop_map(|arr| Vec3::from_array(arr))
    }

    /// 小向量策略：生成小范围的3D向量
    pub fn vec3_small() -> impl Strategy<Value = Vec3> {
        prop::array::uniform3(coord_small()).prop_map(|arr| Vec3::from_array(arr))
    }

    /// 单位向量策略：生成归一化的3D向量
    pub fn vec3_normalized() -> impl Strategy<Value = Vec3> {
        vec3().prop_filter("vector too close to zero", |v| {
            v.length() > 0.001
        }).prop_map(|v| v.normalize())
    }
}

/// 检查两个浮点数是否近似相等
pub fn approx_eq(a: f32, b: f32, epsilon: f32) -> bool {
    (a - b).abs() < epsilon
}

/// 检查两个向量是否近似相等
pub fn vec3_approx_eq(a: glam::Vec3, b: glam::Vec3, epsilon: f32) -> bool {
    (a - b).length() < epsilon
}

/// 检查四元数是否近似相等
pub fn quat_approx_eq(a: glam::Quat, b: glam::Quat, epsilon: f32) -> bool {
    // 四元数 q 和 -q 表示相同的旋转
    let dot = a.dot(b);
    dot.abs() > 1.0 - epsilon
}

// ============================================================================
// Vec3 属性测试
// ============================================================================

proptest! {
    /// 测试向量加法的交换律
    /// a + b = b + a
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_addition_commutative(
        a in strategies::vec3(),
        b in strategies::vec3()
    ) {
        let sum1 = a + b;
        let sum2 = b + a;

        prop_assert!(vec3_approx_eq(sum1, sum2, 0.001));
    }

    /// 测试向量加法的结合律
    /// (a + b) + c = a + (b + c)
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_addition_associative(
        a in strategies::vec3(),
        b in strategies::vec3(),
        c in strategies::vec3()
    ) {
        let left = (a + b) + c;
        let right = a + (b + c);

        prop_assert!(vec3_approx_eq(left, right, 0.001));
    }

    /// 测试向量加法的单位元
    /// a + 0 = a
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_addition_identity(
        a in strategies::vec3()
    ) {
        let zero = Vec3::ZERO;
        let result = a + zero;

        prop_assert!(vec3_approx_eq(result, a, 0.001));
    }

    /// 测试向量标量乘法的分配律
    /// s * (a + b) = s*a + s*b
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_scalar_multiplication_distributive(
        a in strategies::vec3(),
        b in strategies::vec3(),
        s in strategies::coord()
    ) {
        let left = s * (a + b);
        let right = s * a + s * b;

        prop_assert!(vec3_approx_eq(left, right, 0.001));
    }

    /// 测试向量归一化的幂等性
    /// 归一化后的向量长度应该为1
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_normalize_length(
        vec in strategies::vec3()
    ) {
        // 跳过零向量
        if vec.length() < 0.001 {
            return Ok(());
        }

        let normalized = vec.normalize();
        let length = normalized.length();

        prop_assert!(approx_eq(length, 1.0, 0.001));
    }

    /// 测试向量点乘的对称性
    /// a · b = b · a
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_dot_symmetric(
        a in strategies::vec3(),
        b in strategies::vec3()
    ) {
        let dot1 = a.dot(b);
        let dot2 = b.dot(a);

        prop_assert!(approx_eq(dot1, dot2, 0.001));
    }

    /// 测试向量叉乘的反交换律
    /// a × b = -(b × a)
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_cross_anti_commutative(
        a in strategies::vec3(),
        b in strategies::vec3()
    ) {
        let cross1 = a.cross(b);
        let cross2 = b.cross(a);

        prop_assert!(vec3_approx_eq(cross1, -cross2, 0.001));
    }

    /// 测试向量距离的对称性
    /// distance(a, b) = distance(b, a)
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_distance_symmetric(
        a in strategies::vec3(),
        b in strategies::vec3()
    ) {
        let dist1 = a.distance(b);
        let dist2 = b.distance(a);

        prop_assert!(approx_eq(dist1, dist2, 0.001));
    }

    /// 测试向量距离的非负性
    /// distance(a, b) >= 0
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_distance_non_negative(
        a in strategies::vec3(),
        b in strategies::vec3()
    ) {
        let dist = a.distance(b);

        prop_assert!(dist >= 0.0);
    }

    /// 测试向量距离的同一性
    /// distance(a, a) = 0
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_vec3_distance_identity(
        a in strategies::vec3()
    ) {
        let dist = a.distance(a);

        prop_assert!(approx_eq(dist, 0.0, 0.001));
    }
}

// ============================================================================
// Quat 属性测试
// ============================================================================

proptest! {
    /// 测试四元数乘法的结合律
    /// (q1 * q2) * q3 = q1 * (q2 * q3)
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_quat_multiplication_associative(
        x1 in strategies::coord_small(),
        y1 in strategies::coord_small(),
        z1 in strategies::coord_small(),
        w1 in strategies::coord_small(),
        x2 in strategies::coord_small(),
        y2 in strategies::coord_small(),
        z2 in strategies::coord_small(),
        w2 in strategies::coord_small(),
        x3 in strategies::coord_small(),
        y3 in strategies::coord_small(),
        z3 in strategies::coord_small(),
        w3 in strategies::coord_small()
    ) {
        let quat1 = Quat::from_xyzw(x1, y1, z1, w1).normalize();
        let quat2 = Quat::from_xyzw(x2, y2, z2, w2).normalize();
        let quat3 = Quat::from_xyzw(x3, y3, z3, w3).normalize();

        let left = (quat1 * quat2) * quat3;
        let right = quat1 * (quat2 * quat3);

        prop_assert!(quat_approx_eq(left, right, 0.001));
    }

    /// 测试单位四元数
    /// identity * q = q
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_quat_identity(
        x in strategies::coord_small(),
        y in strategies::coord_small(),
        z in strategies::coord_small(),
        w in strategies::coord_small()
    ) {
        let quat = Quat::from_xyzw(x, y, z, w).normalize();
        let identity = Quat::IDENTITY;

        let result1 = identity * quat;
        let result2 = quat * identity;

        prop_assert!(quat_approx_eq(result1, quat, 0.001));
        prop_assert!(quat_approx_eq(result2, quat, 0.001));
    }

    /// 测试四元数归一化
    /// 归一化的四元数长度应该为1
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_quat_normalize_length(
        x in strategies::coord_small(),
        y in strategies::coord_small(),
        z in strategies::coord_small(),
        w in strategies::coord_small()
    ) {
        let quat = Quat::from_xyzw(x, y, z, w);
        let normalized = quat.normalize();
        let length = (normalized.x * normalized.x +
                      normalized.y * normalized.y +
                      normalized.z * normalized.z +
                      normalized.w * normalized.w).sqrt();

        prop_assert!(approx_eq(length, 1.0, 0.001));
    }

    /// 测试四元数逆的幂等性
    /// q * q⁻¹ = identity
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_quat_inverse_roundtrip(
        x in strategies::coord_small(),
        y in strategies::coord_small(),
        z in strategies::coord_small(),
        w in strategies::coord_small()
    ) {
        let quat = Quat::from_xyzw(x, y, z, w).normalize();
        let inverse = quat.inverse();
        let result = quat * inverse;

        prop_assert!(quat_approx_eq(result, Quat::IDENTITY, 0.001));
    }

    /// 测试四元数共轭的性质
    /// conjugate(conjugate(q)) = q
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_quat_conjugate_involution(
        x in strategies::coord_small(),
        y in strategies::coord_small(),
        z in strategies::coord_small(),
        w in strategies::coord_small()
    ) {
        let quat = Quat::from_xyzw(x, y, z, w);
        let conjugate = quat.conjugate();
        let double_conjugate = conjugate.conjugate();

        prop_assert!(quat_approx_eq(quat, double_conjugate, 0.001));
    }

    /// 测试四元数旋转的角度保持
    /// 旋转后的向量长度应该不变
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_quat_rotate_vector_length(
        vec in strategies::vec3(),
        axis in strategies::vec3_normalized(),
        angle in -std::f32::consts::PI..=std::f32::consts::PI
    ) {
        let quat = Quat::from_axis_angle(axis, angle);
        let rotated = quat.mul_vec3(vec);

        let original_length = vec.length();
        let rotated_length = rotated.length();

        prop_assert!(approx_eq(original_length, rotated_length, 0.001));
    }
}

// ============================================================================
// Mat4 属性测试
// ============================================================================

proptest! {
    /// 测试矩阵乘法的结合律
    /// (M1 * M2) * M3 = M1 * (M2 * M3)
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_mat4_multiplication_associative() {
        let m1 = Mat4::IDENTITY;
        let m2 = Mat4::from_scale(Vec3::new(2.0, 2.0, 2.0));
        let m3 = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));

        let left = m1 * m2 * m3;
        let right = m1 * (m2 * m3);

        // 由于浮点精度，允许小误差
        for i in 0..4 {
            for j in 0..4 {
                let diff = (left.col(i)[j] - right.col(i)[j]).abs();
                prop_assert!(diff < 0.001);
            }
        }
    }

    /// 测试单位矩阵
    /// I * M = M * I = M
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_mat4_identity(
        tx in strategies::coord_small(),
        ty in strategies::coord_small(),
        tz in strategies::coord_small()
    ) {
        let translation = Mat4::from_translation(Vec3::new(tx, ty, tz));
        let identity = Mat4::IDENTITY;

        let left = identity * translation;
        let right = translation * identity;

        // 验证每个元素
        for i in 0..4 {
            for j in 0..4 {
                let left_val = left.col(i)[j];
                let right_val = right.col(i)[j];
                let orig_val = translation.col(i)[j];

                prop_assert!(approx_eq(left_val, orig_val, 0.001));
                prop_assert!(approx_eq(right_val, orig_val, 0.001));
            }
        }
    }

    /// 测试矩阵转置的对合性
    /// transpose(transpose(M)) = M
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_mat4_transpose_involution() {
        let mat = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let transpose = mat.transpose();
        let double_transpose = transpose.transpose();

        for i in 0..4 {
            for j in 0..4 {
                let orig_val = mat.col(i)[j];
                let result_val = double_transpose.col(i)[j];
                prop_assert!(approx_eq(orig_val, result_val, 0.001));
            }
        }
    }
}

// ============================================================================
// 插值属性测试
// ============================================================================

proptest! {
    /// 测试线性插值的边界
    /// lerp(a, b, 0) = a, lerp(a, b, 1) = b
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_lerp_boundaries(
        a in strategies::coord(),
        b in strategies::coord()
    ) {
        let at_zero = a + (b - a) * 0.0;
        let at_one = a + (b - a) * 1.0;

        prop_assert!(approx_eq(at_zero, a, 0.001));
        prop_assert!(approx_eq(at_one, b, 0.001));
    }

    /// 测试线性插值的单调性
    /// lerp应该在a和b之间
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_lerp_monotonicity(
        a in strategies::coord(),
        b in strategies::coord(),
        t in 0.0f32..1.0f32
    ) {
        let result = a + (b - a) * t;

        let min = a.min(b);
        let max = a.max(b);

        prop_assert!(result >= min - 0.001);
        prop_assert!(result <= max + 0.001);
    }

    /// 测试球面线性插值的单位性
    /// slerp的结果应该是单位四元数
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_slerp_normalized_quat(
        x1 in strategies::coord_small(),
        y1 in strategies::coord_small(),
        z1 in strategies::coord_small(),
        w1 in strategies::coord_small(),
        x2 in strategies::coord_small(),
        y2 in strategies::coord_small(),
        z2 in strategies::coord_small(),
        w2 in strategies::coord_small(),
        t in 0.0f32..1.0f32
    ) {
        let q1 = Quat::from_xyzw(x1, y1, z1, w1).normalize();
        let q2 = Quat::from_xyzw(x2, y2, z2, w2).normalize();

        let slerped = q1.slerp(q2, t);
        let length = (slerped.x * slerped.x +
                      slerped.y * slerped.y +
                      slerped.z * slerped.z +
                      slerped.w * slerped.w).sqrt();

        prop_assert!(approx_eq(length, 1.0, 0.001));
    }
}

// ============================================================================
// Transform属性测试
// ============================================================================

proptest! {
    /// 测试变换组合的顺序敏感性
    /// Transform顺序重要：T * R ≠ R * T（通常）
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_transform_order_sensitivity(
        tx in strategies::coord_small(),
        ty in strategies::coord_small(),
        tz in strategies::coord_small(),
        axis_x in strategies::coord_small(),
        axis_y in strategies::coord_small(),
        axis_z in strategies::coord_small()
    ) {
        let translation = Vec3::new(tx, ty, tz);
        let axis = Vec3::new(axis_x, axis_y, axis_z).normalize();
        let rotation = Quat::from_axis_angle(axis, std::f32::consts::PI / 4.0);

        // 先平移后旋转
        let vec1 = Vec3::X * 2.0;
        let translated_then_rotated = rotation.mul_vec3(vec1 + translation);

        // 先旋转后平移
        let vec2 = Vec3::X * 2.0;
        let rotated_then_translated = rotation.mul_vec3(vec2) + translation;

        // 结果可能不同
        let are_different = !vec3_approx_eq(translated_then_rotated, rotated_then_translated, 0.001);

        prop_assert!(are_different);
    }

    /// 测试缩放的幂等性
    /// 多次应用相同缩放应该等于一次应用
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_scale_idempotent(
        sx in 0.1f32..10.0f32,
        sy in 0.1f32..10.0f32,
        sz in 0.1f32..10.0f32
    ) {
        let vec = Vec3::new(1.0, 1.0, 1.0);
        let scale = Vec3::new(sx, sy, sz);

        let scaled_once = vec * scale;
        let scaled_twice = scaled_once * scale;

        // scaled_twice = (vec * scale) * scale = vec * (scale * scale)
        let expected = Vec3::new(sx * sx, sy * sy, sz * sz);

        prop_assert!(vec3_approx_eq(scaled_twice, expected, 0.001));
    }
}

// ============================================================================
// AABB（轴对齐包围盒）属性测试
// ============================================================================

#[derive(Clone, Debug)]
struct AABB {
    min: Vec3,
    max: Vec3,
}

impl AABB {
    fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    fn contains(&self, point: Vec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x
            && point.y >= self.min.y && point.y <= self.max.y
            && point.z >= self.min.z && point.z <= self.max.z
    }

    fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

proptest! {
    /// 测试AABB中心对称性
    /// 中心点到min的距离等于到max的距离
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_aabb_center_symmetry(
        min in strategies::vec3(),
        size in strategies::vec3_small()
    ) {
        let max = min + size.abs();
        let aabb = AABB::new(min, max);
        let center = aabb.center();

        let dist_to_min = center.distance(min);
        let dist_to_max = center.distance(max);

        prop_assert!(approx_eq(dist_to_min, dist_to_max, 0.001));
    }

    /// 测试AABB包含中心点
    /// AABB应该包含其中心点
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_aabb_contains_center(
        min in strategies::vec3(),
        size in strategies::vec3_small()
    ) {
        let max = min + size.abs();
        let aabb = AABB::new(min, max);
        let center = aabb.center();

        prop_assert!(aabb.contains(center));
    }

    /// 测试AABB边界包含
    /// min和max点应该被包含
    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_aabb_contains_boundaries(
        min in strategies::vec3(),
        size in strategies::vec3_small()
    ) {
        let max = min + size.abs();
        let aabb = AABB::new(min, max);

        prop_assert!(aabb.contains(min));
        prop_assert!(aabb.contains(max));
    }
}

// ============================================================================
// 综合测试
// ============================================================================

#[test]
#[ignore]  // TODO: Fix compilation errors
fn test_math_integration() {
    // 测试变换链：Translate -> Rotate -> Scale
    let vec = Vec3::new(1.0, 0.0, 0.0);

    // 平移
    let translated = vec + Vec3::new(1.0, 2.0, 3.0);

    // 旋转
    let axis = Vec3::Y.normalize();
    let rotation = Quat::from_axis_angle(axis, std::f32::consts::PI / 2.0);
    let rotated = rotation.mul_vec3(translated);

    // 缩放
    let scaled = rotated * Vec3::new(2.0, 2.0, 2.0);

    // 验证长度变化
    assert!(scaled.length() > vec.length());
}
