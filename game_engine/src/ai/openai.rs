//! # OpenAI适配器
//!
//! 本模块提供OpenAI API的适配器实现。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::ai::openai::OpenAIAdapter;
//!
//! let adapter = OpenAIAdapter::new(
//!     "your-api-key",
//!     "gpt-4"
//! );
//!
//! let dialogue = adapter.generate_dialogue(&context).await?;
//! ```

use super::service::{
    AIError, AIService, Action, ContentPrompt, ContentType, GeneratedContent, Message, NPCContext,
    Situation,
};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// OpenAI API适配器
///
/// 提供对OpenAI GPT模型的访问，支持GPT-3.5和GPT-4等模型。
pub struct OpenAIAdapter {
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
    /// 温度参数（0.0-2.0）
    temperature: f32,
}

/// OpenAI聊天消息
#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

/// OpenAI API请求
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    max_tokens: usize,
    temperature: f32,
}

/// OpenAI API响应
#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
}

/// OpenAI响应选择
#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
    finish_reason: Option<String>,
}

/// OpenAI响应消息
#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

/// Token使用情况
#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
}

impl OpenAIAdapter {
    /// 创建新的OpenAI适配器
    ///
    /// # 参数
    ///
    /// - `api_key`: OpenAI API密钥
    /// - `model`: 模型名称（如"gpt-4", "gpt-3.5-turbo"）
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::ai::openai::OpenAIAdapter;
    ///
    /// let adapter = OpenAIAdapter::new(
    ///     "sk-...",
    ///     "gpt-4"
    /// );
    /// ```
    pub fn new(api_key: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            model: model.into(),
            client: Client::builder().timeout(Duration::from_secs(30)).build().unwrap_or_default(),
            base_url: "https://api.openai.com/v1/chat/completions".to_string(),
            max_tokens: 150,
            temperature: 0.7,
        }
    }

    /// 设置API基础URL
    ///
    /// 用于使用自定义的OpenAI兼容API端点。
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
        self.temperature = temperature.clamp(0.0, 2.0);
        self
    }

    /// 调用OpenAI API
    async fn call_openai(&self, messages: Vec<ChatMessage>) -> Result<ChatResponse, AIError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        let response = self
            .client
            .post(&self.base_url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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
    fn build_dialogue_prompt(&self, context: &NPCContext) -> Vec<ChatMessage> {
        let mut messages = vec![ChatMessage {
            role: "system".to_string(),
            content: self.build_system_prompt(context),
        }];

        // 添加历史对话
        for msg in &context.conversation_history {
            messages.push(ChatMessage {
                role: msg.role.clone(),
                content: msg.content.clone(),
            });
        }

        // 添加当前情境
        messages.push(ChatMessage {
            role: "user".to_string(),
            content: self.build_user_prompt(context),
        });

        messages
    }

    /// 构建系统提示
    fn build_system_prompt(&self, context: &NPCContext) -> String {
        format!(
            "You are an NPC in a game. Your personality traits:\n\
             - Friendliness: {:.1}\n\
             - Formality: {:.1}\n\
             - Humor: {:.1}\n\
             - Bravery: {:.1}\n\
             - Greed: {:.1}\n\
             \n\
             Current mood:\n\
             - Happiness: {:.1}\n\
             - Anger: {:.1}\n\
             - Fear: {:.1}\n\
             - Trust: {:.1}\n\
             \n\
             Respond in character, keeping your response concise (under 100 words).",
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
            "Player is at level {} with {:.0}/{:.0} health.\n\
             Location: {}\n\
             Time: {}\n\
             Weather: {}\n\
             Nearby entities: {:?}\n\
             In combat: {}\n\
             \n\
             Generate an appropriate response to the player.",
            context.player_state.level,
            context.player_state.health,
            context.player_state.max_health,
            context.environment.location,
            context.environment.game_time,
            context.environment.weather,
            context.environment.nearby_entities,
            context.environment.in_combat
        )
    }

    /// 构建决策提示
    fn build_decision_prompt(&self, situation: &Situation) -> Vec<ChatMessage> {
        vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are an AI NPC decision system. Choose the best action based on the situation. \
                          Respond with a JSON object containing the action type and parameters.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "Current situation:\n\
                     - Status: {:?}\n\
                     - Nearby entities: {}\n\
                     - Current goal: {:?}\n\
                     - Available actions: {:?}\n\
                     - Perceived threats: {}\n\
                     \n\
                     Choose the most appropriate action.",
                    situation.npc_status,
                    situation.nearby_entities.len(),
                    situation.current_goal,
                    situation.available_actions.iter().map(|a| format!("{:?}", a.action_type)).collect::<Vec<_>>(),
                    situation.perceived_threats.len()
                ),
            },
        ]
    }

    /// 构建内容生成提示
    fn build_content_prompt(&self, prompt: &ContentPrompt) -> Vec<ChatMessage> {
        vec![
            ChatMessage {
                role: "system".to_string(),
                content: format!(
                    "You are a game content generator. Generate content of type: {:?}. \
                     Keep the output concise and engaging.",
                    prompt.content_type
                ),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!(
                    "{}\n\nConstraints:\n{}\n\nStyle: {}",
                    prompt.prompt,
                    prompt.constraints.join("\n"),
                    prompt.style.as_deref().unwrap_or("neutral")
                ),
            },
        ]
    }
}

