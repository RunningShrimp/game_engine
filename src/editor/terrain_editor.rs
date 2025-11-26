use bevy_ecs::prelude::*;
use glam::Vec3;

/// 地形数据
#[derive(Clone, Debug)]
pub struct TerrainData {
    /// 地形宽度 (顶点数)
    pub width: usize,
    /// 地形高度 (顶点数)
    pub height: usize,
    /// 高度图数据
    pub heightmap: Vec<f32>,
    /// 地形缩放
    pub scale: Vec3,
}

impl TerrainData {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            heightmap: vec![0.0; width * height],
            scale: Vec3::new(1.0, 1.0, 1.0),
        }
    }
    
    /// 获取指定位置的高度
    pub fn get_height(&self, x: usize, y: usize) -> Option<f32> {
        if x >= self.width || y >= self.height {
            return None;
        }
        
        let index = y * self.width + x;
        self.heightmap.get(index).copied()
    }
    
    /// 设置指定位置的高度
    pub fn set_height(&mut self, x: usize, y: usize, height: f32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        
        let index = y * self.width + x;
        if let Some(h) = self.heightmap.get_mut(index) {
            *h = height;
            true
        } else {
            false
        }
    }
    
    /// 平滑地形
    pub fn smooth(&mut self, iterations: usize) {
        for _ in 0..iterations {
            let mut new_heightmap = self.heightmap.clone();
            
            for y in 1..self.height - 1 {
                for x in 1..self.width - 1 {
                    let sum = self.get_height(x - 1, y).unwrap_or(0.0)
                        + self.get_height(x + 1, y).unwrap_or(0.0)
                        + self.get_height(x, y - 1).unwrap_or(0.0)
                        + self.get_height(x, y + 1).unwrap_or(0.0)
                        + self.get_height(x, y).unwrap_or(0.0);
                    
                    let index = y * self.width + x;
                    new_heightmap[index] = sum / 5.0;
                }
            }
            
            self.heightmap = new_heightmap;
        }
    }
    
    /// 生成随机地形
    pub fn generate_random(&mut self, amplitude: f32) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        for height in &mut self.heightmap {
            *height = rng.gen::<f32>() * amplitude;
        }
    }
}

/// 地形编辑工具
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerrainTool {
    Raise,
    Lower,
    Flatten,
    Smooth,
    Paint,
}

impl TerrainTool {
    pub fn name(&self) -> &'static str {
        match self {
            TerrainTool::Raise => "Raise",
            TerrainTool::Lower => "Lower",
            TerrainTool::Flatten => "Flatten",
            TerrainTool::Smooth => "Smooth",
            TerrainTool::Paint => "Paint",
        }
    }
}

/// 地形编辑器
pub struct TerrainEditor {
    /// 地形数据
    pub terrain: TerrainData,
    /// 当前选择的工具
    pub current_tool: TerrainTool,
    /// 笔刷大小
    pub brush_size: f32,
    /// 笔刷强度
    pub brush_strength: f32,
    /// 目标高度 (用于Flatten工具)
    pub target_height: f32,
}

