use bevy_ecs::prelude::*;
use crate::ecs::Transform;
use glam::{Vec3, Quat};

/// 变换工具模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    Translate, // 移动
    Rotate,    // 旋转
    Scale,     // 缩放
}

impl GizmoMode {
    pub fn name(&self) -> &'static str {
        match self {
            GizmoMode::Translate => "Translate",
            GizmoMode::Rotate => "Rotate",
            GizmoMode::Scale => "Scale",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            GizmoMode::Translate => "↔",
            GizmoMode::Rotate => "↻",
            GizmoMode::Scale => "⇔",
        }
    }
}

/// 变换轴
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformAxis {
    X,
    Y,
    Z,
    All,
}

/// 变换工具
pub struct TransformGizmo {
    /// 当前模式
    pub mode: GizmoMode,
    /// 选中的轴
    pub selected_axis: Option<TransformAxis>,
    /// 拖拽起始位置
    drag_start: Option<egui::Pos2>,
    /// 拖拽起始值
    drag_start_value: Option<Vec3>,
}

impl TransformGizmo {
    pub fn new() -> Self {
        Self {
            mode: GizmoMode::Translate,
            selected_axis: None,
            drag_start: None,
            drag_start_value: None,
        }
    }
    
    /// 渲染变换工具UI
    pub fn render(&mut self, ui: &mut egui::Ui, world: &mut World, selected_entity: Option<Entity>) -> bool {
        let mut transform_changed = false;
        
        ui.heading("Transform Gizmo");
        ui.separator();
        
        // 模式选择
        ui.horizontal(|ui| {
            ui.label("Mode:");
            ui.selectable_value(&mut self.mode, GizmoMode::Translate, format!("{} Translate", GizmoMode::Translate.icon()));
            ui.selectable_value(&mut self.mode, GizmoMode::Rotate, format!("{} Rotate", GizmoMode::Rotate.icon()));
            ui.selectable_value(&mut self.mode, GizmoMode::Scale, format!("{} Scale", GizmoMode::Scale.icon()));
        });
        
        ui.separator();
        
        // 如果有选中的实体,显示变换控制
        if let Some(entity) = selected_entity {
            if let Some(mut transform) = world.get_mut::<Transform>(entity) {
                match self.mode {
                    GizmoMode::Translate => {
                        transform_changed |= self.render_translate_controls(ui, &mut transform);
                    }
                    GizmoMode::Rotate => {
                        transform_changed |= self.render_rotate_controls(ui, &mut transform);
                    }
                    GizmoMode::Scale => {
                        transform_changed |= self.render_scale_controls(ui, &mut transform);
                    }
                }
            } else {
                ui.label("Selected entity has no Transform component");
            }
        } else {
            ui.label("No entity selected");
        }
        
        transform_changed
    }
    
    /// 渲染移动控制
    fn render_translate_controls(&mut self, ui: &mut egui::Ui, transform: &mut Transform) -> bool {
        let mut changed = false;
        
        ui.label("Position:");
        
        ui.horizontal(|ui| {
            ui.label("X:");
            changed |= ui.add(egui::DragValue::new(&mut transform.pos.x).speed(0.1)).changed();
        });
        
        ui.horizontal(|ui| {
            ui.label("Y:");
            changed |= ui.add(egui::DragValue::new(&mut transform.pos.y).speed(0.1)).changed();
        });
        
        ui.horizontal(|ui| {
            ui.label("Z:");
            changed |= ui.add(egui::DragValue::new(&mut transform.pos.z).speed(0.1)).changed();
        });
        
        ui.separator();
        
        // 快捷按钮
        ui.horizontal(|ui| {
            if ui.button("Reset Position").clicked() {
                transform.pos = Vec3::ZERO;
                changed = true;
            }
        });
        
        changed
    }
    
