//! # Proptest Helpers
//!
//! 提供基于属性测试的策略和辅助函数。

use proptest::prelude::*;
use proptest::collection::{hash_map, hash_set, vec};
use glam::{Vec2, Vec3, Vec4, Quat, Mat4};

/// 任意f32策略 (排除NaN和无穷)
pub fn any_f32() -> impl Strategy<Value = f32> {
    prop::num::f32::NORMAL.prop_filter("exclude NaN and Inf", |&x| x.is_finite())
}

/// 任意f64策略 (排除NaN和无穷)
pub fn any_f64() -> impl Strategy<Value = f64> {
    prop::num::f64::NORMAL.prop_filter("exclude NaN and Inf", |&x| x.is_finite())
}

/// 任意Vec2策略
pub fn any_vec2() -> impl Strategy<Value = Vec2> {
    any_f32().prop_map(|x| Vec2::new(x, x))
}

/// 任意Vec3策略
pub fn any_vec3() -> impl Strategy<Value = Vec3> {
    (any_f32(), any_f32(), any_f32()).prop_map(|(x, y, z)| Vec3::new(x, y, z))
}

/// 任意Vec4策略
pub fn any_vec4() -> impl Strategy<Value = Vec4> {
    (any_f32(), any_f32(), any_f32(), any_f32())
        .prop_map(|(x, y, z, w)| Vec4::new(x, y, z, w))
}

/// 任意单位Quat策略
pub fn any_quat() -> impl Strategy<Value = Quat> {
    (any_f32(), any_f32(), any_f32(), any_f32())
        .prop_map(|(x, y, z, w)| Quat::from_xyzw(x, y, z, w).normalize())
}

/// 任意Mat4策略
pub fn any_mat4() -> impl Strategy<Value = Mat4> {
    (any_vec3(), any_quat(), any_vec3()).prop_map(|(translation, rotation, scale)| {
        Mat4::from_scale_rotation_translation(scale, rotation, translation)
    })
}

/// 任意非零usize策略
pub fn any_non_zero_usize() -> impl Strategy<Value = usize> {
    1usize..1000usize
}

/// 任意正f32策略
pub fn any_positive_f32() -> impl Strategy<Value = f32> {
    any_f32().prop_map(|x| x.abs())
}

/// 任意范围[0, 1]的f32策略
pub fn any_normalized_f32() -> impl Strategy<Value = f32> {
    0.0f32..1.0f32
}

/// 任意字符串策略 (ASCII)
pub fn any_string() -> impl Strategy<Value = String> {
    prop::string::string_regex("[a-zA-Z0-9]{1,100}").unwrap()
}

/// 任意RGB颜色策略
pub fn any_rgb_color() -> impl Strategy<Value = (u8, u8, u8)> {
    (any::<u8>(), any::<u8>(), any::<u8>())
}

/// 任意RGBA颜色策略
pub fn any_rgba_color() -> impl Strategy<Value = (u8, u8, u8, u8)> {
    (any::<u8>(), any::<u8>(), any::<u8>(), any::<u8>())
}

/// 任意Vec<T>策略
pub fn any_vec<S: Strategy>(strategy: S, size: impl Into<SizeRange>) -> impl Strategy<Value = Vec<S::Value>>
where
    S::Value: Clone,
{
    vec(strategy, size)
}

/// 任意HashMap<K, V>策略
pub fn any_hash_map<K: Strategy + Clone, V: Strategy>(
    key: K,
    value: V,
    size: impl Into<SizeRange>,
) -> impl Strategy<Value = std::collections::HashMap<K::Value, V::Value>>
where
    K::Value: std::hash::Hash + Eq + Clone,
    V::Value: Clone,
{
    hash_map(key, value, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn test_vec3_roundtrip(x in any_f32(), y in any_f32(), z in any_f32()) {
            let v = Vec3::new(x, y, z);
            prop_assert_eq!(v.x, x);
            prop_assert_eq!(v.y, y);
            prop_assert_eq!(v.z, z);
        }

        #[test]
        fn test_quat_normalize(x in any_f32(), y in any_f32(), z in any_f32(), w in any_f32()) {
            let q = Quat::from_xyzw(x, y, z, w).normalize();
            let length = q.length();
            prop_assert!(length.is_finite());
            prop_assert!(length > 0.0);
        }

        #[test]
        fn test_positive_f32(val in any_positive_f32()) {
            prop_assert!(val >= 0.0);
        }

        #[test]
        fn test_normalized_f32(val in any_normalized_f32()) {
            prop_assert!(val >= 0.0 && val <= 1.0);
        }
    }
}
