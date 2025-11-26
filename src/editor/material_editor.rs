use crate::render::pbr::PbrMaterialFull;

/// 材质编辑器
pub struct MaterialEditor {
    pub selected_material: Option<usize>,
    pub materials: Vec<PbrMaterialFull>,
}

impl MaterialEditor {
    pub fn new() -> Self {
        Self {
            selected_material: None,
            materials: vec![
                // 默认材质
                PbrMaterialFull::default(),
            ],
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
        
        // 添加新材质按钮
        if ui.button("+ Add Material").clicked() {
            self.materials.push(PbrMaterialFull::default());
            self.selected_material = Some(self.materials.len() - 1);
        }
        
        ui.separator();
        
        // 材质属性编辑
        if let Some(index) = self.selected_material {
            if let Some(material_full) = self.materials.get_mut(index) {
                let material = &mut material_full.material;
                let textures = &material_full.textures;
                ui.label(format!("Editing Material {}", index));
                ui.separator();
                
                // 基础颜色
                ui.label("Base Color:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut material.base_color.x).prefix("R: ").speed(0.01).clamp_range(0.0..=1.0));
                    ui.add(egui::DragValue::new(&mut material.base_color.y).prefix("G: ").speed(0.01).clamp_range(0.0..=1.0));
                    ui.add(egui::DragValue::new(&mut material.base_color.z).prefix("B: ").speed(0.01).clamp_range(0.0..=1.0));
                    ui.add(egui::DragValue::new(&mut material.base_color.w).prefix("A: ").speed(0.01).clamp_range(0.0..=1.0));
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
                ui.add(egui::Slider::new(&mut material.ambient_occlusion, 0.0..=1.0));
                
                // 法线强度
                ui.label("Normal Scale:");
                ui.add(egui::Slider::new(&mut material.normal_scale, 0.0..=2.0));
                
                ui.separator();
                
                // 自发光
                ui.label("Emissive:");
                ui.horizontal(|ui| {
                    ui.add(egui::DragValue::new(&mut material.emissive.x).prefix("R: ").speed(0.01).clamp_range(0.0..=10.0));
                    ui.add(egui::DragValue::new(&mut material.emissive.y).prefix("G: ").speed(0.01).clamp_range(0.0..=10.0));
                    ui.add(egui::DragValue::new(&mut material.emissive.z).prefix("B: ").speed(0.01).clamp_range(0.0..=10.0));
                });
                
                ui.separator();
                
                // 纹理槽
                ui.label("Textures:");
                ui.label(format!("  Base Color: {:?}", textures.base_color_texture));
                ui.label(format!("  Metallic/Roughness: {:?}", textures.metallic_roughness_texture));
                ui.label(format!("  Normal: {:?}", textures.normal_texture));
                ui.label(format!("  AO: {:?}", textures.ao_texture));
                ui.label(format!("  Emissive: {:?}", textures.emissive_texture));
                
                ui.separator();
                
                // 预览 (占位)
                ui.label("Preview:");
                ui.label("(Material preview will be displayed here)");
            }
        } else {
            ui.label("No material selected");
        }
    }
}

impl Default for MaterialEditor {
    fn default() -> Self {
        Self::new()
    }
}
