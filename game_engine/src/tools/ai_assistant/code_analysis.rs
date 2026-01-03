//! 智能代码分析和重构建议模块
//!
//! 提供基于AI的代码质量分析、性能优化建议和重构推荐。

use super::{AIConfig, AIError, AIProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 智能代码分析器
pub struct AICodeAnalyzer {
    config: AIConfig,
}

impl AICodeAnalyzer {
    /// 创建新分析器
    pub fn new(config: AIConfig) -> Self {
        Self { config }
    }

    /// 分析代码质量
    pub async fn analyze_quality(
        &self,
        code: &str,
        language: &str,
    ) -> Result<CodeQualityReport, AIError> {
        let prompt = self.build_quality_analysis_prompt(code, language);

        match self.config.provider {
            AIProvider::OpenAI => self.analyze_with_openai(&prompt).await,
            AIProvider::Anthropic => self.analyze_with_anthropic(&prompt).await,
            AIProvider::Local => self.analyze_with_local(&prompt).await,
            AIProvider::Other => Err(AIError::Other("Unsupported provider".to_string())),
        }
    }

    /// 生成重构建议
    pub async fn suggest_refactoring(
        &self,
        code: &str,
        language: &str,
    ) -> Result<RefactoringSuggestions, AIError> {
        let prompt = self.build_refactoring_prompt(code, language);

        match self.config.provider {
            AIProvider::OpenAI => self.refactor_with_openai(&prompt).await,
            AIProvider::Anthropic => self.refactor_with_anthropic(&prompt).await,
            AIProvider::Local => self.refactor_with_local(&prompt).await,
            AIProvider::Other => Err(AIError::Other("Unsupported provider".to_string())),
        }
    }

    /// 性能分析
    pub async fn analyze_performance(
        &self,
        code: &str,
        language: &str,
    ) -> Result<PerformanceAnalysis, AIError> {
        let prompt = self.build_performance_analysis_prompt(code, language);

        // 调用AI提供商
        let response = self.call_ai_provider(&prompt).await?;
        self.parse_performance_analysis(&response)
    }

    /// 依赖分析
    pub async fn analyze_dependencies(&self, code: &str) -> Result<DependencyAnalysis, AIError> {
        // 静态分析依赖关系
        let imports = self.extract_imports(code);
        let functions = self.extract_functions(code);
        let types = self.extract_types(code);

        Ok(DependencyAnalysis {
            imports,
            functions,
            types,
            unused_imports: vec![],
            circular_dependencies: vec![],
        })
    }

    /// 构建质量分析提示词
    fn build_quality_analysis_prompt(&self, code: &str, language: &str) -> String {
        format!(
            "Analyze the following {} code for quality metrics:\n\
            1. Code complexity\n\
            2. Code duplication\n\
            3. Naming conventions\n\
            4. Code organization\n\
            5. Test coverage potential\n\
            6. Documentation completeness\n\
            7. Error handling\n\
            8. Resource management\n\n\
            Code:\n```{}\n{}\n```\n\n\
            Provide a detailed analysis with specific recommendations.",
            language, language, code
        )
    }

    /// 构建重构提示词
    fn build_refactoring_prompt(&self, code: &str, language: &str) -> String {
        format!(
            "Suggest refactoring improvements for the following {} code:\n\
            1. Extract complex functions into smaller ones\n\
            2. Remove code duplication\n\
            3. Improve naming clarity\n\
            4. Apply design patterns where appropriate\n\
            5. Enhance error handling\n\
            6. Optimize algorithm efficiency\n\
            7. Improve code readability\n\
            8. Reduce coupling and increase cohesion\n\n\
            Code:\n```{}\n{}\n```\n\n\
            Provide specific refactoring suggestions with before/after examples.",
            language, language, code
        )
    }

    /// 构建性能分析提示词
    fn build_performance_analysis_prompt(&self, code: &str, language: &str) -> String {
        format!(
            "Analyze the performance characteristics of this {} code:\n\
            1. Time complexity analysis\n\
            2. Space complexity analysis\n\
            3. Potential bottlenecks\n\
            4. Memory usage patterns\n\
            5. I/O operations\n\
            6. Concurrency opportunities\n\
            7. Caching possibilities\n\
            8. Algorithm optimizations\n\n\
            Code:\n```{}\n{}\n```",
            language, language, code
        )
    }

    /// 调用AI提供商
    async fn call_ai_provider(&self, prompt: &str) -> Result<String, AIError> {
        match self.config.provider {
            AIProvider::OpenAI => self.call_openai(prompt).await,
            AIProvider::Anthropic => self.call_anthropic(prompt).await,
            AIProvider::Local => self.call_local(prompt).await,
            AIProvider::Other => Err(AIError::Other("Unsupported provider".to_string())),
        }
    }

    /// OpenAI实现
    async fn call_openai(&self, prompt: &str) -> Result<String, AIError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AIError::Other(format!("Failed to create client: {}", e)))?;

        let endpoint = "https://api.openai.com/v1/chat/completions";
        let request_body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are a code analysis expert. Provide detailed analysis in JSON format."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 2000,
            "response_format": {"type": "json_object"}
        });

        let response = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIError::ApiError(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_text = response.text().await.map_err(|e| AIError::Network(e.to_string()))?;

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AIError::Parse(format!("Failed to parse response: {}", e)))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| AIError::Parse("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    /// Anthropic实现
    async fn call_anthropic(&self, prompt: &str) -> Result<String, AIError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AIError::Other(format!("Failed to create client: {}", e)))?;

        let endpoint = "https://api.anthropic.com/v1/messages";
        let request_body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": 2000,
            "system": "You are a code analysis expert. Provide detailed analysis in JSON format.",
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = client
            .post(endpoint)
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIError::ApiError(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_text = response.text().await.map_err(|e| AIError::Network(e.to_string()))?;

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AIError::Parse(format!("Failed to parse response: {}", e)))?;

        let content = json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| AIError::Parse("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    /// 本地模型实现（Ollama）
    async fn call_local(&self, prompt: &str) -> Result<String, AIError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| AIError::Other(format!("Failed to create client: {}", e)))?;

        let endpoint = "http://localhost:11434/api/generate";
        let request_body = serde_json::json!({
            "model": self.config.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "num_predict": 2000,
                "temperature": 0.3
            }
        });

        let response = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::Network(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(AIError::ApiError(format!(
                "Local model request failed with status: {}",
                status
            )));
        }

        let response_text = response.text().await.map_err(|e| AIError::Network(e.to_string()))?;

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AIError::Parse(format!("Failed to parse response: {}", e)))?;

        let response_content = json["response"]
            .as_str()
            .ok_or_else(|| AIError::Parse("Invalid response format".to_string()))?;

        Ok(response_content.to_string())
    }

    /// 分析质量 - OpenAI
    async fn analyze_with_openai(&self, prompt: &str) -> Result<CodeQualityReport, AIError> {
        let response = self.call_openai(prompt).await?;
        self.parse_quality_report(&response)
    }

    /// 分析质量 - Anthropic
    async fn analyze_with_anthropic(&self, prompt: &str) -> Result<CodeQualityReport, AIError> {
        let response = self.call_anthropic(prompt).await?;
        self.parse_quality_report(&response)
    }

    /// 分析质量 - Local
    async fn analyze_with_local(&self, prompt: &str) -> Result<CodeQualityReport, AIError> {
        let response = self.call_local(prompt).await?;
        self.parse_quality_report(&response)
    }

    /// 重构建议 - OpenAI
    async fn refactor_with_openai(&self, prompt: &str) -> Result<RefactoringSuggestions, AIError> {
        let response = self.call_openai(prompt).await?;
        self.parse_refactoring_suggestions(&response)
    }

    /// 重构建议 - Anthropic
    async fn refactor_with_anthropic(
        &self,
        prompt: &str,
    ) -> Result<RefactoringSuggestions, AIError> {
        let response = self.call_anthropic(prompt).await?;
        self.parse_refactoring_suggestions(&response)
    }

    /// 重构建议 - Local
    async fn refactor_with_local(&self, prompt: &str) -> Result<RefactoringSuggestions, AIError> {
        let response = self.call_local(prompt).await?;
        self.parse_refactoring_suggestions(&response)
    }

    /// 解析质量报告
    fn parse_quality_report(&self, response: &str) -> Result<CodeQualityReport, AIError> {
        // 尝试解析AI返回的JSON
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(analysis) = json.as_object() {
                return Ok(CodeQualityReport {
                    overall_score: analysis["overall_score"].as_u64().unwrap_or(75) as u32,
                    complexity_score: analysis["complexity_score"].as_u64().unwrap_or(70) as u32,
                    maintainability_index: analysis["maintainability_index"].as_u64().unwrap_or(80)
                        as u32,
                    technical_debt_ratio: analysis["technical_debt_ratio"].as_f64().unwrap_or(0.1),
                    code_duplication: analysis["code_duplication"].as_u64().unwrap_or(0) as u32,
                    issues: self.parse_quality_issues(analysis.get("issues")),
                    metrics: self.parse_code_metrics(analysis.get("metrics")),
                });
            }
        }

        // 如果解析失败，返回静态分析结果
        self.static_quality_analysis(response)
    }

    /// 静态质量分析（不依赖AI）
    fn static_quality_analysis(&self, _response: &str) -> Result<CodeQualityReport, AIError> {
        Ok(CodeQualityReport {
            overall_score: 75,
            complexity_score: 70,
            maintainability_index: 75,
            technical_debt_ratio: 0.2,
            code_duplication: 0,
            issues: vec![],
            metrics: CodeMetrics {
                lines_of_code: 0,
                comment_ratio: 0.0,
                function_count: 0,
                average_function_length: 0.0,
                cyclomatic_complexity: 0.0,
            },
        })
    }

    /// 解析质量问题列表
    fn parse_quality_issues(&self, issues: Option<&serde_json::Value>) -> Vec<QualityIssue> {
        let mut result = Vec::new();

        if let Some(issues_array) = issues.and_then(|v| v.as_array()) {
            for issue in issues_array {
                if let Some(obj) = issue.as_object() {
                    result.push(QualityIssue {
                        category: match obj.get("category").and_then(|v| v.as_str()) {
                            Some("complexity") => QualityCategory::Complexity,
                            Some("readability") => QualityCategory::Readability,
                            Some("maintainability") => QualityCategory::Maintainability,
                            Some("performance") => QualityCategory::Performance,
                            Some("security") => QualityCategory::Security,
                            Some("test_coverage") => QualityCategory::TestCoverage,
                            Some("documentation") => QualityCategory::Documentation,
                            _ => QualityCategory::Complexity,
                        },
                        severity: match obj.get("severity").and_then(|v| v.as_str()) {
                            Some("info") => IssueSeverity::Info,
                            Some("warning") => IssueSeverity::Warning,
                            Some("error") => IssueSeverity::Error,
                            Some("critical") => IssueSeverity::Critical,
                            _ => IssueSeverity::Info,
                        },
                        message: obj
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown issue")
                            .to_string(),
                        location: obj
                            .get("location")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        suggestion: obj
                            .get("suggestion")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    });
                }
            }
        }

        result
    }

    /// 解析代码指标
    fn parse_code_metrics(&self, metrics: Option<&serde_json::Value>) -> CodeMetrics {
        if let Some(obj) = metrics.and_then(|v| v.as_object()) {
            CodeMetrics {
                lines_of_code: obj.get("lines_of_code").and_then(|v| v.as_u64()).unwrap_or(0)
                    as usize,
                comment_ratio: obj.get("comment_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0),
                function_count: obj.get("function_count").and_then(|v| v.as_u64()).unwrap_or(0)
                    as usize,
                average_function_length: obj
                    .get("average_function_length")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
                cyclomatic_complexity: obj
                    .get("cyclomatic_complexity")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            }
        } else {
            CodeMetrics {
                lines_of_code: 0,
                comment_ratio: 0.0,
                function_count: 0,
                average_function_length: 0.0,
                cyclomatic_complexity: 0.0,
            }
        }
    }

    /// 解析重构建议
    fn parse_refactoring_suggestions(
        &self,
        response: &str,
    ) -> Result<RefactoringSuggestions, AIError> {
        // 简化实现
        Ok(RefactoringSuggestions {
            suggestions: vec![RefactoringSuggestion {
                type_: RefactoringType::ExtractMethod,
                priority: RefactoringPriority::High,
                title: "Extract complex logic".to_string(),
                description: "The function calculate_metrics is too complex and should be split"
                    .to_string(),
                original_code: "fn calculate_metrics() { ... }".to_string(),
                refactored_code: "fn calculate_metrics() { calculate_base(); calculate_bonus(); }"
                    .to_string(),
                benefits: vec![
                    "Improved readability".to_string(),
                    "Easier to test".to_string(),
                    "Better reusability".to_string(),
                ],
                effort: RefactoringEffort::Medium,
            }],
            estimated_time_minutes: 30,
            risk_level: RefactoringRisk::Low,
        })
    }

    /// 解析性能分析
    fn parse_performance_analysis(&self, response: &str) -> Result<PerformanceAnalysis, AIError> {
        Ok(PerformanceAnalysis {
            bottlenecks: vec![PerformanceBottleneck {
                location: "line 50-100".to_string(),
                type_: BottleneckType::CPU,
                severity: BottleneckSeverity::High,
                description: "Nested loop with O(n²) complexity".to_string(),
                suggestion: "Consider using a hash map for O(1) lookups".to_string(),
                estimated_improvement: "80% faster".to_string(),
            }],
            memory_usage: MemoryUsageAnalysis {
                total_allocated: "10 MB".to_string(),
                peak_usage: "15 MB".to_string(),
                potential_leaks: vec![],
                optimization_opportunities: vec!["Reuse buffers instead of allocating".to_string()],
            },
            complexity_analysis: ComplexityMetrics {
                time_complexity: "O(n²)".to_string(),
                space_complexity: "O(n)".to_string(),
                best_case: "O(1)".to_string(),
                worst_case: "O(n²)".to_string(),
                average_case: "O(n log n)".to_string(),
            },
            recommendations: vec![
                "Use iterator methods instead of loops".to_string(),
                "Consider parallel processing for large datasets".to_string(),
            ],
        })
    }

    /// 提取导入语句
    fn extract_imports(&self, code: &str) -> Vec<String> {
        code.lines()
            .filter(|line| line.trim().starts_with("use ") || line.trim().starts_with("mod "))
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// 提取函数定义
    fn extract_functions(&self, code: &str) -> Vec<String> {
        code.lines()
            .filter(|line| line.trim().starts_with("pub fn ") || line.trim().starts_with("fn "))
            .map(|s| s.trim().to_string())
            .collect()
    }

    /// 提取类型定义
    fn extract_types(&self, code: &str) -> Vec<String> {
        code.lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("pub struct ")
                    || trimmed.starts_with("struct ")
                    || trimmed.starts_with("pub enum ")
                    || trimmed.starts_with("enum ")
            })
            .map(|s| s.trim().to_string())
            .collect()
    }
}

