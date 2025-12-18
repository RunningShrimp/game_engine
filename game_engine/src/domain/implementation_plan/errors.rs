//! 实施计划领域特定错误类型

use thiserror::Error;

/// 实施计划领域错误枚举
#[derive(Error, Debug, Clone)]
pub enum ImplementationPlanError {
    /// 任务领域错误
    #[error("Task domain error: {0}")]
    Task(#[from] TaskError),
    /// 里程碑领域错误
    #[error("Milestone domain error: {0}")]
    Milestone(#[from] MilestoneError),
    /// 风险领域错误
    #[error("Risk domain error: {0}")]
    Risk(#[from] RiskError),
    /// 报告领域错误
    #[error("Report domain error: {0}")]
    Report(#[from] ReportError),
    /// 通用实施计划错误
    #[error("Implementation plan error: {0}")]
    General(String),
}

/// 任务领域错误
#[derive(Error, Debug, Clone)]
pub enum TaskError {
    /// 任务未找到
    #[error("Task not found: {0}")]
    TaskNotFound(String),
    /// 无效任务状态转换
    #[error("Invalid task status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },
    /// 任务已存在
    #[error("Task already exists: {0}")]
    TaskAlreadyExists(String),
    /// 无效任务参数
    #[error("Invalid task parameter: {0}")]
    InvalidParameter(String),
    /// 任务依赖循环
    #[error("Task dependency cycle detected")]
    DependencyCycle,
}

/// 里程碑领域错误
#[derive(Error, Debug, Clone)]
pub enum MilestoneError {
    /// 里程碑未找到
    #[error("Milestone not found: {0}")]
    MilestoneNotFound(String),
    /// 里程碑已存在
    #[error("Milestone already exists: {0}")]
    MilestoneAlreadyExists(String),
    /// 无效里程碑参数
    #[error("Invalid milestone parameter: {0}")]
    InvalidParameter(String),
    /// 里程碑任务依赖冲突
    #[error("Milestone task dependency conflict: {0}")]
    TaskDependencyConflict(String),
}

/// 风险领域错误
#[derive(Error, Debug, Clone)]
pub enum RiskError {
    /// 风险未找到
    #[error("Risk not found: {0}")]
    RiskNotFound(String),
    /// 风险已存在
    #[error("Risk already exists: {0}")]
    RiskAlreadyExists(String),
    /// 无效风险参数
    #[error("Invalid risk parameter: {0}")]
    InvalidParameter(String),
    /// 无效风险状态转换
    #[error("Invalid risk status transition: {from} -> {to}")]
    InvalidStatusTransition { from: String, to: String },
}

/// 报告领域错误
#[derive(Error, Debug, Clone)]
pub enum ReportError {
    /// 报告生成失败
    #[error("Report generation failed: {0}")]
    GenerationFailed(String),
    /// 数据不足
    #[error("Insufficient data for report: {0}")]
    InsufficientData(String),
    /// 无效报告参数
    #[error("Invalid report parameter: {0}")]
    InvalidParameter(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_implementation_plan_error_from_task_error() {
        let task_error = TaskError::TaskNotFound("test".to_string());
        let impl_error: ImplementationPlanError = task_error.into();
        assert!(matches!(impl_error, ImplementationPlanError::Task(TaskError::TaskNotFound(_))));
    }

    #[test]
    fn test_implementation_plan_error_from_milestone_error() {
        let milestone_error = MilestoneError::MilestoneNotFound("test".to_string());
        let impl_error: ImplementationPlanError = milestone_error.into();
        assert!(matches!(impl_error, ImplementationPlanError::Milestone(MilestoneError::MilestoneNotFound(_))));
    }

    #[test]
    fn test_implementation_plan_error_from_risk_error() {
        let risk_error = RiskError::RiskNotFound("test".to_string());
        let impl_error: ImplementationPlanError = risk_error.into();
        assert!(matches!(impl_error, ImplementationPlanError::Risk(RiskError::RiskNotFound(_))));
    }

    #[test]
    fn test_implementation_plan_error_from_report_error() {
        let report_error = ReportError::GenerationFailed("test".to_string());
        let impl_error: ImplementationPlanError = report_error.into();
        assert!(matches!(impl_error, ImplementationPlanError::Report(ReportError::GenerationFailed(_))));
    }

    #[test]
    fn test_task_error_variants() {
        assert!(matches!(TaskError::TaskNotFound("test".to_string()), TaskError::TaskNotFound(_)));
        assert!(matches!(TaskError::InvalidStatusTransition { from: "todo".to_string(), to: "done".to_string() }, TaskError::InvalidStatusTransition { .. }));
        assert!(matches!(TaskError::TaskAlreadyExists("test".to_string()), TaskError::TaskAlreadyExists(_)));
        assert!(matches!(TaskError::InvalidParameter("test".to_string()), TaskError::InvalidParameter(_)));
        assert!(matches!(TaskError::DependencyCycle, TaskError::DependencyCycle));
    }

    #[test]
    fn test_milestone_error_variants() {
        assert!(matches!(MilestoneError::MilestoneNotFound("test".to_string()), MilestoneError::MilestoneNotFound(_)));
        assert!(matches!(MilestoneError::MilestoneAlreadyExists("test".to_string()), MilestoneError::MilestoneAlreadyExists(_)));
        assert!(matches!(MilestoneError::InvalidParameter("test".to_string()), MilestoneError::InvalidParameter(_)));
        assert!(matches!(MilestoneError::TaskDependencyConflict("test".to_string()), MilestoneError::TaskDependencyConflict(_)));
    }

    #[test]
    fn test_risk_error_variants() {
        assert!(matches!(RiskError::RiskNotFound("test".to_string()), RiskError::RiskNotFound(_)));
        assert!(matches!(RiskError::RiskAlreadyExists("test".to_string()), RiskError::RiskAlreadyExists(_)));
        assert!(matches!(RiskError::InvalidParameter("test".to_string()), RiskError::InvalidParameter(_)));
        assert!(matches!(RiskError::InvalidStatusTransition { from: "identified".to_string(), to: "mitigated".to_string() }, RiskError::InvalidStatusTransition { .. }));
    }

    #[test]
    fn test_report_error_variants() {
        assert!(matches!(ReportError::GenerationFailed("test".to_string()), ReportError::GenerationFailed(_)));
        assert!(matches!(ReportError::InsufficientData("test".to_string()), ReportError::InsufficientData(_)));
        assert!(matches!(ReportError::InvalidParameter("test".to_string()), ReportError::InvalidParameter(_)));
    }
}