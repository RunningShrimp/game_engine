//! 协程等待条件
//!
//! 提供各种等待机制。

use super::CoroutineError;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

// =============================================================================
// 等待秒数
// =============================================================================

/// 等待指定秒数
pub struct WaitForSeconds {
    duration: Duration,
    start: Instant,
}

impl WaitForSeconds {
    /// 创建新的等待
    pub fn new(seconds: f32) -> Self {
        Self {
            duration: Duration::from_secs_f32(seconds),
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
pub struct WaitForFrames {
    frames_remaining: u32,
}

impl WaitForFrames {
    /// 创建新的等待
    pub fn new(frames: u32) -> Self {
        Self {
            frames_remaining: frames,
        }
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
// 等待协程
// =============================================================================

/// 等待另一个协程完成
pub struct WaitForCoroutine {
    // TODO: 实现协程间等待
    _phantom: std::marker::PhantomData<()>,
}

impl WaitForCoroutine {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl Future for WaitForCoroutine {
    type Output = Result<(), CoroutineError>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: 实现真正的协程等待
        Poll::Ready(Ok(()))
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
        let mut flag = false;
        let handle = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            flag = true;
        });

        let result = WaitCondition::new(|| flag).await;
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
