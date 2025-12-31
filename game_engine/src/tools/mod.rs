//! # 开发工具模块
//!
//! 各种开发和调试工具集合。

pub mod resource_analysis;

pub mod migration;
pub mod ai_assistant;

// 重新导出主要类型
pub use resource_analysis::{
    ResourceDependencyGraph,
    ResourceNode,
    ResourceReferenceAnalyzer,
    ResourceScanner,
    ResourceType,
    UnusedResourceDetector,
    RedundantAssetCleaner,
    DependencyReportGenerator,
    analyze_project_resources,
    AnalysisResult,
    UsageStats,
};
