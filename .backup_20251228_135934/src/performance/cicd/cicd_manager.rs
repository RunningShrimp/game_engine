use crate::impl_default;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

/// CI/CD 阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CicdStage {
    Checkout,
    Build,
    UnitTest,
    IntegrationTest,
    BenchmarkTest,
    RegressionTest,
    Deploy,
}

impl CicdStage {
    pub fn as_str(&self) -> &str {
        match self {
            CicdStage::Checkout => "Checkout",
            CicdStage::Build => "Build",
            CicdStage::UnitTest => "UnitTest",
            CicdStage::IntegrationTest => "IntegrationTest",
            CicdStage::BenchmarkTest => "BenchmarkTest",
            CicdStage::RegressionTest => "RegressionTest",
            CicdStage::Deploy => "Deploy",
        }
    }

    pub fn order(&self) -> u8 {
        match self {
            CicdStage::Checkout => 0,
            CicdStage::Build => 1,
            CicdStage::UnitTest => 2,
            CicdStage::IntegrationTest => 3,
            CicdStage::BenchmarkTest => 4,
            CicdStage::RegressionTest => 5,
            CicdStage::Deploy => 6,
        }
    }
}

/// 阶段执行状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StageStatus {
    Pending,
    Running,
    Passed,
    Failed,
    Skipped,
}

impl StageStatus {
    pub fn as_str(&self) -> &str {
        match self {
            StageStatus::Pending => "Pending",
            StageStatus::Running => "Running",
            StageStatus::Passed => "Passed",
            StageStatus::Failed => "Failed",
            StageStatus::Skipped => "Skipped",
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            StageStatus::Passed | StageStatus::Failed | StageStatus::Skipped
        )
    }
}

/// 阶段执行结果
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage: CicdStage,
    pub status: StageStatus,
    pub duration: Duration,
    pub start_time: SystemTime,
    pub end_time: SystemTime,
    pub output: String,
    pub error: Option<String>,
}

impl StageResult {
    pub fn new(stage: CicdStage) -> Self {
        Self {
            stage,
            status: StageStatus::Pending,
            duration: Duration::ZERO,
            start_time: SystemTime::now(),
            end_time: SystemTime::now(),
            output: String::new(),
            error: None,
        }
    }

    pub fn mark_running(&mut self) {
        self.status = StageStatus::Running;
        self.start_time = SystemTime::now();
    }

    pub fn mark_passed(mut self, output: String) -> Self {
        self.status = StageStatus::Passed;
        self.end_time = SystemTime::now();
        self.output = output;
        if let Ok(duration) = self.end_time.duration_since(self.start_time) {
            self.duration = duration;
        }
        self
    }

    pub fn mark_failed(mut self, error: String) -> Self {
        self.status = StageStatus::Failed;
        self.end_time = SystemTime::now();
        self.error = Some(error);
        if let Ok(duration) = self.end_time.duration_since(self.start_time) {
            self.duration = duration;
        }
        self
    }

    pub fn mark_skipped(mut self) -> Self {
        self.status = StageStatus::Skipped;
        self.end_time = SystemTime::now();
        self
    }
}

/// CI/CD 流水线
pub struct CicdPipeline {
    pipeline_id: String,
    stages: Vec<StageResult>,
    created_at: SystemTime,
    started_at: Option<SystemTime>,
    completed_at: Option<SystemTime>,
    commit_hash: String,
    branch: String,
}

impl CicdPipeline {
    pub fn new(commit_hash: impl Into<String>, branch: impl Into<String>) -> Self {
        let pipeline_id = format!(
            "pipeline_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        );

        Self {
            pipeline_id,
            stages: Vec::new(),
            created_at: SystemTime::now(),
            started_at: None,
            completed_at: None,
            commit_hash: commit_hash.into(),
            branch: branch.into(),
        }
    }

    pub fn get_id(&self) -> &str {
        &self.pipeline_id
    }

    /// 添加阶段
    pub fn add_stage(&mut self, stage: CicdStage) {
        let mut result = StageResult::new(stage);
        result.start_time = SystemTime::now();
        self.stages.push(result);
    }

    /// 更新阶段状态
    pub fn update_stage(
        &mut self,
        stage: CicdStage,
        status: StageStatus,
        message: String,
    ) -> Result<(), &'static str> {
        if let Some(stage_result) = self.stages.iter_mut().find(|s| s.stage == stage) {
            stage_result.end_time = SystemTime::now();
            if let Ok(duration) = stage_result
                .end_time
                .duration_since(stage_result.start_time)
            {
                stage_result.duration = duration;
            }

            match status {
                StageStatus::Passed => {
                    stage_result.status = StageStatus::Passed;
                    stage_result.output = message;
                }
                StageStatus::Failed => {
                    stage_result.status = StageStatus::Failed;
                    stage_result.error = Some(message);
                }
                StageStatus::Skipped => {
                    stage_result.status = StageStatus::Skipped;
                }
                _ => {
                    stage_result.status = status;
                }
            }
            Ok(())
        } else {
            Err("Stage not found")
        }
    }

