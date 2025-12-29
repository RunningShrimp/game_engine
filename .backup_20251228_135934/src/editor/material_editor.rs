use crate::render::pbr::{PbrMaterial, PbrMaterialFull, PbrTextures};
use std::collections::HashMap;
use std::path::PathBuf;

/// 材质预设（增强功能）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPreset {
    StandardMetal,
    StandardNonMetal,
    Emissive,
    Glass,
    Plastic,
    Rubber,
    Skin,
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

/// 材质库条目（增强功能）
#[derive(Debug, Clone)]
pub struct MaterialLibraryEntry {
    pub name: String,
    pub material: PbrMaterialFull,
    pub thumbnail_path: Option<PathBuf>,
    pub tags: Vec<String>,
}

/// 材质编辑器
#[derive(Default)]
pub struct MaterialEditor {
    pub selected_material: Option<usize>,
    pub materials: Vec<PbrMaterialFull>,
    /// 材质名称（增强功能）
    pub material_names: Vec<String>,
    /// 材质库（增强功能）
    pub material_library: HashMap<String, MaterialLibraryEntry>,
    /// 搜索过滤（增强功能）
    pub search_filter: String,
    /// 显示预览（增强功能）
    pub show_preview: bool,
    /// 预览大小（增强功能）
    pub preview_size: f32,
}

impl MaterialEditor {
    pub fn new() -> Self {
        let mut editor = Self {
            materials: vec![PbrMaterialFull::default()],
            material_names: vec!["Material 0".to_string()],
            material_library: HashMap::new(),
            search_filter: String::new(),
            show_preview: true,
            preview_size: 200.0,
            ..Default::default()
        };
        // 初始化材质库预设
        editor.init_presets();
        editor
    }

    /// 初始化材质预设（增强功能）
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

    /// 加载预设（增强功能）
    pub fn load_preset(&mut self, preset: MaterialPreset) {
        if let Some(index) = self.selected_material {
            if let Some(material) = self.materials.get_mut(index) {
                *material = preset.to_material();
            }
        } else {
            let new_material = preset.to_material();
            self.materials.push(new_material);
            self.material_names.push(format!("{} Material", preset.name()));
            self.selected_material = Some(self.materials.len() - 1);
        }
    }

    /// 保存材质到库（增强功能）
    pub fn save_to_library(&mut self, name: String) {
        if let Some(index) = self.selected_material
            && let Some(material) = self.materials.get(index) {
                let entry = MaterialLibraryEntry {
                    name: name.clone(),
                    material: material.clone(),
                    thumbnail_path: None,
                    tags: vec!["Custom".to_string()],
                };
                self.material_library.insert(name, entry);
            }
    }

    /// 从库加载材质（增强功能）
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

    /// 复制材质（增强功能）
    pub fn duplicate_material(&mut self, index: usize) {
        if let Some(material) = self.materials.get(index) {
            self.materials.push(material.clone());
            self.material_names.push(format!("{} Copy", self.material_names[index]));
            self.selected_material = Some(self.materials.len() - 1);
        }
    }

    /// 删除材质（增强功能）
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

    /// 渲染材质编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Material Editor");
        ui.separator();

        // 材质列表
        ui.label("Materials:");
        for (i, _material) in self.materials.iter().enumerate() {
            let is_selected = self.selected_material == Some(i);
            if ui.selectable_label(is_selected, format!("Material {}", i)).clicked() {
                self.selected_material = Some(i);
            }
        }

        // 工具栏（增强功能）
        ui.horizontal(|ui| {
            if ui.button("+ New").clicked() {
                self.materials.push(PbrMaterialFull::default());
                self.material_names.push(format!("Material {}", self.materials.len() - 1));
                self.selected_material = Some(self.materials.len() - 1);
            }
            if ui.button("📋 Duplicate").clicked()
                && let Some(index) = self.selected_material {
                    self.duplicate_material(index);
                }
            if ui.button("💾 Save to Library").clicked()
                && let Some(index) = self.selected_material {
                    let name = self.material_names[index].clone();
                    self.save_to_library(name);
                }
            ui.checkbox(&mut self.show_preview, "Preview");
        });

