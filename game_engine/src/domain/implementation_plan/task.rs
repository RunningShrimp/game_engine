//  任务管理领域对象
// 
//  该模块实现了任务管理的核心业务逻辑，包括任务的创建、更新、
//  状态管理和依赖关系处理。

use crate::domain::implementation_plan::errors::{ImplementationPlanError, TaskError};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 任务唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub u64);

impl TaskId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Task({})", self.0)
    }
}

/// 任务状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// 待办
    Todo,
    /// 进行中
    InProgress,
    /// 已完成
    Done,
    /// 已取消
    Cancelled,
}

impl std::fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskStatus::Todo => write!(f, "Todo"),
            TaskStatus::InProgress => write!(f, "InProgress"),
            TaskStatus::Done => write!(f, "Done"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
        }
    }
}

/// 任务优先级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    /// 低优先级
    Low,
    /// 中优先级
    Medium,
    /// 高优先级
    High,
}

impl std::fmt::Display for TaskPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskPriority::Low => write!(f, "Low"),
            TaskPriority::Medium => write!(f, "Medium"),
            TaskPriority::High => write!(f, "High"),
        }
    }
}

/// 任务 - 聚合根
///
/// 封装任务的所有属性和行为，确保业务规则在边界内执行。
///
/// ## 聚合边界
///
/// **包含**：
/// - `TaskId`：任务唯一标识符
/// - `name`：任务名称
/// - `description`：任务描述（可选）
/// - `status`：任务状态
/// - `priority`：任务优先级
/// - `assignee`：负责人（可选）
/// - `due_date`：截止日期（可选）
/// - `dependencies`：依赖任务ID集合
/// - `created_at`：创建时间戳
/// - `updated_at`：最后更新时间戳
///
/// **不包含**：
/// - 任务执行历史（基础设施层）
/// - 任务通知（基础设施层）
///
/// ## 业务规则
///
/// 1. 任务ID创建后不可变
/// 2. 已取消的任务不能修改状态
/// 3. 已完成的任务不能回到待办状态
/// 4. 任务不能依赖自身
/// 5. 任务名称不能为空
///
/// ## 不变性约束
///
/// - `TaskId`：创建后不可变
/// - `created_at`：创建后不可变
/// - `status`：只能通过聚合根方法修改（`start`, `complete`, `cancel`）
#[derive(Debug, Clone)]
pub struct Task {
    /// 任务ID
    pub id: TaskId,
    /// 任务名称
    pub name: String,
    /// 任务描述
    pub description: Option<String>,
    /// 任务状态
    pub status: TaskStatus,
    /// 任务优先级
    pub priority: TaskPriority,
    /// 负责人
    pub assignee: Option<String>,
    /// 截止日期时间戳（可选）
    pub due_date: Option<u64>,
    /// 依赖任务ID集合
    pub dependencies: HashSet<TaskId>,
    /// 创建时间戳
    pub created_at: u64,
    /// 最后更新时间戳
    pub updated_at: u64,
}

