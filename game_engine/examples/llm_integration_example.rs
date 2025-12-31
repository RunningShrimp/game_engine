//! # LLM集成使用示例
//!
//! 本示例展示如何使用LLM集成接口创建智能NPC。

use game_engine::ai::{
    claude::ClaudeAdapter,
    local::{LLMRuntime, LocalLLMAdapter},
    npc::{HybridMode, IntelligentNPC, NPCConfig, NPCManager},
    openai::OpenAIAdapter,
    service::{
        AIService, Action, ActionType, ContentPrompt, ContentType, EnvironmentState, Message,
        MoodState, NPCContext, NPCStatus, Personality, PlayerState, Situation,
    },
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("=== LLM集成示例 ===\n");

    // 示例1: 使用OpenAI适配器
    #[cfg(feature = "ai-openai")]
    {
        println!("1. OpenAI适配器示例");
        let openai_adapter = setup_openai_adapter();
        demonstrate_dialogue_generation(openai_adapter).await?;
        println!();
    }

    // 示例2: 使用Claude适配器
    #[cfg(feature = "ai-claude")]
    {
        println!("2. Claude适配器示例");
        let claude_adapter = setup_claude_adapter();
        demonstrate_action_decision(claude_adapter).await?;
        println!();
    }

    // 示例3: 使用本地LLM适配器
    #[cfg(feature = "ai-local")]
    {
        println!("3. 本地LLM适配器示例");
        let local_adapter = setup_local_adapter();
        demonstrate_content_generation(local_adapter).await?;
        println!();
    }

    // 示例4: 混合模式NPC
    println!("4. 混合模式NPC示例");
    demonstrate_hybrid_npc().await?;
    println!();

    // 示例5: NPC管理器
    println!("5. NPC管理器示例");
    demonstrate_npc_manager().await?;
    println!();

    println!("=== 示例完成 ===");

    Ok(())
}

/// 设置OpenAI适配器
#[cfg(feature = "ai-openai")]
fn setup_openai_adapter() -> OpenAIAdapter {
    let api_key = std::env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("警告: 未设置OPENAI_API_KEY环境变量");
        "demo-key".to_string()
    });

    OpenAIAdapter::new(&api_key, "gpt-4").with_max_tokens(150).with_temperature(0.7)
}

/// 设置Claude适配器
#[cfg(feature = "ai-claude")]
fn setup_claude_adapter() -> ClaudeAdapter {
    let api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_else(|_| {
        println!("警告: 未设置ANTHROPIC_API_KEY环境变量");
        "demo-key".to_string()
    });

    ClaudeAdapter::new(&api_key, "claude-3-opus-20240229")
        .with_max_tokens(150)
        .with_temperature(0.7)
}

/// 设置本地LLM适配器
#[cfg(feature = "ai-local")]
fn setup_local_adapter() -> LocalLLMAdapter {
    LocalLLMAdapter::new("models/llama-2-7b.gguf", LLMRuntime::LlamaCpp)
        .with_context_size(2048)
        .with_threads(4)
        .with_gpu_layers(32)
}

/// 演示对话生成
async fn demonstrate_dialogue_generation(
    service: impl AIService,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  演示NPC对话生成...");

    let context = create_merchant_context();

    match service.generate_dialogue(&context).await {
        Ok(dialogue) => {
            println!("  生成的对话: {}", dialogue);
        }
        Err(e) => {
            println!("  对话生成失败: {}", e);
        }
    }

    Ok(())
}

/// 演示行为决策
async fn demonstrate_action_decision(
    service: impl AIService,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  演示NPC行为决策...");

    let situation = create_guard_situation();

    match service.decide_action(&situation).await {
        Ok(action) => {
            println!("  决策的动作: {:?}", action.action_type);
            println!("  优先级: {:.2}", action.priority);
        }
        Err(e) => {
            println!("  行为决策失败: {}", e);
        }
    }

    Ok(())
}

