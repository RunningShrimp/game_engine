//! # NPU LLM推理演示
//!
//! 展示游戏引擎的NPU加速LLM推理功能，用于NPC AI和对话。

use game_engine::acceleration::llm::*;
use game_engine::acceleration::npus::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    println!("=== NPU LLM推理演示 ===\n");

    // 1. NPU设备检测
    demo_npu_detection().await?;

    // 2. 基础LLM推理
    demo_basic_llm().await?;

    // 3. NPC对话系统
    demo_npc_dialogue().await?;

    // 4. 流式对话
    demo_streaming_dialogue().await?;

    // 5. NPC行为决策
    demo_npc_behavior().await?;

    // 6. 性能统计
    demo_performance_stats().await?;

    println!("\n=== 演示完成 ===");

    Ok(())
}

/// NPU设备检测演示
async fn demo_npu_detection() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. NPU设备检测");
    println!("   {}", "=".repeat(50));

    // 检测最佳NPU设备
    let device_type = NPUDeviceType::detect_best_device();

    println!("   检测到的NPU设备: {}", device_type.name());
    println!("   硬件加速: {}", device_type.is_hardware_accelerated());

    // 创建NPU运行时
    let runtime = NPURuntime::new().await?;

    println!("   NPU运行时已初始化");

    // 获取设备信息
    let device_info = runtime.get_device_info();

    println!("\n   设备信息:");
    println!("   - 设备名称: {}", device_info.device_name);
    println!("   - 设备类型: {:?}", device_info.device_type);
    println!("   - FP16支持: {}", device_info.supports_fp16);

    if let Some(units) = device_info.compute_units {
        println!("   - 计算单元: {}", units);
    }

    if let Some(memory) = device_info.memory_size_mb {
        println!("   - 内存大小: {} MB", memory);
    }

    println!();
    Ok(())
}

/// 基础LLM推理演示
async fn demo_basic_llm() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. 基础LLM推理");
    println!("   {}", "=".repeat(50));

    // 注意：这个演示需要实际的模型文件
    // 在实际使用中，你需要准备一个量化模型
    let model_path = "models/llama-2-7b-quantized.mlmodel";

    // 检查模型文件是否存在
    if !std::path::PathBuf::from(model_path).exists() {
        println!("   ⚠ 模型文件不存在: {}", model_path);
        println!("   跳过LLM推理演示");
        println!("   提示: 请下载量化模型或使用CPU/GPU fallback\n");
        return Ok(());
    }

    // 创建LLM引擎
    let mut llm = NpuLlmEngine::new(model_path).await?;
    llm.initialize().await?;

    println!("   ✓ LLM模型已加载");

    // 简单推理示例
    let system_prompt = "You are a helpful assistant in a fantasy game.";
    let user_input = "Hello! What can you tell me about this world?";

    println!("\n   系统提示: {}", system_prompt);
    println!("   用户输入: {}", user_input);

    let start = Instant::now();
    let response = llm.chat(system_prompt, user_input).await?;
    let elapsed = start.elapsed();

    println!("\n   NPC响应:");
    println!("   {}", response);
    println!("\n   推理时间: {:?}", elapsed);

    // 获取性能统计
    let stats = llm.get_stats();
    println!("   累计推理: {} 次", stats.total_inferences);
    println!("   累计tokens: {}", stats.total_tokens);
    println!("   平均速度: {:.1} tokens/s", stats.average_tokens_per_second);

    println!();
    Ok(())
}

/// NPC对话系统演示
async fn demo_npc_dialogue() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. NPC对话系统");
    println!("   {}", "=".repeat(50));

    let model_path = "models/llama-2-7b-quantized.mlmodel";

    if !std::path::PathBuf::from(model_path).exists() {
        println!("   ⚠ 模型文件不存在，跳过NPC对话演示\n");
        return Ok(());
    }

    // 创建LLM引擎
    let mut llm = NpuLlmEngine::new(model_path).await?;
    llm.initialize().await?;

    // 定义商店老板NPC
    let shopkeeper_persona = NpcPersona {
        name: "Eldric the Wise".to_string(),
        description: "An old wizard who runs a magic shop in the village".to_string(),
        personality: vec![
            "wise".to_string(),
            "mysterious".to_string(),
            "friendly".to_string(),
            "knowledgeable".to_string(),
        ],
        backstory: "Once served as the royal court wizard for 50 years before retiring to open a small magic shop. Knows many secrets about the kingdom's history.".to_string(),
        dialogue_style: "Formal but warm, occasionally uses archaic words. Often starts sentences with 'Ah,' or 'Well,'. Speaks in a measured, thoughtful manner.".to_string(),
    };

    // 创建NPC AI
    let mut npc = NpcLlmAi::new(llm, shopkeeper_persona).await?;

    println!("   NPC: Eldric the Wise (商店老板)");
    println!("   角色: 魔法师");

    // 对话示例
    let conversations = vec![
        "Hello, do you have any magic swords for sale?",
        "I'm looking for something to defeat a dragon. Any suggestions?",
        "How much for this powerful staff?",
    ];

    for (i, player_input) in conversations.iter().enumerate() {
        println!("\n   --- 对话 {} ---", i + 1);
        println!("   玩家: {}", player_input);

        let start = Instant::now();
        let response = npc.talk(player_input).await?;
        let elapsed = start.elapsed();

        println!("   Eldric: {}", response);
        println!("   响应时间: {:?}", elapsed);
    }

    println!();
    Ok(())
}

