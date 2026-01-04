//! 协程等待条件
//!
//! 提供各种等待机制，包括时间等待、条件等待和协程间等待。

use super::{CoroutineError, CoroutineId};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, mpsc};

// =============================================================================
// 等待秒数
// =============================================================================

/// 等待指定秒数
///
/// # 示例
///
/// ```rust
/// use game_engine::coroutine::WaitForSeconds;
///
/// async fn example() {
///     WaitForSeconds::new(1.5).await.unwrap();
///     // 等待1.5秒后继续
/// }
/// ```
pub struct WaitForSeconds {
    duration: Duration,
    start: Instant,
}

impl WaitForSeconds {
    /// 创建新的等待
    ///
    /// # 参数
    ///
    /// - `seconds`: 等待的秒数（浮点数）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::WaitForSeconds;
    ///
    /// let wait = WaitForSeconds::new(2.0); // 等待2秒
    /// ```
    pub fn new(seconds: f32) -> Self {
        Self {
            duration: Duration::from_secs_f32(seconds),
            start: Instant::now(),
        }
    }

    /// 创建新的等待（Duration版本）
    ///
    /// # 参数
    ///
    /// - `duration`: 等待的时长
    pub fn from_duration(duration: Duration) -> Self {
        Self {
            duration,
            start: Instant::now(),
        }
    }
}

impl Future for WaitForSeconds {
    type Output = Result<(), CoroutineError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.start.elapsed() >= self.duration {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}

// =============================================================================
// 等待帧数
// =============================================================================

/// 等待指定帧数
///
/// # 注意
///
/// 帧等待需要在游戏循环中每帧调用poll才能正常工作。
///
/// # 示例
///
/// ```rust
/// use game_engine::coroutine::WaitForFrames;
///
/// async fn example() {
///     WaitForFrames::new(60).await.unwrap();
///     // 等待60帧后继续
/// }
/// ```
pub struct WaitForFrames {
    frames_remaining: u32,
}

impl WaitForFrames {
    /// 创建新的等待
    ///
    /// # 参数
    ///
    /// - `frames`: 要等待的帧数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::WaitForFrames;
    ///
    /// let wait = WaitForFrames::new(30); // 等待30帧
    /// ```
    pub fn new(frames: u32) -> Self {
        Self {
            frames_remaining: frames,
        }
    }

    /// 获取剩余帧数
    pub fn remaining_frames(&self) -> u32 {
        self.frames_remaining
    }
}

impl Future for WaitForFrames {
    type Output = Result<(), CoroutineError>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.frames_remaining == 0 {
            Poll::Ready(Ok(()))
        } else {
            self.frames_remaining -= 1;
            Poll::Pending
        }
    }
}

// =============================================================================
// 等待条件
// =============================================================================

/// 等待条件满足
///
/// # 示例
///
/// ```rust
/// use game_engine::coroutine::WaitCondition;
/// use std::sync::atomic::{AtomicBool, Ordering};
/// use std::sync::Arc;
///
/// async fn example() {
///     let flag = Arc::new(AtomicBool::new(false));
///     let flag_clone = flag.clone();
///
///     // 在另一个线程中设置条件
///     std::thread::spawn(move || {
///         std::thread::sleep(std::time::Duration::from_millis(100));
///         flag_clone.store(true, Ordering::SeqCst);
///     });
///
///     // 等待条件满足
///     WaitCondition::new(move || flag.load(Ordering::SeqCst)).await.unwrap();
/// }
/// ```
pub struct WaitCondition<F>
where
    F: Fn() -> bool,
{
    condition: F,
}

impl<F> WaitCondition<F>
where
    F: Fn() -> bool,
{
    /// 创建新的等待条件
    ///
    /// # 参数
    ///
    /// - `condition`: 返回bool的闭包，true表示条件满足
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::WaitCondition;
    ///
    /// let ready = false;
    /// let wait = WaitCondition::new(|| ready);
    /// ```
    pub fn new(condition: F) -> Self {
        Self { condition }
    }
}

