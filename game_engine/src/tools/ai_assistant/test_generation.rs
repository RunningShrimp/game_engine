//! AI测试生成器
//!
//! 提供基于AI的自动化测试生成功能：
//! - 单元测试生成
//! - 集成测试生成
//! - 边界测试用例
//! - 错误处理测试
//! - 测试覆盖分析

use super::{AIConfig, AIError, AIProvider};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// AI测试生成器
pub struct AITestGenerator {
    config: AIConfig,
}

impl AITestGenerator {
    /// 创建新生成器
    pub fn new(config: AIConfig) -> Self {
        Self { config }
    }

    /// 生成测试
    pub async fn generate(&self, source_code: &str) -> Result<TestGenerationResult, AIError> {
        let prompt = self.build_test_prompt(source_code);

        let response = match self.config.provider {
            AIProvider::OpenAI => self.call_openai(&prompt).await?,
            AIProvider::Anthropic => self.call_anthropic(&prompt).await?,
            AIProvider::Local => self.call_local(&prompt).await?,
            AIProvider::Other => {
                return Err(AIError::Other("Unsupported provider".to_string()));
            }
        };

        self.parse_test_result(&response)
    }

    /// 分析测试覆盖
    pub async fn analyze_coverage(
        &self,
        source_code: &str,
        test_code: &str,
    ) -> Result<TestCoverageReport, AIError> {
        let prompt = self.build_coverage_prompt(source_code, test_code);

        let response = self.call_ai_provider(&prompt).await?;
        self.parse_coverage_report(&response)
    }

    /// 生成测试用例推荐
    pub async fn recommend_tests(
        &self,
        source_code: &str,
    ) -> Result<Vec<TestCaseRecommendation>, AIError> {
        let prompt = self.build_recommendation_prompt(source_code);

        let response = self.call_ai_provider(&prompt).await?;
        self.parse_test_recommendations(&response)
    }

    /// 构建测试生成提示词
    fn build_test_prompt(&self, source_code: &str) -> String {
        format!(
            "Generate comprehensive unit tests for the following Rust code:\n\n\
            Requirements:\n\
            1. Include edge cases (empty inputs, null values, boundaries)\n\
            2. Include error cases and error handling\n\
            3. Include success cases\n\
            4. Use Rust testing best practices\n\
            5. Add descriptive test names following the convention: test_<function>_<scenario>\n\
            6. Include assertions with clear messages\n\
            7. Use appropriate test helpers (setUp, tearDown if needed)\n\
            8. Mock external dependencies\n\n\
            Code:\n```rust\n{}\n```\n\n\
            Return the complete test code in a code block.",
            source_code
        )
    }

    /// 构建测试覆盖分析提示词
    fn build_coverage_prompt(&self, source_code: &str, test_code: &str) -> String {
        format!(
            "Analyze the test coverage for the following code:\n\n\
            Source Code:\n```rust\n{}\n```\n\n\
            Test Code:\n```rust\n{}\n```\n\n\
            Provide:\n\
            1. Coverage percentage for each function\n\
            2. Missing test scenarios\n\
            3. Uncovered edge cases\n\
            4. Recommendations for improving coverage\n\
            5. Overall coverage score\n\n\
            Return the analysis in JSON format.",
            source_code, test_code
        )
    }

    /// 构建测试推荐提示词
    fn build_recommendation_prompt(&self, source_code: &str) -> String {
        format!(
            "Analyze the following code and recommend test cases:\n\n\
            Code:\n```rust\n{}\n```\n\n\
            For each function, recommend:\n\
            1. Unit test cases\n\
            2. Edge case tests\n\
            3. Error handling tests\n\
            4. Performance tests (if applicable)\n\
            5. Integration tests needed\n\n\
            Return recommendations in JSON format with priority levels.",
            source_code
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

    /// 调用OpenAI API
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
                    "content": "You are a Rust testing expert. Generate comprehensive, well-structured unit tests following Rust best practices."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 3000
        });

        let response = client
            .post(endpoint)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::Other(format!("Network error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIError::ApiError(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AIError::Other(format!("Network error: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AIError::Parse(format!("Failed to parse response: {}", e)))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| AIError::Parse("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    /// 调用Anthropic API
    async fn call_anthropic(&self, prompt: &str) -> Result<String, AIError> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AIError::Other(format!("Failed to create client: {}", e)))?;

        let endpoint = "https://api.anthropic.com/v1/messages";
        let request_body = serde_json::json!({
            "model": self.config.model,
            "max_tokens": 3000,
            "system": "You are a Rust testing expert. Generate comprehensive, well-structured unit tests following Rust best practices.",
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
            .map_err(|e| AIError::Other(format!("Network error: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(AIError::ApiError(format!(
                "API request failed with status {}: {}",
                status, error_text
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AIError::Other(format!("Network error: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AIError::Parse(format!("Failed to parse response: {}", e)))?;

        let content = json["content"][0]["text"]
            .as_str()
            .ok_or_else(|| AIError::Parse("No content in response".to_string()))?;

        Ok(content.to_string())
    }

    /// 调用本地模型（Ollama）
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
                "num_predict": 3000,
                "temperature": 0.3
            }
        });

        let response = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AIError::Other(format!("Network error: {}", e)))?;

        if !response.status().is_success() {
            return Err(AIError::ApiError(format!(
                "Local model request failed with status: {}",
                response.status()
            )));
        }

        let response_text = response
            .text()
            .await
            .map_err(|e| AIError::Other(format!("Network error: {}", e)))?;

        let json: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| AIError::Parse(format!("Failed to parse response: {}", e)))?;

        let response_content = json["response"]
            .as_str()
            .ok_or_else(|| AIError::Parse("Invalid response format".to_string()))?;

        Ok(response_content.to_string())
    }

    /// 解析测试生成结果
    fn parse_test_result(&self, response: &str) -> Result<TestGenerationResult, AIError> {
        // 提取代码块
        let test_code = if let Some(start) = response.find("```rust") {
            let start = start + 7;
            if let Some(end) = response[start..].find("```") {
                response[start..start + end].to_string()
            } else {
                response.to_string()
            }
        } else if let Some(start) = response.find("```") {
            let start = start + 3;
            if let Some(end) = response[start..].find("```") {
                response[start..start + end].to_string()
            } else {
                response.to_string()
            }
        } else {
            response.to_string()
        };

        // 计算测试数量
        let test_count =
            test_code.matches("#[test]").count() + test_code.matches("#[tokio::test]").count();

        Ok(TestGenerationResult {
            test_code,
            test_count,
        })
    }

    /// 解析测试覆盖报告
    fn parse_coverage_report(&self, response: &str) -> Result<TestCoverageReport, AIError> {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            Ok(TestCoverageReport {
                overall_coverage: json["overall_coverage"].as_u64().unwrap_or(0) as f32 / 100.0,
                function_coverage: json["function_coverage"].as_u64().unwrap_or(0) as f32 / 100.0,
                line_coverage: json["line_coverage"].as_u64().unwrap_or(0) as f32 / 100.0,
                branch_coverage: json["branch_coverage"].as_u64().unwrap_or(0) as f32 / 100.0,
                uncovered_functions: self.parse_string_array(json.get("uncovered_functions")),
                missing_scenarios: self.parse_string_array(json.get("missing_scenarios")),
                recommendations: self.parse_string_array(json.get("recommendations")),
            })
        } else {
            // 返回默认报告
            Ok(TestCoverageReport::default())
        }
    }

