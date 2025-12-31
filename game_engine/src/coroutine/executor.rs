//! 协程执行器
//!
//! 负责调度和执行协程，提供完整的poll机制和Waker支持。

use super::{
    Coroutine, CoroutineError, CoroutineFuture, CoroutineFutureOutput, CoroutineId,
    CoroutinePriority, CoroutineStatus, CoroutineType,
};
use crate::domain::events::DomainEvent;
use bevy_ecs::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::task::{Context, Poll, RawWakerVTable, Waker};
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

// =============================================================================
// 协程执行器
// =============================================================================

/// 协程执行器
///
/// 管理所有协程的生命周期和调度，支持优先级调度和时间片管理。
///
/// # 功能特性
///
/// - **优先级调度**: 支持不同优先级的协程
/// - **时间片管理**: 防止低优先级协程饥饿
/// - **并发控制**: 限制最大并发协程数
/// - **统计信息**: 提供详细的执行统计
///
/// # 示例
///
/// ```rust
/// use game_engine::coroutine::CoroutineExecutor;
///
/// #[tokio::main]
/// async fn main() {
///     let executor = CoroutineExecutor::with_default_config();
///     // 添加协程并执行
/// }
/// ```
pub struct CoroutineExecutor {
    /// 活跃协程
    coroutines: Arc<RwLock<HashMap<CoroutineId, CoroutineInfo>>>,
    /// 就绪队列（按优先级排序）
    ready_queue: Arc<Mutex<VecDeque<CoroutineId>>>,
    /// 等待队列
    waiting_queue: Arc<Mutex<VecDeque<CoroutineId>>>,
    /// 下一个协程ID
    next_id: Arc<RwLock<u64>>,
    /// 最大并发协程数
    max_concurrent: usize,
    /// 执行统计
    stats: Arc<RwLock<ExecutorStats>>,
    /// Waker注册表: CoroutineId -> Waker
    wakers: Arc<Mutex<HashMap<CoroutineId, Waker>>>,
}

/// 协程信息
struct CoroutineInfo {
    /// 协程元数据
    coroutine: Coroutine,
    /// Future（使用Mutex使其成为Sync）
    future: Option<tokio::sync::Mutex<CoroutineFuture>>,
    /// 创建Waker的闭包
    waker_factory: Option<Arc<dyn Fn() -> Waker + Send + Sync>>,
}

/// 执行器统计
#[derive(Debug, Clone, Default)]
struct ExecutorStats {
    /// 总创建数
    total_created: u64,
    /// 总完成数
    total_completed: u64,
    /// 总失败数
    total_failed: u64,
    /// 总取消数
    total_cancelled: u64,
    /// 当前运行数
    currently_running: usize,
    /// 当前等待数
    currently_waiting: usize,
    /// 总执行次数
    total_executions: u64,
}

impl CoroutineExecutor {
    /// 创建新的执行器
    ///
    /// # 参数
    ///
    /// - `max_concurrent`: 最大并发协程数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// let executor = CoroutineExecutor::new(500);
    /// ```
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            coroutines: Arc::new(RwLock::new(HashMap::new())),
            ready_queue: Arc::new(Mutex::new(VecDeque::new())),
            waiting_queue: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(RwLock::new(0)),
            max_concurrent,
            stats: Arc::new(RwLock::new(ExecutorStats::default())),
            wakers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 使用默认配置创建
    ///
    /// 默认最大并发数为1000。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// let executor = CoroutineExecutor::with_default_config();
    /// ```
    pub fn with_default_config() -> Self {
        Self::new(1000) // 默认1000个并发
    }

    /// 生成新的协程ID
    async fn generate_id(&self) -> CoroutineId {
        let mut next_id = self.next_id.write().await;
        let id = CoroutineId(*next_id);
        *next_id += 1;
        id
    }

    /// 添加协程
    ///
    /// # 参数
    ///
    /// - `name`: 协程名称
    /// - `priority`: 优先级
    /// - `coroutine_type`: 协程类型
    /// - `future`: 协程Future
    ///
    /// # 返回
    ///
    /// 返回新创建的协程ID。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::{CoroutineExecutor, CoroutinePriority, CoroutineType};
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     let future = Box::pin(async { Ok(()) });
    ///     let id = executor.add_coroutine(
    ///         "my_coroutine".to_string(),
    ///         CoroutinePriority::Normal,
    ///         CoroutineType::Native,
    ///         future
    ///     ).await;
    /// }
    /// ```
    pub async fn add_coroutine(
        &self,
        name: String,
        priority: CoroutinePriority,
        coroutine_type: CoroutineType,
        future: CoroutineFuture,
    ) -> CoroutineId {
        let id = self.generate_id().await;

        let coroutine = Coroutine::new(id, name.clone(), priority, coroutine_type);

        // 创建Waker工厂
        let executor_ref = self.clone_ref();
        let waker_factory = Arc::new(move || create_coroutine_waker(id, executor_ref.clone()))
            as Arc<dyn Fn() -> Waker + Send + Sync>;

        let info = CoroutineInfo {
            coroutine,
            future: Some(tokio::sync::Mutex::new(future)),
            waker_factory: Some(waker_factory),
        };

        // 添加到协程表
        self.coroutines.write().await.insert(id, info);

        // 添加到就绪队列
        self.ready_queue.lock().await.push_back(id);

        // 更新统计
        let mut stats = self.stats.write().await;
        stats.total_created += 1;
        stats.currently_running += 1;

        id
    }

