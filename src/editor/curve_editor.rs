use glam::Vec2;

/// 贝塞尔曲线控制点
#[derive(Debug, Clone, Copy)]
pub struct BezierControlPoint {
    /// 时间
    pub time: f32,
    /// 值
    pub value: f32,
    /// 左切线
    pub left_tangent: Vec2,
    /// 右切线
    pub right_tangent: Vec2,
}

impl BezierControlPoint {
    pub fn new(time: f32, value: f32) -> Self {
        Self {
            time,
            value,
            left_tangent: Vec2::new(-0.1, 0.0),
            right_tangent: Vec2::new(0.1, 0.0),
        }
    }
}

/// 动画曲线
#[derive(Debug, Clone)]
pub struct AnimationCurve {
    /// 控制点列表
    pub control_points: Vec<BezierControlPoint>,
}

impl AnimationCurve {
    pub fn new() -> Self {
        Self {
            control_points: Vec::new(),
        }
    }
    
    /// 添加控制点
    pub fn add_control_point(&mut self, time: f32, value: f32) {
        let point = BezierControlPoint::new(time, value);
        
        // 按时间排序插入
        let index = self.control_points
            .binary_search_by(|p| p.time.partial_cmp(&time).unwrap())
            .unwrap_or_else(|i| i);
        
        self.control_points.insert(index, point);
    }
    
    /// 移除控制点
    pub fn remove_control_point(&mut self, index: usize) {
        if index < self.control_points.len() {
            self.control_points.remove(index);
        }
    }
    
    /// 评估曲线在指定时间的值
    pub fn evaluate(&self, time: f32) -> f32 {
        if self.control_points.is_empty() {
            return 0.0;
        }
        
        // 如果时间在第一个控制点之前
        if time <= self.control_points[0].time {
            return self.control_points[0].value;
        }
        
        // 如果时间在最后一个控制点之后
        if time >= self.control_points.last().unwrap().time {
            return self.control_points.last().unwrap().value;
        }
        
        // 查找相邻的两个控制点
        for i in 0..self.control_points.len() - 1 {
            let p0 = &self.control_points[i];
            let p1 = &self.control_points[i + 1];
            
            if time >= p0.time && time <= p1.time {
                // 使用三次贝塞尔插值
                let t = (time - p0.time) / (p1.time - p0.time);
                return self.cubic_bezier(
                    p0.value,
                    p0.value + p0.right_tangent.y,
                    p1.value + p1.left_tangent.y,
                    p1.value,
                    t,
                );
            }
        }
        
        0.0
    }
    
    /// 三次贝塞尔插值
    fn cubic_bezier(&self, p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;
        
        mt3 * p0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * p3
    }
}

impl Default for AnimationCurve {
    fn default() -> Self {
        Self::new()
    }
}

/// 曲线编辑器
pub struct CurveEditor {
    /// 当前编辑的曲线
    pub curve: AnimationCurve,
    /// 选中的控制点索引
    pub selected_point: Option<usize>,
    /// 视图缩放
    pub zoom: f32,
    /// 视图偏移
    pub offset: Vec2,
}

impl CurveEditor {
    pub fn new() -> Self {
        Self {
            curve: AnimationCurve::new(),
            selected_point: None,
            zoom: 1.0,
            offset: Vec2::ZERO,
        }
    }
    
    /// 渲染曲线编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Curve Editor");
        ui.separator();
        
        // 工具栏
        ui.horizontal(|ui| {
            if ui.button("Add Point").clicked() {
                self.curve.add_control_point(0.5, 0.5);
            }
            
            if ui.button("Remove Point").clicked() {
                if let Some(index) = self.selected_point {
                    self.curve.remove_control_point(index);
                    self.selected_point = None;
                }
            }
            
            ui.separator();
            
            ui.label("Zoom:");
            ui.add(egui::Slider::new(&mut self.zoom, 0.1..=5.0));
        });
        
        ui.separator();
        
        // 曲线视图
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(ui.available_width(), 300.0),
            egui::Sense::click_and_drag(),
        );
        
        let rect = response.rect;
        
        // 绘制背景
        painter.rect_filled(rect, 0.0, egui::Color32::from_gray(30));
        
        // 绘制网格
        let grid_spacing = 50.0 * self.zoom;
        let grid_color = egui::Color32::from_gray(50);
        
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
        
        // 绘制曲线
        if self.curve.control_points.len() >= 2 {
            let mut points = Vec::new();
            let steps = 100;
            
            let min_time = self.curve.control_points.first().unwrap().time;
            let max_time = self.curve.control_points.last().unwrap().time;
            
            for i in 0..=steps {
                let t = i as f32 / steps as f32;
                let time = min_time + (max_time - min_time) * t;
                let value = self.curve.evaluate(time);
                
                let x = rect.left() + (time * rect.width() * self.zoom) + self.offset.x;
                let y = rect.bottom() - (value * rect.height() * self.zoom) - self.offset.y;
                
                points.push(egui::pos2(x, y));
            }
            
            painter.add(egui::Shape::line(
                points,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 200, 255)),
            ));
        }
        
        // 绘制控制点
        for (i, point) in self.curve.control_points.iter().enumerate() {
            let x = rect.left() + (point.time * rect.width() * self.zoom) + self.offset.x;
            let y = rect.bottom() - (point.value * rect.height() * self.zoom) - self.offset.y;
            
            let is_selected = self.selected_point == Some(i);
            let color = if is_selected {
                egui::Color32::from_rgb(255, 200, 0)
            } else {
                egui::Color32::from_rgb(255, 255, 255)
            };
            
            painter.circle_filled(egui::pos2(x, y), 5.0, color);
            
            // 检测点击
            if response.clicked() {
                let click_pos = response.interact_pointer_pos().unwrap();
                let distance = ((click_pos.x - x).powi(2) + (click_pos.y - y).powi(2)).sqrt();
                
                if distance < 10.0 {
                    self.selected_point = Some(i);
                }
            }
        }
        
        ui.separator();
        
        // 控制点属性编辑
        if let Some(index) = self.selected_point {
            if let Some(point) = self.curve.control_points.get_mut(index) {
                ui.label(format!("Control Point {}", index));
                
                ui.horizontal(|ui| {
                    ui.label("Time:");
                    ui.add(egui::DragValue::new(&mut point.time).speed(0.01).clamp_range(0.0..=1.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Value:");
                    ui.add(egui::DragValue::new(&mut point.value).speed(0.01));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Left Tangent:");
                    ui.add(egui::DragValue::new(&mut point.left_tangent.x).prefix("X: ").speed(0.01));
                    ui.add(egui::DragValue::new(&mut point.left_tangent.y).prefix("Y: ").speed(0.01));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Right Tangent:");
                    ui.add(egui::DragValue::new(&mut point.right_tangent.x).prefix("X: ").speed(0.01));
                    ui.add(egui::DragValue::new(&mut point.right_tangent.y).prefix("Y: ").speed(0.01));
                });
            }
        }
    }
}

impl Default for CurveEditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_animation_curve() {
        let mut curve = AnimationCurve::new();
        
        curve.add_control_point(0.0, 0.0);
        curve.add_control_point(1.0, 1.0);
        
        // 测试插值
        let value = curve.evaluate(0.5);
        assert!((value - 0.5).abs() < 0.1); // 近似线性插值
    }
}
