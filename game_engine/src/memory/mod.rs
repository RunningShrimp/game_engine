//! # 内存管理模块（Memory Management）
//!
//! 本模块提供智能内存管理和分析功能。

pub mod memory_advisor;

pub use memory_advisor::{
    AllocationId, AllocationInfo, AllocationStats, CategoryUsage, LeakInfo, LeakReport,
    LeakSeverity, MemoryAdvisor, MemoryAdvisorConfig, MemoryAdvisorResource, MemoryCategory,
    MemoryEvent, MemoryPressure, MemorySnapshot, MemoryStats, OptimizationSuggestion, RiskLevel,
    SuggestionId, SuggestionPriority, SuggestionType,
};