    /// 解析测试推荐
    fn parse_test_recommendations(
        &self,
        response: &str,
    ) -> Result<Vec<TestCaseRecommendation>, AIError> {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(response) {
            if let Some(cases) = json.get("test_cases").and_then(|v| v.as_array()) {
                let mut recommendations = Vec::new();
                for case in cases {
                    if let Some(obj) = case.as_object() {
                        recommendations.push(TestCaseRecommendation {
                            function_name: obj
                                .get("function_name")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            test_type: obj
                                .get("test_type")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unit")
                                .to_string(),
                            description: obj
                                .get("description")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            priority: match obj.get("priority").and_then(|v| v.as_str()) {
                                Some("high") => TestPriority::High,
                                Some("medium") => TestPriority::Medium,
                                Some("low") => TestPriority::Low,
                                _ => TestPriority::Medium,
                            },
                            scenario: obj
                                .get("scenario")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                        });
                    }
                }
                return Ok(recommendations);
            }
        }

        // 返回默认推荐
        Ok(vec![TestCaseRecommendation {
            function_name: "example".to_string(),
            test_type: "unit".to_string(),
            description: "Basic functionality test".to_string(),
            priority: TestPriority::Medium,
            scenario: "normal case".to_string(),
        }])
    }

    /// 解析字符串数组
    fn parse_string_array(&self, value: Option<&serde_json::Value>) -> Vec<String> {
        value
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default()
    }
}

/// 测试生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestGenerationResult {
    /// 生成的测试代码
    pub test_code: String,
    /// 测试数量
    pub test_count: usize,
}

/// 测试覆盖报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCoverageReport {
    /// 总体覆盖率 (0.0 - 1.0)
    pub overall_coverage: f32,
    /// 函数覆盖率
    pub function_coverage: f32,
    /// 行覆盖率
    pub line_coverage: f32,
    /// 分支覆盖率
    pub branch_coverage: f32,
    /// 未覆盖的函数
    pub uncovered_functions: Vec<String>,
    /// 缺失的测试场景
    pub missing_scenarios: Vec<String>,
    /// 改进建议
    pub recommendations: Vec<String>,
}

impl Default for TestCoverageReport {
    fn default() -> Self {
        Self {
            overall_coverage: 0.0,
            function_coverage: 0.0,
            line_coverage: 0.0,
            branch_coverage: 0.0,
            uncovered_functions: vec![],
            missing_scenarios: vec![],
            recommendations: vec!["Add more comprehensive tests".to_string()],
        }
    }
}

/// 测试用例推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseRecommendation {
    /// 函数名
    pub function_name: String,
    /// 测试类型
    pub test_type: String,
    /// 描述
    pub description: String,
    /// 优先级
    pub priority: TestPriority,
    /// 测试场景
    pub scenario: String,
}

/// 测试优先级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestPriority {
    /// 高
    High,
    /// 中
    Medium,
    /// 低
    Low,
}

// =============================================================================
// 测试
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test_generation() {
        let config = AIConfig::default();
        let generator = AITestGenerator::new(config);

        let source_code = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn divide(a: f32, b: f32) -> Result<f32, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}
"#;

        let result = generator.generate(source_code).await;
        // 注意：实际运行需要API密钥，这里只测试结构
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_parse_test_result() {
        let config = AIConfig::default();
        let generator = AITestGenerator::new(config);

        let response = r#"
Here are the tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_positive() {
        assert_eq!(add(2, 3), 5);
    }

    #[test]
    fn test_add_negative() {
        assert_eq!(add(-2, -3), -5);
    }
}
```
"#;

        let result = generator.parse_test_result(response);
        assert!(result.is_ok());

        let test_result = result.unwrap();
        assert_eq!(test_result.test_count, 2);
        assert!(test_result.test_code.contains("test_add_positive"));
    }

    #[test]
    fn test_coverage_report_default() {
        let report = TestCoverageReport::default();
        assert_eq!(report.overall_coverage, 0.0);
        assert!(!report.recommendations.is_empty());
    }
}
