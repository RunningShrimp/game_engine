use crate::impl_default;
use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// 粒子发射器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmitterType {
    Point,
    Sphere,
    Box,
    Cone,
}

impl EmitterType {
    pub fn name(&self) -> &'static str {
        match self {
            EmitterType::Point => "Point",
            EmitterType::Sphere => "Sphere",
            EmitterType::Box => "Box",
            EmitterType::Cone => "Cone",
        }
    }
}

/// 粒子系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystemConfig {
    /// 发射器类型
    pub emitter_type: EmitterType,
    /// 每秒发射的粒子数
    pub emission_rate: f32,
    /// 粒子生命周期 (秒)
    pub lifetime: f32,
    /// 生命周期变化范围
    pub lifetime_variance: f32,

    /// 初始速度
    pub initial_velocity: Vec3,
    /// 速度变化范围
    pub velocity_variance: Vec3,

    /// 初始大小
    pub initial_size: f32,
    /// 大小变化范围
    pub size_variance: f32,
    /// 结束大小
    pub end_size: f32,

    /// 初始颜色
    pub initial_color: Vec4,
    /// 结束颜色
    pub end_color: Vec4,

    /// 重力影响
    pub gravity: Vec3,
    /// 阻力
    pub drag: f32,

    /// 最大粒子数
    pub max_particles: usize,

    /// 是否循环
    pub looping: bool,
}

impl_default!(ParticleSystemConfig {
    emitter_type: EmitterType::Point,
    emission_rate: 10.0,
    lifetime: 2.0,
    lifetime_variance: 0.5,
    initial_velocity: Vec3::new(0.0, 1.0, 0.0),
    velocity_variance: Vec3::new(0.5, 0.5, 0.5),
    initial_size: 1.0,
    size_variance: 0.2,
    end_size: 0.0,
    initial_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
    end_color: Vec4::new(1.0, 1.0, 1.0, 0.0),
    gravity: Vec3::new(0.0, -9.81, 0.0),
    drag: 0.1,
    max_particles: 1000,
    looping: true,
});

/// 粒子系统预设
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticlePreset {
    Fire,
    Smoke,
    Explosion,
    Rain,
    Snow,
    Magic,
}