impl<F> Future for WaitCondition<F>
where
    F: Fn() -> bool,
{
    type Output = Result<(), CoroutineError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if (self.condition)() {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}

// =============================================================================
// 协程等待器
// =============================================================================

/// 协程等待器
///
/// 用于管理协程间的等待关系，允许一个协程等待另一个或多个协程完成。
///
/// # 示例
///
/// ```rust
/// use game_engine::coroutine::{CoroutineId, CoroutineWaiter};
///
/// async fn example(waiter: &CoroutineWaiter) {
///     let target = CoroutineId::new(123);
///     waiter.wait_for(target).await.unwrap();
///     // 等待目标协程完成后继续
/// }
/// ```
#[derive(Clone)]
pub struct CoroutineWaiter {
    /// 等待中的协程映射: CoroutineId -> Vec<Waker>
    waiters: Arc<Mutex<HashMap<CoroutineId, Vec<Waker>>>>,
    /// 通知通道
    notify_tx: mpsc::Sender<CoroutineId>,
}

impl CoroutineWaiter {
    /// 创建新的协程等待器
    ///
    /// # 返回
    ///
    /// 返回一个`CoroutineWaiter`实例，可以在多个协程间共享。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineWaiter;
    ///
    /// let waiter = CoroutineWaiter::new();
    /// ```
    pub fn new() -> Self {
        let waiters = Arc::new(Mutex::new(HashMap::new()));
        let (notify_tx, mut notify_rx) = mpsc::channel::<CoroutineId>(1000);

        // 启动后台任务处理通知
        let waiters_clone = waiters.clone();
        tokio::spawn(async move {
            while let Some(id) = notify_rx.recv().await {
                let mut waiters = waiters_clone.lock().await;
                if let Some(wakers) = waiters.remove(&id) {
                    // 唤醒所有等待该协程的waker
                    for waker in wakers {
                        let waker: Waker = waker;
                        waker.wake();
                    }
                }
            }
        });

        Self { waiters, notify_tx }
    }

    /// 等待单个协程完成
    ///
    /// # 参数
    ///
    /// - `target`: 要等待的目标协程ID
    ///
    /// # 返回
    ///
    /// 当目标协程完成时返回`Ok(())`，否则返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::{CoroutineWaiter, CoroutineId};
    ///
    /// async fn example(waiter: &CoroutineWaiter) {
    ///     let target = CoroutineId::new(1);
    ///     waiter.wait_for(target).await.unwrap();
    ///     println!("Target coroutine completed");
    /// }
    /// ```
    pub async fn wait_for(&self, target: CoroutineId) -> Result<(), CoroutineError> {
        WaitForCoroutine::new(target, self.clone()).await
    }

    /// 等待多个协程全部完成
    ///
    /// # 参数
    ///
    /// - `targets`: 要等待的目标协程ID列表
    ///
    /// # 返回
    ///
    /// 当所有目标协程都完成时返回`Ok(())`，否则返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::{CoroutineWaiter, CoroutineId};
    ///
    /// async fn example(waiter: &CoroutineWaiter) {
    ///     let targets = vec![
    ///         CoroutineId::new(1),
    ///         CoroutineId::new(2),
    ///         CoroutineId::new(3),
    ///     ];
    ///     waiter.wait_all(targets).await.unwrap();
    ///     println!("All coroutines completed");
    /// }
    /// ```
    pub async fn wait_all(&self, targets: Vec<CoroutineId>) -> Result<(), CoroutineError> {
        // 并行等待所有协程
        let mut wait_futures = Vec::new();
        for target in targets {
            wait_futures.push(self.wait_for(target));
        }

        // 等待所有完成
        futures::future::join_all(wait_futures).await;
        Ok(())
    }

    /// 通知等待者某个协程已完成
    ///
    /// # 参数
    ///
    /// - `id`: 已完成的协程ID
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::{CoroutineWaiter, CoroutineId};
    ///
    /// fn example(waiter: &CoroutineWaiter) {
    ///     let id = CoroutineId::new(1);
    ///     waiter.notify(id);
    ///     // 所有等待id的协程将被唤醒
    /// }
    /// ```
    pub fn notify(&self, id: CoroutineId) {
        if self.notify_tx.try_send(id).is_err() {
            eprintln!("Failed to send notification for coroutine {id:?}");
        }
    }

    /// 注册等待者（内部使用）
    async fn register_waiter(&self, target: CoroutineId, waker: Waker) {
        let mut waiters = self.waiters.lock().await;
        waiters.entry(target).or_insert_with(Vec::new).push(waker);
    }
}

impl Default for CoroutineWaiter {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// 等待单个协程
// =============================================================================

/// 等待另一个协程完成
///
/// 这是一个Future，用于在协程内部等待另一个协程完成。
///
/// # 示例
///
/// ```rust
/// use game_engine::coroutine::{CoroutineId, CoroutineWaiter, WaitForCoroutine};
///
/// async fn example(waiter: &CoroutineWaiter) {
///     let target = CoroutineId::new(123);
///     WaitForCoroutine::new(target, waiter.clone()).await.unwrap();
/// }
/// ```
pub struct WaitForCoroutine {
    target: CoroutineId,
    waiter: CoroutineWaiter,
    registered: bool,
}

impl WaitForCoroutine {
    /// 创建新的协程等待
    ///
    /// # 参数
    ///
    /// - `target`: 要等待的目标协程ID
    /// - `waiter`: 协程等待器实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::{CoroutineId, CoroutineWaiter, WaitForCoroutine};
    ///
    /// let wait = WaitForCoroutine::new(
    ///     CoroutineId::new(1),
    ///     waiter.clone()
    /// );
    /// ```
    pub fn new(target: CoroutineId, waiter: CoroutineWaiter) -> Self {
        Self {
            target,
            waiter,
            registered: false,
        }
    }
}

impl Future for WaitForCoroutine {
    type Output = Result<(), CoroutineError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 注册waker
        if !self.registered {
            self.registered = true;

            // 克隆必要的数据
            let target = self.target;
            let waiter = self.waiter.clone();
            let waker = cx.waker().clone();

            // 异步注册等待者
            let register_future = async move {
                waiter.register_waiter(target, waker).await;
            };

            // 在后台执行注册
            tokio::spawn(register_future);
        }

        // 返回Pending，等待通过notify唤醒
        Poll::Pending
    }
}

// =============================================================================
// 便利函数
// =============================================================================

/// 等待指定秒数
pub async fn yield_seconds(seconds: f32) -> Result<(), CoroutineError> {
    WaitForSeconds::new(seconds).await
}

/// 等待指定帧数
pub async fn yield_frames(frames: u32) -> Result<(), CoroutineError> {
    WaitForFrames::new(frames).await
}

/// 等待条件满足
pub async fn wait_until<F>(condition: F) -> Result<(), CoroutineError>
where
    F: Fn() -> bool,
{
    WaitCondition::new(condition).await
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wait_for_seconds() {
        let start = Instant::now();
        WaitForSeconds::new(0.1).await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_wait_for_frames() {
        // 注意：帧等待需要在外部驱动，这里仅测试API
        let wait = WaitForFrames::new(5);
        assert_eq!(wait.frames_remaining, 5);
    }

    #[tokio::test]
    async fn test_wait_condition() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};

        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();

        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            flag_clone.store(true, Ordering::SeqCst);
        });

        let result = WaitCondition::new(|| flag.load(Ordering::SeqCst)).await;
        handle.join().unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_yield_seconds() {
        let start = Instant::now();
        yield_seconds(0.05).await.unwrap();
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(50));
    }
}
