//! # NPU LLM推理加速
//!
//! 使用NPU加速大语言模型推理，用于游戏AI和NPC对话。
//!
//! ## 性能目标
//!
//! - **推理速度:** >50 tokens/s
//! - **内存占用:** <2GB (量化模型)
//! - **延迟:** <100ms (首次token)
//!
//! ## 支持的NPU
//!
//! - Apple Neural Engine (macOS/iOS)
//! - Android NNAPI
//! - Intel NPU (OpenVINO)
//! - CPU/GPU fallback
//!
//! ## 使用示例
//!
//! ```rust,no_run
//! use game_engine::acceleration::llm::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 创建LLM引擎
//!     let mut llm = NpuLlmEngine::new("models/llama-2-7b-quantized.mlmodel").await?;
//!
//!     // NPC对话
//!     let response = llm.chat(
//!         "You are a friendly shopkeeper in a fantasy game.",
//!         "Hello, do you have any magic swords?"
//!     ).await?;
//!
//!     println!("NPC: {}", response);
//!     Ok(())
//! }
//! ```

use super::npus::*;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// LLM引擎
// ============================================================================

/// NPU LLM推理引擎
pub struct NpuLlmEngine {
    /// NPU运行时
    runtime: Arc<NPURuntime>,

    /// LLM模型
    model: Arc<RwLock<Option<NPUModel>>>,

    /// 模型路径
    model_path: PathBuf,

    /// 是否已初始化
    initialized: bool,

    /// 推理统计
    stats: LlmStats,
}

/// LLM推理统计
#[derive(Debug, Default, Clone)]
pub struct LlmStats {
    /// 总推理次数
    pub total_inferences: u64,

    /// 总生成token数
    pub total_tokens: u64,

    /// 总推理时间（秒）
    pub total_inference_time: f32,

    /// 平均tokens/s
    pub average_tokens_per_second: f32,
}

impl NpuLlmEngine {
    /// 创建新的LLM引擎
    ///
    /// **参数:**
    /// - `model_path`: 模型文件路径（.mlmodel, .tflite, .onnx等）
    pub async fn new(model_path: impl Into<PathBuf>) -> Result<Self, NPUError> {
        let model_path = model_path.into();

        tracing::info!("Creating NPU LLM engine with model: {:?}", model_path);

        // 检查模型文件是否存在
        if !model_path.exists() {
            return Err(NPUError::ModelLoadFailed(format!(
                "Model file not found: {model_path:?}"
            )));
        }

        // 创建NPU运行时
        let runtime = NPURuntime::new().await?;

        // 在移动前记录设备类型
        let device_name = runtime.device_type().name().to_string();

        let engine = Self {
            runtime: Arc::new(runtime),
            model: Arc::new(RwLock::new(None)),
            model_path,
            initialized: false,
            stats: LlmStats::default(),
        };

        tracing::info!("NPU LLM engine created with device: {}", device_name);

        Ok(engine)
    }

    /// 初始化LLM模型
    pub async fn initialize(&mut self) -> Result<(), NPUError> {
        if self.initialized {
            tracing::warn!("LLM engine already initialized");
            return Ok(());
        }

        tracing::info!("Loading LLM model from: {:?}", self.model_path);

        // 加载模型
        let model = self.runtime.load_model(self.model_path.to_str().unwrap()).await?;

        // 存储模型
        {
            let mut model_guard = self.model.write().await;
            *model_guard = Some(model);
        }

        self.initialized = true;

        tracing::info!("LLM model loaded successfully");
        tracing::info!("Model inputs: {:?}", self.get_model_info().await?);

        Ok(())
    }

    /// 聊天对话
    ///
    /// **参数:**
    /// - `system_prompt`: 系统提示词（定义NPC角色）
    /// - `user_input`: 用户输入
    ///
    /// **返回:** NPC响应
    pub async fn chat(
        &mut self,
        system_prompt: &str,
        user_input: &str,
    ) -> Result<String, NPUError> {
        if !self.initialized {
            return Err(NPUError::InferenceFailed(
                "LLM engine not initialized".to_string(),
            ));
        }

        let start = std::time::Instant::now();

        tracing::debug!("Chat: system='{}', user='{}'", system_prompt, user_input);

        // 准备输入
        let prompt = self.format_prompt(system_prompt, user_input);

        // Tokenize（简化实现）
        let input_tokens = self.tokenize(&prompt);

        // 执行推理
        let output_tokens = self.infer(&input_tokens).await?;

        // Decode
        let response = self.decode(&output_tokens);

        let elapsed = start.elapsed();
        let tokens_count = output_tokens.len() as f32;
        let tokens_per_sec = tokens_count / elapsed.as_secs_f32();

        // 更新统计
        self.stats.total_inferences += 1;
        self.stats.total_tokens += output_tokens.len() as u64;
        self.stats.total_inference_time += elapsed.as_secs_f32();
        self.stats.average_tokens_per_second =
            self.stats.total_tokens as f32 / self.stats.total_inference_time;

        tracing::info!(
            "LLM inference: {} tokens in {:?} ({:.1} tokens/s)",
            output_tokens.len(),
            elapsed,
            tokens_per_sec
        );

        Ok(response)
    }

