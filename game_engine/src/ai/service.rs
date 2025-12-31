//! # AI服务抽象接口
//!
//! 本模块定义了统一的AI服务抽象接口，支持多种LLM提供商。
//!
//! ## 核心概念
//!
//! - **AIService**: 统一的AI服务trait，支持多种LLM提供商
//! - **NPCContext**: NPC对话和决策的上下文信息
//! - **Situation**: NPC行为决策的情境信息
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::ai::service::{AIService, NPCContext};
//!
//! async fn generate_npc_dialogue(
//!     service: &dyn AIService,
//!     context: &NPCContext
//! ) -> Result<String, AIError> {
//!     service.generate_dialogue(context).await
//! }
//! ```

use async_trait::async_trait;
use bevy_ecs::entity::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// AI服务抽象接口
///
/// 提供统一的AI服务接口，支持多种LLM提供商（OpenAI、Claude、本地模型等）。
/// 所有AI服务实现都应该实现此trait。
///
/// # 泛型参数
///
/// - `EntityId`: 实体ID类型，通常使用`bevy_ecs::entity::Entity`
///
/// # 示例
///
/// ```rust
/// use game_engine::ai::service::AIService;
///
/// struct MyAIService {
///     // 实现细节
/// }
///
/// #[async_trait]
/// impl AIService for MyAIService {
///     async fn generate_dialogue(&self, context: &NPCContext) -> Result<String, AIError> {
///         // 实现对话生成逻辑
///         Ok("Hello!".to_string())
///     }
///     // ... 其他方法
/// }
/// ```
#[async_trait]
pub trait AIService: Send + Sync {
    /// 生成NPC对话
    ///
    /// 根据NPC的上下文信息（包括历史对话、个性、环境等）生成合适的对话内容。
    ///
    /// # 参数
    ///
    /// - `context`: NPC的上下文信息，包括个性、历史对话、环境状态等
    ///
    /// # 返回
    ///
    /// 返回生成的对话文本，失败时返回错误信息。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::ai::service::{AIService, NPCContext};
    ///
    /// let dialogue = ai_service.generate_dialogue(&context).await?;
    /// ```
    async fn generate_dialogue(&self, context: &NPCContext) -> Result<String, AIError>;

    /// 决策NPC行为
    ///
    /// 根据当前情境信息（附近实体、目标、可用动作等）决策NPC应该采取的行动。
    ///
    /// # 参数
    ///
    /// - `situation`: 当前情境信息，包括附近的实体、当前目标、可用动作等
    ///
    /// # 返回
    ///
    /// 返回决策的动作，失败时返回错误信息。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::ai::service::{AIService, Situation};
    ///
    /// let action = ai_service.decide_action(&situation).await?;
    /// ```
    async fn decide_action(&self, situation: &Situation) -> Result<Action, AIError>;

    /// 生成游戏内容
    ///
    /// 根据提示生成游戏内容，如任务描述、物品属性、场景描述等。
    ///
    /// # 参数
    ///
    /// - `prompt`: 内容生成提示，包括类型、约束条件等
    ///
    /// # 返回
    ///
    /// 返回生成的内容，失败时返回错误信息。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::ai::service::{AIService, ContentPrompt};
    ///
    /// let content = ai_service.generate_content(&prompt).await?;
    /// ```
    async fn generate_content(&self, prompt: &ContentPrompt) -> Result<GeneratedContent, AIError>;

    /// 服务健康检查
    ///
    /// 检查AI服务是否可用，包括API连接、模型加载等。
    ///
    /// # 返回
    ///
    /// 服务正常时返回Ok(())，否则返回错误信息。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use game_engine::ai::service::AIService;
    ///
    /// ai_service.health_check().await?;
    /// ```
    async fn health_check(&self) -> Result<(), AIError>;
}

