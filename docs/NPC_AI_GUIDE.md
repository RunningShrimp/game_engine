# NPC & AI System Guide

**版本**: 1.0.0
**更新日期**: 2025-01-01
**作者**: Game Engine Team

## 目录

1. [概述](#概述)
2. [NPC预设系统](#npc预设系统)
3. [LLM集成](#llm集成)
4. [成本控制](#成本控制)
5. [行为树系统](#行为树系统)
6. [调试工具](#调试工具)
7. [最佳实践](#最佳实践)
8. [示例代码](#示例代码)

---

## 概述

游戏引擎的NPC/AI系统提供了三大核心功能：

1. **NPC预设系统** - 11个内置预设模板，5分钟内创建NPC
2. **LLM集成** - 支持OpenAI GPT-3.5/GPT-4的智能对话
3. **成本控制** - 自动缓存、预算限制、成本追踪

### 快速开始

```rust
use game_engine::ai::npc::presets::{PresetManager, NPCPresetCategory};

// 1. 创建预设管理器
let manager = PresetManager::new();

// 2. 获取预设
let merchant = manager.get_preset("friendly_merchant").unwrap();

// 3. 创建NPC
let npc = create_npc_with_preset(entity_id, "friendly_merchant");
```

---

## NPC预设系统

### 内置预设列表

引擎提供11个内置预设，涵盖常见NPC类型：

| 预设ID | 名称 | 类别 | 特点 |
|--------|------|------|------|
| `friendly_merchant` | 友好商人 | Merchant | 高友好度(0.9)、高贪婪(0.8)、爱讲笑话(0.7) |
| `aggressive_guard` | 激进守卫 | Guard | 低友好度(0.2)、高攻击性(0.8)、高勇气(0.9) |
| `curious_villager` | 好奇村民 | Friendly | 高好奇(0.95)、高友好度(0.8) |
| `wise_elder` | 智慧长者 | QuestGiver | 高正式度(0.8)、使用GPT-4 |
| `playful_child` | 顽皮儿童 | Friendly | 极高友好度(0.95)、极高幽默(0.9) |
| `mysterious_stranger` | 神秘陌生人 | Special | 中等性格、使用LLMOnly模式 |
| `brave_knight` | 勇敢骑士 | Guard | 高勇气(0.95)、中等攻击性(0.6) |
| `cunning_thief` | 狡猾盗贼 | Neutral | 高好奇(0.8)、极高贪婪(0.9) |
| `noble_mage` | 高贵法师 | QuestGiver | 高正式度(0.9)、高好奇(0.8) |
| `humble_farmer` | 谦逊农夫 | Friendly | 高友好度(0.8)、低正式度(0.1) |
| `loyal_servant` | 忠诚随从 | Friendly | 高信任(0.8)、高正式度(0.8) |

### 个性参数系统

每个预设有8个核心性格参数（范围0.0-1.0）：

```
友好度 (Friendliness)  ─┐
攻击性 (Aggression)    ─┤
好奇心 (Curiosity)    ─┼─> 决定NPC行为和对话风格
恐惧 (Fear)           ─┤
勇气 (Bravery)        ─┘
贪婪 (Greed)          ─┐
正式度 (Formality)    ─┤
幽默 (Humor)          ─┘
```

### 使用内置预设

```rust
use game_engine::ai::npc::presets::PresetManager;

fn main() {
    let manager = PresetManager::new();

    // 方法1: 通过ID获取
    let merchant = manager.get_preset("friendly_merchant").unwrap();
    println!("{}", merchant.name); // "Friendly Merchant"

    // 方法2: 按类别筛选
    let guards = manager.get_presets_by_category(NPCPresetCategory::Guard);
    for guard in guards {
        println!("{} - {}", guard.name, guard.description);
    }

    // 方法3: 按标签搜索
    let friendly_npcs = manager.search_by_tag("friendly");
}
```

### 创建自定义预设

```rust
use game_engine::ai::npc::presets::{NPCPreset, NPCPresetBuilder, NPCPresetCategory};
use game_engine::ai::npc::HybridMode;

let custom_preset = NPCPreset::builder()
    .id("tavern_keeper")
    .name("Tavern Keeper")
    .description("A jovial innkeeper who loves to share stories")
    .category(NPCPresetCategory::Merchant)
    .friendliness(0.85)
    .aggression(0.1)
    .curiosity(0.7)
    .fear(0.3)
    .bravery(0.5)
    .greed(0.6)
    .formality(0.2)
    .humor(0.8)
    .hybrid_mode(HybridMode::Hybrid)
    .enable_llm(true)
    .dialogue_style("Jovial, hospitable, loves to share local gossip.")
    .add_sample_dialogue("Welcome, traveler! Come in, come in!")
    .add_sample_dialogue("Ah, you've come to the right place!")
    .add_tag("innkeeper")
    .add_tag("friendly")
    .build()
    .unwrap();

// 添加到管理器
let mut manager = PresetManager::new();
manager.add_preset(custom_preset);
```

### 预设配置选项

#### Hybrid Mode（混合模式）

- `HybridMode::TraditionalOnly` - 仅使用传统AI（行为树）
- `HybridMode::LLMOnly` - 仅使用LLM（适合复杂对话）
- `HybridMode::Hybrid` - 混合模式（根据复杂度自动选择）

#### LLM配置

```rust
.enable_llm(true)              // 启用LLM
.llm_model("gpt-4".to_string()) // 指定模型
.complexity_threshold(0.6)     // 混合模式阈值
```

---

## LLM集成

### 支持的模型

| 模型 | 输入价格 | 输出价格 | 推荐用途 |
|------|----------|----------|----------|
| gpt-4 | $0.03/1K | $0.06/1K | 复杂对话、Quest Giver |
| gpt-3.5-turbo | $0.0015/1K | $0.002/1K | 日常对话、Merchant |
| gpt-4-turbo | $0.01/1K | $0.03/1K | 平衡性能和成本 |

### LLM缓存系统

自动缓存LLM响应，减少重复调用：

```rust
use game_engine::ai::llm_cache::{LLMCache, CacheConfig, CacheKey};

// 创建缓存（默认1000条，24小时TTL）
let cache = LLMCache::new(CacheConfig::default());

// 生成缓存键
let key = CacheKey::from_prompt("npc_merchant", "Tell me about your shop.", "gpt-3.5-turbo");

// 检查缓存
if let Some(cached) = cache.get(&key) {
    return Ok(cached); // 命中缓存，直接返回
}

// 调用LLM API
let response = call_llm_api().await?;

// 存入缓存
cache.put(key, response.clone());
```

### 缓存配置

```rust
let config = CacheConfig {
    max_entries: 1000,              // 最大缓存条目
    ttl_seconds: 86400,             // 24小时
    enable_persistence: true,       // 启用持久化
    persistence_path: Some("llm_cache.json".to_string()),
    enable_semantic_search: false,  // 语义相似度匹配（实验性）
    similarity_threshold: 0.85,
};
```

### 缓存统计

```rust
let stats = cache.get_stats();
println!("缓存命中率: {:.1}%", stats.hit_rate() * 100.0);
println!("节省的API调用: {}", stats.saved_calls);
println!("节省的成本: ${:.4}", stats.saved_cost);
```

---

## 成本控制

### 预算配置

```rust
use game_engine::ai::cost_tracking::{BudgetConfig, CostTracker};

let config = BudgetConfig {
    daily_budget_usd: 10.0,       // 每日预算$10
    monthly_budget_usd: 100.0,    // 每月预算$100
    warning_threshold: 0.8,       // 80%时警告
    block_on_exceed: false,       // 超出预算是否阻止调用
    enable_budget_control: true,  // 启用预算控制
};

let tracker = CostTracker::new(config);
```

### 记录API调用

```rust
// 记录成功的调用
tracker.record_call(
    "gpt-3.5-turbo",  // 模型
    1000,             // 输入token
    500,              // 输出token
    "npc_merchant"    // NPC ID
).unwrap();

// 记录失败的调用
tracker.record_failed_call("gpt-4", "npc_elder", "Rate limit exceeded");
```

### 预算监控

```rust
use game_engine::ai::cost_tracking::BudgetStatus;

let status = tracker.get_budget_status();
let usage = tracker.get_budget_usage_percent();

match status {
    BudgetStatus::WithinBudget => println!("✓ 预算正常 ({:.1}%)", usage),
    BudgetStatus::NearBudget => println!("⚠️ 接近预算 ({:.1}%)", usage),
    BudgetStatus::OverBudget => println!("❌ 超出预算 ({:.1}%)", usage),
}
```

### 成本统计

```rust
// 获取24小时统计
let stats = tracker.get_statistics(86400);
println!("总调用次数: {}", stats.total_calls);
println!("总token数: {}", stats.total_tokens);
println!("总成本: ${:.4}", stats.total_cost_usd);
println!("平均成本: ${:.4}", stats.average_cost_per_call);

// 按模型统计
let model_stats = tracker.get_statistics_by_model(86400);
for stat in model_stats {
    println!("{}: {} calls, ${:.4}", stat.model, stat.call_count, stat.total_cost);
}

// 按NPC统计
let npc_stats = tracker.get_statistics_by_npc(86400);
for stat in npc_stats {
    println!("{}: {} calls, ${:.4}", stat.npc_id, stat.call_count, stat.total_cost);
}
```

### 导出报告

```rust
// 导出JSON
tracker.export_to_json("cost_report.json")?;

// 导出CSV
tracker.export_to_csv("cost_report.csv")?;

// 生成完整报告
let report = tracker.generate_report(86400);
println!("报告生成时间: {}", report.generated_at);
println!("预算状态: {:?}", report.budget_status);
```

---

## 行为树系统

### 基础节点类型

```rust
use game_engine::ai::behavior_tree::{Sequence, Selector, Action, Condition};

// Sequence顺序执行节点（全部成功才成功）
let sequence = Sequence {
    children: vec![
        Box::new(CheckHealth),
        Box::new(FindEnemy),
        Box::new(Attack),
    ],
};

// Selector选择节点（任一成功即成功）
let selector = Selector {
    children: vec![
        Box::new(Flee),      // 优先逃跑
        Box::new(Fight),     // 其次战斗
        Box::new(Hide),      // 最后隐藏
    ],
};
```

### 自定义节点

```rust
use game_engine::ai::behavior_tree::{Node, Status};

struct IsAliveCondition {
    health: f32,
}

impl Node for IsAliveCondition {
    fn tick(&mut self) -> Status {
        if self.health > 0.0 {
            Status::Success
        } else {
            Status::Failure
        }
    }
}

struct AttackAction;

impl Node for AttackAction {
    fn tick(&mut self) -> Status {
        println!("⚔️ 攻击目标...");
        Status::Success
    }
}
```

### 完整行为树示例

```rust
use game_engine::ai::behavior_tree::BehaviorTree;

let mut tree = BehaviorTree {
    root: Box::new(Selector {
        children: vec![
            // 战斗分支
            Box::new(Sequence {
                children: vec![
                    Box::new(IsAliveCondition { health: 100.0 }),
                    Box::new(AttackAction),
                ],
            }),
            // 巡逻分支
            Box::new(PatrolAction),
        ],
    }),
};

let status = tree.tick();
println!("行为树状态: {:?}", status);
```

---

## 调试工具

### NPC编辑器面板

游戏内置了NPC编辑器面板（egui调试界面）：

**功能**:
- ✅ 11个内置预设浏览和选择
- ✅ 8个性格参数滑块调节（0.0-1.0）
- ✅ 参数实时解释tooltip
- ✅ 高级设置折叠面板
- ✅ 自定义预设创建
- ✅ 预设导出和导入

**使用方法**:
```rust
// 在调试模式下启用NPC编辑器
#[cfg(feature = "debug")]
game_engine::debug::panels::register_npc_editor_panel();
```

### 成本追踪面板

实时监控LLM成本：

**功能**:
- ✅ 时间段选择（今日/本周/本月）
- ✅ 总成本、平均成本显示
- ✅ 按模型分组统计
- ✅ 成本报告导出

**快捷键**: 在调试界面点击 "LLM Cost Tracker" 面板

### 控制台命令

```rust
// 列出所有预设
game_engine::console::execute("list_npc_presets");

// 获取预设详情
game_engine::console::execute("get_preset friendly_merchant");

// 查看成本统计
game_engine::console::execute("cost_stats --period 24h");

// 清空缓存
game_engine::console::execute("clear_llm_cache");
```

---

## 最佳实践

### 1. 预设选择指南

**快速创建NPC**（5分钟内）:
- 商店/交易 → `friendly_merchant`
- 守卫/士兵 → `aggressive_guard` 或 `brave_knight`
- 村民/路人 → `curious_villager` 或 `humble_farmer`
- 任务发布 → `wise_elder` 或 `noble_mage`
- 特殊角色 → `mysterious_stranger`

**性能优化**:
- 简单NPC → 使用 `TraditionalOnly` 模式（无LLM成本）
- 重要NPC → 使用 `Hybrid` 模式（平衡成本和智能）
- 复杂对话 → 使用 `LLMOnly` 模式（最佳智能）

### 2. 成本控制策略

**预算建议**:
- 小游戏（<50NPC）: 每日$5，每月$50
- 中型游戏（50-200NPC）: 每日$20，每月$200
- 大型游戏（200+NPC）: 每日$50，每月$500

**降低成本**:
```rust
// 1. 启用缓存（自动启用）
CacheConfig {
    max_entries: 1000,  // 增加缓存容量
    ttl_seconds: 86400, // 延长TTL
    ..Default::default()
};

// 2. 使用更便宜的模型
merchant_preset.llm_model = Some("gpt-3.5-turbo".to_string());

// 3. 提高混合模式阈值
merchant_preset.complexity_threshold = 0.8; // 更多使用传统AI
```

### 3. 性能优化

**缓存命中率优化**:
- 使用一致的提示词（避免随机性）
- 启用持久化缓存
- 定期清理过期条目

**行为树优化**:
- 避免深层嵌套（<5层）
- 使用Selector优先处理常见情况
- 条件节点放在前面（快速失败）

### 4. 调试技巧

**LLM调用追踪**:
```rust
// 启用详细日志
RUST_LOG=game_engine::ai=debug

// 查看缓存命中率
let stats = cache.get_stats();
println!("命中率: {:.1}%", stats.hit_rate() * 100.0);

// 导出成本报告
tracker.export_to_csv("debug_costs.csv")?;
```

---

## 示例代码

### 完整示例：创建商人NPC

```rust
use game_engine::ai::npc::presets::PresetManager;
use game_engine::ai::npc::{IntelligentNPC, NPCConfig};
use bevy_ecs::entity::Entity;

fn create_merchant(entity_id: Entity) -> IntelligentNPC {
    let manager = PresetManager::new();
    let preset = manager.get_preset("friendly_merchant").unwrap();

    IntelligentNPC::new(entity_id)
        .with_config(preset.to_npc_config())
        .with_hybrid_mode(preset.hybrid_mode)
        .with_personality(preset.to_personality())
}

// 使用
let merchant_entity = commands.spawn().id();
let merchant_npc = create_merchant(merchant_entity);
```

### 完整示例：成本追踪

```rust
use game_engine::ai::cost_tracking::{CostTracker, BudgetConfig};

fn main() {
    let tracker = CostTracker::new(BudgetConfig {
        daily_budget_usd: 10.0,
        monthly_budget_usd: 100.0,
        ..Default::default()
    });

    // 模拟API调用
    for _ in 0..10 {
        tracker.record_call("gpt-3.5-turbo", 1000, 500, "npc_1").unwrap();
    }

    // 生成报告
    let stats = tracker.get_statistics(86400);
    println!("总成本: ${:.4}", stats.total_cost_usd);
    println!("预算使用: {:.1}%", tracker.get_budget_usage_percent());

    // 导出
    tracker.export_to_json("daily_report.json").unwrap();
}
```

### 完整示例：自定义预设

```rust
use game_engine::ai::npc::presets::{NPCPreset, NPCPresetBuilder, NPCPresetCategory};

fn main() {
    let assassin = NPCPreset::builder()
        .id("shadow_assassin")
        .name("Shadow Assassin")
        .description("A deadly assassin working from the shadows")
        .category(NPCPresetCategory::Neutral)
        .friendliness(0.1)   // 冷酷
        .aggression(0.9)     // 极度攻击性
        .curiosity(0.4)      // 中等好奇
        .fear(0.2)           // 无所畏惧
        .bravery(0.85)       // 勇敢
        .greed(0.95)         // 极度贪婪
        .formality(0.3)      // 非正式
        .humor(0.1)          // 严肃
        .dialogue_style("Cold, calculating, speaks briefly. Uses assassin terminology.")
        .add_sample_dialogue("Your name is on the list.")
        .add_sample_dialogue("The contract is fulfilled.")
        .add_sample_dialogue("Nothing personal, just business.")
        .add_tag("assassin")
        .add_tag("dangerous")
        .build()
        .unwrap();

    println!("Created: {}", assassin.name);
}
```

---

## 故障排除

### 问题1: LLM调用失败

**症状**: `LLM call failed: Rate limit exceeded`

**解决方案**:
```rust
// 1. 启用缓存（减少调用）
let cache = LLMCache::new(CacheConfig::default());

// 2. 降低调用频率
std::thread::sleep(std::time::Duration::from_millis(1000));

// 3. 使用更便宜的模型
preset.llm_model = Some("gpt-3.5-turbo".to_string());
```

### 问题2: 成本超支

**症状**: `Budget exceeded, API call blocked`

**解决方案**:
```rust
// 1. 增加预算
config.daily_budget_usd = 20.0;

// 2. 启用预算阻止
config.block_on_exceed = true;

// 3. 降低复杂度阈值
preset.complexity_threshold = 0.8; // 更多使用传统AI
```

### 问题3: 缓存命中率低

**症状**: 缓存命中率 < 20%

**解决方案**:
```rust
// 1. 增加缓存容量
config.max_entries = 2000;

// 2. 延长TTL
config.ttl_seconds = 172800; // 48小时

// 3. 使用一致的提示词
// 避免在提示词中包含时间戳、随机数等
```

---

## 参考资源

### 示例代码
- `examples/ai_examples.rs` - 行为树示例
- `examples/npc_presets/basic_usage.rs` - 基础预设使用
- `examples/npc_presets/custom_preset.rs` - 自定义预设创建
- `examples/npc_presets/cost_tracking.rs` - 成本追踪示例

### 源代码
- `src/ai/npc/presets.rs` - 预设系统实现
- `src/ai/llm_cache.rs` - LLM缓存实现
- `src/ai/cost_tracking.rs` - 成本追踪实现
- `src/debug/panels/npc_editor_panel.rs` - NPC编辑器UI

### 相关文档
- [API Reference](./api_reference.md)
- [Best Practices](./best_practices.md)
- [Domain Overview](./domain_overview.md)

---

## 更新日志

**v1.0.0** (2025-01-01)
- ✅ 11个内置NPC预设
- ✅ LLM缓存系统（LRU, 1000条，24小时TTL）
- ✅ 成本追踪和预算控制
- ✅ NPC编辑器UI面板
- ✅ 完整示例代码

---

**维护者**: Game Engine Team
**许可证**: MIT
**反馈**: GitHub Issues
