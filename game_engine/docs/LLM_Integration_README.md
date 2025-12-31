# LLM集成接口快速开始指南

## 概述

P2-1 LLM集成接口为游戏引擎提供了统一的AI服务抽象，支持多种大语言模型提供商，用于创建智能NPC和动态游戏内容。

## 特性

- 🤖 **统一接口**: 一个trait支持多种LLM提供商
- 🎭 **智能NPC**: 支持对话生成、行为决策、内容生成
- 🔄 **混合模式**: 传统AI与LLM智能结合
- 📊 **性能监控**: 内置统计和自适应优化
- 🔌 **可扩展**: 易于添加新的LLM提供商

## 快速开始

### 1. 启用AI功能

编辑 `Cargo.toml` 或使用feature flags:

```bash
# 完整AI功能
cargo build --features ai

# 或仅启用特定提供商
cargo build --features ai-openai   # 仅OpenAI
cargo build --features ai-claude    # 仅Claude
cargo build --features ai-local     # 仅本地LLM
```

### 2. 设置API密钥 (可选)

如果使用OpenAI或Claude:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
```

### 3. 基本使用

```rust
use game_engine::ai::{OpenAIAdapter, AIService};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建适配器
    let adapter = Arc::new(OpenAIAdapter::new("api-key", "gpt-4"));

    // 使用AIService trait
    let dialogue = adapter.generate_dialogue(&context).await?;
    println!("NPC说: {}", dialogue);

    Ok(())
}
```

## 核心概念

### AIService Trait

所有LLM提供商实现的统一接口:

```rust
#[async_trait]
pub trait AIService: Send + Sync {
    async fn generate_dialogue(&self, context: &NPCContext) -> Result<String, AIError>;
    async fn decide_action(&self, situation: &Situation) -> Result<Action, AIError>;
    async fn generate_content(&self, prompt: &ContentPrompt) -> Result<GeneratedContent, AIError>;
    async fn health_check(&self) -> Result<(), AIError>;
}
```

### 智能NPC

```rust
use game_engine::ai::{IntelligentNPC, HybridMode};

let mut npc = IntelligentNPC::new(entity_id)
    .with_llm_service(Arc::new(adapter))
    .with_hybrid_mode(HybridMode::Hybrid);

let action = npc.decide().await?;
```

#### 混合模式

- **TraditionalOnly**: 仅使用行为树/状态机
- **LLMOnly**: 仅使用LLM决策
- **Hybrid**: 根据复杂度自动选择
- **Adaptive**: 基于性能统计自适应调整

## 配置示例

### OpenAI配置

```rust
let adapter = OpenAIAdapter::new("api-key", "gpt-4")
    .with_max_tokens(150)
    .with_temperature(0.7);
```

### Claude配置

```rust
let adapter = ClaudeAdapter::new("api-key", "claude-3-opus-20240229")
    .with_max_tokens(150)
    .with_temperature(0.7);
```

### 本地LLM配置

```rust
let adapter = LocalLLMAdapter::new("models/llama-2-7b.gguf", LLMRuntime::LlamaCpp)
    .with_context_size(2048)
    .with_threads(4)
    .with_gpu_layers(32);
```

## NPC配置示例

查看 `/examples/ai_npc_config.json` 获取完整配置示例。

```json
{
  "npcs": [
    {
      "id": "merchant_01",
      "name": "Friendly Merchant",
      "ai_mode": "hybrid",
      "personality": {
        "friendliness": 0.9,
        "formality": 0.3,
        "humor": 0.5
      },
      "llm_config": {
        "provider": "openai",
        "model": "gpt-4"
      }
    }
  ]
}
```

## 完整示例

查看 `/examples/llm_integration_example.rs` 获取详细代码示例。

```bash
cargo run --example llm_integration_example --features ai
```

## 测试

运行单元测试:

```bash
cargo test --package game_engine --lib ai::llm_tests --features ai
```

## 性能考虑

### API延迟
- OpenAI/Claude: 通常200-2000ms
- 本地LLM: 取决于硬件，通常500-5000ms

### 优化建议
1. 使用混合模式减少LLM调用
2. 实现响应缓存
3. 批量处理NPC决策
4. 使用本地模型降低成本

## 错误处理

```rust
match adapter.generate_dialogue(&context).await {
    Ok(dialogue) => println!("{}", dialogue),
    Err(AIError::RateLimitError) => {
        eprintln!("速率限制，请稍后重试");
    }
    Err(AIError::AuthenticationError) => {
        eprintln!("API密钥无效");
    }
    Err(e) => {
        eprintln!("错误: {}", e);
    }
}
```

## 文档

- **技术总结**: `/docs/P2-1_LLM_Integration_Summary.md`
- **任务报告**: `/docs/P2-1_Task_Completion_Report.md`
- **配置示例**: `/examples/ai_npc_config.json`
- **代码示例**: `/examples/llm_integration_example.rs`

## 常见问题

### Q: 如何选择LLM提供商?
**A:**
- **OpenAI**: 模型质量高，支持广泛
- **Claude**: 长文本处理优秀，更安全
- **本地LLM**: 无API费用，隐私保护，需硬件支持

### Q: 混合模式如何工作?
**A:** 混合模式根据情境复杂度自动选择:
- 低复杂度 (< 0.6): 使用传统AI (快速)
- 高复杂度 (>= 0.6): 使用LLM (智能)

### Q: 如何降低API成本?
**A:**
1. 使用混合模式减少LLM调用
2. 设置合理的max_tokens限制
3. 考虑使用本地LLM
4. 实现响应缓存机制

### Q: 本地LLM需要什么硬件?
**A:**
- **CPU**: 4核心以上推荐
- **RAM**: 8GB以上
- **GPU**: 可选，但显著加速 (需要CUDA/Metal)

## 后续开发

1. 添加响应缓存
2. 实现流式对话
3. 支持更多LLM提供商
4. 优化提示工程
5. 添加性能监控面板

## 许可证

MIT License - 详见项目根目录LICENSE文件

## 联系方式

- 项目仓库: [GitHub](https://github.com/username/game_engine)
- 问题反馈: [Issues](https://github.com/username/game_engine/issues)

---

**最后更新**: 2025-12-31
**版本**: v0.1.0 (P2-1完成)