impl Task {
    /// 创建新任务
    pub fn new(id: TaskId, name: impl Into<String>) -> Result<Self, ImplementationPlanError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ImplementationPlanError::Task(TaskError::InvalidParameter(
                "Task name cannot be empty".to_string(),
            )));
        }

        let now = Self::current_timestamp();
        Ok(Self {
            id,
            name,
            description: None,
            status: TaskStatus::Todo,
            priority: TaskPriority::Medium,
            assignee: None,
            due_date: None,
            dependencies: HashSet::new(),
            created_at: now,
            updated_at: now,
        })
    }

    /// 设置任务描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self.updated_at = Self::current_timestamp();
        self
    }

    /// 设置任务优先级
    pub fn with_priority(mut self, priority: TaskPriority) -> Self {
        self.priority = priority;
        self.updated_at = Self::current_timestamp();
        self
    }

    /// 设置负责人
    pub fn assign_to(mut self, assignee: impl Into<String>) -> Self {
        self.assignee = Some(assignee.into());
        self.updated_at = Self::current_timestamp();
        self
    }

    /// 设置截止日期
    pub fn with_due_date(mut self, due_date: u64) -> Self {
        self.due_date = Some(due_date);
        self.updated_at = Self::current_timestamp();
        self
    }

    /// 添加依赖任务
    pub fn add_dependency(&mut self, dependency_id: TaskId) -> Result<(), ImplementationPlanError> {
        if dependency_id == self.id {
            return Err(ImplementationPlanError::Task(TaskError::InvalidParameter(
                "Task cannot depend on itself".to_string(),
            )));
        }

        self.dependencies.insert(dependency_id);
        self.updated_at = Self::current_timestamp();
        Ok(())
    }

    /// 移除依赖任务
    pub fn remove_dependency(
        &mut self,
        dependency_id: &TaskId,
    ) -> Result<(), ImplementationPlanError> {
        if self.dependencies.remove(dependency_id) {
            self.updated_at = Self::current_timestamp();
            Ok(())
        } else {
            Err(ImplementationPlanError::Task(TaskError::InvalidParameter(
                format!("Dependency {} not found", dependency_id),
            )))
        }
    }

    /// 开始任务
    pub fn start(&mut self) -> Result<(), ImplementationPlanError> {
        match self.status {
            TaskStatus::Todo => {
                self.status = TaskStatus::InProgress;
                self.updated_at = Self::current_timestamp();
                Ok(())
            }
            TaskStatus::Cancelled => Err(ImplementationPlanError::Task(
                TaskError::InvalidStatusTransition {
                    from: TaskStatus::Cancelled.to_string(),
                    to: TaskStatus::InProgress.to_string(),
                },
            )),
            TaskStatus::InProgress => Ok(()), // 已经是进行中状态
            TaskStatus::Done => Err(ImplementationPlanError::Task(
                TaskError::InvalidStatusTransition {
                    from: TaskStatus::Done.to_string(),
                    to: TaskStatus::InProgress.to_string(),
                },
            )),
        }
    }

    /// 完成任务
    pub fn complete(&mut self) -> Result<(), ImplementationPlanError> {
        match self.status {
            TaskStatus::Todo | TaskStatus::InProgress => {
                self.status = TaskStatus::Done;
                self.updated_at = Self::current_timestamp();
                Ok(())
            }
            TaskStatus::Cancelled => Err(ImplementationPlanError::Task(
                TaskError::InvalidStatusTransition {
                    from: TaskStatus::Cancelled.to_string(),
                    to: TaskStatus::Done.to_string(),
                },
            )),
            TaskStatus::Done => Ok(()), // 已经是完成状态
        }
    }

    /// 取消任务
    pub fn cancel(&mut self) -> Result<(), ImplementationPlanError> {
        if self.status == TaskStatus::Done {
            return Err(ImplementationPlanError::Task(
                TaskError::InvalidStatusTransition {
                    from: TaskStatus::Done.to_string(),
                    to: TaskStatus::Cancelled.to_string(),
                },
            ));
        }

        self.status = TaskStatus::Cancelled;
        self.updated_at = Self::current_timestamp();
        Ok(())
    }

    /// 检查任务是否可以开始（所有依赖任务已完成）
    pub fn can_start(&self, completed_tasks: &HashSet<TaskId>) -> bool {
        self.dependencies
            .iter()
            .all(|dep| completed_tasks.contains(dep))
    }

    /// 检查任务是否过期
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            Self::current_timestamp() > due_date && self.status != TaskStatus::Done
        } else {
            false
        }
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 任务管理器
pub struct TaskManager {
    tasks: std::collections::HashMap<TaskId, Task>,
    next_id: u64,
}