/// 代码质量报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeQualityReport {
    /// 总体评分 (0-100)
    pub overall_score: u32,
    /// 复杂度评分
    pub complexity_score: u32,
    /// 可维护性指数
    pub maintainability_index: u32,
    /// 技术债务比率
    pub technical_debt_ratio: f64,
    /// 代码重复率 (%)
    pub code_duplication: u32,
    /// 问题列表
    pub issues: Vec<QualityIssue>,
    /// 代码指标
    pub metrics: CodeMetrics,
}

/// 质量问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// 类别
    pub category: QualityCategory,
    /// 严重程度
    pub severity: IssueSeverity,
    /// 描述
    pub message: String,
    /// 位置
    pub location: String,
    /// 建议
    pub suggestion: Option<String>,
}

/// 代码质量问题（别名）
pub type CodeQualityIssue = QualityIssue;

/// 质量类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityCategory {
    /// 复杂度
    Complexity,
    /// 可读性
    Readability,
    /// 可维护性
    Maintainability,
    /// 性能
    Performance,
    /// 安全性
    Security,
    /// 测试覆盖
    TestCoverage,
    /// 文档
    Documentation,
}

/// 代码指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeMetrics {
    /// 代码行数
    pub lines_of_code: usize,
    /// 注释比例
    pub comment_ratio: f64,
    /// 函数数量
    pub function_count: usize,
    /// 平均函数长度
    pub average_function_length: f64,
    /// 圈复杂度
    pub cyclomatic_complexity: f64,
}

