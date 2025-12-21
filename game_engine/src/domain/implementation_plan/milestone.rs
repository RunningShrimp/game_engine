//  里程碑管理领域对象
// 
//  该模块实现了里程碑管理的核心业务逻辑，包括里程碑的创建、
//  任务关联和进度跟踪。

use crate::domain::implementation_plan::errors::{ImplementationPlanError, MilestoneError};
use crate::domain::implementation_plan::task::{TaskId, TaskStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 里程碑唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MilestoneId(pub u64);

impl MilestoneId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for MilestoneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Milestone({})", self.0)
    }
}

/// 里程碑状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MilestoneStatus {
    /// 未开始
    NotStarted,
    /// 进行中
    InProgress,
    /// 已完成
    Completed,
}

impl std::fmt::Display for MilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MilestoneStatus::NotStarted => write!(f, "NotStarted"),
            MilestoneStatus::InProgress => write!(f, "InProgress"),
            MilestoneStatus::Completed => write!(f, "Completed"),
        }
    }
}

/// 里程碑 - 聚合根
///
/// 封装里程碑的所有属性和行为，确保业务规则在边界内执行。
///
/// ## 聚合边界
///
/// **包含**：
/// - `MilestoneId`：里程碑唯一标识符
/// - `name`：里程碑名称
/// - `description`：里程碑描述（可选）
/// - `status`：里程碑状态
/// - `task_ids`：关联任务ID集合
/// - `due_date`：截止日期（可选）
/// - `created_at`：创建时间戳
/// - `updated_at`：最后更新时间戳
///
/// **不包含**：
/// - 任务详情（其他聚合）
/// - 里程碑历史（基础设施层）
///
/// ## 业务规则
///
/// 1. 里程碑ID创建后不可变
/// 2. 已完成的里程碑不能修改任务列表
/// 3. 里程碑名称不能为空
/// 4. 里程碑进度基于关联任务的完成情况自动计算
///
/// ## 不变性约束
///
/// - `MilestoneId`：创建后不可变
/// - `created_at`：创建后不可变
/// - `status`：只能通过聚合根方法修改（`start`, `complete`）
#[derive(Debug, Clone)]
pub struct Milestone {
    /// 里程碑ID
    pub id: MilestoneId,
    /// 里程碑名称
    pub name: String,
    /// 里程碑描述
    pub description: Option<String>,
    /// 里程碑状态
    pub status: MilestoneStatus,
    /// 关联任务ID集合
    pub task_ids: HashSet<TaskId>,
    /// 截止日期时间戳（可选）
    pub due_date: Option<u64>,
    /// 创建时间戳
    pub created_at: u64,
    /// 最后更新时间戳
    pub updated_at: u64,
}

