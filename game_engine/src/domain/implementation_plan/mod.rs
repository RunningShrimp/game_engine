//  实施计划执行系统领域层
//
//  该模块实现了一个完整的实施计划执行系统，支持任务管理、里程碑跟踪、
//  风险管理和报告生成。系统采用DDD（领域驱动设计）原则，确保业务
//  逻辑封装在领域对象中。
//
//  # 弃用说明
//  此模块包含非核心游戏引擎领域的功能（计划管理、任务跟踪等）。
//  根据架构清理计划，此模块将在未来版本中迁移到独立的工具模块或移除。
//  新代码应避免依赖此模块。

pub mod errors;
pub mod milestone;
pub mod report;
pub mod risk;
pub mod task;

// 重新导出主要类型
pub use errors::{ImplementationPlanError, MilestoneError, ReportError, RiskError, TaskError};
pub use milestone::{Milestone, MilestoneId, MilestoneManager, MilestoneStatus};
pub use report::{ImplementationReport, ReportGenerator};
pub use risk::{Risk, RiskId, RiskLevel, RiskManager, RiskStatus};
pub use task::{Task, TaskId, TaskManager, TaskPriority, TaskStatus};
