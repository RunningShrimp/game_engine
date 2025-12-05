//! 报告生成领域对象
//!
//! 该模块实现了报告生成的业务逻辑，支持生成实施计划执行报告。

use crate::domain::implementation_plan::errors::ImplementationPlanError;
use crate::domain::implementation_plan::task::{TaskStatus, TaskManager};
use crate::domain::implementation_plan::milestone::MilestoneManager;
use crate::domain::implementation_plan::risk::RiskManager;
use serde::{Deserialize, Serialize};

/// 任务统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStatistics {
    /// 总任务数
    pub total: usize,
    /// 待办任务数
    pub todo: usize,
    /// 进行中任务数
    pub in_progress: usize,
    /// 已完成任务数
    pub done: usize,
    /// 已取消任务数
    pub cancelled: usize,
    /// 过期任务数
    pub overdue: usize,
    /// 完成率（0.0 到 1.0）
    pub completion_rate: f32,
}

/// 里程碑统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneStatistics {
    /// 总里程碑数
    pub total: usize,
    /// 未开始里程碑数
    pub not_started: usize,
    /// 进行中里程碑数
    pub in_progress: usize,
    /// 已完成里程碑数
    pub completed: usize,
    /// 过期里程碑数
    pub overdue: usize,
    /// 平均进度（0.0 到 1.0）
    pub average_progress: f32,
}

/// 风险统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStatistics {
    /// 总风险数
    pub total: usize,
    /// 已识别风险数
    pub identified: usize,
    /// 正在缓解风险数
    pub mitigating: usize,
    /// 已缓解风险数
    pub mitigated: usize,
    /// 已发生风险数
    pub occurred: usize,
    /// 高风险项目数
    pub high_priority: usize,
    /// 有过期缓解措施的风险数
    pub with_overdue_measures: usize,
    /// 总体缓解进度（0.0 到 1.0）
    pub mitigation_progress: f32,
}

/// 实施报告 - 值对象
///
/// 包含实施计划执行的完整统计信息和分析结果。
///
/// ## 包含内容
///
/// - 任务统计信息
/// - 里程碑统计信息
/// - 风险统计信息
/// - 生成时间戳
/// - 总体进度评估
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationReport {
    /// 报告标题
    pub title: String,
    /// 报告描述
    pub description: Option<String>,
    /// 任务统计
    pub task_stats: TaskStatistics,
    /// 里程碑统计
    pub milestone_stats: MilestoneStatistics,
    /// 风险统计
    pub risk_stats: RiskStatistics,
    /// 总体进度（0.0 到 1.0）
    pub overall_progress: f32,
    /// 关键问题和建议
    pub key_issues: Vec<String>,
    /// 建议行动项
    pub recommendations: Vec<String>,
    /// 生成时间戳
    pub generated_at: u64,
}

