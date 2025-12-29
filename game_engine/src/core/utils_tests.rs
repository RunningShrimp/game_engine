//! Core Utils 扩展单元测试
//!
//! 测试核心工具函数的功能

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::utils::*;

    // ========================================
    // Timestamp 基础测试
    // ========================================

    #[test]
    fn test_current_timestamp_returns_value() {
        let ts = current_timestamp();
        assert!(ts > 0);
    }

    #[test]
    fn test_current_timestamp_increments() {
        let ts1 = current_timestamp();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let ts2 = current_timestamp();
        assert!(ts2 > ts1);
        assert!(ts2 - ts1 >= 1);
    }

    #[test]
    fn test_current_timestamp_ms_precision() {
        let ts1 = current_timestamp_ms();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let ts2 = current_timestamp_ms();
        assert!(ts2 >= ts1);
        assert!(ts2 - ts1 >= 50);
        assert!(ts2 - ts1 < 200); // Should be close to 50ms
    }

    #[test]
    fn test_current_timestamp_nanos_precision() {
        let ts1 = current_timestamp_nanos();
        let ts2 = current_timestamp_nanos();
        // 纳秒时间戳应该不同
        assert!(ts2 > ts1);
        // 差异应该很小（纳秒级）
        assert!(ts2 - ts1 < 1_000_000_000); // Less than 1 second
    }

    #[test]
    fn test_current_timestamp_f64_format() {
        let ts = current_timestamp_f64();
        assert!(ts > 0.0);
        assert!(ts.is_finite());
    }

    #[test]
    fn test_timestamp_relationships() {
        let ts_sec = current_timestamp() as u128;
        let ts_ms = current_timestamp_ms() as u128;
        let ts_ns = current_timestamp_nanos();

        // 验证时间戳之间的关系（近似）
        // ms 应该大约是 sec * 1000
        let ms_from_sec = ts_sec * 1000;
        assert!(ts_ms >= ms_from_sec);
        assert!(ts_ms - ms_from_sec < 1000); // Within 1 second

        // ns 应该大约是 ms * 1_000_000
        let ns_from_ms = ts_ms * 1_000_000;
        assert!(ts_ns >= ns_from_ms);
        assert!(ts_ns - ns_from_ms < 1_000_000_000); // Within 1 second
    }

    // ========================================
    // Timestamp 边界情况测试
    // ========================================

    #[test]
    fn test_timestamp_monotonic() {
        let mut prev_ts = current_timestamp();
        for _ in 0..10 {
            let ts = current_timestamp();
            assert!(ts >= prev_ts);
            prev_ts = ts;
        }
    }

    #[test]
    fn test_timestamp_ms_monotonic() {
        let mut prev_ts = current_timestamp_ms();
        for _ in 0..100 {
            let ts = current_timestamp_ms();
            assert!(ts >= prev_ts);
            prev_ts = ts;
        }
    }

    #[test]
    fn test_timestamp_ordering_consistency() {
        let ts_sec = current_timestamp();
        let ts_ms = current_timestamp_ms();
        let ts_f64 = current_timestamp_f64();

        // 验证不同格式的时间戳排序一致
        assert!(ts_ms as u64 >= ts_sec * 1000);
        assert!(ts_f64 as u64 >= ts_sec);
    }

    // ========================================
    // 性能测试
    // ========================================

    #[test]
    fn test_timestamp_performance() {
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = current_timestamp();
        }
        let duration = start.elapsed();

        // 10000次调用应该在合理时间内完成（< 100ms）
        assert!(duration < std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_timestamp_ms_performance() {
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = current_timestamp_ms();
        }
        let duration = start.elapsed();

        // 10000次调用应该在合理时间内完成（< 100ms）
        assert!(duration < std::time::Duration::from_millis(100));
    }

    #[test]
    fn test_timestamp_nanos_performance() {
        let start = std::time::Instant::now();
        for _ in 0..10000 {
            let _ = current_timestamp_nanos();
        }
        let duration = start.elapsed();

        // 10000次调用应该在合理时间内完成（< 100ms）
        assert!(duration < std::time::Duration::from_millis(100));
    }

    // ========================================
    // 实用性测试
    // ========================================

    #[test]
    fn test_timestamp_for_unique_ids() {
        // 使用纳秒时间戳生成唯一ID
        let mut ids = std::collections::HashSet::new();
        for _ in 0..1000 {
            let id = current_timestamp_nanos();
            ids.insert(id);
        }

        // 所有ID应该是唯一的
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn test_timestamp_for_event_logging() {
        use std::collections::BTreeMap;

        // 模拟事件日志
        let mut event_log: BTreeMap<u64, String> = BTreeMap::new();

        event_log.insert(current_timestamp(), "event1".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        event_log.insert(current_timestamp(), "event2".to_string());
        std::thread::sleep(std::time::Duration::from_millis(10));
        event_log.insert(current_timestamp(), "event3".to_string());

        // 事件应该按时间排序
        let events: Vec<&String> = event_log.values().collect();
        assert_eq!(events[0], "event1");
        assert_eq!(events[1], "event2");
        assert_eq!(events[2], "event3");
    }

    #[test]
    fn test_timestamp_duration_calculation() {
        let start = current_timestamp_ms();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let end = current_timestamp_ms();

        let duration = end - start;
        assert!(duration >= 100);
        assert!(duration < 200); // Should be close to 100ms
    }

    // ========================================
    // 并发测试
    // ========================================

    #[test]
    fn test_timestamp_thread_safety() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let timestamps = Arc::new(Mutex::new(Vec::new()));
        let mut handles = vec![];

        for _ in 0..10 {
            let timestamps_clone = Arc::clone(&timestamps);
            let handle = thread::spawn(move || {
                let ts = current_timestamp();
                let mut data = timestamps_clone.lock().expect("Test: operation should succeed");
                data.push(ts);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Test: operation should succeed");
        }

        let data = timestamps.lock().expect("Test: operation should succeed");
        assert_eq!(data.len(), 10);

        // 所有时间戳应该是不同的
        let unique: std::collections::HashSet<_> = data.iter().collect();
        assert!(unique.len() >= 9); // At least 9 unique (some may collide)
    }

    // ========================================
    // F64 特定测试
    // ========================================

    #[test]
    fn test_timestamp_f64_range() {
        let ts = current_timestamp_f64();

        // 2024年的时间戳应该在1704067200到2000000000之间
        assert!(ts > 1704067200.0);
        assert!(ts < 2000000000.0);
    }

    #[test]
    fn test_timestamp_f64_fractional_seconds() {
        let ts1 = current_timestamp_f64();
        std::thread::sleep(std::time::Duration::from_millis(50));
        let ts2 = current_timestamp_f64();

        let diff = ts2 - ts1;
        // 差异应该大约是0.05秒
        assert!(diff >= 0.04); // At least 40ms
        assert!(diff < 0.1); // Less than 100ms
    }
}
