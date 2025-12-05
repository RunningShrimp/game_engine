//! 实施计划服务层
//!
//! 该模块提供实施计划执行系统的高层服务接口，协调任务管理、
//! 里程碑管理和风险管理等功能。

use crate::domain::implementation_plan::{
    ImplementationPlanError, TaskId, TaskStatus, TaskManager, MilestoneId, MilestoneManager,
    RiskId, RiskManager, ImplementationReport, ReportGenerator,
};
use crate::domain::implementation_plan::task::{Task, TaskPriority};
use crate::domain::implementation_plan::milestone::Milestone;
use crate::domain::implementation_plan::risk::{Risk, MitigationMeasure};

/// 实施计划服务
///
/// 提供统一的接口来管理实施计划的各个方面，包括任务、里程碑、风险和报告生成。
pub struct ImplementationPlanService {
    task_manager: TaskManager,
    milestone_manager: MilestoneManager,
    risk_manager: RiskManager,
    report_generator: ReportGenerator,
}

impl ImplementationPlanService {
    /// 创建新的实施计划服务
    pub fn new() -> Self {
        Self {
            task_manager: TaskManager::new(),
            milestone_manager: MilestoneManager::new(),
            risk_manager: RiskManager::new(),
            report_generator: ReportGenerator::new(),
        }
    }

    /// 创建任务
    pub fn create_task(&mut self, name: impl Into<String>) -> Result<TaskId, ImplementationPlanError> {
        self.task_manager.create_task(name)
    }

    /// 获取任务
    pub fn get_task(&self, id: &TaskId) -> Result<&Task, ImplementationPlanError> {
        self.task_manager.get_task(id)
    }

    /// 更新任务状态
    pub fn update_task_status(&mut self, id: &TaskId, status: TaskStatus) -> Result<(), ImplementationPlanError> {
        let task = self.task_manager.get_task_mut(id)?;
        match status {
            TaskStatus::Todo => Ok(()), // 已经是待办状态
            TaskStatus::InProgress => task.start(),
            TaskStatus::Done => task.complete(),
            TaskStatus::Cancelled => task.cancel(),
        }
    }

    /// 更新任务属性
    pub fn update_task(
        &mut self,
        id: &TaskId,
        name: Option<String>,
        description: Option<String>,
        priority: Option<TaskPriority>,
        assignee: Option<String>,
        due_date: Option<u64>,
    ) -> Result<(), ImplementationPlanError> {
        let task = self.task_manager.get_task_mut(id)?;

        if let Some(_name) = name {
            // 注意：这里我们不直接修改名称，因为名称在创建时设置且不可变
            // 如果需要重命名，应该创建一个新任务
        }

        if let Some(description) = description {
            task.description = Some(description);
        }

        if let Some(priority) = priority {
            task.priority = priority;
        }

        if let Some(assignee) = assignee {
            task.assignee = Some(assignee);
        }

        if let Some(due_date) = due_date {
            task.due_date = Some(due_date);
        }

        task.updated_at = crate::core::utils::current_timestamp();
        Ok(())
    }

    /// 删除任务
    pub fn delete_task(&mut self, id: &TaskId) -> Result<(), ImplementationPlanError> {
        self.task_manager.delete_task(id)
    }

    /// 添加任务依赖
    pub fn add_task_dependency(&mut self, task_id: TaskId, dependency_id: TaskId) -> Result<(), ImplementationPlanError> {
        let task = self.task_manager.get_task_mut(&task_id)?;
        task.add_dependency(dependency_id)
    }

    /// 获取可开始的任务
    pub fn get_startable_tasks(&self) -> Vec<&Task> {
        self.task_manager.get_startable_tasks()
    }

    /// 创建里程碑
    pub fn create_milestone(&mut self, name: impl Into<String>) -> Result<MilestoneId, ImplementationPlanError> {
        self.milestone_manager.create_milestone(name)
    }

