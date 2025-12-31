//! NPC编辑器面板
//!
//! 提供简化的NPC配置界面，支持预设选择和参数调整。

use crate::ai::npc::presets::{NPCPreset, NPCPresetBuilder, NPCPresetCategory, PresetManager};
use crate::ai::service::Personality;
use crate::debug::panels::Panel;
use bevy_ecs::prelude::*;
use egui::{ScrollArea, Ui};
use std::sync::{Arc, Mutex};

/// NPC编辑器面板状态
#[derive(Default)]
pub struct NPCEditorPanel {
    /// 选中的预设ID
    selected_preset_id: Option<String>,
    /// 当前编辑的预设
    current_preset: Option<NPCPreset>,
    /// 显示高级选项
    show_advanced: bool,
    /// 搜索过滤
    search_filter: String,
    /// 选中的类别
    selected_category: Option<NPCPresetCategory>,
    /// 自定义预设编辑器状态
    custom_preset_builder: Option<NPCPresetBuilder>,
    /// 临时存储的参数（用于滑块）
    temp_params: PersonalityParams,
}

/// 个性参数（用于UI编辑）
#[derive(Debug, Clone, Default)]
struct PersonalityParams {
    friendliness: f32,
    aggression: f32,
    curiosity: f32,
    fear: f32,
    bravery: f32,
    greed: f32,
    formality: f32,
    humor: f32,
}

impl From<PersonalityParams> for Personality {
    fn from(params: PersonalityParams) -> Self {
        let mut custom_traits = std::collections::HashMap::new();
        custom_traits.insert("aggression".to_string(), params.aggression);
        custom_traits.insert("curiosity".to_string(), params.curiosity);
        custom_traits.insert("fear".to_string(), params.fear);

        Self {
            friendliness: params.friendliness,
            formality: params.formality,
            humor: params.humor,
            bravery: params.bravery,
            greed: params.greed,
            custom_traits,
        }
    }
}

impl From<&NPCPreset> for PersonalityParams {
    fn from(preset: &NPCPreset) -> Self {
        Self {
            friendliness: preset.friendliness,
            aggression: preset.aggression,
            curiosity: preset.curiosity,
            fear: preset.fear,
            bravery: preset.bravery,
            greed: preset.greed,
            formality: preset.formality,
            humor: preset.humor,
        }
    }
}

impl Panel for NPCEditorPanel {
    fn show(&mut self, ctx: &egui::Context, _world: &World) {
        egui::Window::new("NPC Editor")
            .default_size([800.0, 600.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.show_ui(ui);
            });
    }
}

impl NPCEditorPanel {
    /// 显示UI
    fn show_ui(&mut self, ui: &mut Ui) {
        // 顶部：搜索和筛选
        self.show_search_and_filter(ui);
        ui.separator();

        // 左右分栏
        egui::Splitter::horizontal()
            .ratio(0.4)
            .show(ui, |ui| {
                // 左侧：预设列表
                self.show_preset_list(ui);
            })
            .show(ui, |ui| {
                // 右侧：编辑区域
                self.show_editor_area(ui);
            });
    }

    /// 显示搜索和筛选栏
    fn show_search_and_filter(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.search_filter);

            ui.label("Category:");
            let categories = [
                ("All", None),
                ("Friendly", Some(NPCPresetCategory::Friendly)),
                ("Hostile", Some(NPCPresetCategory::Hostile)),
                ("Merchant", Some(NPCPresetCategory::Merchant)),
                ("Guard", Some(NPCPresetCategory::Guard)),
                ("Quest Giver", Some(NPCPresetCategory::QuestGiver)),
            ];