impl TaskManager {
    /// 创建新的任务管理器
    pub fn new() -> Self {
        Self {
            tasks: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    /// 创建任务
    pub fn create_task(
        &mut self,
        name: impl Into<String>,
    ) -> Result<TaskId, ImplementationPlanError> {
        let id = TaskId::new(self.next_id);
        self.next_id += 1;

        let task = Task::new(id, name)?;
        self.tasks.insert(id, task);
        Ok(id)
    }

    /// 获取任务
    pub fn get_task(&self, id: &TaskId) -> Result<&Task, ImplementationPlanError> {
        self.tasks.get(id).ok_or_else(|| {
            ImplementationPlanError::Task(TaskError::TaskNotFound(format!("{}", id)))
        })
    }

    /// 获取任务的可变引用
    pub fn get_task_mut(&mut self, id: &TaskId) -> Result<&mut Task, ImplementationPlanError> {
        self.tasks.get_mut(id).ok_or_else(|| {
            ImplementationPlanError::Task(TaskError::TaskNotFound(format!("{}", id)))
        })
    }

    /// 删除任务
    pub fn delete_task(&mut self, id: &TaskId) -> Result<(), ImplementationPlanError> {
        if self.tasks.remove(id).is_some() {
            // 移除其他任务对该任务的依赖
            for task in self.tasks.values_mut() {
                task.dependencies.remove(id);
            }
            Ok(())
        } else {
            Err(ImplementationPlanError::Task(TaskError::TaskNotFound(
                format!("{}", id),
            )))
        }
    }

    /// 获取所有任务
    pub fn get_all_tasks(&self) -> Vec<&Task> {
        self.tasks.values().collect()
    }

    /// 获取按状态过滤的任务
    pub fn get_tasks_by_status(&self, status: TaskStatus) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| task.status == status)
            .collect()
    }

    /// 获取可开始的任务（依赖已满足）
    pub fn get_startable_tasks(&self) -> Vec<&Task> {
        let completed_tasks: HashSet<TaskId> = self
            .tasks
            .values()
            .filter(|task| task.status == TaskStatus::Done)
            .map(|task| task.id)
            .collect();

        self.tasks
            .values()
            .filter(|task| task.status == TaskStatus::Todo && task.can_start(&completed_tasks))
            .collect()
    }

    /// 检查是否存在依赖循环
    pub fn has_dependency_cycle(&self) -> bool {
        // 简化的循环检测实现
        // 在实际应用中，可能需要更复杂的图算法
        for task in self.tasks.values() {
            if self.has_cycle_from(task.id, &mut HashSet::new()) {
                return true;
            }
        }
        false
    }