    /// 启动流水线
    pub fn start(&mut self) {
        self.started_at = Some(SystemTime::now());
    }

    /// 完成流水线
    pub fn complete(&mut self) {
        self.completed_at = Some(SystemTime::now());
    }

    /// 获取流水线总耗时
    pub fn get_total_duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => end.duration_since(start).ok(),
            _ => None,
        }
    }

    /// 获取流水线状态
    pub fn get_status(&self) -> PipelineStatus {
        if let Some(failed) = self.stages.iter().find(|s| s.status == StageStatus::Failed) {
            return PipelineStatus::Failed;
        }

        if self
            .stages
            .iter()
            .all(|s| matches!(s.status, StageStatus::Passed | StageStatus::Skipped))
        {
            return PipelineStatus::Passed;
        }

        if self.stages.iter().any(|s| s.status == StageStatus::Running) {
            return PipelineStatus::Running;
        }

        PipelineStatus::Pending
    }

    /// 获取摘要
    pub fn get_summary(&self) -> CicdSummary {
        let total_stages = self.stages.len();
        let passed_stages = self
            .stages
            .iter()
            .filter(|s| s.status == StageStatus::Passed)
            .count();
        let failed_stages = self
            .stages
            .iter()
            .filter(|s| s.status == StageStatus::Failed)
            .count();
        let skipped_stages = self
            .stages
            .iter()
            .filter(|s| s.status == StageStatus::Skipped)
            .count();

        let total_duration = self.get_total_duration().unwrap_or(Duration::ZERO);

        CicdSummary {
            pipeline_id: self.pipeline_id.clone(),
            status: self.get_status(),
            total_stages,
            passed_stages,
            failed_stages,
            skipped_stages,
            total_duration,
            commit_hash: self.commit_hash.clone(),
            branch: self.branch.clone(),
        }
    }

    /// 生成报告
    pub fn generate_report(&self) -> String {
        let summary = self.get_summary();
        let mut report = format!(
            "╔════════════════════════════════════════════╗\n\
             ║ CI/CD Pipeline Report\n\
             ║ Pipeline ID: {}\n\
             ║ Status: {}\n\
             ║ Commit: {}\n\
             ║ Branch: {}\n\
             ╠════════════════════════════════════════════╣\n",
            self.pipeline_id,
            summary.status.as_str(),
            self.commit_hash,
            self.branch
        );

        let total_duration = self.get_total_duration().unwrap_or(Duration::ZERO);
        report.push_str(&format!(
            "║ Total Duration: {:?}\n\
             ║ Stages: {}/{} passed\n",
            total_duration, summary.passed_stages, summary.total_stages
        ));

        report.push_str("╠════════════════════════════════════════════╣\n");

        for stage in &self.stages {
            report.push_str(&format!(
                "║ {} [{:?}] - {:.2}s\n",
                stage.stage.as_str(),
                stage.status,
                stage.duration.as_secs_f64()
            ));
        }

        report.push_str("╚════════════════════════════════════════════╝\n");
        report
    }
}

/// 流水线状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineStatus {
    Pending,
    Running,
    Passed,
    Failed,
}

impl PipelineStatus {
    pub fn as_str(&self) -> &str {
        match self {
            PipelineStatus::Pending => "Pending",
            PipelineStatus::Running => "Running",
            PipelineStatus::Passed => "Passed",
            PipelineStatus::Failed => "Failed",
        }
    }
}

/// CI/CD 流水线摘要
#[derive(Debug, Clone)]
pub struct CicdSummary {
    pub pipeline_id: String,
    pub status: PipelineStatus,
    pub total_stages: usize,
    pub passed_stages: usize,
    pub failed_stages: usize,
    pub skipped_stages: usize,
    pub total_duration: Duration,
    pub commit_hash: String,
    pub branch: String,
}

/// CI/CD 管理器
#[derive(Default)]
pub struct CicdManager {
    pipelines: Vec<CicdPipeline>,
    max_pipelines: usize,
}

impl CicdManager {
    pub fn new() -> Self {
        Self {
            pipelines: Vec::new(),
            max_pipelines: 100,
        }
    }

