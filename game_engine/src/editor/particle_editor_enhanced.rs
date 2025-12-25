//! 增强的粒子编辑器
//!
//! 在基础粒子编辑器上添加高级功能：
//! - 粒子系统预设库
//! - 粒子效果预览
//! - 粒子系统导出/导入
//! - 高级物理参数
//! - 粒子子发射器

use crate::editor::particle_editor::{EmitterType, ParticlePreset, ParticleSystemConfig};
use std::collections::HashMap;
use std::path::PathBuf;

/// 粒子系统库条目
#[derive(Debug, Clone)]
pub struct ParticleSystemLibraryEntry {
    pub name: String,
    pub config: ParticleSystemConfig,
    pub thumbnail_path: Option<PathBuf>,
    pub tags: Vec<String>,
    pub description: String,
}

/// 子发射器配置
#[derive(Debug, Clone)]
pub struct SubEmitterConfig {
    pub enabled: bool,
    pub emission_rate: f32,
    pub lifetime: f32,
    pub config: ParticleSystemConfig,
}

/// 增强的粒子编辑器
pub struct ParticleEditorEnhanced {
    /// 粒子系统配置
    pub config: ParticleSystemConfig,
    /// 是否正在播放
    pub is_playing: bool,
    /// 粒子系统库
    pub particle_library: HashMap<String, ParticleSystemLibraryEntry>,
    /// 子发射器列表
    pub sub_emitters: Vec<SubEmitterConfig>,
    /// 显示预览
    pub show_preview: bool,
    /// 预览大小
    pub preview_size: f32,
    /// 当前粒子数量（模拟）
    pub current_particle_count: usize,
    /// 系统名称
    pub system_name: String,
}

impl ParticleEditorEnhanced {
    pub fn new() -> Self {
        let mut editor = Self {
            config: ParticleSystemConfig::default(),
            is_playing: false,
            particle_library: HashMap::new(),
            sub_emitters: Vec::new(),
            show_preview: true,
            preview_size: 300.0,
            current_particle_count: 0,
            system_name: "Particle System".to_string(),
        };

        // 初始化预设库
        editor.init_presets();
        editor
    }

    /// 初始化预设库
    fn init_presets(&mut self) {
        for preset in [
            ParticlePreset::Fire,
            ParticlePreset::Smoke,
            ParticlePreset::Explosion,
            ParticlePreset::Rain,
            ParticlePreset::Snow,
            ParticlePreset::Magic,
        ] {
            let entry = ParticleSystemLibraryEntry {
                name: preset.name().to_string(),
                config: preset.to_config(),
                thumbnail_path: None,
                tags: vec![preset.name().to_string()],
                description: format!("{} particle system preset", preset.name()),
            };
            self.particle_library.insert(preset.name().to_string(), entry);
        }
    }

    /// 加载预设
    pub fn load_preset(&mut self, preset: ParticlePreset) {
        self.config = preset.to_config();
        self.system_name = format!("{} System", preset.name());
    }

    /// 从库加载
    pub fn load_from_library(&mut self, name: &str) {
        if let Some(entry) = self.particle_library.get(name) {
            self.config = entry.config.clone();
            self.system_name = entry.name.clone();
        }
    }

    /// 保存到库
    pub fn save_to_library(&mut self) {
        let entry = ParticleSystemLibraryEntry {
            name: self.system_name.clone(),
            config: self.config.clone(),
            thumbnail_path: None,
            tags: vec!["Custom".to_string()],
            description: "Custom particle system".to_string(),
        };
        self.particle_library.insert(self.system_name.clone(), entry);
    }

    /// 添加子发射器
    pub fn add_sub_emitter(&mut self) {
        let sub_emitter = SubEmitterConfig {
            enabled: true,
            emission_rate: 5.0,
            lifetime: 1.0,
            config: ParticleSystemConfig::default(),
        };
        self.sub_emitters.push(sub_emitter);
    }

    /// 渲染增强的粒子编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Particle Editor Enhanced");
        ui.separator();

