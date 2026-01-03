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
//!
//! ## 配置管理
//!
//! AI工具支持从环境变量或配置文件加载配置：
//!
//! ```bash
//! # OpenAI配置
//! export OPENAI_API_KEY="your-key"
//! export OPENAI_MODEL="gpt-4"
//!
//! # Anthropic配置
//! export ANTHROPIC_API_KEY="your-key"
//! export ANTHROPIC_MODEL="claude-3-opus-20240229"
//!
//! # 本地模型配置
//! export LOCAL_MODEL_ENDPOINT="http://localhost:11434/api/generate"
//! export LOCAL_MODEL_NAME="llama2"
//! ```

use crate::domain::events::{DomainEvent, EventError};
use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

pub mod code_analysis;
pub mod code_generation;
pub mod code_review;
pub mod test_generation;

pub use code_analysis::{
    AICodeAnalyzer, BottleneckSeverity, BottleneckType, CodeMetrics, CodeQualityIssue,
    CodeQualityReport, ComplexityMetrics, DependencyAnalysis,
    IssueSeverity as AnalysisIssueSeverity, MemoryUsageAnalysis, PerformanceAnalysis,
    PerformanceBottleneck, QualityCategory, QualityIssue, RefactoringEffort, RefactoringPriority,
    RefactoringRisk, RefactoringSuggestion, RefactoringSuggestions, RefactoringType,
};
pub use code_generation::{
    AICodeGenerator, CodeGenerationRequest, CodeOptimizationResult, ProviderInfo,
};
pub use code_review::{
    AICodeReviewer, BestPracticeReport, CodeReviewIssue, CodeReviewReport, IssueCategory,
    IssueSeverity, StyleReport,
};
pub use test_generation::{
    AITestGenerator, TestCaseRecommendation, TestCoverageReport, TestGenerationResult, TestPriority,
};

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
    /// API端点
    pub api_endpoint: Option<String>,
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
            api_endpoint: Some("https://api.openai.com/v1/chat/completions".to_string()),
            max_tokens: 2048,
            temperature: 0.7,
        }
    }
}

impl AIConfig {
    /// 从环境变量加载配置
    pub fn from_env(provider: AIProvider) -> Result<Self, String> {
        let (api_key, model, endpoint) = match provider {
            AIProvider::OpenAI => (
                env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set".to_string())?,
                env::var("OPENAI_MODEL").unwrap_or("gpt-4".to_string()),
                env::var("OPENAI_ENDPOINT").ok(),
            ),
            AIProvider::Anthropic => (
                env::var("ANTHROPIC_API_KEY")
                    .map_err(|_| "ANTHROPIC_API_KEY not set".to_string())?,
                env::var("ANTHROPIC_MODEL").unwrap_or("claude-3-opus-20240229".to_string()),
                env::var("ANTHROPIC_ENDPOINT").ok(),
            ),
            AIProvider::Local => (
                String::new(), // 本地模型不需要API密钥
                env::var("LOCAL_MODEL_NAME").unwrap_or("llama2".to_string()),
                Some(
                    env::var("LOCAL_MODEL_ENDPOINT")
                        .unwrap_or("http://localhost:11434/api/generate".to_string()),
                ),
            ),
            AIProvider::Other => {
                return Err("Unsupported provider".to_string());
            }
        };

        Ok(Self {
            provider,
            api_key,
            model,
            api_endpoint: endpoint,
            ..Default::default()
        })
    }

    /// 创建OpenAI配置
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self {
            provider: AIProvider::OpenAI,
            api_key: api_key.into(),
            model: "gpt-4".to_string(),
            api_endpoint: None,
            ..Default::default()
        }
    }

    /// 创建Anthropic配置
    pub fn anthropic(api_key: impl Into<String>) -> Self {
        Self {
            provider: AIProvider::Anthropic,
            api_key: api_key.into(),
            model: "claude-3-opus-20240229".to_string(),
            api_endpoint: None,
            ..Default::default()
        }
    }

    /// 创建本地模型配置
    pub fn local(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: AIProvider::Local,
            api_key: String::new(),
            model: model.into(),
            api_endpoint: Some(endpoint.into()),
            ..Default::default()
        }
    }

    /// 设置模型
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    /// 设置温度
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.max(0.0).min(1.0);
        self
    }

    /// 设置最大token数
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// 设置API端点
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.api_endpoint = Some(endpoint.into());
        self
    }

    /// 验证配置
    pub fn validate(&self) -> Result<(), String> {
        match self.provider {
            AIProvider::OpenAI | AIProvider::Anthropic => {
                if self.api_key.is_empty() {
                    return Err(format!("{:?} API key is required", self.provider));
                }
            }
            AIProvider::Local => {
                if self.api_endpoint.is_none() {
                    return Err("Local model endpoint is required".to_string());
                }
            }
            AIProvider::Other => {
                return Err("Unsupported provider".to_string());
            }
        }
        Ok(())
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
    Network(String),
    /// 解析错误
    Parse(String),
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
            AIError::Network(msg) => write!(f, "Network error: {}", msg),
            AIError::Parse(msg) => write!(f, "Parse error: {}", msg),
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