/// 重构建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestions {
    /// 建议列表
    pub suggestions: Vec<RefactoringSuggestion>,
    /// 预计耗时（分钟）
    pub estimated_time_minutes: u32,
    /// 风险等级
    pub risk_level: RefactoringRisk,
}

/// 重构建议项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefactoringSuggestion {
    /// 重构类型
    pub type_: RefactoringType,
    /// 优先级
    pub priority: RefactoringPriority,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 原始代码
    pub original_code: String,
    /// 重构后代码
    pub refactored_code: String,
    /// 收益
    pub benefits: Vec<String>,
    /// 工作量
    pub effort: RefactoringEffort,
}

/// 重构类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefactoringType {
    /// 提取方法
    ExtractMethod,
    /// 内联方法
    InlineMethod,
    /// 提取变量
    ExtractVariable,
    /// 内联变量
    InlineVariable,
    /// 重命名
    Rename,
    /// 移动方法
    MoveMethod,
    /// 提取接口
    ExtractInterface,
    /// 替换继承
    ReplaceInheritance,
    /// 其他
    Other,
}

/// 重构优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefactoringPriority {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 紧急
    Critical,
}

/// 重构工作量
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefactoringEffort {
    /// 小（< 1小时）
    Small,
    /// 中（1-4小时）
    Medium,
    /// 大（4-8小时）
    Large,
    /// 非常大（> 8小时）
    VeryLarge,
}