/// 演示内容生成
async fn demonstrate_content_generation(
    service: impl AIService,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("  演示游戏内容生成...");

    let prompt = ContentPrompt {
        content_type: ContentType::QuestDescription,
        prompt: "Create a quest about finding a lost artifact".to_string(),
        constraints: vec![
            "Keep it under 100 words".to_string(),
            "Make it mysterious".to_string(),
        ],
        max_length: Some(100),
        style: Some("Adventure".to_string()),
    };

    match service.generate_content(&prompt).await {
        Ok(content) => {
            println!("  生成的内容:");
            println!("  {}", content.content);
            if let Some(tokens) = content.tokens_used {
                println!("  使用的token数: {}", tokens);
            }
            println!("  置信度: {:.2}", content.confidence);
        }
        Err(e) => {
            println!("  内容生成失败: {}", e);
        }
    }

    Ok(())
}

/// 演示混合模式NPC
async fn demonstrate_hybrid_npc() -> Result<(), Box<dyn std::error::Error>> {
    println!("  创建混合模式NPC...");

    // 创建模拟的LLM服务（实际使用时替换为真实的OpenAI或Claude服务）
    struct MockAIService;

    #[async_trait::async_trait]
    impl AIService for MockAIService {
        async fn generate_dialogue(
            &self,
            _context: &NPCContext,
        ) -> Result<String, game_engine::ai::service::AIError> {
            Ok("Hello, traveler! Welcome to our shop.".to_string())
        }

        async fn decide_action(
            &self,
            _situation: &Situation,
        ) -> Result<Action, game_engine::ai::service::AIError> {
            Ok(Action {
                action_type: ActionType::Speak {
                    message: "Greetings!".to_string(),
                },
                parameters: HashMap::new(),
                priority: 0.8,
                estimated_duration: Some(Duration::from_secs(2)),
            })
        }

        async fn generate_content(
            &self,
            _prompt: &ContentPrompt,
        ) -> Result<game_engine::ai::service::GeneratedContent, game_engine::ai::service::AIError>
        {
            Ok(game_engine::ai::service::GeneratedContent {
                content: "Generated quest content".to_string(),
                content_type: ContentType::QuestDescription,
                tokens_used: Some(50),
                confidence: 0.9,
                metadata: HashMap::new(),
            })
        }

        async fn health_check(&self) -> Result<(), game_engine::ai::service::AIError> {
            Ok(())
        }
    }

    let llm_service = Arc::new(MockAIService);

    let mut npc = IntelligentNPC::new(bevy_ecs::entity::Entity::from_raw(1))
        .with_llm_service(llm_service)
        .with_hybrid_mode(HybridMode::Hybrid)
        .with_config(NPCConfig {
            complexity_threshold: 0.6,
            ..Default::default()
        });

    // 更新情境
    let situation = create_guard_situation();
    npc.update_situation(situation);

    // 执行决策
    let action = npc.decide().await;
    println!("  NPC决策: {:?}", action.action_type);

    // 查看性能统计
    let stats = npc.get_performance_stats();
    println!("  LLM调用次数: {}", stats.llm_calls);
    println!("  传统AI调用次数: {}", stats.traditional_calls);

    Ok(())
}

