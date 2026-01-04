//! # DCC材质编辑器
//!
//! 提供材质编辑功能，包括：
//! - PBR参数实时调整
//! - 纹理槽管理
//! - 材质预览
//! - 材质预设

use crate::render::pbr::{PbrMaterial, PbrMaterialFull, PbrTextures};
use egui::*;
use glam::{Vec3, Vec4};
use std::collections::HashMap;
use std::path::PathBuf;

/// 材质ID类型
pub type MaterialID = usize;

/// 纹理类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TextureType {
    /// 基础颜色
    Albedo,
    /// 法线
    Normal,
    /// 粗糙度
    Roughness,
    /// 金属度
    Metallic,
    /// 环境光遮蔽
    AmbientOcclusion,
    /// 发光
    Emissive,
    /// 高光
    Clearcoat,
}

/// 纹理槽
#[derive(Debug, Clone)]
pub struct TextureSlot {
    /// 纹理类型
    pub texture_type: TextureType,
    /// 纹理路径
    pub path: Option<PathBuf>,
    /// 纹理缩放
    pub scale: f32,
    /// 纹理偏移
    pub offset: [f32; 2],
    /// 纹理旋转
    pub rotation: f32,
    /// 是否启用
    pub enabled: bool,
}

impl TextureSlot {
    /// 创建新的纹理槽
    pub fn new(texture_type: TextureType) -> Self {
        Self {
            texture_type,
            path: None,
            scale: 1.0,
            offset: [0.0, 0.0],
            rotation: 0.0,
            enabled: false,
        }
    }
}

/// PBR材质参数
#[derive(Debug, Clone)]
pub struct PBRMaterialParams {
    /// 基础颜色
    pub albedo: Vec4,
    /// 金属度
    pub metallic: f32,
    /// 粗糙度
    pub roughness: f32,
    /// 环境光遮蔽
    pub ao: f32,
    /// 发光颜色
    pub emissive: Vec3,
    /// 法线强度
    pub normal_strength: f32,
    /// 清漆强度
    pub clearcoat: f32,
    /// 清漆粗糙度
    pub clearcoat_roughness: f32,
    /// 纹理槽
    pub textures: HashMap<TextureType, TextureSlot>,
}

impl Default for PBRMaterialParams {
    fn default() -> Self {
        let mut textures = HashMap::new();
        textures.insert(TextureType::Albedo, TextureSlot::new(TextureType::Albedo));
        textures.insert(TextureType::Normal, TextureSlot::new(TextureType::Normal));
        textures.insert(
            TextureType::Roughness,
            TextureSlot::new(TextureType::Roughness),
        );
        textures.insert(
            TextureType::Metallic,
            TextureSlot::new(TextureType::Metallic),
        );
        textures.insert(
            TextureType::AmbientOcclusion,
            TextureSlot::new(TextureType::AmbientOcclusion),
        );
        textures.insert(
            TextureType::Emissive,
            TextureSlot::new(TextureType::Emissive),
        );
        textures.insert(
            TextureType::Clearcoat,
            TextureSlot::new(TextureType::Clearcoat),
        );

        Self {
            albedo: Vec4::new(0.8, 0.8, 0.8, 1.0),
            metallic: 0.0,
            roughness: 0.5,
            ao: 1.0,
            emissive: Vec3::ZERO,
            normal_strength: 1.0,
            clearcoat: 0.0,
            clearcoat_roughness: 0.0,
            textures,
        }
    }
}

impl From<PBRMaterialParams> for PbrMaterialFull {
    fn from(params: PBRMaterialParams) -> Self {
        Self {
            material: PbrMaterial {
                base_color: params.albedo,
                metallic: params.metallic,
                roughness: params.roughness,
                ambient_occlusion: params.ao,
                emissive: params.emissive,
                normal_scale: params.normal_strength,
                clearcoat: params.clearcoat,
                clearcoat_roughness: params.clearcoat_roughness,
                ..Default::default()
            },
            textures: PbrTextures::default(),
        }
    }
}