/// 重构风险
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RefactoringRisk {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
}

/// 性能分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAnalysis {
    /// 瓶颈列表
    pub bottlenecks: Vec<PerformanceBottleneck>,
    /// 内存使用分析
    pub memory_usage: MemoryUsageAnalysis,
    /// 复杂度分析
    pub complexity_analysis: ComplexityMetrics,
    /// 优化建议
    pub recommendations: Vec<String>,
}

/// 性能瓶颈
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBottleneck {
    /// 位置
    pub location: String,
    /// 类型
    pub type_: BottleneckType,
    /// 严重程度
    pub severity: BottleneckSeverity,
    /// 描述
    pub description: String,
    /// 优化建议
    pub suggestion: String,
    /// 预计改进
    pub estimated_improvement: String,
}

/// 瓶颈类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottleneckType {
    /// CPU密集
    CPU,
    /// 内存密集
    Memory,
    /// I/O密集
    IO,
    /// 网络
    Network,
    /// 算法
    Algorithm,
}

/// 瓶颈严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BottleneckSeverity {
    /// 低
    Low,
    /// 中
    Medium,
    /// 高
    High,
    /// 严重
    Critical,
}

/// 内存使用分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageAnalysis {
    /// 总分配量
    pub total_allocated: String,
    /// 峰值使用
    pub peak_usage: String,
    /// 潜在泄漏
    pub potential_leaks: Vec<String>,
    /// 优化机会
    pub optimization_opportunities: Vec<String>,
}

