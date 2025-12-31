//! 创建自定义NPC预设示例
//!
//! 演示如何使用builder创建自定义NPC预设

use game_engine::ai::npc::presets::{NPCPreset, NPCPresetBuilder, NPCPresetCategory, PresetManager};
use game_engine::ai::npc::HybridMode;

fn main() {
    // 方法1: 使用Builder创建自定义预设
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
        .dialogue_style("Jovial, hospitable, loves to share local gossip and legends. Uses casual language and laughs often.")
        .add_sample_dialogue("Welcome, traveler! Come in, come in! What can I get for you?")
        .add_sample_dialogue("Ah, you've come to the right place! Best ale in three kingdoms!")
        .add_sample_dialogue("Did I ever tell you about the time I saw a dragon? Sit down, pour a drink...")
        .add_tag("innkeeper")
        .add_tag("friendly")
        .add_tag("storyteller")
        .build()
        .unwrap();

    println!("Created custom preset: {}", custom_preset.name);
    println!("Friendliness: {}", custom_preset.friendliness);
    println!("Humor: {}", custom_preset.humor);

    // 方法2: 通过管理器添加预设
    let mut manager = PresetManager::new();
    let result = manager.add_preset(custom_preset);
    if result.is_ok() {
        println!("Custom preset added to manager");
    }

    // 获取刚添加的预设
    if let Some(preset) = manager.get_preset("tavern_keeper") {
        println!("Retrieved preset: {}", preset.name);
    }

    // 方法3: 创建战斗导向的NPC预设
    let warrior_preset = NPCPreset::builder()
        .id("veteran_warrior")
        .name("Veteran Warrior")
        .description("A battle-hardened warrior with years of combat experience")
        .category(NPCPresetCategory::Guard)
        .friendliness(0.4)
        .aggression(0.7)
        .curiosity(0.3)
        .fear(0.1)
        .bravery(0.95)
        .greed(0.3)
        .formality(0.4)
        .humor(0.2)
        .hybrid_mode(HybridMode::TraditionalOnly)
        .enable_llm(false)
        .dialogue_style("Gritty, experienced, speaks of battles and tactics. Uses military terminology and brief sentences.")
        .add_sample_dialogue("I've fought in more battles than you've had hot meals.")
        .add_sample_dialogue("Watch your flank. Always watch your flank.")
        .add_sample_dialogue("There's no glory in war, only survival.")
        .add_tag("warrior")
        .add_tag("veteran")
        .add_tag("combat")
        .build()
        .unwrap();

    println!("\nCreated warrior preset: {}", warrior_preset.name);
    println!("Bravery: {}", warrior_preset.bravery);
    println!("Aggression: {}", warrior_preset.aggression);

    // 方法4: 创建魔法使用者预设
    let mage_preset = NPCPreset::builder()
        .id("arcane_scholar")
        .name("Arcane Scholar")
        .description("A learned mage dedicated to magical research")
        .category(NPCPresetCategory::QuestGiver)
        .friendliness(0.5)
        .aggression(0.2)
        .curiosity(0.95)
        .fear(0.3)
        .bravery(0.6)
        .greed(0.4)
        .formality(0.9)
        .humor(0.3)
        .hybrid_mode(HybridMode::LLMOnly)
        .enable_llm(true)
        .llm_model("gpt-4".to_string())
        .dialogue_style("Scholarly, speaks of arcane matters with precision. Uses formal language and magical terminology.")
        .add_sample_dialogue("The mana flows are particularly strong today.")
        .add_sample_dialogue("You show potential in the arcane arts. Have you considered formal training?")
        .add_sample_dialogue("I sense a disturbance in the magical ether...")
        .add_tag("mage")
        .add_tag("scholar")
        .add_tag("magic")
        .build()
        .unwrap();

    println!("\nCreated mage preset: {}", mage_preset.name);
    println!("Curiosity: {}", mage_preset.curiosity);
    println!("Formality: {}", mage_preset.formality);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_preset_builder() {
        let preset = NPCPreset::builder()
            .id("test_npc")
            .name("Test NPC")
            .description("A test NPC")
            .friendliness(0.7)
            .build()
            .unwrap();

        assert_eq!(preset.id, "test_npc");
        assert_eq!(preset.friendliness, 0.7);
    }

    #[test]
    fn test_preset_validation() {
        // 缺少名称应该失败
        let result = NPCPreset::builder()
            .id("test")
            .friendliness(0.5)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_add_custom_preset() {
        let mut manager = PresetManager::new();
        let custom = NPCPreset::builder()
            .id("custom_test")
            .name("Custom Test")
            .description("A custom test preset")
            .build()
            .unwrap();

        let result = manager.add_preset(custom);
        assert!(result.is_ok());
        assert!(manager.get_preset("custom_test").is_some());
    }

    #[test]
    fn test_remove_preset() {
        let mut manager = PresetManager::new();
        let custom = NPCPreset::builder()
            .id("to_remove")
            .name("To Remove")
            .description("Will be removed")
            .build()
            .unwrap();

        manager.add_preset(custom).unwrap();
        assert!(manager.get_preset("to_remove").is_some());

        manager.remove_preset("to_remove");
        assert!(manager.get_preset("to_remove").is_none());
    }
}
