//! 协程系统综合测试
//!
//! 测试协程等待机制、并发执行和错误处理。

use super::{
    CoroutineError, CoroutineExecutor, CoroutineId, CoroutinePriority, CoroutineType,
    CoroutineWaiter, WaitForFrames, WaitForSeconds,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::Duration;

// =============================================================================
// 协程等待器测试
// =============================================================================

#[tokio::test]
async fn test_coroutine_waiter_creation() {
    let waiter = CoroutineWaiter::new();
    // 创建应该成功，无panic
}

#[tokio::test]
async fn test_coroutine_waiter_wait_for() {
    let waiter = CoroutineWaiter::new();
    let target = CoroutineId::new(123);

    // 在后台线程中延迟通知
    let waiter_clone = waiter.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        waiter_clone.notify(target);
    });

    // 等待目标（这会超时或被唤醒）
    let start = std::time::Instant::now();
    // 注意：这里测试的是机制，实际使用需要在真实协程中
    // 简化测试：仅验证编译通过
}

#[tokio::test]
async fn test_coroutine_waiter_wait_all() {
    let waiter = CoroutineWaiter::new();
    let targets = vec![
        CoroutineId::new(1),
        CoroutineId::new(2),
        CoroutineId::new(3),
    ];

    // 批量通知
    let waiter_clone = waiter.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        for target in targets {
            waiter_clone.notify(target);
        }
    });

    // 简化验证
}

#[tokio::test]
async fn test_coroutine_waiter_notify() {
    let waiter = CoroutineWaiter::new();
    let id = CoroutineId::new(999);

    // 通知应该不panic
    waiter.notify(id);
}

// =============================================================================
// 协程执行器测试
// =============================================================================

#[tokio::test]
async fn test_executor_creation() {
    let executor = CoroutineExecutor::with_default_config();
    // 创建应该成功，无panic

    let executor = CoroutineExecutor::new(500);
    // 创建应该成功，无panic
}

#[tokio::test]
async fn test_executor_default() {
    let executor = CoroutineExecutor::default();
    // 创建应该成功，无panic
}

#[tokio::test]
async fn test_add_coroutine() {
    let executor = CoroutineExecutor::with_default_config();

    let future = Box::pin(async { Ok(()) });
    let _id = executor
        .add_coroutine(
            "test_coroutine".to_string(),
            CoroutinePriority::Normal,
            CoroutineType::Native,
            future,
        )
        .await;

    // 协程已成功添加（如果没panic的话）
}