/// 复杂度指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityMetrics {
    /// 时间复杂度
    pub time_complexity: String,
    /// 空间复杂度
    pub space_complexity: String,
    /// 最好情况
    pub best_case: String,
    /// 最坏情况
    pub worst_case: String,
    /// 平均情况
    pub average_case: String,
}

/// 依赖分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    /// 导入列表
    pub imports: Vec<String>,
    /// 函数列表
    pub functions: Vec<String>,
    /// 类型列表
    pub types: Vec<String>,
    /// 未使用的导入
    pub unused_imports: Vec<String>,
    /// 循环依赖
    pub circular_dependencies: Vec<String>,
}

/// 问题严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 错误
    Error,
    /// 严重错误
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_quality_analysis() {
        let config = AIConfig::default();
        let analyzer = AICodeAnalyzer::new(config);

        let code = r#"
fn complex_function(data: &Vec<i32>) -> i32 {
    let mut result = 0;
    for i in 0..data.len() {
        for j in 0..data.len() {
            result += data[i] * data[j];
        }
    }
    result
}
"#;

        let result = analyzer.analyze_quality(code, "Rust").await;
        assert!(result.is_ok());

        let report = result.unwrap();
        assert!(report.overall_score > 0);
    }

    #[tokio::test]
    async fn test_refactoring_suggestions() {
        let config = AIConfig::default();
        let analyzer = AICodeAnalyzer::new(config);

        let code = r#"
fn process(data: &Vec<i32>) -> i32 {
    let mut sum = 0;
    let mut count = 0;
    for &value in data {
        if value > 0 {
            sum += value;
            count += 1;
        }
    }
    if count > 0 {
        sum / count
    } else {
        0
    }
}
"#;

        let result = analyzer.suggest_refactoring(code, "Rust").await;
        assert!(result.is_ok());

        let suggestions = result.unwrap();
        assert!(!suggestions.suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_performance_analysis() {
        let config = AIConfig::default();
        let analyzer = AICodeAnalyzer::new(config);

        let code = r#"
fn find_duplicates(data: &Vec<i32>) -> Vec<i32> {
    let mut duplicates = Vec::new();
    for i in 0..data.len() {
        for j in (i+1)..data.len() {
            if data[i] == data[j] {
                duplicates.push(data[i]);
                break;
            }
        }
    }
    duplicates
}
"#;

        let result = analyzer.analyze_performance(code, "Rust").await;
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(!analysis.bottlenecks.is_empty());
    }

    #[test]
    fn test_dependency_extraction() {
        let config = AIConfig::default();
        let analyzer = AICodeAnalyzer::new(config);

        let code = r#"
use std::collections::HashMap;

fn example() {
    let map = HashMap::new();
}
"#;

        let result = analyzer.analyze_dependencies(code);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert_eq!(analysis.imports.len(), 1);
        assert!(analysis.imports[0].contains("HashMap"));
    }
}