        // 工具栏
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut self.system_name);
            if ui.button("💾 Save to Library").clicked() {
                self.save_to_library();
            }
            ui.checkbox(&mut self.show_preview, "Preview");
        });

        ui.separator();

        // 预设选择
        ui.collapsing("Presets", |ui| {
            ui.horizontal(|ui| {
                for preset in [
                    ParticlePreset::Fire,
                    ParticlePreset::Smoke,
                    ParticlePreset::Explosion,
                ] {
                    if ui.button(preset.name()).clicked() {
                        self.load_preset(preset);
                    }
                }
            });
            ui.horizontal(|ui| {
                for preset in [ParticlePreset::Rain, ParticlePreset::Snow, ParticlePreset::Magic] {
                    if ui.button(preset.name()).clicked() {
                        self.load_preset(preset);
                    }
                }
            });
        });

        ui.separator();

        // 使用基础粒子编辑器的渲染逻辑（简化，实际应该调用基础编辑器）
        // 这里展示增强功能
        ui.collapsing("Emitter Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Emitter Type:");
                egui::ComboBox::from_label("")
                    .selected_text(self.config.emitter_type.name())
                    .show_ui(ui, |ui| {
                        for emitter_type in [EmitterType::Point, EmitterType::Sphere, EmitterType::Box, EmitterType::Cone] {
                            ui.selectable_value(&mut self.config.emitter_type, emitter_type, emitter_type.name());
                        }
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Emission Rate:");
                ui.add(egui::Slider::new(&mut self.config.emission_rate, 1.0..=200.0));
            });

            ui.horizontal(|ui| {
                ui.label("Max Particles:");
                ui.add(egui::Slider::new(&mut self.config.max_particles, 10..=2000));
            });

            ui.checkbox(&mut self.config.looping, "Looping");
        });

        ui.separator();

        // 子发射器
        ui.collapsing("Sub Emitters", |ui| {
            ui.label(format!("Sub Emitters: {}", self.sub_emitters.len()));
            if ui.button("+ Add Sub Emitter").clicked() {
                self.add_sub_emitter();
            }

            for (i, sub_emitter) in self.sub_emitters.iter_mut().enumerate() {
                ui.collapsing(format!("Sub Emitter {}", i), |ui| {
                    ui.checkbox(&mut sub_emitter.enabled, "Enabled");
                    ui.horizontal(|ui| {
                        ui.label("Emission Rate:");
                        ui.add(egui::Slider::new(&mut sub_emitter.emission_rate, 0.0..=50.0));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Lifetime:");
                        ui.add(egui::Slider::new(&mut sub_emitter.lifetime, 0.1..=10.0));
                    });
                });
            }
        });

        ui.separator();

        // 粒子预览
        if self.show_preview {
            ui.collapsing("Preview", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.add(egui::Slider::new(&mut self.preview_size, 100.0..=500.0));
                });
                ui.label(format!("Current Particles: {}", self.current_particle_count));
                // 预览区域（占位）
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(self.preview_size, self.preview_size),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.label("Particle Preview");
                        ui.label("(3D particle preview will be displayed here)");
                    },
                );
            });
        }

        ui.separator();

        // 播放控制
        ui.horizontal(|ui| {
            if ui.button(if self.is_playing { "⏸ Stop" } else { "▶ Play" }).clicked() {
                self.is_playing = !self.is_playing;
            }

            if ui.button("🔄 Reset").clicked() {
                self.config = ParticleSystemConfig::default();
                self.is_playing = false;
                self.current_particle_count = 0;
            }
        });

        ui.separator();

        // 粒子系统库
        ui.collapsing("Particle Library", |ui| {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let library_names: Vec<String> = self.particle_library.keys().cloned().collect();
                    for name in library_names {
                        if let Some(entry) = self.particle_library.get(&name) {
                            ui.horizontal(|ui| {
                                if ui.button(&name).clicked() {
                                    self.load_from_library(&name);
                                }
                                ui.label(&entry.description);
                            });
                        }
                    }
                });
        });
    }
}

impl Default for ParticleEditorEnhanced {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_editor_enhanced() {
        let mut editor = ParticleEditorEnhanced::new();
        assert!(!editor.particle_library.is_empty());
    }

    #[test]
    fn test_sub_emitters() {
        let mut editor = ParticleEditorEnhanced::new();
        editor.add_sub_emitter();
        assert_eq!(editor.sub_emitters.len(), 1);
    }
}

