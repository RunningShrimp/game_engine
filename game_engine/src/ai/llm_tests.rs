//! # LLM集成单元测试
//!
//! 本模块包含LLM集成接口的单元测试。

use super::npc::{HybridMode, IntelligentNPC, NPCConfig, NPCManager};
use super::service::*;
use bevy_ecs::entity::Entity;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Mock AI服务用于测试
struct MockAIService {
    should_fail: bool,
}

#[async_trait::async_trait]
impl AIService for MockAIService {
    async fn generate_dialogue(&self, context: &NPCContext) -> Result<String, AIError> {
        if self.should_fail {
            return Err(AIError::ApiError("Mock error".to_string()));
        }
        Ok(format!("Hello from {:?}", context.environment.location))
    }

    async fn decide_action(&self, _situation: &Situation) -> Result<Action, AIError> {
        if self.should_fail {
            return Err(AIError::ApiError("Mock error".to_string()));
        }
        Ok(Action {
            action_type: ActionType::Wait,
            parameters: HashMap::new(),
            priority: 0.5,
            estimated_duration: Some(Duration::from_secs(1)),
        })
    }

    async fn generate_content(&self, prompt: &ContentPrompt) -> Result<GeneratedContent, AIError> {
        if self.should_fail {
            return Err(AIError::ApiError("Mock error".to_string()));
        }
        Ok(GeneratedContent {
            content: format!("Generated content for {:?}", prompt.content_type),
            content_type: prompt.content_type.clone(),
            tokens_used: Some(50),
            confidence: 0.9,
            metadata: HashMap::new(),
        })
    }

