//! NPC预设系统使用示例
//!
//! 演示如何使用内置预设快速创建NPC

use game_engine::ai::npc::presets::{PresetManager, NPCPresetCategory};
use game_engine::ai::npc::{IntelligentNPC, NPCConfig};
use game_engine::ai::service::Personality;
use bevy_ecs::entity::Entity;

fn main() {
    // 创建预设管理器
    let preset_manager = PresetManager::new();

    // 获取友好商人预设
    if let Some(merchant_preset) = preset_manager.get_preset("friendly_merchant") {
        println!("Merchant Preset: {}", merchant_preset.name);
        println!("Description: {}", merchant_preset.description);

        // 创建NPC配置
        let npc_config = merchant_preset.to_npc_config();

        // 创建个性
        let personality = merchant_preset.to_personality();
        println!("\nPersonality Traits:");
        println!("  Friendliness: {}", personality.friendliness);
        println!("  Greed: {}", personality.greed);
        println!("  Humor: {}", personality.humor);

        // 获取系统提示词
        let system_prompt = merchant_preset.get_system_prompt();
        println!("\nSystem Prompt:\n{}", system_prompt);

        // 显示示例对话
        println!("\nSample Dialogues:");
        for dialogue in &merchant_preset.sample_dialogues {
            println!("  - {}", dialogue);
        }
    }

    // 按类别获取预设
    println!("\n=== Guard Presets ===");
    let guards = preset_manager.get_presets_by_category(NPCPresetCategory::Guard);
    for guard in guards {
        println!("  - {} ({})", guard.name, guard.description);
    }

    // 按标签搜索预设
    println!("\n=== Friendly Presets ===");
    let friendly = preset_manager.search_by_tag("friendly");
    for preset in friendly {
        println!("  - {} ({:?})", preset.name, preset.category);
    }

    // 统计信息
    println!("\nTotal Presets: {}", preset_manager.preset_count());
}

/// 创建NPC并应用预设
fn create_npc_with_preset(entity_id: Entity, preset_id: &str) -> Option<IntelligentNPC> {
    let preset_manager = PresetManager::new();
    let preset = preset_manager.get_preset(preset_id)?;

    let npc = IntelligentNPC::new(entity_id)
        .with_config(preset.to_npc_config())
        .with_hybrid_mode(preset.hybrid_mode);

    // 注意：个性需要通过其他方式设置，可能需要扩展IntelligentNPC

    Some(npc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_friendly_merchant() {
        let manager = PresetManager::new();
        let preset = manager.get_preset("friendly_merchant");
        assert!(preset.is_some());
        assert_eq!(preset.unwrap().name, "Friendly Merchant");
    }

    #[test]
    fn test_preset_to_personality() {
        let manager = PresetManager::new();
        let preset = manager.get_preset("aggressive_guard").unwrap();
        let personality = preset.to_personality();
        assert!(personality.friendliness < 0.5);
        assert!(personality.custom_traits.get("aggression").unwrap() > &0.5);
    }

    #[test]
    fn test_get_guards() {
        let manager = PresetManager::new();
        let guards = manager.get_presets_by_category(NPCPresetCategory::Guard);
        assert!(!guards.is_empty());
    }
}