/// NPC上下文信息
///
/// 包含NPC进行对话生成和决策所需的所有上下文信息。
#[derive(Debug, Clone)]
pub struct NPCContext {
    /// NPC的实体ID
    pub npc_id: Entity,
    /// 玩家状态信息
    pub player_state: PlayerState,
    /// 环境状态信息
    pub environment: EnvironmentState,
    /// 对话历史
    pub conversation_history: Vec<Message>,
    /// NPC个性特征
    pub personality: Personality,
    /// 当前任务（如果有）
    pub current_quest: Option<String>,
    /// NPC的情绪状态
    pub mood: MoodState,
}

/// 玩家状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    /// 玩家等级
    pub level: u32,
    /// 玩家声望
    pub reputation: HashMap<String, i32>,
    /// 玩家当前生命值
    pub health: f32,
    /// 玩家最大生命值
    pub max_health: f32,
    /// 玩家背包物品
    pub inventory: Vec<String>,
    /// 玩家已完成的任务
    pub completed_quests: Vec<String>,
}

/// 环境状态信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    /// 当前位置
    pub location: String,
    /// 时间（游戏内时间）
    pub game_time: String,
    /// 天气状况
    pub weather: String,
    /// 附近的实体
    pub nearby_entities: Vec<String>,
    /// 战斗状态
    pub in_combat: bool,
}

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息角色（system/user/assistant）
    pub role: String,
    /// 消息内容
    pub content: String,
    /// 消息时间戳
    pub timestamp: u64,
}

/// NPC个性特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Personality {
    /// 友好度（0.0-1.0）
    pub friendliness: f32,
    /// 正式程度（0.0-1.0）
    pub formality: f32,
    /// 幽默感（0.0-1.0）
    pub humor: f32,
    /// 勇气（0.0-1.0）
    pub bravery: f32,
    /// 贪婪（0.0-1.0）
    pub greed: f32,
    /// 自定义特性
    pub custom_traits: HashMap<String, f32>,
}

/// 情绪状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodState {
    /// 快乐值（0.0-1.0）
    pub happiness: f32,
    /// 愤怒值（0.0-1.0）
    pub anger: f32,
    /// 恐惧值（0.0-1.0）
    pub fear: f32,
    /// 信任值（0.0-1.0）
    pub trust: f32,
}

/// 决策情境
///
/// 包含NPC进行行为决策所需的情境信息。
#[derive(Debug, Clone)]
pub struct Situation {
    /// 附近的实体ID列表
    pub nearby_entities: Vec<Entity>,
    /// 当前目标
    pub current_goal: Option<String>,
    /// 可用的动作列表
    pub available_actions: Vec<Action>,
    /// 时间约束
    pub time_constraints: Option<Duration>,
    /// NPC当前状态
    pub npc_status: NPCStatus,
    /// 感知到的威胁
    pub perceived_threats: Vec<Threat>,
    /// 资源状态
    pub resources: HashMap<String, f32>,
}

/// NPC状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NPCStatus {
    /// 空闲
    Idle,
    /// 移动中
    Moving,
    /// 战斗中
    InCombat,
    /// 交互中
    Interacting,
    /// 受伤
    Wounded,
    /// 死亡
    Dead,
}

/// 威胁信息
#[derive(Debug, Clone)]
pub struct Threat {
    /// 威胁源实体ID
    pub entity_id: Entity,
    /// 威胁等级（0.0-1.0）
    pub severity: f32,
    /// 威胁类型
    pub threat_type: ThreatType,
    /// 距离
    pub distance: f32,
}

/// 威胁类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatType {
    /// 物理攻击
    Physical,
    /// 魔法攻击
    Magical,
    /// 环境危害
    Environmental,
    /// 陷阱
    Trap,
}

/// NPC动作
///
/// 表示NPC可以采取的各种动作。
#[derive(Debug, Clone)]
pub struct Action {
    /// 动作类型
    pub action_type: ActionType,
    /// 动作参数
    pub parameters: HashMap<String, serde_json::Value>,
    /// 优先级（0.0-1.0）
    pub priority: f32,
    /// 预估执行时间
    pub estimated_duration: Option<Duration>,
}