    /// 创建新流水线
    pub fn create_pipeline(
        &mut self,
        commit_hash: impl Into<String>,
        branch: impl Into<String>,
    ) -> String {
        let mut pipeline = CicdPipeline::new(commit_hash, branch);
        let id = pipeline.get_id().to_string();

        self.pipelines.push(pipeline);

        if self.pipelines.len() > self.max_pipelines {
            self.pipelines.remove(0);
        }

        id
    }

    /// 获取流水线
    pub fn get_pipeline(&self, pipeline_id: &str) -> Option<&CicdPipeline> {
        self.pipelines.iter().find(|p| p.get_id() == pipeline_id)
    }

    /// 获取可变流水线
    pub fn get_pipeline_mut(&mut self, pipeline_id: &str) -> Option<&mut CicdPipeline> {
        self.pipelines
            .iter_mut()
            .find(|p| p.get_id() == pipeline_id)
    }

    /// 获取最后一个流水线
    pub fn get_latest_pipeline(&self) -> Option<&CicdPipeline> {
        self.pipelines.last()
    }

    /// 获取成功的流水线
    pub fn get_passed_pipelines(&self) -> Vec<&CicdPipeline> {
        self.pipelines
            .iter()
            .filter(|p| p.get_status() == PipelineStatus::Passed)
            .collect()
    }

    /// 获取失败的流水线
    pub fn get_failed_pipelines(&self) -> Vec<&CicdPipeline> {
        self.pipelines
            .iter()
            .filter(|p| p.get_status() == PipelineStatus::Failed)
            .collect()
    }

    /// 获取分支的最新流水线
    pub fn get_latest_pipeline_for_branch(&self, branch: &str) -> Option<&CicdPipeline> {
        self.pipelines.iter().rev().find(|p| p.branch == branch)
    }

    /// 统计信息
    pub fn get_statistics(&self) -> CicdStatistics {
        let total = self.pipelines.len();
        let passed = self
            .pipelines
            .iter()
            .filter(|p| p.get_status() == PipelineStatus::Passed)
            .count();
        let failed = self
            .pipelines
            .iter()
            .filter(|p| p.get_status() == PipelineStatus::Failed)
            .count();
        let running = self
            .pipelines
            .iter()
            .filter(|p| p.get_status() == PipelineStatus::Running)
            .count();

        CicdStatistics {
            total_pipelines: total,
            passed_pipelines: passed,
            failed_pipelines: failed,
            running_pipelines: running,
            success_rate: if total > 0 {
                (passed as f64 / total as f64) * 100.0
            } else {
                100.0
            },
        }
    }
}

/// CI/CD 统计
#[derive(Debug, Clone)]
pub struct CicdStatistics {
    pub total_pipelines: usize,
    pub passed_pipelines: usize,
    pub failed_pipelines: usize,
    pub running_pipelines: usize,
    pub success_rate: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage_result() {
        let stage = StageResult::new(CicdStage::Build);
        assert_eq!(stage.status, StageStatus::Pending);
        assert_eq!(stage.stage, CicdStage::Build);
    }

    #[test]
    fn test_stage_status() {
        assert!(StageStatus::Passed.is_terminal());
        assert!(StageStatus::Failed.is_terminal());
        assert!(!StageStatus::Running.is_terminal());
    }

    #[test]
    fn test_cicd_pipeline_creation() {
        let mut pipeline = CicdPipeline::new("abc123", "main");
        assert_eq!(pipeline.commit_hash, "abc123");
        assert_eq!(pipeline.branch, "main");

        pipeline.add_stage(CicdStage::Build);
        pipeline.add_stage(CicdStage::UnitTest);
        assert_eq!(pipeline.stages.len(), 2);
    }

    #[test]
    fn test_pipeline_status() {
        let mut pipeline = CicdPipeline::new("commit1", "main");
        pipeline.add_stage(CicdStage::Build);

        assert_eq!(pipeline.get_status(), PipelineStatus::Pending);

        let stage_result = pipeline.stages.first_mut().unwrap();
        stage_result.status = StageStatus::Passed;

        assert_eq!(pipeline.get_status(), PipelineStatus::Passed);
    }

    #[test]
    fn test_cicd_manager() {
        let mut manager = CicdManager::new();

        let id1 = manager.create_pipeline("abc123", "main");
        let id2 = manager.create_pipeline("def456", "dev");

        assert_eq!(manager.pipelines.len(), 2);
        assert!(manager.get_pipeline(&id1).is_some());
        assert!(manager.get_latest_pipeline_for_branch("main").is_some());
    }

    #[test]
    fn test_cicd_statistics() {
        let mut manager = CicdManager::new();

        manager.create_pipeline("commit1", "main");
        manager.create_pipeline("commit2", "main");

        let stats = manager.get_statistics();
        assert_eq!(stats.total_pipelines, 2);
    }
}
