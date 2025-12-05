//! 实施计划执行系统领域层
//!
//! 该模块实现了一个完整的实施计划执行系统，支持任务管理、里程碑跟踪、
//! 风险管理和报告生成。系统采用DDD（领域驱动设计）原则，确保业务
//! 逻辑封装在领域对象中。

pub mod errors;
pub mod task;
pub mod milestone;
pub mod risk;
pub mod report;

// 重新导出主要类型
pub use errors::{ImplementationPlanError, TaskError, MilestoneError, RiskError, ReportError};
pub use task::{Task, TaskId, TaskStatus, TaskPriority, TaskManager};
pub use milestone::{Milestone, MilestoneId, MilestoneStatus, MilestoneManager};
pub use risk::{Risk, RiskId, RiskLevel, RiskStatus, RiskManager};
pub use report::{ImplementationReport, ReportGenerator};