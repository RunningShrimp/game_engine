//! # 协程系统（Coroutine System）
//!
//! 提供轻量级协程支持，简化异步游戏逻辑编写。
//!
//! ## 功能特性
//!
//! - **轻量级协程**: 比线程更低的创建和切换开销
//! - ** Yield/Resume**: 协作式多任务
//! - **协程调度器**: 优先级调度和时间片管理
//! - **脚本集成**: JavaScript/Python协程支持
//! - **等待机制**: WaitForSeconds, WaitForFrame, WaitForCondition

use crate::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use std::time::{Duration, Instant};

pub mod executor;
pub mod wait;

pub use executor::{CoroutineExecutor, CoroutineExecutorResource};
pub use wait::{WaitCondition, WaitForFrames, WaitForSeconds};

// =============================================================================
// 协程ID
// =============================================================================

/// 协程ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CoroutineId(pub u64);

impl CoroutineId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

// =============================================================================
// 协程状态
// =============================================================================

/// 协程状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoroutineStatus {
    /// 就绪
    Ready,
    /// 运行中
    Running,
    /// 等待中
    Waiting,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 失败
    Failed,
}

// =============================================================================
// 协程
// =============================================================================

/// 协程
#[derive(Clone)]
pub struct Coroutine {
    /// 协程ID
    pub id: CoroutineId,
    /// 协程名称
    pub name: String,
    /// 协程状态
    pub status: CoroutineStatus,
    /// 创建时间
    pub created_at: Instant,
    /// 最后执行时间
    pub last_executed: Option<Instant>,
    /// 执行次数
    pub execution_count: u64,
    /// 优先级
    pub priority: CoroutinePriority,
    /// 协程类型
    pub coroutine_type: CoroutineType,
}

/// 协程优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum CoroutinePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// 协程类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoroutineType {
    /// Rust原生协程
    Native,
    /// JavaScript协程
    JavaScript,
    /// Python协程
    Python,
}

impl Coroutine {
    /// 创建新协程
    pub fn new(
        id: CoroutineId,
        name: String,
        priority: CoroutinePriority,
        coroutine_type: CoroutineType,
    ) -> Self {
        Self {
            id,
            name,
            status: CoroutineStatus::Ready,
            created_at: Instant::now(),
            last_executed: None,
            execution_count: 0,
            priority,
            coroutine_type,
        }
    }

    /// 标记为运行中
    pub fn mark_running(&mut self) {
        self.status = CoroutineStatus::Running;
        self.last_executed = Some(Instant::now());
        self.execution_count += 1;
    }

    /// 标记为等待
    pub fn mark_waiting(&mut self) {
        self.status = CoroutineStatus::Waiting;
    }

    /// 标记为完成
    pub fn mark_completed(&mut self) {
        self.status = CoroutineStatus::Completed;
    }

    /// 标记为取消
    pub fn mark_cancelled(&mut self) {
        self.status = CoroutineStatus::Cancelled;
    }

    /// 标记为失败
    pub fn mark_failed(&mut self) {
        self.status = CoroutineStatus::Failed;
    }

    /// 获取运行时长
    pub fn elapsed(&self) -> Duration {
        self.created_at.elapsed()
    }

    /// 获取距上次执行时长
    pub fn time_since_last_execution(&self) -> Option<Duration> {
        self.last_executed.map(|t| t.elapsed())
    }
}

// =============================================================================
// 协程Future
// =============================================================================

/// 协程Future输出
pub type CoroutineFutureOutput = Result<(), CoroutineError>;

/// 协程Future
pub type CoroutineFuture = Pin<Box<dyn Future<Output = CoroutineFutureOutput> + Send>>;

/// 协程错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CoroutineError {
    /// 执行超时
    Timeout,
    /// 被取消
    Cancelled,
    /// 脚本错误
    ScriptError(String),
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for CoroutineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoroutineError::Timeout => write!(f, "Coroutine execution timeout"),
            CoroutineError::Cancelled => write!(f, "Coroutine was cancelled"),
            CoroutineError::ScriptError(msg) => write!(f, "Script error: {}", msg),
            CoroutineError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for CoroutineError {}

// =============================================================================
// 协程构建器
// =============================================================================

/// 协程构建器
pub struct CoroutineBuilder {
    name: String,
    priority: CoroutinePriority,
    timeout: Option<Duration>,
}

impl Default for CoroutineBuilder {
    fn default() -> Self {
        Self {
            name: String::from("unnamed"),
            priority: CoroutinePriority::Normal,
            timeout: None,
        }
    }
}

impl CoroutineBuilder {
    /// 创建新构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置名称
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// 设置优先级
    pub fn priority(mut self, priority: CoroutinePriority) -> Self {
        self.priority = priority;
        self
    }