/// 流式对话演示
async fn demo_streaming_dialogue() -> Result<(), Box<dyn std::error::Error>> {
    println!("4. 流式对话（实时显示）");
    println!("   {}", "=".repeat(50));

    let model_path = "models/llama-2-7b-quantized.mlmodel";

    if !std::path::PathBuf::from(model_path).exists() {
        println!("   ⚠ 模型文件不存在，跳过流式对话演示\n");
        return Ok(());
    }

    // 创建LLM引擎
    let mut llm = NpuLlmEngine::new(model_path).await?;
    llm.initialize().await?;

    // 定义吟游诗人NPC
    let bard_persona = NpcPersona {
        name: "Melody".to_string(),
        description: "A cheerful bard who loves to sing and tell stories".to_string(),
        personality: vec![
            "cheerful".to_string(),
            "creative".to_string(),
            "talkative".to_string(),
        ],
        backstory: "Traveled across many lands collecting songs and stories. Loves to share tales of adventure and romance.".to_string(),
        dialogue_style: "Expressive and poetic. Often uses metaphors and rhymes. Speaks with enthusiasm and emotion.".to_string(),
    };

    let mut npc = NpcLlmAi::new(llm, bard_persona).await?;

    println!("   NPC: Melody (吟游诗人)");

    // 流式对话
    let player_input = "Can you sing me a song about brave heroes?";

    println!("\n   玩家: {}", player_input);
    print!("   Melody: ");

    let mut rx = npc.talk_streaming(player_input).await?;

    // 实时显示响应
    while let Some(chunk) = rx.recv().await {
        print!("{}", chunk);
        std::io::stdout().flush()?;
    }

    println!("\n");
    Ok(())
}

/// NPC行为决策演示
async fn demo_npc_behavior() -> Result<(), Box<dyn std::error::Error>> {
    println!("5. NPC行为决策");
    println!("   {}", "=".repeat(50));

    let model_path = "models/llama-2-7b-quantized.mlmodel";

    if !std::path::PathBuf::from(model_path).exists() {
        println!("   ⚠ 模型文件不存在，跳过行为决策演示\n");
        return Ok(());
    }

    // 创建LLM引擎
    let mut llm = NpuLlmEngine::new(model_path).await?;
    llm.initialize().await?;

    // 定义战士NPC
    let warrior_persona = NpcPersona {
        name: "Theron".to_string(),
        description: "A brave warrior sworn to protect the village".to_string(),
        personality: vec![
            "brave".to_string(),
            "loyal".to_string(),
            "strategic".to_string(),
        ],
        backstory: "Former captain of the royal guard. Now protects the village from monsters and bandits.".to_string(),
        dialogue_style: "Direct and confident. Uses military terminology. Speaks with authority.".to_string(),
    };

    let mut npc = NpcLlmAi::new(llm, warrior_persona).await?;

    println!("   NPC: Theron (战士)");

    // 模拟不同的游戏场景
    let scenarios = vec![
        GameContext {
            health: 0.9,
            nearby_enemies: 0,
            nearby_allies: 3,
            objective: "Patrol the village".to_string(),
        },
        GameContext {
            health: 0.5,
            nearby_enemies: 5,
            nearby_allies: 1,
            objective: "Defend the village".to_string(),
        },
        GameContext {
            health: 0.2,
            nearby_enemies: 3,
            nearby_allies: 0,
            objective: "Survive".to_string(),
        },
    ];

    for (i, context) in scenarios.iter().enumerate() {
        println!("\n   --- 场景 {} ---", i + 1);
        println!("   生命值: {:.0}%", context.health * 100.0);
        println!("   附近敌人: {}", context.nearby_enemies);
        println!("   附近盟友: {}", context.nearby_allies);
        println!("   当前目标: {}", context.objective);

        let start = Instant::now();
        let action = npc.decide_action(context).await?;
        let elapsed = start.elapsed();

        println!("\n   决策:");
        println!("   - 动作: {:?}", action.action_type);
        println!("   - 原因: {}", action.reason);
        println!("   - 决策时间: {:?}", elapsed);
    }

    println!();
    Ok(())
}

