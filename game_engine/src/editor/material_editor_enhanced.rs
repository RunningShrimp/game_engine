//! 增强的材质编辑器
//!
//! 提供完整的材质编辑功能：
//! - 材质预设和模板
//! - 纹理导入和管理
//! - 实时材质预览
//! - 材质库管理
//! - 材质导出/导入

use crate::render::pbr::{PbrMaterial, PbrMaterialFull, PbrTextures};
use std::collections::HashMap;
use std::path::PathBuf;

/// 材质预设
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPreset {
    /// 标准金属材质
    StandardMetal,
    /// 标准非金属材质
    StandardNonMetal,
    /// 发光材质
    Emissive,
    /// 玻璃材质
    Glass,
    /// 塑料材质
    Plastic,
    /// 橡胶材质
    Rubber,
    /// 皮肤材质
    Skin,
    /// 布料材质
    Fabric,
}

impl MaterialPreset {
    pub fn name(&self) -> &'static str {
        match self {
            MaterialPreset::StandardMetal => "Standard Metal",
            MaterialPreset::StandardNonMetal => "Standard Non-Metal",
            MaterialPreset::Emissive => "Emissive",
            MaterialPreset::Glass => "Glass",
            MaterialPreset::Plastic => "Plastic",
            MaterialPreset::Rubber => "Rubber",
            MaterialPreset::Skin => "Skin",
            MaterialPreset::Fabric => "Fabric",
        }
    }

    pub fn to_material(&self) -> PbrMaterialFull {
        match self {
            MaterialPreset::StandardMetal => PbrMaterialFull {
                material: PbrMaterial {
                    base_color: glam::Vec4::new(0.8, 0.8, 0.8, 1.0),
                    metallic: 1.0,
                    roughness: 0.2,
                    ambient_occlusion: 1.0,
                    emissive: glam::Vec3::ZERO,
                    normal_scale: 1.0,
                    ..Default::default()
                },
                textures: PbrTextures::default(),
            },
            MaterialPreset::StandardNonMetal => PbrMaterialFull {
                material: PbrMaterial {
                    base_color: glam::Vec4::new(0.8, 0.8, 0.8, 1.0),
                    metallic: 0.0,
                    roughness: 0.5,
                    ambient_occlusion: 1.0,
                    emissive: glam::Vec3::ZERO,
                    normal_scale: 1.0,
                    ..Default::default()
                },
                textures: PbrTextures::default(),
            },
            MaterialPreset::Emissive => PbrMaterialFull {
                material: PbrMaterial {
                    base_color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
                    metallic: 0.0,
                    roughness: 0.0,
                    ambient_occlusion: 1.0,
                    emissive: glam::Vec3::new(1.0, 1.0, 1.0),
                    normal_scale: 1.0,
                    ..Default::default()
                },
                textures: PbrTextures::default(),
            },
            MaterialPreset::Glass => PbrMaterialFull {
                material: PbrMaterial {
                    base_color: glam::Vec4::new(0.9, 0.9, 1.0, 0.3),
                    metallic: 0.0,
                    roughness: 0.0,
                    ambient_occlusion: 1.0,
                    emissive: glam::Vec3::ZERO,
                    normal_scale: 1.0,
                    clearcoat: 1.0,
                    clearcoat_roughness: 0.0,
                    ..Default::default()
                },
                textures: PbrTextures::default(),
            },
            MaterialPreset::Plastic => PbrMaterialFull {
                material: PbrMaterial {
                    base_color: glam::Vec4::new(0.8, 0.8, 0.8, 1.0),
                    metallic: 0.0,
                    roughness: 0.3,
                    ambient_occlusion: 1.0,
                    emissive: glam::Vec3::ZERO,
                    normal_scale: 1.0,
                    ..Default::default()
                },
                textures: PbrTextures::default(),
            },
            MaterialPreset::Rubber => PbrMaterialFull {
                material: PbrMaterial {
                    base_color: glam::Vec4::new(0.2, 0.2, 0.2, 1.0),
                    metallic: 0.0,
                    roughness: 0.8,
                    ambient_occlusion: 1.0,
                    emissive: glam::Vec3::ZERO,
                    normal_scale: 1.0,
                    ..Default::default()
                },
                textures: PbrTextures::default(),
            },
            MaterialPreset::Skin => PbrMaterialFull {
                material: PbrMaterial {
                    base_color: glam::Vec4::new(0.9, 0.7, 0.6, 1.0),
                    metallic: 0.0,
                    roughness: 0.6,
                    ambient_occlusion: 1.0,
                    emissive: glam::Vec3::ZERO,
                    normal_scale: 1.0,
                    ..Default::default()
                },
                textures: PbrTextures::default(),
            },
            MaterialPreset::Fabric => PbrMaterialFull {
                material: PbrMaterial {
                    base_color: glam::Vec4::new(0.8, 0.6, 0.4, 1.0),
                    metallic: 0.0,
                    roughness: 0.9,
                    ambient_occlusion: 1.0,
                    emissive: glam::Vec3::ZERO,
                    normal_scale: 1.0,
                    ..Default::default()
                },
                textures: PbrTextures::default(),
            },
        }
    }
}