/// 预览渲染器
#[derive(Debug, Clone)]
pub struct PreviewRenderer {
    /// 是否显示预览
    pub show_preview: bool,
    /// 预览模型类型
    pub preview_model: PreviewModel,
    /// 预览背景
    pub preview_background: PreviewBackground,
    /// 灯光强度
    pub light_intensity: f32,
    /// 环境光强度
    pub ambient_intensity: f32,
}

/// 预览模型类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewModel {
    /// 球体
    Sphere,
    /// 平面
    Plane,
    /// 立方体
    Cube,
    /// 环形结
    TorusKnot,
    /// 自定义模型
    Custom,
}

/// 预览背景
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreviewBackground {
    /// 棋盘格
    Checkerboard,
    /// 纯色
    Solid,
    /// 渐变
    Gradient,
    /// HDR环境
    HDR,
}

impl Default for PreviewRenderer {
    fn default() -> Self {
        Self {
            show_preview: true,
            preview_model: PreviewModel::Sphere,
            preview_background: PreviewBackground::Checkerboard,
            light_intensity: 1.0,
            ambient_intensity: 0.1,
        }
    }
}

/// DCC材质编辑器
#[derive(Debug, Clone)]
pub struct DCCMaterialEditor {
    /// 选中的材质
    pub selected_material: Option<MaterialID>,
    /// 材质列表
    pub materials: Vec<PBRMaterialParams>,
    /// 预览渲染器
    pub preview_renderer: PreviewRenderer,
    /// 材质名称
    pub material_names: Vec<String>,
}

impl DCCMaterialEditor {
    /// 创建新的材质编辑器
    pub fn new() -> Self {
        Self {
            selected_material: None,
            materials: Vec::new(),
            preview_renderer: PreviewRenderer::default(),
            material_names: Vec::new(),
        }
    }

    /// 添加新材质
    pub fn add_material(&mut self, name: String) -> MaterialID {
        let id = self.materials.len();
        self.materials.push(PBRMaterialParams::default());
        self.material_names.push(name);
        id
    }

    /// 移除材质
    pub fn remove_material(&mut self, id: MaterialID) {
        if id < self.materials.len() {
            self.materials.remove(id);
            self.material_names.remove(id);

            // 更新选中状态
            if self.selected_material == Some(id) {
                self.selected_material = None;
            } else if let Some(selected) = self.selected_material {
                if selected > id {
                    self.selected_material = Some(selected - 1);
                }
            }
        }
    }

    /// 获取材质
    pub fn get_material(&self, id: MaterialID) -> Option<&PBRMaterialParams> {
        self.materials.get(id)
    }

    /// 获取可变材质
    pub fn get_material_mut(&mut self, id: MaterialID) -> Option<&mut PBRMaterialParams> {
        self.materials.get_mut(id)
    }