    fn has_cycle_from(&self, task_id: TaskId, visited: &mut HashSet<TaskId>) -> bool {
        if visited.contains(&task_id) {
            return true;
        }

        visited.insert(task_id);

        if let Some(task) = self.tasks.get(&task_id) {
            for dep in &task.dependencies {
                if self.has_cycle_from(*dep, visited) {
                    return true;
                }
            }
        }

        visited.remove(&task_id);
        false
    }
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(TaskId(1), "Test Task").unwrap();
        assert_eq!(task.id, TaskId(1));
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.status, TaskStatus::Todo);
        assert_eq!(task.priority, TaskPriority::Medium);
    }

    #[test]
    fn test_task_creation_empty_name() {
        let result = Task::new(TaskId(1), "");
        assert!(matches!(
            result,
            Err(ImplementationPlanError::Task(TaskError::InvalidParameter(
                _
            )))
        ));
    }

    #[test]
    fn test_task_with_description() {
        let task = Task::new(TaskId(1), "Test Task")
            .unwrap()
            .with_description("Test description");
        assert_eq!(task.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_task_with_priority() {
        let task = Task::new(TaskId(1), "Test Task")
            .unwrap()
            .with_priority(TaskPriority::High);
        assert_eq!(task.priority, TaskPriority::High);
    }

    #[test]
    fn test_task_assign_to() {
        let task = Task::new(TaskId(1), "Test Task")
            .unwrap()
            .assign_to("John Doe");
        assert_eq!(task.assignee, Some("John Doe".to_string()));
    }

    #[test]
    fn test_task_with_due_date() {
        let task = Task::new(TaskId(1), "Test Task")
            .unwrap()
            .with_due_date(1234567890);
        assert_eq!(task.due_date, Some(1234567890));
    }

    #[test]
    fn test_task_add_dependency() {
        let mut task = Task::new(TaskId(1), "Test Task").unwrap();
        let dep_id = TaskId(2);

        task.add_dependency(dep_id).unwrap();
        assert!(task.dependencies.contains(&dep_id));
    }

    #[test]
    fn test_task_add_self_dependency() {
        let mut task = Task::new(TaskId(1), "Test Task").unwrap();

        let result = task.add_dependency(TaskId(1));
        assert!(matches!(
            result,
            Err(ImplementationPlanError::Task(TaskError::InvalidParameter(
                _
            )))
        ));
    }

    #[test]
    fn test_task_start() {
        let mut task = Task::new(TaskId(1), "Test Task").unwrap();

        task.start().unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);
    }

    #[test]
    fn test_task_complete() {
        let mut task = Task::new(TaskId(1), "Test Task").unwrap();

        task.start().unwrap();
        task.complete().unwrap();
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn test_task_cancel() {
        let mut task = Task::new(TaskId(1), "Test Task").unwrap();

        task.start().unwrap();
        task.cancel().unwrap();
        assert_eq!(task.status, TaskStatus::Cancelled);
    }

    #[test]
    fn test_task_invalid_status_transitions() {
        let mut task = Task::new(TaskId(1), "Test Task").unwrap();

        // 取消任务后不能开始
        task.cancel().unwrap();
        assert!(task.start().is_err());

        // 完成任务后不能取消
        let mut task2 = Task::new(TaskId(2), "Test Task 2").unwrap();
        task2.complete().unwrap();
        assert!(task2.cancel().is_err());
    }

    #[test]
    fn test_task_manager_create_task() {
        let mut manager = TaskManager::new();

        let id = manager.create_task("Test Task").unwrap();
        let task = manager.get_task(&id).unwrap();
        assert_eq!(task.name, "Test Task");
    }

    #[test]
    fn test_task_manager_get_tasks_by_status() {
        let mut manager = TaskManager::new();

        let id1 = manager.create_task("Task 1").unwrap();
        let id2 = manager.create_task("Task 2").unwrap();

        // 验证两个任务都已创建
        assert!(manager.get_task(&id1).is_ok());
        assert!(manager.get_task(&id2).is_ok());

        manager.get_task_mut(&id1).unwrap().start().unwrap();

        let todo_tasks = manager.get_tasks_by_status(TaskStatus::Todo);
        let in_progress_tasks = manager.get_tasks_by_status(TaskStatus::InProgress);

        assert_eq!(todo_tasks.len(), 1);
        assert_eq!(in_progress_tasks.len(), 1);
    }

    #[test]
    fn test_task_manager_get_startable_tasks() {
        let mut manager = TaskManager::new();

        let id1 = manager.create_task("Task 1").unwrap();
        let id2 = manager.create_task("Task 2").unwrap();

        // Task 2 depends on Task 1
        manager
            .get_task_mut(&id2)
            .unwrap()
            .add_dependency(id1)
            .unwrap();

        // Only Task 1 should be startable initially
        let startable = manager.get_startable_tasks();
        assert_eq!(startable.len(), 1);
        assert_eq!(startable[0].id, id1);

        // Complete Task 1
        manager.get_task_mut(&id1).unwrap().complete().unwrap();

        // Now Task 2 should be startable
        let startable = manager.get_startable_tasks();
        assert_eq!(startable.len(), 1);
        assert_eq!(startable[0].id, id2);
    }

    #[test]
    fn test_task_id_display() {
        let id = TaskId::new(42);
        assert_eq!(format!("{}", id), "Task(42)");
    }

    #[test]
    fn test_task_status_display() {
        assert_eq!(format!("{}", TaskStatus::Todo), "Todo");
        assert_eq!(format!("{}", TaskStatus::InProgress), "InProgress");
        assert_eq!(format!("{}", TaskStatus::Done), "Done");
        assert_eq!(format!("{}", TaskStatus::Cancelled), "Cancelled");
    }

    #[test]
    fn test_task_priority_display() {
        assert_eq!(format!("{}", TaskPriority::Low), "Low");
        assert_eq!(format!("{}", TaskPriority::Medium), "Medium");
        assert_eq!(format!("{}", TaskPriority::High), "High");
    }
}
