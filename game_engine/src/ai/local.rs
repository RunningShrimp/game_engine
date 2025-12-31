//! # 本地模型适配器
//!
//! 本模块提供本地LLM推理的适配器实现。
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::ai::local::LocalLLMAdapter;
//!
//! let adapter = LocalLLMAdapter::new(
//!     "/path/to/model.gguf",
//!     LLMRuntime::LlamaCpp
//! );
//!
//! let dialogue = adapter.generate_dialogue(&context).await?;
//! ```

use super::service::{
    Action, AIError, AIService, ContentPrompt, ContentType, GeneratedContent, NPCContext,
    Situation,
};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// 本地LLM运行时类型
#[derive(Debug, Clone, Copy)]
pub enum LLMRuntime {
    /// 使用llama.cpp作为后端
    LlamaCpp,
    /// 使用ggerganov/llama.cpp作为后端
    GGML,
    /// 使用自定义后端
    Custom,
}

/// 本地LLM适配器配置
#[derive(Debug, Clone)]
pub struct LocalLLMConfig {
    /// 模型路径
    pub model_path: PathBuf,
    /// 运行时类型
    pub runtime: LLMRuntime,
    /// 上下文大小
    pub context_size: usize,
    /// 最大token数
    pub max_tokens: usize,
    /// 温度参数
    pub temperature: f32,
    /// 线程数
    pub threads: usize,
    /// GPU层数（用于GPU加速）
    pub gpu_layers: usize,
}

impl Default for LocalLLMConfig {
    fn default() -> Self {
        Self {
            model_path: PathBuf::from("model.gguf"),
            runtime: LLMRuntime::LlamaCpp,
            context_size: 2048,
            max_tokens: 150,
            temperature: 0.7,
            threads: 4,
            gpu_layers: 0,
        }
    }
}

/// 本地LLM适配器
///
/// 提供对本地运行的开源大语言模型的访问，支持llama.cpp等推理引擎。
pub struct LocalLLMAdapter {
    config: LocalLLMConfig,
    /// 可执行文件路径
    executable_path: PathBuf,
}

impl LocalLLMAdapter {
    /// 创建新的本地LLM适配器
    ///
    /// # 参数
    ///
    /// - `model_path`: 模型文件路径（GGUF格式）
    /// - `runtime`: 使用的推理引擎
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::ai::local::{LocalLLMAdapter, LLMRuntime};
    ///
    /// let adapter = LocalLLMAdapter::new(
    ///     "/models/llama-2-7b.gguf",
    ///     LLMRuntime::LlamaCpp
    /// );
    /// ```
    pub fn new(model_path: impl Into<PathBuf>, runtime: LLMRuntime) -> Self {
        let config = LocalLLMConfig {
            model_path: model_path.into(),
            runtime,
            ..Default::default()
        };

        let executable_path = match runtime {
            LLMRuntime::LlamaCpp => PathBuf::from("llama-cli"),
            LLMRuntime::GGML => PathBuf::from("main"),
            LLMRuntime::Custom => PathBuf::from("custom-llm"),
        };

        Self {
            config,
            executable_path,
        }
    }

    /// 使用自定义配置创建适配器
    pub fn with_config(config: LocalLLMConfig) -> Self {
        let executable_path = match config.runtime {
            LLMRuntime::LlamaCpp => PathBuf::from("llama-cli"),
            LLMRuntime::GGML => PathBuf::from("main"),
            LLMRuntime::Custom => PathBuf::from("custom-llm"),
        };

        Self {
            config,
            executable_path,
        }
    }

    /// 设置可执行文件路径
    pub fn with_executable(mut self, path: impl Into<PathBuf>) -> Self {
        self.executable_path = path.into();
        self
    }

    /// 设置上下文大小
    pub fn with_context_size(mut self, context_size: usize) -> Self {
        self.config.context_size = context_size;
        self
    }