impl ImplementationReport {
    /// 创建新的实施报告
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            task_stats: TaskStatistics {
                total: 0,
                todo: 0,
                in_progress: 0,
                done: 0,
                cancelled: 0,
                overdue: 0,
                completion_rate: 0.0,
            },
            milestone_stats: MilestoneStatistics {
                total: 0,
                not_started: 0,
                in_progress: 0,
                completed: 0,
                overdue: 0,
                average_progress: 0.0,
            },
            risk_stats: RiskStatistics {
                total: 0,
                identified: 0,
                mitigating: 0,
                mitigated: 0,
                occurred: 0,
                high_priority: 0,
                with_overdue_measures: 0,
                mitigation_progress: 0.0,
            },
            overall_progress: 0.0,
            key_issues: Vec::new(),
            recommendations: Vec::new(),
            generated_at: Self::current_timestamp(),
        }
    }

    /// 设置报告描述
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// 计算总体进度
    pub fn calculate_overall_progress(&mut self) {
        // 总体进度基于任务完成率、里程碑进度和风险缓解进度的加权平均
        let task_weight = 0.5;
        let milestone_weight = 0.3;
        let risk_weight = 0.2;

        let task_progress = self.task_stats.completion_rate;
        let milestone_progress = self.milestone_stats.average_progress;
        let risk_progress = self.risk_stats.mitigation_progress;

        self.overall_progress = task_progress * task_weight +
                               milestone_progress * milestone_weight +
                               risk_progress * risk_weight;
    }

    /// 生成关键问题分析
    pub fn analyze_key_issues(&mut self) {
        self.key_issues.clear();

        // 分析任务相关问题
        if self.task_stats.overdue > 0 {
            self.key_issues.push(format!("有 {} 个任务已过期", self.task_stats.overdue));
        }

        if self.task_stats.completion_rate < 0.5 {
            self.key_issues.push("任务完成率较低，需要加快进度".to_string());
        }

        // 分析里程碑相关问题
        if self.milestone_stats.overdue > 0 {
            self.key_issues.push(format!("有 {} 个里程碑已过期", self.milestone_stats.overdue));
        }

        if self.milestone_stats.average_progress < 0.3 {
            self.key_issues.push("里程碑整体进度缓慢".to_string());
        }

        // 分析风险相关问题
        if self.risk_stats.occurred > 0 {
            self.key_issues.push(format!("有 {} 个风险已发生", self.risk_stats.occurred));
        }

        if self.risk_stats.high_priority > 0 {
            self.key_issues.push(format!("有 {} 个高优先级风险需要关注", self.risk_stats.high_priority));
        }

        if self.risk_stats.with_overdue_measures > 0 {
            self.key_issues.push(format!("有 {} 个风险的缓解措施已过期", self.risk_stats.with_overdue_measures));
        }

        if self.key_issues.is_empty() {
            self.key_issues.push("项目进展正常，无重大问题".to_string());
        }
    }

    /// 生成建议行动项
    pub fn generate_recommendations(&mut self) {
        self.recommendations.clear();

        // 基于统计数据生成建议
        if self.task_stats.overdue > 0 {
            self.recommendations.push("优先处理过期任务，重新评估时间计划".to_string());
        }

        if self.task_stats.completion_rate < 0.7 {
            self.recommendations.push("增加资源投入，加快任务执行速度".to_string());
        }

        if self.milestone_stats.overdue > 0 {
            self.recommendations.push("审查里程碑计划，调整关键路径".to_string());
        }

        if self.risk_stats.high_priority > 0 {
            self.recommendations.push("制定高风险缓解计划，分配专门资源".to_string());
        }

        if self.risk_stats.with_overdue_measures > 0 {
            self.recommendations.push("跟进过期缓解措施，评估影响".to_string());
        }

        if self.overall_progress > 0.8 {
            self.recommendations.push("项目进展良好，准备收尾工作".to_string());
        } else if self.overall_progress < 0.3 {
            self.recommendations.push("项目进度严重滞后，需要管理层干预".to_string());
        }

        if self.recommendations.is_empty() {
            self.recommendations.push("继续保持当前进度，定期监控项目状态".to_string());
        }
    }

    /// 获取当前时间戳
    fn current_timestamp() -> u64 {
        crate::core::utils::current_timestamp()
    }
}

/// 报告生成器
pub struct ReportGenerator;

impl ReportGenerator {
    /// 创建新的报告生成器
    pub fn new() -> Self {
        Self
    }

    /// 生成实施报告
    pub fn generate_report(
        &self,
        title: impl Into<String>,
        task_manager: &TaskManager,
        milestone_manager: &MilestoneManager,
        risk_manager: &RiskManager,
    ) -> Result<ImplementationReport, ImplementationPlanError> {
        let mut report = ImplementationReport::new(title);

        // 收集任务统计
        self.collect_task_statistics(&mut report.task_stats, task_manager);

        // 收集里程碑统计
        self.collect_milestone_statistics(&mut report.milestone_stats, milestone_manager, task_manager);

        // 收集风险统计
        self.collect_risk_statistics(&mut report.risk_stats, risk_manager);

        // 计算总体进度
        report.calculate_overall_progress();

        // 生成分析
        report.analyze_key_issues();
        report.generate_recommendations();

        Ok(report)
    }

    /// 收集任务统计信息
    fn collect_task_statistics(&self, stats: &mut TaskStatistics, task_manager: &TaskManager) {
        let tasks = task_manager.get_all_tasks();

        stats.total = tasks.len();
        stats.todo = tasks.iter().filter(|t| t.status == TaskStatus::Todo).count();
        stats.in_progress = tasks.iter().filter(|t| t.status == TaskStatus::InProgress).count();
        stats.done = tasks.iter().filter(|t| t.status == TaskStatus::Done).count();
        stats.cancelled = tasks.iter().filter(|t| t.status == TaskStatus::Cancelled).count();
        stats.overdue = tasks.iter().filter(|t| t.is_overdue()).count();

        if stats.total > 0 {
            stats.completion_rate = stats.done as f32 / stats.total as f32;
        }
    }