#[async_trait]
impl AIService for OpenAIAdapter {
    async fn generate_dialogue(&self, context: &NPCContext) -> Result<String, AIError> {
        let messages = self.build_dialogue_prompt(context);
        let response = self.call_openai(messages).await?;

        response
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .ok_or_else(|| AIError::ParseError("No choices in response".to_string()))
    }

    async fn decide_action(&self, situation: &Situation) -> Result<Action, AIError> {
        let messages = self.build_decision_prompt(situation);
        let response = self.call_openai(messages).await?;

        let content = response
            .choices
            .first()
            .map(|choice| choice.message.content.trim())
            .ok_or_else(|| AIError::ParseError("No choices in response".to_string()))?;

        // 简化实现：返回第一个可用动作
        // 实际应用中应该解析LLM返回的JSON
        situation
            .available_actions
            .first()
            .cloned()
            .ok_or_else(|| AIError::InternalError("No available actions".to_string()))
    }

    async fn generate_content(&self, prompt: &ContentPrompt) -> Result<GeneratedContent, AIError> {
        let messages = self.build_content_prompt(prompt);
        let response = self.call_openai(messages).await?;

        let content = response
            .choices
            .first()
            .map(|choice| choice.message.content.trim().to_string())
            .ok_or_else(|| AIError::ParseError("No choices in response".to_string()))?;

        let tokens_used = response.usage.map(|u| u.total_tokens);

        Ok(GeneratedContent {
            content,
            content_type: prompt.content_type.clone(),
            tokens_used,
            confidence: 0.8,
            metadata: {
                let mut meta = serde_json::Map::new();
                meta.insert(
                    "model".to_string(),
                    serde_json::Value::String(self.model.clone()),
                );
                serde_json::Map::into_iter(meta).collect()
            },
        })
    }

    async fn health_check(&self) -> Result<(), AIError> {
        let test_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];

        self.call_openai(test_messages).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::entity::Entity;
    use std::collections::HashMap;

    fn create_test_context() -> NPCContext {
        NPCContext {
            npc_id: Entity::from_raw(1),
            player_state: super::super::service::PlayerState {
                level: 10,
                reputation: HashMap::new(),
                health: 100.0,
                max_health: 100.0,
                inventory: vec![],
                completed_quests: vec![],
            },
            environment: super::super::service::EnvironmentState {
                location: "Town".to_string(),
                game_time: "12:00".to_string(),
                weather: "Sunny".to_string(),
                nearby_entities: vec![],
                in_combat: false,
            },
            conversation_history: vec![],
            personality: Default::default(),
            current_quest: None,
            mood: Default::default(),
        }
    }

    #[test]
    fn test_openai_adapter_creation() {
        let adapter = OpenAIAdapter::new("test-key", "gpt-4");
        assert_eq!(adapter.model, "gpt-4");
        assert_eq!(adapter.api_key, "test-key");
    }

    #[test]
    fn test_with_max_tokens() {
        let adapter = OpenAIAdapter::new("test-key", "gpt-4").with_max_tokens(500);
        assert_eq!(adapter.max_tokens, 500);
    }

    #[test]
    fn test_with_temperature() {
        let adapter = OpenAIAdapter::new("test-key", "gpt-4").with_temperature(1.5);
        assert_eq!(adapter.temperature, 1.5);
    }

    #[test]
    fn test_build_system_prompt() {
        let adapter = OpenAIAdapter::new("test-key", "gpt-4");
        let context = create_test_context();
        let prompt = adapter.build_system_prompt(&context);

        assert!(prompt.contains("Friendliness:"));
        assert!(prompt.contains("Happiness:"));
    }
}
