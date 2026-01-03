//! AI辅助工具使用示例
//!
//! 展示如何使用游戏引擎的AI辅助开发工具
//!
//! 运行示例：
//! ```bash
//! # 设置环境变量
//! export OPENAI_API_KEY="your-api-key"
//!
//! # 运行示例
//! cargo run --example ai_assistant_example
//! ```

use game_engine::tools::ai_assistant::{
    AIAssistant, AIConfig, AIProvider, CodeGenerationRequest,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    println!("=== AI辅助工具使用示例 ===\n");

    // 1. 从环境变量加载配置
    println!("1. 加载AI配置...");
    let config = match AIConfig::from_env(AIProvider::OpenAI) {
        Ok(cfg) => {
            cfg.validate()?;
            println!("   ✓ OpenAI配置加载成功");
            cfg
        }
        Err(e) => {
            println!("   ✗ 无法从环境变量加载配置: {}", e);
            println!("   使用默认配置（需要手动设置API密钥）");
            AIConfig::openai("your-api-key-here")
        }
    };

    // 2. 创建AI助手
    println!("\n2. 创建AI助手...");
    let assistant = AIAssistant::new(config);
    println!("   ✓ AI助手创建成功");

    // 3. 生成ECS组件代码
    println!("\n3. 生成ECS组件代码...");
    let component_request = CodeGenerationRequest {
        language: "rust".to_string(),
        description: "Create a Health component with current and max health values"
            .to_string(),
        context: "ECS component for game engine".to_string(),
        extra_params: Some("Include Default implementation".to_string()),
    };

    match assistant.generate_code(component_request).await {
        Ok(code) => {
            println!("   ✓ 代码生成成功:\n");
            println!("   {}\n", code);
        }
        Err(e) => {
            println!("   ✗ 代码生成失败: {}\n", e);
            println!("   提示: 请确保设置了有效的API密钥");
        }
    }

    // 4. 生成系统代码
    println!("4. 生成ECS系统代码...");
    let system_request = CodeGenerationRequest {
        language: "rust".to_string(),
        description: "Create a health regeneration system that restores health over time"
            .to_string(),
        context: "ECS system for game engine".to_string(),
        extra_params: None,
    };

    match assistant.generate_code(system_request).await {
        Ok(code) => {
            println!("   ✓ 代码生成成功:\n");
            println!("   {}\n", code);
        }
        Err(e) => {
            println!("   ✗ 代码生成失败: {}\n", e);
        }
    }

    // 5. 代码审查示例
    println!("5. 代码审查示例...");
    let sample_code = r#"
pub fn process_entities(entities: &Vec<Entity>) -> Vec<Entity> {
    let mut result = Vec::new();
    for entity in entities {
        if entity.is_alive() {
            result.push(entity.clone());
        }
    }
    result
}
"#;

    match assistant.review_code(sample_code, "rust").await {
        Ok(report) => {
            println!("   ✓ 审查完成");
            println!("   代码评分: {}/100", report.score);
            if report.issues.is_empty() {
                println!("   未发现问题");
            } else {
                println!("   发现 {} 个问题:", report.issues.len());
                for issue in &report.issues {
                    println!(
                        "   - [{:?}] {}: {}",
                        issue.severity, issue.category, issue.message
                    );
                }
            }
            println!();
        }
        Err(e) => {
            println!("   ✗ 审查失败: {}\n", e);
        }
    }

    // 6. 测试生成示例
    println!("6. 测试生成示例...");
    let test_code = r#"
pub fn calculate_damage(base: f32, multiplier: f32) -> f32 {
    base * multiplier
}

pub fn apply_damage(current: f32, damage: f32) -> f32 {
    if damage > current {
        0.0
    } else {
        current - damage
    }
}
"#;

    match assistant.generate_test(test_code).await {
        Ok(result) => {
            println!("   ✓ 测试生成成功");
            println!("   生成了 {} 个测试\n", result.test_count);
            println!("   测试代码:\n{}", result.test_code);
        }
        Err(e) => {
            println!("   ✗ 测试生成失败: {}\n", e);
        }
    }

    println!("=== 示例完成 ===");
    println!("\n提示：");
    println!("- 设置环境变量 OPENAI_API_KEY 来使用OpenAI");
    println!("- 设置环境变量 ANTHROPIC_API_KEY 来使用Anthropic");
    println!("- 使用本地模型：设置 LOCAL_MODEL_ENDPOINT");
    println!("\n更多示例请参考 docs/AI_ASSISTANT_USER_GUIDE.md");

    Ok(())
}

#[allow(dead_code)]
fn demonstrate_config_builder() {
    // 演示配置构建器模式

    // OpenAI配置
    let _openai_config = AIConfig::openai("sk-...")
        .with_model("gpt-4")
        .with_temperature(0.7)
        .with_max_tokens(2048);

    // Anthropic配置
    let _anthropic_config = AIConfig::anthropic("sk-ant-...")
        .with_model("claude-3-opus-20240229")
        .with_temperature(0.5);

    // 本地模型配置
    let _local_config = AIConfig::local(
        "http://localhost:11434/api/generate",
        "llama2"
    ).with_temperature(0.3);
}
