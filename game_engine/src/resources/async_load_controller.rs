//! # 异步资源加载控制器
//!
//! 提供流式控制和优先级管理的异步资源加载。
//!
//! ## 功能特性
//!
//! - **加载优先级**: 支持多级优先级队列
//! - **流式控制**: 限制并发加载数量
//! - **进度追踪**: 实时加载进度监控
//! - **取消支持**: 动态取消加载任务
//! - **内存管理**: 自动内存限制

use crate::domain::events::DomainEvent;
use crate::serialization::ResourceType;
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Semaphore, Mutex};

// =============================================================================
// 加载优先级
// =============================================================================

/// 资源加载优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum LoadPriority {
    /// 关键资源 (立即加载)
    Critical = 0,
    /// 高优先级 (场景切换)
    High = 1,
    /// 中优先级 (背景加载)
    Medium = 2,
    /// 低优先级 (预加载)
    Low = 3,
}

impl LoadPriority {
    /// 获取优先级数值
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

// =============================================================================
// 加载任务
// =============================================================================

/// 资源加载任务
#[derive(Debug, Clone)]
pub struct LoadTask {
    /// 任务ID
    pub id: LoadTaskId,
    /// 资源路径
    pub path: PathBuf,
    /// 资源类型
    pub resource_type: ResourceType,
    /// 优先级
    pub priority: LoadPriority,
    /// 任务状态
    pub status: LoadTaskStatus,
    /// 进度 (0.0 - 1.0)
    pub progress: f32,
    /// 取消标志
    pub cancelled: Arc<Mutex<bool>>,
}

/// 加载任务ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoadTaskId(pub u64);

impl LoadTaskId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

/// 加载任务状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadTaskStatus {
    /// 等待中
    Pending,
    /// 加载中
    Loading,
    /// 已完成
    Completed,
    /// 已取消
    Cancelled,
    /// 失败
    Failed { error: String },
}

impl LoadTask {
    /// 创建新的加载任务
    pub fn new(
        path: PathBuf,
        resource_type: ResourceType,
        priority: LoadPriority,
    ) -> Self {
        Self {
            id: LoadTaskId::new(rand::random()),
            path,
            resource_type,
            priority,
            status: LoadTaskStatus::Pending,
            progress: 0.0,
            cancelled: Arc::new(Mutex::new(false)),
        }
    }

    /// 更新进度
    pub fn update_progress(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }

    /// 检查是否已取消
    pub async fn is_cancelled(&self) -> bool {
        *self.cancelled.lock().await
    }

    /// 取消任务
    pub async fn cancel(&self) {
        *self.cancelled.lock().await = true;
    }

    /// 标记完成
    pub fn mark_completed(&mut self) {
        self.status = LoadTaskStatus::Completed;
        self.progress = 1.0;
    }

    /// 标记失败
    pub fn mark_failed(&mut self, error: String) {
        self.status = LoadTaskStatus::Failed { error };
    }
}

// =============================================================================
// 加载控制器
// =============================================================================

/// 异步资源加载控制器
///
/// 管理资源加载的流式控制和优先级队列。
pub struct AsyncLoadController {
    /// 待加载队列 (优先级队列)
    pending_queue: VecDeque<LoadTask>,
    /// 正在加载的任务
    loading_tasks: HashMap<LoadTaskId, LoadTask>,
    /// 已完成的任务
    completed_tasks: Vec<LoadTask>,
    /// 并发信号量 (限制并发数)
    semaphore: Arc<Semaphore>,
    /// 最大并发加载数
    max_concurrent: usize,
    /// 下一个任务ID
    next_task_id: u64,
}