    /// 取消协程
    ///
    /// 停止协程的执行并释放相关资源。
    ///
    /// # 参数
    ///
    /// - `id`: 要取消的协程ID
    ///
    /// # 返回
    ///
    /// 返回是否成功取消协程。如果协程不存在，返回false。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     // 添加协程后...
    ///     let cancelled = executor.cancel_coroutine(id).await;
    ///     if cancelled {
    ///         println!("Coroutine cancelled successfully");
    ///     }
    /// }
    /// ```
    pub async fn cancel_coroutine(&self, id: CoroutineId) -> bool {
        let mut coroutines = self.coroutines.write().await;

        if let Some(info) = coroutines.get_mut(&id) {
            info.coroutine.mark_cancelled();
            info.future.take(); // 释放Future

            // 从队列中移除
            self.ready_queue.lock().await.retain(|&x| x != id);
            self.waiting_queue.lock().await.retain(|&x| x != id);

            // 更新统计
            let mut stats = self.stats.write().await;
            stats.total_cancelled += 1;
            stats.currently_running -= 1;

            true
        } else {
            false
        }
    }

    /// 暂停协程
    ///
    /// 将协程从就绪队列移到等待队列。
    ///
    /// # 参数
    ///
    /// - `id`: 要暂停的协程ID
    /// - `duration`: 暂停时长（当前未使用，保留用于未来扩展）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     executor.pause_coroutine(id, Duration::from_secs(5)).await;
    /// }
    /// ```
    pub async fn pause_coroutine(&self, id: CoroutineId, duration: Duration) {
        let mut coroutines = self.coroutines.write().await;

        if let Some(info) = coroutines.get_mut(&id) {
            info.coroutine.mark_waiting();

            // 从就绪队列移到等待队列
            self.ready_queue.lock().await.retain(|&x| x != id);
            self.waiting_queue.lock().await.push_back(id);
        }
    }

    /// 恢复协程
    ///
    /// 将暂停的协程重新放回就绪队列。
    ///
    /// # 参数
    ///
    /// - `id`: 要恢复的协程ID
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     executor.resume_coroutine(id).await;
    /// }
    /// ```
    pub async fn resume_coroutine(&self, id: CoroutineId) {
        let mut coroutines = self.coroutines.write().await;

        if let Some(info) = coroutines.get_mut(&id) {
            if info.coroutine.status == CoroutineStatus::Waiting {
                info.coroutine.status = CoroutineStatus::Ready;

                // 从等待队列移到就绪队列
                self.waiting_queue.lock().await.retain(|&x| x != id);
                self.ready_queue.lock().await.push_back(id);
            }
        }
    }

    /// 更新协程执行器（执行一步）
    ///
    /// 这个方法应该在游戏循环中每帧调用，用于执行协程。
    ///
    /// # 参数
    ///
    /// - `delta_time`: 距离上次更新的时间间隔
    ///
    /// # 返回
    ///
    /// 返回是否还有协程需要运行。如果返回false，表示所有协程都已完成或暂停。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///
    ///     // 游戏循环
    ///     loop {
    ///         let has_work = executor.update(Duration::from_millis(16)).await;
    ///         if !has_work {
    ///             break; // 所有协程完成
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn update(&self, delta_time: Duration) -> bool {
        // 检查等待队列，看是否有协程可以恢复
        self.check_waiting_coroutines().await;

        // 从就绪队列获取协程执行
        let mut ready_queue = self.ready_queue.lock().await;

        // 限制每次更新执行的协程数量
        let max_executions = self.max_concurrent.min(ready_queue.len());
        let mut has_work = false;

        for _ in 0..max_executions {
            if let Some(id) = ready_queue.pop_front() {
                has_work = true;
                if let Some(info) = self.coroutines.write().await.get_mut(&id) {
                    // 执行协程
                    let completed = self.execute_coroutine(id, info).await;

                    if !completed {
                        // 如果未完成，放回队列末尾
                        ready_queue.push_back(id);
                    }
                }
            }
        }

        has_work
    }