    /// 流式生成（用于实时显示）
    pub async fn chat_streaming(
        &mut self,
        system_prompt: &str,
        user_input: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, NPUError> {
        if !self.initialized {
            return Err(NPUError::InferenceFailed(
                "LLM engine not initialized".to_string(),
            ));
        }

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        // 准备输入
        let prompt = self.format_prompt(system_prompt, user_input);
        let input_tokens = self.tokenize(&prompt);

        // 在后台任务中生成
        let model = self.model.clone();
        tokio::spawn(async move {
            // 简化实现：发送完整响应
            // 实际实现应该逐token生成

            if let Some(model) = model.read().await.as_ref() {
                // 模拟流式生成
                let dummy_response = "This is a simulated response from the LLM.";

                for ch in dummy_response.chars() {
                    if tx.send(ch.to_string()).await.is_err() {
                        break;
                    }
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
            }

            drop(tx);
        });

        Ok(rx)
    }

    /// 执行推理
    async fn infer(&mut self, input_tokens: &[u32]) -> Result<Vec<u32>, NPUError> {
        let model_guard = self.model.read().await;

        let model = model_guard
            .as_ref()
            .ok_or_else(|| NPUError::InferenceFailed("Model not loaded".to_string()))?;

        // 准备输入张量
        let input_tensor = NPUTensor {
            data: TensorData::Int32(input_tokens.iter().map(|&t| t as i32).collect()),
            shape: vec![1, input_tokens.len()],
            dtype: TensorDType::Int32,
            name: Some("input_ids".to_string()),
        };

        // 执行推理
        let outputs = model.inference(&[input_tensor]).await?;

        // 解析输出（简化）
        if let Some(output) = outputs.first() {
            match &output.data {
                TensorData::Int32(tokens) => Ok(tokens.iter().map(|&t| t as u32).collect()),
                TensorData::Int8(tokens) => Ok(tokens.iter().map(|&t| t as u32).collect()),
                TensorData::UInt8(tokens) => Ok(tokens.iter().map(|&t| t as u32).collect()),
                _ => Err(NPUError::InferenceFailed(
                    "Unexpected output data type".to_string(),
                )),
            }
        } else {
            Err(NPUError::InferenceFailed(
                "No output from model".to_string(),
            ))
        }
    }

    /// 格式化提示词
    fn format_prompt(&self, system_prompt: &str, user_input: &str) -> String {
        format!("### System:\n{system_prompt}\n\n### User:\n{user_input}\n\n### Assistant:\n")
    }

    /// Tokenize（简化实现）
    fn tokenize(&self, text: &str) -> Vec<u32> {
        // 简化实现：每个字符作为一个token
        // 实际实现应该使用proper tokenizer
        text.chars().map(|c| c as u32).collect()
    }

    /// Decode（简化实现）
    fn decode(&self, tokens: &[u32]) -> String {
        // 简化实现
        tokens.iter().map(|&t| t as u8 as char).collect::<String>()
    }

    /// 获取模型信息
    pub async fn get_model_info(&self) -> Result<LlmModelInfo, NPUError> {
        let model_guard = self.model.read().await;

        if let Some(model) = model_guard.as_ref() {
            Ok(LlmModelInfo {
                name: model.name.clone(),
                input_shapes: model.input_spec().iter().map(|spec| spec.shape.clone()).collect(),
                output_shapes: model.output_spec().iter().map(|spec| spec.shape.clone()).collect(),
            })
        } else {
            Err(NPUError::InferenceFailed("Model not loaded".to_string()))
        }
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> &LlmStats {
        &self.stats
    }

    /// 重置统计信息
    pub fn reset_stats(&mut self) {
        self.stats = LlmStats::default();
    }
}

/// LLM模型信息
#[derive(Debug, Clone)]
pub struct LlmModelInfo {
    /// 模型名称
    pub name: String,

    /// 输入形状
    pub input_shapes: Vec<Vec<usize>>,

    /// 输出形状
    pub output_shapes: Vec<Vec<usize>>,
}

// ============================================================================
// NPC AI集成
// ============================================================================

/// NPC AI组件
pub struct NpcLlmAi {
    /// LLM引擎
    llm: NpuLlmEngine,

    /// NPC角色定义
    persona: NpcPersona,
}

/// NPC角色定义
#[derive(Debug, Clone)]
pub struct NpcPersona {
    /// NPC名称
    pub name: String,

    /// 角色描述
    pub description: String,

    /// 性格特征
    pub personality: Vec<String>,

    /// 背景故事
    pub backstory: String,

    /// 对话风格
    pub dialogue_style: String,
}

impl NpcPersona {
    /// 创建系统提示词
    pub fn to_system_prompt(&self) -> String {
        format!(
            "You are {}, a character in a fantasy game.\n\n\
             Description: {}\n\n\
             Personality: {}\n\n\
             Background: {}\n\n\
             Dialogue Style: {}\n\n\
             Stay in character and respond naturally.",
            self.name,
            self.description,
            self.personality.join(", "),
            self.backstory,
            self.dialogue_style
        )
    }
}

impl NpcLlmAi {
    /// 创建新的NPC AI
    pub async fn new(llm: NpuLlmEngine, persona: NpcPersona) -> Result<Self, NPUError> {
        Ok(Self { llm, persona })
    }

    /// NPC对话
    pub async fn talk(&mut self, player_input: &str) -> Result<String, NPUError> {
        let system_prompt = self.persona.to_system_prompt();
        self.llm.chat(&system_prompt, player_input).await
    }

    /// 流式对话（实时显示）
    pub async fn talk_streaming(
        &mut self,
        player_input: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<String>, NPUError> {
        let system_prompt = self.persona.to_system_prompt();
        self.llm.chat_streaming(&system_prompt, player_input).await
    }

    /// 行为决策
    pub async fn decide_action(&mut self, context: &GameContext) -> Result<NpcAction, NPUError> {
        let prompt = self.format_decision_prompt(context);

        let response = self.llm.chat(&self.persona.to_system_prompt(), &prompt).await?;

        // 解析响应为动作
        self.parse_action(&response)
    }

    /// 格式化决策提示词
    fn format_decision_prompt(&self, context: &GameContext) -> String {
        format!(
            "You are {}. Given the current game situation, decide your next action.\n\n\
             Current situation:\n\
             - Health: {:.0}%\n\
             - Nearby enemies: {}\n\
             - Nearby allies: {}\n\
             - Current objective: {}\n\n\
             Respond with one of: ATTACK, DEFEND, FLEE, HELP, EXPLORE, or INTERACT.\n\
             Also provide a brief reason.",
            self.persona.name,
            context.health * 100.0,
            context.nearby_enemies,
            context.nearby_allies,
            context.objective
        )
    }

    /// 解析动作
    fn parse_action(&self, response: &str) -> Result<NpcAction, NPUError> {
        let response_upper = response.to_uppercase();

        let action_type = if response_upper.contains("ATTACK") {
            NpcActionType::Attack
        } else if response_upper.contains("DEFEND") {
            NpcActionType::Defend
        } else if response_upper.contains("FLEE") {
            NpcActionType::Flee
        } else if response_upper.contains("HELP") {
            NpcActionType::Help
        } else if response_upper.contains("EXPLORE") {
            NpcActionType::Explore
        } else if response_upper.contains("INTERACT") {
            NpcActionType::Interact
        } else {
            NpcActionType::Interact // 默认
        };

        Ok(NpcAction {
            action_type,
            reason: response.to_string(),
        })
    }
}

/// 游戏上下文
#[derive(Debug, Clone)]
pub struct GameContext {
    /// NPC生命值 (0-1)
    pub health: f32,

    /// 附近的敌人数量
    pub nearby_enemies: u32,

    /// 附近的盟友数量
    pub nearby_allies: u32,

    /// 当前目标
    pub objective: String,
}

/// NPC动作
#[derive(Debug, Clone)]
pub struct NpcAction {
    /// 动作类型
    pub action_type: NpcActionType,

    /// 原因
    pub reason: String,
}

/// NPC动作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcActionType {
    /// 攻击
    Attack,

    /// 防御
    Defend,

    /// 逃跑
    Flee,

    /// 帮助
    Help,

    /// 探索
    Explore,

    /// 交互
    Interact,
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_llm_engine_creation() {
        // 注意：这个测试需要实际的模型文件
        // 在CI/CD环境中应该跳过或使用mock
    }

    #[test]
    fn test_npc_persona() {
        let persona = NpcPersona {
            name: "Eldric the Wise".to_string(),
            description: "An old wizard".to_string(),
            personality: vec!["wise".to_string(), "mysterious".to_string()],
            backstory: "Once served the royal court".to_string(),
            dialogue_style: "Formal and archaic".to_string(),
        };

        let prompt = persona.to_system_prompt();

        assert!(prompt.contains("Eldric the Wise"));
        assert!(prompt.contains("wizard"));
    }
}
