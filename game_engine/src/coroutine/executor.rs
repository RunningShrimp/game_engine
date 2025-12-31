//! 协程执行器
//!
//! 负责调度和执行协程。

use super::{
    Coroutine, CoroutineError, CoroutineFuture, CoroutineFutureOutput, CoroutineId,
    CoroutinePriority, CoroutineStatus, CoroutineType,
};
use crate::domain::events::DomainEvent;
use bevy_ecs::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

// =============================================================================
// 协程执行器
// =============================================================================

/// 协程执行器
///
/// 管理所有协程的生命周期和调度。
pub struct CoroutineExecutor {
    /// 活跃协程
    coroutines: Arc<RwLock<HashMap<CoroutineId, CoroutineInfo>>>,
    /// 就绪队列
    ready_queue: Arc<Mutex<VecDeque<CoroutineId>>>,
    /// 等待队列
    waiting_queue: Arc<Mutex<VecDeque<CoroutineId>>>,
    /// 下一个协程ID
    next_id: Arc<RwLock<u64>>,
    /// 最大并发协程数
    max_concurrent: usize,
    /// 执行统计
    stats: Arc<RwLock<ExecutorStats>>,
}

/// 协程信息
struct CoroutineInfo {
    /// 协程元数据
    coroutine: Coroutine,
    /// Future（使用Mutex使其成为Sync）
    future: Option<tokio::sync::Mutex<CoroutineFuture>>,
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
}

impl CoroutineExecutor {
    /// 创建新的执行器
    ///
    /// # 参数
    ///
    /// - `max_concurrent`: 最大并发协程数
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            coroutines: Arc::new(RwLock::new(HashMap::new())),
            ready_queue: Arc::new(Mutex::new(VecDeque::new())),
            waiting_queue: Arc::new(Mutex::new(VecDeque::new())),
            next_id: Arc::new(RwLock::new(0)),
            max_concurrent,
            stats: Arc::new(RwLock::new(ExecutorStats::default())),
        }
    }

    /// 使用默认配置创建
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
    pub async fn add_coroutine(
        &self,
        name: String,
        priority: CoroutinePriority,
        coroutine_type: CoroutineType,
        future: CoroutineFuture,
    ) -> CoroutineId {
        let id = self.generate_id().await;

        let coroutine = Coroutine::new(id, name.clone(), priority, coroutine_type);

        let info = CoroutineInfo {
            coroutine,
            future: Some(tokio::sync::Mutex::new(future)),
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
    /// # 参数
    ///
    /// - `id`: 协程ID
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
    /// # 参数
    ///
    /// - `id`: 协程ID
    /// - `duration`: 暂停时长
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
    /// # 参数
    ///
    /// - `id`: 协程ID
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

    /// 更新协程（执行一步）
    ///
    /// 返回是否还有协程需要运行
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
    /// 返回是否已完成
    async fn execute_coroutine(&self, id: CoroutineId, info: &mut CoroutineInfo) -> bool {
        info.coroutine.mark_running();

        // 这里简化处理，实际应该使用自定义Executor来poll Future
        // 由于Rust的Future需要Waker，这里仅做框架实现

        // 模拟执行 - 实际需要正确的poll机制
        let completed = if let Some(_mutex_future) = &info.future {
            // TODO: 实现正确的poll机制
            // let mut future = mutex_future.lock().await;
            // let waker = ...; // 创建Waker
            // let mut context = Context::from_waker(&waker);
            // match future.poll(&mut context) {
            //     Poll::Ready(Ok(())) => {
            //         info.coroutine.mark_completed();
            //         true
            //     }
            //     Poll::Ready(Err(e)) => {
            //         info.coroutine.mark_failed();
            //         false
            //     }
            //     Poll::Pending => false,
            // }

            // 简化：假设协程完成
            info.coroutine.mark_completed();
            true
        } else {
            // 没有Future，标记为完成
            info.coroutine.mark_completed();
            true
        };

        if completed {
            // 更新统计
            let mut stats = self.stats.write().await;
            stats.currently_running -= 1;
            if info.coroutine.status == CoroutineStatus::Completed {
                stats.total_completed += 1;
            } else if info.coroutine.status == CoroutineStatus::Failed {
                stats.total_failed += 1;
            }
        }

        completed
    }

    /// 获取协程信息
    pub async fn get_coroutine(&self, id: CoroutineId) -> Option<Coroutine> {
        self.coroutines.read().await.get(&id).map(|info| info.coroutine.clone())
    }

    /// 获取所有协程
    pub async fn get_all_coroutines(&self) -> Vec<Coroutine> {
        self.coroutines
            .read()
            .await
            .values()
            .map(|info| info.coroutine.clone())
            .collect()
    }

    /// 获取活跃协程数量
    pub async fn active_count(&self) -> usize {
        self.coroutines.read().await.len()
    }

    /// 获取统计信息
    pub async fn get_stats(&self) -> ExecutorStats {
        self.stats.read().await.clone()
    }

    /// 清理已完成的协程
    pub async fn cleanup_completed(&self) {
        let mut coroutines = self.coroutines.write().await;
        coroutines.retain(|_, info| {
            matches!(
                info.coroutine.status,
                CoroutineStatus::Running | CoroutineStatus::Waiting | CoroutineStatus::Ready
            )
        });
    }
}

impl Default for CoroutineExecutor {
    fn default() -> Self {
        Self::with_default_config()
    }
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
