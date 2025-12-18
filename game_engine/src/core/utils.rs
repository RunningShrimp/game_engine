//! 核心工具函数
//!
//! 提供项目中常用的工具函数，避免代码重复

/// 获取当前Unix时间戳（秒）
///
/// 返回自1970年1月1日以来的秒数。
///
/// # 示例
///
/// ```rust
/// use game_engine::core::utils::current_timestamp;
///
/// let timestamp = current_timestamp();
/// println!("Current timestamp: {}", timestamp);
///
/// // 用于记录事件时间
/// let event_time = current_timestamp();
/// ```
pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 获取当前Unix时间戳（毫秒）
///
/// # 示例
///
/// ```rust
/// use game_engine::core::utils::current_timestamp_ms;
///
/// let timestamp_ms = current_timestamp_ms();
/// println!("Current timestamp (ms): {}", timestamp_ms);
/// ```
pub fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// 获取当前Unix时间戳（纳秒）
///
/// 用于生成唯一ID或高精度时间戳
///
/// # 示例
///
/// ```rust
/// use game_engine::core::utils::current_timestamp_nanos;
///
/// let timestamp_ns = current_timestamp_nanos();
/// let unique_id = format!("id_{}", timestamp_ns);
/// ```
pub fn current_timestamp_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

/// 获取当前Unix时间戳（秒，浮点数）
///
/// 用于需要高精度时间戳的场景
///
/// # 示例
///
/// ```rust
/// use game_engine::core::utils::current_timestamp_f64;
///
/// let timestamp = current_timestamp_f64();
/// ```
pub fn current_timestamp_f64() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp() {
        let ts1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let ts2 = current_timestamp();
        assert!(ts2 >= ts1);
    }

    #[test]
    fn test_current_timestamp_ms() {
        let ts1 = current_timestamp_ms();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let ts2 = current_timestamp_ms();
        assert!(ts2 >= ts1);
        assert!(ts2 - ts1 >= 100);
    }
}