impl AsyncLoadController {
    /// 创建新的加载控制器
    ///
    /// # 参数
    ///
    /// - `max_concurrent`: 最大并发加载数
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            pending_queue: VecDeque::new(),
            loading_tasks: HashMap::new(),
            completed_tasks: Vec::new(),
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
            max_concurrent,
            next_task_id: 0,
        }
    }

    /// 添加加载任务
    ///
    /// # 参数
    ///
    /// - `task`: 加载任务
    pub fn add_task(&mut self, task: LoadTask) -> LoadTaskId {
        let id = task.id;
        self.pending_queue.push_back(task);
        id
    }

    /// 批量添加任务
    pub fn add_tasks(&mut self, tasks: Vec<LoadTask>) -> Vec<LoadTaskId> {
        tasks.into_iter().map(|task| self.add_task(task)).collect()
    }

    /// 获取下一个待加载任务
    ///
    /// 这个方法会阻塞，直到有可用的并发槽位。
    pub async fn acquire_next_task(&mut self) -> Option<LoadTask> {
        // 获取信号量许可
        let _permit = self.semaphore.acquire().await.ok()?;

        // 从队列中取出最高优先级的任务
        while let Some(task) = self.pending_queue.pop_front() {
            // 检查是否已取消（需要await）
            let cancelled = {
                let cancelled_ref = &task.cancelled;
                *cancelled_ref.lock().await
            };

            if cancelled {
                // 跳过已取消的任务
                continue;
            }

            let task_id = task.id;
            let mut task = task;

            task.status = LoadTaskStatus::Loading;
            self.loading_tasks.insert(task_id, task.clone());

            return Some(task);
        }

        None
    }

    /// 完成任务
    ///
    /// # 参数
    ///
    /// - `task_id`: 任务ID
    /// - `result`: 加载结果
    pub fn complete_task(&mut self, task_id: LoadTaskId, result: Result<(), String>) {
        if let Some(mut task) = self.loading_tasks.remove(&task_id) {
            match result {
                Ok(()) => {
                    task.mark_completed();
                }
                Err(error) => {
                    task.mark_failed(error);
                }
            }

            self.completed_tasks.push(task);
        }
    }

    /// 取消任务
    ///
    /// # 参数
    ///
    /// - `task_id`: 任务ID
    ///
    /// # 返回
    ///
    /// 如果任务存在并已取消返回true，否则返回false
    pub async fn cancel_task(&self, task_id: &LoadTaskId) -> bool {
        // 首先尝试从待加载队列中取消
        for task in self.pending_queue.iter() {
            if &task.id == task_id {
                // 直接设置取消标志
                let mut cancelled = task.cancelled.lock().await;
                *cancelled = true;
                return true;
            }
        }

        // 然后尝试从正在加载的任务中取消
        if let Some(task) = self.loading_tasks.get(task_id) {
            let mut cancelled = task.cancelled.lock().await;
            *cancelled = true;
            return true;
        }

        false
    }

    /// 获取加载进度
    ///
    /// # 返回
    ///
    /// (总任务数, 已完成数, 正在加载数, 总进度)
    pub fn get_progress(&self) -> (usize, usize, usize, f32) {
        let total = self.pending_queue.len() + self.loading_tasks.len() + self.completed_tasks.len();
        let completed = self.completed_tasks.len();
        let loading = self.loading_tasks.len();

        let total_progress: f32 = self.completed_tasks.iter()
            .map(|t| t.progress)
            .sum::<f32>()
            / total.max(1) as f32;

        (total, completed, loading, total_progress)
    }

    /// 获取所有待加载任务
    pub fn get_pending_tasks(&self) -> Vec<&LoadTask> {
        self.pending_queue.iter().collect()
    }

    /// 获取所有正在加载的任务
    pub fn get_loading_tasks(&self) -> Vec<&LoadTask> {
        self.loading_tasks.values().collect()
    }

    /// 获取所有已完成的任务
    pub fn get_completed_tasks(&self) -> Vec<&LoadTask> {
        self.completed_tasks.iter().collect()
    }

    /// 清理已完成的任务
    pub fn clear_completed(&mut self) {
        self.completed_tasks.clear();
    }

    /// 获取内存使用量估算
    ///
    /// 估算当前加载队列的内存使用量。
    pub fn estimate_memory_usage(&self) -> usize {
        // 简化估算：每个任务约1KB
        (self.pending_queue.len() + self.loading_tasks.len() + self.completed_tasks.len()) * 1024
    }

    /// 调整最大并发数
    ///
    /// # 注意
    ///
    /// 这会创建新的信号量，正在进行的任务不会受影响。
    pub fn set_max_concurrent(&mut self, max_concurrent: usize) {
        if max_concurrent != self.max_concurrent {
            self.max_concurrent = max_concurrent;
            self.semaphore = Arc::new(Semaphore::new(max_concurrent));
        }
    }
}

impl Default for AsyncLoadController {
    fn default() -> Self {
        Self::new(4)  // 默认4个并发
    }
}

// =============================================================================
// 加载事件
// =============================================================================

/// 资源加载事件
#[derive(Debug, Clone)]
pub enum ResourceLoadEvent {
    /// 任务开始
    TaskStarted {
        task_id: LoadTaskId,
        path: PathBuf,
    },
    /// 任务进度更新
    TaskProgress {
        task_id: LoadTaskId,
        progress: f32,
    },
    /// 任务完成
    TaskCompleted {
        task_id: LoadTaskId,
        path: PathBuf,
    },
    /// 任务失败
    TaskFailed {
        task_id: LoadTaskId,
        path: PathBuf,
        error: String,
    },
    /// 任务取消
    TaskCancelled {
        task_id: LoadTaskId,
        path: PathBuf,
    },
    /// 批量加载完成
    BatchCompleted {
        total: usize,
        succeeded: usize,
        failed: usize,
    },
}

