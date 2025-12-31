//! # UV编辑器
//!
//! 提供UV编辑功能，包括：
//! - UV岛显示和选择
//! - UV变换（移动、旋转、缩放）
//! - 网格吸附
//! - UV展开工具

use egui::*;
use glam::{Vec2, Vec3};
use std::collections::HashSet;

/// UV ID类型
pub type UVID = usize;

/// UV岛
#[derive(Debug, Clone)]
pub struct UVIsland {
    /// UV坐标
    pub uvs: Vec<Vec2>,
    /// 三角形索引
    pub triangles: Vec<[usize; 3]>,
    /// 岛的包围盒
    pub bounds: (Vec2, Vec2),
    /// 是否选中
    pub selected: bool,
}

impl UVIsland {
    /// 计算包围盒
    pub fn calculate_bounds(&self) -> (Vec2, Vec2) {
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);

        for uv in &self.uvs {
            min = min.min(*uv);
            max = max.max(*uv);
        }

        (min, max)
    }

    /// 获取中心点
    pub fn get_center(&self) -> Vec2 {
        let (min, max) = self.bounds;
        (min + max) / 2.0
    }
}

/// UV变换
#[derive(Debug, Clone)]
pub struct UVTransform {
    /// 平移
    pub translation: Vec2,
    /// 旋转（弧度）
    pub rotation: f32,
    /// 缩放
    pub scale: Vec2,
}

impl Default for UVTransform {
    fn default() -> Self {
        Self {
            translation: Vec2::ZERO,
            rotation: 0.0,
            scale: Vec2::ONE,
        }
    }
}

/// 吸附设置
#[derive(Debug, Clone)]
pub struct SnapSettings {
    /// 是否启用吸附
    pub enabled: bool,
    /// 吸附到网格
    pub snap_to_grid: bool,
    /// 网格大小
    pub grid_size: f32,
    /// 吸附到其他UV
    pub snap_to_uvs: bool,
    /// 吸附阈值
    pub snap_threshold: f32,
}

impl Default for SnapSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            snap_to_grid: true,
            grid_size: 0.0625, // 1/16
            snap_to_uvs: false,
            snap_threshold: 0.01,
        }
    }
}

/// UV编辑器
#[derive(Debug, Clone)]
pub struct UVEditor {
    /// 选中的UV
    pub selected_uvs: HashSet<UVID>,
    /// UV岛
    pub uv_islands: Vec<UVIsland>,
    /// 显示网格
    pub show_grid: bool,
    /// 网格颜色
    pub grid_color: egui::Color32,
    /// 背景颜色
    pub background_color: egui::Color32,
    /// UV变换
    pub transform: UVTransform,
    /// 吸附设置
    pub snap_settings: SnapSettings,
    /// 显示棋盘格背景
    pub show_checkerboard: bool,
    /// 变换模式
    pub transform_mode: TransformMode,
}

/// 变换模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformMode {
    /// 移动
    Translate,
    /// 旋转
    Rotate,
    /// 缩放
    Scale,
}

impl Default for UVEditor {
    fn default() -> Self {
        Self {
            selected_uvs: HashSet::new(),
            uv_islands: Vec::new(),
            show_grid: true,
            grid_color: egui::Color32::from_rgb(80, 80, 80),
            background_color: egui::Color32::from_rgb(40, 40, 40),
            transform: UVTransform::default(),
            snap_settings: SnapSettings::default(),
            show_checkerboard: true,
            transform_mode: TransformMode::Translate,
        }
    }
}

impl UVEditor {
    /// 创建新的UV编辑器
    pub fn new() -> Self {
        Self::default()
    }

    /// 加载UV数据
    pub fn load_uvs(&mut self, uvs: Vec<Vec2>, triangles: Vec<[usize; 3]>) {
        // TODO: 实现UV岛检测算法
        let island = UVIsland {
            uvs: uvs.clone(),
            triangles,
            bounds: (Vec2::ZERO, Vec2::ONE),
            selected: false,
        };

        self.uv_islands = vec![island];
        self.clear_selection();
    }

    /// 清除选择
    pub fn clear_selection(&mut self) {
        self.selected_uvs.clear();
    }