    /// 获取里程碑
    pub fn get_milestone(&self, id: &MilestoneId) -> Result<&Milestone, ImplementationPlanError> {
        self.milestone_manager.get_milestone(id)
    }

    /// 更新里程碑状态
    pub fn update_milestone_status(&mut self, id: &MilestoneId, start: bool) -> Result<(), ImplementationPlanError> {
        let milestone = self.milestone_manager.get_milestone_mut(id)?;
        if start {
            milestone.start()
        } else {
            milestone.complete()
        }
    }

    /// 向里程碑添加任务
    pub fn add_task_to_milestone(&mut self, milestone_id: MilestoneId, task_id: TaskId) -> Result<(), ImplementationPlanError> {
        let milestone = self.milestone_manager.get_milestone_mut(&milestone_id)?;
        milestone.add_task(task_id)
    }

    /// 从里程碑移除任务
    pub fn remove_task_from_milestone(&mut self, milestone_id: &MilestoneId, task_id: &TaskId) -> Result<(), ImplementationPlanError> {
        let milestone = self.milestone_manager.get_milestone_mut(milestone_id)?;
        milestone.remove_task(task_id)
    }

    /// 获取可以完成的里程碑
    pub fn get_completable_milestones(&self) -> Vec<&Milestone> {
        let task_statuses = self.build_task_status_map();
        self.milestone_manager.get_completable_milestones(&task_statuses)
    }

    /// 创建风险
    pub fn create_risk(&mut self, name: impl Into<String>, description: impl Into<String>) -> Result<RiskId, ImplementationPlanError> {
        self.risk_manager.create_risk(name, description)
    }

    /// 获取风险
    pub fn get_risk(&self, id: &RiskId) -> Result<&Risk, ImplementationPlanError> {
        self.risk_manager.get_risk(id)
    }

    /// 更新风险状态
    pub fn update_risk_status(&mut self, id: &RiskId, start_mitigation: bool) -> Result<(), ImplementationPlanError> {
        let risk = self.risk_manager.get_risk_mut(id)?;
        if start_mitigation {
            risk.start_mitigation()
        } else {
            risk.mitigate()
        }
    }

    /// 向风险添加缓解措施
    pub fn add_mitigation_measure(&mut self, risk_id: RiskId, description: impl Into<String>) -> Result<(), ImplementationPlanError> {
        let risk = self.risk_manager.get_risk_mut(&risk_id)?;
        let measure = MitigationMeasure::new(description);
        risk.add_mitigation_measure(measure)
    }

    /// 完成缓解措施
    pub fn complete_mitigation_measure(&mut self, risk_id: &RiskId, measure_id: &str) -> Result<(), ImplementationPlanError> {
        // 这里需要找到对应的缓解措施并标记完成
        // 简化实现，实际应用中可能需要更复杂的逻辑
        let risk = self.risk_manager.get_risk_mut(risk_id)?;
        if let Some(measure) = risk.mitigation_measures.iter_mut().find(|m| m.id == measure_id) {
            measure.complete();
            Ok(())
        } else {
            Err(ImplementationPlanError::Risk(crate::domain::implementation_plan::errors::RiskError::InvalidParameter(
                format!("Mitigation measure {} not found", measure_id),
            )))
        }
    }

    /// 获取高优先级风险
    pub fn get_high_priority_risks(&self) -> Vec<&Risk> {
        self.risk_manager.get_high_priority_risks()
    }

    /// 生成实施报告
    pub fn generate_report(&self, title: impl Into<String>) -> Result<ImplementationReport, ImplementationPlanError> {
        self.report_generator.generate_report(
            title,
            &self.task_manager,
            &self.milestone_manager,
            &self.risk_manager,
        )
    }

    /// 获取总体进度
    pub fn get_overall_progress(&self) -> f32 {
        let task_completion_rate = {
            let tasks = self.task_manager.get_all_tasks();
            if tasks.is_empty() {
                0.0
            } else {
                let done_count = tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
                done_count as f32 / tasks.len() as f32
            }
        };

        let milestone_progress = {
            let task_statuses = self.build_task_status_map();
            self.milestone_manager.calculate_overall_progress(&task_statuses)
        };

        let risk_progress = self.risk_manager.calculate_overall_mitigation_progress();

        // 加权平均
        0.5 * task_completion_rate + 0.3 * milestone_progress + 0.2 * risk_progress
    }