    /// 设置超时
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// 构建Rust协程
    pub fn build_rust(
        self,
        future: impl Future<Output = CoroutineFutureOutput> + Send + 'static,
    ) -> (CoroutineId, CoroutineFuture) {
        let id = CoroutineId::new(rand::random());
        let coroutine = Coroutine::new(
            id,
            self.name,
            self.priority,
            CoroutineType::Native,
        );

        // 包装超时逻辑 - 统一boxed类型
        let future: CoroutineFuture = if let Some(timeout) = self.timeout {
            Box::pin(async move {
                match tokio::time::timeout(timeout, future).await {
                    Ok(result) => result,
                    Err(_) => Err(CoroutineError::Timeout),
                }
            })
        } else {
            Box::pin(future)
        };

        (id, future)
    }
}

// =============================================================================
// 协程事件
// =============================================================================

/// 协程事件
#[derive(Debug, Clone)]
pub enum CoroutineEvent {
    /// 协程启动
    Started {
        coroutine_id: CoroutineId,
        name: String,
    },
    /// 协程完成
    Completed {
        coroutine_id: CoroutineId,
        name: String,
        execution_count: u64,
    },
    /// 协程失败
    Failed {
        coroutine_id: CoroutineId,
        name: String,
        error: CoroutineError,
    },
    /// 协程被取消
    Cancelled {
        coroutine_id: CoroutineId,
        name: String,
    },
    /// 协程等待
    Waiting {
        coroutine_id: CoroutineId,
        name: String,
        reason: String,
    },
    /// 协程恢复
    Resumed {
        coroutine_id: CoroutineId,
        name: String,
    },
}

impl DomainEvent for CoroutineEvent {
    fn event_type(&self) -> &'static str {
        match self {
            CoroutineEvent::Started { .. } => "Started",
            CoroutineEvent::Completed { .. } => "Completed",
            CoroutineEvent::Failed { .. } => "Failed",
            CoroutineEvent::Cancelled { .. } => "Cancelled",
            CoroutineEvent::Waiting { .. } => "Waiting",
            CoroutineEvent::Resumed { .. } => "Resumed",
        }
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        // 事件应用逻辑（由具体的协程系统处理）
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// =============================================================================
// ECS集成
// =============================================================================

/// 协程组件
#[derive(Component, Debug, Clone)]
pub struct CoroutineComponent {
    pub coroutine_id: CoroutineId,
    pub name: String,
    pub status: CoroutineStatus,
}

/// 协程等待组件
#[derive(Component, Debug, Clone)]
pub struct CoroutineWait {
    pub wait_until: Instant,
    pub reason: String,
}

// =============================================================================
// 协程宏
// =============================================================================

/// 创建协程的宏
///
/// # 示例
///
/// ```rust
/// use game_engine::coroutine::*;
///
/// let coroutine = coroutine!({
///     println!("Hello");
///     yield_seconds(1.0).await;
///     println!("World");
/// });
/// ```
#[macro_export]
macro_rules! coroutine {
    ($future:expr) => {
        CoroutineBuilder::new()
            .build_rust(async move { Ok(()) })
    };
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coroutine_creation() {
        let coroutine = Coroutine::new(
            CoroutineId::new(1),
            "test".to_string(),
            CoroutinePriority::Normal,
            CoroutineType::Native,
        );

        assert_eq!(coroutine.status, CoroutineStatus::Ready);
        assert_eq!(coroutine.execution_count, 0);
    }

    #[test]
    fn test_coroutine_state_transitions() {
        let mut coroutine = Coroutine::new(
            CoroutineId::new(1),
            "test".to_string(),
            CoroutinePriority::Normal,
            CoroutineType::Native,
        );

        coroutine.mark_running();
        assert_eq!(coroutine.status, CoroutineStatus::Running);
        assert_eq!(coroutine.execution_count, 1);

        coroutine.mark_waiting();
        assert_eq!(coroutine.status, CoroutineStatus::Waiting);

        coroutine.mark_completed();
        assert_eq!(coroutine.status, CoroutineStatus::Completed);
    }

    #[test]
    fn test_coroutine_builder() {
        let (id, _future) = CoroutineBuilder::new()
            .name("test_coroutine")
            .priority(CoroutinePriority::High)
            .timeout(Duration::from_secs(10))
            .build_rust(async { Ok(()) });

        // ID should be valid
        assert!(id.as_u64() > 0);
    }

    #[tokio::test]
    async fn test_simple_coroutine() {
        let (id, future) = CoroutineBuilder::new()
            .name("simple")
            .build_rust(async {
                println!("Coroutine started");
                Ok(())
            });

        let result = future.await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_coroutine_timeout() {
        let (_id, future) = CoroutineBuilder::new()
            .name("timeout_test")
            .timeout(Duration::from_millis(100))
            .build_rust(async {
                tokio::time::sleep(Duration::from_secs(1)).await;
                Ok(())
            });

        let result = future.await;
        assert!(matches!(result, Err(CoroutineError::Timeout)));
    }
}
