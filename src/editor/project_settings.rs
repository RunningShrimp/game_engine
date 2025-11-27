use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 项目设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSettings {
    /// 项目名称
    pub project_name: String,
    /// 项目版本
    pub project_version: String,
    /// 项目描述
    pub project_description: String,
    /// 项目作者
    pub project_author: String,
    
    /// 渲染设置
    pub render_settings: RenderSettings,
    /// 物理设置
    pub physics_settings: PhysicsSettings,
    /// 音频设置
    pub audio_settings: AudioSettings,
    /// 输入设置
    pub input_settings: InputSettings,
}

impl Default for ProjectSettings {
    fn default() -> Self {
        Self {
            project_name: "My Game".to_string(),
            project_version: "0.1.0".to_string(),
            project_description: "A game made with Rust Game Engine".to_string(),
            project_author: "Unknown".to_string(),
            render_settings: RenderSettings::default(),
            physics_settings: PhysicsSettings::default(),
            audio_settings: AudioSettings::default(),
            input_settings: InputSettings::default(),
        }
    }
}

/// 渲染设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    /// 窗口宽度
    pub window_width: u32,
    /// 窗口高度
    pub window_height: u32,
    /// 是否全屏
    pub fullscreen: bool,
    /// 是否垂直同步
    pub vsync: bool,
    /// 最大FPS (0表示无限制)
    pub max_fps: u32,
    /// 抗锯齿级别
    pub msaa_samples: u32,
    /// 阴影质量
    pub shadow_quality: ShadowQuality,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            window_width: 1280,
            window_height: 720,
            fullscreen: false,
            vsync: true,
            max_fps: 0,
            msaa_samples: 4,
            shadow_quality: ShadowQuality::High,
        }
    }
}

/// 阴影质量
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShadowQuality {
    Low,
    Medium,
    High,
    Ultra,
}

impl ShadowQuality {
    pub fn resolution(&self) -> u32 {
        match self {
            ShadowQuality::Low => 512,
            ShadowQuality::Medium => 1024,
            ShadowQuality::High => 2048,
            ShadowQuality::Ultra => 4096,
        }
    }
}

/// 物理设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicsSettings {
    /// 重力
    pub gravity: [f32; 3],
    /// 固定时间步长 (秒)
    pub fixed_timestep: f32,
    /// 最大子步数
    pub max_substeps: u32,
}

impl Default for PhysicsSettings {
    fn default() -> Self {
        Self {
            gravity: [0.0, -9.81, 0.0],
            fixed_timestep: 1.0 / 60.0,
            max_substeps: 4,
        }
    }
}

/// 音频设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSettings {
    /// 主音量 (0.0-1.0)
    pub master_volume: f32,
    /// 音效音量 (0.0-1.0)
    pub sfx_volume: f32,
    /// 音乐音量 (0.0-1.0)
    pub music_volume: f32,
}

impl Default for AudioSettings {
    fn default() -> Self {
        Self {
            master_volume: 1.0,
            sfx_volume: 1.0,
            music_volume: 0.8,
        }
    }
}

/// 输入设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputSettings {
    /// 鼠标灵敏度
    pub mouse_sensitivity: f32,
    /// 是否反转Y轴
    pub invert_y_axis: bool,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            invert_y_axis: false,
        }
    }
}

/// 项目设置管理器
pub struct ProjectSettingsManager {
    /// 当前设置
    pub settings: ProjectSettings,
    /// 设置文件路径
    settings_path: PathBuf,
    /// 是否有未保存的更改
    pub has_unsaved_changes: bool,
}

impl ProjectSettingsManager {
    pub fn new(settings_path: PathBuf) -> Self {
        Self {
            settings: ProjectSettings::default(),
            settings_path,
            has_unsaved_changes: false,
        }
    }
    
