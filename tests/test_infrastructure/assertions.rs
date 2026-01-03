//  测试断言辅助
//
//  提供额外的断言宏和函数，增强测试可读性。

use std::time::{Duration, Instant};

/// 断言两个浮点数近似相等
///
/// # 参数
///
/// * `a` - 第一个值
/// * `b` - 第二个值
/// * `epsilon` - 允许的误差
///
/// # 示例
///
/// ```rust
/// use game_engine::test_infrastructure::assert_approx_eq;
///
/// assert_approx_eq(1.0, 1.001, 0.01);
/// ```
pub fn assert_approx_eq(a: f64, b: f64, epsilon: f64) {
    let diff = (a - b).abs();
    assert!(
        diff <= epsilon,
        "Values are not approximately equal: {} vs {} (diff: {}, epsilon: {})",
        a,
        b,
        diff,
        epsilon
    );
}

/// 断言向量近似相等
///
/// # 参数
///
/// * `a` - 第一个向量
/// * `b` - 第二个向量
/// * `epsilon` - 允许的误差
///
/// # 示例
///
/// ```rust
/// use game_engine::test_infrastructure::assert_vec_approx_eq;
///
/// assert_vec_approx_eq(&[1.0, 2.0], &[1.001, 2.001], 0.01);
/// ```
pub fn assert_vec_approx_eq(a: &[f64], b: &[f64], epsilon: f64) {
    assert_eq!(
        a.len(),
        b.len(),
        "Vectors have different lengths: {} vs {}",
        a.len(),
        b.len()
    );

    for (i, (x, y)) in a.iter().zip(b.iter()).enumerate() {
        let diff = (x - y).abs();
        assert!(
            diff <= epsilon,
            "Vectors differ at index {}: {} vs {} (diff: {}, epsilon: {})",
            i,
            x,
            y,
            diff,
            epsilon
        );
    }
}

/// 断言操作在指定时间内完成
///
/// # 参数
///
/// * `max_duration` - 最大允许时长
/// * `operation` - 要执行的操作
///
/// # 示例
///
/// ```rust
/// use game_engine::test_infrastructure::assert_completed_within;
/// use std::time::Duration;
///
/// assert_completed_within(Duration::from_millis(100), || {
///     expensive_operation();
/// });
/// ```
pub fn assert_completed_within<F>(max_duration: Duration, operation: F) -> Duration
where
    F: FnOnce(),
{
    let start = Instant::now();
    operation();
    let duration = start.elapsed();

    assert!(
        duration <= max_duration,
        "Operation exceeded expected duration: {:?} > {:?}",
        duration,
        max_duration
    );

    duration
}

/// 断言数组/向量包含元素
///
/// # 参数
///
/// * `slice` - 要检查的切片
/// * `value` - 要查找的值
///
/// # 示例
///
/// ```rust
/// use game_engine::test_infrastructure::assert_contains;
///
/// assert_contains(&[1, 2, 3], &2);
/// ```
pub fn assert_contains<T: PartialEq + std::fmt::Debug>(slice: &[T], value: &T) {
    assert!(
        slice.contains(value),
        "Slice {:?} does not contain value {:?}",
        slice,
        value
    );
}

/// 断言数组/向量不包含元素
///
/// # 参数
///
/// * `slice` - 要检查的切片
/// * `value` - 要确保不存在的值
///
/// # 示例
///
/// ```rust
/// use game_engine::test_infrastructure::assert_not_contains;
///
/// assert_not_contains(&[1, 2, 3], &4);
/// ```
pub fn assert_not_contains<T: PartialEq + std::fmt::Debug>(slice: &[T], value: &T) {
    assert!(
        !slice.contains(value),
        "Slice {:?} should not contain value {:?}",
        slice,
        value
    );
}

/// 断言操作会panic
///
/// # 参数
///
/// * `operation` - 预期会panic的操作
///
/// # 示例
///
/// ```rust
/// use game_engine::test_infrastructure::assert_panics;
///
/// assert_panics(|| {
///     panic!("Expected panic");
/// });
/// ```
pub fn assert_panics<F>(operation: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    use std::panic;

    panic::catch_unwind(operation)
        .expect_err("Operation should have panicked but didn't");
}

/// 断言操作不会panic
///
/// # 参数
///
/// * `operation` - 预期不会panic的操作
///
/// # 示例
///
/// ```rust
/// use game_engine::test_infrastructure::assert_not_panics;
///
/// assert_not_panics(|| {
///     let result = safe_operation();
///     result
/// });
/// ```
pub fn assert_not_panics<F>(operation: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    use std::panic;

    panic::catch_unwind(operation)
        .expect("Operation should not have panicked");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_approx_eq() {
        assert_approx_eq(1.0, 1.001, 0.01);
    }

    #[test]
    fn test_assert_vec_approx_eq() {
        assert_vec_approx_eq(&[1.0, 2.0], &[1.001, 2.001], 0.01);
    }

    #[test]
    fn test_assert_contains() {
        assert_contains(&[1, 2, 3], &2);
    }

    #[test]
    fn test_assert_not_contains() {
        assert_not_contains(&[1, 2, 3], &4);
    }

    #[test]
    fn test_assert_panics() {
        assert_panics(|| {
            panic!("test panic");
        });
    }

    #[test]
    fn test_assert_not_panics() {
        assert_not_panics(|| {
            let x = 1 + 1;
            x
        });
    }

    #[test]
    fn test_assert_completed_within() {
        assert_completed_within(Duration::from_millis(100), || {
            std::thread::sleep(Duration::from_millis(10));
        });
    }
}
