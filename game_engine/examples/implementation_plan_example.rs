//  实施计划执行系统示例
// 
//  本示例演示如何使用实施计划执行系统来管理项目任务、里程碑和风险。

use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// 任务状态枚举
#[derive(Debug, Clone, PartialEq)]
enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Blocked,
}

/// 任务结构体
#[derive(Debug, Clone)]
struct Task {
    id: String,
    title: String,
    description: String,
    status: TaskStatus,
    responsible: String,
    completion_time: Option<DateTime<Utc>>,
}

/// 里程碑结构体
#[derive(Debug, Clone)]
struct Milestone {
    id: String,
    title: String,
    description: String,
    progress: u8, // 0-100
    status: TaskStatus,
    target_date: DateTime<Utc>,
    completion_time: Option<DateTime<Utc>>,
}

/// 风险严重程度枚举
#[derive(Debug, Clone, PartialEq)]
enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// 风险状态枚举
#[derive(Debug, Clone, PartialEq)]
enum RiskStatus {
    Identified,
    Mitigating,
    Mitigated,
    Resolved,
}

/// 风险结构体
#[derive(Debug, Clone)]
struct Risk {
    id: String,
    description: String,
    severity: RiskSeverity,
    status: RiskStatus,
    mitigation: String,
    mitigation_status: RiskStatus,
}

/// 实施计划执行系统
#[derive(Debug)]
struct ImplementationPlanExecutor {
    tasks: HashMap<String, Task>,
    milestones: HashMap<String, Milestone>,
    risks: HashMap<String, Risk>,
}

impl ImplementationPlanExecutor {
    /// 初始化实施计划执行系统
    fn new() -> Self {
        Self {
            tasks: HashMap::new(),
            milestones: HashMap::new(),
            risks: HashMap::new(),
        }
    }

    /// 创建新任务
    fn create_task(&mut self, id: String, title: String, description: String, responsible: String) {
        let task = Task {
            id: id.clone(),
            title,
            description,
            status: TaskStatus::Pending,
            responsible,
            completion_time: None,
        };
        self.tasks.insert(id, task);
    }

    /// 更新任务状态
    fn update_task_status(&mut self, task_id: &str, status: TaskStatus) {
        if let Some(task) = self.tasks.get_mut(task_id) {
            task.status = status.clone();
            if status == TaskStatus::Completed {
                task.completion_time = Some(Utc::now());
            }
        }
    }

    /// 创建里程碑
    fn create_milestone(
        &mut self,
        id: String,
        title: String,
        description: String,
        target_date: DateTime<Utc>,
    ) {
        let milestone = Milestone {
            id: id.clone(),
            title,
            description,
            progress: 0,
            status: TaskStatus::Pending,
            target_date,
            completion_time: None,
        };
        self.milestones.insert(id, milestone);
    }

    /// 更新里程碑进度
    fn update_milestone_progress(&mut self, milestone_id: &str, progress: u8) {
        if let Some(milestone) = self.milestones.get_mut(milestone_id) {
            milestone.progress = progress.min(100);
            if progress >= 100 {
                milestone.status = TaskStatus::Completed;
                milestone.completion_time = Some(Utc::now());
            } else if progress > 0 {
                milestone.status = TaskStatus::InProgress;
            }
        }
    }

    /// 识别风险
    fn identify_risk(
        &mut self,
        id: String,
        description: String,
        severity: RiskSeverity,
        mitigation: String,
    ) {
        let risk = Risk {
            id: id.clone(),
            description,
            severity,
            status: RiskStatus::Identified,
            mitigation,
            mitigation_status: RiskStatus::Identified,
        };
        self.risks.insert(id, risk);
    }

    /// 更新风险缓解状态
    fn update_risk_mitigation(&mut self, risk_id: &str, status: RiskStatus) {
        if let Some(risk) = self.risks.get_mut(risk_id) {
            risk.mitigation_status = status.clone();
            if status == RiskStatus::Resolved {
                risk.status = RiskStatus::Resolved;
            }
        }
    }

    /// 生成进度报告
    fn generate_progress_report(&self) -> String {
        let mut report = String::from("# 实施计划进度报告\n\n");

        // 任务统计
        let total_tasks = self.tasks.len();
        let completed_tasks = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::Completed)
            .count();
        let in_progress_tasks = self
            .tasks
            .values()
            .filter(|t| t.status == TaskStatus::InProgress)
            .count();

        report.push_str(&format!("## 任务统计\n"));
        report.push_str(&format!("- 总任务数: {}\n", total_tasks));
        report.push_str(&format!("- 已完成: {}\n", completed_tasks));
        report.push_str(&format!("- 进行中: {}\n", in_progress_tasks));
        report.push_str(&format!(
            "- 完成率: {:.1}%\n\n",
            if total_tasks > 0 {
                (completed_tasks as f32 / total_tasks as f32) * 100.0
            } else {
                0.0
            }
        ));

        // 里程碑统计
        let total_milestones = self.milestones.len();
        let completed_milestones = self
            .milestones
            .values()
            .filter(|m| m.status == TaskStatus::Completed)
            .count();
        let avg_progress: f32 = if total_milestones > 0 {
            self.milestones
                .values()
                .map(|m| m.progress as f32)
                .sum::<f32>()
                / total_milestones as f32
        } else {
            0.0
        };

