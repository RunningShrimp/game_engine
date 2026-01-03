# AI辅助工具使用指南

## 概述

游戏引擎集成了强大的AI辅助开发工具，提供代码生成、代码审查、测试生成和代码分析功能，帮助开发者提升开发效率和代码质量。

## 功能特性

### 1. 代码生成（AICodeGenerator）
- 基于描述自动生成代码
- 支持ECS组件、系统、资源、事件等多种模板
- 智能上下文感知
- 代码优化建议

### 2. 代码审查（AICodeReviewer）
- 自动化代码审查
- Bug检测和性能问题识别
- 安全漏洞检测
- 最佳实践检查
- 代码风格验证

### 3. 测试生成（AITestGenerator）
- 自动生成单元测试
- 测试覆盖分析
- 测试用例推荐
- 边界和错误场景覆盖

### 4. 代码分析（AICodeAnalyzer）
- 代码质量评分
- 复杂度分析
- 性能瓶颈检测
- 重构建议

## 配置

### 环境变量配置

创建 `.env` 文件或设置环境变量：

```bash
# OpenAI配置
export OPENAI_API_KEY="sk-..."
export OPENAI_MODEL="gpt-4"        # 可选，默认gpt-4
export OPENAI_ENDPOINT="https://api.openai.com/v1/chat/completions"  # 可选

# Anthropic配置
export ANTHROPIC_API_KEY="sk-ant-..."
export ANTHROPIC_MODEL="claude-3-opus-20240229"  # 可选

# 本地模型配置（Ollama）
export LOCAL_MODEL_ENDPOINT="http://localhost:11434/api/generate"
export LOCAL_MODEL_NAME="llama2"  # 或 codellama, mistral等
```

### 代码配置

```rust
use game_engine::tools::ai_assistant::{AIConfig, AIProvider};

// 从环境变量加载
let config = AIConfig::from_env(AIProvider::OpenAI)?;

// 或手动创建
let config = AIConfig::openai("your-api-key")
    .with_model("gpt-4")
    .with_temperature(0.7)
    .with_max_tokens(2048);

// 本地模型
let config = AIConfig::local(
    "http://localhost:11434/api/generate",
    "llama2"
);
```

## 使用示例

### 1. 代码生成

```rust
use game_engine::tools::ai_assistant::{AICodeGenerator, CodeGenerationRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AIConfig::from_env(AIProvider::OpenAI)?;
    let generator = AICodeGenerator::new(config);

    // 生成ECS组件
    let request = CodeGenerationRequest {
        language: "rust".to_string(),
        description: "Create a health component for game entities".to_string(),
        context: "ECS component for game engine".to_string(),
        extra_params: None,
    };

    let code = generator.generate(request).await?;
    println!("Generated code:\n{}", code);

    Ok(())
}
```

### 2. 代码审查

```rust
use game_engine::tools::ai_assistant::AICodeReviewer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AIConfig::from_env(AIProvider::Anthropic)?;
    let reviewer = AICodeReviewer::new(config);

    let code = r#"
fn process_data(data: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    for item in data {
        if item > 0 {
            result.push(item * 2);
        }
    }
    result
}
"#;

    let report = reviewer.review(code, "rust").await?;
    println!("Code score: {}", report.score);
    for issue in &report.issues {
        println!("{}: {} at line {}", issue.severity, issue.message, issue.line);
    }

    Ok(())
}
```

### 3. 测试生成

```rust
use game_engine::tools::ai_assistant::AITestGenerator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AIConfig::from_env(AIProvider::OpenAI)?;
    let test_gen = AITestGenerator::new(config);

    let source_code = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn divide(a: f32, b: f32) -> Result<f32, String> {
    if b == 0.0 {
        Err("Division by zero".to_string())
    } else {
        Ok(a / b)
    }
}
"#;

    let result = test_gen.generate(source_code).await?;
    println!("Generated {} tests:", result.test_count);
    println!("{}", result.test_code);

    // 分析测试覆盖
    let coverage = test_gen.analyze_coverage(source_code, &result.test_code).await?;
    println!("Coverage: {:.1}%", coverage.overall_coverage * 100.0);

    Ok(())
}
```

### 4. 代码分析

```rust
use game_engine::tools::ai_assistant::AICodeAnalyzer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AIConfig::from_env(AIProvider::OpenAI)?;
    let analyzer = AICodeAnalyzer::new(config);

    let code = r#"
fn calculate_metrics(data: &Vec<i32>) -> f32 {
    let mut sum = 0;
    for i in 0..data.len() {
        for j in 0..data.len() {
            sum += data[i] * data[j];
        }
    }
    sum as f32 / (data.len() * data.len()) as f32
}
"#;

    // 质量分析
    let quality_report = analyzer.analyze_quality(code, "rust").await?;
    println!("Quality score: {}", quality_report.overall_score);
    println!("Complexity: {}", quality_report.complexity_score);

    // 性能分析
    let perf_analysis = analyzer.analyze_performance(code, "rust").await?;
    for bottleneck in &perf_analysis.bottlenecks {
        println!("Bottleneck: {} - {}", bottleneck.type_, bottleneck.description);
    }

    // 重构建议
    let refactoring = analyzer.suggest_refactoring(code, "rust").await?;
    for suggestion in &refactoring.suggestions {
        println!("{}: {}", suggestion.priority, suggestion.title);
    }

    Ok(())
}
```

### 5. 综合使用AI助手