    /// 设置最大token数
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.config.max_tokens = max_tokens;
        self
    }

    /// 设置线程数
    pub fn with_threads(mut self, threads: usize) -> Self {
        self.config.threads = threads;
        self
    }

    /// 设置GPU层数
    pub fn with_gpu_layers(mut self, gpu_layers: usize) -> Self {
        self.config.gpu_layers = gpu_layers;
        self
    }

    /// 执行本地推理
    async fn run_inference(&self, prompt: &str) -> Result<String, AIError> {
        // 检查模型文件是否存在
        if !self.config.model_path.exists() {
            return Err(AIError::ModelUnavailable(format!(
                "Model file not found: {:?}",
                self.config.model_path
            )));
        }

        // 构建命令行参数
        let output = Command::new(&self.executable_path)
            .arg("-m")
            .arg(&self.config.model_path)
            .arg("-p")
            .arg(prompt)
            .arg("-n")
            .arg(self.config.max_tokens.to_string())
            .arg("--ctx-size")
            .arg(self.config.context_size.to_string())
            .arg("--temp")
            .arg(self.config.temperature.to_string())
            .arg("-t")
            .arg(self.config.threads.to_string())
            .output()
            .map_err(|e| {
                AIError::InternalError(format!("Failed to execute LLM inference: {}", e))
            })?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(AIError::InternalError(format!(
                "LLM inference failed: {}",
                error_msg
            )));
        }

        let result = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(result.trim().to_string())
    }

    /// 构建对话提示
    fn build_dialogue_prompt(&self, context: &NPCContext) -> String {
        format!(
            "You are an NPC with the following traits:\n\
             Friendliness: {:.1}, Formality: {:.1}, Humor: {:.1}, Bravery: {:.1}, Greed: {:.1}\n\
             Current mood: Happiness {:.1}, Anger {:.1}, Fear {:.1}, Trust {:.1}\n\
             \n\
             Player is level {} at {} ({}). Time: {}, Weather: {}. In combat: {}.\n\
             \n\
             Generate a brief in-character response (under 100 words):",
            context.personality.friendliness,
            context.personality.formality,
            context.personality.humor,
            context.personality.bravery,
            context.personality.greed,
            context.mood.happiness,
            context.mood.anger,
            context.mood.fear,
            context.mood.trust,
            context.player_state.level,
            context.environment.location,
            if context.environment.in_combat { "combat" } else { "peace" },
            context.environment.game_time,
            context.environment.weather,
            context.environment.in_combat
        )
    }

    /// 构建决策提示
    fn build_decision_prompt(&self, situation: &Situation) -> String {
        format!(
            "NPC Status: {:?}, Goal: {:?}, Threats: {}, Available actions: {}\n\
             Choose the best action and respond with its index.",
            situation.npc_status,
            situation.current_goal,
            situation.perceived_threats.len(),
            situation.available_actions.len()
        )
    }

    /// 构建内容生成提示
    fn build_content_prompt(&self, prompt: &ContentPrompt) -> String {
        format!(
            "Generate {} content.\n\
             Task: {}\n\
             Constraints: {}\n\
             Style: {}",
            format!("{:?}", prompt.content_type).to_lowercase(),
            prompt.prompt,
            prompt.constraints.join(", "),
            prompt.style.as_deref().unwrap_or("neutral")
        )
    }
}

#[async_trait]
impl AIService for LocalLLMAdapter {
    async fn generate_dialogue(&self, context: &NPCContext) -> Result<String, AIError> {
        let prompt = self.build_dialogue_prompt(context);
        tokio::task::spawn_blocking({
            let adapter = self.clone_adapter();
            move || adapter.run_inference(&prompt)
        })
        .await
        .map_err(|e| AIError::InternalError(format!("Task join error: {}", e)))?
    }