impl Milestone {
    /// 创建新里程碑
    pub fn new(id: MilestoneId, name: impl Into<String>) -> Result<Self, ImplementationPlanError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ImplementationPlanError::Milestone(
                MilestoneError::InvalidParameter("Milestone name cannot be empty".to_string()),
            ));
        }

        let now = Self::current_timestamp();
        Ok(Self {
            id,
            name,
            description: None,
            status: MilestoneStatus::NotStarted,
            task_ids: HashSet::new(),
            due_date: None,
            created_at: now,
            updated_at: now,
        })
    }

    /// 设置里程碑描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self.updated_at = Self::current_timestamp();
        self
    }

    /// 设置截止日期
    pub fn with_due_date(mut self, due_date: u64) -> Self {
        self.due_date = Some(due_date);
        self.updated_at = Self::current_timestamp();
        self
    }

    /// 添加任务到里程碑
    pub fn add_task(&mut self, task_id: TaskId) -> Result<(), ImplementationPlanError> {
        if self.status == MilestoneStatus::Completed {
            return Err(ImplementationPlanError::Milestone(
                MilestoneError::InvalidParameter(
                    "Cannot add tasks to completed milestone".to_string(),
                ),
            ));
        }

        self.task_ids.insert(task_id);
        self.updated_at = Self::current_timestamp();
        Ok(())
    }

    /// 从里程碑移除任务
    pub fn remove_task(&mut self, task_id: &TaskId) -> Result<(), ImplementationPlanError> {
        if self.status == MilestoneStatus::Completed {
            return Err(ImplementationPlanError::Milestone(
                MilestoneError::InvalidParameter(
                    "Cannot remove tasks from completed milestone".to_string(),
                ),
            ));
        }

        if self.task_ids.remove(task_id) {
            self.updated_at = Self::current_timestamp();
            Ok(())
        } else {
            Err(ImplementationPlanError::Milestone(
                MilestoneError::InvalidParameter(format!(
                    "Task {} not found in milestone",
                    task_id
                )),
            ))
        }
    }

    /// 开始里程碑
    pub fn start(&mut self) -> Result<(), ImplementationPlanError> {
        match self.status {
            MilestoneStatus::NotStarted => {
                self.status = MilestoneStatus::InProgress;
                self.updated_at = Self::current_timestamp();
                Ok(())
            }
            MilestoneStatus::InProgress => Ok(()), // 已经是进行中状态
            MilestoneStatus::Completed => Err(ImplementationPlanError::Milestone(
                MilestoneError::InvalidParameter("Cannot start completed milestone".to_string()),
            )),
        }
    }

    /// 完成里程碑
    pub fn complete(&mut self) -> Result<(), ImplementationPlanError> {
        match self.status {
            MilestoneStatus::NotStarted | MilestoneStatus::InProgress => {
                self.status = MilestoneStatus::Completed;
                self.updated_at = Self::current_timestamp();
                Ok(())
            }
            MilestoneStatus::Completed => Ok(()), // 已经是完成状态
        }
    }

    /// 计算里程碑进度（0.0 到 1.0）
    pub fn calculate_progress(
        &self,
        task_statuses: &std::collections::HashMap<TaskId, TaskStatus>,
    ) -> f32 {
        if self.task_ids.is_empty() {
            return if self.status == MilestoneStatus::Completed {
                1.0
            } else {
                0.0
            };
        }

        let completed_count = self
            .task_ids
            .iter()
            .filter(|task_id| {
                task_statuses
                    .get(task_id)
                    .map(|status| *status == TaskStatus::Done)
                    .unwrap_or(false)
            })
            .count();

        completed_count as f32 / self.task_ids.len() as f32
    }

    /// 检查里程碑是否可以完成（所有任务已完成）
    pub fn can_complete(
        &self,
        task_statuses: &std::collections::HashMap<TaskId, TaskStatus>,
    ) -> bool {
        self.task_ids.iter().all(|task_id| {
            task_statuses
                .get(task_id)
                .map(|status| *status == TaskStatus::Done)
                .unwrap_or(false)
        })
    }

    /// 检查里程碑是否过期
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = self.due_date {
            Self::current_timestamp() > due_date && self.status != MilestoneStatus::Completed
        } else {
            false
        }
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 里程碑管理器
pub struct MilestoneManager {
    milestones: std::collections::HashMap<MilestoneId, Milestone>,
    next_id: u64,
}