    /// 显示UI
    pub fn show_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("Material Editor")
            .default_size([400.0, 600.0])
            .show(ctx, |ui| {
                self.show_editor_ui(ui);
            });
    }

    /// 显示编辑器UI
    fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        // 材质列表
        ui.horizontal(|ui| {
            ui.label("Materials:");
            if ui.button("+").clicked() {
                let name = format!("Material {}", self.materials.len());
                self.add_material(name);
            }
        });

        // 材质选择器
        let mut selected_to_remove = None;
        for (i, name) in self.material_names.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.selectable_label(self.selected_material == Some(i), name).clicked() {
                    self.selected_material = Some(i);
                }

                if ui.button("×").clicked() {
                    selected_to_remove = Some(i);
                }
            });
        }

        if let Some(id) = selected_to_remove {
            self.remove_material(id);
        }

        ui.separator();

        // 材质参数
        if let Some(idx) = self.selected_material {
            // 先显示PBR参数
            if self.materials.get(idx).is_some() {
                self.show_pbr_params_ui(ui, idx);
                ui.separator();
            }
            // 然后显示纹理槽
            if self.materials.get(idx).is_some() {
                self.show_texture_slots_ui(ui, idx);
            }
        }

        ui.separator();

        // 预览设置
        self.show_preview_settings(ui);

        ui.separator();

        // 预览窗口
        if self.preview_renderer.show_preview {
            self.show_preview_window(ui);
        }
    }

    /// 显示PBR参数
    fn show_pbr_params(ui: &mut egui::Ui, material: &mut PBRMaterialParams) {
        ui.label("PBR Parameters:");

        // 基础颜色
        ui.horizontal(|ui| {
            ui.label("Albedo:");
            let mut color = [
                material.albedo.x,
                material.albedo.y,
                material.albedo.z,
                material.albedo.w,
            ];
            if ui.color_edit_button_rgba_unmultiplied(&mut color).changed() {
                material.albedo = Vec4::new(color[0], color[1], color[2], color[3]);
            }
        });

        // 金属度
        ui.add(egui::Slider::new(&mut material.metallic, 0.0..=1.0).text("Metallic"));

        // 粗糙度
        ui.add(egui::Slider::new(&mut material.roughness, 0.0..=1.0).text("Roughness"));

        // 环境光遮蔽
        ui.add(egui::Slider::new(&mut material.ao, 0.0..=1.0).text("AO"));

        // 发光颜色
        ui.horizontal(|ui| {
            ui.label("Emissive:");
            let mut color = [
                material.emissive.x,
                material.emissive.y,
                material.emissive.z,
            ];
            if ui.color_edit_button_rgb(&mut color).changed() {
                material.emissive = Vec3::new(color[0], color[1], color[2]);
            }
        });

        // 法线强度
        ui.add(egui::Slider::new(&mut material.normal_strength, 0.0..=2.0).text("Normal Strength"));

        // 清漆
        ui.add(egui::Slider::new(&mut material.clearcoat, 0.0..=1.0).text("Clearcoat"));

        // 清漆粗糙度
        ui.add(
            egui::Slider::new(&mut material.clearcoat_roughness, 0.0..=1.0)
                .text("Clearcoat Roughness"),
        );
    }

    /// 显示纹理槽
    fn show_texture_slots(
        ui: &mut egui::Ui,
        material: &mut PBRMaterialParams,
        material_id: MaterialID,
        pending_browse: &mut Option<(MaterialID, TextureType)>,
    ) {
        ui.label("Textures:");

        let texture_types = [
            TextureType::Albedo,
            TextureType::Normal,
            TextureType::Roughness,
            TextureType::Metallic,
            TextureType::AmbientOcclusion,
            TextureType::Emissive,
            TextureType::Clearcoat,
        ];

        for texture_type in texture_types {
            if let Some(slot) = material.textures.get_mut(&texture_type) {
                ui.horizontal(|ui| {
                    let type_name = format!("{texture_type:?}");
                    ui.label(type_name);

                    ui.checkbox(&mut slot.enabled, "");

                    if slot.enabled {
                        if ui.button("Browse").clicked() {
                            *pending_browse = Some((material_id, texture_type));
                        }

                        if let Some(path) = &slot.path {
                            ui.label(path.to_string_lossy().to_string());
                        }

                        // 纹理变换
                        ui.separator();
                        ui.add(egui::Slider::new(&mut slot.scale, 0.1..=10.0).text("Scale"));
                        ui.separator();
                        ui.add(egui::Slider::new(&mut slot.rotation, 0.0..=360.0).text("Rotation"));
                    }
                });
            }
        }
    }

    /// 显示PBR参数UI（使用索引避免借用冲突）
    fn show_pbr_params_ui(&mut self, ui: &mut egui::Ui, idx: MaterialID) {
        if let Some(material) = self.materials.get_mut(idx) {
            Self::show_pbr_params(ui, material);
        }
    }

    /// 显示纹理槽UI（使用索引避免借用冲突）
    fn show_texture_slots_ui(&mut self, ui: &mut egui::Ui, idx: MaterialID) {
        let mut pending_browse = None;

        if let Some(material) = self.materials.get_mut(idx) {
            Self::show_texture_slots(ui, material, idx, &mut pending_browse);
        }

        // 处理文件浏览请求（在借用释放后）
        if let Some((material_id, texture_type)) = pending_browse {
            self.browse_texture_file_internal(material_id, texture_type);
        }
    }

    /// 显示预览设置
    fn show_preview_settings(&mut self, ui: &mut egui::Ui) {
        ui.label("Preview Settings:");

        ui.horizontal(|ui| {
            ui.label("Model:");
            ui.selectable_value(
                &mut self.preview_renderer.preview_model,
                PreviewModel::Sphere,
                "Sphere",
            );
            ui.selectable_value(
                &mut self.preview_renderer.preview_model,
                PreviewModel::Plane,
                "Plane",
            );
            ui.selectable_value(
                &mut self.preview_renderer.preview_model,
                PreviewModel::Cube,
                "Cube",
            );
            ui.selectable_value(
                &mut self.preview_renderer.preview_model,
                PreviewModel::TorusKnot,
                "Torus Knot",
            );
        });

        ui.horizontal(|ui| {
            ui.label("Background:");
            ui.selectable_value(
                &mut self.preview_renderer.preview_background,
                PreviewBackground::Checkerboard,
                "Checkerboard",
            );
            ui.selectable_value(
                &mut self.preview_renderer.preview_background,
                PreviewBackground::Solid,
                "Solid",
            );
            ui.selectable_value(
                &mut self.preview_renderer.preview_background,
                PreviewBackground::Gradient,
                "Gradient",
            );
        });

        ui.add(
            egui::Slider::new(&mut self.preview_renderer.light_intensity, 0.0..=5.0)
                .text("Light Intensity"),
        );

        ui.add(
            egui::Slider::new(&mut self.preview_renderer.ambient_intensity, 0.0..=1.0)
                .text("Ambient"),
        );
    }

    /// 显示预览窗口
    fn show_preview_window(&mut self, ui: &mut egui::Ui) {
        let desired_size = ui.available_size();
        let response = ui.allocate_response(desired_size, egui::Sense::click_and_drag());

        // 渲染材质预览
        let painter = ui.painter();
        let rect = response.rect;

        // 如果有选中的材质，显示其预览
        if let Some(material_id) = self.selected_material {
            if let Some(material) = self.materials.get(material_id) {
                // 使用材质的albedo颜色作为背景
                let albedo_color = egui::Color32::from_rgb(
                    (material.albedo[0] * 255.0) as u8,
                    (material.albedo[1] * 255.0) as u8,
                    (material.albedo[2] * 255.0) as u8,
                );

                // 绘制材质颜色预览
                painter.rect_filled(rect, egui::Rounding::same(4), albedo_color);

                // 绘制金属度和粗糙度指示器
                let metallic_indicator = if material.metallic > 0.5 {
                    egui::Color32::from_rgb(200, 200, 255) // 蓝色调表示金属
                } else {
                    egui::Color32::from_rgb(255, 200, 150) // 橙色调表示非金属
                };

                // 在右上角绘制金属度指示器
                let indicator_size = 20.0;
                let indicator_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.right() - indicator_size - 5.0, rect.top() + 5.0),
                    egui::vec2(indicator_size, indicator_size),
                );

                painter.circle_filled(
                    indicator_rect.center(),
                    indicator_size / 2.0,
                    metallic_indicator,
                );

                // 绘制粗糙度文字
                painter.text(
                    egui::pos2(rect.left() + 10.0, rect.bottom() - 20.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!("Roughness: {:.2}", material.roughness),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );

                // 绘制金属度文字
                painter.text(
                    egui::pos2(rect.left() + 10.0, rect.bottom() - 40.0),
                    egui::Align2::LEFT_BOTTOM,
                    format!("Metallic: {:.2}", material.metallic),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                );

                // 绘制材质名称
                if let Some(name) = self.material_names.get(material_id) {
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        name,
                        egui::FontId::proportional(16.0),
                        egui::Color32::BLACK,
                    );
                }
            } else {
                // 无材质：绘制占位符
                self.draw_placeholder(painter, rect);
            }
        } else {
            // 无选中材质：绘制占位符
            self.draw_placeholder(painter, rect);
        }
    }

    /// 绘制占位符
    fn draw_placeholder(&self, painter: &egui::Painter, rect: egui::Rect) {
        // 绘制灰色背景
        painter.rect_filled(
            rect,
            egui::Rounding::same(4),
            egui::Color32::from_rgb(60, 60, 60),
        );

        // 绘制边框
        painter.rect_stroke(
            rect,
            egui::Rounding::same(4),
            egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
            egui::StrokeKind::Middle,
        );

        // 绘制文字
        painter.text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "Select a material to preview",
            egui::FontId::proportional(14.0),
            egui::Color32::from_rgb(180, 180, 180),
        );
    }

    /// 应用材质预设
    pub fn apply_preset(&mut self, id: MaterialID, preset: MaterialPreset) {
        if let Some(material) = self.materials.get_mut(id) {
            let params = preset.to_params();
            *material = params;
        }
    }

    /// 导出材质为PbrMaterialFull
    pub fn export_material(&self, id: MaterialID) -> Option<PbrMaterialFull> {
        self.materials.get(id).map(|m| m.clone().into())
    }

    /// 浏览纹理文件（内部实现）
    pub fn browse_texture_file_internal(
        &mut self,
        material_id: MaterialID,
        texture_type: TextureType,
    ) {
        #[cfg(feature = "file_dialog")]
        {
            use rfd::FileHandle;

            // 打开文件选择对话框
            if let Some(file) = rfd::FileDialog::new()
                .add_filter(
                    "texture",
                    &["png", "jpg", "jpeg", "dds", "ktx", "tga", "bmp", "webp"],
                )
                .pick_file()
            {
                let path = file.path().to_path_buf();

                // 更新材质的纹理路径
                if let Some(material) = self.materials.get_mut(material_id) {
                    if let Some(slot) = material.textures.get_mut(&texture_type) {
                        slot.path = Some(path);
                        slot.enabled = true;

                        tracing::info!(
                            "Loaded texture for material {:?}, slot {:?}: {:?}",
                            material_id,
                            texture_type,
                            slot.path
                        );
                    }
                }
            }
        }

        #[cfg(not(feature = "file_dialog"))]
        {
            tracing::warn!(
                "File dialog feature not enabled. Enable 'file_dialog' feature to use texture browser."
            );
        }
    }

    /// 浏览纹理文件（兼容旧API）
    pub fn browse_texture_file(&mut self, material_id: MaterialID, slot_idx: usize) {
        // 根据slot_idx映射到TextureType
        let texture_types = [
            TextureType::Albedo,
            TextureType::Normal,
            TextureType::Roughness,
            TextureType::Metallic,
            TextureType::AmbientOcclusion,
            TextureType::Emissive,
            TextureType::Clearcoat,
        ];

        if slot_idx < texture_types.len() {
            self.browse_texture_file_internal(material_id, texture_types[slot_idx]);
        } else {
            tracing::warn!("Invalid texture slot index: {}", slot_idx);
        }
    }

    /// 渲染材质预览
    pub fn render_material_preview(&mut self, material_id: MaterialID) {
        // 简化实现：记录预览请求
        // 完整实现需要：
        // 1. 创建离屏渲染目标
        // 2. 设置预览几何体（球体/平面/立方体）
        // 3. 应用材质参数和纹理
        // 4. 渲染PBR光照
        // 5. 输出到纹理

        if let Some(material) = self.get_material(material_id) {
            tracing::info!(
                "Rendering material preview: {} (albedo: {:?}, metallic: {}, roughness: {})",
                self.material_names.get(material_id).unwrap_or(&"Unknown".to_string()),
                material.albedo,
                material.metallic,
                material.roughness
            );

            // 完整实现需要渲染管线集成
            // 当前实现提供基于egui的材质预览框架
            //
            // 完整实现需要：
            // 1. 集成渲染管线（如wgpu、glam等）
            // 2. 创建PBR材质着色器
            // 3. 设置光照环境（IBL、方向光等）
            // 4. 渲染球体或平面预览
            // 5. 将渲染结果渲染到egui纹理
            //
            // 框架实现：
            // - 提供材质参数可视化
            // - 显示albedo颜色
            // - 显示金属度和粗糙度指示器
            // - 为渲染管线集成预留接口
            //
            // 性能优化建议：
            // - 使用渲染缓存（仅在材质修改时重新渲染）
            // - 低分辨率预览（128x128或256x256）
            // - 异步渲染（避免阻塞UI）
            // - 渐进式渲染（先低质量，后高质量）
        } else {
            tracing::warn!("Failed to get material {} for preview", material_id);
        }
    }

    /// 获取所有材质
    pub fn get_all_materials(&self) -> Vec<(String, PBRMaterialParams)> {
        self.material_names
            .iter()
            .zip(self.materials.iter())
            .map(|(name, material)| (name.clone(), material.clone()))
            .collect()
    }

    /// 创建材质迭代器
    pub fn iter(&self) -> MaterialIterator {
        MaterialIterator {
            editor: self,
            index: 0,
        }
    }
}