/// 材质库条目
#[derive(Debug, Clone)]
pub struct MaterialLibraryEntry {
    pub name: String,
    pub material: PbrMaterialFull,
    pub thumbnail_path: Option<PathBuf>,
    pub tags: Vec<String>,
}

/// 增强的材质编辑器
pub struct MaterialEditorEnhanced {
    /// 材质列表
    pub materials: Vec<PbrMaterialFull>,
    /// 选中的材质索引
    pub selected_material: Option<usize>,
    /// 材质库
    pub material_library: HashMap<String, MaterialLibraryEntry>,
    /// 当前材质名称
    pub material_names: Vec<String>,
    /// 搜索过滤
    pub search_filter: String,
    /// 显示预览
    pub show_preview: bool,
    /// 预览大小
    pub preview_size: f32,
}

impl MaterialEditorEnhanced {
    pub fn new() -> Self {
        let mut editor = Self {
            materials: vec![PbrMaterialFull::default()],
            selected_material: Some(0),
            material_library: HashMap::new(),
            material_names: vec!["Material 0".to_string()],
            search_filter: String::new(),
            show_preview: true,
            preview_size: 200.0,
        };

        // 初始化材质库预设
        editor.init_presets();
        editor
    }

    /// 初始化材质预设
    fn init_presets(&mut self) {
        for preset in [
            MaterialPreset::StandardMetal,
            MaterialPreset::StandardNonMetal,
            MaterialPreset::Emissive,
            MaterialPreset::Glass,
            MaterialPreset::Plastic,
            MaterialPreset::Rubber,
            MaterialPreset::Skin,
            MaterialPreset::Fabric,
        ] {
            let entry = MaterialLibraryEntry {
                name: preset.name().to_string(),
                material: preset.to_material(),
                thumbnail_path: None,
                tags: vec![preset.name().to_string()],
            };
            self.material_library.insert(preset.name().to_string(), entry);
        }
    }

    /// 加载预设
    pub fn load_preset(&mut self, preset: MaterialPreset) {
        if let Some(index) = self.selected_material {
            if let Some(material) = self.materials.get_mut(index) {
                *material = preset.to_material();
            }
        } else {
            // 创建新材质
            let new_material = preset.to_material();
            self.materials.push(new_material);
            self.material_names.push(format!("{} Material", preset.name()));
            self.selected_material = Some(self.materials.len() - 1);
        }
    }

    /// 添加新材质
    pub fn add_material(&mut self) {
        self.materials.push(PbrMaterialFull::default());
        self.material_names.push(format!("Material {}", self.materials.len() - 1));
        self.selected_material = Some(self.materials.len() - 1);
    }

    /// 删除材质
    pub fn delete_material(&mut self, index: usize) {
        if index < self.materials.len() {
            self.materials.remove(index);
            self.material_names.remove(index);
            if self.selected_material == Some(index) {
                self.selected_material = if self.materials.is_empty() {
                    None
                } else {
                    Some((index - 1).min(self.materials.len() - 1))
                };
            }
        }
    }

    /// 复制材质
    pub fn duplicate_material(&mut self, index: usize) {
        if let Some(material) = self.materials.get(index) {
            self.materials.push(material.clone());
            self.material_names.push(format!("{} Copy", self.material_names[index]));
            self.selected_material = Some(self.materials.len() - 1);
        }
    }

    /// 保存材质到库
    pub fn save_to_library(&mut self, name: String) {
        if let Some(index) = self.selected_material {
            if let Some(material) = self.materials.get(index) {
                let entry = MaterialLibraryEntry {
                    name: name.clone(),
                    material: material.clone(),
                    thumbnail_path: None,
                    tags: vec!["Custom".to_string()],
                };
                self.material_library.insert(name, entry);
            }
        }
    }

    /// 从库加载材质
    pub fn load_from_library(&mut self, name: &str) {
        if let Some(entry) = self.material_library.get(name) {
            if let Some(index) = self.selected_material {
                if let Some(material) = self.materials.get_mut(index) {
                    *material = entry.material.clone();
                }
            } else {
                self.materials.push(entry.material.clone());
                self.material_names.push(entry.name.clone());
                self.selected_material = Some(self.materials.len() - 1);
            }
        }
    }