impl MilestoneManager {
    /// 创建新的里程碑管理器
    pub fn new() -> Self {
        Self {
            milestones: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    /// 创建里程碑
    pub fn create_milestone(
        &mut self,
        name: impl Into<String>,
    ) -> Result<MilestoneId, ImplementationPlanError> {
        let id = MilestoneId::new(self.next_id);
        self.next_id += 1;

        let milestone = Milestone::new(id, name)?;
        self.milestones.insert(id, milestone);
        Ok(id)
    }

    /// 获取里程碑
    pub fn get_milestone(&self, id: &MilestoneId) -> Result<&Milestone, ImplementationPlanError> {
        self.milestones.get(id).ok_or_else(|| {
            ImplementationPlanError::Milestone(MilestoneError::MilestoneNotFound(format!("{}", id)))
        })
    }

    /// 获取里程碑的可变引用
    pub fn get_milestone_mut(
        &mut self,
        id: &MilestoneId,
    ) -> Result<&mut Milestone, ImplementationPlanError> {
        self.milestones.get_mut(id).ok_or_else(|| {
            ImplementationPlanError::Milestone(MilestoneError::MilestoneNotFound(format!("{}", id)))
        })
    }

    /// 删除里程碑
    pub fn delete_milestone(&mut self, id: &MilestoneId) -> Result<(), ImplementationPlanError> {
        if self.milestones.remove(id).is_some() {
            Ok(())
        } else {
            Err(ImplementationPlanError::Milestone(
                MilestoneError::MilestoneNotFound(format!("{}", id)),
            ))
        }
    }

    /// 获取所有里程碑
    pub fn get_all_milestones(&self) -> Vec<&Milestone> {
        self.milestones.values().collect()
    }

    /// 获取按状态过滤的里程碑
    pub fn get_milestones_by_status(&self, status: MilestoneStatus) -> Vec<&Milestone> {
        self.milestones
            .values()
            .filter(|milestone| milestone.status == status)
            .collect()
    }

    /// 获取可以开始的里程碑（有任务且状态为未开始）
    pub fn get_startable_milestones(&self) -> Vec<&Milestone> {
        self.milestones
            .values()
            .filter(|milestone| {
                milestone.status == MilestoneStatus::NotStarted && !milestone.task_ids.is_empty()
            })
            .collect()
    }

    /// 获取可以完成的里程碑
    pub fn get_completable_milestones(
        &self,
        task_statuses: &std::collections::HashMap<TaskId, TaskStatus>,
    ) -> Vec<&Milestone> {
        self.milestones
            .values()
            .filter(|milestone| {
                milestone.status != MilestoneStatus::Completed
                    && milestone.can_complete(task_statuses)
            })
            .collect()
    }

    /// 计算所有里程碑的总体进度
    pub fn calculate_overall_progress(
        &self,
        task_statuses: &std::collections::HashMap<TaskId, TaskStatus>,
    ) -> f32 {
        if self.milestones.is_empty() {
            return 0.0;
        }

        let total_progress: f32 = self
            .milestones
            .values()
            .map(|milestone| milestone.calculate_progress(task_statuses))
            .sum();

        total_progress / self.milestones.len() as f32
    }
}

impl Default for MilestoneManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::implementation_plan::task::TaskStatus;
    use std::collections::HashMap;

    #[test]
    fn test_milestone_creation() {
        let milestone = Milestone::new(MilestoneId(1), "Test Milestone").unwrap();
        assert_eq!(milestone.id, MilestoneId(1));
        assert_eq!(milestone.name, "Test Milestone");
        assert_eq!(milestone.status, MilestoneStatus::NotStarted);
    }

    #[test]
    fn test_milestone_creation_empty_name() {
        let result = Milestone::new(MilestoneId(1), "");
        assert!(matches!(
            result,
            Err(ImplementationPlanError::Milestone(
                MilestoneError::InvalidParameter(_)
            ))
        ));
    }

    #[test]
    fn test_milestone_with_description() {
        let milestone = Milestone::new(MilestoneId(1), "Test Milestone")
            .unwrap()
            .with_description("Test description");
        assert_eq!(milestone.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_milestone_with_due_date() {
        let milestone = Milestone::new(MilestoneId(1), "Test Milestone")
            .unwrap()
            .with_due_date(1234567890);
        assert_eq!(milestone.due_date, Some(1234567890));
    }

    #[test]
    fn test_milestone_add_task() {
        let mut milestone = Milestone::new(MilestoneId(1), "Test Milestone").unwrap();
        let task_id = TaskId(1);

        milestone.add_task(task_id).unwrap();
        assert!(milestone.task_ids.contains(&task_id));
    }

    #[test]
    fn test_milestone_remove_task() {
        let mut milestone = Milestone::new(MilestoneId(1), "Test Milestone").unwrap();
        let task_id = TaskId(1);

        milestone.add_task(task_id).unwrap();
        milestone.remove_task(&task_id).unwrap();
        assert!(!milestone.task_ids.contains(&task_id));
    }

    #[test]
    fn test_milestone_start() {
        let mut milestone = Milestone::new(MilestoneId(1), "Test Milestone").unwrap();

        milestone.start().unwrap();
        assert_eq!(milestone.status, MilestoneStatus::InProgress);
    }

    #[test]
    fn test_milestone_complete() {
        let mut milestone = Milestone::new(MilestoneId(1), "Test Milestone").unwrap();

        milestone.start().unwrap();
        milestone.complete().unwrap();
        assert_eq!(milestone.status, MilestoneStatus::Completed);
    }

    #[test]
    fn test_milestone_calculate_progress() {
        let milestone = Milestone::new(MilestoneId(1), "Test Milestone").unwrap();
        let mut task_statuses = HashMap::new();

        // 空里程碑
        assert_eq!(milestone.calculate_progress(&task_statuses), 0.0);

        // 添加任务但不设置状态
        let mut milestone_with_tasks = milestone;
        milestone_with_tasks.task_ids.insert(TaskId(1));
        milestone_with_tasks.task_ids.insert(TaskId(2));
        assert_eq!(milestone_with_tasks.calculate_progress(&task_statuses), 0.0);

        // 设置一个任务为完成
        task_statuses.insert(TaskId(1), TaskStatus::Done);
        assert_eq!(milestone_with_tasks.calculate_progress(&task_statuses), 0.5);

        // 设置所有任务为完成
        task_statuses.insert(TaskId(2), TaskStatus::Done);
        assert_eq!(milestone_with_tasks.calculate_progress(&task_statuses), 1.0);
    }

    #[test]
    fn test_milestone_can_complete() {
        let mut milestone = Milestone::new(MilestoneId(1), "Test Milestone").unwrap();
        let mut task_statuses = HashMap::new();

        // 空里程碑可以完成
        assert!(milestone.can_complete(&task_statuses));

        // 添加任务
        milestone.task_ids.insert(TaskId(1));
        milestone.task_ids.insert(TaskId(2));
        assert!(!milestone.can_complete(&task_statuses));

        // 完成一个任务
        task_statuses.insert(TaskId(1), TaskStatus::Done);
        assert!(!milestone.can_complete(&task_statuses));

        // 完成所有任务
        task_statuses.insert(TaskId(2), TaskStatus::Done);
        assert!(milestone.can_complete(&task_statuses));
    }

    #[test]
    fn test_milestone_manager_create_milestone() {
        let mut manager = MilestoneManager::new();

        let id = manager.create_milestone("Test Milestone").unwrap();
        let milestone = manager.get_milestone(&id).unwrap();
        assert_eq!(milestone.name, "Test Milestone");
    }

    #[test]
    fn test_milestone_manager_get_milestones_by_status() {
        let mut manager = MilestoneManager::new();

        let id1 = manager.create_milestone("Milestone 1").unwrap();
        let id2 = manager.create_milestone("Milestone 2").unwrap();

        manager.get_milestone_mut(&id1).unwrap().start().unwrap();

        let not_started = manager.get_milestones_by_status(MilestoneStatus::NotStarted);
        let in_progress = manager.get_milestones_by_status(MilestoneStatus::InProgress);

        assert_eq!(not_started.len(), 1);
        assert_eq!(in_progress.len(), 1);
    }

    #[test]
    fn test_milestone_manager_get_startable_milestones() {
        let mut manager = MilestoneManager::new();

        let id1 = manager.create_milestone("Milestone 1").unwrap();
        let id2 = manager.create_milestone("Milestone 2").unwrap();

        // 没有任务的里程碑不能开始
        let startable = manager.get_startable_milestones();
        assert_eq!(startable.len(), 0);

        // 添加任务到里程碑1
        manager
            .get_milestone_mut(&id1)
            .unwrap()
            .add_task(TaskId(1))
            .unwrap();

        let startable = manager.get_startable_milestones();
        assert_eq!(startable.len(), 1);
        assert_eq!(startable[0].id, id1);
    }

    #[test]
    fn test_milestone_manager_calculate_overall_progress() {
        let mut manager = MilestoneManager::new();
        let mut task_statuses = HashMap::new();

        // 空管理器
        assert_eq!(manager.calculate_overall_progress(&task_statuses), 0.0);

        let id1 = manager.create_milestone("Milestone 1").unwrap();
        let id2 = manager.create_milestone("Milestone 2").unwrap();

        // 添加任务
        manager
            .get_milestone_mut(&id1)
            .unwrap()
            .add_task(TaskId(1))
            .unwrap();
        manager
            .get_milestone_mut(&id2)
            .unwrap()
            .add_task(TaskId(2))
            .unwrap();

        // 没有任务完成
        assert_eq!(manager.calculate_overall_progress(&task_statuses), 0.0);

        // 完成一个任务
        task_statuses.insert(TaskId(1), TaskStatus::Done);
        assert_eq!(manager.calculate_overall_progress(&task_statuses), 0.25); // (0.0 + 0.5) / 2

        // 完成所有任务
        task_statuses.insert(TaskId(2), TaskStatus::Done);
        assert_eq!(manager.calculate_overall_progress(&task_statuses), 0.75); // (0.5 + 1.0) / 2
    }

    #[test]
    fn test_milestone_id_display() {
        let id = MilestoneId::new(42);
        assert_eq!(format!("{}", id), "Milestone(42)");
    }

    #[test]
    fn test_milestone_status_display() {
        assert_eq!(format!("{}", MilestoneStatus::NotStarted), "NotStarted");
        assert_eq!(format!("{}", MilestoneStatus::InProgress), "InProgress");
        assert_eq!(format!("{}", MilestoneStatus::Completed), "Completed");
    }
}