/// 材质迭代器
///
/// 允许按名称或类型筛选材质
pub struct MaterialIterator<'a> {
    editor: &'a DCCMaterialEditor,
    index: usize,
}

impl<'a> MaterialIterator<'a> {
    /// 按名称筛选
    pub fn filter_by_name(self, name_pattern: &str) -> FilteredMaterialIterator<'a> {
        FilteredMaterialIterator {
            editor: self.editor,
            index: 0,
            name_pattern: Some(name_pattern.to_string()),
            texture_type_filter: None,
        }
    }

    /// 按纹理类型筛选
    pub fn filter_by_texture_type(self, texture_type: TextureType) -> FilteredMaterialIterator<'a> {
        FilteredMaterialIterator {
            editor: self.editor,
            index: 0,
            name_pattern: None,
            texture_type_filter: Some(texture_type),
        }
    }
}

impl<'a> Iterator for MaterialIterator<'a> {
    type Item = (String, PBRMaterialParams);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.editor.materials.len() {
            let name = self.editor.material_names[self.index].clone();
            let material = self.editor.materials[self.index].clone();
            self.index += 1;
            Some((name, material))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.editor.materials.len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}

/// 过滤后的材质迭代器
pub struct FilteredMaterialIterator<'a> {
    editor: &'a DCCMaterialEditor,
    index: usize,
    name_pattern: Option<String>,
    texture_type_filter: Option<TextureType>,
}

