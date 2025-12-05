use crate::ecs::Transform;
use crate::impl_default;
use bevy_ecs::prelude::*;
use glam::Vec3;

/// 场景编辑器的视图模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Top,         // 俯视图
    Front,       // 正视图
    Side,        // 侧视图
    Perspective, // 透视图
}

impl ViewMode {
    pub fn name(&self) -> &'static str {
        match self {
            ViewMode::Top => "Top View",
            ViewMode::Front => "Front View",
            ViewMode::Side => "Side View",
            ViewMode::Perspective => "Perspective",
        }
    }
}

/// 场景编辑器
pub struct SceneEditor {
    /// 当前视图模式
    pub view_mode: ViewMode,
    /// 相机位置
    pub camera_position: Vec3,
    /// 相机旋转 (欧拉角,度)
    pub camera_rotation: Vec3,
    /// 视图缩放
    pub zoom: f32,
    /// 选中的实体
    pub selected_entity: Option<Entity>,
    /// 网格大小
    pub grid_size: f32,
    /// 是否显示网格
    pub show_grid: bool,
    /// 是否显示辅助线
    pub show_gizmos: bool,
}

impl SceneEditor {
    pub fn new() -> Self {
        Self::default()
    }

    /// 渲染场景编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui, world: &mut World) {
        ui.heading("Scene Editor");
        ui.separator();

        // 工具栏
        ui.horizontal(|ui| {
            ui.label("View:");
            ui.selectable_value(&mut self.view_mode, ViewMode::Top, "Top");
            ui.selectable_value(&mut self.view_mode, ViewMode::Front, "Front");
            ui.selectable_value(&mut self.view_mode, ViewMode::Side, "Side");
            ui.selectable_value(&mut self.view_mode, ViewMode::Perspective, "3D");

            ui.separator();

            ui.checkbox(&mut self.show_grid, "Grid");
            ui.checkbox(&mut self.show_gizmos, "Gizmos");
        });

        ui.separator();