            for (name, category) in categories {
                if ui.selectable_value(&mut self.selected_category, category, name).changed() {
                    // 类别变更
                }
            }
        });

        ui.separator();
    }

    /// 显示预设列表
    fn show_preset_list(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.heading("Presets");
            ui.separator();

            ScrollArea::vertical().show(ui, |ui| {
                let manager = PresetManager::new();
                let presets = manager.get_all_presets();

                for preset in presets {
                    // 应用筛选
                    if let Some(category) = self.selected_category {
                        if preset.category != category {
                            continue;
                        }
                    }

                    if !self.search_filter.is_empty()
                        && !preset.name.to_lowercase().contains(&self.search_filter.to_lowercase())
                        && !preset
                            .description
                            .to_lowercase()
                            .contains(&self.search_filter.to_lowercase())
                    {
                        continue;
                    }

                    // 显示预设项
                    let is_selected = self
                        .selected_preset_id
                        .as_ref()
                        .map(|id| id == &preset.id)
                        .unwrap_or(false);

                    if ui.selectable_label(is_selected, &preset.name).clicked() {
                        self.selected_preset_id = Some(preset.id.clone());
                        self.current_preset = Some(preset.clone());
                        self.temp_params = PersonalityParams::from(preset);
                    }

                    // 显示类别标签
                    ui.label(format!("{:?}", preset.category));
                    ui.separator();
                }

                // 添加"自定义预设"选项
                if ui
                    .selectable_label(
                        self.selected_preset_id.as_ref().map(|id| id == "custom").unwrap_or(false),
                        "➕ Custom Preset",
                    )
                    .clicked()
                {
                    self.selected_preset_id = Some("custom".to_string());
                    self.current_preset = None;
                    self.custom_preset_builder = Some(NPCPreset::builder());
                    self.temp_params = PersonalityParams::default();
                }
            });
        });
    }

    /// 显示编辑区域
    fn show_editor_area(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            if let Some(preset) = &self.current_preset {
                // 显示预设信息
                self.show_preset_info(ui, preset);
                ui.separator();
            } else if self.selected_preset_id.as_ref().map(|id| id == "custom").unwrap_or(false) {
                // 显示自定义预设编辑器
                self.show_custom_preset_editor(ui);
            } else {
                ui.label("Select a preset to edit");
            }

            // 底部：操作按钮
            ui.separator();
            self.show_action_buttons(ui);
        });
    }

    /// 显示预设信息
    fn show_preset_info(&mut self, ui: &mut Ui, preset: &NPCPreset) {
        ui.heading(&preset.name);
        ui.label(&preset.description);

        ui.separator();
        ui.heading("Personality Traits");
        self.show_personality_sliders(ui);

        ui.separator();
        ui.checkbox(&mut self.show_advanced, "Show Advanced Options");

        if self.show_advanced {
            ui.separator();
            ui.heading("Advanced Settings");
            self.show_advanced_settings(ui, preset);
        }

        ui.separator();
        ui.heading("Sample Dialogues");
        for dialogue in &preset.sample_dialogues {
            ui.label(format!("• {}", dialogue));
        }
    }

    /// 显示个性参数滑块
    fn show_personality_sliders(&mut self, ui: &mut Ui) {
        ui.add(
            egui::Slider::new(&mut self.temp_params.friendliness, 0.0..=1.0)
                .text("Friendliness")
                .show_value(true)
                .step_by(0.01),
        );
        ui.label(
            egui::RichText::new(get_trait_description(
                "friendliness",
                self.temp_params.friendliness,
            ))
            .small()
            .color(egui::Color32::GRAY),
        );

        ui.add(
            egui::Slider::new(&mut self.temp_params.aggression, 0.0..=1.0)
                .text("Aggression")
                .show_value(true)
                .step_by(0.01),
        );
        ui.label(
            egui::RichText::new(get_trait_description(
                "aggression",
                self.temp_params.aggression,
            ))
            .small()
            .color(egui::Color32::GRAY),
        );

        ui.add(
            egui::Slider::new(&mut self.temp_params.curiosity, 0.0..=1.0)
                .text("Curiosity")
                .show_value(true)
                .step_by(0.01),
        );
        ui.label(
            egui::RichText::new(get_trait_description(
                "curiosity",
                self.temp_params.curiosity,
            ))
            .small()
            .color(egui::Color32::GRAY),
        );

        ui.add(
            egui::Slider::new(&mut self.temp_params.fear, 0.0..=1.0)
                .text("Fear")
                .show_value(true)
                .step_by(0.01),
        );
        ui.label(
            egui::RichText::new(get_trait_description("fear", self.temp_params.fear))
                .small()
                .color(egui::Color32::GRAY),
        );

        ui.add(
            egui::Slider::new(&mut self.temp_params.bravery, 0.0..=1.0)
                .text("Bravery")
                .show_value(true)
                .step_by(0.01),
        );
        ui.label(
            egui::RichText::new(get_trait_description("bravery", self.temp_params.bravery))
                .small()
                .color(egui::Color32::GRAY),
        );

        ui.add(
            egui::Slider::new(&mut self.temp_params.greed, 0.0..=1.0)
                .text("Greed")
                .show_value(true)
                .step_by(0.01),
        );
        ui.label(
            egui::RichText::new(get_trait_description("greed", self.temp_params.greed))
                .small()
                .color(egui::Color32::GRAY),
        );

        ui.add(
            egui::Slider::new(&mut self.temp_params.formality, 0.0..=1.0)
                .text("Formality")
                .show_value(true)
                .step_by(0.01),
        );
        ui.label(
            egui::RichText::new(get_trait_description(
                "formality",
                self.temp_params.formality,
            ))
            .small()
            .color(egui::Color32::GRAY),
        );

        ui.add(
            egui::Slider::new(&mut self.temp_params.humor, 0.0..=1.0)
                .text("Humor")
                .show_value(true)
                .step_by(0.01),
        );
        ui.label(
            egui::RichText::new(get_trait_description("humor", self.temp_params.humor))
                .small()
                .color(egui::Color32::GRAY),
        );
    }

    /// 显示高级设置
    fn show_advanced_settings(&mut self, ui: &mut Ui, preset: &NPCPreset) {
        ui.horizontal(|ui| {
            ui.label("Hybrid Mode:");
            ui.label(format!("{:?}", preset.hybrid_mode));
        });

        ui.horizontal(|ui| {
            ui.label("Enable LLM:");
            ui.label(if preset.enable_llm { "Yes" } else { "No" });
        });

        ui.horizontal(|ui| {
            ui.label("Model:");
            ui.label(preset.llm_model.as_deref().unwrap_or("N/A"));
        });

        ui.horizontal(|ui| {
            ui.label("Complexity Threshold:");
            ui.label(format!("{:.2}", preset.complexity_threshold));
        });

        ui.horizontal(|ui| {
            ui.label("Dialogue Style:");
            ui.label(&preset.dialogue_style_prompt);
        });
    }

    /// 显示自定义预设编辑器
    fn show_custom_preset_editor(&mut self, ui: &mut Ui) {
        ui.heading("Create Custom Preset");

        ui.horizontal(|ui| {
            ui.label("Preset ID:");
            // 这里需要为custom_preset_builder添加ID字段
            ui.label("(Custom)");
        });

        ui.separator();
        self.show_personality_sliders(ui);
    }

    /// 显示操作按钮
    fn show_action_buttons(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Apply to NPC").clicked() {
                // 应用当前预设到选中的NPC
                log::info!("Applied preset to NPC");
            }

            if ui.button("Save as New Preset").clicked() {
                // 保存为新预设
                log::info!("Saved as new preset");
            }

            if ui.button("Export Configuration").clicked() {
                // 导出配置
                log::info!("Exported configuration");
            }

            if ui.button("Reset").clicked() {
                // 重置
                if let Some(preset) = &self.current_preset {
                    self.temp_params = PersonalityParams::from(preset);
                }
            }
        });
    }
}

