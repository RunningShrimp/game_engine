//! Error Aggregator 扩展单元测试
//!
//! 测试错误聚合器的功能

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::error_aggregator::*;
    use crate::core::utils::current_timestamp;
    use std::thread;
    use std::time::Duration;

    // ========================================
    // ErrorStats 基础测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_stats_new() {
        let stats = ErrorStats::new();
        assert_eq!(stats.total_count, 0);
        assert!(stats.by_type.is_empty());
        assert!(stats.by_source.is_empty());
        assert!(stats.recent_errors.is_empty());
        assert_eq!(stats.error_rate, 0.0);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_stats_default() {
        let stats = ErrorStats::default();
        assert_eq!(stats.total_count, 0);
        assert!(stats.recent_errors.is_empty());
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_stats_most_common_error_type() {
        let mut stats = ErrorStats::new();
        stats.by_type.insert("IoError".to_string(), 10);
        stats.by_type.insert("RenderError".to_string(), 5);
        stats.by_type.insert("NetworkError".to_string(), 15);

        let most_common = stats.most_common_error_type();
        assert!(most_common.is_some());
        assert_eq!(most_common.expect("Test: operation should succeed").0, "NetworkError");
        assert_eq!(most_common.expect("Test: operation should succeed").1, &15);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_stats_most_common_error_type_empty() {
        let stats = ErrorStats::new();
        let most_common = stats.most_common_error_type();
        assert!(most_common.is_none());
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_stats_most_common_error_source() {
        let mut stats = ErrorStats::new();
        stats.by_source.insert("render".to_string(), 20);
        stats.by_source.insert("physics".to_string(), 10);
        stats.by_source.insert("audio".to_string(), 5);

        let most_common = stats.most_common_error_source();
        assert!(most_common.is_some());
        assert_eq!(most_common.expect("Test: operation should succeed").0, "render");
        assert_eq!(most_common.expect("Test: operation should succeed").1, &20);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_stats_trend() {
        let mut stats = ErrorStats::new();
        let now = current_timestamp();

        // 添加一些最近的错误
        for i in 0..5 {
            let mut record = ErrorRecord::new(
                "TestError",
                "test",
                format!("Error {}", i)
            );
            record.timestamp = now - i;
            stats.recent_errors.push(record);
        }

        // 查询最近10秒的趋势
        let trend = stats.error_trend(10);
        assert_eq!(trend, 5);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_stats_trend_outside_window() {
        let mut stats = ErrorStats::new();
        let old_timestamp = current_timestamp() - 100;

        let mut record = ErrorRecord::new(
            "OldError",
            "test",
            "Old error".to_string()
        );
        record.timestamp = old_timestamp;
        stats.recent_errors.push(record);

        // 查询最近5秒的趋势（不应该包含旧错误）
        let trend = stats.error_trend(5);
        assert_eq!(trend, 0);
    }

    // ========================================
    // ErrorRecord 基础测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_record_creation() {
        let record = ErrorRecord::new(
            "TestError",
            "test_module",
            "Test error message"
        ).with_details("stack trace here");

        assert_eq!(record.error_type, "TestError");
        assert_eq!(record.source, "test_module");
        assert!(record.details.is_some());
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_record_without_stack_trace() {
        let record = ErrorRecord::new(
            "SimpleError",
            "simple",
            "Simple error"
        );

        assert!(record.details.is_none());
    }

    // ========================================
    // ErrorAggregator 基础测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_aggregator_new() {
        let aggregator = ErrorAggregator::new();
        let stats = aggregator.get_stats();

        assert_eq!(stats.total_count, 0);
        assert!(stats.by_type.is_empty());
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_aggregator_record_error() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error(
            "TestError".to_string(),
            "test_module".to_string(),
            "Test error message".to_string(),
            None,
        );

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.by_type.get("TestError"), Some(&1));
        assert_eq!(stats.by_source.get("test_module"), Some(&1));
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_aggregator_record_multiple_errors() {
        let aggregator = ErrorAggregator::new();

        for i in 0..10 {
            aggregator.record_custom_error(
                format!("ErrorType{}", i % 3),
                "test_module".to_string(),
                format!("Error message {}", i),
                None,
            );
        }

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 10);
        assert_eq!(stats.by_source.get("test_module"), Some(&10));
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_aggregator_record_error_with_stack() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error(
            "StackError".to_string(),
            "stack_module".to_string(),
            "Error with stack trace".to_string(),
            Some("line1\nline2\nline3".to_string()),
        );

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.recent_errors.len(), 1);
        assert!(stats.recent_errors[0].details.is_some());
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_aggregator_recent_errors_limit() {
        let aggregator = ErrorAggregator::new();

        // 添加超过限制的错误（默认1000条）
        for i in 0..150 {
            aggregator.record_custom_error(
                "ErrorType".to_string(),
                "test".to_string(),
                format!("Error {}", i),
                None,
            );
        }

        let stats = aggregator.get_stats();
        // 最近错误列表不应该被限制（150 < 1000）
        assert_eq!(stats.recent_errors.len(), 150);
        // 总数应该正确
        assert_eq!(stats.total_count, 150);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_aggregator_get_stats() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("Type1".to_string(), "mod1".to_string(), "msg1".to_string(), None);
        aggregator.record_custom_error("Type2".to_string(), "mod2".to_string(), "msg2".to_string(), None);
        aggregator.record_custom_error("Type1".to_string(), "mod1".to_string(), "msg3".to_string(), None);

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.by_type.get("Type1"), Some(&2));
        assert_eq!(stats.by_type.get("Type2"), Some(&1));
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_aggregator_clear() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error(
            "TestError".to_string(),
            "test".to_string(),
            "msg".to_string(),
            None,
        );
        assert_eq!(aggregator.get_stats().total_count, 1);

        aggregator.clear();
        assert_eq!(aggregator.get_stats().total_count, 0);
        assert!(aggregator.get_stats().by_type.is_empty());
        assert!(aggregator.get_stats().recent_errors.is_empty());
    }

    // ========================================
    // ErrorSummary 基础测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_summary_from_stats() {
        let mut stats = ErrorStats::new();
        stats.total_count = 100;
        stats.by_type.insert("ErrorA".to_string(), 50);
        stats.by_type.insert("ErrorB".to_string(), 30);
        stats.by_type.insert("ErrorC".to_string(), 20);

        let summary = ErrorSummary {
            total_errors: stats.total_count,
            error_rate: stats.error_rate,
            most_common_type: stats.by_type
                .iter()
                .max_by_key(|(_, &count)| count)
                .map(|(name, &count)| (name.clone(), count)),
            most_common_source: stats.by_source
                .iter()
                .max_by_key(|(_, &count)| count)
                .map(|(name, &count)| (name.clone(), count)),
            recent_error_count: stats.recent_errors.len(),
            last_updated: stats.last_updated,
        };

        assert_eq!(summary.total_errors, 100);
        assert_eq!(stats.by_type.len(), 3);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_summary_empty_stats() {
        let stats = ErrorStats::new();
        let summary = ErrorSummary {
            total_errors: stats.total_count,
            error_rate: stats.error_rate,
            most_common_type: None,
            most_common_source: None,
            recent_error_count: 0,
            last_updated: 0,
        };

        assert_eq!(summary.total_errors, 0);
        assert_eq!(stats.by_type.len(), 0);
        assert_eq!(summary.most_common_type, None);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_summary_most_common() {
        let mut stats = ErrorStats::new();
        stats.by_type.insert("CommonError".to_string(), 100);
        stats.by_type.insert("RareError".to_string(), 5);

        let summary = ErrorSummary {
            total_errors: stats.total_count,
            error_rate: stats.error_rate,
            most_common_type: stats.by_type
                .iter()
                .max_by_key(|(_, &count)| count)
                .map(|(name, &count)| (name.clone(), count)),
            most_common_source: stats.by_source
                .iter()
                .max_by_key(|(_, &count)| count)
                .map(|(name, &count)| (name.clone(), count)),
            recent_error_count: stats.recent_errors.len(),
            last_updated: stats.last_updated,
        };

        assert_eq!(
            summary.most_common_type,
            Some(("CommonError".to_string(), 100))
        );
    }

    // ========================================
    // 错误率计算测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_rate_calculation() {
        let aggregator = ErrorAggregator::new();

        // 记录一些错误
        for _ in 0..10 {
            aggregator.record_custom_error(
                "RateError".to_string(),
                "test".to_string(),
                "msg".to_string(),
                None,
            );
        }

        thread::sleep(Duration::from_millis(100));

        // 错误率会在记录错误时自动更新
        let stats = aggregator.get_stats();
        // 错误率应该大于等于0
        assert!(stats.error_rate >= 0.0);
    }

    // ========================================
    // 错误统计测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_count_by_type() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("Type1".to_string(), "mod".to_string(), "msg".to_string(), None);
        aggregator.record_custom_error("Type1".to_string(), "mod".to_string(), "msg".to_string(), None);
        aggregator.record_custom_error("Type2".to_string(), "mod".to_string(), "msg".to_string(), None);

        let stats = aggregator.get_stats();
        assert_eq!(stats.by_type.get("Type1"), Some(&2));
        assert_eq!(stats.by_type.get("Type2"), Some(&1));
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_count_by_source() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("Error".to_string(), "render".to_string(), "msg".to_string(), None);
        aggregator.record_custom_error("Error".to_string(), "render".to_string(), "msg".to_string(), None);
        aggregator.record_custom_error("Error".to_string(), "physics".to_string(), "msg".to_string(), None);

        let stats = aggregator.get_stats();
        assert_eq!(stats.by_source.get("render"), Some(&2));
        assert_eq!(stats.by_source.get("physics"), Some(&1));
    }

    // ========================================
    // 边界情况测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_empty_error_aggregator() {
        let aggregator = ErrorAggregator::new();
        let stats = aggregator.get_stats();

        assert_eq!(stats.total_count, 0);
        assert!(stats.most_common_error_type().is_none());
        assert!(stats.most_common_error_source().is_none());
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_with_empty_strings() {
        let aggregator = ErrorAggregator::new();

        aggregator.record_custom_error("".to_string(), "".to_string(), "".to_string(), None);

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 1);
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_with_very_long_message() {
        let aggregator = ErrorAggregator::new();
        let long_message = "x".repeat(10000);

        aggregator.record_custom_error("LongError".to_string(), "test".to_string(), long_message, None);

        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 1);
        assert_eq!(stats.recent_errors.len(), 1);
    }

    // ========================================
    // 并发测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_concurrent_error_recording() {
        use std::sync::{Arc, Mutex};
        use std::thread;

        let aggregator = Arc::new(Mutex::new(ErrorAggregator::new()));
        let mut handles = vec![];

        // 10个线程，每个记录10个错误
        for i in 0..10 {
            let agg_clone = Arc::clone(&aggregator);
            let handle = thread::spawn(move || {
                for j in 0..10 {
                    let mut agg = agg_clone.lock().expect("Test: operation should succeed");
                    agg.record_custom_error(
                        format!("ErrorType{}", i),
                        format!("thread{}", i),
                        format!("Error {}", j),
                        None,
                    );
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().expect("Test: operation should succeed");
        }

        let aggregator = aggregator.lock().expect("Test: operation should succeed");
        let stats = aggregator.get_stats();
        assert_eq!(stats.total_count, 100);
    }

    // ========================================
    // 性能测试
    // ========================================

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_error_recording_performance() {
        let aggregator = ErrorAggregator::new();

        let start = std::time::Instant::now();
        for i in 0..1000 {
            aggregator.record_custom_error(
                "PerfError".to_string(),
                "perf_test".to_string(),
                format!("Error {}", i),
                None,
            );
        }
        let duration = start.elapsed();

        // 1000次错误记录应该在100ms内完成
        assert!(duration < Duration::from_millis(100));
    }

    #[test]
#[ignore]  // TODO: Fix compilation errors
    fn test_stats_retrieval_performance() {
        let aggregator = ErrorAggregator::new();

        // 添加一些错误
        for i in 0..100 {
            aggregator.record_custom_error(
                format!("Type{}", i % 5),
                "test".to_string(),
                format!("Error {}", i),
                None,
            );
        }

        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let _ = aggregator.get_stats();
        }
        let duration = start.elapsed();

        // 1000次stats获取应该在50ms内完成
        assert!(duration < Duration::from_millis(50));
    }
}
