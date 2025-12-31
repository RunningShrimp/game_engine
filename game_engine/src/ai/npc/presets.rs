//! # NPC预设系统
//!
//! 提供预定义的NPC个性模板，简化NPC创建和配置。
//!
//! ## 功能特性
//!
//! - **10+预设模板** - 覆盖常见NPC类型
//! - **性格参数** - 友好度、攻击性、好奇心、恐惧等
//! - **行为树预设** - 每个预设包含推荐行为树
//! - **LLM配置** - 优化的提示词模板
//! - **对话示例** - 每个预设的示例对话
//!
//! ## 使用示例
//!
//! ```rust
//! use game_engine::ai::npc::presets::{NPCPreset, PresetManager};
//!
//! // 获取预设
//! let preset = PresetManager::get_preset("friendly_merchant")?;
//!
//! // 创建NPC配置
//! let config = preset.to_npc_config();
//!
//! // 或者使用builder创建自定义预设
//! let custom = NPCPreset::builder()
//!     .name("custom_guard")
//!     .friendliness(0.3)
//!     .aggression(0.8)
//!     .build();
//! ```

use super::super::service::{MoodState, Personality};
use super::{HybridMode, IntelligentNPC, NPCConfig};
use bevy_ecs::entity::Entity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// NPC预设模板
///
/// 包含NPC的完整配置，包括个性、行为模式、LLM配置等。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NPCPreset {
    /// 预设唯一标识
    pub id: String,
    /// 预设名称
    pub name: String,
    /// 预设描述
    pub description: String,
    /// 预设类别
    pub category: NPCPresetCategory,

    // 性格参数
    /// 友好度（0.0-1.0）
    pub friendliness: f32,
    /// 攻击性（0.0-1.0）
    pub aggression: f32,
    /// 好奇心（0.0-1.0）
    pub curiosity: f32,
    /// 恐惧（0.0-1.0）
    pub fear: f32,
    /// 勇气（0.0-1.0）
    pub bravery: f32,
    /// 贪婪（0.0-1.0）
    pub greed: f32,
    /// 正式程度（0.0-1.0）
    pub formality: f32,
    /// 幽默感（0.0-1.0）
    pub humor: f32,

    // AI配置
    /// 混合模式
    pub hybrid_mode: HybridMode,
    /// 是否启用LLM
    pub enable_llm: bool,
    /// LLM模型选择
    pub llm_model: Option<String>,
    /// 复杂度阈值
    pub complexity_threshold: f32,

    // 行为设置
    /// 初始情绪状态
    pub initial_mood: MoodState,
    /// 推荐行为树模板（JSON格式）
    pub behavior_tree_template: Option<String>,
    /// 对话风格提示词
    pub dialogue_style_prompt: String,
    /// 示例对话
    pub sample_dialogues: Vec<String>,

    // 元数据
    /// 标签
    pub tags: Vec<String>,
    /// 作者
    pub author: String,
    /// 版本
    pub version: String,
}

/// NPC预设类别
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum NPCPresetCategory {
    /// 友好NPC
    Friendly,
    /// 敌对NPC
    Hostile,
    /// 中立NPC
    Neutral,
    /// 特殊NPC
    Special,
    /// 商人
    Merchant,
    /// 守卫
    Guard,
    /// 任务发布者
    QuestGiver,
    /// 自定义
    Custom,
}

impl NPCPreset {
    /// 创建预设builder
    pub fn builder() -> NPCPresetBuilder {
        NPCPresetBuilder::new()
    }

    /// 转换为Personality
    pub fn to_personality(&self) -> Personality {
        Personality {
            friendliness: self.friendliness,
            formality: self.formality,
            humor: self.humor,
            bravery: self.bravery,
            greed: self.greed,
            custom_traits: {
                let mut traits = HashMap::new();
                traits.insert("aggression".to_string(), self.aggression);
                traits.insert("curiosity".to_string(), self.curiosity);
                traits.insert("fear".to_string(), self.fear);
                traits
            },
        }
    }

