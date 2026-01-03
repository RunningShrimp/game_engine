// Assertion Helpers
// 提供自定义断言函数

use std::time::Duration;

/// 断言两个浮点数近似相等
pub fn assert_approx_eq(a: f32, b: f32, epsilon: f32) {
    let diff = (a - b).abs();
    assert!(
        diff <= epsilon,
        "Values not approximately equal: {} vs {} (diff: {}, max allowed: {})",
        a, b, diff, epsilon
    );
}

/// 断言向量近似相等
pub fn assert_vec3_approx_eq(a: [f32; 3], b: [f32; 3], epsilon: f32) {
    for i in 0..3 {
        assert_approx_eq(a[i], b[i], epsilon);
    }
}

/// 断言操作在指定时间内完成
pub fn assert_completes_within<F>(duration: Duration, action: F)
where
    F: FnOnce() + std::panic::UnwindSafe,
{
    let start = std::time::Instant::now();
    let result = std::panic::catch_unwind(action);
    let elapsed = start.elapsed();

    assert!(
        elapsed <= duration,
        "Operation took too long: {:?} (max allowed: {:?})",
        elapsed, duration
    );

    if let Err(err) = result {
        std::panic::resume_unwind(err);
    }
}

/// 断言集合包含指定元素
pub fn assert_contains<T: PartialEq + std::fmt::Debug>(collection: &[T], item: &T) {
    assert!(
        collection.contains(item),
        "Collection does not contain expected item: {:?}\nActual: {:?}",
        item, collection
    );
}

/// 断言集合不包含指定元素
pub fn assert_not_contains<T: PartialEq + std::fmt::Debug>(collection: &[T], item: &T) {
    assert!(
        !collection.contains(item),
        "Collection should not contain item: {:?}",
        item
    );
}

/// 断言字符串包含子串
pub fn assert_string_contains(haystack: &str, needle: &str) {
    assert!(
        haystack.contains(needle),
        "String does not contain expected substring.\nExpected substring: {:?}\nActual string: {:?}",
        needle, haystack
    );
}

/// 断言结果为Ok
pub fn assert_ok<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(err) => panic!("Expected Ok, got Err: {:?}", err),
    }
}

/// 断言结果为Err
pub fn assert_err<T, E: std::fmt::Debug>(result: Result<T, E>) -> E {
    match result {
        Ok(value) => panic!("Expected Err, got Ok: {:?}", value),
        Err(err) => err,
    }
}

/// 断言选项为Some
pub fn assert_some<T: std::fmt::Debug>(option: Option<T>) -> T {
    match option {
        Some(value) => value,
        None => panic!("Expected Some, got None"),
    }
}

/// 断言选项为None
pub fn assert_none<T: std::fmt::Debug>(option: Option<T>) {
    match option {
        Some(value) => panic!("Expected None, got Some: {:?}", value),
        None => (),
    }
}

/// 断言计数器在范围内
pub fn assert_in_range<T: PartialOrd + std::fmt::Debug>(value: T, min: T, max: T) {
    assert!(
        value >= min && value <= max,
        "Value {:?} is not in range [{:?}, {:?}]",
        value, min, max
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_approx_eq() {
        assert_approx_eq(1.0, 1.001, 0.01);
    }

    #[test]
    fn test_assert_vec3_approx_eq() {
        assert_vec3_approx_eq([1.0, 2.0, 3.0], [1.001, 2.001, 3.001], 0.01);
    }

    #[test]
    fn test_assert_contains() {
        let vec = vec![1, 2, 3];
        assert_contains(&vec, &2);
    }

    #[test]
    fn test_assert_string_contains() {
        assert_string_contains("hello world", "world");
    }

    #[test]
    fn test_assert_ok() {
        let result: Result<i32, &str> = Ok(42);
        assert_eq!(assert_ok(result), 42);
    }

    #[test]
    fn test_assert_some() {
        let option: Option<i32> = Some(42);
        assert_eq!(assert_some(option), 42);
    }
}