        report.push_str(&format!("## 里程碑统计\n"));
        report.push_str(&format!("- 总里程碑数: {}\n", total_milestones));
        report.push_str(&format!("- 已完成: {}\n", completed_milestones));
        report.push_str(&format!("- 平均进度: {:.1}%\n\n", avg_progress));

        // 风险统计
        let total_risks = self.risks.len();
        let resolved_risks = self
            .risks
            .values()
            .filter(|r| r.status == RiskStatus::Resolved)
            .count();
        let high_severity_risks = self
            .risks
            .values()
            .filter(|r| r.severity == RiskSeverity::High || r.severity == RiskSeverity::Critical)
            .count();

        report.push_str(&format!("## 风险统计\n"));
        report.push_str(&format!("- 总风险数: {}\n", total_risks));
        report.push_str(&format!("- 已解决: {}\n", resolved_risks));
        report.push_str(&format!("- 高严重程度风险: {}\n\n", high_severity_risks));

        // 详细任务列表
        report.push_str("## 任务详情\n");
        for task in self.tasks.values() {
            let status_str = match task.status {
                TaskStatus::Pending => "待处理",
                TaskStatus::InProgress => "进行中",
                TaskStatus::Completed => "已完成",
                TaskStatus::Blocked => "已阻塞",
            };
            report.push_str(&format!(
                "- **{}**: {} ({})\n",
                task.title, status_str, task.responsible
            ));
        }

        report.push_str("\n## 里程碑详情\n");
        for milestone in self.milestones.values() {
            let status_str = match milestone.status {
                TaskStatus::Pending => "待处理",
                TaskStatus::InProgress => "进行中",
                TaskStatus::Completed => "已完成",
                TaskStatus::Blocked => "已阻塞",
            };
            report.push_str(&format!(
                "- **{}**: {}% 完成 ({})\n",
                milestone.title, milestone.progress, status_str
            ));
        }

        report.push_str("\n## 风险详情\n");
        for risk in self.risks.values() {
            let severity_str = match risk.severity {
                RiskSeverity::Low => "低",
                RiskSeverity::Medium => "中",
                RiskSeverity::High => "高",
                RiskSeverity::Critical => "严重",
            };
            let status_str = match risk.status {
                RiskStatus::Identified => "已识别",
                RiskStatus::Mitigating => "缓解中",
                RiskStatus::Mitigated => "已缓解",
                RiskStatus::Resolved => "已解决",
            };
            report.push_str(&format!(
                "- **{}** ({}): {}\n",
                risk.description, severity_str, status_str
            ));
        }

        report
    }
}

fn main() {
    println!("=== 实施计划执行系统示例 ===\n");

    // 1. 初始化系统
    let mut executor = ImplementationPlanExecutor::new();
    println!("✅ 实施计划执行系统已初始化\n");

    // 2. 创建和管理任务
    println!("2. 创建和管理任务:");
    executor.create_task(
        "task-001".to_string(),
        "实现GPU视锥剔除".to_string(),
        "实现视锥剔除以优化渲染性能".to_string(),
        "渲染团队".to_string(),
    );
    executor.create_task(
        "task-002".to_string(),
        "实现NPU加速".to_string(),
        "实现神经处理单元加速用于AI操作".to_string(),
        "硬件优化团队".to_string(),
    );
    executor.create_task(
        "task-003".to_string(),
        "优化SIMD批处理操作".to_string(),
        "使用SIMD指令优化批处理操作".to_string(),
        "性能团队".to_string(),
    );

    // 更新任务状态
    executor.update_task_status("task-003", TaskStatus::Completed);
    executor.update_task_status("task-001", TaskStatus::InProgress);
    executor.update_task_status("task-002", TaskStatus::InProgress);

    println!("   已创建3个任务，其中1个已完成，2个进行中\n");

    // 3. 创建和跟踪里程碑
    println!("3. 创建和跟踪里程碑:");
    executor.create_milestone(
        "milestone-001".to_string(),
        "渲染优化第一阶段".to_string(),
        "完成第一阶段的所有渲染优化任务".to_string(),
        Utc::now() + chrono::Duration::days(15),
    );
    executor.create_milestone(
        "milestone-002".to_string(),
        "硬件加速实现".to_string(),
        "实现NPU和GPU加速功能".to_string(),
        Utc::now() + chrono::Duration::days(45),
    );

    // 更新里程碑进度
    executor.update_milestone_progress("milestone-001", 70);
    executor.update_milestone_progress("milestone-002", 40);

    println!("   已创建2个里程碑，进度分别为70%和40%\n");

    // 4. 识别和缓解风险
    println!("4. 识别和缓解风险:");
    executor.identify_risk(
        "risk-001".to_string(),
        "GPU视锥剔除实现可能存在硬件兼容性问题".to_string(),
        RiskSeverity::Medium,
        "在多种硬件配置上测试，实现后备机制".to_string(),
    );
    executor.identify_risk(
        "risk-002".to_string(),
        "NPU加速在低端设备上可能无法达到预期性能".to_string(),
        RiskSeverity::High,
        "优化NPU内核以适应不同设备能力".to_string(),
    );

    // 更新风险缓解状态
    executor.update_risk_mitigation("risk-001", RiskStatus::Mitigating);

    println!("   已识别2个风险，其中1个正在缓解\n");

    // 5. 生成和使用报告
    println!("5. 生成进度报告:");
    let report = executor.generate_progress_report();
    println!("{}", report);

    println!("✅ 示例执行完成！");
}