    /// 渲染材质编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Material Editor");
        ui.separator();

        // 工具栏
        ui.horizontal(|ui| {
            if ui.button("+ New").clicked() {
                self.add_material();
            }
            if ui.button("📋 Duplicate").clicked() {
                if let Some(index) = self.selected_material {
                    self.duplicate_material(index);
                }
            }
            if ui.button("💾 Save to Library").clicked() {
                if let Some(index) = self.selected_material {
                    let name = self.material_names[index].clone();
                    self.save_to_library(name);
                }
            }
            ui.checkbox(&mut self.show_preview, "Preview");
        });

        ui.separator();

        // 材质列表
        ui.collapsing("Materials", |ui| {
            // 搜索框
            ui.text_edit_singleline(&mut self.search_filter);
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for (i, name) in self.material_names.iter().enumerate() {
                        if !self.search_filter.is_empty()
                            && !name.to_lowercase().contains(&self.search_filter.to_lowercase())
                        {
                            continue;
                        }

                        let is_selected = self.selected_material == Some(i);
                        ui.horizontal(|ui| {
                            if ui.selectable_label(is_selected, name).clicked() {
                                self.selected_material = Some(i);
                            }
                            if ui.button("🗑").clicked() {
                                self.delete_material(i);
                            }
                        });
                    }
                });
        });

        ui.separator();

        // 材质预设
        ui.collapsing("Presets", |ui| {
            ui.horizontal(|ui| {
                for preset in [
                    MaterialPreset::StandardMetal,
                    MaterialPreset::StandardNonMetal,
                    MaterialPreset::Emissive,
                ] {
                    if ui.button(preset.name()).clicked() {
                        self.load_preset(preset);
                    }
                }
            });
            ui.horizontal(|ui| {
                for preset in [
                    MaterialPreset::Glass,
                    MaterialPreset::Plastic,
                    MaterialPreset::Rubber,
                ] {
                    if ui.button(preset.name()).clicked() {
                        self.load_preset(preset);
                    }
                }
            });
            ui.horizontal(|ui| {
                for preset in [MaterialPreset::Skin, MaterialPreset::Fabric] {
                    if ui.button(preset.name()).clicked() {
                        self.load_preset(preset);
                    }
                }
            });
        });

        ui.separator();

        // 材质属性编辑
        if let Some(index) = self.selected_material {
            if let Some(material_full) = self.materials.get_mut(index) {
                let material = &mut material_full.material;
                let textures = &mut material_full.textures;

                // 材质名称
                ui.horizontal(|ui| {
                    ui.label("Name:");
                    ui.text_edit_singleline(&mut self.material_names[index]);
                });

                ui.separator();

                // 基础颜色
                ui.collapsing("Base Color", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color = [
                            material.base_color.x,
                            material.base_color.y,
                            material.base_color.z,
                            material.base_color.w,
                        ];
                        if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
                            material.base_color = glam::Vec4::from_array(color);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut material.base_color.x)
                                .prefix("R: ")
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.add(
                            egui::DragValue::new(&mut material.base_color.y)
                                .prefix("G: ")
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.add(
                            egui::DragValue::new(&mut material.base_color.z)
                                .prefix("B: ")
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                        ui.add(
                            egui::DragValue::new(&mut material.base_color.w)
                                .prefix("A: ")
                                .speed(0.01)
                                .range(0.0..=1.0),
                        );
                    });
                });

                ui.separator();

                // PBR参数
                ui.collapsing("PBR Properties", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Metallic:");
                        ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Roughness:");
                        ui.add(egui::Slider::new(&mut material.roughness, 0.0..=1.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Ambient Occlusion:");
                        ui.add(egui::Slider::new(
                            &mut material.ambient_occlusion,
                            0.0..=1.0,
                        ));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Normal Scale:");
                        ui.add(egui::Slider::new(&mut material.normal_scale, 0.0..=2.0));
                    });
                });

                ui.separator();

                // 自发光
                ui.collapsing("Emissive", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let mut color = [
                            material.emissive.x,
                            material.emissive.y,
                            material.emissive.z,
                        ];
                        if ui.color_edit_button_rgb(&mut color).changed() {
                            material.emissive = glam::Vec3::from_array(color);
                        }
                    });

                    ui.horizontal(|ui| {
                        ui.add(
                            egui::DragValue::new(&mut material.emissive.x)
                                .prefix("R: ")
                                .speed(0.1)
                                .range(0.0..=10.0),
                        );
                        ui.add(
                            egui::DragValue::new(&mut material.emissive.y)
                                .prefix("G: ")
                                .speed(0.1)
                                .range(0.0..=10.0),
                        );
                        ui.add(
                            egui::DragValue::new(&mut material.emissive.z)
                                .prefix("B: ")
                                .speed(0.1)
                                .range(0.0..=10.0),
                        );
                    });
                });

                ui.separator();

                // 高级属性
                ui.collapsing("Advanced", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Clearcoat:");
                        ui.add(egui::Slider::new(&mut material.clearcoat, 0.0..=1.0));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Clearcoat Roughness:");
                        ui.add(egui::Slider::new(
                            &mut material.clearcoat_roughness,
                            0.0..=1.0,
                        ));
                    });

                    ui.horizontal(|ui| {
                        ui.label("Anisotropy:");
                        ui.add(egui::Slider::new(&mut material.anisotropy, 0.0..=1.0));
                    });

                    // UV变换
                    ui.label("UV Transform:");
                    ui.horizontal(|ui| {
                        ui.label("Offset:");
                        ui.add(
                            egui::DragValue::new(&mut material.uv_offset[0])
                                .prefix("U: ")
                                .speed(0.01),
                        );
                        ui.add(
                            egui::DragValue::new(&mut material.uv_offset[1])
                                .prefix("V: ")
                                .speed(0.01),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Scale:");
                        ui.add(
                            egui::DragValue::new(&mut material.uv_scale[0])
                                .prefix("U: ")
                                .speed(0.01)
                                .range(0.1..=10.0),
                        );
                        ui.add(
                            egui::DragValue::new(&mut material.uv_scale[1])
                                .prefix("V: ")
                                .speed(0.01)
                                .range(0.1..=10.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        ui.label("Rotation:");
                        ui.add(
                            egui::DragValue::new(&mut material.uv_rotation)
                                .suffix(" rad")
                                .speed(0.01),
                        );
                    });
                });

                ui.separator();

                // 纹理槽
                ui.collapsing("Textures", |ui| {
                    ui.label("Base Color Texture:");
                    ui.label(format!("  {:?}", textures.base_color_texture));
                    if ui.button("Load Texture").clicked() {
                        // TODO: 实现纹理加载
                    }

                    ui.label("Metallic/Roughness Texture:");
                    ui.label(format!("  {:?}", textures.metallic_roughness_texture));
                    if ui.button("Load Texture").clicked() {
                        // TODO: 实现纹理加载
                    }

                    ui.label("Normal Texture:");
                    ui.label(format!("  {:?}", textures.normal_texture));
                    if ui.button("Load Texture").clicked() {
                        // TODO: 实现纹理加载
                    }

                    ui.label("AO Texture:");
                    ui.label(format!("  {:?}", textures.ao_texture));
                    if ui.button("Load Texture").clicked() {
                        // TODO: 实现纹理加载
                    }

                    ui.label("Emissive Texture:");
                    ui.label(format!("  {:?}", textures.emissive_texture));
                    if ui.button("Load Texture").clicked() {
                        // TODO: 实现纹理加载
                    }
                });

                ui.separator();

                // 材质预览
                if self.show_preview {
                    ui.collapsing("Preview", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Size:");
                            ui.add(egui::Slider::new(&mut self.preview_size, 50.0..=500.0));
                        });
                        // 预览区域（占位）
                        ui.allocate_ui_with_layout(
                            egui::Vec2::new(self.preview_size, self.preview_size),
                            egui::Layout::top_down(egui::Align::Center),
                            |ui| {
                                ui.label("Material Preview");
                                ui.label("(3D preview will be displayed here)");
                            },
                        );
                    });
                }
            }
        } else {
            ui.label("No material selected");
        }

        ui.separator();

        // 材质库
        ui.collapsing("Material Library", |ui| {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let library_names: Vec<String> = self.material_library.keys().cloned().collect();
                    for name in library_names {
                        if let Some(entry) = self.material_library.get(&name) {
                            ui.horizontal(|ui| {
                                if ui.button(&name).clicked() {
                                    self.load_from_library(&name);
                                }
                                ui.label(format!("Tags: {}", entry.tags.join(", ")));
                            });
                        }
                    }
                });
        });
    }
}

impl Default for MaterialEditorEnhanced {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_presets() {
        let metal = MaterialPreset::StandardMetal.to_material();
        assert_eq!(metal.material.metallic, 1.0);

        let glass = MaterialPreset::Glass.to_material();
        assert!(glass.material.clearcoat > 0.0);
    }

    #[test]
    fn test_material_editor() {
        let mut editor = MaterialEditorEnhanced::new();
        assert!(!editor.materials.is_empty());
        assert!(editor.selected_material.is_some());
    }
}