    /// 转换为NPCConfig
    pub fn to_npc_config(&self) -> NPCConfig {
        NPCConfig {
            enable_llm: self.enable_llm,
            llm_latency_threshold: 2.0,
            complexity_threshold: self.complexity_threshold,
            adaptive_adjustment_interval: 60,
            min_confidence: 0.5,
        }
    }

    /// 获取完整的对话系统提示词
    pub fn get_system_prompt(&self) -> String {
        format!(
            "You are playing the role of {}, {}. \n\n\
             Personality traits:\n\
             - Friendliness: {:.0}%\n\
             - Aggression: {:.0}%\n\
             - Curiosity: {:.0}%\n\
             - Fear: {:.0}%\n\
             - Bravery: {:.0}%\n\
             - Greed: {:.0}%\n\
             - Formality: {:.0}%\n\
             - Humor: {:.0}%\n\n\
             Dialogue style: {}\n\n\
             Stay in character and respond naturally.",
            self.name,
            self.description,
            self.friendliness * 100.0,
            self.aggression * 100.0,
            self.curiosity * 100.0,
            self.fear * 100.0,
            self.bravery * 100.0,
            self.greed * 100.0,
            self.formality * 100.0,
            self.humor * 100.0,
            self.dialogue_style_prompt
        )
    }

    /// 应用到IntelligentNPC
    pub fn apply_to_npc(&self, npc: &mut IntelligentNPC) {
        npc.set_hybrid_mode(self.hybrid_mode);
        // 注意：这里需要通过某种方式设置个性，可能需要扩展IntelligentNPC的API
    }
}

/// NPC预设构建器
///
/// 提供流式API来创建自定义NPC预设。
pub struct NPCPresetBuilder {
    preset: NPCPreset,
}

impl NPCPresetBuilder {
    pub fn new() -> Self {
        Self {
            preset: NPCPreset {
                id: String::new(),
                name: String::new(),
                description: String::new(),
                category: NPCPresetCategory::Custom,
                friendliness: 0.5,
                aggression: 0.5,
                curiosity: 0.5,
                fear: 0.5,
                bravery: 0.5,
                greed: 0.5,
                formality: 0.5,
                humor: 0.5,
                hybrid_mode: HybridMode::Hybrid,
                enable_llm: true,
                llm_model: None,
                complexity_threshold: 0.6,
                initial_mood: MoodState::default(),
                behavior_tree_template: None,
                dialogue_style_prompt: String::new(),
                sample_dialogues: Vec::new(),
                tags: Vec::new(),
                author: "Custom".to_string(),
                version: "1.0.0".to_string(),
            },
        }
    }

    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.preset.id = id.into();
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.preset.name = name.into();
        self
    }

    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.preset.description = description.into();
        self
    }

    pub fn category(mut self, category: NPCPresetCategory) -> Self {
        self.preset.category = category;
        self
    }

    pub fn friendliness(mut self, value: f32) -> Self {
        self.preset.friendliness = value.clamp(0.0, 1.0);
        self
    }

    pub fn aggression(mut self, value: f32) -> Self {
        self.preset.aggression = value.clamp(0.0, 1.0);
        self
    }

    pub fn curiosity(mut self, value: f32) -> Self {
        self.preset.curiosity = value.clamp(0.0, 1.0);
        self
    }

    pub fn fear(mut self, value: f32) -> Self {
        self.preset.fear = value.clamp(0.0, 1.0);
        self
    }

    pub fn bravery(mut self, value: f32) -> Self {
        self.preset.bravery = value.clamp(0.0, 1.0);
        self
    }

    pub fn greed(mut self, value: f32) -> Self {
        self.preset.greed = value.clamp(0.0, 1.0);
        self
    }

    pub fn formality(mut self, value: f32) -> Self {
        self.preset.formality = value.clamp(0.0, 1.0);
        self
    }

    pub fn humor(mut self, value: f32) -> Self {
        self.preset.humor = value.clamp(0.0, 1.0);
        self
    }

    pub fn hybrid_mode(mut self, mode: HybridMode) -> Self {
        self.preset.hybrid_mode = mode;
        self
    }

    pub fn enable_llm(mut self, enable: bool) -> Self {
        self.preset.enable_llm = enable;
        self
    }

    pub fn dialogue_style(mut self, style: impl Into<String>) -> Self {
        self.preset.dialogue_style_prompt = style.into();
        self
    }

    pub fn add_sample_dialogue(mut self, dialogue: impl Into<String>) -> Self {
        self.preset.sample_dialogues.push(dialogue.into());
        self
    }

    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.preset.tags.push(tag.into());
        self
    }

    pub fn build(self) -> Result<NPCPreset, String> {
        if self.preset.name.is_empty() {
            return Err("Preset name cannot be empty".to_string());
        }
        if self.preset.id.is_empty() {
            return Err("Preset ID cannot be empty".to_string());
        }
        Ok(self.preset)
    }
}