    async fn decide_action(&self, situation: &Situation) -> Result<Action, AIError> {
        let prompt = self.build_decision_prompt(situation);

        let _response = tokio::task::spawn_blocking({
            let adapter = self.clone_adapter();
            move || adapter.run_inference(&prompt)
        })
        .await
        .map_err(|e| AIError::InternalError(format!("Task join error: {}", e)))??;

        // 简化实现：返回第一个可用动作
        // 实际应用中应该解析LLM返回的决策
        situation
            .available_actions
            .first()
            .cloned()
            .ok_or_else(|| AIError::InternalError("No available actions".to_string()))
    }

    async fn generate_content(&self, prompt: &ContentPrompt) -> Result<GeneratedContent, AIError> {
        let llm_prompt = self.build_content_prompt(prompt);

        let content = tokio::task::spawn_blocking({
            let adapter = self.clone_adapter();
            move || adapter.run_inference(&llm_prompt)
        })
        .await
        .map_err(|e| AIError::InternalError(format!("Task join error: {}", e)))??;

        Ok(GeneratedContent {
            content,
            content_type: prompt.content_type.clone(),
            tokens_used: None, // 本地模型通常不返回token使用情况
            confidence: 0.7,
            metadata: {
                let mut meta = serde_json::Map::new();
                meta.insert(
                    "model".to_string(),
                    serde_json::Value::String(
                        self.config.model_path.to_string_lossy().to_string(),
                    ),
                );
                meta.insert(
                    "runtime".to_string(),
                    serde_json::Value::String(format!("{:?}", self.config.runtime)),
                );
                serde_json::Map::into_iter(meta).collect()
            },
        })
    }

    async fn health_check(&self) -> Result<(), AIError> {
        // 检查可执行文件是否存在
        if which::which(&self.executable_path).is_err() {
            return Err(AIError::ModelUnavailable(format!(
                "LLM executable not found: {:?}",
                self.executable_path
            )));
        }

        // 检查模型文件是否存在
        if !self.config.model_path.exists() {
            return Err(AIError::ModelUnavailable(format!(
                "Model file not found: {:?}",
                self.config.model_path
            )));
        }

        Ok(())
    }
}

impl LocalLLMAdapter {
    /// 克隆适配器（用于在spawn_blocking中使用）
    fn clone_adapter(&self) -> Self {
        Self {
            config: self.config.clone(),
            executable_path: self.executable_path.clone(),
        }
    }
}

impl Clone for LocalLLMAdapter {
    fn clone(&self) -> Self {
        self.clone_adapter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_llm_adapter_creation() {
        let adapter = LocalLLMAdapter::new("model.gguf", LLMRuntime::LlamaCpp);
        assert_eq!(adapter.config.model_path, PathBuf::from("model.gguf"));
        assert_eq!(adapter.config.runtime, LLMRuntime::LlamaCpp);
    }

    #[test]
    fn test_with_context_size() {
        let adapter = LocalLLMAdapter::new("model.gguf", LLMRuntime::LlamaCpp)
            .with_context_size(4096);
        assert_eq!(adapter.config.context_size, 4096);
    }

    #[test]
    fn test_with_threads() {
        let adapter = LocalLLMAdapter::new("model.gguf", LLMRuntime::LlamaCpp)
            .with_threads(8);
        assert_eq!(adapter.config.threads, 8);
    }

    #[test]
    fn test_with_gpu_layers() {
        let adapter = LocalLLMAdapter::new("model.gguf", LLMRuntime::LlamaCpp)
            .with_gpu_layers(32);
        assert_eq!(adapter.config.gpu_layers, 32);
    }

    #[test]
    fn test_config_default() {
        let config = LocalLLMConfig::default();
        assert_eq!(config.context_size, 2048);
        assert_eq!(config.max_tokens, 150);
        assert_eq!(config.threads, 4);
    }

    #[test]
    fn test_clone_adapter() {
        let adapter1 = LocalLLMAdapter::new("model.gguf", LLMRuntime::LlamaCpp)
            .with_max_tokens(200);
        let adapter2 = adapter1.clone();

        assert_eq!(adapter1.config.max_tokens, adapter2.config.max_tokens);
        assert_eq!(adapter1.config.model_path, adapter2.config.model_path);
    }
}