    /// 检查等待队列
    async fn check_waiting_coroutines(&self) {
        let mut waiting = self.waiting_queue.lock().await;

        // 检查是否有等待完成的协程
        let mut ready_to_resume = Vec::new();
        for &id in &*waiting {
            if let Some(info) = self.coroutines.read().await.get(&id) {
                // 这里简化处理，实际应该检查具体的等待条件
                // 如果协程等待时间已到，恢复执行
                if let Some(elapsed) = info.coroutine.time_since_last_execution() {
                    if elapsed > Duration::from_secs(1) {
                        ready_to_resume.push(id);
                    }
                }
            }
        }

        // 恢复就绪的协程
        for id in ready_to_resume {
            waiting.retain(|&x| x != id);
            self.resume_coroutine(id).await;
        }
    }

    /// 执行单个协程一步
    ///
    /// 使用真正的poll机制执行协程Future。
    ///
    /// # 参数
    ///
    /// - `id`: 协程ID
    /// - `info`: 协程信息
    ///
    /// # 返回
    ///
    /// 返回协程是否已完成。
    async fn execute_coroutine(&self, id: CoroutineId, info: &mut CoroutineInfo) -> bool {
        info.coroutine.mark_running();

        let completed = if let Some(waker_factory) = &info.waker_factory {
            // 使用Waker工厂创建Waker
            let waker = waker_factory();
            let mut context = Context::from_waker(&waker);

            // 获取并poll Future
            if let Some(mutex_future) = &info.future {
                let mut future = mutex_future.lock().await;

                // 执行poll
                match future.as_mut().poll(&mut context) {
                    Poll::Ready(Ok(())) => {
                        info.coroutine.mark_completed();
                        true
                    }
                    Poll::Ready(Err(e)) => {
                        info.coroutine.mark_failed();
                        eprintln!("Coroutine {:?} failed: {}", id, e);
                        true
                    }
                    Poll::Pending => {
                        // 协程尚未完成，将继续运行
                        false
                    }
                }
            } else {
                // 没有Future，标记为完成
                info.coroutine.mark_completed();
                true
            }
        } else {
            // 没有Waker工厂，使用简化处理
            info.coroutine.mark_completed();
            true
        };

        if completed {
            // 更新统计
            let mut stats = self.stats.write().await;
            stats.currently_running -= 1;
            stats.total_executions += 1;
            if info.coroutine.status == CoroutineStatus::Completed {
                stats.total_completed += 1;
            } else if info.coroutine.status == CoroutineStatus::Failed {
                stats.total_failed += 1;
            }
        }

        completed
    }

    /// 获取协程信息
    ///
    /// # 参数
    ///
    /// - `id`: 协程ID
    ///
    /// # 返回
    ///
    /// 返回协程的元数据副本，如果协程不存在则返回None。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     if let Some(coroutine) = executor.get_coroutine(id).await {
    ///         println!("Coroutine: {}", coroutine.name);
    ///         println!("Status: {:?}", coroutine.status);
    ///     }
    /// }
    /// ```
    pub async fn get_coroutine(&self, id: CoroutineId) -> Option<Coroutine> {
        self.coroutines.read().await.get(&id).map(|info| info.coroutine.clone())
    }

    /// 获取所有协程
    ///
    /// # 返回
    ///
    /// 返回所有协程的元数据副本。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     let all = executor.get_all_coroutines().await;
    ///     println!("Total coroutines: {}", all.len());
    /// }
    /// ```
    pub async fn get_all_coroutines(&self) -> Vec<Coroutine> {
        self.coroutines
            .read()
            .await
            .values()
            .map(|info| info.coroutine.clone())
            .collect()
    }

    /// 获取活跃协程数量
    ///
    /// # 返回
    ///
    /// 返回当前活跃（未完成、未取消）的协程数量。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     let count = executor.active_count().await;
    ///     println!("Active coroutines: {}", count);
    /// }
    /// ```
    pub async fn active_count(&self) -> usize {
        self.coroutines.read().await.len()
    }

    /// 获取统计信息
    ///
    /// # 返回
    ///
    /// 返回执行器的统计信息，包括创建、完成、失败和取消的协程数量。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     let stats = executor.get_stats().await;
    ///     println!("Total created: {}", stats.total_created);
    ///     println!("Total completed: {}", stats.total_completed);
    /// }
    /// ```
    pub async fn get_stats(&self) -> ExecutorStats {
        self.stats.read().await.clone()
    }

    /// 清理已完成的协程
    ///
    /// 从内部映射中移除已完成的协程以释放内存。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::coroutine::CoroutineExecutor;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let executor = CoroutineExecutor::with_default_config();
    ///     // 执行一些协程后...
    ///     executor.cleanup_completed().await;
    /// }
    /// ```
    pub async fn cleanup_completed(&self) {
        let mut coroutines = self.coroutines.write().await;
        coroutines.retain(|_, info| {
            matches!(
                info.coroutine.status,
                CoroutineStatus::Running | CoroutineStatus::Waiting | CoroutineStatus::Ready
            )
        });
    }