    /// 构建任务状态映射
    fn build_task_status_map(&self) -> std::collections::HashMap<TaskId, TaskStatus> {
        let mut map = std::collections::HashMap::new();
        for task in self.task_manager.get_all_tasks() {
            map.insert(task.id, task.status);
        }
        map
    }
}

impl Default for ImplementationPlanService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implementation_plan_service_creation() {
        let service = ImplementationPlanService::new();
        // 测试创建成功
    }

    #[test]
    fn test_task_management_workflow() {
        let mut service = ImplementationPlanService::new();

        // 创建任务
        let task_id = service.create_task("Test Task").unwrap();

        // 获取任务
        let task = service.get_task(&task_id).unwrap();
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.status, TaskStatus::Todo);

        // 开始任务
        service.update_task_status(&task_id, TaskStatus::InProgress).unwrap();
        let task = service.get_task(&task_id).unwrap();
        assert_eq!(task.status, TaskStatus::InProgress);

        // 完成任务
        service.update_task_status(&task_id, TaskStatus::Done).unwrap();
        let task = service.get_task(&task_id).unwrap();
        assert_eq!(task.status, TaskStatus::Done);
    }

    #[test]
    fn test_milestone_management_workflow() {
        let mut service = ImplementationPlanService::new();

        // 创建里程碑
        let milestone_id = service.create_milestone("Test Milestone").unwrap();

        // 获取里程碑
        let milestone = service.get_milestone(&milestone_id).unwrap();
        assert_eq!(milestone.name, "Test Milestone");

        // 创建任务并添加到里程碑
        let task_id = service.create_task("Milestone Task").unwrap();
        service.add_task_to_milestone(milestone_id, task_id).unwrap();

        // 验证任务已添加到里程碑
        let milestone = service.get_milestone(&milestone_id).unwrap();
        assert!(milestone.task_ids.contains(&task_id));
    }

    #[test]
    fn test_risk_management_workflow() {
        let mut service = ImplementationPlanService::new();

        // 创建风险
        let risk_id = service.create_risk("Test Risk", "Test description").unwrap();

        // 获取风险
        let risk = service.get_risk(&risk_id).unwrap();
        assert_eq!(risk.name, "Test Risk");

        // 添加缓解措施
        service.add_mitigation_measure(risk_id, "Test mitigation").unwrap();

        // 验证缓解措施已添加
        let risk = service.get_risk(&risk_id).unwrap();
        assert_eq!(risk.mitigation_measures.len(), 1);
    }

    #[test]
    fn test_report_generation() {
        let service = ImplementationPlanService::new();

        // 生成报告
        let report = service.generate_report("Test Report").unwrap();
        assert_eq!(report.title, "Test Report");
        assert_eq!(report.task_stats.total, 0);
        assert_eq!(report.milestone_stats.total, 0);
        assert_eq!(report.risk_stats.total, 0);
    }

    #[test]
    fn test_overall_progress_calculation() {
        let service = ImplementationPlanService::new();

        // 空服务应该返回0进度
        let progress = service.get_overall_progress();
        assert_eq!(progress, 0.0);
    }

    #[test]
    fn test_get_startable_tasks() {
        let mut service = ImplementationPlanService::new();

        // 创建任务
        let task1_id = service.create_task("Task 1").unwrap();
        let task2_id = service.create_task("Task 2").unwrap();

        // Task 2 依赖 Task 1
        service.add_task_dependency(task2_id, task1_id).unwrap();

        // 只有 Task 1 可以开始
        let startable = service.get_startable_tasks();
        assert_eq!(startable.len(), 1);
        assert_eq!(startable[0].id, task1_id);
    }
}