impl Default for NPCPresetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 预设管理器
///
/// 管理所有可用的NPC预设。
pub struct PresetManager {
    presets: HashMap<String, NPCPreset>,
}

impl PresetManager {
    pub fn new() -> Self {
        let mut manager = Self {
            presets: HashMap::new(),
        };
        manager.load_builtin_presets();
        manager
    }

    /// 加载内置预设
    fn load_builtin_presets(&mut self) {
        // 友好商人
        self.presets.insert(
            "friendly_merchant".to_string(),
            NPCPreset {
                id: "friendly_merchant".to_string(),
                name: "Friendly Merchant".to_string(),
                description: "A cheerful and helpful merchant who loves to trade".to_string(),
                category: NPCPresetCategory::Merchant,
                friendliness: 0.9,
                aggression: 0.1,
                curiosity: 0.6,
                fear: 0.4,
                bravery: 0.5,
                greed: 0.8,
                formality: 0.3,
                humor: 0.7,
                hybrid_mode: HybridMode::Hybrid,
                enable_llm: true,
                llm_model: Some("gpt-3.5-turbo".to_string()),
                complexity_threshold: 0.5,
                initial_mood: MoodState {
                    happiness: 0.9,
                    anger: 0.1,
                    fear: 0.2,
                    trust: 0.7,
                },
                behavior_tree_template: Some(include_str!("trees/merchant.json").to_string()),
                dialogue_style_prompt: "Cheerful, helpful, always looking for a good deal. Uses casual language and occasional business jargon.".to_string(),
                sample_dialogues: vec![
                    "Welcome, welcome! Have a look at my wares!".to_string(),
                    "Ah, a customer with discerning taste! This item is specially priced for you.".to_string(),
                    "Business has been good today! What can I get for you?".to_string(),
                ],
                tags: vec!["merchant".to_string(), "friendly".to_string(), "trade".to_string()],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 激进守卫
        self.presets.insert(
            "aggressive_guard".to_string(),
            NPCPreset {
                id: "aggressive_guard".to_string(),
                name: "Aggressive Guard".to_string(),
                description: "A stern and suspicious guard who takes duty seriously".to_string(),
                category: NPCPresetCategory::Guard,
                friendliness: 0.2,
                aggression: 0.8,
                curiosity: 0.3,
                fear: 0.2,
                bravery: 0.9,
                greed: 0.3,
                formality: 0.7,
                humor: 0.1,
                hybrid_mode: HybridMode::TraditionalOnly,
                enable_llm: false,
                llm_model: None,
                complexity_threshold: 0.7,
                initial_mood: MoodState {
                    happiness: 0.4,
                    anger: 0.3,
                    fear: 0.1,
                    trust: 0.2,
                },
                behavior_tree_template: Some(include_str!("trees/guard.json").to_string()),
                dialogue_style_prompt:
                    "Stern, authoritative, suspicious. Uses formal language and commands."
                        .to_string(),
                sample_dialogues: vec![
                    "Halt! Identify yourself!".to_string(),
                    "Move along, nothing to see here.".to_string(),
                    "You're not authorized to be here.".to_string(),
                ],
                tags: vec![
                    "guard".to_string(),
                    "hostile".to_string(),
                    "authority".to_string(),
                ],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 好奇村民
        self.presets.insert(
            "curious_villager".to_string(),
            NPCPreset {
                id: "curious_villager".to_string(),
                name: "Curious Villager".to_string(),
                description: "An inquisitive villager who loves to gossip and learn new things".to_string(),
                category: NPCPresetCategory::Friendly,
                friendliness: 0.8,
                aggression: 0.1,
                curiosity: 0.95,
                fear: 0.3,
                bravery: 0.4,
                greed: 0.2,
                formality: 0.2,
                humor: 0.6,
                hybrid_mode: HybridMode::Hybrid,
                enable_llm: true,
                llm_model: Some("gpt-3.5-turbo".to_string()),
                complexity_threshold: 0.4,
                initial_mood: MoodState {
                    happiness: 0.8,
                    anger: 0.1,
                    fear: 0.2,
                    trust: 0.6,
                },
                behavior_tree_template: None,
                dialogue_style_prompt: "Inquisitive, friendly, talks a lot. Uses casual language and asks many questions.".to_string(),
                sample_dialogues: vec![
                    "Oh! A new face in town! Where are you from?".to_string(),
                    "Did you hear what happened at the old mill?".to_string(),
                    "I've never seen equipment like yours before!".to_string(),
                ],
                tags: vec!["villager".to_string(), "friendly".to_string(), "curious".to_string()],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 智慧长者
        self.presets.insert(
            "wise_elder".to_string(),
            NPCPreset {
                id: "wise_elder".to_string(),
                name: "Wise Elder".to_string(),
                description: "A knowledgeable elder who offers wisdom and guidance".to_string(),
                category: NPCPresetCategory::QuestGiver,
                friendliness: 0.7,
                aggression: 0.0,
                curiosity: 0.5,
                fear: 0.2,
                bravery: 0.6,
                greed: 0.1,
                formality: 0.8,
                humor: 0.4,
                hybrid_mode: HybridMode::Hybrid,
                enable_llm: true,
                llm_model: Some("gpt-4".to_string()),
                complexity_threshold: 0.6,
                initial_mood: MoodState {
                    happiness: 0.7,
                    anger: 0.0,
                    fear: 0.1,
                    trust: 0.6,
                },
                behavior_tree_template: None,
                dialogue_style_prompt: "Wise, speaks in metaphors and proverbs, offers guidance. Uses formal but warm language.".to_string(),
                sample_dialogues: vec![
                    "Patience, young one. The path to wisdom is long.".to_string(),
                    "I have seen many seasons come and go. Your trouble is but a leaf in the wind.".to_string(),
                    "Seek not the answers without, but look within.".to_string(),
                ],
                tags: vec!["elder".to_string(), "wise".to_string(), "quest_giver".to_string()],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 顽皮儿童
        self.presets.insert(
            "playful_child".to_string(),
            NPCPreset {
                id: "playful_child".to_string(),
                name: "Playful Child".to_string(),
                description: "An energetic and playful child who loves games".to_string(),
                category: NPCPresetCategory::Friendly,
                friendliness: 0.95,
                aggression: 0.0,
                curiosity: 0.9,
                fear: 0.5,
                bravery: 0.3,
                greed: 0.4,
                formality: 0.0,
                humor: 0.9,
                hybrid_mode: HybridMode::TraditionalOnly,
                enable_llm: false,
                llm_model: None,
                complexity_threshold: 0.3,
                initial_mood: MoodState {
                    happiness: 0.95,
                    anger: 0.0,
                    fear: 0.3,
                    trust: 0.8,
                },
                behavior_tree_template: None,
                dialogue_style_prompt:
                    "Energetic, simple language, lots of exclamations, playful tone.".to_string(),
                sample_dialogues: vec![
                    "Let's play tag!".to_string(),
                    "Wow! You're so strong!".to_string(),
                    "Can you show me that cool sword again?".to_string(),
                ],
                tags: vec![
                    "child".to_string(),
                    "playful".to_string(),
                    "friendly".to_string(),
                ],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 神秘陌生人
        self.presets.insert(
            "mysterious_stranger".to_string(),
            NPCPreset {
                id: "mysterious_stranger".to_string(),
                name: "Mysterious Stranger".to_string(),
                description: "An enigmatic figure with unclear motives".to_string(),
                category: NPCPresetCategory::Special,
                friendliness: 0.4,
                aggression: 0.3,
                curiosity: 0.7,
                fear: 0.3,
                bravery: 0.7,
                greed: 0.5,
                formality: 0.5,
                humor: 0.2,
                hybrid_mode: HybridMode::LLMOnly,
                enable_llm: true,
                llm_model: Some("gpt-4".to_string()),
                complexity_threshold: 0.8,
                initial_mood: MoodState {
                    happiness: 0.5,
                    anger: 0.2,
                    fear: 0.2,
                    trust: 0.3,
                },
                behavior_tree_template: None,
                dialogue_style_prompt:
                    "Cryptic, speaks in riddles, reveals little. Uses neutral and vague language."
                        .to_string(),
                sample_dialogues: vec![
                    "The threads of fate are complex...".to_string(),
                    "You seek answers, but are you ready for the truth?".to_string(),
                    "All in good time... all in good time.".to_string(),
                ],
                tags: vec![
                    "mysterious".to_string(),
                    "special".to_string(),
                    "cryptic".to_string(),
                ],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 勇敢骑士
        self.presets.insert(
            "brave_knight".to_string(),
            NPCPreset {
                id: "brave_knight".to_string(),
                name: "Brave Knight".to_string(),
                description: "A noble and courageous knight dedicated to protecting others"
                    .to_string(),
                category: NPCPresetCategory::Guard,
                friendliness: 0.6,
                aggression: 0.6,
                curiosity: 0.3,
                fear: 0.1,
                bravery: 0.95,
                greed: 0.2,
                formality: 0.6,
                humor: 0.3,
                hybrid_mode: HybridMode::Hybrid,
                enable_llm: true,
                llm_model: Some("gpt-3.5-turbo".to_string()),
                complexity_threshold: 0.5,
                initial_mood: MoodState {
                    happiness: 0.6,
                    anger: 0.2,
                    fear: 0.0,
                    trust: 0.5,
                },
                behavior_tree_template: Some(include_str!("trees/knight.json").to_string()),
                dialogue_style_prompt:
                    "Noble, courageous, speaks of honor and duty. Uses formal but warm language."
                        .to_string(),
                sample_dialogues: vec![
                    "Fear not, citizen! I shall protect you.".to_string(),
                    "My blade is at the service of the innocent.".to_string(),
                    "Stand back! This foe is dangerous.".to_string(),
                ],
                tags: vec![
                    "knight".to_string(),
                    "brave".to_string(),
                    "noble".to_string(),
                ],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 狡猾盗贼
        self.presets.insert(
            "cunning_thief".to_string(),
            NPCPreset {
                id: "cunning_thief".to_string(),
                name: "Cunning Thief".to_string(),
                description: "A clever and elusive thief who lives in the shadows".to_string(),
                category: NPCPresetCategory::Neutral,
                friendliness: 0.3,
                aggression: 0.2,
                curiosity: 0.8,
                fear: 0.6,
                bravery: 0.5,
                greed: 0.9,
                formality: 0.2,
                humor: 0.5,
                hybrid_mode: HybridMode::Hybrid,
                enable_llm: true,
                llm_model: Some("gpt-3.5-turbo".to_string()),
                complexity_threshold: 0.6,
                initial_mood: MoodState {
                    happiness: 0.5,
                    anger: 0.1,
                    fear: 0.4,
                    trust: 0.2,
                },
                behavior_tree_template: None,
                dialogue_style_prompt: "Sly, cunning, talks about opportunities and deals. Uses informal language and street slang.".to_string(),
                sample_dialogues: vec![
                    "Psst... looking for something... unusual?".to_string(),
                    "I might know where to find what you seek... for a price.".to_string(),
                    "Keep your voice down, unless you want the guards on us.".to_string(),
                ],
                tags: vec!["thief".to_string(), "cunning".to_string(), "neutral".to_string()],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 高贵法师
        self.presets.insert(
            "noble_mage".to_string(),
            NPCPreset {
                id: "noble_mage".to_string(),
                name: "Noble Mage".to_string(),
                description: "A learned and powerful magic user with vast knowledge".to_string(),
                category: NPCPresetCategory::QuestGiver,
                friendliness: 0.5,
                aggression: 0.2,
                curiosity: 0.8,
                fear: 0.2,
                bravery: 0.7,
                greed: 0.3,
                formality: 0.9,
                humor: 0.3,
                hybrid_mode: HybridMode::LLMOnly,
                enable_llm: true,
                llm_model: Some("gpt-4".to_string()),
                complexity_threshold: 0.7,
                initial_mood: MoodState {
                    happiness: 0.6,
                    anger: 0.1,
                    fear: 0.1,
                    trust: 0.4,
                },
                behavior_tree_template: None,
                dialogue_style_prompt: "Scholarly, speaks of arcane matters, dignified. Uses formal and academic language.".to_string(),
                sample_dialogues: vec![
                    "The arcane currents are strong today.".to_string(),
                    "Your aura suggests great potential... or great danger.".to_string(),
                    "I have researched this phenomenon extensively.".to_string(),
                ],
                tags: vec!["mage".to_string(), "noble".to_string(), "magic".to_string()],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 谦逊农夫
        self.presets.insert(
            "humble_farmer".to_string(),
            NPCPreset {
                id: "humble_farmer".to_string(),
                name: "Humble Farmer".to_string(),
                description: "A hardworking and simple farmer who tends to the land".to_string(),
                category: NPCPresetCategory::Friendly,
                friendliness: 0.8,
                aggression: 0.0,
                curiosity: 0.4,
                fear: 0.4,
                bravery: 0.4,
                greed: 0.2,
                formality: 0.1,
                humor: 0.5,
                hybrid_mode: HybridMode::TraditionalOnly,
                enable_llm: false,
                llm_model: None,
                complexity_threshold: 0.3,
                initial_mood: MoodState {
                    happiness: 0.7,
                    anger: 0.1,
                    fear: 0.2,
                    trust: 0.7,
                },
                behavior_tree_template: None,
                dialogue_style_prompt: "Simple, humble, talks about crops and weather. Uses rural dialect and simple language.".to_string(),
                sample_dialogues: vec![
                    "The harvest's been good this year, praise be.".to_string(),
                    "Need any fresh vegetables? Just picked 'em this morning!".to_string(),
                    "Hard work builds character, that's what my pa always said.".to_string(),
                ],
                tags: vec!["farmer".to_string(), "humble".to_string(), "friendly".to_string()],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );

        // 忠诚随从
        self.presets.insert(
            "loyal_servant".to_string(),
            NPCPreset {
                id: "loyal_servant".to_string(),
                name: "Loyal Servant".to_string(),
                description: "A devoted and loyal servant ready to assist".to_string(),
                category: NPCPresetCategory::Friendly,
                friendliness: 0.7,
                aggression: 0.2,
                curiosity: 0.3,
                fear: 0.3,
                bravery: 0.6,
                greed: 0.1,
                formality: 0.8,
                humor: 0.2,
                hybrid_mode: HybridMode::TraditionalOnly,
                enable_llm: false,
                llm_model: None,
                complexity_threshold: 0.4,
                initial_mood: MoodState {
                    happiness: 0.6,
                    anger: 0.1,
                    fear: 0.2,
                    trust: 0.8,
                },
                behavior_tree_template: None,
                dialogue_style_prompt:
                    "Deferential, service-oriented, proper. Uses formal and respectful language."
                        .to_string(),
                sample_dialogues: vec![
                    "How may I be of service, milord?".to_string(),
                    "Right away, sir. Your wish is my command.".to_string(),
                    "I live to serve. What are your orders?".to_string(),
                ],
                tags: vec![
                    "servant".to_string(),
                    "loyal".to_string(),
                    "friendly".to_string(),
                ],
                author: "System".to_string(),
                version: "1.0.0".to_string(),
            },
        );
    }

    /// 获取预设
    pub fn get_preset(&self, id: &str) -> Option<&NPCPreset> {
        self.presets.get(id)
    }

    /// 获取所有预设
    pub fn get_all_presets(&self) -> Vec<&NPCPreset> {
        self.presets.values().collect()
    }

    /// 按类别获取预设
    pub fn get_presets_by_category(&self, category: NPCPresetCategory) -> Vec<&NPCPreset> {
        self.presets.values().filter(|p| p.category == category).collect()
    }

    /// 按标签搜索预设
    pub fn search_by_tag(&self, tag: &str) -> Vec<&NPCPreset> {
        self.presets.values().filter(|p| p.tags.iter().any(|t| t == tag)).collect()
    }

    /// 添加自定义预设
    pub fn add_preset(&mut self, preset: NPCPreset) -> Result<(), String> {
        if self.presets.contains_key(&preset.id) {
            return Err(format!("Preset with id '{}' already exists", preset.id));
        }
        self.presets.insert(preset.id.clone(), preset);
        Ok(())
    }

    /// 移除预设
    pub fn remove_preset(&mut self, id: &str) -> Option<NPCPreset> {
        self.presets.remove(id)
    }

    /// 获取预设数量
    pub fn preset_count(&self) -> usize {
        self.presets.len()
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_manager_creation() {
        let manager = PresetManager::new();
        assert!(manager.preset_count() > 0);
    }

    #[test]
    fn test_get_preset() {
        let manager = PresetManager::new();
        let preset = manager.get_preset("friendly_merchant");
        assert!(preset.is_some());
        assert_eq!(preset.unwrap().name, "Friendly Merchant");
    }

    #[test]
    fn test_preset_to_personality() {
        let manager = PresetManager::new();
        let preset = manager.get_preset("friendly_merchant").unwrap();
        let personality = preset.to_personality();
        assert_eq!(personality.friendliness, 0.9);
        assert_eq!(personality.greed, 0.8);
    }

    #[test]
    fn test_preset_builder() {
        let preset = NPCPreset::builder()
            .id("test_preset")
            .name("Test Preset")
            .description("A test preset")
            .friendliness(0.7)
            .aggression(0.2)
            .build()
            .unwrap();

        assert_eq!(preset.id, "test_preset");
        assert_eq!(preset.friendliness, 0.7);
        assert_eq!(preset.aggression, 0.2);
    }

    #[test]
    fn test_preset_builder_validation() {
        let result = NPCPreset::builder().friendliness(0.7).build();
        assert!(result.is_err());
    }

    #[test]
    fn test_get_presets_by_category() {
        let manager = PresetManager::new();
        let merchants = manager.get_presets_by_category(NPCPresetCategory::Merchant);
        assert!(!merchants.is_empty());
    }
}