    /// 显示UI

    pub fn show_ui(&mut self, ctx: &egui::Context) {
        egui::Window::new("UV Editor").default_size([600.0, 500.0]).show(ctx, |ui| {
            self.show_editor_ui(ui);
        });
    }

    /// 显示编辑器UI

    fn show_editor_ui(&mut self, ui: &mut egui::Ui) {
        // 工具栏
        ui.horizontal(|ui| {
            ui.label("Transform:");
            ui.selectable_value(&mut self.transform_mode, TransformMode::Translate, "Move");
            ui.selectable_value(&mut self.transform_mode, TransformMode::Rotate, "Rotate");
            ui.selectable_value(&mut self.transform_mode, TransformMode::Scale, "Scale");
        });

        ui.separator();

        // 视图设置
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.show_checkerboard, "Checkerboard");
        });

        ui.separator();

        // 吸附设置
        ui.label("Snap Settings:");
        ui.checkbox(&mut self.snap_settings.enabled, "Enable Snapping");
        ui.checkbox(&mut self.snap_settings.snap_to_grid, "Snap to Grid");
        if self.snap_settings.snap_to_grid {
            ui.add(
                egui::Slider::new(&mut self.snap_settings.grid_size, 0.01..=0.5)
                    .text("Grid Size")
                    .logarithmic(),
            );
        }

        ui.separator();

        // 变换值
        ui.label("Transform:");
        ui.horizontal(|ui| {
            ui.label("Position:");
            ui.add(egui::DragValue::new(&mut self.transform.translation.x).speed(0.01));
            ui.add(egui::DragValue::new(&mut self.transform.translation.y).speed(0.01));
        });

        ui.horizontal(|ui| {
            ui.label("Scale:");
            ui.add(egui::DragValue::new(&mut self.transform.scale.x).speed(0.01));
            ui.add(egui::DragValue::new(&mut self.transform.scale.y).speed(0.01));
        });

        ui.add(egui::Slider::new(&mut self.transform.rotation, 0.0..=360.0).text("Rotation"));

        ui.separator();

        // 统计信息
        ui.label(format!("UV Islands: {}", self.uv_islands.len()));
        ui.label(format!("Selected UVs: {}", self.selected_uvs.len()));

        ui.separator();

        // 2D UV视图
        self.show_uv_view(ui);
    }

    /// 显示UV视图

    fn show_uv_view(&mut self, ui: &mut egui::Ui) {
        let available_size = ui.available_size();
        let response = ui.allocate_response(available_size, egui::Sense::click_and_drag());

        let painter = ui.painter();
        let rect = response.rect;

        // 绘制背景
        if self.show_checkerboard {
            self.draw_checkerboard(painter, rect);
        } else {
            painter.rect_filled(rect, egui::Rounding::none(), self.background_color);
        }

        // 绘制网格
        if self.show_grid {
            self.draw_uv_grid(painter, rect);
        }

        // 绘制UV岛
        for island in &self.uv_islands {
            self.draw_uv_island(painter, rect, island);
        }

        // 绘制选中的UV
        for &uv_id in &self.selected_uvs {
            // 查找UV坐标
            for island in &self.uv_islands {
                if uv_id < island.uvs.len() {
                    let uv = &island.uvs[uv_id];
                    let pos = self.uv_to_screen(uv, rect);
                    painter.circle_filled(pos, 5.0, egui::Color32::GREEN);
                    break;
                }
            }
        }

        // 绘制边界（0-1范围）
        self.draw_uv_bounds(painter, rect);
    }

    /// 绘制棋盘格背景

    fn draw_checkerboard(&self, painter: &egui::Painter, rect: egui::Rect) {
        let checker_size = 20.0;
        let mut white = true;

        for y in 0..=(rect.height() as usize / checker_size as usize) {
            for x in 0..=(rect.width() as usize / checker_size as usize) {
                let x0 = rect.left() + (x as f32 * checker_size);
                let y0 = rect.top() + (y as f32 * checker_size);
                let x1 = (x0 + checker_size).min(rect.right());
                let y1 = (y0 + checker_size).min(rect.bottom());

                let color = if white {
                    egui::Color32::from_rgb(80, 80, 80)
                } else {
                    egui::Color32::from_rgb(60, 60, 60)
                };

                painter.rect_filled(
                    egui::Rect::from_min_max(egui::pos2(x0, y0), egui::pos2(x1, y1)),
                    egui::Rounding::none(),
                    color,
                );

                white = !white;
            }
            white = !white;
        }
    }

    /// 绘制UV网格

    fn draw_uv_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let grid_steps = 8;
        let step_x = rect.width() / grid_steps as f32;
        let step_y = rect.height() / grid_steps as f32;

        // 垂直线
        for i in 0..=grid_steps {
            let x = rect.left() + (i as f32 * step_x);
            painter.line(
                egui::pos2(x, rect.top()),
                egui::pos2(x, rect.bottom()),
                (1.0, self.grid_color),
            );
        }

        // 水平线
        for i in 0..=grid_steps {
            let y = rect.top() + (i as f32 * step_y);
            painter.line(
                egui::pos2(rect.left(), y),
                egui::pos2(rect.right(), y),
                (1.0, self.grid_color),
            );
        }

        // 绘制0.5线（更亮）
        let center_x = rect.left() + rect.width() / 2.0;
        let center_y = rect.top() + rect.height() / 2.0;

        painter.line(
            egui::pos2(center_x, rect.top()),
            egui::pos2(center_x, rect.bottom()),
            (2.0, egui::Color32::from_rgb(120, 120, 120)),
        );

        painter.line(
            egui::pos2(rect.left(), center_y),
            egui::pos2(rect.right(), center_y),
            (2.0, egui::Color32::from_rgb(120, 120, 120)),
        );
    }

    /// 绘制UV岛

    fn draw_uv_island(&self, painter: &egui::Painter, rect: egui::Rect, island: &UVIsland) {
        let color = if island.selected {
            egui::Color32::YELLOW
        } else {
            egui::Color32::BLUE
        };

        // 绘制三角形
        for triangle in &island.triangles {
            if triangle[0] < island.uvs.len()
                && triangle[1] < island.uvs.len()
                && triangle[2] < island.uvs.len()
            {
                let uv0 = self.uv_to_screen(&island.uvs[triangle[0]], rect);
                let uv1 = self.uv_to_screen(&island.uvs[triangle[1]], rect);
                let uv2 = self.uv_to_screen(&island.uvs[triangle[2]], rect);

                // 填充三角形
                painter.add(egui::epaint::PathShape::convex_polygon(
                    vec![uv0, uv1, uv2],
                    egui::Color32::from_rgba_unmultiplied(100, 150, 255, 50),
                    egui::Stroke::new(1.0, color),
                ));
            }
        }

        // 绘制包围盒
        let (min, max) = island.bounds;
        let min_screen = self.uv_to_screen(&min, rect);
        let max_screen = self.uv_to_screen(&max, rect);

        painter.rect_stroke(
            egui::Rect::from_min_max(min_screen, max_screen),
            egui::Rounding::none(),
            egui::Stroke::new(1.0, egui::Color32::GREEN),
        );
    }

    /// 绘制UV边界（0-1范围）

    fn draw_uv_bounds(&self, painter: &egui::Painter, rect: egui::Rect) {
        let padding = 2.0;
        let bounds = egui::Rect::from_min_max(
            egui::pos2(rect.left() + padding, rect.top() + padding),
            egui::pos2(rect.right() - padding, rect.bottom() - padding),
        );

        painter.rect_stroke(
            bounds,
            egui::Rounding::none(),
            egui::Stroke::new(2.0, egui::Color32::RED),
        );
    }

    /// UV坐标转屏幕坐标
    fn uv_to_screen(&self, uv: &Vec2, rect: egui::Rect) -> egui::Pos2 {
        let padding = 2.0;
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left() + padding, rect.top() + padding),
            egui::pos2(rect.right() - padding, rect.bottom() - padding),
        );

        egui::pos2(
            content_rect.left() + uv.x * content_rect.width(),
            content_rect.top() + uv.y * content_rect.height(),
        )
    }

    /// 屏幕坐标转UV坐标
    fn screen_to_uv(&self, pos: egui::Pos2, rect: egui::Rect) -> Vec2 {
        let padding = 2.0;
        let content_rect = egui::Rect::from_min_max(
            egui::pos2(rect.left() + padding, rect.top() + padding),
            egui::pos2(rect.right() - padding, rect.bottom() - padding),
        );

        Vec2::new(
            (pos.x - content_rect.left()) / content_rect.width(),
            (pos.y - content_rect.top()) / content_rect.height(),
        )
    }

    /// 应用UV变换
    pub fn apply_transform(&mut self) {
        if !self.selected_uvs.is_empty() {
            for island in &mut self.uv_islands {
                for &uv_id in &self.selected_uvs {
                    if uv_id < island.uvs.len() {
                        let uv = &mut island.uvs[uv_id];

                        // 应用缩放
                        *uv *= self.transform.scale;

                        // 应用旋转
                        if self.transform.rotation != 0.0 {
                            let angle = self.transform.rotation.to_radians();
                            let cos = angle.cos();
                            let sin = angle.sin();
                            let center = Vec2::new(0.5, 0.5);

                            let centered = *uv - center;
                            let rotated = Vec2::new(
                                centered.x * cos - centered.y * sin,
                                centered.x * sin + centered.y * cos,
                            );
                            *uv = rotated + center;
                        }

                        // 应用平移
                        *uv += self.transform.translation;

                        // 吸附
                        if self.snap_settings.enabled && self.snap_settings.snap_to_grid {
                            uv.x = (uv.x / self.snap_settings.grid_size).round()
                                * self.snap_settings.grid_size;
                            uv.y = (uv.y / self.snap_settings.grid_size).round()
                                * self.snap_settings.grid_size;
                        }

                        // 限制在0-1范围内
                        uv.x = uv.x.clamp(0.0, 1.0);
                        uv.y = uv.y.clamp(0.0, 1.0);
                    }
                }
            }

            // 重置变换
            self.transform = UVTransform::default();
        }
    }

    /// 展开UV
    pub fn unwrap_uvs(&mut self) {
        // TODO: 实现UV展开算法（如LSCM、ABF等）
    }

    /// 松弛UV
    pub fn relax_uvs(&mut self) {
        // TODO: 实现UV松弛算法
    }

    /// 打包UV岛
    pub fn pack_uv_islands(&mut self) {
        // TODO: 实现UV岛打包算法
    }

    /// 获取UV坐标
    pub fn get_uv(&self, uv_id: UVID) -> Option<Vec2> {
        for island in &self.uv_islands {
            if uv_id < island.uvs.len() {
                return Some(island.uvs[uv_id]);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uv_editor_creation() {
        let editor = UVEditor::new();
        assert!(editor.uv_islands.is_empty());
        assert!(editor.selected_uvs.is_empty());
        assert_eq!(editor.transform_mode, TransformMode::Translate);
    }

    #[test]
    fn test_load_uvs() {
        let mut editor = UVEditor::new();
        let uvs = vec![
            Vec2::new(0.0, 0.0),
            Vec2::new(1.0, 0.0),
            Vec2::new(0.0, 1.0),
        ];
        let triangles = vec![[0, 1, 2]];

        editor.load_uvs(uvs, triangles);
        assert_eq!(editor.uv_islands.len(), 1);
        assert_eq!(editor.uv_islands[0].uvs.len(), 3);
    }

    #[test]
    fn test_uv_to_screen() {
        let editor = UVEditor::new();
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(100.0, 100.0));

        let uv = Vec2::new(0.5, 0.5);
        let screen = editor.uv_to_screen(&uv, rect);

        assert!((screen.x - 50.0).abs() < 1.0);
        assert!((screen.y - 50.0).abs() < 1.0);
    }

    #[test]
    fn test_selection() {
        let mut editor = UVEditor::new();
        editor.selected_uvs.insert(0);
        editor.selected_uvs.insert(1);

        assert_eq!(editor.selected_uvs.len(), 2);

        editor.clear_selection();
        assert!(editor.selected_uvs.is_empty());
    }
}