/// 动作类型
#[derive(Debug, Clone)]
pub enum ActionType {
    /// 移动
    Move { target: [f32; 3] },
    /// 攻击
    Attack { target: Entity },
    /// 防御
    Defend,
    /// 交互
    Interact { target: Entity },
    /// 说话
    Speak { message: String },
    /// 等待
    Wait,
    /// 使用物品
    UseItem { item_id: String },
    /// 施法
    CastSpell { spell_id: String },
    /// 逃跑
    Flee,
    /// 自定义动作
    Custom {
        name: String,
        data: serde_json::Value,
    },
}

/// 内容生成提示
///
/// 用于生成游戏内容的提示信息。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPrompt {
    /// 内容类型
    pub content_type: ContentType,
    /// 提示文本
    pub prompt: String,
    /// 约束条件
    pub constraints: Vec<String>,
    /// 最大长度
    pub max_length: Option<usize>,
    /// 风格要求
    pub style: Option<String>,
}

/// 内容类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    /// 任务描述
    QuestDescription,
    /// 物品属性
    ItemLore,
    /// 场景描述
    SceneDescription,
    /// 角色背景
    CharacterBackground,
    /// 对话选项
    DialogueOptions,
    /// 自定义内容
    Custom(String),
}

/// 生成的内容
///
/// AI服务生成的内容结果。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedContent {
    /// 内容文本
    pub content: String,
    /// 内容类型
    pub content_type: ContentType,
    /// 使用的token数（如果适用）
    pub tokens_used: Option<usize>,
    /// 置信度（0.0-1.0）
    pub confidence: f32,
    /// 额外元数据
    pub metadata: HashMap<String, serde_json::Value>,
}

/// AI错误类型
///
/// AI服务可能返回的各种错误。
#[derive(Debug, Clone, thiserror::Error)]
pub enum AIError {
    /// API请求失败
    #[error("API request failed: {0}")]
    ApiError(String),

    /// 认证失败
    #[error("Authentication failed")]
    AuthenticationError,

    /// 速率限制
    #[error("Rate limit exceeded")]
    RateLimitError,

    /// 网络错误
    #[error("Network error: {0}")]
    NetworkError(String),

    /// 解析错误
    #[error("Parse error: {0}")]
    ParseError(String),

    /// 无效输入
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// 模型不可用
    #[error("Model unavailable: {0}")]
    ModelUnavailable(String),

    /// 超时
    #[error("Operation timed out")]
    TimeoutError,

    /// 内部错误
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl Default for Personality {
    fn default() -> Self {
        Self {
            friendliness: 0.5,
            formality: 0.5,
            humor: 0.3,
            bravery: 0.5,
            greed: 0.3,
            custom_traits: HashMap::new(),
        }
    }
}

impl Default for MoodState {
    fn default() -> Self {
        Self {
            happiness: 0.7,
            anger: 0.1,
            fear: 0.1,
            trust: 0.5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_default() {
        let personality = Personality::default();
        assert_eq!(personality.friendliness, 0.5);
        assert_eq!(personality.formality, 0.5);
    }

    #[test]
    fn test_mood_state_default() {
        let mood = MoodState::default();
        assert_eq!(mood.happiness, 0.7);
        assert_eq!(mood.anger, 0.1);
    }

    #[test]
    fn test_action_serialization() {
        let action = Action {
            action_type: ActionType::Move {
                target: [1.0, 2.0, 3.0],
            },
            parameters: HashMap::new(),
            priority: 0.8,
            estimated_duration: Some(Duration::from_secs(2)),
        };

        let serialized = serde_json::to_string(&action).unwrap();
        assert!(serialized.contains("Move"));
    }
}