/// 获取个性参数的描述
fn get_trait_description(trait_name: &str, value: f32) -> String {
    match trait_name {
        "friendliness" => match value {
            x if x < 0.2 => "Very cold and distant",
            x if x < 0.4 => "Somewhat reserved",
            x if x < 0.6 => "Moderately friendly",
            x if x < 0.8 => "Very warm and welcoming",
            _ => "Extremely friendly and outgoing",
        }
        .to_string(),
        "aggression" => match value {
            x if x < 0.2 => "Very passive",
            x if x < 0.4 => "Non-aggressive",
            x if x < 0.6 => "Moderately assertive",
            x if x < 0.8 => "Quite aggressive",
            _ => "Extremely aggressive",
        }
        .to_string(),
        "curiosity" => match value {
            x if x < 0.2 => "Not curious at all",
            x if x < 0.4 => "Mildly curious",
            x if x < 0.6 => "Moderately curious",
            x if x < 0.8 => "Very inquisitive",
            _ => "Extremely curious about everything",
        }
        .to_string(),
        "fear" => match value {
            x if x < 0.2 => "Fearless",
            x if x < 0.4 => "Brave",
            x if x < 0.6 => "Normal fear response",
            x if x < 0.8 => "Quite cautious",
            _ => "Very fearful",
        }
        .to_string(),
        "bravery" => match value {
            x if x < 0.2 => "Cowardly",
            x if x < 0.4 => "Somewhat timid",
            x if x < 0.6 => "Average courage",
            x if x < 0.8 => "Very brave",
            _ => "Extremely heroic",
        }
        .to_string(),
        "greed" => match value {
            x if x < 0.2 => "Very generous",
            x if x < 0.4 => "Somewhat generous",
            x if x < 0.6 => "Moderate self-interest",
            x if x < 0.8 => "Quite greedy",
            _ => "Extremely greedy",
        }
        .to_string(),
        "formality" => match value {
            x if x < 0.2 => "Very casual",
            x if x < 0.4 => "Informal",
            x if x < 0.6 => "Neutral formality",
            x if x < 0.8 => "Formal",
            _ => "Very formal",
        }
        .to_string(),
        "humor" => match value {
            x if x < 0.2 => "Very serious",
            x if x < 0.4 => "Rarely humorous",
            x if x < 0.6 => "Occasionally funny",
            x if x < 0.8 => "Quite humorous",
            _ => "Very funny and playful",
        }
        .to_string(),
        _ => String::new(),
    }
}