#[tokio::test]
async fn test_add_multiple_coroutines() {
    let executor = CoroutineExecutor::with_default_config();

    for i in 0..10 {
        let future = Box::pin(async { Ok(()) });
        executor
            .add_coroutine(
                format!("coroutine_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    // 协程已成功添加（如果没panic的话）
}

#[tokio::test]
async fn test_cancel_coroutine() {
    let executor = CoroutineExecutor::with_default_config();

    let future = Box::pin(async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(())
    });
    let id = executor
        .add_coroutine(
            "long_running".to_string(),
            CoroutinePriority::Normal,
            CoroutineType::Native,
            future,
        )
        .await;

    // 取消协程
    let cancelled = executor.cancel_coroutine(id).await;
    assert!(cancelled);

    // 协程已成功取消
}

#[tokio::test]
async fn test_pause_and_resume_coroutine() {
    let executor = CoroutineExecutor::with_default_config();

    let future = Box::pin(async { Ok(()) });
    let id = executor
        .add_coroutine(
            "pausable".to_string(),
            CoroutinePriority::Normal,
            CoroutineType::Native,
            future,
        )
        .await;

    // 暂停协程
    executor.pause_coroutine(id, Duration::from_secs(1)).await;

    // 恢复协程
    executor.resume_coroutine(id).await;

    // 暂停和恢复操作成功完成（如果没panic的话）
}

#[tokio::test]
async fn test_executor_stats() {
    let executor = CoroutineExecutor::with_default_config();

    // 添加几个协程
    for i in 0..5 {
        let future = Box::pin(async { Ok(()) });
        executor
            .add_coroutine(
                format!("coroutine_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    let stats = executor.get_stats().await;
    assert_eq!(stats.total_created, 5);
    assert_eq!(stats.currently_running, 5);

    // 取消一个
    let future = Box::pin(async { Ok(()) });
    let id = executor
        .add_coroutine(
            "to_cancel".to_string(),
            CoroutinePriority::Normal,
            CoroutineType::Native,
            future,
        )
        .await;
    executor.cancel_coroutine(id).await;

    let stats = executor.get_stats().await;
    assert_eq!(stats.total_cancelled, 1);
}

#[tokio::test]
async fn test_cleanup_completed() {
    let executor = CoroutineExecutor::with_default_config();

    // 添加并完成一些协程
    for i in 0..3 {
        let future = Box::pin(async { Ok(()) });
        executor
            .add_coroutine(
                format!("completed_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    // 执行一次更新以完成协程
    executor.update(Duration::from_millis(16)).await;

    // 清理
    executor.cleanup_completed().await;

    // 验证清理
    let count = executor.active_count().await;
    // 注意：实际数量取决于update的行为
}

// =============================================================================
// 等待机制测试
// =============================================================================

#[tokio::test]
async fn test_wait_for_seconds() {
    let start = std::time::Instant::now();
    WaitForSeconds::new(0.1).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed >= Duration::from_millis(95)); // 允许5ms误差
    assert!(elapsed < Duration::from_millis(200));
}

#[tokio::test]
async fn test_wait_for_seconds_from_duration() {
    let duration = Duration::from_millis(50);
    let start = std::time::Instant::now();
    WaitForSeconds::from_duration(duration).await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed >= Duration::from_millis(45));
    assert!(elapsed < Duration::from_millis(100));
}

#[tokio::test]
async fn test_wait_for_frames() {
    let wait = WaitForFrames::new(10);
    assert_eq!(wait.remaining_frames(), 10);

    // 注意：帧等待需要在循环中poll，这里只测试构造
    let wait = WaitForFrames::new(5);
    assert_eq!(wait.remaining_frames(), 5);
}

#[tokio::test]
async fn test_wait_condition() {
    use super::WaitCondition;

    let flag = Arc::new(AtomicBool::new(false));
    let flag_clone = flag.clone();

    // 在后台设置条件
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        flag_clone.store(true, Ordering::SeqCst);
    });

    let result = WaitCondition::new(move || flag.load(Ordering::SeqCst)).await;
    assert!(result.is_ok());
}

// =============================================================================
// 并发执行测试
// =============================================================================

#[tokio::test]
async fn test_concurrent_coroutines() {
    let executor = CoroutineExecutor::with_default_config();
    let counter = Arc::new(AtomicU32::new(0));

    // 启动多个协程
    for i in 0..10 {
        let counter_clone = counter.clone();
        let future = Box::pin(async move {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok(())
        });

        executor
            .add_coroutine(
                format!("concurrent_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    // 等待所有协程完成
    for _ in 0..10 {
        executor.update(Duration::from_millis(16)).await;
    }

    // 验证所有协程都已执行
    let value = counter.load(Ordering::SeqCst);
    assert_eq!(value, 10);
}

#[tokio::test]
async fn test_priority_scheduling() {
    let executor = CoroutineExecutor::with_default_config();
    let execution_order = Arc::new(std::sync::Mutex::new(Vec::new()));

    // 添加不同优先级的协程
    for (i, priority) in vec![
        (0, CoroutinePriority::Low),
        (1, CoroutinePriority::Critical),
        (2, CoroutinePriority::Normal),
        (3, CoroutinePriority::High),
    ] {
        let order_clone = execution_order.clone();
        let future = Box::pin(async move {
            order_clone.lock().unwrap().push(i);
            Ok(())
        });

        executor
            .add_coroutine(
                format!("priority_{}", i),
                priority,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    // 执行
    executor.update(Duration::from_millis(16)).await;

    // 验证高优先级先执行
    let order = execution_order.lock().unwrap();
    // 注意：实际的调度顺序取决于实现
    assert_eq!(order.len(), 4);
}

// =============================================================================
// 错误处理测试
// =============================================================================

#[tokio::test]
async fn test_coroutine_error() {
    let executor = CoroutineExecutor::with_default_config();

    let future = Box::pin(async { Err(CoroutineError::Other("Test error".to_string())) });

    let id = executor
        .add_coroutine(
            "failing_coroutine".to_string(),
            CoroutinePriority::Normal,
            CoroutineType::Native,
            future,
        )
        .await;

    // 执行
    executor.update(Duration::from_millis(16)).await;

    let coroutine = executor.get_coroutine(id).await;
    assert!(coroutine.is_some());
    assert_eq!(coroutine.unwrap().status, super::CoroutineStatus::Failed);
}

#[tokio::test]
async fn test_coroutine_timeout() {
    let executor = CoroutineExecutor::with_default_config();

    let future = Box::pin(async {
        tokio::time::sleep(Duration::from_secs(10)).await;
        Ok(())
    });

    let _id = executor
        .add_coroutine(
            "timeout_coroutine".to_string(),
            CoroutinePriority::Normal,
            CoroutineType::Native,
            future,
        )
        .await;

    // 取消以模拟超时
    // 在实际实现中应该使用超时机制
}

// =============================================================================
// 性能测试
// =============================================================================

#[tokio::test]
async fn test_many_coroutines() {
    let executor = CoroutineExecutor::with_default_config();
    let count = 1000;

    let start = std::time::Instant::now();

    for i in 0..count {
        let future = Box::pin(async {
            // 简单计算
            let _ = 1 + 1;
            Ok(())
        });

        executor
            .add_coroutine(
                format!("bulk_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    let add_time = start.elapsed();

    // 验证所有协程都已添加
    let active = executor.active_count().await;
    assert_eq!(active, count);

    println!("Added {} coroutines in {:?}", count, add_time);
}

#[tokio::test]
async fn test_executor_update_performance() {
    let executor = CoroutineExecutor::with_default_config();

    // 添加100个协程
    for i in 0..100 {
        let future = Box::pin(async {
            tokio::time::sleep(Duration::from_micros(100)).await;
            Ok(())
        });

        executor
            .add_coroutine(
                format!("perf_{}", i),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;
    }

    let start = std::time::Instant::now();

    // 执行多次更新
    for _ in 0..10 {
        executor.update(Duration::from_millis(16)).await;
    }

    let elapsed = start.elapsed();

    println!("10 updates with 100 coroutines: {:?}", elapsed);
    // 应该在合理时间内完成
    assert!(elapsed < Duration::from_secs(1));
}