    /// 收集里程碑统计信息
    fn collect_milestone_statistics(
        &self,
        stats: &mut MilestoneStatistics,
        milestone_manager: &MilestoneManager,
        task_manager: &TaskManager,
    ) {
        let milestones = milestone_manager.get_all_milestones();

        // 构建任务状态映射
        let mut task_statuses = std::collections::HashMap::new();
        for task in task_manager.get_all_tasks() {
            task_statuses.insert(task.id, task.status);
        }

        stats.total = milestones.len();
        stats.not_started = milestones.iter().filter(|m| m.status == crate::domain::implementation_plan::milestone::MilestoneStatus::NotStarted).count();
        stats.in_progress = milestones.iter().filter(|m| m.status == crate::domain::implementation_plan::milestone::MilestoneStatus::InProgress).count();
        stats.completed = milestones.iter().filter(|m| m.status == crate::domain::implementation_plan::milestone::MilestoneStatus::Completed).count();
        stats.overdue = milestones.iter().filter(|m| m.is_overdue()).count();

        if stats.total > 0 {
            let total_progress: f32 = milestones.iter()
                .map(|m| m.calculate_progress(&task_statuses))
                .sum();
            stats.average_progress = total_progress / stats.total as f32;
        }
    }

    /// 收集风险统计信息
    fn collect_risk_statistics(&self, stats: &mut RiskStatistics, risk_manager: &RiskManager) {
        let risks = risk_manager.get_all_risks();

        stats.total = risks.len();
        stats.identified = risks.iter().filter(|r| r.status == crate::domain::implementation_plan::risk::RiskStatus::Identified).count();
        stats.mitigating = risks.iter().filter(|r| r.status == crate::domain::implementation_plan::risk::RiskStatus::Mitigating).count();
        stats.mitigated = risks.iter().filter(|r| r.status == crate::domain::implementation_plan::risk::RiskStatus::Mitigated).count();
        stats.occurred = risks.iter().filter(|r| r.status == crate::domain::implementation_plan::risk::RiskStatus::Occurred).count();
        stats.high_priority = risk_manager.get_high_priority_risks().len();
        stats.with_overdue_measures = risk_manager.get_risks_with_overdue_measures().len();
        stats.mitigation_progress = risk_manager.calculate_overall_mitigation_progress();
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::implementation_plan::task::TaskManager;
    use crate::domain::implementation_plan::milestone::MilestoneManager;
    use crate::domain::implementation_plan::risk::RiskManager;

    #[test]
    fn test_implementation_report_creation() {
        let report = ImplementationReport::new("Test Report");
        assert_eq!(report.title, "Test Report");
        assert_eq!(report.task_stats.total, 0);
        assert_eq!(report.overall_progress, 0.0);
    }

    #[test]
    fn test_implementation_report_with_description() {
        let report = ImplementationReport::new("Test Report")
            .with_description("Test description");
        assert_eq!(report.description, Some("Test description".to_string()));
    }

    #[test]
    fn test_implementation_report_calculate_overall_progress() {
        let mut report = ImplementationReport::new("Test Report");

        // 设置测试数据
        report.task_stats.completion_rate = 0.8; // 80%
        report.milestone_stats.average_progress = 0.6; // 60%
        report.risk_stats.mitigation_progress = 0.9; // 90%

        report.calculate_overall_progress();

        // 预期: 0.8 * 0.5 + 0.6 * 0.3 + 0.9 * 0.2 = 0.4 + 0.18 + 0.18 = 0.76
        assert!((report.overall_progress - 0.76).abs() < 0.001);
    }

    #[test]
    fn test_implementation_report_analyze_key_issues() {
        let mut report = ImplementationReport::new("Test Report");

        // 设置有问题的统计数据
        report.task_stats.overdue = 2;
        report.task_stats.completion_rate = 0.3;
        report.milestone_stats.overdue = 1;
        report.risk_stats.occurred = 1;
        report.risk_stats.high_priority = 3;

        report.analyze_key_issues();

        assert!(report.key_issues.len() > 0);
        assert!(report.key_issues.iter().any(|issue| issue.contains("过期")));
        assert!(report.key_issues.iter().any(|issue| issue.contains("完成率较低")));
        assert!(report.key_issues.iter().any(|issue| issue.contains("已发生")));
        assert!(report.key_issues.iter().any(|issue| issue.contains("高优先级")));
    }

    #[test]
    fn test_implementation_report_generate_recommendations() {
        let mut report = ImplementationReport::new("Test Report");

        // 设置需要建议的情况
        report.task_stats.overdue = 1;
        report.task_stats.completion_rate = 0.4;
        report.risk_stats.high_priority = 2;
        report.overall_progress = 0.2;

        report.generate_recommendations();

        assert!(report.recommendations.len() > 0);
        assert!(report.recommendations.iter().any(|rec| rec.contains("过期")));
        assert!(report.recommendations.iter().any(|rec| rec.contains("资源投入")));
        assert!(report.recommendations.iter().any(|rec| rec.contains("高风险")));
        assert!(report.recommendations.iter().any(|rec| rec.contains("严重滞后")));
    }

    #[test]
    fn test_report_generator_new() {
        let generator = ReportGenerator::new();
        // 只是测试创建，没有其他断言
    }

    #[test]
    fn test_report_generator_generate_report() {
        let generator = ReportGenerator::new();
        let task_manager = TaskManager::new();
        let milestone_manager = MilestoneManager::new();
        let risk_manager = RiskManager::new();

        let report = generator.generate_report(
            "Test Report",
            &task_manager,
            &milestone_manager,
            &risk_manager,
        ).unwrap();

        assert_eq!(report.title, "Test Report");
        assert_eq!(report.task_stats.total, 0);
        assert_eq!(report.milestone_stats.total, 0);
        assert_eq!(report.risk_stats.total, 0);
    }

    #[test]
    fn test_report_generator_collect_task_statistics() {
        let generator = ReportGenerator::new();
        let mut task_manager = TaskManager::new();
        let mut stats = TaskStatistics {
            total: 0,
            todo: 0,
            in_progress: 0,
            done: 0,
            cancelled: 0,
            overdue: 0,
            completion_rate: 0.0,
        };

        // 创建一些测试任务
        let task1_id = task_manager.create_task("Task 1").unwrap();
        let task2_id = task_manager.create_task("Task 2").unwrap();
        let task3_id = task_manager.create_task("Task 3").unwrap();

        task_manager.get_task_mut(&task1_id).unwrap().complete().unwrap();
        task_manager.get_task_mut(&task2_id).unwrap().start().unwrap();

        generator.collect_task_statistics(&mut stats, &task_manager);

        assert_eq!(stats.total, 3);
        assert_eq!(stats.done, 1);
        assert_eq!(stats.in_progress, 1);
        assert_eq!(stats.todo, 1);
        assert_eq!(stats.completion_rate, 1.0 / 3.0);
    }

    #[test]
    fn test_report_generator_collect_milestone_statistics() {
        let generator = ReportGenerator::new();
        let mut milestone_manager = MilestoneManager::new();
        let task_manager = TaskManager::new();
        let mut stats = MilestoneStatistics {
            total: 0,
            not_started: 0,
            in_progress: 0,
            completed: 0,
            overdue: 0,
            average_progress: 0.0,
        };

        // 创建测试里程碑
        let milestone_id = milestone_manager.create_milestone("Test Milestone").unwrap();

        generator.collect_milestone_statistics(&mut stats, &milestone_manager, &task_manager);

        assert_eq!(stats.total, 1);
        assert_eq!(stats.not_started, 1);
        assert_eq!(stats.average_progress, 0.0);
    }

    #[test]
    fn test_report_generator_collect_risk_statistics() {
        let generator = ReportGenerator::new();
        let mut risk_manager = RiskManager::new();
        let mut stats = RiskStatistics {
            total: 0,
            identified: 0,
            mitigating: 0,
            mitigated: 0,
            occurred: 0,
            high_priority: 0,
            with_overdue_measures: 0,
            mitigation_progress: 0.0,
        };

        // 创建测试风险
        let risk_id = risk_manager.create_risk("Test Risk", "Test description").unwrap();

        generator.collect_risk_statistics(&mut stats, &risk_manager);

        assert_eq!(stats.total, 1);
        assert_eq!(stats.identified, 1);
        assert_eq!(stats.mitigation_progress, 0.0);
    }
}