```rust
use game_engine::tools::ai_assistant::AIAssistant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AIConfig::from_env(AIProvider::OpenAI)?;
    let assistant = AIAssistant::new(config);

    // 生成代码
    let code_request = CodeGenerationRequest {
        language: "rust".to_string(),
        description: "Create a player movement system".to_string(),
        context: "ECS system for game engine".to_string(),
        extra_params: None,
    };

    let code = assistant.generate_code(code_request).await?;
    println!("Generated:\n{}", code);

    // 审查生成的代码
    let review = assistant.review_code(&code, "rust").await?;
    println!("Review score: {}", review.score);

    // 生成测试
    let tests = assistant.generate_test(&code).await?;
    println!("Generated {} tests", tests.test_count);

    Ok(())
}
```

## ECS集成

在游戏引擎中使用AI助手作为ECS资源：

```rust
use bevy_ecs::prelude::*;
use game_engine::tools::ai_assistant::{AIAssistant, AIAssistantResource};

fn main() {
    let mut app = App::new();

    // 添加AI助手作为资源
    let config = AIConfig::from_env(AIProvider::OpenAI)
        .expect("Failed to load AI config");
    let assistant = AIAssistant::new(config);

    app.insert_resource(AIAssistantResource { assistant });

    // 在系统中使用
    app.add_systems(Update, ai_assisted_code_generation);
}

fn ai_assisted_code_generation(
    ai: Res<AIAssistantResource>,
    mut commands: Commands,
) {
    // 在游戏系统中使用AI生成代码
}
```

## 代码模板

AI生成器支持以下模板：

### ECS组件模板
```rust
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: f32,
    pub max: f32,
}
```

### ECS系统模板
```rust
pub fn health_system(
    mut query: Query<&mut Health>,
    time: Res<Time>,
) {
    for mut health in query.iter_mut() {
        // System logic
    }
}
```

### 事件模板
```rust
#[derive(Event, Debug, Clone)]
pub struct DamageEvent {
    pub entity: Entity,
    pub amount: f32,
}
```

### 插件模板
```rust
pub struct HealthPlugin;

impl Plugin for HealthPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HealthConfig>()
           .add_event::<DamageEvent>()
           .add_systems(Update, health_system);
    }
}
```

## 最佳实践

### 1. 选择合适的提供商
- **OpenAI GPT-4**: 代码质量最佳，适合生产环境
- **Anthropic Claude**: 长文本处理能力强，适合大型项目
- **本地模型**: 隐私保护，离线使用，成本低

### 2. 提示词编写
- 描述要具体明确
- 提供足够的上下文
- 指定代码风格和约定
- 包含使用场景

### 3. API密钥管理
```rust
// 使用环境变量，不要硬编码
let config = AIConfig::from_env(AIProvider::OpenAI)?;

// 或使用配置管理工具
let api_key = std::env::var("OPENAI_API_KEY")
    .expect("OPENAI_API_KEY must be set");
```

### 4. 错误处理
```rust
match generator.generate(request).await {
    Ok(code) => println!("Success: {}", code),
    Err(AIError::RateLimited) => {
        eprintln!("Rate limited, retry later");
    }
    Err(AIError::ApiError(msg)) => {
        eprintln!("API error: {}", msg);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### 5. 性能优化
- 使用本地模型减少延迟
- 缓存常用的生成结果
- 批量处理多个请求
- 合理设置max_tokens限制

## 故障排除

### 问题：API密钥无效
```
Error: Invalid API key
```
**解决方案**: 检查环境变量是否正确设置，验证API密钥是否有效。

### 问题：请求超时
```
Error: Network error: deadline has elapsed
```
**解决方案**:
- 检查网络连接
- 使用本地模型
- 增加超时时间

### 问题：生成质量不佳
**解决方案**:
- 使用更高质量的模型（GPT-4 > GPT-3.5）
- 降低temperature参数（0.2-0.5）
- 提供更详细的上下文
- 明确代码风格要求

### 问题：本地模型连接失败
```
Error: Local model request failed
```
**解决方案**:
- 确保Ollama正在运行：`ollama serve`
- 检查端点地址：`http://localhost:11434/api/generate`
- 验证模型是否已下载：`ollama list`

## 进阶功能

### 代码优化
```rust
let optimized = generator.optimize_code(
    original_code,
    "rust"
).await?;

println!("Improvements:");
for improvement in &optimized.improvements {
    println!("- {}", improvement);
}
println!("Performance gain: {}", optimized.performance_gain);
```

### 测试推荐
```rust
let recommendations = test_gen.recommend_tests(source_code).await?;

for rec in recommendations {
    println!("{}: {} - {}",
        rec.priority, rec.function_name, rec.description
    );
}
```

### 最佳实践检查
```rust
let bp_report = reviewer.check_best_practices(code, "rust").await?;

println!("Best practices score: {}", bp_report.score);
for violation in &bp_report.violations {
    println!("Violation: {}", violation);
}
```

## 参考资料

- [OpenAI API文档](https://platform.openai.com/docs)
- [Anthropic API文档](https://docs.anthropic.com)
- [Ollama文档](https://ollama.ai/docs)
- [bevy_ecs文档](https://docs.rs/bevy_ecs)

## 版本历史

- **v0.3.0** (2024-01-03)
  - 完善代码分析工具
  - 增强测试生成功能
  - 添加代码优化功能
  - 改进配置管理

- **v0.2.0** (2023-12-15)
  - 添加代码审查功能
  - 支持测试生成
  - 集成多个AI提供商

- **v0.1.0** (2023-12-01)
  - 初始版本
  - 基础代码生成功能