/// 成本追踪面板
#[derive(Default)]
pub struct CostTrackingPanel {
    show_details: bool,
    selected_period: usize,
}

impl Panel for CostTrackingPanel {
    fn show(&mut self, ctx: &egui::Context, _world: &World) {
        egui::Window::new("LLM Cost Tracker")
            .default_size([600.0, 400.0])
            .resizable(true)
            .show(ctx, |ui| {
                self.show_ui(ui);
            });
    }
}

impl CostTrackingPanel {
    fn show_ui(&mut self, ui: &mut Ui) {
        ui.heading("Cost Tracking");

        ui.horizontal(|ui| {
            ui.label("Time Period:");
            let periods = ["Today", "This Week", "This Month"];
            for (i, period) in periods.iter().enumerate() {
                if ui.selectable_value(&mut self.selected_period, i, *period).changed() {
                    // 切换时间段
                }
            }
        });

        ui.separator();

        // 显示统计数据（这里需要实际的CostTracker实例）
        ui.label("Total API Calls: -");
        ui.label("Total Tokens Used: -");
        ui.label("Total Cost: $-");
        ui.label("Average Cost per Call: $-");

        ui.separator();
        ui.checkbox(&mut self.show_details, "Show Details");

        if self.show_details {
            ui.separator();
            ui.heading("Cost by Model");
            // 显示按模型分组的统计
        }

        ui.separator();

        if ui.button("Export Report").clicked() {
            log::info!("Exporting cost report...");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_personality_params_conversion() {
        let params = PersonalityParams {
            friendliness: 0.8,
            aggression: 0.2,
            ..Default::default()
        };

        let personality: Personality = params.into();
        assert_eq!(personality.friendliness, 0.8);
    }

    #[test]
    fn test_trait_descriptions() {
        let desc = get_trait_description("friendliness", 0.9);
        assert!(desc.contains("friendly"));
    }
}