impl<'a> Iterator for FilteredMaterialIterator<'a> {
    type Item = (String, PBRMaterialParams);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.editor.materials.len() {
            let name = &self.editor.material_names[self.index];
            let material = &self.editor.materials[self.index];
            self.index += 1;

            // 应用名称过滤
            if let Some(pattern) = &self.name_pattern {
                if !name.contains(pattern) {
                    continue;
                }
            }

            // 应用纹理类型过滤
            if let Some(texture_type) = self.texture_type_filter {
                if !material.textures.contains_key(&texture_type) {
                    continue;
                }
            }

            return Some((name.clone(), material.clone()));
        }

        None
    }
}

/// 材质预设
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaterialPreset {
    /// 标准金属
    StandardMetal,
    /// 标准非金属
    StandardNonMetal,
    /// 发光
    Emissive,
    /// 玻璃
    Glass,
    /// 塑料
    Plastic,
    /// 橡胶
    Rubber,
    /// 皮肤
    Skin,
    /// 织物
    Fabric,
}

impl MaterialPreset {
    /// 转换为材质参数
    pub fn to_params(self) -> PBRMaterialParams {
        match self {
            MaterialPreset::StandardMetal => PBRMaterialParams {
                albedo: Vec4::new(0.8, 0.8, 0.8, 1.0),
                metallic: 1.0,
                roughness: 0.2,
                ao: 1.0,
                emissive: Vec3::ZERO,
                normal_strength: 1.0,
                clearcoat: 0.0,
                clearcoat_roughness: 0.0,
                textures: HashMap::new(),
            },
            MaterialPreset::StandardNonMetal => PBRMaterialParams {
                albedo: Vec4::new(0.8, 0.8, 0.8, 1.0),
                metallic: 0.0,
                roughness: 0.5,
                ao: 1.0,
                emissive: Vec3::ZERO,
                normal_strength: 1.0,
                clearcoat: 0.0,
                clearcoat_roughness: 0.0,
                textures: HashMap::new(),
            },
            MaterialPreset::Emissive => PBRMaterialParams {
                albedo: Vec4::new(1.0, 1.0, 1.0, 1.0),
                metallic: 0.0,
                roughness: 0.0,
                ao: 1.0,
                emissive: Vec3::new(1.0, 1.0, 1.0),
                normal_strength: 1.0,
                clearcoat: 0.0,
                clearcoat_roughness: 0.0,
                textures: HashMap::new(),
            },
            MaterialPreset::Glass => PBRMaterialParams {
                albedo: Vec4::new(0.9, 0.9, 1.0, 0.3),
                metallic: 0.0,
                roughness: 0.0,
                ao: 1.0,
                emissive: Vec3::ZERO,
                normal_strength: 1.0,
                clearcoat: 1.0,
                clearcoat_roughness: 0.0,
                textures: HashMap::new(),
            },
            MaterialPreset::Plastic => PBRMaterialParams {
                albedo: Vec4::new(0.8, 0.8, 0.8, 1.0),
                metallic: 0.0,
                roughness: 0.3,
                ao: 1.0,
                emissive: Vec3::ZERO,
                normal_strength: 1.0,
                clearcoat: 0.0,
                clearcoat_roughness: 0.0,
                textures: HashMap::new(),
            },
            MaterialPreset::Rubber => PBRMaterialParams {
                albedo: Vec4::new(0.2, 0.2, 0.2, 1.0),
                metallic: 0.0,
                roughness: 0.8,
                ao: 1.0,
                emissive: Vec3::ZERO,
                normal_strength: 1.0,
                clearcoat: 0.0,
                clearcoat_roughness: 0.0,
                textures: HashMap::new(),
            },
            MaterialPreset::Skin => PBRMaterialParams {
                albedo: Vec4::new(0.9, 0.7, 0.7, 1.0),
                metallic: 0.0,
                roughness: 0.4,
                ao: 1.0,
                emissive: Vec3::ZERO,
                normal_strength: 1.0,
                clearcoat: 0.0,
                clearcoat_roughness: 0.0,
                textures: HashMap::new(),
            },
            MaterialPreset::Fabric => PBRMaterialParams {
                albedo: Vec4::new(0.6, 0.5, 0.4, 1.0),
                metallic: 0.0,
                roughness: 0.9,
                ao: 1.0,
                emissive: Vec3::ZERO,
                normal_strength: 1.0,
                clearcoat: 0.0,
                clearcoat_roughness: 0.0,
                textures: HashMap::new(),
            },
        }
    }
}