/// 性能统计演示
async fn demo_performance_stats() -> Result<(), Box<dyn std::error::Error>> {
    println!("6. 性能统计");
    println!("   {}", "=".repeat(50));

    let model_path = "models/llama-2-7b-quantized.mlmodel";

    if !std::path::PathBuf::from(model_path).exists() {
        println!("   ⚠ 模型文件不存在，跳过性能统计演示\n");
        return Ok(());
    }

    // 创建LLM引擎
    let mut llm = NpuLlmEngine::new(model_path).await?;
    llm.initialize().await?;

    println!("   运行性能测试...\n");

    let test_prompts = vec![
        "Tell me about the history of this kingdom.",
        "What kind of creatures live in these lands?",
        "Can you recommend a good sword for a beginner?",
        "How do I defeat a dragon?",
        "What's the best way to travel safely?",
    ];

    let start = Instant::now();

    for (i, prompt) in test_prompts.iter().enumerate() {
        println!("   测试 {}/{}", i + 1, test_prompts.len());

        let _response = llm
            .chat(
                "You are a helpful game guide.",
                prompt,
            )
            .await?;

        // 显示当前统计
        let stats = llm.get_stats();
        println!(
            "   - 累计tokens: {}, 平均速度: {:.1} tokens/s",
            stats.total_tokens,
            stats.average_tokens_per_second
        );
    }

    let total_elapsed = start.elapsed();

    // 最终统计
    let stats = llm.get_stats();

    println!("\n   最终统计:");
    println!("   - 总推理次数: {}", stats.total_inferences);
    println!("   - 总生成tokens: {}", stats.total_tokens);
    println!("   - 总推理时间: {:.2}s", stats.total_inference_time);
    println!("   - 实际测试时间: {:.2}s", total_elapsed.as_secs_f32());
    println!("   - 平均速度: {:.1} tokens/s", stats.average_tokens_per_second);

    // 性能评估
    println!("\n   性能评估:");
    if stats.average_tokens_per_second >= 50.0 {
        println!("   ✓ 优秀 (>=50 tokens/s)");
    } else if stats.average_tokens_per_second >= 30.0 {
        println!("   ✓ 良好 (>=30 tokens/s)");
    } else if stats.average_tokens_per_second >= 10.0 {
        println!("   ⚠ 一般 (>=10 tokens/s)");
    } else {
        println!("   ⚠ 较慢 (<10 tokens/s)");
    }

    println!();
    Ok(())
}

/// 额外示例：多NPC对话
async fn demo_multi_npc_dialogue() -> Result<(), Box<dyn std::error::Error>> {
    println!("7. 多NPC对话");
    println!("   {}", "=".repeat(50));

    let model_path = "models/llama-2-7b-quantized.mlmodel";

    if !std::path::PathBuf::from(model_path).exists() {
        println!("   ⚠ 模型文件不存在，跳过多NPC对话演示\n");
        return Ok(());
    }

    // 创建多个NPC
    let personas = vec![
        NpcPersona {
            name: "King Aldric".to_string(),
            description: "The wise ruler of the kingdom".to_string(),
            personality: vec!["noble".to_string(), "just".to_string()],
            backstory: "Has ruled for 30 years, known for fairness and wisdom".to_string(),
            dialogue_style: "Formal and regal".to_string(),
        },
        NpcPersona {
            name: "General Mara".to_string(),
            description: "The kingdom's military commander".to_string(),
            personality: vec!["stern".to_string(), "tactical".to_string()],
            backstory: "Veteran of many wars, highly disciplined".to_string(),
            dialogue_style: "Direct and military".to_string(),
        },
    ];

    println!("   场景: 国王和将军讨论防御策略");
    println!();

    // 注意：这需要为每个NPC创建独立的LLM引擎
    // 实际实现中可能需要使用模型共享或批处理

    println!("   (多NPC对话需要多个LLM实例，此处省略)\n");

    Ok(())
}
