//! # Claude适配器
//!
//! 本模块提供Anthropic Claude API的适配器实现。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::ai::claude::ClaudeAdapter;
//!
//! let adapter = ClaudeAdapter::new(
//!     "your-api-key",
//!     "claude-3-opus-20240229"
//! );
//!
//! let dialogue = adapter.generate_dialogue(&context).await?;
//! ```

use super::service::{
    Action, AIError, AIService, ContentPrompt, ContentType, GeneratedContent, Message,
    NPCContext, Situation,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Claude API适配器
///
/// 提供对Anthropic Claude模型的访问，支持Claude 3系列模型。
pub struct ClaudeAdapter {
    /// API密钥
    api_key: String,
    /// 模型名称
    model: String,
    /// HTTP客户端
    client: Client,
    /// API基础URL
    base_url: String,
    /// 最大token数
    max_tokens: usize,
    /// 温度参数（0.0-1.0）
    temperature: f32,
}

/// Claude消息内容
#[derive(Debug, Serialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Claude API请求
#[derive(Debug, Serialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<ClaudeMessage>,
    max_tokens: usize,
    temperature: f32,
    system: Option<String>,
}

/// Claude消息
#[derive(Debug, Serialize)]
struct ClaudeMessage {
    role: String,
    content: Vec<ContentBlock>,
}

/// Claude API响应
#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<ResponseContent>,
    usage: Option<ClaudeUsage>,
}

/// Claude响应内容
#[derive(Debug, Deserialize)]
struct ResponseContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

/// Claude token使用情况
#[derive(Debug, Deserialize)]
struct ClaudeUsage {
    input_tokens: usize,
    output_tokens: usize,
}

impl ClaudeAdapter {
    /// 创建新的Claude适配器
    ///
    /// # 参数
    ///
    /// - `api_key`: Anthropic API密钥
    /// - `model`: 模型名称（如"claude-3-opus-20240229", "claude-3-sonnet-20240229"）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::ai::claude::ClaudeAdapter;
    ///
    /// let adapter = ClaudeAdapter::new(
    ///     "sk-ant-...",
    ///     "claude-3-opus-20240229"
    /// );
    /// ```
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            client: Client::builder()
                .timeout(Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
            base_url: "https://api.anthropic.com/v1/messages".to_string(),
            max_tokens: 150,
            temperature: 0.7,
        }
    }

    /// 设置API基础URL
    ///
    /// 用于使用自定义的Claude兼容API端点。
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// 设置最大token数
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }

    /// 设置温度参数
    ///
    /// 较高的值（如0.8）会使输出更随机，较低的值（如0.2）会使输出更确定。
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 1.0);
        self
    }

    /// 调用Claude API
    async fn call_claude(
        &self,
        messages: Vec<ClaudeMessage>,
        system_prompt: Option<String>,
    ) -> Result<ClaudeResponse, AIError> {
        let request = ClaudeRequest {
            model: self.model.clone(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            system: system_prompt,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::NetworkError(format!("Request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .map_err(|e| AIError::NetworkError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(if status.as_u16() == 401 {
                AIError::AuthenticationError
            } else if status.as_u16() == 429 {
                AIError::RateLimitError
            } else {
                AIError::ApiError(response_text)
            });
        }

        serde_json::from_str(&response_text)
            .map_err(|e| AIError::ParseError(format!("Failed to parse response: {}", e)))
    }

    /// 构建对话提示
    fn build_dialogue_messages(&self, context: &NPCContext) -> (Vec<ClaudeMessage>, String) {
        let system = self.build_system_prompt(context);
        let mut messages = Vec::new();

        // 添加历史对话
        for msg in &context.conversation_history {
            messages.push(ClaudeMessage {
                role: msg.role.clone(),
                content: vec![ContentBlock {
                    content_type: "text".to_string(),
                    text: msg.content.clone(),
                }],
            });
        }

        // 添加当前情境
        messages.push(ClaudeMessage {
            role: "user".to_string(),
            content: vec![ContentBlock {
                content_type: "text".to_string(),
                text: self.build_user_prompt(context),
            }],
        });

        (messages, system)
    }

    /// 构建系统提示
    fn build_system_prompt(&self, context: &NPCContext) -> String {
        format!(
            "You are an NPC in a game world. Your personality:\n\
             - Friendliness: {:.1}/1.0\n\
             - Formality: {:.1}/1.0\n\
             - Humor: {:.1}/1.0\n\
             - Bravery: {:.1}/1.0\n\
             - Greed: {:.1}/1.0\n\
             \n\
             Current emotional state:\n\
             - Happiness: {:.1}/1.0\n\
             - Anger: {:.1}/1.0\n\
             - Fear: {:.1}/1.0\n\
             - Trust: {:.1}/1.0\n\
             \n\
             Guidelines:\n\
             1. Stay in character according to your personality traits\n\
             2. Keep responses concise (under 100 words)\n\
             3. Reflect your current emotional state\n\
             4. Adapt your formality based on the friendliness level",
            context.personality.friendliness,
            context.personality.formality,
            context.personality.humor,
            context.personality.bravery,
            context.personality.greed,
            context.mood.happiness,
            context.mood.anger,
            context.mood.fear,
            context.mood.trust
        )
    }

    /// 构建用户提示
    fn build_user_prompt(&self, context: &NPCContext) -> String {
        format!(
            "Player Information:\n\
             - Level: {}\n\
             - Health: {:.0}/{:.0}\n\
             - Location: {}\n\
             - Time: {}\n\
             - Weather: {}\n\
             - In combat: {}\n\
             - Nearby: {:?}\n\
             \n\
             Please respond to the player appropriately based on this context.",
            context.player_state.level,
            context.player_state.health,
            context.player_state.max_health,
            context.environment.location,
            context.environment.game_time,
            context.environment.weather,
            context.environment.in_combat,
            context.environment.nearby_entities
        )
    }

    /// 构建决策提示
    fn build_decision_messages(&self, situation: &Situation) -> (Vec<ClaudeMessage>, String) {
        let system = "You are an AI NPC decision-making system. Analyze the situation and choose the best action. \
                      Consider the NPC's status, available actions, and threats."
            .to_string();

        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: vec![ContentBlock {
                content_type: "text".to_string(),
                text: format!(
                    "Situation Analysis:\n\
                     - NPC Status: {:?}\n\
                     - Nearby Entities: {}\n\
                     - Current Goal: {:?}\n\
                     - Available Actions: {}\n\
                     - Threats: {}\n\
                     - Resources: {:?}\n\
                     \n\
                     Select the most appropriate action based on this situation.",
                    situation.npc_status,
                    situation.nearby_entities.len(),
                    situation.current_goal,
                    situation.available_actions.len(),
                    situation.perceived_threats.len(),
                    situation.resources
                ),
            }],
        }];

        (messages, system)
    }

    /// 构建内容生成提示
    fn build_content_messages(&self, prompt: &ContentPrompt) -> (Vec<ClaudeMessage>, String) {
        let system = format!(
            "You are a creative game content generator specializing in {:?}. \
             Generate engaging, concise content that fits the game world.",
            prompt.content_type
        );

        let user_message = format!(
            "Task: {}\n\nConstraints:\n- {}\n\nStyle Guidelines: {}",
            prompt.prompt,
            prompt.constraints.join("\n- "),
            prompt.style.as_deref().unwrap_or("neutral")
        );

        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: vec![ContentBlock {
                content_type: "text".to_string(),
                text: user_message,
            }],
        }];

        (messages, system)
    }
}