        ui.separator();

        // 材质列表（增强功能）
        ui.collapsing("Materials", |ui| {
            // 搜索框
            ui.text_edit_singleline(&mut self.search_filter);
            ui.separator();

            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    // 先收集需要删除的索引
                    let mut to_delete: Option<usize> = None;
                    
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
                                to_delete = Some(i);
                            }
                        });
                    }
                    
                    // 在循环外处理删除
                    if let Some(idx) = to_delete {
                        self.delete_material(idx);
                    }
                });
        });

        ui.separator();

        // 材质预设（增强功能）
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

        ui.separator();

        // 材质属性编辑
        if let Some(index) = self.selected_material {
            // 先获取材质名称
            let material_name = format!("Editing Material {}", index);
            ui.label(material_name);
            ui.separator();
            
            if let Some(material_full) = self.materials.get_mut(index) {
                // 获取可变引用到材质
                let material = &mut material_full.material;

                // 基础颜色
                ui.label("Base Color:");
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

                // 颜色选择器
                let mut color = [
                    material.base_color.x,
                    material.base_color.y,
                    material.base_color.z,
                ];
                if ui.color_edit_button_rgb(&mut color).changed() {
                    material.base_color.x = color[0];
                    material.base_color.y = color[1];
                    material.base_color.z = color[2];
                }

                ui.separator();

                // 金属度
                ui.label("Metallic:");
                ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0));

                // 粗糙度
                ui.label("Roughness:");
                ui.add(egui::Slider::new(&mut material.roughness, 0.0..=1.0));

                // 环境光遮蔽
                ui.label("Ambient Occlusion:");
                ui.add(egui::Slider::new(
                    &mut material.ambient_occlusion,
                    0.0..=1.0,
                ));

                // 法线强度
                ui.label("Normal Scale:");
                ui.add(egui::Slider::new(&mut material.normal_scale, 0.0..=2.0));

                ui.separator();

                // 自发光
                ui.label("Emissive:");
                ui.horizontal(|ui| {
                    ui.add(
                        egui::DragValue::new(&mut material.emissive.x)
                            .prefix("R: ")
                            .speed(0.01)
                            .range(0.0..=10.0),
                    );
                    ui.add(
                        egui::DragValue::new(&mut material.emissive.y)
                            .prefix("G: ")
                            .speed(0.01)
                            .range(0.0..=10.0),
                    );
                    ui.add(
                        egui::DragValue::new(&mut material.emissive.z)
                            .prefix("B: ")
                            .speed(0.01)
                            .range(0.0..=10.0),
                    );
                });

                ui.separator();
 
                // 纹理槽
                ui.label("Textures:");
                ui.label(format!("  Base Color: {:?}", material_full.textures.base_color_texture));
                ui.label(format!(
                    "  Metallic/Roughness: {:?}",
                    material_full.textures.metallic_roughness_texture
                ));
                ui.label(format!("  Normal: {:?}", material_full.textures.normal_texture));
                ui.label(format!("  AO: {:?}", material_full.textures.ao_texture));
                ui.label(format!("  Emissive: {:?}", material_full.textures.emissive_texture));

                ui.separator();

                // 材质预览（增强功能）
                if self.show_preview {
                    ui.collapsing("Preview", |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Size:");
                            ui.add(egui::Slider::new(&mut self.preview_size, 50.0..=500.0));
                        });
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

        // 材质库（增强功能）
        ui.collapsing("Material Library", |ui| {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let library_names: Vec<String> = self.material_library.keys().cloned().collect();
                    for name in library_names {
                        let name_clone = name.clone();
                        let entry = self.material_library.get(&name).cloned();
                        ui.horizontal(|ui| {
                            if ui.button(&name).clicked() {
                                let name_for_load = name_clone.clone();
                                self.load_from_library(&name_for_load);
                            }
                            let tags = entry.as_ref().map(|e| e.tags.join(", ")).unwrap_or_else(String::new);
                            ui.label(format!("Tags: {}", tags));
                        });
                    }
                });
        });
    }
}