        // 场景视图
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), 400.0),
            egui::Sense::click_and_drag(),
        );

        let rect = response.rect;

        // 绘制背景
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(40));

        // 绘制网格
        if self.show_grid {
            self.draw_grid(&painter, rect);
        }

        // 绘制实体
        self.draw_entities(&painter, rect, world);

        // 处理交互
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                // 检测点击的实体
                self.handle_click(pos, rect, world);
            }
        }

        // 处理相机移动
        if response.dragged() {
            let delta = response.drag_delta();
            self.move_camera(delta);
        }

        ui.separator();

        // 相机控制
        ui.collapsing("Camera", |ui| {
            ui.horizontal(|ui| {
                ui.label("Position:");
                ui.add(
                    egui::DragValue::new(&mut self.camera_position.x)
                        .prefix("X: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::new(&mut self.camera_position.y)
                        .prefix("Y: ")
                        .speed(0.1),
                );
                ui.add(
                    egui::DragValue::new(&mut self.camera_position.z)
                        .prefix("Z: ")
                        .speed(0.1),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Rotation:");
                ui.add(
                    egui::DragValue::new(&mut self.camera_rotation.x)
                        .prefix("X: ")
                        .speed(1.0)
                        .suffix("°"),
                );
                ui.add(
                    egui::DragValue::new(&mut self.camera_rotation.y)
                        .prefix("Y: ")
                        .speed(1.0)
                        .suffix("°"),
                );
                ui.add(
                    egui::DragValue::new(&mut self.camera_rotation.z)
                        .prefix("Z: ")
                        .speed(1.0)
                        .suffix("°"),
                );
            });

            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom, 0.1..=10.0));
            });
        });
    }

    /// 绘制网格
    fn draw_grid(&self, painter: &egui::Painter, rect: egui::Rect) {
        let grid_color = egui::Color32::from_gray(60);
        let grid_spacing = self.grid_size * 50.0 * self.zoom;

        // 垂直网格线
        let mut x = rect.left();
        while x < rect.right() {
            painter.line_segment(
                [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                egui::Stroke::new(1.0, grid_color),
            );
            x += grid_spacing;
        }

        // 水平网格线
        let mut y = rect.top();
        while y < rect.bottom() {
            painter.line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                egui::Stroke::new(1.0, grid_color),
            );
            y += grid_spacing;
        }

        // 绘制原点
        let center_x = rect.center().x;
        let center_y = rect.center().y;

        // X轴 (红色)
        painter.line_segment(
            [
                egui::pos2(center_x, center_y),
                egui::pos2(center_x + 50.0, center_y),
            ],
            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0)),
        );

        // Y轴 (绿色)
        painter.line_segment(
            [
                egui::pos2(center_x, center_y),
                egui::pos2(center_x, center_y - 50.0),
            ],
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
        );
    }

    /// 绘制实体
    fn draw_entities(&self, painter: &egui::Painter, rect: egui::Rect, world: &mut World) {
        let mut query = world.query::<(Entity, &Transform)>();

        for (entity, transform) in query.iter(world) {
            let screen_pos = self.world_to_screen(transform.pos, rect);

            let is_selected = self.selected_entity == Some(entity);
            let color = if is_selected {
                egui::Color32::from_rgb(255, 200, 0)
            } else {
                egui::Color32::from_rgb(100, 150, 255)
            };

            // 绘制实体为一个圆点
            painter.circle_filled(screen_pos, 5.0 * self.zoom, color);

            // 如果选中,绘制辅助线
            if is_selected && self.show_gizmos {
                self.draw_gizmo(painter, screen_pos);
            }
        }
    }

    /// 绘制辅助线 (Gizmo)
    fn draw_gizmo(&self, painter: &egui::Painter, pos: egui::Pos2) {
        let size = 30.0 * self.zoom;

        // X轴 (红色)
        painter.arrow(
            pos,
            egui::vec2(size, 0.0),
            egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 0, 0)),
        );

        // Y轴 (绿色)
        painter.arrow(
            pos,
            egui::vec2(0.0, -size),
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 255, 0)),
        );

        // Z轴 (蓝色) - 在2D视图中简化为对角线
        painter.arrow(
            pos,
            egui::vec2(size * 0.7, size * 0.7),
            egui::Stroke::new(2.0, egui::Color32::from_rgb(0, 0, 255)),
        );
    }

    /// 世界坐标转屏幕坐标
    pub fn world_to_screen(&self, world_pos: Vec3, rect: egui::Rect) -> egui::Pos2 {
        let center = rect.center();

        // 根据视图模式选择投影方式
        let (x, y) = match self.view_mode {
            ViewMode::Top => (world_pos.x, world_pos.z),
            ViewMode::Front => (world_pos.x, world_pos.y),
            ViewMode::Side => (world_pos.z, world_pos.y),
            ViewMode::Perspective => {
                // 简化的透视投影
                let relative_pos = world_pos - self.camera_position;
                (relative_pos.x, relative_pos.y)
            }
        };

        egui::pos2(
            center.x + x * 50.0 * self.zoom,
            center.y - y * 50.0 * self.zoom,
        )
    }

    /// 屏幕坐标转世界坐标
    fn screen_to_world(&self, screen_pos: egui::Pos2, rect: egui::Rect) -> Vec3 {
        let center = rect.center();

        let x = (screen_pos.x - center.x) / (50.0 * self.zoom);
        let y = -(screen_pos.y - center.y) / (50.0 * self.zoom);

        match self.view_mode {
            ViewMode::Top => Vec3::new(x, 0.0, y),
            ViewMode::Front => Vec3::new(x, y, 0.0),
            ViewMode::Side => Vec3::new(0.0, y, x),
            ViewMode::Perspective => self.camera_position + Vec3::new(x, y, 0.0),
        }
    }

    /// 处理点击事件
    fn handle_click(&mut self, click_pos: egui::Pos2, rect: egui::Rect, world: &mut World) {
        let mut closest_entity = None;
        let mut closest_distance = f32::MAX;

        let mut query = world.query::<(Entity, &Transform)>();

        for (entity, transform) in query.iter(world) {
            let screen_pos = self.world_to_screen(transform.pos, rect);
            let distance = ((click_pos.x - screen_pos.x).powi(2)
                + (click_pos.y - screen_pos.y).powi(2))
            .sqrt();

            if distance < 10.0 * self.zoom && distance < closest_distance {
                closest_entity = Some(entity);
                closest_distance = distance;
            }
        }

        self.selected_entity = closest_entity;
    }

    /// 移动相机
    fn move_camera(&mut self, delta: egui::Vec2) {
        match self.view_mode {
            ViewMode::Perspective => {
                self.camera_position.x -= delta.x * 0.01 / self.zoom;
                self.camera_position.y += delta.y * 0.01 / self.zoom;
            }
            _ => {
                // 2D视图中移动相机实际上是移动视图偏移
                // 这里简化处理,直接调整相机位置
                self.camera_position.x -= delta.x * 0.01 / self.zoom;
                self.camera_position.y += delta.y * 0.01 / self.zoom;
            }
        }
    }
}

impl Default for SceneEditor {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::Perspective,
            camera_position: Vec3::new(0.0, 5.0, 10.0),
            camera_rotation: Vec3::new(-30.0, 0.0, 0.0),
            zoom: 1.0,
            selected_entity: None,
            grid_size: 1.0,
            show_grid: true,
            show_gizmos: true,
        }
    }
}