impl ParticlePreset {
    pub fn name(&self) -> &'static str {
        match self {
            ParticlePreset::Fire => "Fire",
            ParticlePreset::Smoke => "Smoke",
            ParticlePreset::Explosion => "Explosion",
            ParticlePreset::Rain => "Rain",
            ParticlePreset::Snow => "Snow",
            ParticlePreset::Magic => "Magic",
        }
    }

    pub fn to_config(&self) -> ParticleSystemConfig {
        match self {
            ParticlePreset::Fire => ParticleSystemConfig {
                emitter_type: EmitterType::Point,
                emission_rate: 50.0,
                lifetime: 1.0,
                lifetime_variance: 0.3,
                initial_velocity: Vec3::new(0.0, 2.0, 0.0),
                velocity_variance: Vec3::new(0.5, 0.5, 0.5),
                initial_size: 0.5,
                size_variance: 0.2,
                end_size: 0.0,
                initial_color: Vec4::new(1.0, 0.5, 0.0, 1.0),
                end_color: Vec4::new(1.0, 0.0, 0.0, 0.0),
                gravity: Vec3::new(0.0, 1.0, 0.0),
                drag: 0.5,
                max_particles: 500,
                looping: true,
            },
            ParticlePreset::Smoke => ParticleSystemConfig {
                emitter_type: EmitterType::Point,
                emission_rate: 20.0,
                lifetime: 3.0,
                lifetime_variance: 0.5,
                initial_velocity: Vec3::new(0.0, 1.0, 0.0),
                velocity_variance: Vec3::new(0.3, 0.3, 0.3),
                initial_size: 0.5,
                size_variance: 0.2,
                end_size: 2.0,
                initial_color: Vec4::new(0.5, 0.5, 0.5, 0.8),
                end_color: Vec4::new(0.3, 0.3, 0.3, 0.0),
                gravity: Vec3::new(0.0, 0.5, 0.0),
                drag: 0.3,
                max_particles: 300,
                looping: true,
            },
            ParticlePreset::Explosion => ParticleSystemConfig {
                emitter_type: EmitterType::Sphere,
                emission_rate: 200.0,
                lifetime: 0.5,
                lifetime_variance: 0.2,
                initial_velocity: Vec3::new(0.0, 0.0, 0.0),
                velocity_variance: Vec3::new(5.0, 5.0, 5.0),
                initial_size: 1.0,
                size_variance: 0.5,
                end_size: 0.0,
                initial_color: Vec4::new(1.0, 0.8, 0.0, 1.0),
                end_color: Vec4::new(0.5, 0.0, 0.0, 0.0),
                gravity: Vec3::new(0.0, -5.0, 0.0),
                drag: 0.8,
                max_particles: 1000,
                looping: false,
            },
            ParticlePreset::Rain => ParticleSystemConfig {
                emitter_type: EmitterType::Box,
                emission_rate: 100.0,
                lifetime: 2.0,
                lifetime_variance: 0.3,
                initial_velocity: Vec3::new(0.0, -10.0, 0.0),
                velocity_variance: Vec3::new(0.5, 1.0, 0.5),
                initial_size: 0.1,
                size_variance: 0.05,
                end_size: 0.1,
                initial_color: Vec4::new(0.5, 0.5, 1.0, 0.8),
                end_color: Vec4::new(0.5, 0.5, 1.0, 0.5),
                gravity: Vec3::new(0.0, -9.81, 0.0),
                drag: 0.0,
                max_particles: 1000,
                looping: true,
            },
            ParticlePreset::Snow => ParticleSystemConfig {
                emitter_type: EmitterType::Box,
                emission_rate: 50.0,
                lifetime: 5.0,
                lifetime_variance: 1.0,
                initial_velocity: Vec3::new(0.0, -1.0, 0.0),
                velocity_variance: Vec3::new(0.5, 0.3, 0.5),
                initial_size: 0.2,
                size_variance: 0.1,
                end_size: 0.2,
                initial_color: Vec4::new(1.0, 1.0, 1.0, 1.0),
                end_color: Vec4::new(1.0, 1.0, 1.0, 0.8),
                gravity: Vec3::new(0.0, -1.0, 0.0),
                drag: 0.5,
                max_particles: 500,
                looping: true,
            },
            ParticlePreset::Magic => ParticleSystemConfig {
                emitter_type: EmitterType::Sphere,
                emission_rate: 30.0,
                lifetime: 1.5,
                lifetime_variance: 0.5,
                initial_velocity: Vec3::new(0.0, 0.0, 0.0),
                velocity_variance: Vec3::new(2.0, 2.0, 2.0),
                initial_size: 0.3,
                size_variance: 0.1,
                end_size: 0.0,
                initial_color: Vec4::new(0.5, 0.0, 1.0, 1.0),
                end_color: Vec4::new(0.0, 1.0, 1.0, 0.0),
                gravity: Vec3::new(0.0, 0.0, 0.0),
                drag: 0.2,
                max_particles: 300,
                looping: true,
            },
        }
    }
}

/// 粒子系统库条目（增强功能）
#[derive(Debug, Clone)]
pub struct ParticleSystemLibraryEntry {
    pub name: String,
    pub config: ParticleSystemConfig,
    pub thumbnail_path: Option<PathBuf>,
    pub tags: Vec<String>,
    pub description: String,
}

/// 子发射器配置（增强功能）
#[derive(Debug, Clone)]
pub struct SubEmitterConfig {
    pub enabled: bool,
    pub emission_rate: f32,
    pub lifetime: f32,
    pub config: ParticleSystemConfig,
}

/// 粒子编辑器增强配置
#[derive(Debug, Clone)]
pub struct ParticleEditorEnhancedConfig {
    /// 显示预览
    pub show_preview: bool,
    /// 预览大小
    pub preview_size: f32,
    /// 启用粒子系统库
    pub enable_library: bool,
    /// 启用子发射器
    pub enable_sub_emitters: bool,
}

impl Default for ParticleEditorEnhancedConfig {
    fn default() -> Self {
        Self {
            show_preview: true,
            preview_size: 300.0,
            enable_library: true,
            enable_sub_emitters: true,
        }
    }
}

/// 粒子编辑器
pub struct ParticleEditor {
    /// 粒子系统配置
    pub config: ParticleSystemConfig,
    /// 是否正在播放
    pub is_playing: bool,
    /// 增强配置
    pub enhanced_config: ParticleEditorEnhancedConfig,
    /// 粒子系统库（增强功能）
    pub particle_library: HashMap<String, ParticleSystemLibraryEntry>,
    /// 子发射器列表（增强功能）
    pub sub_emitters: Vec<SubEmitterConfig>,
    /// 当前粒子数量（模拟，增强功能）
    pub current_particle_count: usize,
    /// 系统名称（增强功能）
    pub system_name: String,
}

