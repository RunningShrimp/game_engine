//  时间工具模块
//
//  提供时间戳相关功能，避免依赖core模块，消除循环依赖。

use std::time::SystemTime;

/// 获取当前Unix时间戳（毫秒）
///
/// 用于资源缓存时间戳、分配时间记录等。
///
/// # 返回
///
/// 自UNIX纪元以来的毫秒数
pub fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_timestamp_ms() {
        let ts = current_timestamp_ms();
        assert!(ts > 0);
        // 测试时间戳是合理的（应该是最近的时间）
        assert!(ts > 1_600_000_000_000); // 2020年之后
    }
}