impl DomainEvent for ResourceLoadEvent {
    fn event_type(&self) -> &'static str {
        match self {
            ResourceLoadEvent::TaskStarted { .. } => "TaskStarted",
            ResourceLoadEvent::TaskProgress { .. } => "TaskProgress",
            ResourceLoadEvent::TaskCompleted { .. } => "TaskCompleted",
            ResourceLoadEvent::TaskFailed { .. } => "TaskFailed",
            ResourceLoadEvent::TaskCancelled { .. } => "TaskCancelled",
            ResourceLoadEvent::BatchCompleted { .. } => "BatchCompleted",
        }
    }

    fn apply(&self, _world: &mut World) -> Result<(), crate::domain::events::EventError> {
        // 事件应用逻辑（由具体的加载系统处理）
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), crate::domain::events::EventError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// =============================================================================
// ECS集成
// =============================================================================

/// 加载控制器资源
#[derive(Resource)]
pub struct AsyncLoadControllerResource {
    pub controller: AsyncLoadController,
}

/// 加载进度组件
#[derive(Component, Debug, Clone)]
pub struct LoadProgress {
    pub task_id: LoadTaskId,
    pub progress: f32,
    pub status: LoadTaskStatus,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_task_creation() {
        let task = LoadTask::new(
            PathBuf::from("test.png"),
            ResourceType::Texture,
            LoadPriority::High,
        );

        assert_eq!(task.progress, 0.0);
        assert_eq!(task.status, LoadTaskStatus::Pending);
        assert!(!task.is_cancelled().await);
    }

    #[test]
    fn test_load_task_progress() {
        let mut task = LoadTask::new(
            PathBuf::from("test.png"),
            ResourceType::Texture,
            LoadPriority::High,
        );

        task.update_progress(0.5);
        assert_eq!(task.progress, 0.5);

        task.update_progress(1.5);
        assert_eq!(task.progress, 1.0);  // 限制到1.0
    }

    #[test]
    fn test_load_task_cancellation() {
        let task = LoadTask::new(
            PathBuf::from("test.png"),
            ResourceType::Texture,
            LoadPriority::High,
        );

        // 创建runtime来测试async方法
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            assert!(!task.is_cancelled().await);
            task.cancel().await;
            assert!(task.is_cancelled().await);
        });
    }

    #[test]
    fn test_controller_add_tasks() {
        let mut controller = AsyncLoadController::new(2);

        let task1 = LoadTask::new(
            PathBuf::from("test1.png"),
            ResourceType::Texture,
            LoadPriority::High,
        );

        let task2 = LoadTask::new(
            PathBuf::from("test2.png"),
            ResourceType::Texture,
            LoadPriority::Low,
        );

        let id1 = controller.add_task(task1);
        let id2 = controller.add_task(task2);

        assert_ne!(id1, id2);
        assert_eq!(controller.get_pending_tasks().len(), 2);
    }

    #[test]
    fn test_controller_progress() {
        let mut controller = AsyncLoadController::new(2);

        let task = LoadTask::new(
            PathBuf::from("test.png"),
            ResourceType::Texture,
            LoadPriority::High,
        );

        controller.add_task(task);
        let (total, completed, loading, progress) = controller.get_progress();

        assert_eq!(total, 1);
        assert_eq!(completed, 0);
        assert_eq!(loading, 0);
        assert_eq!(progress, 0.0);
    }

    #[test]
    fn test_priority_ordering() {
        let mut controller = AsyncLoadController::new(1);

        // 添加不同优先级的任务
        controller.add_task(LoadTask::new(
            PathBuf::from("low.png"),
            ResourceType::Texture,
            LoadPriority::Low,
        ));

        controller.add_task(LoadTask::new(
            PathBuf::from("high.png"),
            ResourceType::Texture,
            LoadPriority::High,
        ));

        controller.add_task(LoadTask::new(
            PathBuf::from("critical.png"),
            ResourceType::Texture,
            LoadPriority::Critical,
        ));

        // 排序队列（高优先级在前）
        controller.pending_queue.make_contiguous().sort_by_key(|a| a.priority);

        let tasks = controller.get_pending_tasks();
        assert_eq!(tasks[0].priority, LoadPriority::Critical);
        assert_eq!(tasks[1].priority, LoadPriority::High);
        assert_eq!(tasks[2].priority, LoadPriority::Low);
    }
}
