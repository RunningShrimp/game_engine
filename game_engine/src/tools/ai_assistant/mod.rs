//! # AI辅助工具
//!
//! 基于AI的代码生成、审查和测试生成工具。
//!
//! ## 功能特性
//!
//! - **代码生成**: AI辅助代码生成
//! - **代码审查**: 自动代码审查和优化建议
//! - **测试生成**: 自动生成单元测试
//! - **LSP集成**: 代码补全和提示

use crate::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod code_analysis;
pub mod code_generation;
pub mod code_review;
pub mod test_generation;

pub use code_analysis::{
    AICodeAnalyzer, BottleneckSeverity, BottleneckType, CodeMetrics, CodeQualityIssue,
    CodeQualityReport, ComplexityMetrics, DependencyAnalysis, MemoryUsageAnalysis,
    PerformanceAnalysis, PerformanceBottleneck, QualityCategory, RefactoringEffort,
    RefactoringPriority, RefactoringRisk, RefactoringSuggestion, RefactoringSuggestions,
    RefactoringType,
};
pub use code_generation::{AICodeGenerator, CodeGenerationRequest};
pub use code_review::{AICodeReviewer, CodeReviewIssue, CodeReviewReport};
pub use test_generation::{AITestGenerator, TestGenerationResult};

// =============================================================================
// AI提供者
// =============================================================================

/// AI提供者
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AIProvider {
    /// OpenAI (GPT-4)
    OpenAI,
    /// Anthropic (Claude)
    Anthropic,
    /// 本地模型
    Local,
    /// 其他
    Other,
}

/// AI配置
#[derive(Debug, Clone)]
pub struct AIConfig {
    /// 提供者
    pub provider: AIProvider,
    /// API密钥
    pub api_key: String,
    /// 模型名称
    pub model: String,
    /// 最大token数
    pub max_tokens: u32,
    /// 温度 (0.0 - 1.0)
    pub temperature: f32,
}

impl Default for AIConfig {
    fn default() -> Self {
        Self {
            provider: AIProvider::OpenAI,
            api_key: String::new(),
            model: "gpt-4".to_string(),
            max_tokens: 2048,
            temperature: 0.7,
        }
    }
}

// =============================================================================
// AI助手
// =============================================================================

/// AI助手
pub struct AIAssistant {
    /// 配置
    config: AIConfig,
    /// 代码生成器
    code_generator: AICodeGenerator,
    /// 代码审查器
    code_reviewer: AICodeReviewer,
    /// 测试生成器
    test_generator: AITestGenerator,
}

impl AIAssistant {
    /// 创建新助手
    pub fn new(config: AIConfig) -> Self {
        Self {
            config: config.clone(),
            code_generator: AICodeGenerator::new(config.clone()),
            code_reviewer: AICodeReviewer::new(config.clone()),
            test_generator: AITestGenerator::new(config),
        }
    }

    /// 生成代码
    pub async fn generate_code(&self, request: CodeGenerationRequest) -> Result<String, AIError> {
        self.code_generator.generate(request).await
    }

    /// 审查代码
    pub async fn review_code(
        &self,
        code: &str,
        language: &str,
    ) -> Result<CodeReviewReport, AIError> {
        self.code_reviewer.review(code, language).await
    }

    /// 生成测试
    pub async fn generate_test(&self, source_code: &str) -> Result<TestGenerationResult, AIError> {
        self.test_generator.generate(source_code).await
    }

    /// 获取配置
    pub fn config(&self) -> &AIConfig {
        &self.config
    }
}

/// AI错误
#[derive(Debug, Clone)]
pub enum AIError {
    /// API错误
    ApiError(String),
    /// 网络错误
    NetworkError(String),
    /// 解析错误
    ParseError(String),
    /// 配额限制
    RateLimited,
    /// 无效API密钥
    InvalidApiKey,
    /// 其他错误
    Other(String),
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIError::ApiError(msg) => write!(f, "API error: {}", msg),
            AIError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            AIError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AIError::RateLimited => write!(f, "Rate limited"),
            AIError::InvalidApiKey => write!(f, "Invalid API key"),
            AIError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for AIError {}

// =============================================================================
// AI事件
// =============================================================================

/// AI事件
#[derive(Debug, Clone)]
pub enum AIEvent {
    /// 代码生成完成
    CodeGenerated { request_id: String, code: String },
    /// 代码审查完成
    CodeReviewed {
        file_path: PathBuf,
        report: CodeReviewReport,
    },
    /// 测试生成完成
    TestGenerated {
        file_path: PathBuf,
        result: TestGenerationResult,
    },
    /// AI请求失败
    RequestFailed { error: AIError },
}

impl DomainEvent for AIEvent {
    fn event_type(&self) -> &'static str {
        match self {
            AIEvent::CodeGenerated { .. } => "CodeGenerated",
            AIEvent::CodeReviewed { .. } => "CodeReviewed",
            AIEvent::TestGenerated { .. } => "TestGenerated",
            AIEvent::RequestFailed { .. } => "RequestFailed",
        }
    }

    fn apply(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn revert(&self, _world: &mut World) -> Result<(), EventError> {
        Ok(())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

// =============================================================================
// ECS集成
// =============================================================================

/// AI助手资源
#[derive(Resource)]
pub struct AIAssistantResource {
    pub assistant: AIAssistant,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = AIConfig::default();
        assert_eq!(config.provider, AIProvider::OpenAI);
        assert_eq!(config.model, "gpt-4");
    }

    #[test]
    fn test_assistant_creation() {
        let config = AIConfig::default();
        let assistant = AIAssistant::new(config);
        assert_eq!(assistant.config().model, "gpt-4");
    }
}
