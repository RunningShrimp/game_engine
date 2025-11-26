use glam::{Vec3, Vec4};
use serde::{Deserialize, Serialize};

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

impl Default for ParticleSystemConfig {
    fn default() -> Self {
        Self {
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
        }
    }
}

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

/// 粒子编辑器
pub struct ParticleEditor {
    /// 粒子系统配置
    pub config: ParticleSystemConfig,
    /// 是否正在播放
    pub is_playing: bool,
}

impl ParticleEditor {
    pub fn new() -> Self {
        Self {
            config: ParticleSystemConfig::default(),
            is_playing: false,
        }
    }
    
    /// 加载预设
    pub fn load_preset(&mut self, preset: ParticlePreset) {
        self.config = preset.to_config();
    }
    
    /// 渲染粒子编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Particle Editor");
        ui.separator();
        
        // 预设选择
        ui.horizontal(|ui| {
            ui.label("Presets:");
            if ui.button(ParticlePreset::Fire.name()).clicked() {
                self.load_preset(ParticlePreset::Fire);
            }
            if ui.button(ParticlePreset::Smoke.name()).clicked() {
                self.load_preset(ParticlePreset::Smoke);
            }
            if ui.button(ParticlePreset::Explosion.name()).clicked() {
                self.load_preset(ParticlePreset::Explosion);
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("");
            if ui.button(ParticlePreset::Rain.name()).clicked() {
                self.load_preset(ParticlePreset::Rain);
            }
            if ui.button(ParticlePreset::Snow.name()).clicked() {
                self.load_preset(ParticlePreset::Snow);
            }
            if ui.button(ParticlePreset::Magic.name()).clicked() {
                self.load_preset(ParticlePreset::Magic);
            }
        });
        
        ui.separator();
        
        // 发射器设置
        ui.collapsing("Emitter Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Emitter Type:");
                egui::ComboBox::from_label("")
                    .selected_text(self.config.emitter_type.name())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.config.emitter_type, EmitterType::Point, "Point");
                        ui.selectable_value(&mut self.config.emitter_type, EmitterType::Sphere, "Sphere");
                        ui.selectable_value(&mut self.config.emitter_type, EmitterType::Box, "Box");
                        ui.selectable_value(&mut self.config.emitter_type, EmitterType::Cone, "Cone");
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
        
        // 粒子生命周期
        ui.collapsing("Lifetime Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Lifetime:");
                ui.add(egui::Slider::new(&mut self.config.lifetime, 0.1..=10.0));
            });
            
            ui.horizontal(|ui| {
                ui.label("Variance:");
                ui.add(egui::Slider::new(&mut self.config.lifetime_variance, 0.0..=2.0));
            });
        });
        
        ui.separator();
        
        // 速度设置
        ui.collapsing("Velocity Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Initial Velocity:");
                ui.add(egui::DragValue::new(&mut self.config.initial_velocity.x).prefix("X: ").speed(0.1));
                ui.add(egui::DragValue::new(&mut self.config.initial_velocity.y).prefix("Y: ").speed(0.1));
                ui.add(egui::DragValue::new(&mut self.config.initial_velocity.z).prefix("Z: ").speed(0.1));
            });
            
            ui.horizontal(|ui| {
                ui.label("Variance:");
                ui.add(egui::DragValue::new(&mut self.config.velocity_variance.x).prefix("X: ").speed(0.1));
                ui.add(egui::DragValue::new(&mut self.config.velocity_variance.y).prefix("Y: ").speed(0.1));
                ui.add(egui::DragValue::new(&mut self.config.velocity_variance.z).prefix("Z: ").speed(0.1));
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
        
        // 播放控制
        ui.horizontal(|ui| {
            if ui.button(if self.is_playing { "Stop" } else { "Play" }).clicked() {
                self.is_playing = !self.is_playing;
            }
            
            if ui.button("Reset").clicked() {
                self.config = ParticleSystemConfig::default();
                self.is_playing = false;
            }
        });
    }
}

impl Default for ParticleEditor {
    fn default() -> Self {
        Self::new()
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