/// 演示NPC管理器
async fn demonstrate_npc_manager() -> Result<(), Box<dyn std::error::Error>> {
    println!("  创建NPC管理器...");

    struct MockAIService;

    #[async_trait::async_trait]
    impl AIService for MockAIService {
        async fn generate_dialogue(
            &self,
            _context: &NPCContext,
        ) -> Result<String, game_engine::ai::service::AIError> {
            Ok("Mock response".to_string())
        }

        async fn decide_action(
            &self,
            _situation: &Situation,
        ) -> Result<Action, game_engine::ai::service::AIError> {
            Ok(Action {
                action_type: ActionType::Wait,
                parameters: HashMap::new(),
                priority: 0.5,
                estimated_duration: Some(Duration::from_secs(1)),
            })
        }

        async fn generate_content(
            &self,
            _prompt: &ContentPrompt,
        ) -> Result<game_engine::ai::service::GeneratedContent, game_engine::ai::service::AIError>
        {
            Ok(game_engine::ai::service::GeneratedContent {
                content: "Mock content".to_string(),
                content_type: ContentType::QuestDescription,
                tokens_used: Some(10),
                confidence: 1.0,
                metadata: HashMap::new(),
            })
        }

        async fn health_check(&self) -> Result<(), game_engine::ai::service::AIError> {
            Ok(())
        }
    }

    let mut manager = NPCManager::new();
    manager.set_default_llm_service(Arc::new(MockAIService));

    // 添加多个NPC
    for i in 1..=3 {
        let npc = IntelligentNPC::new(bevy_ecs::entity::Entity::from_raw(i))
            .with_hybrid_mode(HybridMode::Hybrid);
        manager.add_npc(npc);
    }

    println!("  已添加 {} 个NPC", manager.npcs.len());

    // 批量决策
    let actions = manager.decide_all().await;
    println!("  批量决策完成，生成 {} 个动作", actions.len());

    // 查看所有NPC的性能统计
    let all_stats = manager.get_all_stats();
    println!("  NPC性能统计:");
    for (entity_id, stats) in all_stats {
        println!(
            "    NPC {}: {} 次LLM调用, {} 次传统AI调用",
            entity_id.index(),
            stats.llm_calls,
            stats.traditional_calls
        );
    }

    Ok(())
}

/// 创建商人NPC上下文
fn create_merchant_context() -> NPCContext {
    NPCContext {
        npc_id: bevy_ecs::entity::Entity::from_raw(1),
        player_state: PlayerState {
            level: 5,
            reputation: {
                let mut rep = HashMap::new();
                rep.insert("merchants".to_string(), 10);
                rep
            },
            health: 80.0,
            max_health: 100.0,
            inventory: vec!["gold coins".to_string(), "sword".to_string()],
            completed_quests: vec![],
        },
        environment: EnvironmentState {
            location: "Marketplace".to_string(),
            game_time: "14:30".to_string(),
            weather: "Sunny".to_string(),
            nearby_entities: vec!["player".to_string(), "guard".to_string()],
            in_combat: false,
        },
        conversation_history: vec![],
        personality: Personality {
            friendliness: 0.9,
            formality: 0.3,
            humor: 0.5,
            bravery: 0.4,
            greed: 0.7,
            ..Default::default()
        },
        current_quest: None,
        mood: MoodState {
            happiness: 0.8,
            anger: 0.1,
            fear: 0.05,
            trust: 0.6,
        },
    }
}

/// 创建守卫NPC情境
fn create_guard_situation() -> Situation {
    Situation {
        nearby_entities: vec![
            bevy_ecs::entity::Entity::from_raw(2),
            bevy_ecs::entity::Entity::from_raw(3),
        ],
        current_goal: Some("Patrol the city streets".to_string()),
        available_actions: vec![
            Action {
                action_type: ActionType::Move {
                    target: [10.0, 0.0, 10.0],
                },
                parameters: HashMap::new(),
                priority: 0.7,
                estimated_duration: Some(Duration::from_secs(5)),
            },
            Action {
                action_type: ActionType::Interact {
                    target: bevy_ecs::entity::Entity::from_raw(2),
                },
                parameters: HashMap::new(),
                priority: 0.9,
                estimated_duration: Some(Duration::from_secs(3)),
            },
            Action {
                action_type: ActionType::Wait,
                parameters: HashMap::new(),
                priority: 0.3,
                estimated_duration: Some(Duration::from_secs(2)),
            },
        ],
        time_constraints: None,
        npc_status: NPCStatus::Idle,
        perceived_threats: vec![],
        resources: HashMap::new(),
    }
}