    async fn health_check(&self) -> Result<(), AIError> {
        if self.should_fail {
            Err(AIError::ApiError("Mock error".to_string()))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod llm_integration_tests {
    use super::*;

    fn create_test_context() -> NPCContext {
        NPCContext {
            npc_id: Entity::from_raw(1),
            player_state: PlayerState {
                level: 10,
                reputation: HashMap::new(),
                health: 100.0,
                max_health: 100.0,
                inventory: vec!["sword".to_string()],
                completed_quests: vec![],
            },
            environment: EnvironmentState {
                location: "Town".to_string(),
                game_time: "12:00".to_string(),
                weather: "Sunny".to_string(),
                nearby_entities: vec!["merchant".to_string()],
                in_combat: false,
            },
            conversation_history: vec![],
            personality: Personality::default(),
            current_quest: None,
            mood: MoodState::default(),
        }
    }

    fn create_test_situation() -> Situation {
        Situation {
            nearby_entities: vec![Entity::from_raw(2)],
            current_goal: Some("Patrol".to_string()),
            available_actions: vec![
                Action {
                    action_type: ActionType::Move {
                        target: [10.0, 0.0, 10.0],
                    },
                    parameters: HashMap::new(),
                    priority: 0.8,
                    estimated_duration: Some(Duration::from_secs(5)),
                },
                Action {
                    action_type: ActionType::Wait,
                    parameters: HashMap::new(),
                    priority: 0.2,
                    estimated_duration: Some(Duration::from_secs(1)),
                },
            ],
            time_constraints: None,
            npc_status: NPCStatus::Idle,
            perceived_threats: vec![],
            resources: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_mock_ai_service_generate_dialogue() {
        let service = MockAIService { should_fail: false };
        let context = create_test_context();

        let result = service.generate_dialogue(&context).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Town"));
    }

    #[tokio::test]
    async fn test_mock_ai_service_generate_dialogue_failure() {
        let service = MockAIService { should_fail: true };
        let context = create_test_context();

        let result = service.generate_dialogue(&context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_ai_service_decide_action() {
        let service = MockAIService { should_fail: false };
        let situation = create_test_situation();

        let result = service.decide_action(&situation).await;
        assert!(result.is_ok());
        let action = result.unwrap();
        assert!(matches!(action.action_type, ActionType::Wait));
    }

    #[tokio::test]
    async fn test_mock_ai_service_generate_content() {
        let service = MockAIService { should_fail: false };
        let prompt = ContentPrompt {
            content_type: ContentType::QuestDescription,
            prompt: "Create a quest".to_string(),
            constraints: vec!["Short".to_string()],
            max_length: Some(100),
            style: Some("Epic".to_string()),
        };

        let result = service.generate_content(&prompt).await;
        assert!(result.is_ok());
        let content = result.unwrap();
        assert_eq!(content.tokens_used, Some(50));
        assert_eq!(content.confidence, 0.9);
    }

    #[tokio::test]
    async fn test_mock_ai_service_health_check() {
        let service = MockAIService { should_fail: false };
        let result = service.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_intelligent_npc_traditional_only() {
        let mut npc =
            IntelligentNPC::new(Entity::from_raw(1)).with_hybrid_mode(HybridMode::TraditionalOnly);

        let situation = create_test_situation();
        npc.update_situation(situation);

        let action = npc.decide().await;
        assert_eq!(action.action_type, ActionType::Wait);
        assert_eq!(npc.get_performance_stats().traditional_calls, 1);
    }

    #[tokio::test]
    async fn test_intelligent_npc_llm_only() {
        let service = Arc::new(MockAIService { should_fail: false });
        let mut npc = IntelligentNPC::new(Entity::from_raw(1))
            .with_hybrid_mode(HybridMode::LLMOnly)
            .with_llm_service(service);

        let situation = create_test_situation();
        npc.update_situation(situation);

        let action = npc.decide().await;
        assert_eq!(action.action_type, ActionType::Wait);
        assert_eq!(npc.get_performance_stats().llm_calls, 1);
    }

    #[tokio::test]
    async fn test_intelligent_npc_hybrid_low_complexity() {
        let service = Arc::new(MockAIService { should_fail: false });
        let mut npc = IntelligentNPC::new(Entity::from_raw(1))
            .with_hybrid_mode(HybridMode::Hybrid)
            .with_llm_service(service)
            .with_config(NPCConfig {
                complexity_threshold: 0.8,
                ..Default::default()
            });

        let situation = create_test_situation();
        npc.update_situation(situation);

        let action = npc.decide().await;
        // 低复杂度应该使用传统AI
        assert_eq!(npc.get_performance_stats().traditional_calls, 1);
        assert_eq!(npc.get_performance_stats().llm_calls, 0);
    }

    #[tokio::test]
    async fn test_intelligent_npc_generate_dialogue() {
        let service = Arc::new(MockAIService { should_fail: false });
        let npc = IntelligentNPC::new(Entity::from_raw(1)).with_llm_service(service);

        let context = create_test_context();
        let result = npc.generate_dialogue(&context).await;

        assert!(result.is_ok());
        assert!(result.unwrap().contains("Town"));
    }

    #[tokio::test]
    async fn test_npc_manager() {
        let service = Arc::new(MockAIService { should_fail: false });
        let mut manager = NPCManager::new();
        manager.set_default_llm_service(service);

        let npc1 =
            IntelligentNPC::new(Entity::from_raw(1)).with_hybrid_mode(HybridMode::TraditionalOnly);
        let npc2 = IntelligentNPC::new(Entity::from_raw(2)).with_hybrid_mode(HybridMode::LLMOnly);

        manager.add_npc(npc1);
        manager.add_npc(npc2);

        assert_eq!(manager.npcs.len(), 2);

        // 更新情境
        let mut situations = HashMap::new();
        situations.insert(Entity::from_raw(1), create_test_situation());
        situations.insert(Entity::from_raw(2), create_test_situation());
        manager.update_situations(situations);

        // 批量决策
        let actions = manager.decide_all().await;
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn test_action_type_serialization() {
        let action = Action {
            action_type: ActionType::Move {
                target: [1.0, 2.0, 3.0],
            },
            parameters: HashMap::new(),
            priority: 0.8,
            estimated_duration: Some(Duration::from_secs(5)),
        };

        let serialized = serde_json::to_string(&action).unwrap();
        assert!(serialized.contains("Move"));

        let deserialized: Action = serde_json::from_str(&serialized).unwrap();
        match deserialized.action_type {
            ActionType::Move { target } => {
                assert_eq!(target, [1.0, 2.0, 3.0]);
            }
            _ => panic!("Expected Move action"),
        }
    }

    #[test]
    fn test_personality_traits() {
        let personality = Personality {
            friendliness: 0.9,
            formality: 0.2,
            humor: 0.8,
            bravery: 0.7,
            greed: 0.1,
            custom_traits: {
                let mut traits = HashMap::new();
                traits.insert("loyalty".to_string(), 0.95);
                traits
            },
        };

        assert_eq!(personality.friendliness, 0.9);
        assert_eq!(personality.custom_traits.get("loyalty"), Some(&0.95));
    }

    #[test]
    fn test_mood_state() {
        let mood = MoodState {
            happiness: 0.8,
            anger: 0.1,
            fear: 0.05,
            trust: 0.7,
        };

        assert_eq!(mood.happiness, 0.8);
        assert!(mood.fear < 0.1);
    }

    #[test]
    fn test_npc_status() {
        let status = NPCStatus::Idle;
        assert!(matches!(status, NPCStatus::Idle));

        let status = NPCStatus::InCombat;
        assert!(matches!(status, NPCStatus::InCombat));
    }

    #[test]
    fn test_content_type() {
        let content_type = ContentType::QuestDescription;
        assert!(matches!(content_type, ContentType::QuestDescription));

        let custom_type = ContentType::Custom("MyCustomType".to_string());
        match custom_type {
            ContentType::Custom(s) => assert_eq!(s, "MyCustomType"),
            _ => panic!("Expected Custom type"),
        }
    }

    #[test]
    fn test_generated_content() {
        let content = GeneratedContent {
            content: "Test content".to_string(),
            content_type: ContentType::QuestDescription,
            tokens_used: Some(100),
            confidence: 0.95,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert(
                    "model".to_string(),
                    serde_json::Value::String("gpt-4".to_string()),
                );
                meta
            },
        };

        assert_eq!(content.content, "Test content");
        assert_eq!(content.tokens_used, Some(100));
        assert_eq!(content.confidence, 0.95);
    }

    #[test]
    fn test_ai_error_display() {
        let error = AIError::ApiError("Test error".to_string());
        assert!(format!("{}", error).contains("Test error"));

        let error = AIError::AuthenticationError;
        assert!(format!("{}", error).contains("Authentication"));

        let error = AIError::RateLimitError;
        assert!(format!("{}", error).contains("Rate limit"));

        let error = AIError::TimeoutError;
        assert!(format!("{}", error).contains("timed out"));
    }
}

// 集成测试（需要实际API密钥）
#[cfg(feature = "ai-integration-tests")]
mod integration_tests {
    use super::*;

    #[tokio::test]
    #[ignore] // 需要API密钥才能运行
    async fn test_openai_adapter_integration() {
        // 在实际环境中设置API密钥
        let api_key = std::env::var("OPENAI_API_KEY").unwrap();
        let adapter = crate::ai::openai::OpenAIAdapter::new(&api_key, "gpt-3.5-turbo");

        let result = adapter.health_check().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore] // 需要API密钥才能运行
    async fn test_claude_adapter_integration() {
        let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap();
        let adapter = crate::ai::claude::ClaudeAdapter::new(&api_key, "claude-3-opus-20240229");

        let result = adapter.health_check().await;
        assert!(result.is_ok());
    }
}