    /// 加载设置
    pub fn load(&mut self) -> Result<(), String> {
        if !self.settings_path.exists() {
            return Ok(());
        }
        
        let content = std::fs::read_to_string(&self.settings_path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;
        
        self.settings = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings file: {}", e))?;
        
        self.has_unsaved_changes = false;
        
        Ok(())
    }
    
    /// 保存设置
    pub fn save(&mut self) -> Result<(), String> {
        let content = serde_json::to_string_pretty(&self.settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;
        
        // 确保目录存在
        if let Some(parent) = self.settings_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }
        
        std::fs::write(&self.settings_path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;
        
        self.has_unsaved_changes = false;
        
        Ok(())
    }
    
    /// 渲染设置UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Project Settings");
        ui.separator();
        
        // 项目信息
        ui.collapsing("Project Info", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                if ui.text_edit_singleline(&mut self.settings.project_name).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Version:");
                if ui.text_edit_singleline(&mut self.settings.project_version).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Author:");
                if ui.text_edit_singleline(&mut self.settings.project_author).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Description:");
                if ui.text_edit_multiline(&mut self.settings.project_description).changed() {
                    self.has_unsaved_changes = true;
                }
            });
        });
        
        ui.separator();
        
        // 渲染设置
        ui.collapsing("Render Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Window Size:");
                if ui.add(egui::DragValue::new(&mut self.settings.render_settings.window_width).prefix("W: ")).changed() {
                    self.has_unsaved_changes = true;
                }
                if ui.add(egui::DragValue::new(&mut self.settings.render_settings.window_height).prefix("H: ")).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            if ui.checkbox(&mut self.settings.render_settings.fullscreen, "Fullscreen").changed() {
                self.has_unsaved_changes = true;
            }
            
            if ui.checkbox(&mut self.settings.render_settings.vsync, "VSync").changed() {
                self.has_unsaved_changes = true;
            }
            
            ui.horizontal(|ui| {
                ui.label("Max FPS:");
                if ui.add(egui::DragValue::new(&mut self.settings.render_settings.max_fps).range(0..=300)).changed() {
                    self.has_unsaved_changes = true;
                }
                ui.label("(0 = unlimited)");
            });
            
            ui.horizontal(|ui| {
                ui.label("MSAA Samples:");
                if ui.add(egui::DragValue::new(&mut self.settings.render_settings.msaa_samples).range(1..=8)).changed() {
                    self.has_unsaved_changes = true;
                }
            });
        });
        
        ui.separator();
        
        // 物理设置
        ui.collapsing("Physics Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Gravity:");
                if ui.add(egui::DragValue::new(&mut self.settings.physics_settings.gravity[0]).prefix("X: ").speed(0.1)).changed() {
                    self.has_unsaved_changes = true;
                }
                if ui.add(egui::DragValue::new(&mut self.settings.physics_settings.gravity[1]).prefix("Y: ").speed(0.1)).changed() {
                    self.has_unsaved_changes = true;
                }
                if ui.add(egui::DragValue::new(&mut self.settings.physics_settings.gravity[2]).prefix("Z: ").speed(0.1)).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Fixed Timestep:");
                if ui.add(egui::DragValue::new(&mut self.settings.physics_settings.fixed_timestep).speed(0.001).range(0.001..=0.1)).changed() {
                    self.has_unsaved_changes = true;
                }
                ui.label("seconds");
            });
        });
        
        ui.separator();
        
        // 音频设置
        ui.collapsing("Audio Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Master Volume:");
                if ui.add(egui::Slider::new(&mut self.settings.audio_settings.master_volume, 0.0..=1.0)).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("SFX Volume:");
                if ui.add(egui::Slider::new(&mut self.settings.audio_settings.sfx_volume, 0.0..=1.0)).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            ui.horizontal(|ui| {
                ui.label("Music Volume:");
                if ui.add(egui::Slider::new(&mut self.settings.audio_settings.music_volume, 0.0..=1.0)).changed() {
                    self.has_unsaved_changes = true;
                }
            });
        });
        
        ui.separator();
        
        // 输入设置
        ui.collapsing("Input Settings", |ui| {
            ui.horizontal(|ui| {
                ui.label("Mouse Sensitivity:");
                if ui.add(egui::Slider::new(&mut self.settings.input_settings.mouse_sensitivity, 0.1..=5.0)).changed() {
                    self.has_unsaved_changes = true;
                }
            });
            
            if ui.checkbox(&mut self.settings.input_settings.invert_y_axis, "Invert Y Axis").changed() {
                self.has_unsaved_changes = true;
            }
        });
        
        ui.separator();
        
        // 保存按钮
        ui.horizontal(|ui| {
            if ui.button("Save Settings").clicked() {
                if let Err(e) = self.save() {
                    eprintln!("Failed to save settings: {}", e);
                }
            }
            
            if self.has_unsaved_changes {
                ui.label("⚠ Unsaved changes");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    
    #[test]
    fn test_project_settings() {
        let settings = ProjectSettings::default();
        assert_eq!(settings.project_name, "My Game");
        assert_eq!(settings.render_settings.window_width, 1280);
    }
    
    #[test]
    fn test_settings_serialization() {
        let settings = ProjectSettings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: ProjectSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.project_name, settings.project_name);
    }
    
    #[test]
    fn test_settings_manager() {
        let temp_dir = env::temp_dir();
        let settings_path = temp_dir.join("test_settings.json");
        
        let mut manager = ProjectSettingsManager::new(settings_path.clone());
        manager.settings.project_name = "Test Project".to_string();
        
        // 保存
        manager.save().unwrap();
        assert!(!manager.has_unsaved_changes);
        
        // 加载
        let mut manager2 = ProjectSettingsManager::new(settings_path.clone());
        manager2.load().unwrap();
        assert_eq!(manager2.settings.project_name, "Test Project");
        
        // 清理
        std::fs::remove_file(settings_path).ok();
    }
}