impl TerrainEditor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            terrain: TerrainData::new(width, height),
            current_tool: TerrainTool::Raise,
            brush_size: 5.0,
            brush_strength: 1.0,
            target_height: 0.0,
        }
    }
    
    /// 应用工具到指定位置
    pub fn apply_tool(&mut self, x: usize, y: usize) {
        let brush_radius = self.brush_size as usize;
        
        for dy in 0..=brush_radius * 2 {
            for dx in 0..=brush_radius * 2 {
                let tx = x.saturating_sub(brush_radius).saturating_add(dx);
                let ty = y.saturating_sub(brush_radius).saturating_add(dy);
                
                if tx >= self.terrain.width || ty >= self.terrain.height {
                    continue;
                }
                
                // 计算距离
                let dist_x = (tx as f32 - x as f32).abs();
                let dist_y = (ty as f32 - y as f32).abs();
                let dist = (dist_x * dist_x + dist_y * dist_y).sqrt();
                
                if dist > self.brush_size {
                    continue;
                }
                
                // 计算衰减
                let falloff = 1.0 - (dist / self.brush_size).min(1.0);
                let strength = self.brush_strength * falloff;
                
                // 应用工具
                if let Some(current_height) = self.terrain.get_height(tx, ty) {
                    let new_height = match self.current_tool {
                        TerrainTool::Raise => current_height + strength,
                        TerrainTool::Lower => current_height - strength,
                        TerrainTool::Flatten => {
                            current_height + (self.target_height - current_height) * strength
                        }
                        TerrainTool::Smooth => {
                            // 简化的平滑:取周围平均值
                            let mut sum = current_height;
                            let mut count = 1;
                            
                            if let Some(h) = self.terrain.get_height(tx.saturating_sub(1), ty) {
                                sum += h;
                                count += 1;
                            }
                            if let Some(h) = self.terrain.get_height(tx + 1, ty) {
                                sum += h;
                                count += 1;
                            }
                            if let Some(h) = self.terrain.get_height(tx, ty.saturating_sub(1)) {
                                sum += h;
                                count += 1;
                            }
                            if let Some(h) = self.terrain.get_height(tx, ty + 1) {
                                sum += h;
                                count += 1;
                            }
                            
                            let avg = sum / count as f32;
                            current_height + (avg - current_height) * strength
                        }
                        TerrainTool::Paint => current_height, // Paint工具不修改高度
                    };
                    
                    self.terrain.set_height(tx, ty, new_height);
                }
            }
        }
    }
    
    /// 渲染地形编辑器UI
    pub fn render(&mut self, ui: &mut egui::Ui) {
        ui.heading("Terrain Editor");
        ui.separator();
        
        // 地形信息
        ui.label(format!("Terrain Size: {}x{}", self.terrain.width, self.terrain.height));
        ui.separator();
        
        // 工具选择
        ui.horizontal(|ui| {
            ui.label("Tool:");
            ui.selectable_value(&mut self.current_tool, TerrainTool::Raise, "Raise");
            ui.selectable_value(&mut self.current_tool, TerrainTool::Lower, "Lower");
            ui.selectable_value(&mut self.current_tool, TerrainTool::Flatten, "Flatten");
            ui.selectable_value(&mut self.current_tool, TerrainTool::Smooth, "Smooth");
        });
        
        ui.separator();
        
        // 笔刷设置
        ui.horizontal(|ui| {
            ui.label("Brush Size:");
            ui.add(egui::Slider::new(&mut self.brush_size, 1.0..=20.0));
        });
        
        ui.horizontal(|ui| {
            ui.label("Brush Strength:");
            ui.add(egui::Slider::new(&mut self.brush_strength, 0.1..=5.0));
        });
        
        if self.current_tool == TerrainTool::Flatten {
            ui.horizontal(|ui| {
                ui.label("Target Height:");
                ui.add(egui::Slider::new(&mut self.target_height, -10.0..=10.0));
            });
        }
        
        ui.separator();
        
        // 地形操作
        ui.horizontal(|ui| {
            if ui.button("Generate Random").clicked() {
                self.terrain.generate_random(10.0);
            }
            
            if ui.button("Smooth All").clicked() {
                self.terrain.smooth(3);
            }
            
            if ui.button("Clear").clicked() {
                for height in &mut self.terrain.heightmap {
                    *height = 0.0;
                }
            }
        });
        
        ui.separator();
        
        // 地形预览 (简化版)
        ui.label("Terrain Preview:");
        let preview_size = 200.0;
        let (response, painter) = ui.allocate_painter(
            egui::Vec2::new(preview_size, preview_size),
            egui::Sense::click(),
        );
        
        // 绘制简化的地形预览
        let rect = response.rect;
        let cell_width = rect.width() / self.terrain.width as f32;
        let cell_height = rect.height() / self.terrain.height as f32;
        
        for y in 0..self.terrain.height {
            for x in 0..self.terrain.width {
                if let Some(height) = self.terrain.get_height(x, y) {
                    let normalized_height = (height / 10.0).clamp(0.0, 1.0);
                    let gray = (normalized_height * 255.0) as u8;
                    let color = egui::Color32::from_gray(gray);
                    
                    let x_pos = rect.left() + x as f32 * cell_width;
                    let y_pos = rect.top() + y as f32 * cell_height;
                    
                    painter.rect_filled(
                        egui::Rect::from_min_size(
                            egui::Pos2::new(x_pos, y_pos),
                            egui::Vec2::new(cell_width, cell_height),
                        ),
                        0.0,
                        color,
                    );
                }
            }
        }
        
        // 处理点击
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let x = ((pos.x - rect.left()) / cell_width) as usize;
                let y = ((pos.y - rect.top()) / cell_height) as usize;
                self.apply_tool(x, y);
            }
        }
    }
}

impl Default for TerrainEditor {
    fn default() -> Self {
        Self::new(64, 64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_terrain_data() {
        let mut terrain = TerrainData::new(10, 10);
        
        // 设置高度
        assert!(terrain.set_height(5, 5, 10.0));
        assert_eq!(terrain.get_height(5, 5), Some(10.0));
        
        // 边界检查
        assert_eq!(terrain.get_height(100, 100), None);
        assert!(!terrain.set_height(100, 100, 5.0));
    }
    
    #[test]
    fn test_terrain_editor() {
        let mut editor = TerrainEditor::new(10, 10);
        
        // 应用工具
        editor.apply_tool(5, 5);
        
        // 验证高度已改变
        let height = editor.terrain.get_height(5, 5).unwrap();
        assert!(height > 0.0);
    }
}
