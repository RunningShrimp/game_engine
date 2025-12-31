//! # 开发工具模块
//!
//! 各种开发和调试工具集合。

pub mod resource_analysis;

pub mod ai_assistant;
pub mod migration;

// 重新导出主要类型
pub use resource_analysis::{
    AnalysisResult, DependencyReportGenerator, RedundantAssetCleaner, ResourceDependencyGraph,
    ResourceNode, ResourceReferenceAnalyzer, ResourceScanner, ResourceType, UnusedResourceDetector,
    UsageStats, analyze_project_resources,
};