#[async_trait]
impl AIService for ClaudeAdapter {
    async fn generate_dialogue(&self, context: &NPCContext) -> Result<String, AIError> {
        let (messages, system) = self.build_dialogue_messages(context);
        let response = self.call_claude(messages, Some(system)).await?;

        response
            .content
            .first()
            .map(|content| content.text.trim().to_string())
            .ok_or_else(|| AIError::ParseError("No content in response".to_string()))
    }

    async fn decide_action(&self, situation: &Situation) -> Result<Action, AIError> {
        let (messages, system) = self.build_decision_messages(situation);
        self.call_claude(messages, Some(system)).await?;

        // 简化实现：返回第一个可用动作
        // 实际应用中应该解析LLM返回的决策
        situation
            .available_actions
            .first()
            .cloned()
            .ok_or_else(|| AIError::InternalError("No available actions".to_string()))
    }

    async fn generate_content(&self, prompt: &ContentPrompt) -> Result<GeneratedContent, AIError> {
        let (messages, system) = self.build_content_messages(prompt);
        let response = self.call_claude(messages, Some(system)).await?;

        let content = response
            .content
            .first()
            .map(|c| c.text.trim().to_string())
            .ok_or_else(|| AIError::ParseError("No content in response".to_string()))?;

        let tokens_used = response.usage.map(|u| u.input_tokens + u.output_tokens);

        Ok(GeneratedContent {
            content,
            content_type: prompt.content_type.clone(),
            tokens_used,
            confidence: 0.85,
            metadata: {
                let mut meta = serde_json::Map::new();
                meta.insert("model".to_string(), serde_json::Value::String(self.model.clone()));
                serde_json::Map::into_iter(meta).collect()
            },
        })
    }

    async fn health_check(&self) -> Result<(), AIError> {
        let messages = vec![ClaudeMessage {
            role: "user".to_string(),
            content: vec![ContentBlock {
                content_type: "text".to_string(),
                text: "Hello".to_string(),
            }],
        }];

        self.call_claude(messages, None).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_claude_adapter_creation() {
        let adapter = ClaudeAdapter::new("test-key", "claude-3-opus-20240229");
        assert_eq!(adapter.model, "claude-3-opus-20240229");
        assert_eq!(adapter.api_key, "test-key");
    }

    #[test]
    fn test_with_max_tokens() {
        let adapter = ClaudeAdapter::new("test-key", "claude-3-opus-20240229")
            .with_max_tokens(500);
        assert_eq!(adapter.max_tokens, 500);
    }

    #[test]
    fn test_with_temperature() {
        let adapter = ClaudeAdapter::new("test-key", "claude-3-opus-20240229")
            .with_temperature(0.8);
        assert_eq!(adapter.temperature, 0.8);
    }

    #[test]
    fn test_temperature_clamping() {
        let adapter1 = ClaudeAdapter::new("test-key", "claude-3-opus-20240229")
            .with_temperature(1.5);
        assert_eq!(adapter1.temperature, 1.0);

        let adapter2 = ClaudeAdapter::new("test-key", "claude-3-opus-20240229")
            .with_temperature(-0.5);
        assert_eq!(adapter2.temperature, 0.0);
    }
}