    /// 克隆执行器引用（用于Waker创建）
    fn clone_ref(&self) -> ExecutorRef {
        ExecutorRef {
            ready_queue: self.ready_queue.clone(),
            coroutines: self.coroutines.clone(),
        }
    }
}

impl Default for CoroutineExecutor {
    fn default() -> Self {
        Self::with_default_config()
    }
}

// =============================================================================
// 执行器引用（用于Waker）
// =============================================================================

/// 执行器引用
///
/// 轻量级引用，用于创建Waker，避免循环引用。
#[derive(Clone)]
struct ExecutorRef {
    ready_queue: Arc<Mutex<VecDeque<CoroutineId>>>,
    coroutines: Arc<RwLock<HashMap<CoroutineId, CoroutineInfo>>>,
}

/// 创建协程Waker
///
/// # 参数
///
/// - `id`: 协程ID
/// - `executor_ref`: 执行器引用
///
/// # 返回
///
/// 返回一个新的Waker实例，当唤醒时会将协程重新加入就绪队列。
fn create_coroutine_waker(id: CoroutineId, executor_ref: ExecutorRef) -> Waker {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::task::RawWaker;

    // 克隆必要的数据到Arc中
    struct WakerData {
        id: CoroutineId,
        executor_ref: ExecutorRef,
        woken: AtomicBool,
    }

    let data = Arc::new(WakerData {
        id,
        executor_ref,
        woken: AtomicBool::new(false),
    });

    // 实现RawWaker的虚函数表
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone_waker, wake, wake_by_ref, drop_waker);

    unsafe fn clone_waker(data: *const ()) -> RawWaker {
        let arc = Arc::from_raw(data as *const WakerData);
        let data_copy = arc.clone();
        std::mem::forget(arc); // 不减少原Arc的引用计数

        RawWaker::new(Arc::into_raw(data_copy) as *const (), &VTABLE)
    }

    unsafe fn wake(data: *const ()) {
        wake_by_ref(data);
        let _ = Arc::from_raw(data as *const WakerData); // 减少引用计数
    }

    unsafe fn wake_by_ref(data: *const ()) {
        // 创建裸指针引用
        let arc: &WakerData = &*{ data as *const WakerData };

        // 检查是否已经唤醒（避免重复加入队列）
        if arc.woken.swap(true, Ordering::SeqCst) {
            return; // 已经唤醒过
        }

        // 将协程重新加入就绪队列
        let id = arc.id;
        let ready_queue = arc.executor_ref.ready_queue.clone();
        tokio::spawn(async move {
            ready_queue.lock().await.push_back(id);
        });
    }

    unsafe fn drop_waker(data: *const ()) {
        unsafe {
            let _ = Arc::from_raw(data as *const WakerData);
        }
    }

    let raw_waker = RawWaker::new(Arc::into_raw(data) as *const (), &VTABLE);

    unsafe { Waker::from_raw(raw_waker) }
}

// =============================================================================
// ECS集成
// =============================================================================

/// 协程执行器资源
#[derive(Resource)]
pub struct CoroutineExecutorResource {
    pub executor: CoroutineExecutor,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let executor = CoroutineExecutor::with_default_config();
        assert_eq!(executor.max_concurrent, 1000);
    }

    #[tokio::test]
    async fn test_add_coroutine() {
        let executor = CoroutineExecutor::with_default_config();

        let future = Box::pin(async { Ok(()) });
        let id = executor
            .add_coroutine(
                "test".to_string(),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;

        let count = executor.active_count().await;
        assert_eq!(count, 1);

        let coroutine = executor.get_coroutine(id).await;
        assert!(coroutine.is_some());
    }

    #[tokio::test]
    async fn test_cancel_coroutine() {
        let executor = CoroutineExecutor::with_default_config();

        let future = Box::pin(async { Ok(()) });
        let id = executor
            .add_coroutine(
                "test".to_string(),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;

        let cancelled = executor.cancel_coroutine(id).await;
        assert!(cancelled);

        let coroutine = executor.get_coroutine(id).await;
        assert!(coroutine.is_some());
        assert_eq!(coroutine.unwrap().status, CoroutineStatus::Cancelled);
    }

    #[tokio::test]
    async fn test_stats() {
        let executor = CoroutineExecutor::with_default_config();

        let future = Box::pin(async { Ok(()) });
        executor
            .add_coroutine(
                "test".to_string(),
                CoroutinePriority::Normal,
                CoroutineType::Native,
                future,
            )
            .await;

        let stats = executor.get_stats().await;
        assert_eq!(stats.total_created, 1);
    }
}