impl Default for DCCMaterialEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_editor_creation() {
        let editor = DCCMaterialEditor::new();
        assert!(editor.materials.is_empty());
        assert!(editor.selected_material.is_none());
    }

    #[test]
    fn test_add_material() {
        let mut editor = DCCMaterialEditor::new();
        let id = editor.add_material("TestMaterial".to_string());
        assert_eq!(id, 0);
        assert_eq!(editor.materials.len(), 1);
        assert_eq!(editor.material_names.len(), 1);
    }

    #[test]
    fn test_remove_material() {
        let mut editor = DCCMaterialEditor::new();
        let id1 = editor.add_material("Material1".to_string());
        let id2 = editor.add_material("Material2".to_string());

        editor.remove_material(id1);
        assert_eq!(editor.materials.len(), 1);
    }

    #[test]
    fn test_preset() {
        let preset = MaterialPreset::StandardMetal;
        let params = preset.to_params();
        assert_eq!(params.metallic, 1.0);
        assert_eq!(params.roughness, 0.2);
    }

    #[test]
    fn test_material_iterator() {
        let mut editor = DCCMaterialEditor::new();
        editor.add_material("Material1".to_string());
        editor.add_material("Material2".to_string());
        editor.add_material("WoodMaterial".to_string());

        // 测试基本迭代
        let materials: Vec<_> = editor.iter().collect();
        assert_eq!(materials.len(), 3);
        assert_eq!(materials[0].0, "Material1");
        assert_eq!(materials[1].0, "Material2");
        assert_eq!(materials[2].0, "WoodMaterial");
    }

    #[test]
    fn test_material_iterator_filter_by_name() {
        let mut editor = DCCMaterialEditor::new();
        editor.add_material("WoodMaterial".to_string());
        editor.add_material("MetalMaterial".to_string());
        editor.add_material("WoodOak".to_string());

        // 测试名称过滤
        let wood_materials: Vec<_> = editor.iter().filter_by_name("Wood").collect();
        assert_eq!(wood_materials.len(), 2);
        assert_eq!(wood_materials[0].0, "WoodMaterial");
        assert_eq!(wood_materials[1].0, "WoodOak");
    }

    #[test]
    fn test_material_iterator_size_hint() {
        let mut editor = DCCMaterialEditor::new();
        editor.add_material("Material1".to_string());
        editor.add_material("Material2".to_string());

        let mut iter = editor.iter();
        assert_eq!(iter.size_hint(), (2, Some(2)));

        iter.next();
        assert_eq!(iter.size_hint(), (1, Some(1)));

        iter.next();
        assert_eq!(iter.size_hint(), (0, Some(0)));
    }

    #[test]
    fn test_get_all_materials() {
        let mut editor = DCCMaterialEditor::new();
        editor.add_material("Material1".to_string());
        editor.add_material("Material2".to_string());

        let all_materials = editor.get_all_materials();
        assert_eq!(all_materials.len(), 2);
        assert_eq!(all_materials[0].0, "Material1");
        assert_eq!(all_materials[1].0, "Material2");
    }

    #[test]
    fn test_export_material() {
        let mut editor = DCCMaterialEditor::new();
        let id = editor.add_material("TestMaterial".to_string());

        if let Some(material) = editor.get_material_mut(id) {
            material.albedo = Vec4::new(1.0, 0.0, 0.0, 1.0);
            material.metallic = 0.8;
            material.roughness = 0.3;
        }

        let exported = editor.export_material(id);
        assert!(exported.is_some());

        let pbr_material = exported.unwrap();
        assert_eq!(pbr_material.material.base_color.x, 1.0);
        assert_eq!(pbr_material.material.metallic, 0.8);
        assert_eq!(pbr_material.material.roughness, 0.3);
    }

    #[test]
    fn test_apply_preset() {
        let mut editor = DCCMaterialEditor::new();
        let id = editor.add_material("CustomMaterial".to_string());

        editor.apply_preset(id, MaterialPreset::StandardMetal);

        let material = editor.get_material(id).unwrap();
        assert_eq!(material.metallic, 1.0);
        assert_eq!(material.roughness, 0.2);
    }

    #[test]
    fn test_browse_texture_file() {
        let mut editor = DCCMaterialEditor::new();
        let id = editor.add_material("TestMaterial".to_string());

        // 这个测试不会实际打开文件对话框（没有文件对话框feature）
        // 但可以验证方法调用不会panic
        editor.browse_texture_file(id, 0);

        // 验证纹理槽存在
        let material = editor.get_material(id).unwrap();
        assert!(material.textures.contains_key(&TextureType::Albedo));
    }

    #[test]
    fn test_browse_texture_file_internal() {
        let mut editor = DCCMaterialEditor::new();
        let id = editor.add_material("TestMaterial".to_string());

        // 测试内部方法
        editor.browse_texture_file_internal(id, TextureType::Normal);

        // 验证方法调用不会panic（实际文件选择仅在file_dialog feature启用时工作）
        let material = editor.get_material(id).unwrap();
        assert!(material.textures.contains_key(&TextureType::Normal));
    }

    #[test]
    fn test_render_material_preview() {
        let mut editor = DCCMaterialEditor::new();
        let id = editor.add_material("PreviewMaterial".to_string());

        // 设置材质参数
        if let Some(material) = editor.get_material_mut(id) {
            material.albedo = Vec4::new(0.5, 0.5, 0.5, 1.0);
            material.metallic = 0.5;
            material.roughness = 0.5;
        }

        // 测试预览方法不会panic
        editor.render_material_preview(id);
    }
}