    /// 渲染旋转控制
    fn render_rotate_controls(&mut self, ui: &mut egui::Ui, transform: &mut Transform) -> bool {
        let mut changed = false;
        
        ui.label("Rotation (Euler Angles):");
        
        // 转换为欧拉角
        let (mut x, mut y, mut z) = transform.rot.to_euler(glam::EulerRot::XYZ);
        x = x.to_degrees();
        y = y.to_degrees();
        z = z.to_degrees();
        
        ui.horizontal(|ui| {
            ui.label("X:");
            if ui.add(egui::DragValue::new(&mut x).speed(1.0).suffix("°")).changed() {
                changed = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Y:");
            if ui.add(egui::DragValue::new(&mut y).speed(1.0).suffix("°")).changed() {
                changed = true;
            }
        });
        
        ui.horizontal(|ui| {
            ui.label("Z:");
            if ui.add(egui::DragValue::new(&mut z).speed(1.0).suffix("°")).changed() {
                changed = true;
            }
        });
        
        if changed {
            transform.rot = Quat::from_euler(
                glam::EulerRot::XYZ,
                x.to_radians(),
                y.to_radians(),
                z.to_radians(),
            );
        }
        
        ui.separator();
        
        // 快捷按钮
        ui.horizontal(|ui| {
            if ui.button("Reset Rotation").clicked() {
                transform.rot = Quat::IDENTITY;
                changed = true;
            }
        });
        
        changed
    }
    
    /// 渲染缩放控制
    fn render_scale_controls(&mut self, ui: &mut egui::Ui, transform: &mut Transform) -> bool {
        let mut changed = false;
        
        ui.label("Scale:");
        
        ui.horizontal(|ui| {
            ui.label("X:");
            changed |= ui.add(egui::DragValue::new(&mut transform.scale.x).speed(0.01).clamp_range(0.01..=10.0)).changed();
        });
        
        ui.horizontal(|ui| {
            ui.label("Y:");
            changed |= ui.add(egui::DragValue::new(&mut transform.scale.y).speed(0.01).clamp_range(0.01..=10.0)).changed();
        });
        
        ui.horizontal(|ui| {
            ui.label("Z:");
            changed |= ui.add(egui::DragValue::new(&mut transform.scale.z).speed(0.01).clamp_range(0.01..=10.0)).changed();
        });
        
        ui.separator();
        
        // 统一缩放
        ui.horizontal(|ui| {
            ui.label("Uniform:");
            let mut uniform_scale = transform.scale.x;
            if ui.add(egui::DragValue::new(&mut uniform_scale).speed(0.01).clamp_range(0.01..=10.0)).changed() {
                transform.scale = Vec3::splat(uniform_scale);
                changed = true;
            }
        });
        
        ui.separator();
        
        // 快捷按钮
        ui.horizontal(|ui| {
            if ui.button("Reset Scale").clicked() {
                transform.scale = Vec3::ONE;
                changed = true;
            }
        });
        
        changed
    }
    
    /// 绘制3D变换工具 (在场景视图中)
    pub fn draw_3d_gizmo(&self, painter: &egui::Painter, screen_pos: egui::Pos2, zoom: f32) {
        let size = 50.0 * zoom;
        
        match self.mode {
            GizmoMode::Translate => {
                self.draw_translate_gizmo(painter, screen_pos, size);
            }
            GizmoMode::Rotate => {
                self.draw_rotate_gizmo(painter, screen_pos, size);
            }
            GizmoMode::Scale => {
                self.draw_scale_gizmo(painter, screen_pos, size);
            }
        }
    }
    
    /// 绘制移动工具
    fn draw_translate_gizmo(&self, painter: &egui::Painter, pos: egui::Pos2, size: f32) {
        // X轴 (红色)
        let x_color = if self.selected_axis == Some(TransformAxis::X) {
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            egui::Color32::from_rgb(255, 0, 0)
        };
        painter.arrow(pos, egui::vec2(size, 0.0), egui::Stroke::new(3.0, x_color));
        
        // Y轴 (绿色)
        let y_color = if self.selected_axis == Some(TransformAxis::Y) {
            egui::Color32::from_rgb(100, 255, 100)
        } else {
            egui::Color32::from_rgb(0, 255, 0)
        };
        painter.arrow(pos, egui::vec2(0.0, -size), egui::Stroke::new(3.0, y_color));
        
        // Z轴 (蓝色)
        let z_color = if self.selected_axis == Some(TransformAxis::Z) {
            egui::Color32::from_rgb(100, 100, 255)
        } else {
            egui::Color32::from_rgb(0, 0, 255)
        };
        painter.arrow(pos, egui::vec2(size * 0.7, size * 0.7), egui::Stroke::new(3.0, z_color));
    }
    
    /// 绘制旋转工具
    fn draw_rotate_gizmo(&self, painter: &egui::Painter, pos: egui::Pos2, size: f32) {
        // X轴旋转圆环 (红色)
        let x_color = if self.selected_axis == Some(TransformAxis::X) {
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            egui::Color32::from_rgb(255, 0, 0)
        };
        painter.circle_stroke(pos, size * 0.8, egui::Stroke::new(2.0, x_color));
        
        // Y轴旋转圆环 (绿色)
        let y_color = if self.selected_axis == Some(TransformAxis::Y) {
            egui::Color32::from_rgb(100, 255, 100)
        } else {
            egui::Color32::from_rgb(0, 255, 0)
        };
        painter.circle_stroke(pos, size * 0.6, egui::Stroke::new(2.0, y_color));
        
        // Z轴旋转圆环 (蓝色)
        let z_color = if self.selected_axis == Some(TransformAxis::Z) {
            egui::Color32::from_rgb(100, 100, 255)
        } else {
            egui::Color32::from_rgb(0, 0, 255)
        };
        painter.circle_stroke(pos, size * 0.4, egui::Stroke::new(2.0, z_color));
    }
    
    /// 绘制缩放工具
    fn draw_scale_gizmo(&self, painter: &egui::Painter, pos: egui::Pos2, size: f32) {
        // X轴 (红色)
        let x_color = if self.selected_axis == Some(TransformAxis::X) {
            egui::Color32::from_rgb(255, 100, 100)
        } else {
            egui::Color32::from_rgb(255, 0, 0)
        };
        painter.line_segment(
            [pos, egui::pos2(pos.x + size, pos.y)],
            egui::Stroke::new(3.0, x_color),
        );
        painter.circle_filled(egui::pos2(pos.x + size, pos.y), 5.0, x_color);
        
        // Y轴 (绿色)
        let y_color = if self.selected_axis == Some(TransformAxis::Y) {
            egui::Color32::from_rgb(100, 255, 100)
        } else {
            egui::Color32::from_rgb(0, 255, 0)
        };
        painter.line_segment(
            [pos, egui::pos2(pos.x, pos.y - size)],
            egui::Stroke::new(3.0, y_color),
        );
        painter.circle_filled(egui::pos2(pos.x, pos.y - size), 5.0, y_color);
        
        // Z轴 (蓝色)
        let z_color = if self.selected_axis == Some(TransformAxis::Z) {
            egui::Color32::from_rgb(100, 100, 255)
        } else {
            egui::Color32::from_rgb(0, 0, 255)
        };
        painter.line_segment(
            [pos, egui::pos2(pos.x + size * 0.7, pos.y + size * 0.7)],
            egui::Stroke::new(3.0, z_color),
        );
        painter.circle_filled(egui::pos2(pos.x + size * 0.7, pos.y + size * 0.7), 5.0, z_color);
    }
}

impl Default for TransformGizmo {
    fn default() -> Self {
        Self::new()
    }
}