impl ParticleEditor {
    pub fn new() -> Self {
        let mut editor = Self::default();
        // 初始化预设库
        if editor.enhanced_config.enable_library {
            editor.init_presets();
        }
        editor
    }

    /// 初始化预设库（增强功能）
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
        if self.enhanced_config.enable_library {
            self.system_name = format!("{} System", preset.name());
        }
    }

    /// 从库加载（增强功能）
    pub fn load_from_library(&mut self, name: &str) {
        if let Some(entry) = self.particle_library.get(name) {
            self.config = entry.config.clone();
            self.system_name = entry.name.clone();
        }
    }

    /// 保存到库（增强功能）
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

    /// 添加子发射器（增强功能）
    pub fn add_sub_emitter(&mut self) {
        if !self.enhanced_config.enable_sub_emitters {
            return;
        }
        let sub_emitter = SubEmitterConfig {
            enabled: true,
            emission_rate: 5.0,
            lifetime: 1.0,
            config: ParticleSystemConfig::default(),
        };
        self.sub_emitters.push(sub_emitter);
    }

    /// 渲染粒子编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Particle Editor");
        ui.separator();

        // 工具栏（增强功能）
        if self.enhanced_config.enable_library {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.system_name);
                if ui.button("💾 Save to Library").clicked() {
                    self.save_to_library();
                }
                ui.checkbox(&mut self.enhanced_config.show_preview, "Preview");
            });
            ui.separator();
        }

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
                for preset in [
                    ParticlePreset::Rain,
                    ParticlePreset::Snow,
                    ParticlePreset::Magic,
                ] {
                    if ui.button(preset.name()).clicked() {
                        self.load_preset(preset);
                    }
                }
            });
        });

        ui.separator();

        // 发射器设置
        ui.collapsing("Emitter Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Emitter Type:");
                egui::ComboBox::from_label("")
                    .selected_text(self.config.emitter_type.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.config.emitter_type,
                            EmitterType::Point,
                            "Point",
                        );
                        ui.selectable_value(
                            &mut self.config.emitter_type,
                            EmitterType::Sphere,
                            "Sphere",
                        );
                        ui.selectable_value(&mut self.config.emitter_type, EmitterType::Box, "Box");
                        ui.selectable_value(
                            &mut self.config.emitter_type,
                            EmitterType::Cone,
                            "Cone",
                        );
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Emission Rate:");
                ui.add(egui::Slider::new(
                    &mut self.config.emission_rate,
                    1.0..=200.0,
                ));
            });

            ui.horizontal(|ui| {
                ui.label("Max Particles:");
                ui.add(egui::Slider::new(&mut self.config.max_particles, 10..=2000));
            });

            ui.checkbox(&mut self.config.looping, "Looping");
        });

        ui.separator();

        // 粒子生命周期
        ui.collapsing("Lifetime Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Lifetime:");
                ui.add(egui::Slider::new(&mut self.config.lifetime, 0.1..=10.0));
            });

            ui.horizontal(|ui| {
                ui.label("Variance:");
                ui.add(egui::Slider::new(
                    &mut self.config.lifetime_variance,
                    0.0..=2.0,
                ));
            });
        });

        ui.separator();

        // 速度设置
        ui.collapsing("Velocity Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Initial Velocity:");
                ui.add(
                    egui::DragValue::new(&mut self.config.initial_velocity.x)
                        .prefix("X: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::new(&mut self.config.initial_velocity.y)
                        .prefix("Y: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::new(&mut self.config.initial_velocity.z)
                        .prefix("Z: ")
                        .speed(0.1),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Variance:");
                ui.add(
                    egui::DragValue::new(&mut self.config.velocity_variance.x)
                        .prefix("X: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::new(&mut self.config.velocity_variance.y)
                        .prefix("Y: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::new(&mut self.config.velocity_variance.z)
                        .prefix("Z: ")
                        .speed(0.1),
                );
            });
        });

        ui.separator();

        // 大小设置
        ui.collapsing("Size Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Initial Size:");
                ui.add(egui::Slider::new(&mut self.config.initial_size, 0.1..=5.0));
            });

            ui.horizontal(|ui| {
                ui.label("End Size:");
                ui.add(egui::Slider::new(&mut self.config.end_size, 0.0..=5.0));
            });

            ui.horizontal(|ui| {
                ui.label("Size Variance:");
                ui.add(egui::Slider::new(&mut self.config.size_variance, 0.0..=1.0));
            });
        });

        ui.separator();

        // 颜色设置
        ui.collapsing("Color Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Initial Color:");
                let mut color = [
                    self.config.initial_color.x,
                    self.config.initial_color.y,
                    self.config.initial_color.z,
                    self.config.initial_color.w,
                ];
                if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
                    self.config.initial_color = Vec4::from_array(color);
                }
            });

            ui.horizontal(|ui| {
                ui.label("End Color:");
                let mut color = [
                    self.config.end_color.x,
                    self.config.end_color.y,
                    self.config.end_color.z,
                    self.config.end_color.w,
                ];
                if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
                    self.config.end_color = Vec4::from_array(color);
                }
            });
        });

        ui.separator();

        // 物理设置
        ui.collapsing("Physics Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Gravity:");
                ui.add(egui::DragValue::new(&mut self.config.gravity.x).prefix("X: ").speed(0.1));
                ui.add(egui::DragValue::new(&mut self.config.gravity.y).prefix("Y: ").speed(0.1));
                ui.add(egui::DragValue::new(&mut self.config.gravity.z).prefix("Z: ").speed(0.1));
            });

            ui.horizontal(|ui| {
                ui.label("Drag:");
                ui.add(egui::Slider::new(&mut self.config.drag, 0.0..=1.0));
            });
        });

        ui.separator();

        // 子发射器（增强功能）
        if self.enhanced_config.enable_sub_emitters {
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
                            ui.add(egui::Slider::new(
                                &mut sub_emitter.emission_rate,
                                0.0..=50.0,
                            ));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Lifetime:");
                            ui.add(egui::Slider::new(&mut sub_emitter.lifetime, 0.1..=10.0));
                        });
                    });
                }
            });
            ui.separator();
        }

        // 粒子预览（增强功能）
        if self.enhanced_config.show_preview {
            ui.collapsing("Preview", |ui| {
                ui.horizontal(|ui| {
                    ui.label("Size:");
                    ui.add(egui::Slider::new(
                        &mut self.enhanced_config.preview_size,
                        100.0..=500.0,
                    ));
                });
                ui.label(format!(
                    "Current Particles: {}",
                    self.current_particle_count
                ));
                // 预览区域（占位）
                ui.allocate_ui_with_layout(
                    egui::Vec2::new(
                        self.enhanced_config.preview_size,
                        self.enhanced_config.preview_size,
                    ),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        ui.label("Particle Preview");
                        ui.label("(3D particle preview will be displayed here)");
                    },
                );
            });
            ui.separator();
        }

        // 播放控制
        ui.horizontal(|ui| {
            if ui
                .button(if self.is_playing {
                    "⏸ Stop"
                } else {
                    "▶ Play"
                })
                .clicked()
            {
                self.is_playing = !self.is_playing;
            }

            if ui.button("🔄 Reset").clicked() {
                self.config = ParticleSystemConfig::default();
                self.is_playing = false;
                self.current_particle_count = 0;
            }
        });

        // 粒子系统库（增强功能）
        if self.enhanced_config.enable_library {
            ui.separator();
            ui.collapsing("Particle Library", |ui| {
                egui::ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
                    let library_entries: Vec<(String, String)> = self
                        .particle_library
                        .iter()
                        .map(|(name, entry)| (name.clone(), entry.description.clone()))
                        .collect();

                    for (name, description) in library_entries {
                        ui.horizontal(|ui| {
                            let name_clone = name.clone();
                            if ui.button(&name).clicked() {
                                self.load_from_library(&name_clone);
                            }
                            ui.label(&description);
                        });
                    }
                });
            });
        }
    }
}

impl Default for ParticleEditor {
    fn default() -> Self {
        Self {
            config: ParticleSystemConfig::default(),
            is_playing: false,
            enhanced_config: ParticleEditorEnhancedConfig::default(),
            particle_library: HashMap::new(),
            sub_emitters: Vec::new(),
            current_particle_count: 0,
            system_name: "Particle System".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_config() {
        let config = ParticleSystemConfig::default();
        assert_eq!(config.emitter_type, EmitterType::Point);
        assert_eq!(config.emission_rate, 10.0);
    }

    #[test]
    fn test_particle_presets() {
        let fire_config = ParticlePreset::Fire.to_config();
        assert_eq!(fire_config.emitter_type, EmitterType::Point);

        let explosion_config = ParticlePreset::Explosion.to_config();
        assert!(!explosion_config.looping);
    }

    #[test]
    fn test_particle_editor() {
        let mut editor = ParticleEditor::new();
        editor.load_preset(ParticlePreset::Fire);

        assert_eq!(editor.config.emitter_type, EmitterType::Point);
        assert!(editor.config.emission_rate > 0.0);
    }
}
